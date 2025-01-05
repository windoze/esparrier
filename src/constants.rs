use const_env::from_env;

// Pin 21 is for XIAO ESP32S3
#[cfg(feature = "led")]
#[from_env]
pub const LED_PIN: u8 = 21;

// Pin 35 is for M5Atom S3 Lite
#[cfg(feature = "smartled")]
#[from_env]
pub const SMART_LED_PIN: u8 = 35;

#[cfg(feature = "smartled")]
#[from_env]
pub const SMART_LED_COUNT: usize = 1;

// Pin 41 is for M5Atom S3 and M5Atom S3 Lite
#[cfg(feature = "clipboard")]
#[from_env]
pub const PASTE_BUTTON_PIN: u8 = 41;

#[cfg(feature = "clipboard")]
#[from_env]
pub const MAX_CLIPBOARD_SIZE: usize = 1024;

// Default config settings
#[from_env]
pub const WIFI_SSID: &str = "my-ssid";
#[from_env]
pub const WIFI_PASSWORD: &str = "my-password";
#[from_env]
pub const BARRIER_SERVER: &str = "192.168.100.200:24800";
#[from_env]
pub const SCREEN_NAME: &str = "my-screen";
#[from_env]
pub const SCREEN_WIDTH: u16 = 1920;
#[from_env]
pub const SCREEN_HEIGHT: u16 = 1080;
#[from_env]
pub const REVERSED_WHEEL: bool = false;
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
#[from_env]
pub const WATCHDOG_TIMEOUT: u32 = 15;
