#[derive(Debug, Default)]
pub struct MouseReport {
    button: u8,
}

impl MouseReport {
    pub fn mouse_move(&mut self, dx: i16, dy: i16) -> [u8; 7] {
        self.send(dx, dy, None, None)
    }

    pub fn mouse_down(&mut self, button: u8) -> [u8; 7] {
        self.button |= button;
        self.send(0, 0, None, None)
    }

    pub fn mouse_up(&mut self, button: u8) -> [u8; 7] {
        self.button &= !button;
        self.send(0, 0, None, None)
    }

    pub fn mouse_wheel(&mut self, scroll: i8, pan: i8) -> [u8; 7] {
        self.send(0, 0, scroll, pan)
    }

    pub fn clear(&mut self) -> [u8; 7] {
        self.button = 0;
        self.send(0, 0, None, None)
    }

    pub fn is_empty(&self) -> bool {
        self.button == 0
    }

    fn send<S: Into<Option<i8>>, P: Into<Option<i8>>>(&self, dx: i16, dy: i16, scroll: S, pan: P) -> [u8; 7] {
        let scroll = scroll.into().unwrap_or(0);
        let pan = pan.into().unwrap_or(0);
        let mut report = [0u8; 7];
        report[0] = self.button;
        report[1] = (dx & 0xff) as u8;
        report[2] = (dx >> 8) as u8;
        report[3] = (dy & 0xff) as u8;
        report[4] = (dy >> 8) as u8;
        report[5] = scroll as u8;
        report[6] = pan as u8;
        report
    }
}

#[derive(Debug, Default)]
pub struct KeyboardReport {
    modifier: u8,
    keycode: [u8; 6],
}

impl KeyboardReport {
    pub fn press(&mut self, key: u8) -> [u8; 8] {
        match self.get_modifier(key) {
            Some(modifier) => self.modifier |= modifier,
            None => {
                // Don't add the same key twice
                for i in 0..6 {
                    if self.keycode[i] == key {
                        return self.send();
                    }
                }

                let mut found = false;
                for i in 0..6 {
                    if self.keycode[i] == 0 {
                        self.keycode[i] = key;
                        found = true;
                        break;
                    }
                }
                if !found {
                    // roll over the first key
                    for i in 1..6 {
                        self.keycode.swap(i - 1, i);
                    }
                    self.keycode[6 - 1] = key;
                }
            }
        }
        self.send()
    }

    pub fn release(&mut self, key: u8) -> [u8; 8] {
        match self.get_modifier(key) {
            Some(modifier) => self.modifier &= !modifier,
            None => {
                for i in 0..6 {
                    if self.keycode[i] == key {
                        self.keycode[i] = 0;
                        break;
                    }
                }
                // Compact the keycode array
                let mut pos = 0;
                for i in 0..6 {
                    if self.keycode[i] != 0 {
                        self.keycode.swap(i, pos);
                        pos += 1;
                    }
                }
            }
        }
        self.send()
    }

    pub fn clear(&mut self) -> [u8; 8] {
        self.modifier = 0;
        self.keycode = [0; 6];
        self.send()
    }

    pub fn is_empty(&self) -> bool {
        self.modifier == 0 && self.keycode.iter().all(|&x| x == 0)
    }

    fn send(&self) -> [u8; 8] {
        let mut report = [0u8; 8];
        report[0] = self.modifier;
        report[1] = 0;
        report[2..(6 + 2)].copy_from_slice(&self.keycode);
        report
    }

    fn get_modifier(&self, key: u8) -> Option<u8> {
        match key {
            0xE0 => Some(0x01), // Left Control
            0xE1 => Some(0x02), // Left Shift
            0xE2 => Some(0x04), // Left Alt
            0xE3 => Some(0x08), // Left GUI
            0xE4 => Some(0x10), // Right Control
            0xE5 => Some(0x20), // Right Shift
            0xE6 => Some(0x40), // Right Alt
            0xE7 => Some(0x80), // Right GUI
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct ConsumerReport {
    code: u16,
}

impl ConsumerReport {
    pub fn press(&mut self, code: u16) -> [u8; 2] {
        self.code = code;
        self.send()
    }

    pub fn release(&mut self) -> [u8; 2] {
        self.code = 0;
        self.send()
    }

    pub fn clear(&mut self) -> [u8; 2] {
        self.code = 0;
        self.send()
    }

    pub fn is_empty(&self) -> bool {
        self.code == 0
    }

    fn send(&self) -> [u8; 2] {
        let mut report = [0u8; 2];
        report[0] = (self.code & 0xff) as u8;
        report[1] = (self.code >> 8) as u8;
        report
    }
}
