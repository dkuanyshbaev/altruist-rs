use super::{Sensor, SensorReading, SensorError, SensorData, SensorType, SensorInfo, Quality};
use embassy_time::Duration;

/// SDS011 Particulate Matter sensor
/// Communicates via UART at 9600 baud
pub struct Sds011Sensor {
    initialized: bool,
    counter: u32, // Simulated counter for now
}

impl Sds011Sensor {
    /// Create new SDS011 sensor instance
    pub fn new() -> Self {
        Self {
            initialized: false,
            counter: 0,
        }
    }
}

impl Sensor for Sds011Sensor {
    async fn init(&mut self) -> Result<(), SensorError> {
        // TODO: Initialize UART for SDS011 communication
        // For now, just simulate initialization
        self.initialized = true;
        Ok(())
    }
    
    async fn read(&mut self) -> Result<SensorReading, SensorError> {
        if !self.initialized {
            return Err(SensorError::NotInitialized);
        }
        
        // TODO: Read actual UART data from SDS011 and validate checksum
        // For now, simulate reading
        self.counter += 1;
        
        // Simulate PM readings (in µg/m³)
        let (pm25, pm10) = match self.counter % 12 {
            0..=8 => {
                // Normal readings
                let base_pm25 = 5.0 + (self.counter as f32 * 0.7) % 15.0;
                let base_pm10 = base_pm25 * 1.3 + 2.0;
                (Some(base_pm25), Some(base_pm10))
            }
            9..=10 => {
                // Higher pollution readings
                (Some(35.0 + (self.counter as f32) % 25.0), Some(55.0 + (self.counter as f32) % 35.0))
            }
            _ => {
                // Occasional failed reading
                (None, None)
            }
        };
        
        let quality = match (pm25, pm10) {
            (Some(pm2), Some(pm1)) if pm2 < 25.0 && pm1 < 50.0 => Quality::Good,
            (Some(pm2), Some(pm1)) if pm2 < 75.0 && pm1 < 150.0 => Quality::Degraded,
            (Some(_), Some(_)) => Quality::Good, // High pollution but sensor working
            _ => Quality::Bad, // Failed reading
        };
        
        let data = SensorData::AirQuality { pm25, pm10 };
        
        Ok(SensorReading::new(SensorType::SDS011, data, quality))
    }
    
    fn info(&self) -> SensorInfo {
        SensorInfo {
            name: "SDS011",
            sensor_type: SensorType::SDS011,
            version: "1.0.0",
            manufacturer: "Nova Fitness",
        }
    }
    
    fn warm_up_time(&self) -> Duration {
        Duration::from_secs(15) // SDS011 needs 15 seconds warm-up
    }
    
    fn reading_interval(&self) -> Duration {
        Duration::from_secs(30) // Standard 30-second interval
    }
}