#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    peripheral::Peripheral,
    peripherals::TIMG1,
    rng::Rng,
    timer::{
        systimer::SystemTimer,
        timg::{MwdtStage, MwdtStageAction, TimerGroup, Wdt},
    },
};
use esp_hal_embassy::main;
use esp_println::println;
use esp_wifi::{
    config::PowerSaveMode,
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiController,
};
use fugit::ExtU64;
use log::{debug, error, info, warn};

#[allow(unused_imports)]
use esparrier::constants::*;

use esparrier::{
    mk_static, set_indicator_status, start_barrier_client, start_hid_task, start_indicator_task,
    AppConfig, IndicatorStatus, UsbActuator,
};

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    println!(
        "Firmware version: {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // Load the configuration
    println!("Config: {:?}", AppConfig::get());

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(160 * 1024);

    // Setup Embassy
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    // Setup watchdog on TIMG1, which is by default disabled by the bootloader
    let wdt1 = mk_static!(Wdt<TIMG1>, TimerGroup::new(peripherals.TIMG1).wdt);
    wdt1.set_timeout(MwdtStage::Stage0, 1.secs());
    wdt1.set_stage_action(MwdtStage::Stage0, MwdtStageAction::ResetSystem);
    wdt1.enable();
    wdt1.feed();

    // Start watchdog task
    spawner.must_spawn(watchdog_task(wdt1));

    // Setup HID task
    start_hid_task(spawner);

    // Setup paste button task
    #[cfg(feature = "clipboard")]
    spawner
        .spawn(esparrier::button_task())
        .inspect_err(|e| error!("Failed to start button task: {:?}", e))
        .unwrap();

    // Setup indicator
    start_indicator_task(spawner).await;
    set_indicator_status(IndicatorStatus::WifiConnecting).await;

    // Initialize WiFi
    let mut rng = Rng::new(peripherals.RNG);
    let seed: u64 = ((rng.random() as u64) << 32) | (rng.random() as u64);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK,).unwrap()
    );

    let wifi = peripherals.WIFI;
    let (wifi_interface, mut controller) =
        esp_wifi::wifi::new_with_mode(init, wifi, WifiStaDevice).unwrap();

    // Disable power saving for maximum performance
    controller.set_power_saving(PowerSaveMode::None).ok();

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        AppConfig::get().get_ip_config(),
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    // Start WiFi connection task
    spawner.must_spawn(connection(controller));
    // Start network stack task
    spawner.must_spawn(net_task(runner));

    info!("Waiting for WiFi to connect...");
    loop {
        if stack.is_link_up() {
            info!("WiFi connected!");
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
            warn!(
                "Disconnected from Barrier, error: {:?}, reconnecting in 5 seconds...",
                e
            )
        })
        .ok();
        Timer::after(Duration::from_millis(5000)).await;
    }
}

#[embassy_executor::task]
async fn watchdog_task(watchdog: &'static mut Wdt<<TIMG1 as Peripheral>::P>) {
    loop {
        watchdog.feed();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    debug!("start connection task");
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: AppConfig::get().ssid.clone(),
                password: AppConfig::get().password.clone().into(),
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
async fn net_task(mut runner: Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await
}
