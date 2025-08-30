fn main() {
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    
    // Add ESP-IDF app descriptor metadata
    println!("cargo:rustc-env=ESP_IDF_VERSION=v5.0");
    println!("cargo:rustc-env=IDF_TARGET=esp32c6");
}