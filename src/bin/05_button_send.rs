//! This example sends incomming udp packets to an endpoint depending on the state of input pin GP15. See button example first.
//! In order to connect to the wifi network please create the following two files in the `src` folder:
//! WIFI_SSID.txt and WIFI_PASSWORD.txt
//! The files above should contain the exact ssid and password to connect to the wifi network. No newline characters or quotes.
//!
//! This example is for a RP Pico2 W or PR Pico2 WH. It does not work with the RP Pico2 board (non-wifi).
//!
//! How to run with a standard usb cable (no debug probe):
//! The pico has a builtin bootloader that can be used as a replacement for a debug probe (like an ST link v2).
//! Start with the usb cable unplugged then, while holding down the BOOTSEL button, plug it in. Then you can release the button.
//! Mount the usb drive (this will be enumerated as USB mass storage) then run the following command:
//! cargo run --bin 05_button_send --release
//!
//! Troubleshoot:
//! `Error: "Unable to find mounted pico"`
//! This is because the pico is not in bootloader mode. You need to press down the BOOTSEL button when you plug it in and then release the button.
//! Then, if your're on linux, you need to mount the drive (click on it in your explorer and it should mount automatically). Or run a command to do it.
//! You need to do this every time you download firmware onto the device.

#![no_std]
#![no_main]

use cyw43::JoinOptions;
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use embassy_executor::Spawner;
use embassy_net::{
    udp::{PacketMetadata, UdpSocket},
    IpEndpoint, Ipv4Address, StackResources,
};
use embassy_rp::{
    bind_interrupts,
    block::ImageDef,
    clocks::RoscRng,
    gpio::{Input, Level, Output, Pull},
    peripherals::{DMA_CH0, PIO0, USB},
    pio::{self, Pio},
    usb::{self, Driver},
};
use embassy_time::{Duration, Timer};
use log::info;
use rand::RngCore;
use static_cell::StaticCell;
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

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    const REMOTE_IP: Ipv4Address = Ipv4Address::new(192, 168, 1, 50);
    const REMOTE_PORT: u16 = 47900;
    const LOCAL_IP: Ipv4Address = Ipv4Address::new(192, 168, 1, 49);
    const LOCAL_PORT: u16 = 47901;
    const ON: [u8; 1] = [1];
    const OFF: [u8; 1] = [0];

    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;

    // setup logging over usb serial port
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // wait for host to connect to usb serial port
    Timer::after(Duration::from_secs(1)).await;
    info!("started");

    // modem firmware
    let fw = include_bytes!("../../cyw43-firmware/43439A0.bin");

    // country locale matrix (regulatory config)
    let clm = include_bytes!("../../cyw43-firmware/43439A0_clm.bin");

    // setup spi bus for wifi modem
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    // setup network buffers and init the modem
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    // run the wifi runtime on an async task
    spawner.spawn(cyw43_task(runner)).unwrap();

    // set the country locale matrix and power management
    // wifi_task MUST be running before this gets called
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    info!("wifi module setup complete");

    //let config = embassy_net::Config::dhcpv4(Default::default());
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(LOCAL_IP, 24),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    spawner.spawn(net_task(runner)).unwrap();

    // this is GP15 (not the physical chip pin number!)
    let mut button = Input::new(p.PIN_15, Pull::Up);

    // make sure these files exist in your `src` folder
    let wifi_ssid: &str = include_str!("../WIFI_SSID.txt");
    let wifi_password: &str = include_str!("../WIFI_PASSWORD.txt");

    info!("connecting to wifi network '{}'", wifi_ssid);

    loop {
        let options = JoinOptions::new(wifi_password.as_bytes());
        match control.join(wifi_ssid, options).await {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }
    info!("connected to wifi network");

    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

    let mut rx_buffer = [0u8; 4096];
    let mut tx_buffer = [0u8; 4096];
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    let remote_endpoint = IpEndpoint::new(REMOTE_IP.into(), REMOTE_PORT);
    socket.bind(LOCAL_PORT).unwrap();

    loop {
        info!("waiting for button press");
        button.wait_for_low().await;

        info!("send led on!");
        socket.send_to(&ON, remote_endpoint).await.unwrap();
        control.gpio_set(0, true).await;

        // debounce the button
        Timer::after(Duration::from_millis(100)).await;

        info!("waiting for button release");
        button.wait_for_high().await;

        info!("send led off!");
        socket.send_to(&OFF, remote_endpoint).await.unwrap();
        control.gpio_set(0, false).await;

        // debounce the button
        Timer::after(Duration::from_millis(100)).await;
    }
}
