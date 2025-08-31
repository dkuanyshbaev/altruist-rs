use super::{Sensor, SensorReading, SensorError, SensorData, SensorType, SensorInfo, Quality};
use embassy_time::{Duration, Timer, with_timeout};
use embedded_io_async::{Read, Write};
use esp_hal::uart::Uart;
use esp_hal::peripherals::UART0;

/// Type alias for the concrete UART type we use
pub type Sds011Uart = Uart<'static, UART0, esp_hal::Async>;

/// SDS011 Particulate Matter sensor
/// Uses async UART communication
/// Communicates on UART2: RX=GPIO5, TX=GPIO4 at 9600 baud
pub struct Sds011Sensor {
    uart: Sds011Uart,
    initialized: bool,
    is_running: bool,
    error_count: u32,
    last_error_time: Option<embassy_time::Instant>,
}

impl Sds011Sensor {
    /// Create new SDS011 sensor instance
    pub fn new(uart: Sds011Uart) -> Self {
        Self {
            uart,
            initialized: false,
            is_running: false,
            error_count: 0,
            last_error_time: None,
        }
    }
}

impl Sds011Sensor {
    /// Send raw command to SDS011
    async fn send_command(&mut self, cmd: &[u8]) -> Result<(), SensorError> {
        esp_println::println!("[SDS011] Sending command: {:02X?}", cmd);
        match with_timeout(Duration::from_millis(500), self.uart.write_all(cmd)).await {
            Ok(Ok(())) => {
                esp_println::println!("[SDS011] Command sent successfully");
                Ok(())
            },
            Ok(Err(_)) => {
                esp_println::println!("[SDS011] Command send failed");
                Err(SensorError::CommunicationError)
            },
            Err(_) => {
                esp_println::println!("[SDS011] Command send timeout");
                Err(SensorError::Timeout)
            },
        }
    }

    /// Start continuous mode and stop sensor
    async fn cmd_stop(&mut self) -> Result<(), SensorError> {
        let cmd: [u8; 19] = [0xAA, 0xB4, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x05, 0xAB];
        self.send_command(&cmd).await
    }

    /// Start sensor
    async fn cmd_start(&mut self) -> Result<(), SensorError> {
        let cmd: [u8; 19] = [0xAA, 0xB4, 0x06, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x06, 0xAB];
        self.send_command(&cmd).await
    }

    /// Set continuous mode - sends two separate commands like original firmware
    async fn cmd_continuous_mode(&mut self) -> Result<(), SensorError> {
        // Set working mode to continuous (0x08, 0x01, 0x00)
        let cmd1: [u8; 19] = [0xAA, 0xB4, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x07, 0xAB];
        self.send_command(&cmd1).await?;
        Timer::after(Duration::from_millis(100)).await;
        
        // Set reporting mode to continuous (0x02, 0x01, 0x00)  
        let cmd2: [u8; 19] = [0xAA, 0xB4, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x01, 0xAB];
        self.send_command(&cmd2).await
    }

    /// Validate SDS011 checksum
    fn checksum_valid(&self, data: &[u8; 8]) -> bool {
        let mut checksum: u8 = 0;
        for i in 0..6 {
            checksum = checksum.wrapping_add(data[i]);
        }
        data[7] == 0xAB && checksum == data[6]
    }

    /// Read measurement from SDS011
    async fn read_measurement(&mut self) -> Result<(f32, f32), SensorError> {
        const MAX_READ_SIZE: usize = 100;
        let mut buffer = [0u8; MAX_READ_SIZE];
        let mut read_pos = 0;
        
        // Clear any pending data first
        while let Ok(n) = with_timeout(Duration::from_millis(10), self.uart.read(&mut buffer)).await {
            if let Ok(0) | Err(_) = n {
                break;
            }
        }
        
        // SDS011 sends data continuously once per second when in continuous mode
        // We need to wait and read the stream to find a valid packet
        let timeout = Duration::from_secs(2);
        let start = embassy_time::Instant::now();
        
        while start.elapsed() < timeout {
            // Try to read one byte at a time to find the header
            let mut byte = [0u8; 1];
            match with_timeout(Duration::from_millis(100), self.uart.read(&mut byte)).await {
                Ok(Ok(1)) => {
                    buffer[read_pos] = byte[0];
                    
                    // Look for header sequence
                    if read_pos > 0 && buffer[read_pos - 1] == 0xAA && buffer[read_pos] == 0xC0 {
                        // Found measurement header, read remaining 8 bytes
                        let mut data = [0u8; 8];
                        let mut data_pos = 0;
                        
                        while data_pos < 8 {
                            match with_timeout(Duration::from_millis(100), self.uart.read(&mut data[data_pos..])).await {
                                Ok(Ok(n)) if n > 0 => {
                                    data_pos += n;
                                }
                                _ => break,
                            }
                        }
                        
                        if data_pos == 8 {
                            esp_println::println!("[SDS011] Got full packet: {:02X?}", data);
                            
                            if self.checksum_valid(&data) {
                                let pm25_raw = (data[0] as u16) | ((data[1] as u16) << 8);
                                let pm10_raw = (data[2] as u16) | ((data[3] as u16) << 8);
                                
                                let pm25 = pm25_raw as f32 / 10.0;
                                let pm10 = pm10_raw as f32 / 10.0;
                                
                                esp_println::println!("[SDS011] Valid measurement: PM2.5={} µg/m³, PM10={} µg/m³", pm25, pm10);
                                return Ok((pm25, pm10));
                            } else {
                                esp_println::println!("[SDS011] Checksum failed");
                            }
                        }
                    }
                    
                    read_pos = (read_pos + 1) % MAX_READ_SIZE;
                }
                Ok(Ok(0)) => {
                    // No data available, wait a bit
                    Timer::after(Duration::from_millis(10)).await;
                }
                _ => {
                    // Timeout or error, keep trying
                    Timer::after(Duration::from_millis(10)).await;
                }
            }
        }
        
        esp_println::println!("[SDS011] No valid data received within timeout");
        Err(SensorError::Timeout)
    }
}

impl Sensor for Sds011Sensor {
    async fn init(&mut self) -> Result<(), SensorError> {
        esp_println::println!("[SDS011] Initializing UART communication...");
        
        // Set continuous mode
        self.cmd_continuous_mode().await?;
        Timer::after(Duration::from_millis(100)).await;
        
        // Stop sensor initially
        self.cmd_stop().await?;
        self.is_running = false;
        
        self.initialized = true;
        esp_println::println!("[SDS011] Initialized successfully");
        Ok(())
    }
    
    async fn read(&mut self) -> Result<SensorReading, SensorError> {
        if !self.initialized {
            return Err(SensorError::NotInitialized);
        }
        
        // Check if we should back off due to errors
        if self.error_count >= 5 {
            if let Some(last_error) = self.last_error_time {
                let elapsed = last_error.elapsed();
                if elapsed < Duration::from_secs(60) {
                    esp_println::println!("[SDS011] Too many errors, backing off");
                    return Err(SensorError::Timeout);
                } else {
                    // Reset error count after backoff period
                    self.error_count = 0;
                    self.last_error_time = None;
                }
            }
        }
        
        // Start sensor if not running
        if !self.is_running {
            match self.cmd_start().await {
                Ok(_) => {
                    self.is_running = true;
                    esp_println::println!("[SDS011] Sensor started, waiting for warm-up...");
                    // SDS011 needs time to start sending data after being turned on
                    Timer::after(Duration::from_secs(3)).await;
                }
                Err(e) => {
                    self.error_count += 1;
                    self.last_error_time = Some(embassy_time::Instant::now());
                    esp_println::println!("[SDS011] Failed to start sensor");
                    return Err(e);
                }
            }
        }

        // Read measurement
        match self.read_measurement().await {
            Ok((pm25, pm10)) => {
                // Validate reasonable range
                if pm25 >= 0.0 && pm25 < 1000.0 && pm10 >= 0.0 && pm10 < 1000.0 {
                    self.error_count = 0; // Reset error count on success
                    let data = SensorData::AirQuality { 
                        pm25: Some(pm25), 
                        pm10: Some(pm10) 
                    };
                    return Ok(SensorReading::new(SensorType::SDS011, data, Quality::Good));
                } else {
                    esp_println::println!("[SDS011] Data out of range: PM2.5={}, PM10={}", pm25, pm10);
                    self.error_count += 1;
                    self.last_error_time = Some(embassy_time::Instant::now());
                    Err(SensorError::InvalidData)
                }
            }
            Err(e) => {
                self.error_count += 1;
                self.last_error_time = Some(embassy_time::Instant::now());
                esp_println::println!("[SDS011] Read error ({}): {:?}", self.error_count, e);
                Err(e)
            }
        }
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