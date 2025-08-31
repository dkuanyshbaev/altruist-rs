use super::{Sensor, SensorReading, SensorError, SensorType};
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;
use heapless::Vec;

/// Global channel for sensor readings
/// All sensor tasks send their readings here
/// Buffer size of 32 should handle bursts from multiple sensors
pub static SENSOR_CHANNEL: Channel<CriticalSectionRawMutex, SensorReading, 32> = Channel::new();

/// Get sender for sensor readings
pub fn get_sensor_sender() -> Sender<'static, CriticalSectionRawMutex, SensorReading, 32> {
    SENSOR_CHANNEL.sender()
}

/// Get receiver for sensor readings  
pub fn get_sensor_receiver() -> Receiver<'static, CriticalSectionRawMutex, SensorReading, 32> {
    SENSOR_CHANNEL.receiver()
}

/// Registry entry for a sensor
pub struct SensorRegistry {
    pub sensor_type: SensorType,
    pub task_spawned: bool,
    pub last_reading_time: u64,
    pub error_count: u32,
}

/// Sensor manager handles sensor registration and coordination
/// Keeps track of all active sensors without storing the sensor objects
/// (since they're owned by their tasks)
pub struct SensorManager {
    registry: Vec<SensorRegistry, 16>, // Support up to 16 sensors
}

impl SensorManager {
    /// Create new sensor manager
    pub const fn new() -> Self {
        Self {
            registry: Vec::new(),
        }
    }
    
    /// Register a sensor type in the manager
    /// Call this before spawning the sensor task
    pub fn register_sensor(&mut self, sensor_type: SensorType) -> Result<(), SensorError> {
        // Check if sensor already registered
        if self.registry.iter().any(|s| s.sensor_type == sensor_type) {
            return Err(SensorError::ConfigError);
        }
        
        let entry = SensorRegistry {
            sensor_type,
            task_spawned: false,
            last_reading_time: 0,
            error_count: 0,
        };
        
        self.registry.push(entry).map_err(|_| SensorError::ConfigError)?;
        Ok(())
    }
    
    /// Mark a sensor task as spawned
    pub fn mark_task_spawned(&mut self, sensor_type: SensorType) {
        if let Some(entry) = self.registry.iter_mut().find(|s| s.sensor_type == sensor_type) {
            entry.task_spawned = true;
        }
    }
    
    /// Update sensor statistics when a reading is received
    pub fn update_sensor_stats(&mut self, sensor_type: SensorType, timestamp: u64, had_error: bool) {
        if let Some(entry) = self.registry.iter_mut().find(|s| s.sensor_type == sensor_type) {
            entry.last_reading_time = timestamp;
            if had_error {
                entry.error_count += 1;
            }
        }
    }
    
    /// Get list of registered sensors
    pub fn get_registered_sensors(&self) -> &[SensorRegistry] {
        &self.registry
    }
    
    /// Check if a sensor type is registered
    pub fn is_sensor_registered(&self, sensor_type: SensorType) -> bool {
        self.registry.iter().any(|s| s.sensor_type == sensor_type)
    }
    
    /// Get sensor statistics
    pub fn get_sensor_stats(&self, sensor_type: SensorType) -> Option<&SensorRegistry> {
        self.registry.iter().find(|s| s.sensor_type == sensor_type)
    }
}

/// Create sensor tasks dynamically for different sensor types
/// This works around Embassy's limitation of no generic tasks

/// ME2-CO sensor task
#[embassy_executor::task]
pub async fn me2co_sensor_task(mut sensor: super::me2co::Me2CoSensorWrapper) {
    sensor_task_impl(&mut sensor).await;
}

/// SDS011 sensor task  
#[embassy_executor::task]
pub async fn sds011_sensor_task(mut sensor: super::sds011::Sds011Sensor) {
    sensor_task_impl(&mut sensor).await;
}

/// BME280 sensor task
#[embassy_executor::task]
pub async fn bme280_sensor_task(mut sensor: super::bme280::Bme280Sensor) {
    sensor_task_impl(&mut sensor).await;
}

/// Generic sensor task implementation that can work with any sensor
/// This is the core logic shared by all sensor-specific tasks
async fn sensor_task_impl<S: Sensor>(sensor: &mut S) {
    let sensor_info = sensor.info();
    let sender = get_sensor_sender();
    
    esp_println::println!("[{}] Starting sensor task", sensor_info.name);
    
    // Initialize sensor
    loop {
        match sensor.init().await {
            Ok(()) => {
                esp_println::println!("[{}] Initialized successfully", sensor_info.name);
                break;
            }
            Err(e) => {
                esp_println::println!("[{}] Init failed: {}, retrying in 5s", sensor_info.name, e);
                Timer::after(embassy_time::Duration::from_secs(5)).await;
            }
        }
    }
    
    // Wait for warm-up if needed
    let warm_up = sensor.warm_up_time();
    if warm_up.as_secs() > 0 {
        esp_println::println!("[{}] Warming up for {}s", sensor_info.name, warm_up.as_secs());
        Timer::after(warm_up).await;
    }
    
    // Main reading loop
    let interval = sensor.reading_interval();
    let mut consecutive_errors = 0u32;
    
    esp_println::println!("[{}] Starting readings every {}s", sensor_info.name, interval.as_secs());
    
    loop {
        match sensor.read().await {
            Ok(reading) => {
                // Reset error counter on successful read
                consecutive_errors = 0;
                
                // Send reading to aggregator
                match sender.try_send(reading) {
                    Ok(()) => {
                        // Success - reading sent
                    }
                    Err(_) => {
                        esp_println::println!("[{}] Channel full, dropping reading", sensor_info.name);
                    }
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                esp_println::println!("[{}] Read error ({}): {}", 
                    sensor_info.name, consecutive_errors, e);
                
                // If too many consecutive errors, increase delay
                if consecutive_errors > 3 {
                    esp_println::println!("[{}] Too many errors, backing off", sensor_info.name);
                    Timer::after(embassy_time::Duration::from_secs(60)).await;
                    continue;
                }
            }
        }
        
        Timer::after(interval).await;
    }
}

/// Sensor aggregator task that receives all sensor readings
/// This is where we can add data processing, filtering, etc.
#[embassy_executor::task]
pub async fn sensor_aggregator_task() {
    let receiver = get_sensor_receiver();
    
    esp_println::println!("[AGGREGATOR] Starting sensor data aggregator");
    
    loop {
        let reading = receiver.receive().await;
        
        // For now, just print the reading
        // Later this will do aggregation, filtering, forwarding to APIs, etc.
        match reading.data {
            super::SensorData::Environmental { temperature, humidity, pressure, .. } => {
                if let (Some(t), Some(h), Some(p)) = (temperature, humidity, pressure) {
                    esp_println::println!("[{}] T: {:.1}°C, H: {:.1}%, P: {:.1}hPa", 
                        reading.sensor_type.name(), t, h, p);
                }
            }
            super::SensorData::AirQuality { pm25, pm10 } => {
                if let (Some(pm2), Some(pm1)) = (pm25, pm10) {
                    esp_println::println!("[{}] PM2.5: {:.1} µg/m³, PM10: {:.1} µg/m³", 
                        reading.sensor_type.name(), pm2, pm1);
                }
            }
            super::SensorData::Gas { co_ppm, .. } => {
                if let Some(co) = co_ppm {
                    esp_println::println!("[{}] CO: {:.1} ppm", reading.sensor_type.name(), co);
                }
            }
            super::SensorData::Radiation { dose_rate, .. } => {
                esp_println::println!("[{}] Radiation: {:.3} µSv/h", 
                    reading.sensor_type.name(), dose_rate);
            }
            super::SensorData::Noise { db_a, .. } => {
                esp_println::println!("[{}] Noise: {:.1} dB(A)", 
                    reading.sensor_type.name(), db_a);
            }
            super::SensorData::Location { latitude, longitude, .. } => {
                esp_println::println!("[{}] Location: {:.6}, {:.6}", 
                    reading.sensor_type.name(), latitude, longitude);
            }
            super::SensorData::Analog { voltage, converted_value, units, .. } => {
                esp_println::println!("[{}] {:.3}V = {:?} {}", 
                    reading.sensor_type.name(), voltage, converted_value, units);
            }
        }
    }
}