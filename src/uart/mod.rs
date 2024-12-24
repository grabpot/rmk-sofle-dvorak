use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::task::Poll;
use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use embassy_rp::{
    bind_interrupts,
    clocks::clk_sys_freq,
    gpio::{Drive, Level, Pull, SlewRate},
    interrupt::{
        typelevel::{Binding, Handler, Interrupt, PIO0_IRQ_0},
        Priority,
    },
    peripherals::PIO0,
    pio::{
        Common, Config, Direction, FifoJoin, Instance, InterruptHandler, Pin, Pio, PioPin,
        ShiftDirection, StateMachine,
    },
    uart::Error,
};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::{block_for, Duration, Timer};
use embedded_io_async::{ErrorType, Read, Write};
use fixed::traits::ToFixed;
use pio_proc;
use rp_pac::{io::vals::Oeover, PIO0};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => UartInterruptHandler<PIO0>;
});

pub struct IrqBinding;
unsafe impl Binding<PIO0_IRQ_0, InterruptHandler<PIO0>> for IrqBinding {}

const BAUD_RATE: u32 = 115_200;

pub struct UartBuffer {
    buf_tx: RingBuffer,
    buf_rx: RingBuffer,
    waker_rx: AtomicWaker,
}

impl UartBuffer {
    pub const fn new() -> Self {
        Self {
            buf_rx: RingBuffer::new(),
            buf_tx: RingBuffer::new(),
            waker_rx: AtomicWaker::new(),
        }
    }
}

pub trait UartPioAccess {
    fn uart_buffer() -> &'static UartBuffer;
    fn regs() -> &'static rp_pac::pio::Pio;
}

impl<T: Instance> UartPioAccess for T {
    fn uart_buffer() -> &'static UartBuffer {
        static BUFFER: UartBuffer = UartBuffer::new();
        &BUFFER
    }
    fn regs() -> &'static rp_pac::pio::Pio {
        &PIO0
    }
}

pub struct BufferedHalfDuplexUart<'a> {
    uart: HalfDuplexUart<'a, PIO0>,
}

impl<'a> BufferedHalfDuplexUart<'a> {
    pub fn new(pio: PIO0, pin: impl PioPin, tx_buf: &'a mut [u8], rx_buf: &'a mut [u8]) -> Self {
        Self {
            uart: HalfDuplexUart::new(pio, pin, tx_buf, rx_buf),
        }
    }
}

pub struct HalfDuplexUart<'a, PIO: Instance + UartPioAccess> {
    pin: Pin<'a, PIO0>,
    common: Common<'a, PIO0>,
    sm_tx: StateMachine<'a, PIO0, 0>,
    sm_rx: StateMachine<'a, PIO0, 1>,
    _pio: PhantomData<PIO>,
}

impl<'a, PIO: Instance + UartPioAccess> HalfDuplexUart<'a, PIO> {
    pub fn new(pio: PIO0, pin: impl PioPin, tx_buf: &mut [u8], rx_buf: &mut [u8]) -> Self {
        let Pio {
            mut common,
            sm0: sm_tx,
            sm1: sm_rx,
            ..
        } = Pio::new(pio, IrqBinding);

        let pio_pin = common.make_pio_pin(pin);

        let buffer = PIO::uart_buffer();
        unsafe { buffer.buf_rx.init(rx_buf.as_mut_ptr(), rx_buf.len()) };
        unsafe { buffer.buf_tx.init(tx_buf.as_mut_ptr(), tx_buf.len()) };

        PIO::Interrupt::disable();
        PIO::Interrupt::set_priority(Priority::P0);
        PIO::regs().irqs(0).inte().write(|m| {
            m.set_sm1(true);
            m.set_sm1_rxnempty(true);
        });
        PIO::Interrupt::unpend();
        unsafe { PIO::Interrupt::enable() };

        let mut uart = Self {
            pin: pio_pin,
            common,
            sm_tx,
            sm_rx,
            _pio: PhantomData,
        };

        uart.setup_pin();
        uart.setup_sm_tx();
        uart.setup_sm_rx();
        uart.enable_sm_rx();

        uart
    }

    fn setup_pin(&mut self) {
        rp_pac::IO_BANK0
            .gpio(self.pin.pin() as _)
            .ctrl()
            .modify(|f| f.set_oeover(Oeover::INVERT));
        self.pin.set_schmitt(true);
        self.pin.set_pull(Pull::Up);
        self.pin.set_slew_rate(SlewRate::Fast);
        self.pin.set_drive_strength(Drive::_12mA);
    }

    fn setup_sm_tx(&mut self) {
        let prg = pio_proc::pio_asm!(
            ".side_set 1 opt pindirs"
            ".wrap_target",
            "pull   block           side 1 [7]",
            "set    x, 7            side 0 [7]",
            "out    pindirs, 1",
            "jmp    x--, 2                 [6]"
            ".wrap",
        );

        let mut cfg = Config::default();
        cfg.use_program(&self.common.load_program(&prg.program), &[&self.pin]);
        cfg.set_out_pins(&[&self.pin]);
        let div = clk_sys_freq() / (BAUD_RATE as u32 * 8u32);
        cfg.clock_divider = div.to_fixed();
        cfg.shift_out.auto_fill = false;
        cfg.shift_out.direction = ShiftDirection::Right;
        cfg.shift_out.threshold = 32;
        cfg.fifo_join = FifoJoin::TxOnly;
        self.sm_tx.set_config(&cfg);
    }

    fn setup_sm_rx(&mut self) {
        let prg = pio_proc::pio_asm!(
            ".wrap_target",
            "wait   0 pin, 0",
            "set    x, 7                   [10]"
            "in     pins, 1",
            "jmp    x--, 2                 [6]",
            "jmp    pin, 8",
            "irq    wait 0",
            "wait   1 pin, 0",
            "jmp    0",
            "push   block",
            ".wrap",
        );

        let mut cfg = Config::default();
        cfg.use_program(&self.common.load_program(&prg.program), &[]);
        cfg.set_in_pins(&[&self.pin]);
        cfg.set_jmp_pin(&self.pin);
        let div = clk_sys_freq() / (BAUD_RATE as u32 * 8u32);
        cfg.clock_divider = div.to_fixed();
        cfg.shift_in.auto_fill = false;
        cfg.shift_in.direction = ShiftDirection::Right;
        cfg.shift_in.threshold = 32;
        cfg.fifo_join = FifoJoin::RxOnly;
        self.sm_rx.set_config(&cfg);

        // OEOVER set to INVERT, Direction::Out inverted to Direction:In
        self.sm_rx.set_pin_dirs(Direction::Out, &[&self.pin]);
        self.sm_tx.set_pins(Level::Low, &[&self.pin]);
    }

    fn enable_sm_tx(&mut self) {
        self.sm_rx.set_enable(false);
        self.sm_tx.restart();
        self.sm_tx.set_enable(true);
    }

    fn enable_sm_rx(&mut self) {
        while !self.sm_tx.tx().empty() {}
        block_for(Duration::from_micros(
            ((1_000_000u32 * 11) / BAUD_RATE) as u64,
        ));
        self.sm_tx.set_enable(false);
        self.sm_rx.set_enable(true);
    }

    fn read_buffer(&'a self, buf: &'a mut [u8]) -> impl Future<Output = Result<usize, Error>> + 'a {
        poll_fn(move |cx| {
            if let Poll::Ready(r) = self.try_read(buf) {
                return Poll::Ready(r);
            }
            PIO::uart_buffer().waker_rx.register(cx.waker());
            Poll::Pending
        })
    }

    fn try_read(&self, buf: &mut [u8]) -> Poll<Result<usize, Error>> {
        if buf.len() == 0 {
            return Poll::Ready(Ok(0));
        }
        self.read_ring(buf)
    }

    fn read_ring(&self, buf: &mut [u8]) -> Poll<Result<usize, Error>> {
        let mut reader = unsafe { PIO::uart_buffer().buf_rx.reader() };
        let data = reader.pop_slice();
        if data.len() == 0 {
            return Poll::Pending;
        };
        let n = data.len().min(buf.len());
        buf[..n].copy_from_slice(&data[..n]);
        reader.pop_done(n);
        Poll::Ready(Ok(n))
    }

    async fn write_buffer(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.write_ring(buf);
        if !self.sm_tx.is_enabled() {
            self.enable_sm_tx();
        }
        let result = self.write_fifo().await;
        self.enable_sm_rx();
        result
    }

    fn write_ring(&self, buf: &[u8]) -> () {
        let mut writer = unsafe { PIO::uart_buffer().buf_tx.writer() };
        for &byte in buf.iter() {
            writer.push_one(byte);
        }
    }

    async fn write_fifo(&mut self) -> Result<usize, Error> {
        let mut reader = unsafe { PIO::uart_buffer().buf_tx.reader() };
        let data = reader.pop_slice();
        let n = data.len();
        for &byte in data.iter() {
            self.sm_tx.tx().wait_push(byte as u32).await;
        }
        reader.pop_done(n);
        Ok(n)
    }

    async fn flush(&mut self) -> Result<(), Error> {
        if !self.sm_tx.tx().empty() {
            while !self.sm_tx.tx().empty() {}
            Timer::after(Duration::from_micros(
                ((1_000_000u32 * 11) / BAUD_RATE) as u64,
            ))
            .await;
        }
        Ok(())
    }
}

pub struct UartInterruptHandler<PIO: Instance + UartPioAccess> {
    _pio: PhantomData<PIO>,
}

impl<PIO: Instance + UartPioAccess> Handler<PIO::Interrupt> for UartInterruptHandler<PIO> {
    unsafe fn on_interrupt() {
        const SM1_RX_BIT: u32 = 1 << 1;
        let pio = PIO::regs();
        let irq = PIO::regs().irq().read().irq();
        let ints = PIO::regs().irqs(0).ints().read().0;
        if PIO::uart_buffer().buf_rx.is_available() {
            if ints & SM1_RX_BIT != 0 {
                let mut writer = unsafe { PIO::uart_buffer().buf_rx.writer() };
                let rx_buf = writer.push_slice();
                if rx_buf.len() > 0 {
                    let mut n = 0;
                    while (pio.fstat().read().rxempty() & SM1_RX_BIT as u8) == 0 && n < rx_buf.len()
                    {
                        let byte = pio.rxf(1).read();
                        rx_buf[n] = (byte >> 24) as u8;
                        n += 1;
                    }
                    writer.push_done(n);
                    PIO::Interrupt::unpend();
                    PIO::uart_buffer().waker_rx.wake();
                }
            } else if irq & (1 << 0) == 1 {
                // RX_SM Invalid Stop Bit Raised IRQ 0
                pio.irq().write(|f| f.set_irq(1 << 0));
                PIO::Interrupt::unpend();
            }
        }
    }
}

impl<'a> Read for BufferedHalfDuplexUart<'a> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.uart.read_buffer(buf).await
    }
}

impl<'a> Write for BufferedHalfDuplexUart<'a> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.uart.write_buffer(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.uart.flush().await
    }
}

impl<'a> ErrorType for BufferedHalfDuplexUart<'a> {
    type Error = Error;
}
