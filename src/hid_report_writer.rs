use core::future::Future;

use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, Receiver, Sender},
};
use embassy_usb::class::hid::HidWriter;
use esp_hal::otg_fs::asynch::Driver;
use log::debug;

type ReportWriter<'a, const N: usize> = HidWriter<'a, Driver<'a>, N>;

#[derive(Debug)]
pub enum HidReport {
    Keyboard([u8; 9]),
    Mouse([u8; 8]),
    Consumer([u8; 3]),
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
        debug!("Sending report: {:?}", report);
        let data: &[u8] = match &report {
            HidReport::Keyboard(data) => data,
            HidReport::Mouse(data) => data,
            HidReport::Consumer(data) => data,
        };
        self.hid_report_writer.write(data).await.ok();
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
