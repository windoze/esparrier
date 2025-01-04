#![no_std]
#![no_main]

extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering};

use alloc::borrow::ToOwned;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use embassy_usb::{class::hid::HidWriter, Builder};
use esp_backtrace as _;
use esp_hal::{
    otg_fs::Usb,
    prelude::*,
    timer::timg::{MwdtStage, MwdtStageAction, TimerGroup},
};
use esp_println::println;
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiController,
};
use heapless::Vec;
use log::{debug, error, info, warn};

use esparrier::{
    indicator_task, mk_static, start, start_hid_report_writer, AppConfig, HidReportChannel,
    HidReportSender, IndicatorChannel, IndicatorSender, IndicatorStatus, SynergyHid, UsbActuator,
};

#[cfg(feature = "led")]
#[const_env::from_env]
const LED_PIN: u8 = 21;

#[cfg(feature = "smartled")]
#[const_env::from_env]
const SMART_LED_PIN: u8 = 35;

#[cfg(feature = "clipboard")]
#[const_env::from_env]
const PASTE_BUTTON_PIN: u8 = 41;

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    println!(
        "Firmware version: {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(160 * 1024);

    let channel = mk_static!(IndicatorChannel, IndicatorChannel::new());
    let receiver = channel.receiver();

    cfg_if::cfg_if! {
        if #[cfg(feature = "led")] {
            let indicator_config = esparrier::IndicatorConfig {
                pin: unsafe { esp_hal::gpio::GpioPin::<LED_PIN>::steal() }.into(),
            };
        } else if #[cfg(feature = "smartled")]{
            let indicator_config = esparrier::IndicatorConfig {
                rmt: peripherals.RMT,
                pin: unsafe { esp_hal::gpio::GpioPin::<SMART_LED_PIN>::steal() }.into(),
            };
        } else if #[cfg(feature = "graphics")]{
            let indicator_config = esparrier::IndicatorConfig {
                width: 128,
                height: 128,
                spi: peripherals.SPI3.into(),
                mosi: peripherals.GPIO21.into(),
                sck: peripherals.GPIO17.into(),
                dc: peripherals.GPIO33.into(),
                cs: peripherals.GPIO15.into(),
                rst: peripherals.GPIO34.into(),
                backlight: peripherals.GPIO16.into(),
                color_inversion: mipidsi::options::ColorInversion::Inverted,
                color_order: mipidsi::options::ColorOrder::Bgr,
            };
        } else {
            let indicator_config = esparrier::IndicatorConfig;
        }
    };
    spawner
        .spawn(indicator_task(indicator_config, receiver))
        .ok();

    let sender: IndicatorSender = channel.sender();
    sender.try_send(IndicatorStatus::WifiConnecting).ok();

    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER)
        .split::<esp_hal::timer::systimer::Target>();
    esp_hal_embassy::init(systimer.alarm0);

    info!("Embassy initialized!");

    // Load the configuration
    let app_config = mk_static!(AppConfig, AppConfig::load());

    // Setup watchdog on TIMG1, which is by default disabled by the bootloader
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    let mut wdt1 = timg1.wdt;
    wdt1.set_timeout(
        MwdtStage::Stage0,
        fugit::MicrosDurationU64::secs(app_config.watchdog_timeout as u64),
    );
    wdt1.set_stage_action(MwdtStage::Stage0, MwdtStageAction::ResetSystem);
    wdt1.enable();
    wdt1.feed();

    // Initialize the USB peripheral
    let hid_dev_state = mk_static!(
        embassy_usb::class::hid::State<'static>,
        embassy_usb::class::hid::State::new()
    );

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    let mut builder = init_hid(
        usb,
        app_config.vid,
        app_config.pid,
        app_config.manufacturer.as_str(),
        app_config.product.as_str(),
        app_config.serial_number.as_str(),
    );

    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: SynergyHid::get_report_descriptor().1,
        request_handler: None,
        poll_ms: 1,
        max_packet_size: 64,
    };

    let hid_dev = HidWriter::<'_, esp_hal::otg_fs::asynch::Driver<'_>, 9>::new(
        &mut builder,
        hid_dev_state,
        config,
    );

    // Build the builder.
    #[cfg(feature = "usb")]
    let mut usb = builder.build();
    #[cfg(not(feature = "usb"))]
    let _usb = builder.build();

    // // Run the USB device.
    #[cfg(feature = "usb")]
    let usb_fut = usb.run();
    #[cfg(not(feature = "usb"))]
    let usb_fut = async {
        loop {
            Timer::after(Duration::from_secs(1)).await;
        }
    };

    let hid_channel = mk_static!(HidReportChannel, HidReportChannel::new());
    let hid_receiver = hid_channel.receiver();
    let hid_sender = hid_channel.sender();
    spawner.must_spawn(start_hid_report_writer(hid_dev, hid_receiver));

    #[cfg(feature = "clipboard")]
    {
        let button = unsafe { esp_hal::gpio::GpioPin::<PASTE_BUTTON_PIN>::steal() }.into();
        spawner
            .spawn(esparrier::button_task(button, hid_sender))
            .ok();
    }

    // Initialize WiFi
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let seed: u64 = ((rng.random() as u64) << 32) | (rng.random() as u64);

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK,).unwrap()
    );

    let config = match app_config.get_ip_addr() {
        Some(addr) => embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: addr,
            dns_servers: Vec::new(),           // We don't really need DNS
            gateway: app_config.get_gateway(), // Gateway is optional if server is on the same subnet
        }),
        None => embassy_net::Config::dhcpv4(Default::default()),
    };

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(init, wifi, WifiStaDevice).unwrap();

    // Init network stack
    let stack = &*mk_static!(
        Stack<WifiDevice<'_, WifiStaDevice>>,
        Stack::new(
            wifi_interface,
            config,
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            seed,
        )
    );

    spawner.must_spawn(connection(
        controller,
        app_config.ssid.to_owned(),
        app_config.password.to_owned(),
    ));
    spawner.must_spawn(net_task(stack));

    info!("Waiting for WiFi to connect...");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    wdt1.feed();

    info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    wdt1.feed();

    spawner.must_spawn(barrier_client_task(
        app_config, stack, sender, hid_sender, wdt1,
    ));

    // TODO: How can I start it earlier? Now we have to wait until the WiFi is connected
    usb_fut.await;
}

#[embassy_executor::task]
async fn connection(
    mut controller: WifiController<'static>,
    ssid: heapless::String<32>,
    password: heapless::String<64>,
) {
    debug!("start connection task");
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: ssid.clone(),
                password: password.clone(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");
        }
        debug!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => info!("Wifi connected!"),
            Err(e) => {
                error!("Failed to connect to wifi, retrying in 5 seconds: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}

#[embassy_executor::task]
async fn barrier_client_task(
    app_config: &'static AppConfig,
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    indicator: IndicatorSender,
    hid_sender: HidReportSender,
    mut wdt: esp_hal::timer::timg::Wdt<
        <esp_hal::peripherals::TIMG1 as esp_hal::peripheral::Peripheral>::P,
    >,
) {
    loop {
        let mut actuator = UsbActuator::new(app_config, indicator, hid_sender);
        start(
            app_config.get_server_endpoint(),
            &app_config.screen_name,
            stack,
            &mut actuator,
            &mut wdt,
        )
        .await
        .inspect_err(|e| error!("Failed to connect: {:?}", e))
        .ok();
        warn!("Disconnected from Barrier, reconnecting in 5 seconds...");
        Timer::after(Duration::from_millis(5000)).await
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

fn init_hid<'a>(
    usb: Usb<'a>,
    vid: u16,
    pid: u16,
    manufacturer: &'static str,
    product: &'static str,
    serial_number: &'static str,
) -> Builder<'a, esp_hal::otg_fs::asynch::Driver<'a>> {
    // Create the driver, from the HAL.
    let ep_out_buffer = mk_static!([u8; 1024], [0u8; 1024]);
    let config = esp_hal::otg_fs::asynch::Config::default();
    let driver = esp_hal::otg_fs::asynch::Driver::new(usb, ep_out_buffer, config);
    let mut config = embassy_usb::Config::new(vid, pid);
    config.manufacturer = Some(manufacturer);
    config.product = Some(product);
    config.device_class = 0x03; // HID
    config.device_sub_class = 0x01; // Boot Interface Subclass
    config.device_protocol = 0x01; // Keyboard
    config.serial_number = Some(serial_number);
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
    builder
}
