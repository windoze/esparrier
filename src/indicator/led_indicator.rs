use super::{IndicatorReceiver, IndicatorStatus};
use embassy_time::Duration;
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};

use crate::constants::LED_PIN;

struct LedConfig {
    on_duration: Duration,
    off_duration: Duration,
}

fn get_led_config(status: IndicatorStatus) -> LedConfig {
    match status {
        IndicatorStatus::WifiConnecting => LedConfig {
            on_duration: Duration::from_millis(100),
            off_duration: Duration::from_millis(100),
        },
        IndicatorStatus::WifiConnected(_) => LedConfig {
            on_duration: Duration::from_millis(100),
            off_duration: Duration::from_millis(100),
        },
        IndicatorStatus::ServerConnecting => LedConfig {
            on_duration: Duration::from_millis(100),
            off_duration: Duration::from_millis(100),
        },
        IndicatorStatus::ServerConnected => LedConfig {
            on_duration: Duration::from_millis(500),
            off_duration: Duration::from_millis(500),
        },
        IndicatorStatus::Active => LedConfig {
            on_duration: Duration::from_millis(1000),
            off_duration: Duration::from_millis(0),
        },
    }
}

pub struct IndicatorConfig {
    pub pin: AnyPin<'static>,
    pub high_on: bool,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self {
            pin: unsafe { esp_hal::gpio::AnyPin::steal(LED_PIN) },
            high_on: false,
        }
    }
}

pub async fn start_indicator(config: IndicatorConfig, receiver: IndicatorReceiver) {
    let mut led_pin = Output::new(config.pin, Level::Low, OutputConfig::default());
    let mut status = IndicatorStatus::WifiConnecting;

    loop {
        let led_config = get_led_config(status);
        let interval = led_config.on_duration;
        if interval.as_micros() > 0 {
            // XIAO ESP32S3 User LED turns on when the PIN 21 is set to **low**
            // @see https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/
            // Turn on the LED
            if config.high_on {
                led_pin.set_high();
            } else {
                led_pin.set_low();
            }
            if let Ok(s) = embassy_time::with_timeout(interval, receiver.receive()).await {
                status = s;
                continue;
            }
        }
        let interval = led_config.off_duration;
        if interval.as_micros() > 0 {
            // Turn off the LED
            if config.high_on {
                led_pin.set_low();
            } else {
                led_pin.set_high();
            }
            if let Ok(s) = embassy_time::with_timeout(interval, receiver.receive()).await {
                status = s;
            }
        }
    }
}
