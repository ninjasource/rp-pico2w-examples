use core::str;

use embassy_executor::Spawner;
use embassy_rp::{peripherals::USB, rom_data::reboot, usb::Driver};
use embassy_usb_logger::ReceiverHandler;

pub(crate) const REBOOT_TYPE_BOOTSEL: u32 = 0x0002;

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver, Handler);
}

pub fn setup_logging(spawner: &Spawner, driver: Driver<'static, USB>) {
    spawner.spawn(logger_task(driver)).unwrap();
}

pub struct Handler;

impl ReceiverHandler for Handler {
    async fn handle_data(&self, data: &[u8]) {
        if let Ok(data) = str::from_utf8(data) {
            let data = data.trim();

            // If you are using elf2uf2-term with the '-t' flag, then when closing the serial monitor,
            // this will automatically put the pico into boot mode
            if data == "elf2uf2-term" {
                // see reboot section "5.4.8.24" of rp2350 datasheet
                reboot(REBOOT_TYPE_BOOTSEL, 100, 0, 0);
            }
        }
    }

    fn new() -> Self {
        Self
    }
}
