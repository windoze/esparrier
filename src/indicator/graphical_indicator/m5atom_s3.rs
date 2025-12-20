use embedded_graphics::prelude::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    gpio::{self, AnyPin, Level, Output, OutputConfig},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
    peripherals::{LEDC, SPI3},
    spi::{
        Mode,
        master::{AnySpi, Config, Spi},
    },
    time::Rate,
};
use mipidsi::{
    Builder,
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, ColorOrder},
};

use crate::mk_static;

pub struct IndicatorConfig {
    pub width: u16,
    pub height: u16,
    pub offset_x: u16,
    pub offset_y: u16,
    pub spi: AnySpi<'static>,
    pub mosi: AnyPin<'static>,
    pub sck: AnyPin<'static>,
    pub dc: AnyPin<'static>,
    pub cs: AnyPin<'static>,
    pub rst: AnyPin<'static>,
    pub backlight: AnyPin<'static>,
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
            mosi: unsafe { AnyPin::steal(21) },
            sck: unsafe { AnyPin::steal(17) },
            dc: unsafe { AnyPin::steal(33) },
            cs: unsafe { AnyPin::steal(15) },
            rst: unsafe { AnyPin::steal(34) },
            backlight: unsafe { AnyPin::steal(16) },
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
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(500),
        })
        .inspect_err(|e| log::error!("{:?}", e))
        .unwrap();
    let mut channel0 = ledc.channel(channel::Number::Channel0, config.backlight);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: config.max_brightness,
            drive_mode: gpio::DriveMode::PushPull,
        })
        .unwrap();

    // Initialize the SPI bus
    let mut delay = esp_hal::delay::Delay::new();
    let spi = Spi::new(
        config.spi,
        Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(config.sck)
    .with_mosi(config.mosi);

    let cs_output = Output::new(config.cs, Level::High, OutputConfig::default());
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs_output).unwrap();

    let dc = Output::new(config.dc, Level::Low, OutputConfig::default());
    let buffer = mk_static!([u8; 1024], [0; 1024]);
    let di = SpiInterface::new(spi_device, dc, buffer);
    let rst = Output::new(config.rst, Level::High, OutputConfig::default());

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
