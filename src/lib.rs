#![no_std]

mod barrier_client;
mod config;
mod synergy_hid;
mod usb_actuator;

pub use barrier_client::*;
pub use config::AppConfig;
pub use synergy_hid::{ReportType, SynergyHid};
pub use usb_actuator::UsbActuator;

pub type ReportWriter<'a, const N: usize> =
    embassy_usb::class::hid::HidWriter<'a, esp_hal::otg_fs::asynch::Driver<'a>, N>;
