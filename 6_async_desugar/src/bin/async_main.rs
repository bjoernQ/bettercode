#![no_std]
#![no_main]

extern crate alloc;

use core::{future::Future, pin::Pin};

use alloc::boxed::Box;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Io, Level, Output, Pull},
    prelude::*,
};
use esp_println::println;

#[main]
async fn main(spawner: Spawner) {
    esp_alloc::heap_allocator!(8192);

    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let led1 = Output::new(io.pins.gpio4, Level::Low);
    let led2 = Output::new(io.pins.gpio5, Level::Low);

    let button = Input::new(io.pins.gpio9, Pull::Up);

    spawner.must_spawn(blinky_task(led1));

    handle_gpio(button, led2).await;
}

#[embassy_executor::task]
async fn blinky_task(led: Output<'static>) {
    // blinky(led).await;

    Blinky {
        state: State::State0,
        led,
    }
    .await;
}

#[allow(unused)]
async fn blinky(mut led: Output<'static>) {
    loop {
        Timer::after(Duration::from_millis(500)).await;

        println!("toggle LED2");
        led.toggle();

    }
}

async fn handle_gpio(mut button: Input<'static>, mut led: Output<'static>) {
    loop {
        button.wait_for_rising_edge().await;

        println!("toggle LED1");
        led.toggle();
    }
}

// This is NOT what the compiler really does but should
// illustrate the idea.

struct Blinky {
    state: State,
    led: Output<'static>,
}

enum State {
    State0,
    State1(Box<dyn Future<Output = ()>>),
    State2,
}

impl Future for Blinky {
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        match &mut self.state {
            State::State0 => {
                // = "Timer::after(Duration::from_millis(500))"

                let wait = Timer::after(Duration::from_millis(500));
                let wait = Box::new(wait);
                self.state = State::State1(wait);
            }
            State::State1(ref mut wait) => {
                // = ".await;"

                let mut fut = unsafe { Pin::new_unchecked(&mut **wait) };
                match fut.as_mut().poll(cx) {
                    core::task::Poll::Ready(_) => self.state = State::State2,
                    _ => (),
                }
            }
            State::State2 => {
                println!("toggle LED2");
                self.led.toggle();

                self.state = State::State0
            }
        }

        cx.waker().wake_by_ref();
        core::task::Poll::Pending
    }
}
