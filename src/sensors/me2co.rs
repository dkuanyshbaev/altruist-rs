use super::{Sensor, SensorReading, SensorError, SensorData, SensorType, SensorInfo, Quality};
use embassy_time::{Duration, Timer, with_timeout};
use embedded_io_async::{Read, Write};
use esp_hal::uart::Uart;
use esp_hal::peripherals::UART1;

/// Type alias for the concrete UART type we use
pub type Me2CoUart = Uart<'static, UART1, esp_hal::Async>;

/// ME2-CO Carbon Monoxide sensor
/// Uses async UART communication with ZE07-CO protocol
/// Communicates on UART1: RX=GPIO19, TX=GPIO18 at 9600 baud
pub struct Me2CoSensorWrapper {
    uart: Me2CoUart,
    initialized: bool,
}

impl Me2CoSensorWrapper {
    pub fn new(uart: Me2CoUart) -> Self {
        Self {
            uart,
            initialized: false,
        }
    }
}

impl Sensor for Me2CoSensorWrapper {
    async fn init(&mut self) -> Result<(), SensorError> {
        esp_println::println!("[ME2-CO] Initializing async UART communication...");
        
        // Send initialization command to set Q&A mode
        let init_cmd: [u8; 9] = [0xFF, 0x01, 0x78, 0x41, 0x00, 0x00, 0x00, 0x00, 0x46];
        
        // Use timeout to avoid blocking forever
        match with_timeout(Duration::from_millis(1000), self.uart.write_all(&init_cmd)).await {
            Ok(Ok(())) => {
                esp_println::println!("[ME2-CO] Initialization command sent");
                Timer::after(Duration::from_millis(100)).await;
                self.initialized = true;
                Ok(())
            }
            Ok(Err(_)) => {
                esp_println::println!("[ME2-CO] Failed to send initialization command");
                Err(SensorError::CommunicationError)
            }
            Err(_) => {
                esp_println::println!("[ME2-CO] Initialization timeout");
                Err(SensorError::Timeout)
            }
        }
    }
    
    async fn read(&mut self) -> Result<SensorReading, SensorError> {
        if !self.initialized {
            return Err(SensorError::NotInitialized);
        }
        
        // Send read command with timeout
        let read_cmd: [u8; 9] = [0xFF, 0x01, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79];
        
        match with_timeout(Duration::from_millis(500), self.uart.write_all(&read_cmd)).await {
            Ok(Ok(())) => {},
            Ok(Err(_)) => return Err(SensorError::CommunicationError),
            Err(_) => return Err(SensorError::Timeout),
        }
        
        // Wait for sensor to process command
        Timer::after(Duration::from_millis(50)).await;
        
        // Read response with timeout (try to read any available bytes, not exactly 9)
        let mut response = [0u8; 9];
        let mut bytes_read = 0;
        
        // Try reading for up to 1 second, collecting any available bytes
        let start_time = embassy_time::Instant::now();
        while bytes_read < 9 && start_time.elapsed() < Duration::from_millis(1000) {
            match self.uart.read(&mut response[bytes_read..]).await {
                Ok(0) => {
                    Timer::after(Duration::from_millis(10)).await;
                }
                Ok(n) => {
                    bytes_read += n;
                }
                Err(_) => break,
            }
        }
        
        if bytes_read >= 9 {
                
                // Validate response format
                if response[0] != 0xFF || response[1] != 0x86 {
                    return Err(SensorError::InvalidData);
                }
                
                // Extract CO reading from bytes 2-3 (high-low) - match original firmware
                let co_raw = ((response[2] as u16) << 8) | (response[3] as u16);
                let co_ppm = co_raw as f32 * 0.1; // Convert to ppm (original uses * 0.1f)
                
                // Validate checksum - match original firmware: (~sum) + 1
                let sum: u8 = response[1..8].iter().map(|&b| b).sum();
                let calculated_checksum = (!sum).wrapping_add(1);
                if calculated_checksum != response[8] {
                    return Err(SensorError::InvalidData);
                }
                
                // Validate CO reading range (0-1000 ppm is reasonable)
                if co_ppm < 0.0 || co_ppm > 1000.0 {
                    return Err(SensorError::InvalidData);
                }
                
                let data = SensorData::Gas {
                    co_ppm: Some(co_ppm),
                    co2_ppm: None,
                    voc_index: None,
                };
                
                Ok(SensorReading::new(SensorType::ME2CO, data, Quality::Good))
        } else {
            Err(SensorError::Timeout)
        }
    }
    
    fn info(&self) -> SensorInfo {
        SensorInfo {
            name: "ME2-CO",
            sensor_type: SensorType::ME2CO,
            version: "1.0.0",
            manufacturer: "Winsen Electronics",
        }
    }
    
    fn warm_up_time(&self) -> Duration {
        Duration::from_secs(10)
    }
    
    fn reading_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
}