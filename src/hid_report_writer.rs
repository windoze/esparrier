use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};

use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Channel, Receiver, Sender},
    signal::Signal,
};
use embassy_usb::class::hid::HidWriter;
use esp_hal::otg_fs::asynch::Driver;
use log::debug;

type ReportWriter<'a, const N: usize> = HidWriter<'a, Driver<'a>, N>;

pub static SUSPENDED: AtomicBool = AtomicBool::new(false);
pub static REMOTE_WAKEUP_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[derive(Debug)]
pub enum HidReport {
    Keyboard([u8; 9]),
    Mouse([u8; 8]),
    Consumer([u8; 3]),
}

impl HidReport {
    pub fn keyboard(data: [u8; 8]) -> Self {
        Self::Keyboard([
            1, data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ])
    }

    pub fn mouse(data: [u8; 7]) -> Self {
        Self::Mouse([
            2, data[0], data[1], data[2], data[3], data[4], data[5], data[6],
        ])
    }

    pub fn consumer(data: [u8; 2]) -> Self {
        Self::Consumer([3, data[0], data[1]])
    }
}

pub type HidReportChannel = Channel<NoopRawMutex, HidReport, 3>;

pub type HidReportSender = Sender<'static, NoopRawMutex, HidReport, 3>;

pub type HidReportReceiver = Receiver<'static, NoopRawMutex, HidReport, 3>;

trait HidReportWriter {
    fn write_report(&mut self, report: HidReport) -> impl Future<Output = ()>;
}

struct UsbHidReportWriter<'a> {
    hid_report_writer: ReportWriter<'a, 9>,
}

impl<'a> UsbHidReportWriter<'a> {
    pub fn new(hid_report_writer: ReportWriter<'a, 9>) -> Self {
        Self { hid_report_writer }
    }
}

impl<'a> HidReportWriter for UsbHidReportWriter<'a> {
    async fn write_report(&mut self, report: HidReport) {
        if SUSPENDED.load(Ordering::Acquire) {
            debug!("Triggering remote wakeup");
            REMOTE_WAKEUP_SIGNAL.signal(());
        } else {
            debug!("Sending report: {:?}", report);
            let data: &[u8] = match &report {
                HidReport::Keyboard(data) => data,
                HidReport::Mouse(data) => data,
                HidReport::Consumer(data) => data,
            };
            self.hid_report_writer.write(data).await.ok();
        }
    }
}

#[embassy_executor::task]
pub async fn start_hid_report_writer(
    writer: ReportWriter<'static, 9>,
    receiver: HidReportReceiver,
) {
    let mut writer = UsbHidReportWriter::new(writer);
    loop {
        let report = receiver.receive().await;
        writer.write_report(report).await;
    }
}
