use core::{cmp::min, str::FromStr};

use const_env::from_env;
use embedded_storage::ReadStorage;
use esp_storage::FlashStorage;
use heapless::String;
use log::{debug, warn};
use serde::Deserialize;

#[from_env]
pub const SSID: &str = "my-ssid";
#[from_env]
pub const PASSWORD: &str = "my-password";
#[from_env]
pub const SERVER: &str = "192.168.100.200:24800";
#[from_env]
pub const SCREEN_NAME: &str = "my-screen";
#[from_env]
pub const WIDTH: u16 = 1920;
#[from_env]
pub const HEIGHT: u16 = 1080;
#[from_env]
pub const FLIP_WHEEL: bool = false;
#[from_env]
pub const USB_VID: u16 = 0x0d0a;
#[from_env]
pub const USB_PID: u16 = 0xc0de;
#[from_env]
pub const USB_MANUFACTURER: &str = "0d0a.com";
#[from_env]
pub const USB_PRODUCT: &str = "Esparrier KVM";
#[from_env]
pub const USB_SERIAL_NUMBER: &str = "88888888";

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    // These fields must be set
    pub ssid: String<32>,
    pub password: String<64>,
    pub server: String<64>,
    pub screen_name: String<64>,

    // Screen configuration
    #[serde(default = "get_default_screen_width")]
    pub screen_width: u16,
    #[serde(default = "get_default_screen_height")]
    pub screen_height: u16,
    #[serde(default)]
    pub flip_wheel: bool,

    // USB HID configuration
    #[serde(default = "get_default_vid")]
    pub vid: u16,
    #[serde(default = "get_default_pid")]
    pub pid: u16,
    #[serde(default = "get_default_manufacturer")]
    pub manufacturer: String<64>,
    #[serde(default = "get_default_product")]
    pub product: String<64>,
    #[serde(default = "get_default_serial_number")]
    pub serial_number: String<64>,
}

// Kinda stupid
fn get_default_screen_width() -> u16 {
    WIDTH
}

fn get_default_screen_height() -> u16 {
    HEIGHT
}

fn get_default_vid() -> u16 {
    USB_VID
}

fn get_default_pid() -> u16 {
    USB_PID
}

fn get_default_manufacturer() -> String<64> {
    String::from_str(USB_MANUFACTURER).unwrap()
}

fn get_default_product() -> String<64> {
    String::from_str(USB_PRODUCT).unwrap()
}

fn get_default_serial_number() -> String<64> {
    String::from_str(USB_SERIAL_NUMBER).unwrap()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ssid: String::from_str(SSID).unwrap(),
            password: String::from_str(PASSWORD).unwrap(),
            server: String::from_str(SERVER).unwrap(),
            screen_name: String::from_str(SCREEN_NAME).unwrap(),
            screen_width: WIDTH,
            screen_height: HEIGHT,
            flip_wheel: FLIP_WHEEL,
            vid: USB_VID,
            pid: USB_PID,
            manufacturer: String::from_str(USB_MANUFACTURER).unwrap(),
            product: String::from_str(USB_PRODUCT).unwrap(),
            serial_number: String::from_str(USB_SERIAL_NUMBER).unwrap(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        // TODO: Use proper way to read the config
        let mut bytes = [0u8; 4096];
        let mut flash = FlashStorage::new();
        // Default NVS partition address
        // @see partition_single_app.csv
        let flash_addr = 0x9000;
        flash.read(flash_addr, &mut bytes).unwrap();
        // Find the valid JSON range
        let bytes = json_range(&bytes);
        serde_json_core::from_slice(bytes)
            .map(|(c, _)| c)
            .inspect_err(|e| {
                warn!("Failed to load config, using default, error: {:?}", e);
                debug!(
                    "Config content (first 16 bytes): {:?}",
                    &bytes[0..min(16, bytes.len())]
                );
            })
            .unwrap_or_default()
    }
}

fn is_valid_json_byte(b: u8) -> bool {
    // CR, LF, tab, space, visible ASCII characters, and UTF-8 sequences
    // Not a strict JSON check, but should be good enough for our use case
    b == 0x0D || b == 0x0A || b == 0x09 || (0x20..=0xF7).contains(&b)
}

fn json_range(buf: &[u8]) -> &[u8] {
    let start = buf.iter().position(|&b| b == b'{').unwrap_or(0);
    buf[start..]
        .iter()
        .position(|&b| !is_valid_json_byte(b))
        .map(|i: usize| &buf[start..i])
        .unwrap_or(&buf[start..])
}
