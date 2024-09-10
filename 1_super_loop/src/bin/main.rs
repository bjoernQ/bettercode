#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Io, Level, Output};
use esp_hal::prelude::*;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let delay = Delay::new();

    let mut led = Output::new(io.pins.gpio4, Level::Low);
    loop {
        led.toggle();
        delay.delay(500.millis());
    }
}
