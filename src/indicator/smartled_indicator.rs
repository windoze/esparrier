use embassy_time::Duration;
use esp_hal::{gpio::AnyPin, peripherals::RMT, prelude::*, rmt::Rmt};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB8,
};

use crate::{smartLedBuffer, SmartLedsAdapter};

use super::{IndicatorReceiver, IndicatorStatus};

const BLACK: Hsv = Hsv {
    hue: 0,
    sat: 0,
    val: 0,
};

const RED: Hsv = Hsv {
    hue: 0,
    sat: 255,
    val: 255,
};

const BLUE: Hsv = Hsv {
    hue: 240,
    sat: 255,
    val: 255,
};

const YELLOW: Hsv = Hsv {
    hue: 60,
    sat: 255,
    val: 255,
};

const GREEN: Hsv = Hsv {
    hue: 120,
    sat: 255,
    val: 255,
};

struct LedConfig {
    on_duration: Duration,
    on_color: RGB8,
    off_duration: Duration,
    off_color: RGB8,
}

impl LedConfig {
    fn get_interval(&self, is_on: bool) -> Duration {
        if is_on {
            self.off_duration
        } else {
            self.on_duration
        }
    }

    fn get_color(&self, is_on: bool) -> RGB8 {
        if is_on {
            self.off_color
        } else {
            self.on_color
        }
    }
}

fn get_led_config(status: IndicatorStatus) -> LedConfig {
    match status {
        IndicatorStatus::WifiConnecting => LedConfig {
            on_duration: Duration::from_millis(100),
            on_color: hsv2rgb(RED),
            off_duration: Duration::from_millis(100),
            off_color: hsv2rgb(BLACK),
        },
        IndicatorStatus::WifiConnected => LedConfig {
            on_duration: Duration::from_millis(100),
            on_color: hsv2rgb(BLUE),
            off_duration: Duration::from_millis(100),
            off_color: hsv2rgb(BLACK),
        },
        IndicatorStatus::ServerConnected => LedConfig {
            on_duration: Duration::from_millis(500),
            on_color: hsv2rgb(YELLOW),
            off_duration: Duration::from_millis(500),
            off_color: hsv2rgb(BLACK),
        },
        IndicatorStatus::Active => LedConfig {
            on_duration: Duration::from_millis(500),
            on_color: hsv2rgb(GREEN),
            off_duration: Duration::from_millis(500),
            off_color: hsv2rgb(GREEN),
        },
    }
}

pub async fn start_indicator(rmt: RMT, pin: AnyPin, receiver: IndicatorReceiver) {
    let rmt = Rmt::new(rmt, 80.MHz()).unwrap();
    let rmt_buffer = smartLedBuffer!(1);
    let mut led = SmartLedsAdapter::new(rmt.channel0, pin, rmt_buffer);

    let mut status = IndicatorStatus::WifiConnecting;
    let mut is_on = false;

    loop {
        let led_config = get_led_config(status);
        let interval = led_config.get_interval(is_on);
        let color = [led_config.get_color(is_on)];
        is_on = !is_on;
        led.write(brightness(gamma(color.into_iter()), 10)).unwrap();

        if let Ok(s) = embassy_time::with_timeout(interval, receiver.receive()).await {
            status = s;
        }
    }
}
