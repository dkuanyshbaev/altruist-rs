#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    println!("Altruist-RS starting up!");
    
    // Spawn our async hello world task
    spawner.spawn(hello_task()).unwrap();
}

#[embassy_executor::task]
async fn hello_task() {
    let mut counter = 0u32;
    
    loop {
        println!("Hello from Altruist-RS! Counter: {}", counter);
        counter += 1;
        
        // Async delay for 5 seconds
        Timer::after(Duration::from_secs(5)).await;
    }
}
