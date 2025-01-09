use display_interface_spi::SPIInterface;
use embassy_time::{with_timeout, Duration};
use embedded_graphics::{
    draw_target::DrawTarget, image::ImageDrawable, pixelcolor::Rgb565, prelude::RgbColor,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    gpio::{AnyPin, GpioPin, Level, Output},
    ledc::{channel, timer, LSGlobalClkSource, Ledc, LowSpeed},
    peripherals::{LEDC, SPI3},
    prelude::*,
    spi::{master::Spi, AnySpi, SpiMode},
};
use mipidsi::{
    options::{ColorInversion, ColorOrder},
    Builder,
};
use tinygif::Gif;

use crate::IndicatorStatus;

use super::IndicatorReceiver;

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3")] {
        use mipidsi::models::ST7789;
        mod m5atom_s3;
        pub use m5atom_s3::IndicatorConfig;
        use m5atom_s3::*;
    }
    else {
        compile_error!("No graphical indicator for this board");
    }
}

const CONNECTING: &[u8] = include_bytes!("assets/connecting.gif");
const INACTIVE: &[u8] = include_bytes!("assets/inactive.gif");
const ACTIVE: &[u8] = include_bytes!("assets/active.gif");

pub async fn start_indicator(config: IndicatorConfig, receiver: IndicatorReceiver) {
    let mut display = init_display(config);
    display.clear(Rgb565::BLACK).unwrap();

    let connecting_gif: Gif<'_, Rgb565> = Gif::from_slice(CONNECTING).unwrap();
    let inactive_gif: Gif<'_, Rgb565> = Gif::from_slice(INACTIVE).unwrap();
    let active_gif: Gif<'_, Rgb565> = Gif::from_slice(ACTIVE).unwrap();

    let mut status = IndicatorStatus::WifiConnecting;

    loop {
        let gif = match status {
            IndicatorStatus::WifiConnecting => &connecting_gif,
            IndicatorStatus::WifiConnected(_) => &connecting_gif,
            IndicatorStatus::ServerConnecting => &connecting_gif,
            IndicatorStatus::ServerConnected => &inactive_gif,
            IndicatorStatus::Active => &active_gif,
        };
        if status == IndicatorStatus::Active {
            // Don't waste time on animation, just show the first frame and wait for the next status forever
            // The SPI bus is pretty slow, showing animation in the active state may cause jitter and lag when receiving data and sending HID reports.
            gif.frames().next().unwrap().draw(&mut display).unwrap();
            status = receiver.receive().await;
        } else {
            // Show the animation and wait for the next status, we can afford it because there is no user interaction in the connecting and inactive states.
            for frame in gif.frames() {
                frame.draw(&mut display).unwrap();
                if let Ok(s) = with_timeout(
                    Duration::from_millis(frame.delay_centis as u64),
                    receiver.receive(),
                )
                .await
                {
                    status = s;
                    break;
                }
            }
        }
    }
}
