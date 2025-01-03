use super::IndicatorReceiver;

pub struct IndicatorConfig;

pub async fn start_indicator(_config: IndicatorConfig, receiver: IndicatorReceiver) {
    loop {
        receiver.receive().await;
    }
}
