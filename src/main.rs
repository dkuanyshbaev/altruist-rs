//! Altruist - ESP32-C6 firmware
//!
//! Smart home system with async task management using Embassy framework.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn me2_co_task() {
    let mut counter = 0u32;
    loop {
        println!("[ME2-CO]: {}", counter);
        counter = counter.wrapping_add(1);
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[embassy_executor::task]
async fn sds_task() {
    let mut counter = 0u32;
    loop {
        println!("[SDS]: {}", counter);
        counter = counter.wrapping_add(1);
        Timer::after(Duration::from_secs(3)).await;
    }
}

#[esp_hal::entry]
fn main() -> ! {
    println!("Altruist");

    // Initialize system and take peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize Embassy timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Static storage for the executor
    static EXECUTOR: StaticCell<esp_hal_embassy::Executor> = StaticCell::new();
    let executor = EXECUTOR.init(esp_hal_embassy::Executor::new());

    println!("Starting async tasks...");

    // Run the executor with spawned tasks
    executor.run(|spawner| {
        // Spawn ME2-CO sensor task
        spawner.must_spawn(me2_co_task());

        // Spawn SDS sensor task
        spawner.must_spawn(sds_task());

        println!("All tasks started!");
    })
}
