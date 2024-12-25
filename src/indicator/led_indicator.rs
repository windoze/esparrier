use super::{IndicatorReceiver, IndicatorStatus};
use embassy_time::Duration;
use esp_hal::gpio::{AnyPin, Level, Output};

struct LedConfig {
    on_duration: Duration,
    off_duration: Duration,
}

impl LedConfig {
    fn get_interval(&self, is_on: bool) -> Duration {
        if is_on {
            self.on_duration
        } else {
            self.off_duration
        }
    }
}

fn get_led_config(status: IndicatorStatus) -> LedConfig {
    match status {
        IndicatorStatus::WifiConnecting => LedConfig {
            on_duration: Duration::from_millis(100),
            off_duration: Duration::from_millis(100),
        },
        IndicatorStatus::WifiConnected => LedConfig {
            on_duration: Duration::from_millis(100),
            off_duration: Duration::from_millis(100),
        },
        IndicatorStatus::ServerConnected => LedConfig {
            on_duration: Duration::from_millis(500),
            off_duration: Duration::from_millis(500),
        },
        IndicatorStatus::Active => LedConfig {
            on_duration: Duration::from_millis(999),
            off_duration: Duration::from_millis(1),
        },
    }
}

pub async fn start_indicator(pin: AnyPin, receiver: IndicatorReceiver) {
    let mut p = Output::new(pin, Level::Low);
    let mut status = IndicatorStatus::WifiConnecting;
    let mut is_on = false;

    loop {
        if is_on {
            p.set_high();
        } else {
            p.set_low();
        }
        let led_config = get_led_config(status);
        let interval = led_config.get_interval(is_on);
        is_on = !is_on;

        if let Ok(s) = embassy_time::with_timeout(interval, receiver.receive()).await {
            status = s;
        }
    }
}
