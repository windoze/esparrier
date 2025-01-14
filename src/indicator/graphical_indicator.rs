use embassy_time::{with_timeout, Duration};
use embedded_graphics::image::ImageDrawable;
use tinygif::Gif;

use crate::IndicatorStatus;

use super::IndicatorReceiver;

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3")] {
        mod m5atom_s3;
        pub use m5atom_s3::IndicatorConfig;
        use m5atom_s3::*;
    }
    else if #[cfg(feature = "m5atoms3r")] {
        mod m5atom_s3r;
        pub use m5atom_s3r::IndicatorConfig;
        use m5atom_s3r::*;
    } else {
        compile_error!("No graphical indicator for this board");
    }
}

const CONNECTING: &[u8] = include_bytes!("assets/connecting.gif");
const INACTIVE: &[u8] = include_bytes!("assets/inactive.gif");
const ACTIVE: &[u8] = include_bytes!("assets/active.gif");

pub async fn start_indicator(config: IndicatorConfig, receiver: IndicatorReceiver) {
    let mut display = init_display(config);

    let connecting_gif: Gif<'_, ColorFormat> = Gif::from_slice(CONNECTING).unwrap();
    let inactive_gif: Gif<'_, ColorFormat> = Gif::from_slice(INACTIVE).unwrap();
    let active_gif: Gif<'_, ColorFormat> = Gif::from_slice(ACTIVE).unwrap();

    let mut status = IndicatorStatus::WifiConnecting;

    loop {
        let gif = match status {
            IndicatorStatus::WifiConnecting => &connecting_gif,
            IndicatorStatus::WifiConnected(_) => &connecting_gif,
            IndicatorStatus::ServerConnecting => &connecting_gif,
            IndicatorStatus::ServerConnected => &inactive_gif,
            IndicatorStatus::Active => &active_gif,
        };
        if status == IndicatorStatus::Active {
            // Don't waste time on animation, just show the first frame and wait for the next status forever
            // The SPI bus is pretty slow, showing animation in the active state may cause jitter and lag when receiving data and sending HID reports.
            gif.frames().next().unwrap().draw(&mut display).unwrap();
            status = receiver.receive().await;
        } else {
            // Show the animation and wait for the next status, we can afford it because there is no user interaction in the connecting and inactive states.
            for frame in gif.frames() {
                frame.draw(&mut display).unwrap();
                if let Ok(s) = with_timeout(
                    Duration::from_millis(frame.delay_centis as u64),
                    receiver.receive(),
                )
                .await
                {
                    status = s;
                    break;
                }
            }
        }
    }
}
