use embassy_time::Duration;
use esp_hal::{gpio::AnyPin, peripherals::RMT, rmt::Rmt, time::Rate};
use log::error;
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};

use crate::constants::*;
use crate::esp_hal_smartled::SmartLedsAdapter;

use super::{IndicatorReceiver, IndicatorStatus};

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

async fn wait_for_duration(
    duration: Duration,
    receiver: IndicatorReceiver,
) -> Result<(), IndicatorStatus> {
    match embassy_time::with_timeout(duration, receiver.receive()).await {
        Ok(s) => Err(s),
        Err(_) => Ok(()),
    }
}

async fn do_fade_in_out<const N: usize>(
    led: &mut SmartLedsAdapter<esp_hal::rmt::Channel<esp_hal::Blocking, 0>, N>,
    color: Hsv,
    receiver: IndicatorReceiver,
    min_brightness: u8,
    max_brightness: u8,
    step: usize,
) -> Result<(), IndicatorStatus> {
    led.write(brightness(
        gamma([hsv2rgb(color); SMART_LED_COUNT].into_iter()),
        min_brightness,
    ))
    .unwrap();
    let delta = (max_brightness - min_brightness) as usize;
    let step = if delta < step { 1usize } else { delta / step };

    loop {
        if (min_brightness == max_brightness) || (step == 0) {
            // No need to write the same value multiple times, just wait forever
            wait_for_duration(Duration::from_secs(86400), receiver).await?;
            continue;
        }

        for b in (min_brightness..=max_brightness).step_by(step) {
            led.write(brightness(
                gamma([hsv2rgb(color); SMART_LED_COUNT].into_iter()),
                b,
            ))
            .inspect_err(|e| {
                error!("Error writing to LED: {:?}", e);
            })
            .unwrap();
            wait_for_duration(Duration::from_millis(100), receiver).await?;
        }
        for b in (min_brightness..=max_brightness).step_by(step).rev() {
            led.write(brightness(
                gamma([hsv2rgb(color); SMART_LED_COUNT].into_iter()),
                b,
            ))
            .inspect_err(|e| {
                error!("Error writing to LED: {:?}", e);
            })
            .unwrap();
            wait_for_duration(Duration::from_millis(100), receiver).await?;
        }
    }
}

async fn fade_in_out<const N: usize>(
    led: &mut SmartLedsAdapter<esp_hal::rmt::Channel<esp_hal::Blocking, 0>, N>,
    color: Hsv,
    receiver: IndicatorReceiver,
    min_brightness: u8,
    max_brightness: u8,
    step: usize,
) -> IndicatorStatus {
    loop {
        if let Err(s) =
            do_fade_in_out(led, color, receiver, min_brightness, max_brightness, step).await
        {
            return s;
        }
    }
}

pub struct IndicatorConfig {
    pub rmt: RMT<'static>,
    pub pin: AnyPin<'static>,
    pub max_brightness: u8,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self {
            rmt: unsafe { RMT::steal() },
            pin: unsafe { esp_hal::gpio::AnyPin::steal(SMART_LED_PIN) },
            max_brightness: crate::AppConfig::get().brightness,
        }
    }
}

pub async fn start_indicator(config: IndicatorConfig, receiver: IndicatorReceiver) {
    let rmt = Rmt::new(config.rmt, Rate::from_mhz(80)).unwrap();
    let rmt_buffer = [0u32; SMART_LED_COUNT * 24 + 1];
    let mut led = SmartLedsAdapter::new(rmt.channel0, config.pin, rmt_buffer);

    let mut status = IndicatorStatus::WifiConnecting;

    loop {
        match status {
            IndicatorStatus::WifiConnecting => {
                status = fade_in_out(&mut led, RED, receiver, 0, config.max_brightness, 10).await;
            }
            IndicatorStatus::WifiConnected(_) => {
                status = fade_in_out(&mut led, BLUE, receiver, 0, config.max_brightness, 10).await;
            }
            IndicatorStatus::ServerConnecting => {
                status = fade_in_out(&mut led, BLUE, receiver, 0, config.max_brightness, 10).await;
            }
            IndicatorStatus::ServerConnected => {
                status =
                    fade_in_out(&mut led, YELLOW, receiver, 0, config.max_brightness, 10).await;
            }
            IndicatorStatus::Active => {
                status = fade_in_out(
                    &mut led,
                    GREEN,
                    receiver,
                    config.max_brightness,
                    config.max_brightness,
                    1,
                )
                .await;
            }
        }
    }
}
