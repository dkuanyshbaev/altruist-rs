use super::{Sensor, SensorReading, SensorError, SensorData, SensorType, SensorInfo, Quality};
use embassy_time::{Duration, Timer, with_timeout};
use esp_hal::i2c::I2c as EspI2c;
use esp_hal::peripherals::I2C0;

/// Type alias for the concrete I2C type we use
pub type Bme280I2c = EspI2c<'static, I2C0, esp_hal::Async>;

/// BME280 I2C addresses
const BME280_ADDRESS_PRIMARY: u8 = 0x76;
const BME280_ADDRESS_SECONDARY: u8 = 0x77;

/// BME280 chip ID
const BME280_CHIP_ID: u8 = 0x60;

/// BME280 register addresses
const BME280_REG_CHIP_ID: u8 = 0xD0;
const BME280_REG_RESET: u8 = 0xE0;
const BME280_REG_CTRL_HUM: u8 = 0xF2;
const BME280_REG_CTRL_MEAS: u8 = 0xF4;
const BME280_REG_CONFIG: u8 = 0xF5;
const BME280_REG_PRESS_MSB: u8 = 0xF7;
const BME280_REG_TEMP_MSB: u8 = 0xFA;
const BME280_REG_HUM_MSB: u8 = 0xFD;

/// Calibration register starts
const BME280_REG_DIG_T1: u8 = 0x88;
const BME280_REG_DIG_H1: u8 = 0xA1;
const BME280_REG_DIG_H2: u8 = 0xE1;

/// BME280 Environmental sensor (Temperature, Humidity, Pressure)
/// Communicates via I2C
pub struct Bme280Sensor {
    i2c: Bme280I2c,
    address: u8,
    initialized: bool,
    // Calibration coefficients
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
    dig_h1: u8,
    dig_h2: i16,
    dig_h3: u8,
    dig_h4: i16,
    dig_h5: i16,
    dig_h6: i8,
    // Temperature fine value for pressure and humidity compensation
    t_fine: i32,
}

impl Bme280Sensor {
    /// Create new BME280 sensor instance
    pub fn new(i2c: Bme280I2c) -> Self {
        Self {
            i2c,
            address: BME280_ADDRESS_PRIMARY, // Will try both addresses during init
            initialized: false,
            // Initialize calibration coefficients to zero
            dig_t1: 0, dig_t2: 0, dig_t3: 0,
            dig_p1: 0, dig_p2: 0, dig_p3: 0, dig_p4: 0, dig_p5: 0,
            dig_p6: 0, dig_p7: 0, dig_p8: 0, dig_p9: 0,
            dig_h1: 0, dig_h2: 0, dig_h3: 0, dig_h4: 0, dig_h5: 0, dig_h6: 0,
            t_fine: 0,
        }
    }

    /// Read a single byte from a register
    async fn read_register(&mut self, register: u8) -> Result<u8, SensorError> {
        let mut data = [0u8; 1];
        match with_timeout(Duration::from_millis(100), self.i2c.write_read(self.address, &[register], &mut data)).await {
            Ok(Ok(())) => Ok(data[0]),
            Ok(Err(_)) => Err(SensorError::CommunicationError),
            Err(_) => Err(SensorError::Timeout),
        }
    }

    /// Read multiple bytes from a register
    async fn read_registers(&mut self, register: u8, buffer: &mut [u8]) -> Result<(), SensorError> {
        match with_timeout(Duration::from_millis(100), self.i2c.write_read(self.address, &[register], buffer)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(SensorError::CommunicationError),
            Err(_) => Err(SensorError::Timeout),
        }
    }

    /// Write a single byte to a register
    async fn write_register(&mut self, register: u8, value: u8) -> Result<(), SensorError> {
        let data = [register, value];
        match with_timeout(Duration::from_millis(100), self.i2c.write(self.address, &data)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(SensorError::CommunicationError),
            Err(_) => Err(SensorError::Timeout),
        }
    }

    /// Scan I2C bus to see what devices are present
    async fn scan_i2c_bus(&mut self) -> Result<(), SensorError> {
        esp_println::println!("[BME280] Scanning I2C bus...");
        let mut found_devices = 0;
        
        for addr in 0x08..=0x77 {
            match with_timeout(Duration::from_millis(10), self.i2c.write(addr, &[])).await {
                Ok(Ok(())) => {
                    esp_println::println!("[BME280] Found I2C device at 0x{:02X}", addr);
                    found_devices += 1;
                }
                _ => {
                    // No device at this address, continue scanning
                }
            }
        }
        
        if found_devices == 0 {
            esp_println::println!("[BME280] No I2C devices found!");
        } else {
            esp_println::println!("[BME280] Found {} I2C devices total", found_devices);
        }
        
        Ok(())
    }

    /// Try to find the BME280 at both possible I2C addresses
    async fn find_sensor(&mut self) -> Result<(), SensorError> {
        // First scan the bus to see what's there
        self.scan_i2c_bus().await?;
        
        // Try primary address first
        self.address = BME280_ADDRESS_PRIMARY;
        esp_println::println!("[BME280] Trying address 0x{:02X}", self.address);
        if let Ok(chip_id) = self.read_register(BME280_REG_CHIP_ID).await {
            esp_println::println!("[BME280] Chip ID at 0x{:02X}: 0x{:02X} (expected: 0x{:02X})", self.address, chip_id, BME280_CHIP_ID);
            if chip_id == BME280_CHIP_ID {
                return Ok(());
            }
        } else {
            esp_println::println!("[BME280] No response at 0x{:02X}", self.address);
        }

        // Try secondary address
        self.address = BME280_ADDRESS_SECONDARY;
        esp_println::println!("[BME280] Trying address 0x{:02X}", self.address);
        if let Ok(chip_id) = self.read_register(BME280_REG_CHIP_ID).await {
            esp_println::println!("[BME280] Chip ID at 0x{:02X}: 0x{:02X} (expected: 0x{:02X})", self.address, chip_id, BME280_CHIP_ID);
            if chip_id == BME280_CHIP_ID {
                return Ok(());
            }
        } else {
            esp_println::println!("[BME280] No response at 0x{:02X}", self.address);
        }

        Err(SensorError::HardwareFailure)
    }

    /// Read calibration coefficients from the sensor
    async fn read_calibration(&mut self) -> Result<(), SensorError> {
        // Read temperature and pressure calibration data
        let mut buf = [0u8; 24];
        self.read_registers(BME280_REG_DIG_T1, &mut buf).await?;
        
        self.dig_t1 = u16::from_le_bytes([buf[0], buf[1]]);
        self.dig_t2 = i16::from_le_bytes([buf[2], buf[3]]);
        self.dig_t3 = i16::from_le_bytes([buf[4], buf[5]]);
        
        self.dig_p1 = u16::from_le_bytes([buf[6], buf[7]]);
        self.dig_p2 = i16::from_le_bytes([buf[8], buf[9]]);
        self.dig_p3 = i16::from_le_bytes([buf[10], buf[11]]);
        self.dig_p4 = i16::from_le_bytes([buf[12], buf[13]]);
        self.dig_p5 = i16::from_le_bytes([buf[14], buf[15]]);
        self.dig_p6 = i16::from_le_bytes([buf[16], buf[17]]);
        self.dig_p7 = i16::from_le_bytes([buf[18], buf[19]]);
        self.dig_p8 = i16::from_le_bytes([buf[20], buf[21]]);
        self.dig_p9 = i16::from_le_bytes([buf[22], buf[23]]);

        // Read humidity calibration data
        self.dig_h1 = self.read_register(BME280_REG_DIG_H1).await?;
        
        let mut h_buf = [0u8; 7];
        self.read_registers(BME280_REG_DIG_H2, &mut h_buf).await?;
        
        self.dig_h2 = i16::from_le_bytes([h_buf[0], h_buf[1]]);
        self.dig_h3 = h_buf[2];
        self.dig_h4 = ((h_buf[3] as i16) << 4) | ((h_buf[4] as i16) & 0x0F);
        self.dig_h5 = ((h_buf[5] as i16) << 4) | ((h_buf[4] as i16) >> 4);
        self.dig_h6 = h_buf[6] as i8;

        Ok(())
    }

    /// Configure sensor for forced mode measurements  
    async fn configure_sensor(&mut self) -> Result<(), SensorError> {
        // Set humidity oversampling (1x)
        self.write_register(BME280_REG_CTRL_HUM, 0x01).await?;
        
        // Set temperature and pressure oversampling (1x) and forced mode
        self.write_register(BME280_REG_CTRL_MEAS, 0x25).await?;
        
        // Set filter and standby time (filter off, standby 1000ms)
        self.write_register(BME280_REG_CONFIG, 0xA0).await?;
        
        Ok(())
    }

    /// Read raw sensor data and compensate using calibration
    async fn read_compensated_data(&mut self) -> Result<(f32, f32, f32), SensorError> {
        // Trigger forced mode measurement
        self.write_register(BME280_REG_CTRL_MEAS, 0x25).await?;
        
        // Wait for measurement to complete
        Timer::after(Duration::from_millis(50)).await;
        
        // Read all measurement data at once (8 bytes starting from pressure)
        let mut data = [0u8; 8];
        self.read_registers(BME280_REG_PRESS_MSB, &mut data).await?;
        
        // Extract raw values
        let press_raw = ((data[0] as u32) << 12) | ((data[1] as u32) << 4) | ((data[2] as u32) >> 4);
        let temp_raw = ((data[3] as u32) << 12) | ((data[4] as u32) << 4) | ((data[5] as u32) >> 4);
        let hum_raw = ((data[6] as u32) << 8) | (data[7] as u32);
        
        // Compensate temperature first (needed for pressure and humidity)
        let temperature = self.compensate_temperature(temp_raw);
        let pressure = self.compensate_pressure(press_raw);
        let humidity = self.compensate_humidity(hum_raw);
        
        Ok((temperature, humidity, pressure))
    }

    /// Temperature compensation formula from BME280 datasheet
    fn compensate_temperature(&mut self, adc_t: u32) -> f32 {
        let var1 = (((adc_t >> 3) as i32) - ((self.dig_t1 << 1) as i32)) * (self.dig_t2 as i32) >> 11;
        let var2 = (((((adc_t >> 4) as i32) - (self.dig_t1 as i32)) * (((adc_t >> 4) as i32) - (self.dig_t1 as i32))) >> 12) * (self.dig_t3 as i32) >> 14;
        
        self.t_fine = var1 + var2;
        ((self.t_fine * 5 + 128) >> 8) as f32 / 100.0
    }

    /// Pressure compensation formula from BME280 datasheet  
    fn compensate_pressure(&self, adc_p: u32) -> f32 {
        let mut var1: i64 = (self.t_fine as i64) - 128000;
        let mut var2: i64 = var1 * var1 * (self.dig_p6 as i64);
        var2 = var2 + ((var1 * (self.dig_p5 as i64)) << 17);
        var2 = var2 + ((self.dig_p4 as i64) << 35);
        var1 = ((var1 * var1 * (self.dig_p3 as i64)) >> 8) + ((var1 * (self.dig_p2 as i64)) << 12);
        var1 = ((((1i64) << 47) + var1) * (self.dig_p1 as i64)) >> 33;

        if var1 == 0 {
            return 0.0; // Avoid division by zero
        }

        let mut p: i64 = 1048576 - (adc_p as i64);
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = ((self.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        var2 = ((self.dig_p8 as i64) * p) >> 19;
        p = ((p + var1 + var2) >> 8) + ((self.dig_p7 as i64) << 4);

        (p as f32) / 25600.0
    }

    /// Humidity compensation formula from BME280 datasheet
    fn compensate_humidity(&self, adc_h: u32) -> f32 {
        let v_x1_u32r = self.t_fine - 76800;
        
        if v_x1_u32r == 0 {
            return 0.0;
        }

        // Step by step calculation for better readability
        let h_var1 = (adc_h as i32) - (((self.dig_h4 as i32) << 12) + ((self.dig_h5 as i32) * v_x1_u32r));
        
        let h_var2 = ((v_x1_u32r >> 15) * (v_x1_u32r >> 15)) >> 7;
        let h_var3 = (h_var2 * (self.dig_h1 as i32)) >> 4;
        let h_var4 = (h_var1 * (self.dig_h3 as i32)) >> 14;
        let h_var5 = (h_var2 * (self.dig_h6 as i32)) >> 4;
        let h_var6 = h_var4 * h_var5;
        
        let h_var7 = h_var1 * (h_var3 + h_var6 + 134217728) >> 10;
        let h_var8 = h_var7 * ((self.dig_h2 as i32) + 65536) >> 13;
        let h_var9 = h_var8 - (((((h_var8 >> 15) * (h_var8 >> 15)) >> 7) * 25) >> 9);
        
        let h_final = if h_var9 < 0 { 0 } else { h_var9 };
        let h_final = if h_final > 419430400 { 419430400 } else { h_final };

        (h_final >> 12) as f32 / 1024.0
    }
}

impl Sensor for Bme280Sensor {
    async fn init(&mut self) -> Result<(), SensorError> {
        esp_println::println!("[BME280] Initializing I2C communication...");
        
        // Find sensor at correct I2C address
        self.find_sensor().await?;
        esp_println::println!("[BME280] Found sensor at address 0x{:02X}", self.address);
        
        // Read calibration coefficients
        self.read_calibration().await?;
        esp_println::println!("[BME280] Calibration data loaded");
        
        // Configure sensor
        self.configure_sensor().await?;
        
        self.initialized = true;
        esp_println::println!("[BME280] Initialized successfully");
        Ok(())
    }
    
    async fn read(&mut self) -> Result<SensorReading, SensorError> {
        if !self.initialized {
            return Err(SensorError::NotInitialized);
        }
        
        // Read compensated sensor data
        match self.read_compensated_data().await {
            Ok((temperature, humidity, pressure)) => {
                // Validate reasonable ranges
                let temp_valid = temperature >= -40.0 && temperature <= 85.0;
                let hum_valid = humidity >= 0.0 && humidity <= 100.0;
                let press_valid = pressure >= 300.0 && pressure <= 1100.0;
                
                let quality = if temp_valid && hum_valid && press_valid {
                    Quality::Good
                } else {
                    Quality::Bad
                };
                
                let data = SensorData::Environmental {
                    temperature: if temp_valid { Some(temperature) } else { None },
                    humidity: if hum_valid { Some(humidity) } else { None },
                    pressure: if press_valid { Some(pressure) } else { None },
                    gas_resistance: None, // BME280 doesn't have gas sensor (BME680 does)
                };
                
                Ok(SensorReading::new(SensorType::BME280, data, quality))
            }
            Err(e) => Err(e),
        }
    }
    
    fn info(&self) -> SensorInfo {
        SensorInfo {
            name: "BME280",
            sensor_type: SensorType::BME280,
            version: "1.0.0",
            manufacturer: "Bosch",
        }
    }
    
    fn warm_up_time(&self) -> Duration {
        Duration::from_secs(2) // BME280 is ready quickly
    }
    
    fn reading_interval(&self) -> Duration {
        Duration::from_secs(30) // Standard interval
    }
}