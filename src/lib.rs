#![no_std]

use embassy_rp::{block::ImageDef, rom_data::reboot};
use logging::REBOOT_TYPE_BOOTSEL;

pub mod logging;
pub mod network;
pub mod radio;

#[panic_handler]
fn core_panic(_info: &core::panic::PanicInfo) -> ! {
    reboot(REBOOT_TYPE_BOOTSEL, 100, 0, 0);
    loop {}
}

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();
