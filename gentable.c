/*
 * barrier -- mouse and keyboard sharing utility
 * Copyright (C) 2012-2016 Symless Ltd.
 * Copyright (C) 2004 Chris Schoeneman
 * Copyright (C) 2024 Chen Xu
 *
 * This package is free software; you can redistribute it and/or
 * modify it under the terms of the GNU General Public License
 * found in the file LICENSE that should have accompanied this file.
 *
 * This package is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#include <stdint.h>
#include <stdio.h>

#define HID_KEY_NONE 0x00
#define HID_KEY_A 0x04
#define HID_KEY_B 0x05
#define HID_KEY_C 0x06
#define HID_KEY_D 0x07
#define HID_KEY_E 0x08
#define HID_KEY_F 0x09
#define HID_KEY_G 0x0A
#define HID_KEY_H 0x0B
#define HID_KEY_I 0x0C
#define HID_KEY_J 0x0D
#define HID_KEY_K 0x0E
#define HID_KEY_L 0x0F
#define HID_KEY_M 0x10
#define HID_KEY_N 0x11
#define HID_KEY_O 0x12
#define HID_KEY_P 0x13
#define HID_KEY_Q 0x14
#define HID_KEY_R 0x15
#define HID_KEY_S 0x16
#define HID_KEY_T 0x17
#define HID_KEY_U 0x18
#define HID_KEY_V 0x19
#define HID_KEY_W 0x1A
#define HID_KEY_X 0x1B
#define HID_KEY_Y 0x1C
#define HID_KEY_Z 0x1D
#define HID_KEY_1 0x1E
#define HID_KEY_2 0x1F
#define HID_KEY_3 0x20
#define HID_KEY_4 0x21
#define HID_KEY_5 0x22
#define HID_KEY_6 0x23
#define HID_KEY_7 0x24
#define HID_KEY_8 0x25
#define HID_KEY_9 0x26
#define HID_KEY_0 0x27
#define HID_KEY_ENTER 0x28
#define HID_KEY_ESCAPE 0x29
#define HID_KEY_BACKSPACE 0x2A
#define HID_KEY_TAB 0x2B
#define HID_KEY_SPACE 0x2C
#define HID_KEY_MINUS 0x2D
#define HID_KEY_EQUAL 0x2E
#define HID_KEY_BRACKET_LEFT 0x2F
#define HID_KEY_BRACKET_RIGHT 0x30
#define HID_KEY_BACKSLASH 0x31
#define HID_KEY_EUROPE_1 0x32
#define HID_KEY_SEMICOLON 0x33
#define HID_KEY_APOSTROPHE 0x34
#define HID_KEY_GRAVE 0x35
#define HID_KEY_COMMA 0x36
#define HID_KEY_PERIOD 0x37
#define HID_KEY_SLASH 0x38
#define HID_KEY_CAPS_LOCK 0x39
#define HID_KEY_F1 0x3A
#define HID_KEY_F2 0x3B
#define HID_KEY_F3 0x3C
#define HID_KEY_F4 0x3D
#define HID_KEY_F5 0x3E
#define HID_KEY_F6 0x3F
#define HID_KEY_F7 0x40
#define HID_KEY_F8 0x41
#define HID_KEY_F9 0x42
#define HID_KEY_F10 0x43
#define HID_KEY_F11 0x44
#define HID_KEY_F12 0x45
#define HID_KEY_PRINT_SCREEN 0x46
#define HID_KEY_SCROLL_LOCK 0x47
#define HID_KEY_PAUSE 0x48
#define HID_KEY_INSERT 0x49
#define HID_KEY_HOME 0x4A
#define HID_KEY_PAGE_UP 0x4B
#define HID_KEY_DELETE 0x4C
#define HID_KEY_END 0x4D
#define HID_KEY_PAGE_DOWN 0x4E
#define HID_KEY_ARROW_RIGHT 0x4F
#define HID_KEY_ARROW_LEFT 0x50
#define HID_KEY_ARROW_DOWN 0x51
#define HID_KEY_ARROW_UP 0x52
#define HID_KEY_NUM_LOCK 0x53
#define HID_KEY_KEYPAD_DIVIDE 0x54
#define HID_KEY_KEYPAD_MULTIPLY 0x55
#define HID_KEY_KEYPAD_SUBTRACT 0x56
#define HID_KEY_KEYPAD_ADD 0x57
#define HID_KEY_KEYPAD_ENTER 0x58
#define HID_KEY_KEYPAD_1 0x59
#define HID_KEY_KEYPAD_2 0x5A
#define HID_KEY_KEYPAD_3 0x5B
#define HID_KEY_KEYPAD_4 0x5C
#define HID_KEY_KEYPAD_5 0x5D
#define HID_KEY_KEYPAD_6 0x5E
#define HID_KEY_KEYPAD_7 0x5F
#define HID_KEY_KEYPAD_8 0x60
#define HID_KEY_KEYPAD_9 0x61
#define HID_KEY_KEYPAD_0 0x62
#define HID_KEY_KEYPAD_DECIMAL 0x63
#define HID_KEY_EUROPE_2 0x64
#define HID_KEY_APPLICATION 0x65
#define HID_KEY_POWER 0x66
#define HID_KEY_KEYPAD_EQUAL 0x67
#define HID_KEY_F13 0x68
#define HID_KEY_F14 0x69
#define HID_KEY_F15 0x6A
#define HID_KEY_F16 0x6B
#define HID_KEY_F17 0x6C
#define HID_KEY_F18 0x6D
#define HID_KEY_F19 0x6E
#define HID_KEY_F20 0x6F
#define HID_KEY_F21 0x70
#define HID_KEY_F22 0x71
#define HID_KEY_F23 0x72
#define HID_KEY_F24 0x73
#define HID_KEY_EXECUTE 0x74
#define HID_KEY_HELP 0x75
#define HID_KEY_MENU 0x76
#define HID_KEY_SELECT 0x77
#define HID_KEY_STOP 0x78
#define HID_KEY_AGAIN 0x79
#define HID_KEY_UNDO 0x7A
#define HID_KEY_CUT 0x7B
#define HID_KEY_COPY 0x7C
#define HID_KEY_PASTE 0x7D
#define HID_KEY_FIND 0x7E
#define HID_KEY_MUTE 0x7F
#define HID_KEY_VOLUME_UP 0x80
#define HID_KEY_VOLUME_DOWN 0x81
#define HID_KEY_LOCKING_CAPS_LOCK 0x82
#define HID_KEY_LOCKING_NUM_LOCK 0x83
#define HID_KEY_LOCKING_SCROLL_LOCK 0x84
#define HID_KEY_KEYPAD_COMMA 0x85
#define HID_KEY_KEYPAD_EQUAL_SIGN 0x86
#define HID_KEY_KANJI1 0x87
#define HID_KEY_KANJI2 0x88
#define HID_KEY_KANJI3 0x89
#define HID_KEY_KANJI4 0x8A
#define HID_KEY_KANJI5 0x8B
#define HID_KEY_KANJI6 0x8C
#define HID_KEY_KANJI7 0x8D
#define HID_KEY_KANJI8 0x8E
#define HID_KEY_KANJI9 0x8F
#define HID_KEY_LANG1 0x90
#define HID_KEY_LANG2 0x91
#define HID_KEY_LANG3 0x92
#define HID_KEY_LANG4 0x93
#define HID_KEY_LANG5 0x94
#define HID_KEY_LANG6 0x95
#define HID_KEY_LANG7 0x96
#define HID_KEY_LANG8 0x97
#define HID_KEY_LANG9 0x98
#define HID_KEY_ALTERNATE_ERASE 0x99
#define HID_KEY_SYSREQ_ATTENTION 0x9A
#define HID_KEY_CANCEL 0x9B
#define HID_KEY_CLEAR 0x9C
#define HID_KEY_PRIOR 0x9D
#define HID_KEY_RETURN 0x9E
#define HID_KEY_SEPARATOR 0x9F
#define HID_KEY_OUT 0xA0
#define HID_KEY_OPER 0xA1
#define HID_KEY_CLEAR_AGAIN 0xA2
#define HID_KEY_CRSEL_PROPS 0xA3
#define HID_KEY_EXSEL 0xA4
// RESERVED					                      0xA5-DF
#define HID_KEY_CONTROL_LEFT 0xE0
#define HID_KEY_SHIFT_LEFT 0xE1
#define HID_KEY_ALT_LEFT 0xE2
#define HID_KEY_GUI_LEFT 0xE3
#define HID_KEY_CONTROL_RIGHT 0xE4
#define HID_KEY_SHIFT_RIGHT 0xE5
#define HID_KEY_ALT_RIGHT 0xE6
#define HID_KEY_GUI_RIGHT 0xE7

// Generic Control
enum {
  // Generic Control
  HID_USAGE_CONSUMER_CONTROL = 0x0001,

  // Power Control
  HID_USAGE_CONSUMER_POWER = 0x0030,
  HID_USAGE_CONSUMER_RESET = 0x0031,
  HID_USAGE_CONSUMER_SLEEP = 0x0032,

  // Screen Brightness
  HID_USAGE_CONSUMER_BRIGHTNESS_INCREMENT = 0x006F,
  HID_USAGE_CONSUMER_BRIGHTNESS_DECREMENT = 0x0070,

  // Keyboard Backlight Brightness
  HID_USAGE_CONSUMER_KEYBOARD_BRIGHTNESS_INCREMENT = 0x0079,
  HID_USAGE_CONSUMER_KEYBOARD_BRIGHTNESS_DECREMENT = 0x007A,

  // These HID usages operate only on mobile systems (battery powered) and
  // require Windows 8 (build 8302 or greater).
  HID_USAGE_CONSUMER_WIRELESS_RADIO_CONTROLS = 0x000C,
  HID_USAGE_CONSUMER_WIRELESS_RADIO_BUTTONS = 0x00C6,
  HID_USAGE_CONSUMER_WIRELESS_RADIO_LED = 0x00C7,
  HID_USAGE_CONSUMER_WIRELESS_RADIO_SLIDER_SWITCH = 0x00C8,

  // Media Control
  HID_USAGE_CONSUMER_PLAY = 0x00B0,
  HID_USAGE_CONSUMER_PAUSE = 0x00B1,
  HID_USAGE_CONSUMER_RECORD = 0x00B2,
  HID_USAGE_CONSUMER_FAST_FORWARD = 0x00B3,
  HID_USAGE_CONSUMER_REWIND = 0x00B4,
  HID_USAGE_CONSUMER_SCAN_NEXT = 0x00B5,
  HID_USAGE_CONSUMER_SCAN_PREVIOUS = 0x00B6,
  HID_USAGE_CONSUMER_STOP = 0x00B7,
  HID_USAGE_CONSUMER_EJECT = 0x00B8,
  HID_USAGE_CONSUMER_STOP_EJECT = 0x00CC,
  HID_USAGE_CONSUMER_PLAY_PAUSE = 0x00CD,
  HID_USAGE_CONSUMER_PLAY_SKIP = 0x00CE,
  HID_USAGE_CONSUMER_VOLUME = 0x00E0,
  HID_USAGE_CONSUMER_MUTE = 0x00E2,
  HID_USAGE_CONSUMER_BASS = 0x00E3,
  HID_USAGE_CONSUMER_TREBLE = 0x00E4,
  HID_USAGE_CONSUMER_BASS_BOOST = 0x00E5,
  HID_USAGE_CONSUMER_VOLUME_INCREMENT = 0x00E9,
  HID_USAGE_CONSUMER_VOLUME_DECREMENT = 0x00EA,
  HID_USAGE_CONSUMER_BASS_INCREMENT = 0x0152,
  HID_USAGE_CONSUMER_BASS_DECREMENT = 0x0153,
  HID_USAGE_CONSUMER_TREBLE_INCREMENT = 0x0154,
  HID_USAGE_CONSUMER_TREBLE_DECREMENT = 0x0155,

  // Application Launcher
  HID_USAGE_CONSUMER_AL_CONSUMER_CONTROL_CONFIGURATION = 0x0183,
  HID_USAGE_CONSUMER_AL_EMAIL_READER = 0x018A,
  HID_USAGE_CONSUMER_AL_CALCULATOR = 0x0192,
  HID_USAGE_CONSUMER_AL_LOCAL_BROWSER = 0x0194,

  // Browser/Explorer Specific
  HID_USAGE_CONSUMER_AC_SEARCH = 0x0221,
  HID_USAGE_CONSUMER_AC_HOME = 0x0223,
  HID_USAGE_CONSUMER_AC_BACK = 0x0224,
  HID_USAGE_CONSUMER_AC_FORWARD = 0x0225,
  HID_USAGE_CONSUMER_AC_STOP = 0x0226,
  HID_USAGE_CONSUMER_AC_REFRESH = 0x0227,
  HID_USAGE_CONSUMER_AC_BOOKMARKS = 0x022A,

  // Mouse Horizontal scroll
  HID_USAGE_CONSUMER_AC_PAN = 0x0238,
};

//! Key ID
/*!
Type to hold a key symbol identifier.  The encoding is UTF-32, using
U+E000 through U+EFFF for the various control keys (e.g. arrow
keys, function keys, modifier keys, etc).
*/
typedef uint32_t KeyID;

//! Key Code
/*!
Type to hold a physical key identifier.  That is, it identifies a
physical key on the keyboard.  KeyButton 0 is reserved to be an
invalid key;  platforms that use 0 as a physical key identifier
will have to remap that value to some arbitrary unused id.
*/
typedef uint16_t KeyButton;

//! Modifier key mask
/*!
Type to hold a bitmask of key modifiers (e.g. shift keys).
*/
typedef uint32_t KeyModifierMask;

//! Modifier key ID
/*!
Type to hold the id of a key modifier (e.g. a shift key).
*/
typedef uint32_t KeyModifierID;

//! @name Modifier key masks
//@{
static const KeyModifierMask KeyModifierShift = 0x0001;
static const KeyModifierMask KeyModifierControl = 0x0002;
static const KeyModifierMask KeyModifierAlt = 0x0004;
static const KeyModifierMask KeyModifierMeta = 0x0008;
static const KeyModifierMask KeyModifierSuper = 0x0010;
static const KeyModifierMask KeyModifierAltGr = 0x0020;
static const KeyModifierMask KeyModifierLevel5Lock = 0x0040;
static const KeyModifierMask KeyModifierCapsLock = 0x1000;
static const KeyModifierMask KeyModifierNumLock = 0x2000;
static const KeyModifierMask KeyModifierScrollLock = 0x4000;
//@}

//! @name Modifier key bits
//@{
static const uint32_t kKeyModifierBitNone = 16;
static const uint32_t kKeyModifierBitShift = 0;
static const uint32_t kKeyModifierBitControl = 1;
static const uint32_t kKeyModifierBitAlt = 2;
static const uint32_t kKeyModifierBitMeta = 3;
static const uint32_t kKeyModifierBitSuper = 4;
static const uint32_t kKeyModifierBitAltGr = 5;
static const uint32_t kKeyModifierBitLevel5Lock = 6;
static const uint32_t kKeyModifierBitCapsLock = 12;
static const uint32_t kKeyModifierBitNumLock = 13;
static const uint32_t kKeyModifierBitScrollLock = 14;
static const int32_t kKeyModifierNumBits = 16;
//@}

//! @name Modifier key identifiers
//@{
static const KeyModifierID kKeyModifierIDNull = 0;
static const KeyModifierID kKeyModifierIDShift = 1;
static const KeyModifierID kKeyModifierIDControl = 2;
static const KeyModifierID kKeyModifierIDAlt = 3;
static const KeyModifierID kKeyModifierIDMeta = 4;
static const KeyModifierID kKeyModifierIDSuper = 5;
static const KeyModifierID kKeyModifierIDAltGr = 6;
static const KeyModifierID kKeyModifierIDLast = 7;
//@}

//! @name Key identifiers
//@{
// all identifiers except kKeyNone and those in 0xE000 to 0xE0FF
// inclusive are equal to the corresponding X11 keysym - 0x1000.

// no key
static const KeyID kKeyNone = 0x0000;

// TTY functions
static const KeyID kKeyBackSpace = 0xEF08; /* back space, back char */
static const KeyID kKeyTab = 0xEF09;
static const KeyID kKeyLinefeed = 0xEF0A; /* Linefeed, LF */
static const KeyID kKeyClear = 0xEF0B;
static const KeyID kKeyReturn = 0xEF0D; /* Return, enter */
static const KeyID kKeyPause = 0xEF13;  /* Pause, hold */
static const KeyID kKeyScrollLock = 0xEF14;
static const KeyID kKeySysReq = 0xEF15;
static const KeyID kKeyEscape = 0xEF1B;
static const KeyID kKeyMuhenkan = 0xEF22;         /* Cancel Conversion */
static const KeyID kKeyHenkan = 0xEF23;           /* Start/Stop Conversion */
static const KeyID kKeyKana = 0xEF26;             /* Kana */
static const KeyID kKeyHiraganaKatakana = 0xEF27; /* Hiragana/Katakana toggle */
static const KeyID kKeyZenkaku = 0xEF2A;          /* Zenkaku/Hankaku */
static const KeyID kKeyKanzi = 0xEF2A;            /* Kanzi */
static const KeyID kKeyEisuToggle = 0xEF30;       /* Alphanumeric toggle */
static const KeyID kKeyHangul = 0xEF31;           /* Hangul */
static const KeyID kKeyHanja = 0xEF34;            /* Hanja */
static const KeyID kKeyDelete = 0xEFFF;           /* Delete, rubout */

// cursor control
static const KeyID kKeyHome = 0xEF50;
static const KeyID kKeyLeft = 0xEF51;  /* Move left, left arrow */
static const KeyID kKeyUp = 0xEF52;    /* Move up, up arrow */
static const KeyID kKeyRight = 0xEF53; /* Move right, right arrow */
static const KeyID kKeyDown = 0xEF54;  /* Move down, down arrow */
static const KeyID kKeyPageUp = 0xEF55;
static const KeyID kKeyPageDown = 0xEF56;
static const KeyID kKeyEnd = 0xEF57;   /* EOL */
static const KeyID kKeyBegin = 0xEF58; /* BOL */

// misc functions
static const KeyID kKeySelect = 0xEF60; /* Select, mark */
static const KeyID kKeyPrint = 0xEF61;
static const KeyID kKeyExecute = 0xEF62; /* Execute, run, do */
static const KeyID kKeyInsert = 0xEF63;  /* Insert, insert here */
static const KeyID kKeyUndo = 0xEF65;    /* Undo, oops */
static const KeyID kKeyRedo = 0xEF66;    /* redo, again */
static const KeyID kKeyMenu = 0xEF67;
static const KeyID kKeyFind = 0xEF68;   /* Find, search */
static const KeyID kKeyCancel = 0xEF69; /* Cancel, stop, abort, exit */
static const KeyID kKeyHelp = 0xEF6A;   /* Help */
static const KeyID kKeyBreak = 0xEF6B;
static const KeyID kKeyAltGr = 0xEF7E; /* Character set switch */
static const KeyID kKeyNumLock = 0xEF7F;

// keypad
static const KeyID kKeyKP_Space = 0xEF80; /* space */
static const KeyID kKeyKP_Tab = 0xEF89;
static const KeyID kKeyKP_Enter = 0xEF8D; /* enter */
static const KeyID kKeyKP_F1 = 0xEF91;    /* PF1, KP_A, ... */
static const KeyID kKeyKP_F2 = 0xEF92;
static const KeyID kKeyKP_F3 = 0xEF93;
static const KeyID kKeyKP_F4 = 0xEF94;
static const KeyID kKeyKP_Home = 0xEF95;
static const KeyID kKeyKP_Left = 0xEF96;
static const KeyID kKeyKP_Up = 0xEF97;
static const KeyID kKeyKP_Right = 0xEF98;
static const KeyID kKeyKP_Down = 0xEF99;
static const KeyID kKeyKP_PageUp = 0xEF9A;
static const KeyID kKeyKP_PageDown = 0xEF9B;
static const KeyID kKeyKP_End = 0xEF9C;
static const KeyID kKeyKP_Begin = 0xEF9D;
static const KeyID kKeyKP_Insert = 0xEF9E;
static const KeyID kKeyKP_Delete = 0xEF9F;
static const KeyID kKeyKP_Equal = 0xEFBD; /* equals */
static const KeyID kKeyKP_Multiply = 0xEFAA;
static const KeyID kKeyKP_Add = 0xEFAB;
static const KeyID kKeyKP_Separator = 0xEFAC; /* separator, often comma */
static const KeyID kKeyKP_Subtract = 0xEFAD;
static const KeyID kKeyKP_Decimal = 0xEFAE;
static const KeyID kKeyKP_Divide = 0xEFAF;
static const KeyID kKeyKP_0 = 0xEFB0;
static const KeyID kKeyKP_1 = 0xEFB1;
static const KeyID kKeyKP_2 = 0xEFB2;
static const KeyID kKeyKP_3 = 0xEFB3;
static const KeyID kKeyKP_4 = 0xEFB4;
static const KeyID kKeyKP_5 = 0xEFB5;
static const KeyID kKeyKP_6 = 0xEFB6;
static const KeyID kKeyKP_7 = 0xEFB7;
static const KeyID kKeyKP_8 = 0xEFB8;
static const KeyID kKeyKP_9 = 0xEFB9;

// function keys
static const KeyID kKeyF1 = 0xEFBE;
static const KeyID kKeyF2 = 0xEFBF;
static const KeyID kKeyF3 = 0xEFC0;
static const KeyID kKeyF4 = 0xEFC1;
static const KeyID kKeyF5 = 0xEFC2;
static const KeyID kKeyF6 = 0xEFC3;
static const KeyID kKeyF7 = 0xEFC4;
static const KeyID kKeyF8 = 0xEFC5;
static const KeyID kKeyF9 = 0xEFC6;
static const KeyID kKeyF10 = 0xEFC7;
static const KeyID kKeyF11 = 0xEFC8;
static const KeyID kKeyF12 = 0xEFC9;
static const KeyID kKeyF13 = 0xEFCA;
static const KeyID kKeyF14 = 0xEFCB;
static const KeyID kKeyF15 = 0xEFCC;
static const KeyID kKeyF16 = 0xEFCD;
static const KeyID kKeyF17 = 0xEFCE;
static const KeyID kKeyF18 = 0xEFCF;
static const KeyID kKeyF19 = 0xEFD0;
static const KeyID kKeyF20 = 0xEFD1;
static const KeyID kKeyF21 = 0xEFD2;
static const KeyID kKeyF22 = 0xEFD3;
static const KeyID kKeyF23 = 0xEFD4;
static const KeyID kKeyF24 = 0xEFD5;
static const KeyID kKeyF25 = 0xEFD6;
static const KeyID kKeyF26 = 0xEFD7;
static const KeyID kKeyF27 = 0xEFD8;
static const KeyID kKeyF28 = 0xEFD9;
static const KeyID kKeyF29 = 0xEFDA;
static const KeyID kKeyF30 = 0xEFDB;
static const KeyID kKeyF31 = 0xEFDC;
static const KeyID kKeyF32 = 0xEFDD;
static const KeyID kKeyF33 = 0xEFDE;
static const KeyID kKeyF34 = 0xEFDF;
static const KeyID kKeyF35 = 0xEFE0;

// modifiers
static const KeyID kKeyShift_L = 0xEFE1;   /* Left shift */
static const KeyID kKeyShift_R = 0xEFE2;   /* Right shift */
static const KeyID kKeyControl_L = 0xEFE3; /* Left control */
static const KeyID kKeyControl_R = 0xEFE4; /* Right control */
static const KeyID kKeyCapsLock = 0xEFE5;  /* Caps lock */
static const KeyID kKeyShiftLock = 0xEFE6; /* Shift lock */
static const KeyID kKeyMeta_L = 0xEFE7;    /* Left meta */
static const KeyID kKeyMeta_R = 0xEFE8;    /* Right meta */
static const KeyID kKeyAlt_L = 0xEFE9;     /* Left alt */
static const KeyID kKeyAlt_R = 0xEFEA;     /* Right alt */
static const KeyID kKeySuper_L = 0xEFEB;   /* Left super */
static const KeyID kKeySuper_R = 0xEFEC;   /* Right super */
static const KeyID kKeyHyper_L = 0xEFED;   /* Left hyper */
static const KeyID kKeyHyper_R = 0xEFEE;   /* Right hyper */

static const KeyID kKeyCopy = 0x1008EF57;
static const KeyID kKeyCut = 0x1008EF58;
static const KeyID kKeyOpen = 0x1008EF6b;
static const KeyID kKeyPaste = 0x1008EF6d;
static const KeyID kKeyProps = 0x1005EF70;
static const KeyID kKeyFront = 0x1005EF71;

// multi-key character composition
static const KeyID kKeyCompose = 0xEF20;
static const KeyID kKeyDeadGrave = 0x0300;
static const KeyID kKeyDeadAcute = 0x0301;
static const KeyID kKeyDeadCircumflex = 0x0302;
static const KeyID kKeyDeadTilde = 0x0303;
static const KeyID kKeyDeadMacron = 0x0304;
static const KeyID kKeyDeadBreve = 0x0306;
static const KeyID kKeyDeadAbovedot = 0x0307;
static const KeyID kKeyDeadDiaeresis = 0x0308;
static const KeyID kKeyDeadAbovering = 0x030a;
static const KeyID kKeyDeadDoubleacute = 0x030b;
static const KeyID kKeyDeadCaron = 0x030c;
static const KeyID kKeyDeadCedilla = 0x0327;
static const KeyID kKeyDeadOgonek = 0x0328;

// more function and modifier keys
static const KeyID kKeyLeftTab = 0xEE20;

// update modifiers
static const KeyID kKeySetModifiers = 0xEE06;
static const KeyID kKeyClearModifiers = 0xEE07;

// group change
static const KeyID kKeyNextGroup = 0xEE08;
static const KeyID kKeyPrevGroup = 0xEE0A;

// extended keys
static const KeyID kKeyEject = 0xE001;
static const KeyID kKeySleep = 0xE05F;
static const KeyID kKeyWWWBack = 0xE0A6;
static const KeyID kKeyWWWForward = 0xE0A7;
static const KeyID kKeyWWWRefresh = 0xE0A8;
static const KeyID kKeyWWWStop = 0xE0A9;
static const KeyID kKeyWWWSearch = 0xE0AA;
static const KeyID kKeyWWWFavorites = 0xE0AB;
static const KeyID kKeyWWWHome = 0xE0AC;
static const KeyID kKeyAudioMute = 0xE0AD;
static const KeyID kKeyAudioDown = 0xE0AE;
static const KeyID kKeyAudioUp = 0xE0AF;
static const KeyID kKeyAudioNext = 0xE0B0;
static const KeyID kKeyAudioPrev = 0xE0B1;
static const KeyID kKeyAudioStop = 0xE0B2;
static const KeyID kKeyAudioPlay = 0xE0B3;
static const KeyID kKeyAppMail = 0xE0B4;
static const KeyID kKeyAppMedia = 0xE0B5;
static const KeyID kKeyAppUser1 = 0xE0B6;
static const KeyID kKeyAppUser2 = 0xE0B7;
static const KeyID kKeyBrightnessDown = 0xE0B8;
static const KeyID kKeyBrightnessUp = 0xE0B9;
static const KeyID kKeyKbdBrightnessDown = 0xE0BA;
static const KeyID kKeyKbdBrightnessUp = 0xE0BB;
static const KeyID kKeyMissionControl = 0xE0C0;
static const KeyID kKeyLaunchpad = 0xE0C1;

uint8_t table[0x10000] = {0};
uint16_t consumer_table[0x10000] = {0};

void init_synergy_hid_key_table() {
  for (int i = 'A'; i <= 'Z'; i++) {
    table[i] = HID_KEY_A + (i - 'A');
  }
  for (int i = 'a'; i <= 'z'; i++) {
    table[i] = HID_KEY_A + (i - 'a');
  }

  for (int i = 0; i <= 9; i++) {
    table['1' + i] = HID_KEY_1 + i;
  }
  table[' '] = HID_KEY_SPACE;
  table['/'] = HID_KEY_SLASH;
  table['?'] = HID_KEY_SLASH;
  table['0'] = HID_KEY_0;
  table['!'] = HID_KEY_1;
  table['@'] = HID_KEY_2;
  table['#'] = HID_KEY_3;
  table['$'] = HID_KEY_4;
  table['%'] = HID_KEY_5;
  table['^'] = HID_KEY_6;
  table['&'] = HID_KEY_7;
  table['*'] = HID_KEY_8;
  table['('] = HID_KEY_9;
  table[')'] = HID_KEY_0;
  table['['] = HID_KEY_BRACKET_LEFT;
  table[']'] = HID_KEY_BRACKET_RIGHT;
  table['{'] = HID_KEY_BRACKET_LEFT;
  table['}'] = HID_KEY_BRACKET_RIGHT;
  table['-'] = HID_KEY_MINUS;
  table['_'] = HID_KEY_MINUS;
  table['='] = HID_KEY_EQUAL;
  table['+'] = HID_KEY_EQUAL;
  table[';'] = HID_KEY_SEMICOLON;
  table[':'] = HID_KEY_SEMICOLON;
  table['\''] = HID_KEY_APOSTROPHE;
  table['"'] = HID_KEY_APOSTROPHE;
  table['.'] = HID_KEY_PERIOD;
  table[','] = HID_KEY_COMMA;
  table['>'] = HID_KEY_PERIOD;
  table['<'] = HID_KEY_COMMA;
  table['`'] = HID_KEY_GRAVE;
  table['~'] = HID_KEY_GRAVE;
  table['\\'] = HID_KEY_BACKSLASH;
  table['|'] = HID_KEY_BACKSLASH;

  // NOTE: Since Barrier reads input events from the OS instead of the hardware,
  // it supports much more event types than USB HID standard, e.g. Hyper key is
  // emulated on most systems but Barrier can see it and send it out, but HID
  // device cannot send it to the host as there is no corresponding key code and
  // USB device doesn't know how host OS emulates the event.
  table[kKeyBackSpace] = HID_KEY_BACKSPACE;
  table[kKeyTab] = HID_KEY_TAB;
  // table[ kKeyLinefeed ] = HID_KEY_ ;
  table[kKeyClear] = HID_KEY_CLEAR;
  table[kKeyReturn] = HID_KEY_ENTER;
  table[kKeyPause] = HID_KEY_PAUSE;
  table[kKeyScrollLock] = HID_KEY_SCROLL_LOCK;
  table[kKeySysReq] = HID_KEY_SYSREQ_ATTENTION;
  table[kKeyEscape] = HID_KEY_ESCAPE;
  // table[ kKeyMuhenkan ] = HID_KEY_ ;
  // table[ kKeyHenkan ] = HID_KEY_ ;
  // table[ kKeyKana ] = HID_KEY_ ;
  // table[ kKeyHiraganaKatakana ] = HID_KEY_ ;
  // table[ kKeyZenkaku ] = HID_KEY_ ;
  // table[ kKeyKanzi ] = HID_KEY_ ;
  // table[ kKeyEisuToggle ] = HID_KEY_ ;
  // table[ kKeyHangul ] = HID_KEY_ ;
  // table[ kKeyHanja ] = HID_KEY_ ;
  table[kKeyDelete] = HID_KEY_DELETE;
  table[kKeyHome] = HID_KEY_HOME;
  table[kKeyLeft] = HID_KEY_ARROW_LEFT;
  table[kKeyUp] = HID_KEY_ARROW_UP;
  table[kKeyRight] = HID_KEY_ARROW_RIGHT;
  table[kKeyDown] = HID_KEY_ARROW_DOWN;
  table[kKeyPageUp] = HID_KEY_PAGE_UP;
  table[kKeyPageDown] = HID_KEY_PAGE_DOWN;
  table[kKeyEnd] = HID_KEY_END;
  // table[ kKeyBegin ] = HID_KEY_ ;
  table[kKeySelect] = HID_KEY_SELECT;
  table[kKeyPrint] = HID_KEY_PRINT_SCREEN;
  table[kKeyExecute] = HID_KEY_EXECUTE;
  table[kKeyInsert] = HID_KEY_INSERT;
  table[kKeyUndo] = HID_KEY_UNDO;
  // table[ kKeyRedo ] = HID_KEY_REDO ;
  table[kKeyMenu] = HID_KEY_MENU;
  table[kKeyFind] = HID_KEY_FIND;
  table[kKeyCancel] = HID_KEY_CANCEL;
  table[kKeyHelp] = HID_KEY_HELP;
  // table[ kKeyBreak ] = HID_KEY_ ;
  // table[ kKeyAltGr ] = HID_KEY_ ;
  table[kKeyNumLock] = HID_KEY_NUM_LOCK;

  // NOTE: These keys are for KeyPad only, not exist in USB HID standard
  // table[ kKeyKP_Space ] = HID_KEY_KEYPAD_ ;
  // table[ kKeyKP_Tab ] = HID_KEY_KEYPAD_ ;
  table[kKeyKP_Enter] = HID_KEY_KEYPAD_ENTER;
  // table[ kKeyKP_F1 ] = HID_KEY_KEYPAD_ ;
  // table[ kKeyKP_F2 ] = HID_KEY_KEYPAD_ ;
  // table[ kKeyKP_F3 ] = HID_KEY_KEYPAD_ ;
  // table[ kKeyKP_F4 ] = HID_KEY_KEYPAD_ ;
  // table[ kKeyKP_Home ] = HID_KEY_KEYPAD_ ;
  // table[ kKeyKP_Left ] = HID_KEY_KEYPAD_ ;
  // table[ kKeyKP_Up ] = HID_KEY_ ;
  // table[ kKeyKP_Right ] = HID_KEY_ ;
  // table[ kKeyKP_Down ] = HID_KEY_ ;
  // table[ kKeyKP_PageUp ] = HID_KEY_ ;
  // table[ kKeyKP_PageDown ] = HID_KEY_ ;
  // table[ kKeyKP_End ] = HID_KEY_ ;
  // table[ kKeyKP_Begin ] = HID_KEY_KEYPAD_ ;
  // table[ kKeyKP_Insert ] = HID_KEY_ ;
  // table[ kKeyKP_Delete ] = HID_KEY_ ;
  table[kKeyKP_Equal] = HID_KEY_KEYPAD_EQUAL;
  table[kKeyKP_Multiply] = HID_KEY_KEYPAD_MULTIPLY;
  table[kKeyKP_Add] = HID_KEY_KEYPAD_ADD;
  // table[ kKeyKP_Separator ] = HID_KEY_ ;
  table[kKeyKP_Subtract] = HID_KEY_KEYPAD_SUBTRACT;
  table[kKeyKP_Decimal] = HID_KEY_KEYPAD_DECIMAL;
  table[kKeyKP_Divide] = HID_KEY_KEYPAD_DIVIDE;
  table[kKeyKP_0] = HID_KEY_KEYPAD_0;
  table[kKeyKP_1] = HID_KEY_KEYPAD_1;
  table[kKeyKP_2] = HID_KEY_KEYPAD_2;
  table[kKeyKP_3] = HID_KEY_KEYPAD_3;
  table[kKeyKP_4] = HID_KEY_KEYPAD_4;
  table[kKeyKP_5] = HID_KEY_KEYPAD_5;
  table[kKeyKP_6] = HID_KEY_KEYPAD_6;
  table[kKeyKP_7] = HID_KEY_KEYPAD_7;
  table[kKeyKP_8] = HID_KEY_KEYPAD_8;
  table[kKeyKP_9] = HID_KEY_KEYPAD_9;
  table[kKeyF1] = HID_KEY_F1;
  table[kKeyF2] = HID_KEY_F2;
  table[kKeyF3] = HID_KEY_F3;
  table[kKeyF4] = HID_KEY_F4;
  table[kKeyF5] = HID_KEY_F5;
  table[kKeyF6] = HID_KEY_F6;
  table[kKeyF7] = HID_KEY_F7;
  table[kKeyF8] = HID_KEY_F8;
  table[kKeyF9] = HID_KEY_F9;
  table[kKeyF10] = HID_KEY_F10;
  table[kKeyF11] = HID_KEY_F11;
  table[kKeyF12] = HID_KEY_F12;
  table[kKeyF13] = HID_KEY_F13;
  table[kKeyF14] = HID_KEY_F14;
  table[kKeyF15] = HID_KEY_F15;
  table[kKeyF16] = HID_KEY_F16;
  table[kKeyF17] = HID_KEY_F17;
  table[kKeyF18] = HID_KEY_F18;
  table[kKeyF19] = HID_KEY_F19;
  table[kKeyF20] = HID_KEY_F20;
  table[kKeyF21] = HID_KEY_F21;
  table[kKeyF22] = HID_KEY_F22;
  table[kKeyF23] = HID_KEY_F23;
  table[kKeyF24] = HID_KEY_F24;
  // NOTE: USB HID standard has only 24 F keys
  // table[ kKeyF25 ] = HID_KEY_F25 ;
  // table[ kKeyF26 ] = HID_KEY_F26 ;
  // table[ kKeyF27 ] = HID_KEY_F27 ;
  // table[ kKeyF28 ] = HID_KEY_F28 ;
  // table[ kKeyF29 ] = HID_KEY_F29 ;
  // table[ kKeyF30 ] = HID_KEY_F30 ;
  // table[ kKeyF31 ] = HID_KEY_F31 ;
  // table[ kKeyF32 ] = HID_KEY_F32 ;
  // table[ kKeyF33 ] = HID_KEY_F33 ;
  // table[ kKeyF34 ] = HID_KEY_F34 ;
  // table[ kKeyF35 ] = HID_KEY_F35 ;
  table[kKeyShift_L] = HID_KEY_SHIFT_LEFT;
  table[kKeyShift_R] = HID_KEY_SHIFT_RIGHT;
  table[kKeyControl_L] = HID_KEY_CONTROL_LEFT;
  table[kKeyControl_R] = HID_KEY_CONTROL_RIGHT;
  table[kKeyCapsLock] = HID_KEY_CAPS_LOCK;
  // NOTE: In USB HID standard ShiftLock is not a key code but a generic desktop
  // usage
  // TODO: Need to have a separated report to send GD usage, and this report
  // cannot be inside of Keyboard descriptor because the Keyboard report is the
  // standard USB boot protocol, maybe I can add it into the Consumer report.
  // table[ kKeyShiftLock ] = HID_KEY_ ;
  table[kKeyMeta_L] = HID_KEY_ALT_LEFT;
  table[kKeyMeta_R] = HID_KEY_ALT_RIGHT;
  table[kKeyAlt_L] = HID_KEY_ALT_LEFT;
  table[kKeyAlt_R] = HID_KEY_ALT_RIGHT;
  table[kKeySuper_L] = HID_KEY_GUI_LEFT;
  table[kKeySuper_R] = HID_KEY_GUI_RIGHT;

  // NOTE: USB HID standard doesn't have Hyper key
  // table[ kKeyHyper_L ] = HID_KEY_ ;
  // table[ kKeyHyper_R ] = HID_KEY_ ;

  consumer_table[kKeyEject] = HID_USAGE_CONSUMER_EJECT;
  consumer_table[kKeySleep] = HID_USAGE_CONSUMER_SLEEP;
  consumer_table[kKeyWWWBack] = HID_USAGE_CONSUMER_AC_BACK;
  consumer_table[kKeyWWWForward] = HID_USAGE_CONSUMER_AC_FORWARD;
  consumer_table[kKeyWWWRefresh] = HID_USAGE_CONSUMER_AC_REFRESH;
  consumer_table[kKeyWWWStop] = HID_USAGE_CONSUMER_AC_STOP;
  consumer_table[kKeyWWWSearch] = HID_USAGE_CONSUMER_AC_SEARCH;
  consumer_table[kKeyWWWFavorites] = HID_USAGE_CONSUMER_AC_BOOKMARKS;
  consumer_table[kKeyWWWHome] = HID_USAGE_CONSUMER_AC_HOME;
  consumer_table[kKeyAudioMute] = HID_USAGE_CONSUMER_MUTE;
  consumer_table[kKeyAudioDown] = HID_USAGE_CONSUMER_VOLUME_DECREMENT;
  consumer_table[kKeyAudioUp] = HID_USAGE_CONSUMER_VOLUME_INCREMENT;
  consumer_table[kKeyAudioNext] = HID_USAGE_CONSUMER_SCAN_NEXT;
  consumer_table[kKeyAudioPrev] = HID_USAGE_CONSUMER_SCAN_PREVIOUS;
  consumer_table[kKeyAudioStop] = HID_USAGE_CONSUMER_STOP;
  consumer_table[kKeyAudioPlay] = HID_USAGE_CONSUMER_PLAY_PAUSE;
  consumer_table[kKeyAppMail] = HID_USAGE_CONSUMER_AL_EMAIL_READER;
  // consumer_table[ kKeyAppMedia ] = HID_USAGE_CONSUMER_ ;
  // consumer_table[ kKeyAppUser1 ] = HID_KEY_ ;
  // consumer_table[ kKeyAppUser2 ] = HID_KEY_ ;
  consumer_table[kKeyBrightnessDown] = HID_USAGE_CONSUMER_BRIGHTNESS_DECREMENT;
  consumer_table[kKeyBrightnessUp] = HID_USAGE_CONSUMER_BRIGHTNESS_INCREMENT;
  consumer_table[kKeyKbdBrightnessDown] =
      HID_USAGE_CONSUMER_KEYBOARD_BRIGHTNESS_DECREMENT;
  consumer_table[kKeyKbdBrightnessUp] =
      HID_USAGE_CONSUMER_KEYBOARD_BRIGHTNESS_INCREMENT;

  // These 2 keys are only available on Apple branded keyboards
  // consumer_table[ kKeyMissionControl ] = HID_KEY_ ;
  // consumer_table[ kKeyLaunchpad ] = HID_KEY_ ;

  table[kKeyAudioPlay] = HID_USAGE_CONSUMER_PLAY;
  table[kKeyAudioNext] = HID_USAGE_CONSUMER_SCAN_NEXT;
  table[kKeyAudioPrev] = HID_USAGE_CONSUMER_SCAN_PREVIOUS;
}

int main() {
  init_synergy_hid_key_table();
  printf("const TABLE: [u8; 256] = [");
  for (int i = 0; i < 0x100; i++) {
    printf("0x%02X,", table[i]);
  }
  printf("];\n");
  printf("const EXT_TAB: [u8; 256] = [");
  for (int i = 0xEF00; i < 0xF000; i++) {
    printf("0x%02X,", table[i]);
  }
  printf("];\n");
  printf("const MEDIA_TAB: [u16; 256] = [");
  for (int i = 0xE000; i < 0xE100; i++) {
    printf("0x%04X,", consumer_table[i]);
  }
  printf("];\n");
  return 0;
}

void f() {
  // TTY functions
}