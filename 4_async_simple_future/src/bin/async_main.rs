#![no_std]
#![no_main]

use core::future::Future;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal_embassy::main;
use esp_println::println;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    spawner.must_spawn(example());

    loop {
        Timer::after(Duration::from_millis(500)).await;
        println!("tick");
    }
}

#[embassy_executor::task]
async fn example() {
    loop {
        let res = MyFuture { count: 100_0000 }.await;
        println!("Result is {res}");
    }
}

struct MyFuture {
    count: usize,
}

impl Future for MyFuture {
    type Output = usize;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.count == 100_0000 {
            println!("Polled first time");
        }

        if self.count == 0 {
            println!("Ready");
        }

        if self.count > 0 {
            self.count -= 1;

            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        } else {
            core::task::Poll::Ready(42)
        }
    }
}
