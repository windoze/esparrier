use embassy_net::Ipv4Cidr;
use embassy_sync::once_lock::OnceLock;
use log::info;

use crate::running_state::get_running_state_mut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorStatus {
    WifiConnecting,
    WifiConnected(Ipv4Cidr),
    ServerConnecting,
    ServerConnected,
    Active,
}

type IndicatorSender = embassy_sync::channel::Sender<
    'static,
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    IndicatorStatus,
    8,
>;

type IndicatorChannel = embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    IndicatorStatus,
    8,
>;

type IndicatorReceiver = embassy_sync::channel::Receiver<
    'static,
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    IndicatorStatus,
    8,
>;

cfg_if::cfg_if! {
    if #[cfg(feature = "led")] {
        // LED Indicator
        mod led_indicator;
        use led_indicator::{start_indicator, IndicatorConfig};
    }
    else if #[cfg(feature = "smartled")] {
        // SmartLED/NeoPixel Indicator
        mod smartled_indicator;
        use smartled_indicator::{start_indicator, IndicatorConfig};
    }
    else if #[cfg(feature = "graphics")] {
        // LCD Graphical Indicator
        mod graphical_indicator;
        use graphical_indicator::{start_indicator, IndicatorConfig};
    }
    else {
        // Dummy Indicator
        mod dummy_indicator;
        use dummy_indicator::{start_indicator, IndicatorConfig};
    }
}

#[embassy_executor::task]
async fn indicator_task(config: IndicatorConfig, receiver: IndicatorReceiver) {
    start_indicator(config, receiver).await;
}

static INDICATOR_SENDER: OnceLock<IndicatorSender> = OnceLock::new();

pub async fn start_indicator_task(spawner: embassy_executor::Spawner) {
    info!("Starting indicator task...");
    let channel = crate::mk_static!(IndicatorChannel, IndicatorChannel::new());
    let receiver = channel.receiver();
    let sender: IndicatorSender = channel.sender();

    let config = IndicatorConfig::default();

    spawner.spawn(indicator_task(config, receiver)).ok();
    get_running_state_mut().await.server_connected = false;
    INDICATOR_SENDER.init(sender).ok();
    info!("Indicator task started.");
}

pub async fn set_indicator_status(status: IndicatorStatus) {
    match status {
        IndicatorStatus::WifiConnecting => {
            let mut guard = get_running_state_mut().await;
            guard.ip_address = None;
            guard.server_connected = false;
            guard.active = false;
        }
        IndicatorStatus::WifiConnected(ip_address) => {
            let mut guard = get_running_state_mut().await;
            guard.ip_address = Some(ip_address);
            guard.server_connected = false;
            guard.active = false;
        }
        IndicatorStatus::ServerConnecting => {
            let mut guard = get_running_state_mut().await;
            guard.server_connected = false;
            guard.active = false;
        }
        IndicatorStatus::ServerConnected => {
            let mut guard = get_running_state_mut().await;
            guard.server_connected = true;
            guard.active = false;
        }
        IndicatorStatus::Active => {
            let mut guard = get_running_state_mut().await;
            guard.server_connected = true;
            guard.active = true;
        }
    }
    INDICATOR_SENDER.get().await.try_send(status).ok();
}
