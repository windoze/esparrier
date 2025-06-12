use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver, Sender},
    once_lock::OnceLock,
};
use embassy_time::{with_timeout, Duration};
use embassy_usb::{
    class::hid::HidWriter,
    msos::{self, windows_version},
};
use esp_hal::{
    otg_fs::{asynch::Driver, Usb},
    system,
};
use log::{debug, info, warn};

use crate::{constants::DEVICE_INTERFACE_GUIDS, mk_static, AppConfig, SynergyHid};

type ReportWriter<'a, const N: usize> = HidWriter<'a, Driver<'a>, N>;

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

pub type HidReportSender = Sender<'static, CriticalSectionRawMutex, HidReport, 32>;

type HidReportChannel = Channel<CriticalSectionRawMutex, HidReport, 32>;
type HidReportReceiver = Receiver<'static, CriticalSectionRawMutex, HidReport, 32>;

trait HidReportWriter {
    fn write_report(&mut self, report: HidReport) -> impl Future<Output = ()>;
}

struct UsbHidReportWriter<'a> {
    hid_report_writer: ReportWriter<'a, 9>,
    polling_interval: u8,
}

impl<'a> UsbHidReportWriter<'a> {
    pub fn new(hid_report_writer: ReportWriter<'a, 9>) -> Self {
        let config = AppConfig::get();
        Self {
            hid_report_writer,
            polling_interval: config.get_polling_interval(),
        }
    }
}

impl HidReportWriter for UsbHidReportWriter<'_> {
    async fn write_report(&mut self, report: HidReport) {
        debug!("Sending report: {:?}", report);
        let data: &[u8] = match &report {
            HidReport::Keyboard(data) => data,
            HidReport::Mouse(data) => data,
            HidReport::Consumer(data) => data,
        };
        // Assuming 3 * polling_interval is enough time for the host to poll the device, but not too short or too long.
        let timeout = Duration::from_millis((self.polling_interval as u64 * 3).clamp(10, 100));
        if with_timeout(timeout, async {
            self.hid_report_writer
                .write(data)
                .await
                .inspect_err(|e| {
                    warn!("Error writing HID report: {:?}", e);
                })
                .ok();
        })
        .await
        .is_err()
        {
            // This can happen if the device is writing the report while unplugged.
            // Some board doesn't really support `self_powered` because the VBUS pin
            // in USB-OTG port is not solely powered by the host, or, it has not
            // configured a GPIO pin to monitor the VBUS voltage, which doesn't meet
            // the standard of USB self-powered device.
            // In this case, the board cannot detect the unplugging event even the
            // function is already implemented by the underlying OTG driver. And if a
            // report is being sent while the device is unplugged, the USB stack will
            // be stalled.
            // Above scenario may happen if the device is plugged into a USB hub which
            // supplies power to the device even if the host is disconnected or powered
            // off.
            // There is no way we can resume the USB stack, so we just panic here, and
            // the watchdog will reset the board.
            // @see https://docs.espressif.com/projects/esp-idf/zh_CN/latest/esp32s3/api-reference/peripherals/usb_device.html#self-powered-device
            warn!("Timeout writing HID report, resetting the system.");
            system::software_reset()
        }
    }
}

#[embassy_executor::task]
async fn start_hid_report_writer(writer: ReportWriter<'static, 9>, receiver: HidReportReceiver) {
    let mut writer = UsbHidReportWriter::new(writer);
    loop {
        let report = receiver.receive().await;
        writer.write_report(report).await;
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

static HID_REPORT_SENDER: OnceLock<HidReportSender> = OnceLock::new();

pub async fn send_hid_report(report: HidReport) {
    HID_REPORT_SENDER.get().await.send(report).await;
}

pub fn start_hid_task(spawner: Spawner, usb: Usb<'static>) {
    let app_config = AppConfig::get();

    let ep_out_buffer = mk_static!([u8; 1024], [0u8; 1024]);
    let config = esp_hal::otg_fs::asynch::Config::default();
    let driver = esp_hal::otg_fs::asynch::Driver::new(usb, ep_out_buffer, config);
    let mut config = embassy_usb::Config::new(app_config.vid, app_config.pid);
    config.manufacturer = Some(&app_config.manufacturer);
    config.product = Some(&app_config.product);
    // TODO: MacOs doesn't like these settings, why? Not sure about the last 2 but the 1st one is definitely the issue.
    // config.device_class = 0x03; // HID
    // config.device_sub_class = 0x01; // Boot Interface Subclass
    // config.device_protocol = 0x01; // Keyboard
    config.device_class = 0xEF; // Miscellaneous Device
    config.device_sub_class = 0x02; // Common Class
    config.device_protocol = 0x01; // Interface Association Descriptor
    config.composite_with_iads = true;
    config.serial_number = Some(&app_config.serial_number);
    config.max_power = 100;
    config.max_packet_size_0 = 64;

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
    builder.msos_descriptor(windows_version::WIN8_1, 0);

    // Initialize the USB peripheral
    let hid_dev_state = mk_static!(
        embassy_usb::class::hid::State<'static>,
        embassy_usb::class::hid::State::new()
    );

    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: SynergyHid::get_report_descriptor().1,
        request_handler: None,
        poll_ms: app_config.get_polling_interval(),
        max_packet_size: 64,
    };

    let hid_dev = HidWriter::<'_, esp_hal::otg_fs::asynch::Driver<'_>, 9>::new(
        &mut builder,
        hid_dev_state,
        config,
    );

    // Add a vendor-specific function (class 0xFF), and corresponding interface,
    // that uses our custom handler.
    let mut function = builder.function(0xFF, 0x0D, 0x0A);
    function.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    function.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    ));
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xFF, 0x0D, 0x0A, None);
    let read_ep = alt.endpoint_bulk_out(64);
    let write_ep = alt.endpoint_bulk_in(64);
    drop(function);
    spawner.must_spawn(crate::control::control_task(read_ep, write_ep));

    // // Run the USB device.
    spawner.must_spawn(usb_task(builder));

    let hid_channel = mk_static!(HidReportChannel, HidReportChannel::new());
    let hid_receiver = hid_channel.receiver();
    let hid_sender = hid_channel.sender();
    spawner.must_spawn(start_hid_report_writer(hid_dev, hid_receiver));

    HID_REPORT_SENDER.init(hid_sender).ok();
}

#[embassy_executor::task]
async fn usb_task(builder: embassy_usb::Builder<'static, Driver<'static>>) {
    // I highly doubt there are some kind of race conditions inside of the OTG_FS driver.
    // M5Atom S3 cannot start the USB peripheral without a delay, but S3 Lite can.
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    cfg_if::cfg_if! {
        if #[cfg(feature = "usb")] {
            // Build the builder.
            let mut usb = builder.build();
            usb.run().await;
        } else {
            let _builder = builder;
            log::warn!("USB feature is disabled.");
            loop {
                embassy_time::Timer::after(embassy_time::Duration::from_secs(3600)).await;
            }
        }
    }
}
