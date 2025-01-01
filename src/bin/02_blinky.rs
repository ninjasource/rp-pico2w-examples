//! This example tests the RP Pico2 W on board LED and USB serial port logging.
//!
//! This example is for a RP Pico2 W or PR Pico2 WH. It does not work with the RP Pico2 board (non-wifi).
//!
//! How to run with a standard usb cable (no debug probe):
//! The pico has a builtin bootloader that can be used as a replacement for a debug probe (like an ST link v2).
//! Start with the usb cable unplugged then, while holding down the BOOTSEL button, plug it in. Then you can release the button.
//! Mount the usb drive (this will be enumerated as USB mass storage) then run the following command:
//! cargo run --bin 02_blinky --release
//!
//! Why is it so complicated for a blinky? The led is physically connected to the wifi chip which is separate from the rp2040 mcu.
//! Therefore the wifi chip needs to be setup first and that is quite a procedure because we need to load its firmware and set the country locale martix.
//! We also need to setup the wifi task.
//! Other things that complicate this board are the fact that if you want to use the bootloader you beed to convert from elf to uf2 format using the elf2uf2-rs tool
//! The pico bootloader enumerates the USB device as a USB serial port is the button is not pressed on startup, otherwise as a USB mass storage device allowing you to copy firmware onto it.
//!
//! Troubleshoot:
//! `Error: "Unable to find mounted pico"`
//! This is because the pico is not in bootloader mode. You need to press down the BOOTSEL button when you plug it in and then release the button.
//! Then, if your're on linux, you need to mount the drive (click on it in your explorer and it should mount automatically). Or run a command to do it.
//! You need to do this every time you download firmware onto the device.

#![no_std]
#![no_main]

use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    block::ImageDef,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIO0, USB},
    pio::{self, Pio},
    usb::{self, Driver},
};
use embassy_time::{Duration, Timer};
use log::info;
use static_cell::StaticCell;

// NOTE: panic messages are not sent over usb serial because the pico has to be in a somewhat decent state to send data over USB
// However, when it panics it stops the scheduler. In order to see panic messages we really need to use a probe
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // modem firmware
    let fw = include_bytes!("../../cyw43-firmware/43439A0.bin");

    // country locale matrix (regulatory config)
    let clm = include_bytes!("../../cyw43-firmware/43439A0_clm.bin");

    // setup logging over usb serial port
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // wait for host to connect to usb serial port
    Timer::after(Duration::from_millis(500)).await;
    info!("started");

    // setup spi bus for wifi modem
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    // setup network buffers and init the modem
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    // run the cyw43 runtime (wifi radio) on an async task
    spawner.spawn(cyw43_task(runner)).unwrap();

    // set the country locale matrix and power management
    // cyw43_task MUST be running before this gets called
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    info!("wifi module setup complete");

    let delay = Duration::from_secs(1);
    loop {
        info!("led on!");
        control.gpio_set(0, true).await;
        Timer::after(delay).await;

        info!("led off!");
        control.gpio_set(0, false).await;
        Timer::after(delay).await;
    }
}
