#![no_std]
#![no_main]

use core::{
    future::Future,
    pin::Pin,
    ptr,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Io, Level, Output, Pull},
    prelude::*,
};
use esp_println::println;

// we are back to non-async main
#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let led1 = Output::new(io.pins.gpio4, Level::Low);
    let led2 = Output::new(io.pins.gpio5, Level::Low);

    let button = Input::new(io.pins.gpio9, Pull::Up);

    // run our own executor
    run(blinky(led1), handle_gpio(button, led2));
}

async fn handle_gpio(mut button: Input<'static>, mut led: Output<'static>) {
    loop {
        button.wait_for_rising_edge().await;

        println!("toggle LED2");
        led.toggle();
    }
}

async fn blinky(mut led: Output<'static>) {
    loop {
        println!("toggle LED1");
        led.toggle();

        Timer::after(Duration::from_millis(500)).await;
    }
}

// A polling executor - always executing two Futures, never expected to return
pub fn run<F1: Future, F2: Future>(mut fut1: F1, mut fut2: F2) -> ! {
    // A virtual function pointer table that specifies the behavior of a [`RawWaker`].
    //
    // The pointer passed to all functions inside the vtable is the `data` pointer
    // from the enclosing [`RawWaker`] object.
    //
    // The functions inside this struct are only intended to be called on the `data`
    // pointer of a properly constructed [`RawWaker`] object from inside the
    // [`RawWaker`] implementation. Calling one of the contained functions using
    // any other `data` pointer will cause undefined behavior.    
    static VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(ptr::null(), &VTABLE),
        |_| {},
        |_| {},
        |_| {},
    );

    // safety: we don't move the future after this line.
    let mut fut1 = unsafe { Pin::new_unchecked(&mut fut1) };
    let mut fut2 = unsafe { Pin::new_unchecked(&mut fut2) };

    let raw_waker = RawWaker::new(ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(_) = fut1.as_mut().poll(&mut cx) {
            panic!("Task returned");
        }

        if let Poll::Ready(_) = fut2.as_mut().poll(&mut cx) {
            panic!("Task returned");
        }
    }
}
