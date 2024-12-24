use log::{debug, info, warn};

use crate::Actuator;

#[derive(Debug)]
pub struct UsbActuator {
    screen_width: u16,
    screen_height: u16,
    cursor_x: u16,
    cursor_y: u16,
}

impl UsbActuator {
    pub fn new(screen_width: u16, screen_height: u16) -> Self {
        Self {
            screen_width,
            screen_height,
            cursor_x: 0,
            cursor_y: 0,
        }
    }
}

impl Actuator for UsbActuator {
    async fn connected(&mut self) -> Result<(), crate::BarrierError> {
        info!("Connected to Barrier server");
        Ok(())
    }

    async fn disconnected(&mut self) -> Result<(), crate::BarrierError> {
        warn!("Disconnected from Barrier server");
        Ok(())
    }

    async fn get_screen_size(&self) -> Result<(u16, u16), crate::BarrierError> {
        debug!("Getting screen size");
        Ok((self.screen_width, self.screen_height))
    }

    async fn get_cursor_position(&self) -> Result<(u16, u16), crate::BarrierError> {
        debug!("Getting cursor position");
        Ok((self.cursor_x, self.cursor_y))
    }

    async fn set_cursor_position(&mut self, x: u16, y: u16) -> Result<(), crate::BarrierError> {
        debug!("Setting cursor position to ({}, {})", x, y);
        self.cursor_x = x;
        self.cursor_y = y;
        Ok(())
    }

    async fn move_cursor(&mut self, x: i16, y: i16) -> Result<(), crate::BarrierError> {
        debug!("Moving cursor by ({}, {})", x, y);
        self.cursor_x = self.cursor_x.wrapping_add(x as u16);
        self.cursor_y = self.cursor_y.wrapping_add(y as u16);
        Ok(())
    }

    async fn mouse_down(&mut self, button: i8) -> Result<(), crate::BarrierError> {
        debug!("Mouse down on button {}", button);
        Ok(())
    }

    async fn mouse_up(&mut self, button: i8) -> Result<(), crate::BarrierError> {
        debug!("Mouse up on button {}", button);
        Ok(())
    }

    async fn mouse_wheel(&mut self, x: i16, y: i16) -> Result<(), crate::BarrierError> {
        debug!("Mouse wheel by ({}, {})", x, y);
        Ok(())
    }

    async fn key_down(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
    ) -> Result<(), crate::BarrierError> {
        debug!("Key down on key {}, mask {}, button {}", key, mask, button);
        Ok(())
    }

    async fn key_repeat(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
        count: u16,
    ) -> Result<(), crate::BarrierError> {
        debug!(
            "Key repeat on key {}, mask {}, button {}, count {}",
            key, mask, button, count
        );
        Ok(())
    }

    async fn key_up(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
    ) -> Result<(), crate::BarrierError> {
        debug!("Key up on key {}, mask {}, button {}", key, mask, button);
        Ok(())
    }

    async fn enter(&mut self) -> Result<(), crate::BarrierError> {
        info!("Entering");
        Ok(())
    }

    async fn leave(&mut self) -> Result<(), crate::BarrierError> {
        info!("Leaving");
        Ok(())
    }
}
