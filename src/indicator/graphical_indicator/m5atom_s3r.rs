use embedded_graphics::prelude::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    gpio::{AnyPin, Level, Output, OutputConfig},
    i2c::master::{AnyI2c, Config, I2c},
    peripherals::{I2C1, SPI3},
    spi::{
        self, Mode,
        master::{AnySpi, Spi},
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
    pub sda: AnyPin<'static>,
    pub scl: AnyPin<'static>,
    pub i2c: AnyI2c<'static>,
    pub color_inversion: ColorInversion,
    pub color_order: ColorOrder,
    pub max_brightness: u8,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        // Hardware configuration is found at:
        // https://github.com/m5stack/M5GFX/blob/2c12f148d6e3df64ead33b04c7989fe6d90a7a83/src/M5GFX.cpp#L1782
        Self {
            width: 128,
            height: 128,
            // Field tested values, may vary on different devices
            offset_x: 2,
            offset_y: 1,
            spi: unsafe { SPI3::steal() }.into(),
            mosi: unsafe { AnyPin::steal(21) },
            sck: unsafe { AnyPin::steal(15) },
            dc: unsafe { AnyPin::steal(42) },
            cs: unsafe { AnyPin::steal(14) },
            rst: unsafe { AnyPin::steal(48) },
            sda: unsafe { AnyPin::steal(45) },
            scl: unsafe { AnyPin::steal(0) },
            i2c: unsafe { I2C1::steal().into() },
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
    let mut delay = esp_hal::delay::Delay::new();

    // Turn on the backlight with LP5562
    // @see https://github.com/m5stack/M5GFX/blob/2c12f148d6e3df64ead33b04c7989fe6d90a7a83/src/M5GFX.cpp#L566
    let mut i2c = I2c::new(
        config.i2c,
        Config::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_scl(config.scl)
    .with_sda(config.sda);
    i2c.write(48, &[0x00, 0b01000000]).unwrap();
    delay.delay_millis(1);
    i2c.write(48, &[0x08, 0b00000001]).unwrap();
    i2c.write(48, &[0x70, 0b00000000]).unwrap();
    i2c.write(48, &[0x0E, config.max_brightness]).unwrap();

    // Initialize the SPI bus
    let spi = Spi::new(
        config.spi,
        spi::master::Config::default()
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
