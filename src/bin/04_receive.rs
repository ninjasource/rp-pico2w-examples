//! This example receives incomming udp packets and turns an led on or off depending on the payload
//! In order to connect to the wifi network please create the following two files in the `src` folder:
//! WIFI_SSID.txt and WIFI_PASSWORD.txt
//! The files above should contain the exact ssid and password to connect to the wifi network. No newline characters or quotes.
//!
//! NOTE: This targets a RP Pico2 W or PR Pico2 WH. It does not work with the RP Pico2 board (non-wifi).
//!
//! How to run with a standard usb cable (no debug probe):
//! The pico has a builtin bootloader that can be used as a replacement for a debug probe (like an ST link v2).
//! Start with the usb cable unplugged then, while holding down the BOOTSEL button, plug it in. Then you can release the button.
//! Mount the usb drive (this will be enumerated as USB mass storage) then run the following command:
//! cargo run --bin 04_receive --release
//!
//! Troubleshoot:
//! `Error: "Unable to find mounted pico"`
//! This is because the pico is not in bootloader mode. You need to press down the BOOTSEL button when you plug it in and then release the button.
//! You need to do this every time you download firmware onto the device.

#![no_std]
#![no_main]

use core::str::{from_utf8, FromStr};

use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use embassy_executor::Spawner;
use embassy_net::Ipv4Address;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    peripherals::{PIO0, USB},
    pio::{self, Pio},
    usb::{self},
};
use embassy_time::{Duration, Timer};
use log::{error, info, warn};
use rp_pico2w_examples::{
    self as _, logging::setup_logging, network::setup_network, radio::setup_radio,
};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    const LOCAL_PORT: u16 = 47900;
    let local_ip = Ipv4Address::from_str(include_str!("../LOCAL_IP.txt")).ok();

    let p = embassy_rp::init(Default::default());

    // setup logging over usb serial port
    let driver = usb::Driver::new(p.USB, Irqs);
    setup_logging(&spawner, driver);

    // wait for host to connect to usb serial port
    Timer::after(Duration::from_millis(1000)).await;
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
    let (net_device, mut control) = setup_radio(&spawner, pwr, spi).await;

    let socket = setup_network(&spawner, net_device, &mut control, local_ip, LOCAL_PORT).await;
    info!("waiting for udp packets on port {LOCAL_PORT}");

    let mut buf: [u8; 1500] = [0; 1500];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, meta)) => match from_utf8(&buf[..len]) {
                Ok(s) => {
                    info!("received '{}' from {:?}", s, meta);
                    match s {
                        "on" => control.gpio_set(0, true).await,
                        "off" => control.gpio_set(0, false).await,
                        _ => warn!("unknown command received"),
                    }
                }
                Err(e) => warn!("received {} bytes from {:?}: {:?}", len, meta, e),
            },
            Err(e) => error!("error receiving packet: {:?}", e),
        }
    }
}
