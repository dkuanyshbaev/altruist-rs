#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    gpio::{Io, Level, Output},
    timer::timg::TimerGroup,
};
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

    // Setup GPIO for LED (if available)
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio8, Level::Low);
    
    println!("Firmware started successfully!");
    println!("Embassy timers initialized!");

    // Simple Embassy-powered loop
    let mut main_counter = 0u32;
    let mut led_counter = 0u32;
    let mut fast_counter = 0u32;
    
    loop {
        // Main loop task (every 5 seconds)
        if main_counter % 50 == 0 {
            println!("[MAIN] Tick {}", main_counter / 50);
        }
        main_counter = main_counter.wrapping_add(1);
        
        // LED blink task (every 500ms)
        if led_counter % 5 == 0 {
            if (led_counter / 5) % 2 == 0 {
                led.set_high();
                println!("[LED] On");
            } else {
                led.set_low();
                println!("[LED] Off");
            }
        }
        led_counter = led_counter.wrapping_add(1);
        
        // Fast counter task (every 200ms)
        if fast_counter % 2 == 0 {
            println!("[COUNTER] Fast count: {}", fast_counter / 2);
        }
        fast_counter = fast_counter.wrapping_add(1);
        
        // Use Embassy timer for delay - non-blocking!
        embassy_time::block_for(Duration::from_millis(100));
    }
}