use embassy_time::Duration;
use log::{debug, info};

use crate::{constants::*, HidReport, HidReportSender};

static CLIPBOARD_STORAGE: embassy_sync::mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    Option<heapless::Vec<u8, MAX_CLIPBOARD_SIZE>>,
> = embassy_sync::mutex::Mutex::new(None);

const KEY_PRESS_INTERVAL: Duration = Duration::from_millis(5);

async fn send_clipboard(hid_writer: HidReportSender) {
    info!("Paste button pressed, sending clipboard...");
    let data = CLIPBOARD_STORAGE.lock().await.clone();
    if let Some(data) = data {
        debug!(
            "Clipboard (first 16 bytes): {:?}",
            &data.as_slice()[0..core::cmp::min(data.len(), 16)]
        );
        for byte in data {
            // Ignore non-ASCII characters
            if byte > 0x7F {
                continue;
            }
            let [k, m] = crate::synergy_hid::ASCII_2_HID[byte as usize];
            if k == 0 {
                continue;
            }
            let mut report = crate::synergy_hid::KeyboardReport::default();
            if m != 0 {
                // A key with a modifier
                // Press modifier key
                hid_writer.send(HidReport::keyboard(report.press(m))).await;
                embassy_time::Timer::after(KEY_PRESS_INTERVAL).await;
                // Press key
                hid_writer.send(HidReport::keyboard(report.press(k))).await;
                embassy_time::Timer::after(KEY_PRESS_INTERVAL).await;
                // Release key
                hid_writer
                    .send(HidReport::keyboard(report.release(k)))
                    .await;
                embassy_time::Timer::after(KEY_PRESS_INTERVAL).await;
                // Release modifier key
                hid_writer
                    .send(HidReport::keyboard(report.release(m)))
                    .await;
                embassy_time::Timer::after(KEY_PRESS_INTERVAL).await;
            } else {
                // A key without a modifier
                // Press key
                hid_writer.send(HidReport::keyboard(report.press(k))).await;
                embassy_time::Timer::after(KEY_PRESS_INTERVAL).await;
                // Release key
                hid_writer
                    .send(HidReport::keyboard(report.release(k)))
                    .await;
                embassy_time::Timer::after(KEY_PRESS_INTERVAL).await;
            }
        }
    }
}

pub async fn set_clipboard(data: heapless::Vec<u8, MAX_CLIPBOARD_SIZE>) {
    debug!(
        "Set clipboard: length: {}, data: {:?}",
        data.len(),
        &data[0..core::cmp::min(data.len(), 16)]
    );
    CLIPBOARD_STORAGE.lock().await.replace(data);
}

#[embassy_executor::task]
pub async fn button_task(button: esp_hal::gpio::AnyPin, hid_writer: HidReportSender) {
    use embedded_hal_async::digital::Wait;
    let input = esp_hal::gpio::Input::new(button, esp_hal::gpio::Pull::Up);
    let mut debouncer = async_debounce::Debouncer::new(input, Duration::from_millis(50));

    loop {
        debouncer.wait_for_rising_edge().await.ok();
        send_clipboard(hid_writer).await;
    }
}
