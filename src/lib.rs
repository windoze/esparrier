#![no_std]

mod barrier_client;
mod config;
#[cfg(feature = "smartled")]
mod esp_hal_smartled;
mod indicator;
mod synergy_hid;
mod usb_actuator;

pub use barrier_client::*;
pub use config::AppConfig;
#[cfg(feature = "smartled")]
pub use esp_hal_smartled::*;
pub use indicator::*;
pub use synergy_hid::{ReportType, SynergyHid};
pub use usb_actuator::UsbActuator;

pub type ReportWriter<'a, const N: usize> =
    embassy_usb::class::hid::HidWriter<'a, esp_hal::otg_fs::asynch::Driver<'a>, N>;
