use embedded_graphics::prelude::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    gpio::{AnyPin, GpioPin, Level, Output},
    ledc::{channel, timer, LSGlobalClkSource, Ledc, LowSpeed},
    peripherals::{LEDC, SPI3},
    prelude::*,
    spi::{master::Spi, AnySpi, SpiMode},
};
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, ColorOrder},
    Builder,
};

use crate::mk_static;

pub struct IndicatorConfig {
    pub width: u16,
    pub height: u16,
    pub offset_x: u16,
    pub offset_y: u16,
    pub spi: AnySpi,
    pub mosi: AnyPin,
    pub sck: AnyPin,
    pub dc: AnyPin,
    pub cs: AnyPin,
    pub rst: AnyPin,
    pub backlight: AnyPin,
    pub color_inversion: ColorInversion,
    pub color_order: ColorOrder,
    pub max_brightness: u8,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        // Hardware configuration is found at:
        // https://github.com/m5stack/M5GFX/blob/2c12f148d6e3df64ead33b04c7989fe6d90a7a83/src/M5GFX.cpp#L1501
        Self {
            width: 128,
            height: 128,
            // Field tested values, may vary on different devices
            offset_x: 2,
            offset_y: 1,
            spi: unsafe { SPI3::steal() }.into(),
            mosi: unsafe { GpioPin::<21>::steal() }.into(),
            sck: unsafe { GpioPin::<17>::steal() }.into(),
            dc: unsafe { GpioPin::<33>::steal() }.into(),
            cs: unsafe { GpioPin::<15>::steal() }.into(),
            rst: unsafe { GpioPin::<34>::steal() }.into(),
            backlight: unsafe { GpioPin::<16>::steal() }.into(),
            color_inversion: ColorInversion::Inverted,
            color_order: ColorOrder::Bgr,
            max_brightness: crate::AppConfig::get().brightness,
        }
    }
}

pub type ColorFormat = <ST7789 as mipidsi::models::Model>::ColorFormat;
pub type Display<'a> = mipidsi::Display<
    SpiInterface<
        'a,
        ExclusiveDevice<Spi<'a, esp_hal::Blocking>, Output<'a>, embedded_hal_bus::spi::NoDelay>,
        Output<'a>,
    >,
    ST7789,
    Output<'a>,
>;

pub fn init_display<'a>(config: IndicatorConfig) -> Display<'a> {
    // Turn on the backlight with LEDC
    let mut ledc = Ledc::new(unsafe { LEDC::steal() });
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty5Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: 500.Hz(),
        })
        .unwrap();
    let mut channel0 = ledc.channel(channel::Number::Channel0, config.backlight);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: config.max_brightness,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    // Initialize the SPI bus
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
    let buffer = mk_static!([u8; 1024], [0; 1024]);
    let di = SpiInterface::new(spi_device, dc, buffer);
    let rst = Output::new(config.rst, Level::High);

    // Define the display from the display interface and initialize it
    let mut display = Builder::new(ST7789, di)
        .reset_pin(rst)
        .display_size(config.width, config.height)
        .display_offset(config.offset_x, config.offset_y)
        .invert_colors(config.color_inversion)
        .color_order(config.color_order)
        .init(&mut delay)
        .unwrap();
    display.clear(ColorFormat::BLACK).unwrap();
    display
}
