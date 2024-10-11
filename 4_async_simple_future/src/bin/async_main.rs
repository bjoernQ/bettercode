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
        // async functions return `impl Future` - so we also can just `await` our own future
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
        // Pinning is an add-on contract that states the memory of the pointer cannot be moved. 
        // For Futures, this is a requirement because the future may contain separate references that also point to same memory.
        // If the memory were moved, those references would become invalid and cause UB.
        mut self: core::pin::Pin<&mut Self>,

        // The context of an asynchronous task.
        //
        // Currently, `Context` only serves to provide access to a [`&Waker`](Waker)
        // which can be used to wake the current task.
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

            // immediately wake
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        } else {
            core::task::Poll::Ready(42)
        }
    }
}
