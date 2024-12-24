use log::{debug, info, warn};

use crate::{
    synergy_hid::{ReportType, SynergyHid},
    Actuator, BarrierError, ReportWriter,
};

pub struct UsbActuator<'a, 'b, 'c> {
    width: u16,
    height: u16,
    x: u16,
    y: u16,
    hid: SynergyHid,
    keyboard_writer: ReportWriter<'a, 8>,
    mouse_writer: ReportWriter<'b, 7>,
    consumer_writer: ReportWriter<'c, 2>,
}

impl<'a, 'b, 'c> UsbActuator<'a, 'b, 'c> {
    pub fn new(
        width: u16,
        height: u16,
        flip_mouse_wheel: bool,
        keyboard_writer: ReportWriter<'a, 8>,
        mouse_writer: ReportWriter<'b, 7>,
        consumer_writer: ReportWriter<'c, 2>,
    ) -> Self {
        Self {
            width,
            height,
            x: 0,
            y: 0,
            hid: SynergyHid::new(flip_mouse_wheel),
            keyboard_writer,
            mouse_writer,
            consumer_writer,
        }
    }

    async fn send_report(&mut self, report: (ReportType, &[u8])) {
        debug!("Sending report: {}, {:?}", report.0 as u8, report.1);
        match report.0 {
            ReportType::Keyboard => {
                self.keyboard_writer.write(report.1).await.ok();
            }
            ReportType::Mouse => {
                self.mouse_writer.write(report.1).await.ok();
            }
            ReportType::Consumer => {
                self.consumer_writer.write(report.1).await.ok();
            }
        }
    }
}

impl<'a, 'b, 'c> Actuator for UsbActuator<'a, 'b, 'c> {
    async fn connected(&mut self) -> Result<(), BarrierError> {
        info!("Connected to Barrier");
        // self.sender.send(IndicatorStatus::ServerConnected).await;
        Ok(())
    }

    async fn disconnected(&mut self) -> Result<(), BarrierError> {
        warn!("Disconnected from Barrier");
        // self.sender.send(IndicatorStatus::ServerDisconnected).await;
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

    async fn enter(&mut self) -> Result<(), BarrierError> {
        info!("Entering");
        // self.sender.send(IndicatorStatus::EnterScreen).await;
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
        // self.sender.send(IndicatorStatus::LeaveScreen).await;
        Ok(())
    }
}
