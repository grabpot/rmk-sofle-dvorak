#![no_main]
#![no_std]

#[macro_use]
mod keymap;
#[macro_use]
mod macros;
mod uart;
mod vial;

use crate::keymap::{COL, NUM_LAYER, ROW};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::{
    bind_interrupts,
    flash::{Async, Flash},
    gpio::{AnyPin, Input, Output},
    peripherals::{self, USB},
    usb::{Driver, InterruptHandler},
};
use panic_probe as _;
use rmk::{
    config::{KeyboardUsbConfig, RmkConfig, VialConfig},
    split::{
        central::{run_peripheral_monitor, run_rmk_split_central},
        SPLIT_MESSAGE_MAX_SIZE,
    },
};
use uart::BufferedHalfDuplexUart;
use static_cell::StaticCell;
use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

const FLASH_SIZE: usize = 2 * 1024 * 1024;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("RMK start!");
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Create the usb driver, from the HAL
    let driver = Driver::new(p.USB, Irqs);

    // Pin config
    let (input_pins, output_pins) = config_matrix_pins_rp!(peripherals: p, input: [PIN_5, PIN_6, PIN_7, PIN_8, PIN_9], output: [PIN_27, PIN_26, PIN_22, PIN_20, PIN_23, PIN_21]);

    // Use internal flash to emulate eeprom
    // Both blocking and async flash are support, use different API
    // let flash = Flash::<_, Blocking, FLASH_SIZE>::new_blocking(p.FLASH);
    let flash = Flash::<_, Async, FLASH_SIZE>::new(p.FLASH, p.DMA_CH0);

    let keyboard_usb_config = KeyboardUsbConfig {
        vid: 0x4c4b,
        pid: 0x4643,
        manufacturer: "Haobo",
        product_name: "RMK Keyboard",
        serial_number: "vial:f64c2b3c:000001",
    };

    let vial_config = VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF);

    let keyboard_config = RmkConfig {
        usb_config: keyboard_usb_config,
        vial_config,
        ..Default::default()
    };

    static TX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    static RX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    let uart_receiver = BufferedHalfDuplexUart::new(p.PIO0, p.PIN_1, tx_buf, rx_buf);
    // Start serving
    join(
        run_rmk_split_central::<
            Input<'_>,
            Output<'_>,
            Driver<'_, USB>,
            Flash<peripherals::FLASH, Async, FLASH_SIZE>,
            ROW,
            COL,
            5,
            6,
            0,
            0,
            NUM_LAYER,
        >(
            input_pins,
            output_pins,
            driver,
            flash,
            &mut keymap::get_default_keymap(),
            keyboard_config,
            spawner,
        ),
        run_peripheral_monitor::<5, 6, 5, 0, _>(0, uart_receiver),
    )
    .await;
}
