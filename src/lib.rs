#![no_std]

mod barrier_client;
#[cfg(feature = "clipboard")]
mod clipboard;
mod config;
mod control;
#[cfg(feature = "smartled")]
mod esp_hal_smartled;
mod global_state;
mod hid_report_writer;
mod indicator;
mod synergy_hid;
mod usb_actuator;

pub mod constants;
pub use barrier_client::*;
#[cfg(feature = "clipboard")]
pub use clipboard::{button_task, set_clipboard};
pub use config::{AppConfig, ConfigStore};
pub use global_state::{get_running_state, RunningState};
pub use hid_report_writer::{send_hid_report, start_hid_task, HidReport};
pub use indicator::*;
pub use synergy_hid::{ReportType, SynergyHid};
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
