use log::{debug, info, warn};

use crate::{
    synergy_hid::{ReportType, SynergyHid},
    Actuator, BarrierError, HidReport, HidReportSender, IndicatorSender, IndicatorStatus,
};

#[cfg(feature = "clipboard")]
static CLIPBOARD_STORAGE: embassy_sync::mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    Option<heapless::Vec<u8, 1024>>,
> = embassy_sync::mutex::Mutex::new(None);

#[cfg(feature = "clipboard")]
pub async fn send_clipboard(hid_writer: HidReportSender) {
    let data = CLIPBOARD_STORAGE.lock().await.clone();
    if let Some(data) = data {
        let data = data.as_slice();
        debug!(
            "Send clipboard: {:?}",
            &data[0..core::cmp::min(data.len(), 16)]
        );
    }
}

pub struct UsbActuator {
    width: u16,
    height: u16,
    x: u16,
    y: u16,
    hid: SynergyHid,
    indicator: IndicatorSender,
    hid_writer: HidReportSender,
}

impl UsbActuator {
    pub fn new(
        app_config: &crate::AppConfig,
        indicator: IndicatorSender,
        hid_writer: HidReportSender,
    ) -> Self {
        Self {
            width: app_config.screen_width,
            height: app_config.screen_height,
            x: 0,
            y: 0,
            hid: SynergyHid::new(app_config.flip_wheel),
            indicator,
            hid_writer,
        }
    }

    async fn send_report(&mut self, report: (ReportType, &[u8])) {
        match report.0 {
            ReportType::Keyboard => {
                self.hid_writer
                    .send(HidReport::Keyboard(report.1.try_into().unwrap()))
                    .await;
            }
            ReportType::Mouse => {
                self.hid_writer
                    .send(HidReport::Mouse(report.1.try_into().unwrap()))
                    .await;
            }
            ReportType::Consumer => {
                self.hid_writer
                    .send(HidReport::Consumer(report.1.try_into().unwrap()))
                    .await;
            }
        }
    }
}

impl Actuator for UsbActuator {
    async fn connected(&mut self) -> Result<(), BarrierError> {
        info!("Connected to Barrier");
        self.indicator.send(IndicatorStatus::ServerConnected).await;
        Ok(())
    }

    async fn disconnected(&mut self) -> Result<(), BarrierError> {
        warn!("Disconnected from Barrier");
        self.indicator.send(IndicatorStatus::WifiConnected).await;
        Ok(())
    }

    async fn get_screen_size(&self) -> Result<(u16, u16), BarrierError> {
        // TODO:
        Ok((self.width, self.height))
    }

    async fn get_cursor_position(&self) -> Result<(u16, u16), BarrierError> {
        Ok((self.x, self.y))
    }

    async fn set_cursor_position(&mut self, x: u16, y: u16) -> Result<(), BarrierError> {
        self.x = x;
        self.y = y;
        let mut report = [0; 9];
        let ret = self.hid.set_cursor_position(x, y, &mut report);
        self.send_report(ret).await;
        Ok(())
    }

    async fn move_cursor(&mut self, x: i16, y: i16) -> Result<(), BarrierError> {
        let (cx, cy) = self.get_cursor_position().await?;
        self.set_cursor_position((cx as i32 + x as i32) as u16, (cy as i32 + y as i32) as u16)
            .await
    }

    async fn mouse_down(&mut self, button: i8) -> Result<(), BarrierError> {
        let mut report = [0; 9];
        let ret = self.hid.mouse_down(button, &mut report);
        self.send_report(ret).await;
        Ok(())
    }

    async fn mouse_up(&mut self, button: i8) -> Result<(), BarrierError> {
        let mut report = [0; 9];
        let ret = self.hid.mouse_up(button, &mut report);
        self.send_report(ret).await;
        Ok(())
    }

    async fn mouse_wheel(&mut self, x: i16, y: i16) -> Result<(), BarrierError> {
        let mut report = [0; 9];
        let ret = self.hid.mouse_scroll(x, y, &mut report);
        self.send_report(ret).await;
        Ok(())
    }

    async fn key_down(&mut self, key: u16, mask: u16, button: u16) -> Result<(), BarrierError> {
        let mut report = [0; 9];
        let ret = self.hid.key_down(key, mask, button, &mut report);
        self.send_report(ret).await;
        Ok(())
    }

    async fn key_repeat(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
        count: u16,
    ) -> Result<(), BarrierError> {
        debug!(
            "Key repeat on key: {}, mask: {}, button: {}, count: {}",
            key, mask, button, count
        );
        Ok(())
    }

    async fn key_up(&mut self, key: u16, mask: u16, button: u16) -> Result<(), BarrierError> {
        let mut report = [0; 9];
        let ret = self.hid.key_up(key, mask, button, &mut report);
        self.send_report(ret).await;
        Ok(())
    }

    #[cfg(feature = "clipboard")]
    async fn set_clipboard(&mut self, data: heapless::Vec<u8, 1024>) -> Result<(), BarrierError> {
        debug!(
            "Set clipboard: length: {}, data: {:?}",
            data.len(),
            &data[0..core::cmp::min(data.len(), 16)]
        );
        CLIPBOARD_STORAGE.lock().await.replace(data);
        Ok(())
    }

    async fn enter(&mut self) -> Result<(), BarrierError> {
        info!("Entering");
        self.indicator.send(IndicatorStatus::Active).await;
        Ok(())
    }

    async fn leave(&mut self) -> Result<(), BarrierError> {
        info!("Leaving");
        let mut report = [0; 9];
        let ret = self.hid.clear(ReportType::Keyboard, &mut report);
        self.send_report(ret).await;
        let ret = self.hid.clear(ReportType::Mouse, &mut report);
        self.send_report(ret).await;
        let ret = self.hid.clear(ReportType::Consumer, &mut report);
        self.send_report(ret).await;
        self.indicator.send(IndicatorStatus::ServerConnected).await;
        Ok(())
    }
}
