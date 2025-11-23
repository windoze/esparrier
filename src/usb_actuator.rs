use log::{debug, info, warn};

use crate::{
    Actuator, AppConfig, BarrierError, HidReport, IndicatorStatus, send_hid_report,
    set_indicator_status,
    synergy_hid::{ReportType, SynergyHid, modifier_mask_to_synergy},
};

#[cfg(feature = "ble")]
use crate::ble::publish_report as publish_ble_report;

pub struct UsbActuator {
    width: u16,
    height: u16,
    x: u16,
    y: u16,
    hid: SynergyHid,
}

impl UsbActuator {
    pub fn new() -> Self {
        Self {
            width: AppConfig::get().screen_width,
            height: AppConfig::get().screen_height,
            x: 0,
            y: 0,
            hid: SynergyHid::new(AppConfig::get().flip_wheel),
        }
    }

    async fn send_report(&mut self, report: (ReportType, &[u8])) {
        match report.0 {
            ReportType::Keyboard => {
                send_hid_report(HidReport::Keyboard(report.1.try_into().unwrap())).await;
            }
            ReportType::Mouse => {
                send_hid_report(HidReport::Mouse(report.1.try_into().unwrap())).await;
            }
            ReportType::Consumer => {
                send_hid_report(HidReport::Consumer(report.1.try_into().unwrap())).await;
            }
        }
        #[cfg(feature = "ble")]
        publish_ble_report(report);
    }
}

impl Default for UsbActuator {
    fn default() -> Self {
        Self::new()
    }
}

impl Actuator for UsbActuator {
    async fn connected(&mut self) -> Result<(), BarrierError> {
        info!("Connected to Barrier");
        set_indicator_status(IndicatorStatus::ServerConnected).await;
        Ok(())
    }

    async fn disconnected(&mut self) -> Result<(), BarrierError> {
        warn!("Disconnected from Barrier");
        set_indicator_status(IndicatorStatus::ServerConnected).await;
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
        debug!("Key repeat on key: {key}, mask: {mask}, button: {button}, count: {count}");
        Ok(())
    }

    async fn key_up(&mut self, key: u16, mask: u16, button: u16) -> Result<(), BarrierError> {
        let mut report = [0; 9];
        let ret = self.hid.key_up(key, mask, button, &mut report);
        self.send_report(ret).await;
        Ok(())
    }

    async fn jiggle(&mut self) -> Result<(), BarrierError> {
        debug!("Jiggle the host");
        if self.hid.is_empty() {
            let mut report = [0; 9];
            let ret = self
                .hid
                .set_cursor_position(self.x + 1, self.y, &mut report);
            self.send_report(ret).await;
            let ret = self.hid.set_cursor_position(self.x, self.y, &mut report);
            self.send_report(ret).await;
        }
        Ok(())
    }

    #[cfg(feature = "clipboard")]
    async fn set_clipboard(
        &mut self,
        data: heapless::Vec<u8, { crate::constants::MAX_CLIPBOARD_SIZE }>,
    ) -> Result<(), BarrierError> {
        crate::clipboard::set_clipboard(data).await;
        Ok(())
    }

    async fn enter(&mut self, x: u16, y: u16, mask: u16) -> Result<(), BarrierError> {
        info!("Entering, x: {x}, y: {y}, mask: {mask:#018b}");
        // Server sends cursor position on entering, client should move the cursor
        self.set_cursor_position(x, y).await?;
        // Server sends modifier mask on entering, client should press the keys
        let mut modifiers = [0u16; 16];
        let mods = modifier_mask_to_synergy(mask, &mut modifiers);
        for key in mods {
            self.key_down(*key, 0, 0).await?;
        }
        set_indicator_status(IndicatorStatus::Active).await;
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
        set_indicator_status(IndicatorStatus::ServerConnected).await;
        Ok(())
    }
}
