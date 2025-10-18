#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    otg_fs::Usb,
    peripherals::TIMG1,
    rng::Rng,
    timer::timg::{MwdtStage, MwdtStageAction, TimerGroup, Wdt},
};
use esp_println::println;
use esp_rtos::main;
use log::{debug, error, info, warn};

#[allow(unused_imports)]
use esparrier::constants::*;

use esparrier::{
    AppConfig, IndicatorStatus, UsbActuator, mk_static, set_indicator_status, start_barrier_client,
    start_hid_task, start_indicator_task,
};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
async fn main(spawner: Spawner) {
    #[cfg(not(any(feature = "wifi", feature = "ethernet")))]
    compile_error!("At least one of the features 'wifi' or 'ethernet' must be enabled");

    #[cfg(all(feature = "wifi", feature = "ethernet"))]
    compile_error!("Only one of the features 'wifi' or 'ethernet' can be enabled");

    esp_println::logger::init_logger_from_env();

    println!(
        "Firmware version: {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(size: 160 * 1024);

    // Setup Embassy
    // let systimer = SystemTimer::new(peripherals.SYSTIMER);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Load the configuration
    AppConfig::init(peripherals.FLASH).await;
    println!("Config: {:?}", AppConfig::get());

    // Setup watchdog on TIMG1, which is by default disabled by the bootloader
    let wdt1 = mk_static!(Wdt<TIMG1>, TimerGroup::new(peripherals.TIMG1).wdt);
    wdt1.set_timeout(MwdtStage::Stage0, esp_hal::time::Duration::from_secs(1));
    wdt1.set_stage_action(MwdtStage::Stage0, MwdtStageAction::ResetSystem);
    wdt1.enable();
    wdt1.feed();

    // Start watchdog task
    spawner.must_spawn(watchdog_task(wdt1));

    // Setup HID task
    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    start_hid_task(spawner, usb);

    // Setup paste button task
    #[cfg(feature = "clipboard")]
    spawner
        .spawn(esparrier::button_task())
        .inspect_err(|e| error!("Failed to start button task: {e:?}"))
        .unwrap();

    // Setup indicator
    start_indicator_task(spawner).await;
    set_indicator_status(IndicatorStatus::WifiConnecting).await;

    // Initialize network
    let rng = Rng::new();
    let seed: u64 = ((rng.random() as u64) << 32) | (rng.random() as u64);

    #[cfg(feature = "wifi")]
    let stack = {
        use esp_radio::{Controller, wifi::PowerSaveMode};

        let esp_radio_ctrl = &*mk_static!(Controller<'static>, esp_radio::init().unwrap());

        let (mut controller, interfaces) =
            esp_radio::wifi::new(esp_radio_ctrl, peripherals.WIFI, Default::default())
                .inspect_err(|e| {
                    error!("Failed to initialize WiFi: {e:?}");
                })
                .unwrap();
        // Disable power saving for maximum performance
        controller.set_power_saving(PowerSaveMode::None).ok();

        let wifi_interface = interfaces.sta;

        // Init network stack
        let (stack, runner) = embassy_net::new(
            wifi_interface,
            AppConfig::get().get_ip_config(),
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            seed,
        );
        // Start WiFi connection task
        spawner.must_spawn(wifi_task(controller));

        // Start network stack task
        spawner.must_spawn(net_task(runner));

        stack
    };
    #[cfg(feature = "ethernet")]
    let stack = {
        use embassy_net_wiznet::State;
        use embassy_time::Delay;
        use embedded_hal_bus::spi::ExclusiveDevice;
        use esp_hal::{
            gpio::{Input, Level, Output, Pull},
            spi::master::Spi,
            time::Rate,
        };
        use static_cell::StaticCell;

        let spi_cfg = esp_hal::spi::master::Config::default()
            .with_frequency(Rate::from_hz(50_000_000))
            .with_mode(esp_hal::spi::Mode::_0);

        let spi = Spi::new(peripherals.SPI2, spi_cfg)
            .unwrap()
            .with_miso(unsafe { esp_hal::gpio::AnyPin::steal(W5500_MISO_PIN) })
            .with_mosi(unsafe { esp_hal::gpio::AnyPin::steal(W5500_MOSI_PIN) })
            .with_sck(unsafe { esp_hal::gpio::AnyPin::steal(W5500_SCK_PIN) })
            .into_async();
        let w5500_cs = Output::new(
            unsafe { esp_hal::gpio::AnyPin::steal(W5500_CS_PIN) },
            Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );
        let w5500_int = Input::new(
            unsafe { esp_hal::gpio::AnyPin::steal(W5500_INT_PIN) },
            esp_hal::gpio::InputConfig::default().with_pull(Pull::Up),
        );
        let w5500_reset = Output::new(
            unsafe { esp_hal::gpio::AnyPin::steal(W5500_RESET_PIN) },
            Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );

        let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
        static STATE: StaticCell<State<8, 8>> = StaticCell::new();
        let state = STATE.init(State::<8, 8>::new());
        let (device, runner) = embassy_net_wiznet::new(
            mac_addr,
            state,
            ExclusiveDevice::new(spi, w5500_cs, Delay).unwrap(),
            w5500_int,
            w5500_reset,
        )
        .await
        .inspect_err(|e| {
            error!("Failed to initialize W5500: {e:?}");
        })
        .unwrap();
        // Launch ethernet task
        spawner.spawn(ethernet_task(runner)).unwrap();

        // Init network stack
        static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
        let (stack, runner) = embassy_net::new(
            device,
            embassy_net::Config::dhcpv4(Default::default()),
            RESOURCES.init(StackResources::new()),
            seed,
        );

        // Start network stack task
        spawner.must_spawn(net_task(runner));

        stack
    };

    info!("Waiting for net link up...");
    loop {
        if stack.is_link_up() {
            info!("Net link is up!");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            set_indicator_status(IndicatorStatus::WifiConnected(config.address)).await;
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    loop {
        // Start the Barrier client
        let actuator = UsbActuator::default();
        start_barrier_client(
            AppConfig::get().get_server_endpoint(),
            &AppConfig::get().screen_name,
            AppConfig::get().jiggle_interval,
            stack,
            actuator,
        )
        .await
        .inspect_err(|e| {
            warn!("Disconnected from Barrier, error: {e:?}, reconnecting in 5 seconds...")
        })
        .ok();
        Timer::after(Duration::from_millis(5000)).await;
    }
}

#[embassy_executor::task]
async fn watchdog_task(watchdog: &'static mut Wdt<TIMG1<'static>>) {
    loop {
        watchdog.feed();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[cfg(feature = "wifi")]
#[embassy_executor::task]
async fn wifi_task(mut controller: esp_radio::wifi::WifiController<'static>) {
    use esp_radio::wifi::{ClientConfig, ModeConfig, WifiEvent, WifiStaState};

    debug!("start connection task");
    loop {
        if esp_radio::wifi::sta_state() == WifiStaState::Connected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(AppConfig::get().ssid.as_str().into())
                    .with_password(AppConfig::get().password.as_ref().into()),
            );
            controller.set_config(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");
        }
        debug!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => info!("Wifi connected!"),
            Err(e) => {
                error!("Failed to connect to wifi, retrying in 5 seconds: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[cfg(feature = "ethernet")]
#[embassy_executor::task]
async fn ethernet_task(
    runner: embassy_net_wiznet::Runner<
        'static,
        embassy_net_wiznet::chip::W5500,
        embedded_hal_bus::spi::ExclusiveDevice<
            esp_hal::spi::master::Spi<'static, esp_hal::Async>,
            esp_hal::gpio::Output<'static>,
            embassy_time::Delay,
        >,
        esp_hal::gpio::Input<'static>,
        esp_hal::gpio::Output<'static>,
    >,
) -> ! {
    debug!("start ethernet task");
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(
    #[cfg(feature = "wifi")] mut runner: embassy_net::Runner<
        'static,
        esp_radio::wifi::WifiDevice<'static>,
    >,
    #[cfg(feature = "ethernet")] mut runner: embassy_net::Runner<
        'static,
        embassy_net_wiznet::Device<'static>,
    >,
) {
    runner.run().await
}
