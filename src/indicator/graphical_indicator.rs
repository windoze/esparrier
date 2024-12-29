use display_interface_spi::SPIInterface;
use embassy_time::Duration;
use embedded_graphics::{
    draw_target::DrawTarget, image::ImageDrawable, pixelcolor::Rgb565, prelude::RgbColor,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    gpio::{AnyPin, Level, Output},
    prelude::*,
    spi::{master::Spi, AnySpi, SpiMode},
};
use mipidsi::{models::ST7789, options::ColorInversion, Builder};
use tinygif::Gif;

use crate::IndicatorStatus;

use super::IndicatorReceiver;

pub struct GraphicalIndicatorConfig {
    pub width: u16,
    pub height: u16,
    pub spi: AnySpi,
    pub mosi: AnyPin,
    pub sck: AnyPin,
    pub dc: AnyPin,
    pub cs: AnyPin,
    pub rst: AnyPin,
    pub backlight: AnyPin,
}

type Display<'a> = mipidsi::Display<
    SPIInterface<
        ExclusiveDevice<Spi<'a, esp_hal::Blocking>, Output<'a>, embedded_hal_bus::spi::NoDelay>,
        Output<'a>,
    >,
    ST7789,
    Output<'a>,
>;

fn init_display<'a>(config: GraphicalIndicatorConfig) -> Display<'a> {
    // Turn on the backlight
    let _backlight = Output::new(config.backlight, Level::High);

    let mut delay = esp_hal::delay::Delay::new();
    let spi = Spi::new_with_config(
        config.spi,
        esp_hal::spi::master::Config {
            frequency: 40.MHz(),
            mode: SpiMode::Mode0,
            ..Default::default()
        },
    )
    .with_sck(config.sck)
    .with_mosi(config.mosi);

    let cs_output = Output::new(config.cs, Level::High);
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs_output).unwrap();

    let dc = Output::new(config.dc, Level::Low);
    let di = SPIInterface::new(spi_device, dc);
    let rst = Output::new(config.rst, Level::High);

    // Define the display from the display interface and initialize it
    Builder::new(ST7789, di)
        .reset_pin(rst)
        .display_size(128, 128)
        .invert_colors(ColorInversion::Inverted)
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .init(&mut delay)
        .unwrap()
}

const CONNECTING: &[u8] = include_bytes!("assets/connecting.gif");
const INACTIVE: &[u8] = include_bytes!("assets/inactive.gif");
const ACTIVE: &[u8] = include_bytes!("assets/active.gif");

pub async fn start_indicator(config: GraphicalIndicatorConfig, receiver: IndicatorReceiver) {
    let mut display = init_display(config);
    display.clear(Rgb565::BLACK).unwrap();

    let connecting_gif: Gif<'_, Rgb565> = Gif::from_slice(CONNECTING).unwrap();
    let inactive_gif: Gif<'_, Rgb565> = Gif::from_slice(INACTIVE).unwrap();
    let active_gif: Gif<'_, Rgb565> = Gif::from_slice(ACTIVE).unwrap();

    let mut status = IndicatorStatus::WifiConnecting;

    loop {
        let gif = match status {
            IndicatorStatus::WifiConnecting => &connecting_gif,
            IndicatorStatus::WifiConnected => &connecting_gif,
            IndicatorStatus::ServerConnected => &inactive_gif,
            IndicatorStatus::Active => &active_gif,
        };
        if status == IndicatorStatus::Active {
            // Don't waste time on animation, just show the first frame and wait for the next status forever
            gif.frames().next().unwrap().draw(&mut display).unwrap();
            if let Ok(s) =
                embassy_time::with_timeout(Duration::from_secs(86400 * 100), receiver.receive())
                    .await
            {
                status = s;
                continue;
            }
        } else {
            for frame in gif.frames() {
                frame.draw(&mut display).unwrap();
                if let Ok(s) = embassy_time::with_timeout(
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
