#![no_std]
#![no_main]

use core::{
    arch::asm,
    future::Future,
    pin::Pin,
    ptr::addr_of_mut,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Io, Level, Output, Pull},
    prelude::*,
};
use esp_println::println;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let led1 = Output::new(io.pins.gpio4, Level::Low);
    let led2 = Output::new(io.pins.gpio5, Level::Low);

    let button = Input::new(io.pins.gpio9, Pull::Up);

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

// Executor - always executing two Futures, never expected to return
pub fn run<F1: Future, F2: Future>(mut fut1: F1, mut fut2: F2) -> ! {
    static VTABLE: RawWakerVTable = RawWakerVTable::new(
        |cx| RawWaker::new(cx, &VTABLE),
        |cx| {
            let cx = cx as *mut bool;
            unsafe {
                *cx = true;
            }
        },
        |cx| {
            let cx = cx as *mut bool;
            unsafe {
                *cx = true;
            }
        },
        |_| {},
    );

    let mut task1_woken = true;
    let mut task2_woken = true;

    // safety: we don't move the future after this line.
    let mut fut1 = unsafe { Pin::new_unchecked(&mut fut1) };
    let mut fut2 = unsafe { Pin::new_unchecked(&mut fut2) };

    let raw_waker1 = RawWaker::new(addr_of_mut!(task1_woken).cast(), &VTABLE);
    let waker1 = unsafe { Waker::from_raw(raw_waker1) };
    let mut cx1 = Context::from_waker(&waker1);

    let raw_waker2 = RawWaker::new(addr_of_mut!(task2_woken).cast(), &VTABLE);
    let waker2 = unsafe { Waker::from_raw(raw_waker2) };
    let mut cx2 = Context::from_waker(&waker2);

    loop {
        if task1_woken {
            if let Poll::Ready(_) = fut1.as_mut().poll(&mut cx1) {
                panic!("Task returned");
            }
            task1_woken = false;
        }

        if task2_woken {
            if let Poll::Ready(_) = fut2.as_mut().poll(&mut cx2) {
                panic!("Task returned");
            }
            task2_woken = false;
        }

        unsafe {
            asm!("wfi");
        }
    }
}
