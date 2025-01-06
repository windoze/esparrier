#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorStatus {
    WifiConnecting,
    WifiConnected,
    ServerConnected,
    Active,
}

pub type IndicatorSender = embassy_sync::channel::Sender<
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

pub fn start_indicator_task(spawner: embassy_executor::Spawner) -> IndicatorSender {
    let channel = crate::mk_static!(IndicatorChannel, IndicatorChannel::new());
    let receiver = channel.receiver();
    let sender: IndicatorSender = channel.sender();

    let config = IndicatorConfig::default();

    spawner.spawn(indicator_task(config, receiver)).ok();
    sender
}
