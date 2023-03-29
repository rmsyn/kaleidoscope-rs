#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::{key_defs::*, keymaps};

pub const QWERTY: u8 = 0;
pub const FUN: u8 = 1;
pub const UPPER: u8 = 2;

const MACRO_QWERTY: u8 = 0;
const MACRO_VERSION_INFO: u8 = 1;
pub const NUM_LAYERS: usize = 3;

pub const Key_Exclamation: Key = lshift!(Key_1);
pub const Key_At: Key = lshift!(Key_2);
pub const Key_Hash: Key = lshift!(Key_3);
pub const Key_Dollar: Key = lshift!(Key_4);
pub const Key_Percent: Key = lshift!(Key_5);
pub const Key_Caret: Key = lshift!(Key_6);
pub const Key_And: Key = lshift!(Key_7);
pub const Key_Star: Key = lshift!(Key_8);
pub const Key_LeftParen: Key = lshift!(Key_9);
pub const Key_RightParen: Key = lshift!(Key_0);
pub const Key_Plus: Key = lshift!(Key_Equals);

#[rustfmt::skip(keymaps)]
keymaps! {
    KEYMAP_LINEAR,
    [
        [   // QWERTY
            Key_Q,      Key_W,   Key_E,       Key_R,         Key_T,
            Key_A,      Key_S,   Key_D,       Key_F,         Key_G,
            Key_Z,      Key_X,   Key_C,       Key_V,         Key_B,         Key_Backtick,
            Key_Escape, Key_Tab, Key_LeftGui, Key_LeftShift, Key_Backspace, Key_LeftControl,

                           Key_Y,        Key_U,    Key_I,     Key_O,      Key_P,
                           Key_H,        Key_J,    Key_K,     Key_L,      Key_Semicolon,
            Key_Backslash, Key_N,        Key_M,    Key_Comma, Key_Period, Key_Slash,
            Key_LeftAlt,   Key_Spacebar, MO!(FUN), Key_Minus, Key_Quote,  Key_Enter,
            XXX, XXX, XXX, XXX,
        ],

        [   // FUN
            Key_Exclamation, Key_At,           Key_UpArrow,   Key_Dollar,           Key_Percent,
            Key_LeftParen,   Key_LeftArrow,    Key_DownArrow, Key_RightArrow,       Key_RightParen,
            Key_LeftBracket, Key_RightBracket, Key_Hash,      Key_LeftCurlyBracket, Key_RightCurlyBracket, Key_Caret,
            TG!(UPPER),      Key_Insert,       Key_LeftGui,   Key_LeftShift,        Key_Delete,            Key_LeftControl,

                         Key_PageUp,   Key_7, Key_8,      Key_9, Key_Backspace,
                         Key_PageDown, Key_4, Key_5,      Key_6, ___,
            Key_And,     Key_Star,     Key_1, Key_2,      Key_3, Key_Plus,
            Key_LeftAlt, Key_Spacebar, ___,   Key_Period, Key_0, Key_Equals,
            XXX, XXX, XXX, XXX,
        ],

        [   // UPPER
            Key_Insert,                 Key_Home,                 Key_UpArrow,   Key_End,        Key_PageUp,
            Key_Delete,                 Key_LeftArrow,            Key_DownArrow, Key_RightArrow, Key_PageDown,
            M!(MACRO_VERSION_INFO),     Consumer_VolumeIncrement, XXX,           XXX,            ___,          ___,
            ML!(QWERTY),                Consumer_VolumeDecrement, ___,           ___,            ___,          ___,

                 Key_UpArrow,   Key_F7,      Key_F8,          Key_F9,         Key_F10,
                 Key_DownArrow, Key_F4,      Key_F5,          Key_F6,         Key_F11,
            ___, XXX,           Key_F1,      Key_F2,          Key_F3,         Key_F12,
            ___, ___,           ML!(QWERTY), Key_PrintScreen, Key_ScrollLock, Consumer_PlaySlashPause,
            XXX, XXX, XXX, XXX,
        ],
    ],
    NUM_LAYERS
}
