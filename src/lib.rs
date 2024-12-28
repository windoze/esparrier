#![no_std]

mod barrier_client;
mod config;
#[cfg(feature = "smartled")]
mod esp_hal_smartled;
mod hid_report_writer;
mod indicator;
mod synergy_hid;
mod usb_actuator;

pub use barrier_client::*;
pub use config::AppConfig;
pub use hid_report_writer::{
    start_hid_report_writer, HidReport, HidReportChannel, HidReportReceiver, HidReportSender,
};
pub use indicator::*;
pub use synergy_hid::{ReportType, SynergyHid};
#[cfg(feature = "clipboard")]
pub use usb_actuator::send_clipboard;
pub use usb_actuator::UsbActuator;

#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
