#[cfg(not(feature = "indicator"))]
mod dummy_indicator;
#[cfg(feature = "graphics")]
mod graphical_indicator;
#[cfg(feature = "led")]
mod led_indicator;
#[cfg(feature = "smartled")]
mod smartled_indicator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorStatus {
    WifiConnecting,
    WifiConnected,
    ServerConnected,
    Active,
}

pub type IndicatorChannel = embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::NoopRawMutex,
    IndicatorStatus,
    3,
>;

pub type IndicatorSender = embassy_sync::channel::Sender<
    'static,
    embassy_sync::blocking_mutex::raw::NoopRawMutex,
    IndicatorStatus,
    3,
>;

pub type IndicatorReceiver = embassy_sync::channel::Receiver<
    'static,
    embassy_sync::blocking_mutex::raw::NoopRawMutex,
    IndicatorStatus,
    3,
>;

// LED Indicator
#[cfg(feature = "led")]
pub use led_indicator::{start_indicator, IndicatorConfig};

// SmartLED/NeoPixel Indicator
#[cfg(feature = "smartled")]
pub use smartled_indicator::{start_indicator, IndicatorConfig};

// LCD Graphical Indicator
#[cfg(feature = "graphics")]
pub use graphical_indicator::{start_indicator, IndicatorConfig};

#[cfg(not(feature = "indicator"))]
pub use dummy_indicator::{start_indicator, IndicatorConfig};

#[embassy_executor::task]
pub async fn indicator_task(config: IndicatorConfig, receiver: IndicatorReceiver) {
    start_indicator(config, receiver).await;
}
