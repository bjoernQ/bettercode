#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Io, Level, Output, Pull},
    prelude::*,
};

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let led1 = Output::new(io.pins.gpio4, Level::Low);
    let mut led2 = Output::new(io.pins.gpio5, Level::Low);

    let mut button = Input::new(io.pins.gpio9, Pull::Up);

    spawner.must_spawn(blinky(led1));

    loop {
        button.wait_for_rising_edge().await;
        led2.toggle();
    }
}

#[embassy_executor::task]
async fn blinky(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
