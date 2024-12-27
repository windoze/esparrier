use core::future::Future;

use embassy_futures::select::select4;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, Receiver, Sender},
};
use embassy_usb::{
    class::hid::{HidReaderWriter, HidWriter, ReportId, RequestHandler},
    control::OutResponse,
};
use esp_hal::otg_fs::asynch::Driver;
use log::{debug, info};

type ReportWriter<'a, const N: usize> = HidWriter<'a, Driver<'a>, N>;

type ReportReaderWriter<'a, const READ_N: usize, const WRITE_N: usize> =
    HidReaderWriter<'a, Driver<'a>, READ_N, WRITE_N>;

#[derive(Debug)]
pub enum HidReport {
    Keyboard([u8; 8]),
    Mouse([u8; 7]),
    Consumer([u8; 2]),
}

pub type HidReportChannel = Channel<NoopRawMutex, HidReport, 3>;

pub type HidReportSender = Sender<'static, NoopRawMutex, HidReport, 3>;

pub type HidReportReceiver = Receiver<'static, NoopRawMutex, HidReport, 3>;

trait HidReportWriter {
    fn write_report(&mut self, report: HidReport) -> impl Future<Output = ()>;
}

struct UsbHidReportWriter<'a, 'b, 'c> {
    keyboard_writer: ReportWriter<'a, 8>,
    mouse_writer: ReportWriter<'b, 7>,
    consumer_writer: ReportWriter<'c, 2>,
}

impl<'a, 'b, 'c> UsbHidReportWriter<'a, 'b, 'c> {
    pub fn new(
        keyboard_writer: ReportWriter<'a, 8>,
        mouse_writer: ReportWriter<'b, 7>,
        consumer_writer: ReportWriter<'c, 2>,
    ) -> Self {
        Self {
            keyboard_writer,
            mouse_writer,
            consumer_writer,
        }
    }
}

impl<'a, 'b, 'c> HidReportWriter for UsbHidReportWriter<'a, 'b, 'c> {
    async fn write_report(&mut self, report: HidReport) {
        debug!("Sending report: {:?}", report);
        match report {
            HidReport::Keyboard(data) => {
                self.keyboard_writer.write(&data).await.ok();
            }
            HidReport::Mouse(data) => {
                self.mouse_writer.write(&data).await.ok();
            }
            HidReport::Consumer(data) => {
                self.consumer_writer.write(&data).await.ok();
            }
        }
    }
}

pub async fn start_hid_report_writer<'a, 'b, 'c>(
    keyboard: ReportReaderWriter<'a, 1, 8>,
    mouse: ReportReaderWriter<'b, 1, 7>,
    consumer: ReportReaderWriter<'c, 1, 2>,
    receiver: HidReportReceiver,
) {
    let (keyboard_writer, keyboard_out_fut) = start_hid_dev(keyboard);
    let (mouse_writer, mouse_out_fut) = start_hid_dev(mouse);
    let (consumer_writer, consumer_out_fut) = start_hid_dev(consumer);

    let writer_task = async {
        let mut writer = UsbHidReportWriter::new(keyboard_writer, mouse_writer, consumer_writer);
        loop {
            let report = receiver.receive().await;
            writer.write_report(report).await;
        }
    };
    select4(
        keyboard_out_fut,
        mouse_out_fut,
        consumer_out_fut,
        writer_task,
    )
    .await;
}

fn start_hid_dev<'a, const N: usize>(
    dev: HidReaderWriter<'a, Driver<'a>, 1, N>,
) -> (
    HidWriter<'a, Driver<'a>, N>,
    impl Future<Output = ()> + use<'a, N>,
) {
    let (reader, writer) = dev.split();
    let out_fut = async {
        let mut request_handler = MyRequestHandler {};
        reader.run(false, &mut request_handler).await;
    };
    (writer, out_fut)
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {:?}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}
