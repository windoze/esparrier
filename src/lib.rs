#![no_std]

mod barrier_client;
mod config;
#[cfg(feature = "smartled")]
mod esp_hal_smartled;
mod hid_report_writer;
mod indicator;
mod synergy_hid;
mod usb_actuator;
mod usb_report_writer;

pub use barrier_client::*;
pub use config::AppConfig;
#[cfg(feature = "smartled")]
pub use esp_hal_smartled::*;
pub use hid_report_writer::{
    start_hid_report_writer, HidReport, HidReportChannel, HidReportReceiver, HidReportSender,
};
pub use indicator::*;
pub use synergy_hid::{ReportType, SynergyHid};
pub use usb_actuator::UsbActuator;
pub use usb_report_writer::start_writer;

pub type ReportWriter<'a, const N: usize> =
    embassy_usb::class::hid::HidWriter<'a, esp_hal::otg_fs::asynch::Driver<'a>, N>;

pub type ReportReaderWriter<'a, const READ_N: usize, const WRITE_N: usize> =
    embassy_usb::class::hid::HidReaderWriter<
        'a,
        esp_hal::otg_fs::asynch::Driver<'a>,
        READ_N,
        WRITE_N,
    >;

#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
