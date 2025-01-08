use core::{net::Ipv4Addr, sync::atomic::AtomicU8};

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, once_lock::OnceLock,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorStatus {
    WifiConnecting = 0,
    WifiConnected = 1,
    ServerConnected = 2,
    Active = 3,
}

type IndicatorSender = embassy_sync::channel::Sender<
    'static,
    embassy_sync::blocking_mutex::raw::NoopRawMutex,
    IndicatorStatus,
    3,
>;

type IndicatorChannel = embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::NoopRawMutex,
    IndicatorStatus,
    3,
>;

type IndicatorReceiver = embassy_sync::channel::Receiver<
    'static,
    embassy_sync::blocking_mutex::raw::NoopRawMutex,
    IndicatorStatus,
    3,
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
static CURRENT_STATUS: AtomicU8 = AtomicU8::new(0);

pub fn start_indicator_task(spawner: embassy_executor::Spawner) {
    let channel = crate::mk_static!(IndicatorChannel, IndicatorChannel::new());
    let receiver = channel.receiver();
    let sender: IndicatorSender = channel.sender();

    let config = IndicatorConfig::default();

    spawner.spawn(indicator_task(config, receiver)).ok();
    INDICATOR_SENDER.init(sender).ok();
}

pub async fn set_indicator_status(status: IndicatorStatus) {
    INDICATOR_SENDER.get().await.send(status).await;
    CURRENT_STATUS.store(status as u8, core::sync::atomic::Ordering::Relaxed);
}

pub fn get_indicator_status() -> IndicatorStatus {
    match CURRENT_STATUS.load(core::sync::atomic::Ordering::Relaxed) {
        0 => IndicatorStatus::WifiConnecting,
        1 => IndicatorStatus::WifiConnected,
        2 => IndicatorStatus::ServerConnected,
        3 => IndicatorStatus::Active,
        _ => IndicatorStatus::WifiConnecting,
    }
}

#[derive(Clone, Debug)]
pub struct RunningState {
    pub ip_address: Option<Ipv4Addr>,
    pub server_connected: bool,
}

impl RunningState {
    pub const fn new() -> Self {
        Self {
            ip_address: None,
            server_connected: false,
        }
    }

    pub fn set_ip_address(&self, ip_address: Option<Ipv4Addr>) -> Self {
        Self {
            ip_address,
            ..self.clone()
        }
    }

    pub fn set_server_connected(&self, server_connected: bool) -> Self {
        Self {
            server_connected,
            ..self.clone()
        }
    }
}

impl Default for RunningState {
    fn default() -> Self {
        Self::new()
    }
}

static RUNNING_STATE: Mutex<CriticalSectionRawMutex, RunningState> =
    Mutex::new(RunningState::new());

pub async fn get_running_state() -> RunningState {
    RUNNING_STATE.lock().await.clone()
}

pub async fn set_running_state(state: RunningState) {
    *RUNNING_STATE.lock().await = state;
}
