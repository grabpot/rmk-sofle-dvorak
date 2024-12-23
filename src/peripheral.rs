#![no_main]
#![no_std]

#[macro_use]
mod macros;
mod uart;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{AnyPin, Input, Output},
    peripherals::USB,
    usb::InterruptHandler,
};
use panic_probe as _;
use rmk::split::{peripheral::run_rmk_split_peripheral,  SPLIT_MESSAGE_MAX_SIZE};
use uart::BufferedHalfDuplexUart;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("RMK start!");
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Pin config
    let (input_pins, output_pins) =
        config_matrix_pins_rp!(peripherals: p, input: [PIN_5, PIN_6, PIN_7, PIN_8, PIN_9], output: [PIN_27, PIN_26, PIN_22, PIN_20, PIN_23, PIN_21]);

    static TX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    static RX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    let uart_instance = BufferedHalfDuplexUart::new(p.PIO0, p.PIN_1, tx_buf, rx_buf);

    // Start serving
    run_rmk_split_peripheral::<Input<'_>, Output<'_>, _, 5, 6>(
        input_pins,
        output_pins,
        uart_instance,
    )
    .await;
}
