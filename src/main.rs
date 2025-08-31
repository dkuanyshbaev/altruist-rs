//! Altruist - ESP32-C6 firmware
//!
//! Smart home system with extensible sensor framework using Embassy async.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::Uart;
use esp_hal::gpio::Io;
use esp_println::println;
use static_cell::StaticCell;

// Import our sensor abstraction
mod sensors;
use sensors::{
    bme280::Bme280Sensor,
    manager::{
        bme280_sensor_task, me2co_sensor_task, sds011_sensor_task, sensor_aggregator_task,
        SensorManager,
    },
    me2co::Me2CoSensorWrapper,
    sds011::Sds011Sensor,
};

#[esp_hal::entry]
fn main() -> ! {
    println!("Altruist");
    println!("==============================================");

    // Initialize system and take peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize Embassy timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Static storage for the executor and sensor manager
    static EXECUTOR: StaticCell<esp_hal_embassy::Executor> = StaticCell::new();
    static SENSOR_MANAGER: StaticCell<SensorManager> = StaticCell::new();

    let executor = EXECUTOR.init(esp_hal_embassy::Executor::new());
    let sensor_manager = SENSOR_MANAGER.init(SensorManager::new());

    println!("Initializing sensor framework...");
    
    // Configure async UARTs for sensors
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    
    // UART1 for ME2-CO sensor (pins RX=19, TX=18, 9600 baud)
    let uart1 = Uart::new_async_with_config(
        peripherals.UART1,
        esp_hal::uart::config::Config::default()
            .baudrate(9600),
        io.pins.gpio19,  // RX
        io.pins.gpio18,  // TX
    ).expect("Failed to create async UART1 with config");

    // UART0 for SDS011 sensor (pins RX=5, TX=4, 9600 baud)
    let uart0 = Uart::new_async_with_config(
        peripherals.UART0,
        esp_hal::uart::config::Config::default()
            .baudrate(9600),
        io.pins.gpio5,   // RX
        io.pins.gpio4,   // TX
    ).expect("Failed to create async UART0 with config");

    // Run the executor with our sensor tasks
    executor.run(|spawner| {
        println!("Spawning sensor aggregator task...");
        spawner.must_spawn(sensor_aggregator_task());

        println!("Spawning sensor tasks...");

        // Spawn ME2-CO sensor task with async UART  
        let me2co_sensor = Me2CoSensorWrapper::new(uart1);
        spawner.must_spawn(me2co_sensor_task(me2co_sensor));

        // Spawn SDS011 sensor task with async UART
        let sds_sensor = Sds011Sensor::new(uart0);
        spawner.must_spawn(sds011_sensor_task(sds_sensor));

        // Spawn BME280 sensor task
        let bme_sensor = Bme280Sensor::new();
        spawner.must_spawn(bme280_sensor_task(bme_sensor));

        println!("All sensor tasks started!");
        println!("Monitor sensor readings below:");
        println!("------------------------------");
    })
}
