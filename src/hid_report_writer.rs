use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, Receiver, Sender},
};
use embassy_usb::class::hid::HidWriter;
use esp_hal::{
    gpio::GpioPin,
    otg_fs::{asynch::Driver, Usb},
    peripherals::USB0,
};
use log::{debug, info};

use crate::{mk_static, AppConfig, SynergyHid};

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

pub type HidReportSender = Sender<'static, NoopRawMutex, HidReport, 3>;

type HidReportChannel = Channel<NoopRawMutex, HidReport, 3>;
type HidReportReceiver = Receiver<'static, NoopRawMutex, HidReport, 3>;
type ReportWriter<'a, const N: usize> = HidWriter<'a, Driver<'a>, N>;

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

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

// TODO: Make remote wakeup work.
impl embassy_usb::Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Device {}", if enabled { "enabled" } else { "disabled" });
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}

pub fn init_hid(spawner: Spawner, app_config: &'static AppConfig) -> HidReportSender {
    // We cannot take ownership from `peripherals` as it cannot be `mk_static`.
    let usb = Usb::new(
        unsafe { USB0::steal() },
        unsafe { GpioPin::<20>::steal() },
        unsafe { GpioPin::<19>::steal() },
    );
    // Create the driver, from the HAL.
    let ep_out_buffer = mk_static!([u8; 1024], [0u8; 1024]);
    let config = esp_hal::otg_fs::asynch::Config::default();
    let driver = esp_hal::otg_fs::asynch::Driver::new(usb, ep_out_buffer, config);
    let mut config = embassy_usb::Config::new(app_config.vid, app_config.pid);
    config.manufacturer = Some(&app_config.manufacturer);
    config.product = Some(&app_config.product);
    config.device_class = 0x03; // HID
    config.device_sub_class = 0x01; // Boot Interface Subclass
    config.device_protocol = 0x01; // Keyboard
    config.serial_number = Some(&app_config.serial_number);
    config.max_power = 100;
    config.supports_remote_wakeup = true;
    config.max_packet_size_0 = 16;

    // Create embassy-usb DeviceBuilder using the driver and config.
    let config_descriptor_buf = mk_static!([u8; 256], [0u8; 256]);
    let bos_descriptor_buf = mk_static!([u8; 256], [0u8; 256]);
    let msos_descriptor_buf = mk_static!([u8; 256], [0u8; 256]);
    let control_buf = mk_static!([u8; 256], [0u8; 256]);
    let device_handler = mk_static!(MyDeviceHandler, MyDeviceHandler::new());

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        config_descriptor_buf,
        bos_descriptor_buf,
        msos_descriptor_buf,
        control_buf,
    );

    builder.handler(device_handler);

    // Create classes on the builder.
    let hid_dev_state = mk_static!(
        embassy_usb::class::hid::State<'static>,
        embassy_usb::class::hid::State::new()
    );

    let hid_dev = HidWriter::<'_, esp_hal::otg_fs::asynch::Driver<'_>, 9>::new(
        &mut builder,
        hid_dev_state,
        embassy_usb::class::hid::Config {
            report_descriptor: SynergyHid::get_report_descriptor().1,
            request_handler: None,
            poll_ms: 1, // As fast as possible, we already have a long latency from WiFi.
            max_packet_size: 16,
        },
    );

    // Build the builder.
    cfg_if::cfg_if! {
        if #[cfg(feature = "usb")] {
            // Start the USB stack.
            spawner.must_spawn(usb_task(builder.build()));
        } else {
            info!("USB is disabled");
        }
    }

    // Create the HID report channel.
    let hid_channel = mk_static!(HidReportChannel, HidReportChannel::new());
    let hid_receiver = hid_channel.receiver();
    let hid_sender = hid_channel.sender();

    // Start the HID report writer task.
    spawner.must_spawn(start_hid_report_writer(hid_dev, hid_receiver));

    // Return the HID report sender.
    hid_sender
}

#[embassy_executor::task]
async fn start_hid_report_writer(writer: ReportWriter<'static, 9>, receiver: HidReportReceiver) {
    let mut writer = UsbHidReportWriter::new(writer);
    loop {
        let report = receiver.receive().await;
        writer.write_report(report).await;
    }
}

#[embassy_executor::task]
async fn usb_task(
    mut usb: embassy_usb::UsbDevice<'static, esp_hal::otg_fs::asynch::Driver<'static>>,
) {
    usb.run().await;
}
