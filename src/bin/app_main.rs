#![no_std]
#![no_main]

extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering};

use alloc::borrow::ToOwned;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use embassy_usb::{class::hid::HidReaderWriter, Builder};
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
use log::{debug, error, info, warn};

use esparrier::{
    mk_static, start, start_hid_report_writer, AppConfig, HidReportChannel, HidReportSender,
    IndicatorChannel, IndicatorReceiver, IndicatorSender, IndicatorStatus, SynergyHid, UsbActuator,
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
            spawner
            .spawn(indicator_task(
                // Is there any other way to obtain a pin from pin number?
                unsafe { esp_hal::gpio::GpioPin::<LED_PIN>::steal() }.into(),
                receiver,
            ))
            .ok();
        } else if #[cfg(feature = "smartled")]{
            spawner
            .spawn(indicator_task(
                peripherals.RMT,
                unsafe { esp_hal::gpio::GpioPin::<SMART_LED_PIN>::steal() }.into(),
                receiver,
            ))
            .ok();
        } else {
            spawner.spawn(indicator_task(receiver)).ok();
        }
    }

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
    let mut keyboard_state = embassy_usb::class::hid::State::new();
    let mut mouse_state = embassy_usb::class::hid::State::new();
    let mut consumer_state = embassy_usb::class::hid::State::new();

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    let mut builder = init_hid(
        usb,
        app_config.vid,
        app_config.pid,
        app_config.manufacturer.as_str(),
        app_config.product.as_str(),
        app_config.serial_number.as_str(),
    );

    let keyboard = init_hid_dev(
        &mut builder,
        &mut keyboard_state,
        esparrier::ReportType::Keyboard,
    );
    let mouse = init_hid_dev(&mut builder, &mut mouse_state, esparrier::ReportType::Mouse);
    let consumer = init_hid_dev(
        &mut builder,
        &mut consumer_state,
        esparrier::ReportType::Consumer,
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
    let report_task = start_hid_report_writer(keyboard, mouse, consumer, hid_receiver);

    #[cfg(feature = "clipboard")]
    {
        let button = unsafe { esp_hal::gpio::GpioPin::<PASTE_BUTTON_PIN>::steal() }.into();
        spawner.spawn(button_task(button, hid_sender)).ok();
    }

    // Initialize WiFi
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let seed: u64 = ((rng.random() as u64) << 32) | (rng.random() as u64);

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK,).unwrap()
    );

    let config = embassy_net::Config::dhcpv4(Default::default());

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

    // Wait for anything to finish, then restart.
    // These are all necessary tasks, none of them should exit prematurely.
    select(usb_fut, report_task).await;
    warn!("Critical task exited, restarting...");
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
            app_config.server.clone(),
            app_config.screen_name.clone(),
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
    config.serial_number = Some(serial_number);
    config.max_power = 150;
    config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let config_descriptor = mk_static!([u8; 256], [0u8; 256]);
    let bos_descriptor = mk_static!([u8; 256], [0u8; 256]);
    // You can also add a Microsoft OS descriptor.
    let msos_descriptor = mk_static!([u8; 256], [0u8; 256]);
    let control_buf = mk_static!([u8; 256], [0u8; 256]);
    let device_handler = mk_static!(MyDeviceHandler, MyDeviceHandler::new());

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
    );

    builder.handler(device_handler);
    builder
}

fn init_hid_dev<'a, const N: usize>(
    builder: &mut Builder<'a, esp_hal::otg_fs::asynch::Driver<'a>>,
    state: &'a mut embassy_usb::class::hid::State<'a>,
    report_type: esparrier::ReportType,
) -> HidReaderWriter<'a, esp_hal::otg_fs::asynch::Driver<'a>, 1, N> {
    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: SynergyHid::get_report_descriptor(report_type).1,
        request_handler: None,
        poll_ms: 1,
        max_packet_size: 64,
    };

    HidReaderWriter::<'a, esp_hal::otg_fs::asynch::Driver<'a>, 1, N>::new(builder, state, config)
}

#[cfg(feature = "led")]
#[embassy_executor::task]
async fn indicator_task(pin: esp_hal::gpio::AnyPin, receiver: IndicatorReceiver) {
    esparrier::start_indicator(pin, receiver).await;
}

#[cfg(feature = "smartled")]
#[embassy_executor::task]
async fn indicator_task(
    rmt: esp_hal::peripherals::RMT,
    pin: esp_hal::gpio::AnyPin,
    receiver: IndicatorReceiver,
) {
    esparrier::start_indicator(rmt, pin, receiver).await;
}

// Fallback to dummy indicator, no-op
#[cfg(not(feature = "indicator"))]
#[embassy_executor::task]
async fn indicator_task(receiver: IndicatorReceiver) {
    loop {
        receiver.receive().await;
    }
}

#[cfg(feature = "clipboard")]
#[embassy_executor::task]
async fn button_task(button: esp_hal::gpio::AnyPin, hid_writer: HidReportSender) {
    use embedded_hal_async::digital::Wait;
    let input = esp_hal::gpio::Input::new(button, esp_hal::gpio::Pull::Up);
    let mut debouncer = async_debounce::Debouncer::new(input, Duration::from_millis(50));

    loop {
        debouncer.wait_for_rising_edge().await.ok();
        esparrier::send_clipboard(hid_writer).await;
    }
}
