#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::prelude::*;
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiController,
};

use esparrier::{start, AppConfig, UsbActuator};
use log::{error, info};

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);

    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER)
        .split::<esp_hal::timer::systimer::Target>();
    esp_hal_embassy::init(systimer.alarm0);

    info!("Embassy initialized!");

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

    let app_config = AppConfig::load();

    spawner
        .spawn(connection(controller, app_config.ssid, app_config.password))
        .ok();
    spawner.spawn(net_task(stack)).ok();

    loop {
        Timer::after(Duration::from_millis(5000)).await;
        let mut actuator = UsbActuator::new(app_config.screen_width, app_config.screen_height);

        let actuator_task = start(
            app_config.server.clone(),
            app_config.screen_name.clone(),
            stack,
            &mut actuator,
        );

        actuator_task.await.ok();
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.22.0/examples/src/bin
}

#[embassy_executor::task]
async fn connection(
    mut controller: WifiController<'static>,
    ssid: heapless::String<32>,
    password: heapless::String<64>,
) {
    info!("start connection task");
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
        info!("About to connect...");

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
