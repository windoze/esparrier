use const_env::from_env;

cfg_if::cfg_if! {
    if #[cfg(feature = "xiao-esp32s3")] {
        pub const LED_PIN: u8 = 21;
    } else if #[cfg(feature = "led")] {
        #[from_env]
        pub const LED_PIN: u8 = 0;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3-lite")] {
        pub const SMART_LED_PIN: u8 = 35;
    } else if #[cfg(feature = "devkitc-1_0")] {
        pub const SMART_LED_PIN: u8 = 48;
    } else if #[cfg(feature = "devkitc-1_1")] {
        pub const SMART_LED_PIN: u8 = 38;
    } else if #[cfg(feature = "smartled")] {
        #[from_env]
        pub const SMART_LED_PIN: u8 = 0;
    }
}

#[cfg(feature = "smartled")]
#[from_env]
pub const SMART_LED_COUNT: usize = 1;

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "m5atoms3r")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "m5atoms3-lite")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "clipboard")] {
        #[from_env]
        pub const PASTE_BUTTON_PIN: u8 = 0;
    }
}

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

cfg_if::cfg_if! {
    if #[cfg(feature = "graphics")] {
        // 30 is too dim for LCD display
        #[from_env]
        pub const BRIGHTNESS: u8 = 50;
    } else {
        // But is good for SmartLED
        #[from_env]
        pub const BRIGHTNESS: u8 = 30;
    }
}

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
pub const DEVICE_INTERFACE_GUIDS: &[&str] = &["{4d36e96c-e325-11ce-bfc1-08002be10318}"];
