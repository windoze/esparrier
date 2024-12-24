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

// Fallback indicator, no-op
#[cfg(all(not(feature = "smartled"), not(feature = "led")))]
pub async fn start_indicator(receiver: IndicatorReceiver) {
    loop {
        embassy_time::with_timeout(embassy_time::Duration::from_millis(100), receiver.receive())
            .await
            .ok();
    }
}

// LED Indicator
#[cfg(feature = "led")]
pub use led_indicator::start_indicator;

// SmartLED/NeoPixel Indicator
#[cfg(feature = "smartled")]
pub use smartled_indicator::start_indicator;
