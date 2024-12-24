# Sofle RP2040 RMK Firmware
An [RMK](https://github.com/HaoboGu/rmk) keyboard firmware for the [Sofle](https://josefadamcik.github.io/SofleKeyboard/) split keyboard and RP2040 Pro Micro controller with communication via half-duplex UART. A basic two-layer Dvorak keyboard layout is provided.

The Sofle keyboard was originally designed for two ATmega32U4-based SparkFun Pro Micro boards. This is replaced with the newer Raspberry Pi RP2040-based Pro Micro RP2040, or equivalent clone, e.g. from TENSTAR Robot on AliExpress, boards.

The keyboard halves were designed to communicate on Pin D2 (GPIO Pin 01 on RP2040) via "serial" using bit banging, the default on QMK, or I2C on Pins D1 and D0. A half-duplex, single wire UART serial driver, BufferedHalfDuplexUart, using the RP2040 PIO subsystem, is developed using the Embassy framework and implements the `embedded-io-async::Read` and `embedded-io-async::Write` traits compatible with RMK.

All credit for the Sofle keyboard goes to Josef Adamčík, and to HaoboGu for the RMK firmware and original RP2040 Split example.

## Build and Install

Follow the [instructions](https://rustup.rs/) to install Rust, and then install the required dependencies and RP2040 compilation target:
```
cargo install flip-link cargo-make
rustup target add thumbv6m-none-eabi
```

Compile the firmware and generate the uf2 files by running:
```
cargo build --release
cargo make uf2 --release
```

This will create `rmk-central.uf2` and `rmk-peripheral.uf2` in the root of the project. Plug in the left-hand RP2040 controller to USB, press and hold the Boot button while tapping Reset to enter the bootloader mode. If necessary, mount the USB storage device, and then copy the `rmk-central.uf2` file to the device. Repeat the process for the right-hand controller and `rmk-peripheral.uf2`.

For further information on using RMK, refer to the documentation: https://haobogu.github.io/rmk/index.html

## Development

Optionally, for development, two additional Raspberry Pi Pico, or equivalent clones, may be used instead of the left- or right-hand Pro Micro controller. The first should be flashed with the [Raspberry Pi Debug Probe](https://github.com/raspberrypi/debugprobe) firmware. The second can be flashed with RMK using [probe-rs](https://probe.rs/) from VS Code with the probe-rs debugger extension (see `launch.json` for configuration).

## Licence
This work is licensed under the MIT License.