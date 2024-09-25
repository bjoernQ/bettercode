#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Io, Level, Output, Pull};
use esp_hal::prelude::*;
use esp_println::println;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let delay = Delay::new();

    let mut led1 = Output::new(io.pins.gpio4, Level::Low);
    let mut led2 = Output::new(io.pins.gpio5, Level::Low);

    let button = Input::new(io.pins.gpio9, Pull::Up);
    let mut prev_button_state = true;

    loop {
        println!("toggle LED1");
        led1.toggle();
        delay.delay(500.millis());

        if button.is_low() && prev_button_state {
            println!("toggle LED2");
            led2.toggle();
        }

        prev_button_state = button.get_level().into();
    }
}
