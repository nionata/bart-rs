#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, main};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let _peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    loop {
        esp_println::println!("bart-c3 alive");
        delay.delay_millis(1000);
    }
}
