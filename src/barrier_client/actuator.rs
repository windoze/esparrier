use super::BarrierError;

pub trait Actuator {
    fn connected(&mut self) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn disconnected(
        &mut self,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn get_screen_size(
        &self,
    ) -> impl core::future::Future<Output = Result<(u16, u16), BarrierError>> + Send;

    fn get_cursor_position(
        &self,
    ) -> impl core::future::Future<Output = Result<(u16, u16), BarrierError>> + Send;

    fn set_cursor_position(
        &mut self,
        x: u16,
        y: u16,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn move_cursor(
        &mut self,
        x: i16,
        y: i16,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn mouse_down(
        &mut self,
        button: i8,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn mouse_up(
        &mut self,
        button: i8,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn mouse_wheel(
        &mut self,
        x: i16,
        y: i16,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn key_down(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn key_repeat(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
        count: u16,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn key_up(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
    ) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn enter(&mut self) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;

    fn leave(&mut self) -> impl core::future::Future<Output = Result<(), BarrierError>> + Send;
}
