#[rustfmt::skip]
pub const ABSOLUTE_WHEEL_MOUSE_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01,        // Usage Page (Generic Desktop),
    0x09, 0x02,        // Usage (Mouse),
    0xA1, 0x01,        // Collection (Application),
    0x09, 0x01,        //   Usage (Pointer),
    0xA1, 0x00,        //   Collection (Physical),

    0x05, 0x09,        //     Usage Page (Buttons),
    0x19, 0x01,        //     Usage Minimum (1),
    0x29, 0x08,        //     Usage Maximum (8),
    0x15, 0x00,        //     Logical Minimum (0),
    0x25, 0x01,        //     Logical Maximum (1),
    0x95, 0x08,        //     Report Count (8),
    0x75, 0x01,        //     Report Size (1),
    0x81, 0x02,        //     Input (Data, Variable, Absolute),

    0x05, 0x01,        //     Usage Page (Generic Desktop),
    0x09, 0x30,        //     Usage (X),
    0x09, 0x31,        //     Usage (Y),
    0x15, 0x00,        //     Logical Minimum (0),
    0x26, 0xFF, 0x7F,  //     Logical Maximum (32767),
    0x35, 0x00,        //     Physical Minimum (0),
    0x46, 0xFF, 0x7F,  //     Physical Maximum (32767),
    0x95, 0x02,        //     Report Count (2),
    0x75, 0x10,        //     Report Size (16),
    0x81, 0x02,        //     Input (Data, Variable, Absolute),

    0x09, 0x38,        //     Usage (Wheel)
    0x15, 0x81,        //     Logical Minimum (-127)
    0x25, 0x7F,        //     Logical Maximum (127)
    0x75, 0x08,        //     Report Size (8)
    0x95, 0x01,        //     Report Count (1)
    0x81, 0x06,        //     Input (Data,Var,Rel,No Wrap,Linear,Preferred State,No Null Position)

    0x05, 0x0C,        //     Usage Page (Consumer)
    0x0A, 0x38, 0x02,  //     Usage (AC Pan)
    0x75, 0x08,        //     Report Size (8)
    0x95, 0x01,        //     Report Count (1)
    0x15, 0x81,        //     Logical Minimum (-127)
    0x25, 0x7F,        //     Logical Maximum (127)
    0x81, 0x06,        //     Input (Data,Var,Rel,No Wrap,Linear,Preferred State,No Null Position)

    0xC0,              //   End Collection
    0xC0,              // End Collection
];

#[rustfmt::skip]
pub const BOOT_KEYBOARD_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01,        // Usage Page (Generic Desktop),
    0x09, 0x06,        // Usage (Keyboard),
    0xA1, 0x01,        // Collection (Application),
    0x75, 0x01,        //     Report Size (1),
    0x95, 0x08,        //     Report Count (8),
    0x05, 0x07,        //     Usage Page (Key Codes),
    0x19, 0xE0,        //     Usage Minimum (224),
    0x29, 0xE7,        //     Usage Maximum (231),
    0x15, 0x00,        //     Logical Minimum (0),
    0x25, 0x01,        //     Logical Maximum (1),
    0x81, 0x02,        //     Input (Data, Variable, Absolute), ;Modifier byte

    0x95, 0x01,        //     Report Count (1),
    0x75, 0x08,        //     Report Size (8),
    0x81, 0x01,        //     Input (Constant), ;Reserved byte

    0x95, 0x05,        //     Report Count (5),
    0x75, 0x01,        //     Report Size (1),
    0x05, 0x08,        //     Usage Page (LEDs),
    0x19, 0x01,        //     Usage Minimum (1),
    0x29, 0x05,        //     Usage Maximum (5),
    0x91, 0x02,        //     Output (Data, Variable, Absolute), ;LED report
    
    0x95, 0x01,        //     Report Count (1),
    0x75, 0x03,        //     Report Size (3),
    0x91, 0x01,        //     Output (Constant), ;LED report padding

    0x95, 0x06,        //     Report Count (6),
    0x75, 0x08,        //     Report Size (8),
    0x15, 0x00,        //     Logical Minimum (0),
    0x26, 0xFF, 0x00,  //     Logical Maximum(255),
    0x05, 0x07,        //     Usage Page (Key Codes),
    0x19, 0x00,        //     Usage Minimum (0),
    0x2A, 0xFF, 0x00,  //     Usage Maximum (255),
    0x81, 0x00,        //     Input (Data, Array),
    0xC0,              // End Collection
];

#[rustfmt::skip]
pub const CONSUMER_CONTROL_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x0C,        // Usage Page (Consumer),
    0x09, 0x01,        // Usage (Consumer Control),
    0xA1, 0x01,        // Collection (Application),
    0x75, 0x10,        //     Report Size(16)
    0x95, 0x01,        //     Report Count(1)
    0x15, 0x00,        //     Logical Minimum(0)
    0x26, 0xA0, 0x02,  //     Logical Maximum(0x02A0)
    0x19, 0x00,        //     Usage Minimum(0)
    0x2A, 0xA0, 0x02,  //     Usage Maximum(0x02A0)
    0x81, 0x00,        //     Input (Array, Data, Variable)
    0xC0,              // End Collection
];
