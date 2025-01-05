use super::IndicatorReceiver;

#[derive(Default)]
pub struct IndicatorConfig {
    _dummy: u8,
}

pub async fn start_indicator(_config: IndicatorConfig, receiver: IndicatorReceiver) {
    loop {
        receiver.receive().await;
    }
}
