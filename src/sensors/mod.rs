pub mod sds011;
pub mod bme280;
pub mod me2co;
pub mod manager;

use embassy_time::Duration;

/// Main sensor trait that all sensors must implement
/// This trait is designed to be async-first and extensible
pub trait Sensor: Send {
    /// Initialize the sensor hardware
    /// Called once when the sensor is first started
    async fn init(&mut self) -> Result<(), SensorError>;
    
    /// Read sensor data asynchronously
    /// Should return quickly - use internal buffering if needed
    async fn read(&mut self) -> Result<SensorReading, SensorError>;
    
    /// Get static information about this sensor
    fn info(&self) -> SensorInfo;
    
    /// How long the sensor needs to warm up after power-on
    /// Default is no warm-up required
    fn warm_up_time(&self) -> Duration {
        Duration::from_secs(0)
    }
    
    /// Recommended interval between readings
    /// Default is 30 seconds (matches original firmware)
    fn reading_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
    
    /// Whether the sensor needs calibration
    /// Default is no calibration required
    fn needs_calibration(&self) -> bool {
        false
    }
}

/// Sensor reading with timestamp and quality information
/// This is the unified data format that flows through channels
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_type: SensorType,
    pub data: SensorData,
    pub timestamp: u64,     // milliseconds since boot
    pub quality: Quality,   // data quality indicator
}

/// All possible sensor data types
/// New sensor types can be added here without breaking existing code
#[derive(Debug, Clone)]
pub enum SensorData {
    /// Environmental sensors (BME280, BME680, etc.)
    Environmental {
        temperature: Option<f32>,  // Celsius
        humidity: Option<f32>,     // Percentage
        pressure: Option<f32>,     // hPa
        gas_resistance: Option<f32>, // BME680 only
    },
    
    /// Particulate matter sensors (SDS011, etc.)
    AirQuality {
        pm25: Option<f32>,  // µg/m³
        pm10: Option<f32>,  // µg/m³
    },
    
    /// Gas sensors (ME2-CO, SCD4x, etc.)
    Gas {
        co_ppm: Option<f32>,    // Carbon monoxide in ppm
        co2_ppm: Option<u16>,   // Carbon dioxide in ppm
        voc_index: Option<f32>, // Volatile organic compounds index
    },
    
    /// Radiation sensors (RadSens, etc.)
    Radiation {
        dose_rate: f32,     // µSv/h
        total_dose: Option<f32>, // Total accumulated dose
    },
    
    /// Noise sensors (I2S microphones, etc.)
    Noise {
        db_a: f32,          // A-weighted decibels
        db_c: Option<f32>,  // C-weighted decibels
        frequency_data: Option<[f32; 8]>, // Optional frequency bands
    },
    
    /// GPS sensors
    Location {
        latitude: f64,
        longitude: f64,
        altitude: Option<f32>,
        satellites: Option<u8>,
    },
    
    /// Generic analog sensors
    Analog {
        voltage: f32,
        raw_value: u16,
        converted_value: Option<f32>,
        units: &'static str,
    },
}

/// Data quality indicator
/// Helps downstream processing decide how to handle readings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    /// Sensor operating normally, data is reliable
    Good,
    /// Sensor has minor issues but data is still usable
    Degraded,
    /// Sensor failed or data is invalid
    Bad,
}

/// All supported sensor types
/// Add new types here when implementing new sensors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorType {
    // Environmental sensors
    BME280,
    BME680,
    SHT30,
    
    // Air quality sensors  
    SDS011,
    PMS7003,
    
    // Gas sensors
    ME2CO,   // Carbon monoxide
    SCD4X,   // CO2
    SGP30,   // VOC
    
    // Radiation sensors
    RadSens,
    
    // Noise sensors
    ICS43434, // I2S microphone
    
    // Location sensors
    GPS,
    
    // Generic
    AnalogSensor,
}

/// Static information about a sensor
#[derive(Debug, Clone)]
pub struct SensorInfo {
    pub name: &'static str,
    pub sensor_type: SensorType,
    pub version: &'static str,
    pub manufacturer: &'static str,
}

/// Unified error type for all sensors
/// Provides consistent error handling across different sensor types
#[derive(Debug, Clone)]
pub enum SensorError {
    /// Sensor hasn't been initialized yet
    NotInitialized,
    
    /// Communication error (I2C, UART, SPI, etc.)
    CommunicationError,
    
    /// Received invalid data (checksum failure, out of range, etc.)
    InvalidData,
    
    /// Operation timed out
    Timeout,
    
    /// Sensor needs calibration before use
    CalibrationRequired,
    
    /// Hardware failure detected
    HardwareFailure,
    
    /// Sensor is warming up
    WarmingUp,
    
    /// Configuration error
    ConfigError,
}

impl SensorReading {
    /// Create a new sensor reading with current timestamp
    pub fn new(sensor_type: SensorType, data: SensorData, quality: Quality) -> Self {
        Self {
            sensor_type,
            data,
            timestamp: Self::current_timestamp(),
            quality,
        }
    }
    
    /// Get current timestamp (milliseconds since boot)
    /// TODO: Replace with proper time source when available
    fn current_timestamp() -> u64 {
        // For now, use a simple counter
        // In future: embassy_time::Instant::now().as_millis() or similar
        0
    }
    
    /// Check if this reading is valid for processing
    pub fn is_valid(&self) -> bool {
        self.quality != Quality::Bad
    }
}

impl SensorType {
    /// Get human-readable name for this sensor type
    pub fn name(&self) -> &'static str {
        match self {
            SensorType::BME280 => "BME280",
            SensorType::BME680 => "BME680", 
            SensorType::SHT30 => "SHT30",
            SensorType::SDS011 => "SDS011",
            SensorType::PMS7003 => "PMS7003",
            SensorType::ME2CO => "ME2-CO",
            SensorType::SCD4X => "SCD4x",
            SensorType::SGP30 => "SGP30",
            SensorType::RadSens => "RadSens",
            SensorType::ICS43434 => "ICS43434",
            SensorType::GPS => "GPS",
            SensorType::AnalogSensor => "Analog",
        }
    }
    
    /// Get expected data type for this sensor
    pub fn expected_data_type(&self) -> &'static str {
        match self {
            SensorType::BME280 | SensorType::BME680 | SensorType::SHT30 => "Environmental",
            SensorType::SDS011 | SensorType::PMS7003 => "AirQuality",
            SensorType::ME2CO | SensorType::SCD4X | SensorType::SGP30 => "Gas",
            SensorType::RadSens => "Radiation",
            SensorType::ICS43434 => "Noise", 
            SensorType::GPS => "Location",
            SensorType::AnalogSensor => "Analog",
        }
    }
}

impl core::fmt::Display for SensorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SensorError::NotInitialized => write!(f, "Sensor not initialized"),
            SensorError::CommunicationError => write!(f, "Communication error"),
            SensorError::InvalidData => write!(f, "Invalid data received"),
            SensorError::Timeout => write!(f, "Operation timed out"),
            SensorError::CalibrationRequired => write!(f, "Calibration required"),
            SensorError::HardwareFailure => write!(f, "Hardware failure"),
            SensorError::WarmingUp => write!(f, "Sensor warming up"),
            SensorError::ConfigError => write!(f, "Configuration error"),
        }
    }
}

impl core::fmt::Display for SensorType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sensor_reading_creation() {
        let reading = SensorReading::new(
            SensorType::BME280,
            SensorData::Environmental {
                temperature: Some(25.0),
                humidity: Some(60.0),
                pressure: Some(1013.25),
                gas_resistance: None,
            },
            Quality::Good
        );
        
        assert_eq!(reading.sensor_type, SensorType::BME280);
        assert!(reading.is_valid());
    }
    
    #[test]
    fn test_invalid_reading() {
        let reading = SensorReading::new(
            SensorType::SDS011,
            SensorData::AirQuality {
                pm25: None,
                pm10: None,
            },
            Quality::Bad
        );
        
        assert!(!reading.is_valid());
    }
}