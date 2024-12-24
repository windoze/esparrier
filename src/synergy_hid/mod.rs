mod descriptors;
mod hid;
mod keycodes;

pub(super) use hid::*;
pub(crate) use keycodes::{synergy_mouse_button, synergy_to_hid, KeyCode};

pub(super) use descriptors::{
    ABSOLUTE_WHEEL_MOUSE_REPORT_DESCRIPTOR, BOOT_KEYBOARD_REPORT_DESCRIPTOR,
    CONSUMER_CONTROL_REPORT_DESCRIPTOR,
};
use log::{debug, warn};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReportType {
    Keyboard = 1,
    Mouse = 2,
    Consumer = 3,
}

impl ReportType {
    pub const fn get_report_size(&self) -> usize {
        match self {
            Self::Keyboard => 8,
            Self::Mouse => 7,
            Self::Consumer => 2,
        }
    }

    pub const fn get_report_descriptor(&self) -> &'static [u8] {
        match *self {
            ReportType::Keyboard => BOOT_KEYBOARD_REPORT_DESCRIPTOR,
            ReportType::Mouse => ABSOLUTE_WHEEL_MOUSE_REPORT_DESCRIPTOR,
            ReportType::Consumer => CONSUMER_CONTROL_REPORT_DESCRIPTOR,
        }
    }
}

#[derive(Debug)]
pub struct SynergyHid {
    flip_mouse_wheel: bool,
    x: u16,
    y: u16,
    server_buttons: [u16; 512],

    // Report 1
    keyboard_report: KeyboardReport,
    // Report 2
    mouse_report: AbsMouseReport,
    // Report 3
    consumer_report: ConsumerReport,
}

impl SynergyHid {
    pub fn new(flip_mouse_wheel: bool) -> Self {
        Self {
            flip_mouse_wheel,
            x: 0,
            y: 0,
            server_buttons: [0; 512],
            keyboard_report: KeyboardReport::default(),
            mouse_report: AbsMouseReport::default(),
            consumer_report: ConsumerReport::default(),
        }
    }

    pub const fn get_report_descriptor(report_type: ReportType) -> (u8, &'static [u8]) {
        (
            report_type.get_report_size() as u8,
            report_type.get_report_descriptor(),
        )
    }

    pub fn key_down<'a>(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
        report: &'a mut [u8],
    ) -> (ReportType, &'a [u8]) {
        debug!("Key down {} {} {}", key, mask, button);
        self.server_buttons[button as usize] = key;
        let hid = synergy_to_hid(key);
        // debug!("Key Down {:#04x} -> Keycode: {:?}", key, hid);
        match hid {
            KeyCode::None => {
                warn!("Keycode not found");
                report[..8].copy_from_slice(&self.keyboard_report.clear());
                (ReportType::Keyboard, &report[0..8])
            }
            KeyCode::Key(key) => {
                report[..8].copy_from_slice(&self.keyboard_report.press(key));
                (ReportType::Keyboard, &report[0..8])
            }
            KeyCode::Consumer(key) => {
                report[..2].copy_from_slice(&self.consumer_report.press(key));
                (ReportType::Consumer, &report[0..2])
            }
        }
    }

    pub fn key_up<'a>(
        &mut self,
        key: u16,
        mask: u16,
        button: u16,
        report: &'a mut [u8],
    ) -> (ReportType, &'a [u8]) {
        debug!("Key up {} {} {}", key, mask, button);
        let key = self.server_buttons[button as usize];
        let hid = if self.server_buttons[button as usize] != 0 {
            // debug!("Key {key} up");
            self.server_buttons[button as usize] = 0;
            synergy_to_hid(key)
        } else if key == 0 {
            debug!("Key 0 up, clear all key down");
            KeyCode::None
        } else {
            // warn!("Key {key} up with no key down");
            KeyCode::None
        };
        // debug!("Key Down {:#04x} -> Keycode: {:?}", key, hid);
        match hid {
            KeyCode::None => {
                warn!("Keycode not found");
                report[..8].copy_from_slice(&self.keyboard_report.clear());
                (ReportType::Keyboard, &report[0..8])
            }
            KeyCode::Key(key) => {
                report[..8].copy_from_slice(&self.keyboard_report.release(key));
                (ReportType::Keyboard, &report[0..8])
            }
            KeyCode::Consumer(_key) => {
                report[..2].copy_from_slice(&self.consumer_report.release());
                (ReportType::Consumer, &report[0..2])
            }
        }
    }

    pub fn set_cursor_position<'a>(
        &mut self,
        x: u16,
        y: u16,
        report: &'a mut [u8],
    ) -> (ReportType, &'a [u8]) {
        (self.x, self.y) = (x, y);
        report[..7].copy_from_slice(&self.mouse_report.move_to(x, y));
        (ReportType::Mouse, &report[..7])
    }

    pub fn mouse_down<'a>(&mut self, button: i8, report: &'a mut [u8]) -> (ReportType, &'a [u8]) {
        report[..7].copy_from_slice(&self.mouse_report.mouse_down(synergy_mouse_button(button)));
        (ReportType::Mouse, &report[..7])
    }

    pub fn mouse_up<'a>(&mut self, button: i8, report: &'a mut [u8]) -> (ReportType, &'a [u8]) {
        report[..7].copy_from_slice(&self.mouse_report.mouse_up(synergy_mouse_button(button)));
        (ReportType::Mouse, &report[..7])
    }

    pub fn mouse_scroll<'a>(
        &mut self,
        x: i16,
        y: i16,
        report: &'a mut [u8],
    ) -> (ReportType, &'a [u8]) {
        let x = (x as f32 / 120.0) as i16;
        let y = (y as f32 / 120.0) as i16;
        let mut x = x as i8;
        let mut y = y as i8;
        if self.flip_mouse_wheel {
            x = -x;
            y = -y;
        }
        report[..7].copy_from_slice(&self.mouse_report.mouse_wheel(y, x));
        (ReportType::Mouse, &report[..7])
    }

    pub fn clear<'a>(
        &mut self,
        report_type: ReportType,
        report: &'a mut [u8],
    ) -> (ReportType, &'a [u8]) {
        match report_type {
            ReportType::Keyboard => {
                report[..8].copy_from_slice(&self.keyboard_report.clear());
                (ReportType::Keyboard, &report[..8])
            }
            ReportType::Mouse => {
                report[..7].copy_from_slice(&self.mouse_report.clear());
                (ReportType::Mouse, &report[..7])
            }
            ReportType::Consumer => {
                report[..2].copy_from_slice(&self.consumer_report.clear());
                (ReportType::Consumer, &report[..2])
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        keycodes::{HID_KEY_A, HID_KEY_B},
        ReportType,
    };

    #[test]
    fn test_key() {
        let mut hid = super::SynergyHid::new(false);
        let mut report = [0; 9];
        assert_eq!(
            hid.key_down(0x0000, 0x0000, 0x0000, &mut report),
            (ReportType::Keyboard, [0, 0, 0, 0, 0, 0, 0, 0].as_ref())
        );
        assert_eq!(
            hid.key_down('A' as u16, 0x0000, 0x0000, &mut report),
            (
                ReportType::Keyboard,
                [0, 0, HID_KEY_A, 0, 0, 0, 0, 0].as_ref()
            )
        );

        assert_eq!(
            hid.key_down('B' as u16, 0x0000, 0x0000, &mut report),
            (
                ReportType::Keyboard,
                [0, 0, HID_KEY_A, HID_KEY_B, 0, 0, 0, 0].as_ref()
            )
        );
        assert_eq!(
            hid.key_up('B' as u16, 0x0000, 0x0000, &mut report),
            (
                ReportType::Keyboard,
                [0, 0, HID_KEY_A, 0, 0, 0, 0, 0].as_ref()
            )
        );
        // Wrong key up, report is cleared
        assert_eq!(
            hid.key_up('C' as u16, 0x0000, 0x0000, &mut report),
            (ReportType::Keyboard, [0, 0, 0, 0, 0, 0, 0, 0].as_ref())
        );

        // kKeyAudioMute(0xE0AD) -> HID_USAGE_CONSUMER_MUTE(0x00E2)
        assert_eq!(
            hid.key_down(0xE0AD, 0x0000, 1, &mut report),
            (ReportType::Consumer, [0x00, 0xE2].as_ref())
        );
    }
}
