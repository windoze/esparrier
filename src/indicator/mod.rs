use embassy_net::Ipv4Cidr;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, once_lock::OnceLock,
};
use log::info;

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

pub async fn start_indicator_task(spawner: embassy_executor::Spawner) {
    info!("Starting indicator task...");
    let channel = crate::mk_static!(IndicatorChannel, IndicatorChannel::new());
    let receiver = channel.receiver();
    let sender: IndicatorSender = channel.sender();

    let config = IndicatorConfig::default();

    spawner.spawn(indicator_task(config, receiver)).ok();
    RUNNING_STATE.lock().await.server_connected = false;
    INDICATOR_SENDER.init(sender).ok();
    info!("Indicator task started.");
}

pub async fn set_indicator_status(status: IndicatorStatus) {
    match status {
        IndicatorStatus::WifiConnecting => {
            let mut guard = RUNNING_STATE.lock().await;
            guard.ip_address = None;
            guard.server_connected = false;
            guard.active = false;
        }
        IndicatorStatus::WifiConnected(ip_address) => {
            let mut guard = RUNNING_STATE.lock().await;
            guard.ip_address = Some(ip_address);
            guard.server_connected = false;
            guard.active = false;
        }
        IndicatorStatus::ServerConnecting => {
            let mut guard = RUNNING_STATE.lock().await;
            guard.server_connected = false;
            guard.active = false;
        }
        IndicatorStatus::ServerConnected => {
            let mut guard = RUNNING_STATE.lock().await;
            guard.server_connected = true;
            guard.active = false;
        }
        IndicatorStatus::Active => {
            let mut guard = RUNNING_STATE.lock().await;
            guard.server_connected = true;
            guard.active = true;
        }
    }
    INDICATOR_SENDER.get().await.try_send(status).ok();
}

#[derive(Clone, Debug)]
pub struct RunningState {
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    pub feature_flags: u8,
    pub ip_address: Option<Ipv4Cidr>,
    pub server_connected: bool,
    pub active: bool,
}

impl RunningState {
    pub const fn new() -> Self {
        Self {
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            version_patch: VERSION_PATCH,
            feature_flags: FEATURE_FLAGS,
            ip_address: None,
            server_connected: false,
            active: false,
        }
    }

    pub fn set_ip_address(&self, ip_address: Option<Ipv4Cidr>) -> Self {
        Self {
            ip_address,
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

const VERSION_SEGMENTS: [&str; 3] = const_str::split!(env!("CARGO_PKG_VERSION"), ".");
const VERSION_MAJOR: u8 = const_str::parse!(VERSION_SEGMENTS[0], u8);
const VERSION_MINOR: u8 = const_str::parse!(VERSION_SEGMENTS[1], u8);
const VERSION_PATCH: u8 = const_str::parse!(VERSION_SEGMENTS[2], u8);

cfg_if::cfg_if! {
    if #[cfg(feature = "led")] {
        const INDICATOR_FLAGS: u8 = 0b0000_0001;
    }
    else if #[cfg(feature = "smartled")] {
        const INDICATOR_FLAGS: u8 = 0b0000_0010;
    }
    else if #[cfg(feature = "graphics")] {
        const INDICATOR_FLAGS: u8 = 0b0000_0100;
    }
    else {
        const INDICATOR_FLAGS: u8 = 0b0000_0000;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "clipboard")] {
        const CLIPBOARD_FLAG: u8 = 0b1000_0000;
    }
    else {
        const CLIPBOARD_FLAG: u8 = 0b0000_0000;
    }
}

const FEATURE_FLAGS: u8 = INDICATOR_FLAGS | CLIPBOARD_FLAG;
