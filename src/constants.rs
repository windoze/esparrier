use const_env::env_item;
use const_str::{parse, split};

const VERSION_SEGMENTS: [&str; 3] = split!(env!("CARGO_PKG_VERSION"), ".");
pub const VERSION_MAJOR: u8 = parse!(VERSION_SEGMENTS[0], u8);
pub const VERSION_MINOR: u8 = parse!(VERSION_SEGMENTS[1], u8);
pub const VERSION_PATCH: u8 = parse!(VERSION_SEGMENTS[2], u8);

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3-lite")] {
        pub const MODEL_ID: u8 = 1;
    } else if #[cfg(feature = "m5atoms3")] {
        pub const MODEL_ID: u8 = 2;
    } else if #[cfg(feature = "m5atoms3r")] {
        pub const MODEL_ID: u8 = 3;
    } else if #[cfg(feature = "devkitc-1_0")] {
        pub const MODEL_ID: u8 = 4;
    } else if #[cfg(feature = "devkitc-1_1")] {
        pub const MODEL_ID: u8 = 5;
    } else if #[cfg(feature = "xiao-esp32s3")] {
        pub const MODEL_ID: u8 = 6;
    } else if #[cfg(feature = "esp32-s3-eth")] {
        pub const MODEL_ID: u8 = 7;
    } else {
        #[env_item]
        pub const MODEL_ID: u8 = 255; // Generic ESP32 S3 Device
    }
}

const LED_INDICATOR_FLAG: u8 = if cfg!(feature = "led") {
    0b0000_0001
} else {
    0b0000_0000
};

const SMARTLED_INDICATOR_FLAG: u8 = if cfg!(feature = "smartled") {
    0b0000_0010
} else {
    0b0000_0000
};

const GRAPHICS_INDICATOR_FLAG: u8 = if cfg!(feature = "graphics") {
    0b0000_0100
} else {
    0b0000_0000
};

const CLIPBOARD_FLAG: u8 = if cfg!(feature = "graphics") {
    0b1000_0000
} else {
    0b0000_0000
};

pub const FEATURE_FLAGS: u8 =
    LED_INDICATOR_FLAG | SMARTLED_INDICATOR_FLAG | GRAPHICS_INDICATOR_FLAG | CLIPBOARD_FLAG;

cfg_if::cfg_if! {
    if #[cfg(feature = "xiao-esp32s3")] {
        pub const LED_PIN: u8 = 21;
    } else if #[cfg(feature = "led")] {
        #[env_item]
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
    } else if #[cfg(feature = "esp32-s3-eth")] {
        pub const SMART_LED_PIN: u8 = 21;
    } else if #[cfg(feature = "smartled")] {
        #[env_item]
        pub const SMART_LED_PIN: u8 = 0;
    }
}

#[cfg(feature = "smartled")]
#[env_item]
pub const SMART_LED_COUNT: usize = 1;

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "m5atoms3r")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "m5atoms3-lite")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "clipboard")] {
        #[env_item]
        pub const PASTE_BUTTON_PIN: u8 = 0;
    }
}

#[cfg(feature = "clipboard")]
#[env_item]
pub const MAX_CLIPBOARD_SIZE: usize = 1024;

// Default config settings
#[env_item]
pub const WIFI_SSID: &str = "my-ssid";
#[env_item]
pub const WIFI_PASSWORD: &str = "my-password";
#[env_item]
pub const BARRIER_SERVER: &str = "192.168.100.200:24800";
#[env_item]
pub const SCREEN_NAME: &str = "my-screen";
#[env_item]
pub const SCREEN_WIDTH: u16 = 1920;
#[env_item]
pub const SCREEN_HEIGHT: u16 = 1080;
#[env_item]
pub const JIGGLE_INTERVAL: u16 = 60;
#[env_item]
pub const POLLING_RATE: u16 = 200;
#[env_item]
pub const REVERSED_WHEEL: bool = false;

cfg_if::cfg_if! {
    if #[cfg(feature = "graphics")] {
        // 30 is too dim for LCD display
        #[env_item]
        pub const BRIGHTNESS: u8 = 50;
    } else {
        // But is good for SmartLED
        #[env_item]
        pub const BRIGHTNESS: u8 = 30;
    }
}

#[env_item]
pub const USB_VID: u16 = 0x0d0a;
#[env_item]
pub const USB_PID: u16 = 0xc0de;
#[env_item]
pub const USB_MANUFACTURER: &str = "0d0a.com";
#[env_item]
pub const USB_PRODUCT: &str = "Esparrier KVM";
#[env_item]
pub const USB_SERIAL_NUMBER: &str = "88888888";

pub const DEVICE_INTERFACE_GUIDS: &[&str] = &["{4d36e96c-e325-11ce-bfc1-08002be10318}"];

#[cfg(feature = "ethernet")]
#[env_item]
pub const W5500_MISO_PIN: u8 = 12;

#[cfg(feature = "ethernet")]
#[env_item]
pub const W5500_MOSI_PIN: u8 = 11;

#[cfg(feature = "ethernet")]
#[env_item]
pub const W5500_SCK_PIN: u8 = 13;

#[cfg(feature = "ethernet")]
#[env_item]
pub const W5500_CS_PIN: u8 = 14;

#[cfg(feature = "ethernet")]
#[env_item]
pub const W5500_INT_PIN: u8 = 10;

#[cfg(feature = "ethernet")]
#[env_item]
pub const W5500_RESET_PIN: u8 = 9;
