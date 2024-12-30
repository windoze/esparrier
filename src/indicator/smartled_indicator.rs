use embassy_time::Duration;
use esp_hal::{gpio::AnyPin, peripherals::RMT, prelude::*, rmt::Rmt};
use log::error;
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};

use crate::{esp_hal_smartled::SmartLedsAdapter, smartLedBuffer};

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
        gamma([hsv2rgb(color)].into_iter()),
        min_brightness,
    ))
    .unwrap();

    loop {
        if (min_brightness == max_brightness) || (step == 0) {
            // No need to write the same value multiple times, just wait forever
            wait_for_duration(Duration::from_secs(86400), receiver).await?;
            continue;
        }

        for b in (min_brightness..=max_brightness).step_by(step) {
            led.write(brightness(gamma([hsv2rgb(color)].into_iter()), b))
                .inspect_err(|e| {
                    error!("Error writing to LED: {:?}", e);
                })
                .unwrap();
            wait_for_duration(Duration::from_millis(100), receiver).await?;
        }
        for b in (min_brightness..=max_brightness).step_by(step).rev() {
            led.write(brightness(gamma([hsv2rgb(color)].into_iter()), b))
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

pub async fn start_indicator(rmt: RMT, pin: AnyPin, receiver: IndicatorReceiver) {
    let rmt = Rmt::new(rmt, 80.MHz()).unwrap();
    let rmt_buffer = smartLedBuffer!(1);
    let mut led = SmartLedsAdapter::new(rmt.channel0, pin, rmt_buffer);

    let mut status = IndicatorStatus::WifiConnecting;

    loop {
        match status {
            IndicatorStatus::WifiConnecting => {
                status = fade_in_out(&mut led, RED, receiver, 0, 5, 1).await;
            }
            IndicatorStatus::WifiConnected => {
                status = fade_in_out(&mut led, BLUE, receiver, 0, 5, 1).await;
            }
            IndicatorStatus::ServerConnected => {
                status = fade_in_out(&mut led, YELLOW, receiver, 0, 5, 1).await;
            }
            IndicatorStatus::Active => {
                status = fade_in_out(&mut led, GREEN, receiver, 5, 5, 1).await;
            }
        }
    }
}
