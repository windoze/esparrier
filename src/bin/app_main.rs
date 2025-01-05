#![no_std]
#![no_main]

extern crate alloc;

use alloc::borrow::ToOwned;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
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

#[allow(unused_imports)]
use esparrier::constants::*;

use esparrier::{
    indicator_task, init_hid, mk_static, start_barrier_client, AppConfig, IndicatorChannel,
    IndicatorSender, IndicatorStatus, UsbActuator,
};

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

    // TODO: We can read pin number from the config after the [#2854](https://github.com/esp-rs/esp-hal/pull/2854) is released, then we can enable `smartled` and `led` at the same time, and the indicator cat be selected by the config at runtime.
    // But the graphics indicator is totally another story.
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

    let indicator_sender: IndicatorSender = channel.sender();
    indicator_sender
        .try_send(IndicatorStatus::WifiConnecting)
        .ok();

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

    let (hid_sender, hid_fut) = init_hid(spawner, app_config);

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
    let (wifi_interface, mut controller) =
        esp_wifi::wifi::new_with_mode(init, wifi, WifiStaDevice).unwrap();

    // Disable power saving for maximum performance
    controller
        .set_power_saving(esp_wifi::config::PowerSaveMode::None)
        .ok();

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

    // Start WiFi connection task
    spawner.must_spawn(connection(
        controller,
        app_config.ssid.to_owned(),
        app_config.password.to_owned(),
    ));
    // Start network stack task
    spawner.must_spawn(net_task(stack));

    let barrier_fut = async {
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

        loop {
            let mut actuator = UsbActuator::new(app_config, indicator_sender, hid_sender);
            start_barrier_client(
                app_config.get_server_endpoint(),
                &app_config.screen_name,
                stack,
                &mut actuator,
                &mut wdt1,
            )
            .await
            .inspect_err(|e| error!("Failed to connect: {:?}", e))
            .ok();
            warn!("Disconnected from Barrier, reconnecting in 5 seconds...");
            Timer::after(Duration::from_millis(5000)).await
        }
    };

    select(hid_fut, barrier_fut).await;
    error!("Critical task exited, restarting...");
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
