use cyw43::{Control, NetDriver};
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_rp::{gpio::Output, peripherals::PIO0};
use log::info;
use static_cell::StaticCell;

type Cyw43Spi = PioSpi<'static, PIO0, 0, embassy_rp::peripherals::DMA_CH0>;

#[embassy_executor::task]
async fn wifi_task(runner: cyw43::Runner<'static, Output<'static>, Cyw43Spi>) -> ! {
    runner.run().await
}

// by putting this in a known location every time the bootloader can detect it (via MD5 hashing) and not have to reflash this
#[link_section = ".modem_firmware"]
static MODEM_FIRMWARE: &[u8] = include_bytes!("../cyw43-firmware/43439A0.bin");

// country locale matrix (regulatory config)
#[link_section = ".modem_firmware"]
static COUNTRY_LOCALE_MATRIX: &[u8] = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

pub async fn setup_radio(
    spawner: &Spawner,
    pwr: Output<'static>,
    spi: Cyw43Spi,
) -> (NetDriver<'static>, Control<'static>) {
    // setup network buffers and init the modem
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, MODEM_FIRMWARE).await;

    // run the wifi runtime on an async task
    spawner.spawn(wifi_task(runner)).unwrap();

    // set the country locale matrix and power management
    // wifi_task MUST be running before this gets called
    control.init(COUNTRY_LOCALE_MATRIX).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    info!("wifi module setup complete");

    (net_device, control)
}
