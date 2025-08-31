#![no_std]
#![no_main]

use embassy_time::Duration;
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

#[esp_hal::entry]
fn main() -> ! {
    println!("Altruist");

    // Initialize system and take peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize Embassy timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let mut counter = 0u32;

    // Go!
    loop {
        println!("Count: {}", counter);
        counter = counter.wrapping_add(1);

        // Use Embassy timer for delay - non-blocking!
        embassy_time::block_for(Duration::from_secs(1));
    }
}
