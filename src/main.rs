#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use embassy_time::Duration;

#[esp_hal::entry]
fn main() -> ! {
    println!("ESP32-C6: Embassy Hello World!");
    
    // Initialize system and take peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize Embassy timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    
    println!("Embassy firmware started successfully!");

    let mut counter = 0u32;
    
    loop {
        println!("Hello from Embassy! Count: {}", counter);
        counter = counter.wrapping_add(1);
        
        // Use Embassy timer for delay - non-blocking!
        embassy_time::block_for(Duration::from_secs(1));
    }
}