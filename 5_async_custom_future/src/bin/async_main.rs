#![no_std]
#![no_main]

use core::{cell::RefCell, future::Future, task::Waker};

use critical_section::Mutex;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Event, Input, Io, Level, Output, Pull},
    prelude::*,
};
use esp_println::println;

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(gpio_interrupt_handler);

    let led1 = Output::new(io.pins.gpio4, Level::Low);
    let led2 = Output::new(io.pins.gpio5, Level::Low);

    let button = Input::new(io.pins.gpio9, Pull::Up);
    critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

    spawner.must_spawn(blinky(led1));

    handle_gpio(led2).await;
}

async fn handle_gpio(mut led: Output<'static>) {
    loop {
        WaitForButtonFuture::new().await;

        println!("toggle LED2");
        led.toggle();
    }
}

#[embassy_executor::task]
async fn blinky(mut led: Output<'static>) {
    loop {
        println!("toggle LED1");
        led.toggle();

        Timer::after(Duration::from_millis(500)).await;
    }
}

#[handler]
fn gpio_interrupt_handler() {
    critical_section::with(|cs| {
        BUTTON_WAKER
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .wake_by_ref();

        BUTTON.borrow_ref_mut(cs).as_mut().unwrap().unlisten()
    });
}

static BUTTON_WAKER: Mutex<RefCell<Option<Waker>>> = Mutex::new(RefCell::new(None));

struct WaitForButtonFuture {}

impl WaitForButtonFuture {
    fn new() -> Self {
        critical_section::with(|cs| {
            let mut button = BUTTON.borrow_ref_mut(cs);
            let button = button.as_mut().unwrap();

            button.clear_interrupt();
            button.listen(Event::FallingEdge);
        });
        Self {}
    }
}

impl Future for WaitForButtonFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let waker = cx.waker().clone();
        let triggered = critical_section::with(|cs| {
            BUTTON_WAKER.replace(cs, Some(waker));
            let mut button = BUTTON.borrow_ref_mut(cs);
            let button = button.as_mut().unwrap();

            if button.is_interrupt_set() {
                button.clear_interrupt();
                true
            } else {
                false
            }
        });

        if triggered {
            core::task::Poll::Ready(())
        } else {
            core::task::Poll::Pending
        }
    }
}
