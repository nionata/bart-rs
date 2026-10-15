#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
    rmt::Rmt,
    time::Rate,
};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds, WS2812_TIMING};
use smart_leds::{SmartLedsWrite, RGB8};

esp_bootloader_esp_idf::esp_app_desc!();

const NUM_LEDS: usize = 2;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // Supermini's on-board blue LED is wired active-low on GPIO8.
    // Toggled every loop iteration so a frozen loop produces a frozen LED.
    let mut blue_led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());

    let rmt_freq = Rate::from_mhz(80);
    let rmt = Rmt::new(peripherals.RMT, rmt_freq).unwrap();

    let mut led =
        RmtSmartLeds::<{ buffer_size::<RGB8>(NUM_LEDS) }, _, RGB8, color_order::Grb>::new(
            WS2812_TIMING,
            rmt.channel0,
            peripherals.GPIO3,
            rmt_freq,
        )
        .unwrap();

    let colors = [
        RGB8 { r: 32, g: 0, b: 0 },
        RGB8 { r: 0, g: 32, b: 0 },
        RGB8 { r: 0, g: 0, b: 32 },
    ];
    let mut i = 0usize;

    loop {
        blue_led.toggle();

        let color = colors[i % colors.len()];
        let frame = [color; NUM_LEDS];
        led.write(frame.into_iter()).ok();
        esp_println::println!("bart-c3 color {}", i);
        i = i.wrapping_add(1);
        delay.delay_millis(500);
    }
}
