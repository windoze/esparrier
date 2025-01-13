use esp_hal::{
    i2c::{
        self,
        master::{AnyI2c, I2c},
    },
    peripherals::I2C1,
};

use super::*;
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
    pub sda: AnyPin,
    pub scl: AnyPin,
    pub i2c: AnyI2c,
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
            offset_x: 2, // M5Atom S3 has 1px offset on both x and y axis
            offset_y: 1,
            spi: unsafe { SPI3::steal() }.into(),
            mosi: unsafe { GpioPin::<21>::steal() }.into(),
            sck: unsafe { GpioPin::<15>::steal() }.into(),
            dc: unsafe { GpioPin::<42>::steal() }.into(),
            cs: unsafe { GpioPin::<14>::steal() }.into(),
            rst: unsafe { GpioPin::<48>::steal() }.into(),
            sda: unsafe { GpioPin::<45>::steal() }.into(),
            scl: unsafe { GpioPin::<0>::steal() }.into(),
            i2c: unsafe { I2C1::steal().into() },
            color_inversion: ColorInversion::Inverted,
            color_order: ColorOrder::Bgr,
            max_brightness: crate::AppConfig::get().brightness,
        }
    }
}

pub type Display<'a> = mipidsi::Display<
    SPIInterface<
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
        i2c::master::Config {
            frequency: 400.kHz(),
            ..Default::default()
        },
    )
    .with_scl(config.scl)
    .with_sda(config.sda);
    i2c.write(48, &[0x00, 0b01000000]).unwrap();
    delay.delay_millis(1);
    i2c.write(48, &[0x08, 0b00000001]).unwrap();
    i2c.write(48, &[0x70, 0b00000000]).unwrap();
    i2c.write(48, &[0x0E, config.max_brightness]).unwrap();

    // Initialize the SPI bus
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
        .display_size(config.width, config.height)
        .display_offset(config.offset_x, config.offset_y)
        .invert_colors(config.color_inversion)
        .color_order(config.color_order)
        .init(&mut delay)
        .unwrap()
}
