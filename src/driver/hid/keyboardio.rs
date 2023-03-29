use keyboardio_hid::{Keyboard as HIDKeyboard, KeyboardUsbBusAllocator};
use keyboardio_hid::{boot, media, nkro, system_control};

use super::base::keyboard::{ActiveKeyboard, Keyboard};

use crate::{Result, key_defs::*};

pub struct Keyboardio<'k> {
    pub boot_keyboard: HIDKeyboard<'k>,
    pub nkro_keyboard: HIDKeyboard<'k>,
    pub media_keyboard: HIDKeyboard<'k>,
    pub system_control_keyboard: HIDKeyboard<'k>,
    active_keyboard: ActiveKeyboard,
    last_system_control_keycode: u8,
}

impl<'k> Keyboardio<'k> {
    /// Creates a new [Keyboardio] keyboard.
    ///
    /// Use [`active_keyboard`](super::keyboard::ActiveKeyboard) to determine which keyboard
    /// implementation is currently active.
    ///
    /// Caller must call [attach_to_host](Self::attach_to_host) before using UsbDevice functions.
    /// `attach_to_host` creates and initializes the UsbDevice.
    pub fn new(bus: &'k KeyboardUsbBusAllocator, active_keyboard: ActiveKeyboard) -> Self {
        Self {
            boot_keyboard: HIDKeyboard::new_boot(bus),
            nkro_keyboard: HIDKeyboard::new_nkro(bus),
            media_keyboard: HIDKeyboard::new_media(bus),
            system_control_keyboard: HIDKeyboard::new_system_control(bus),
            active_keyboard,
            last_system_control_keycode: 0,
        }
    }

    /// Sends the current USB report from the device to the host.
    pub fn send_report(&mut self) -> Result<()> {
        match self.active_keyboard {
            ActiveKeyboard::Boot => {
                use boot::BootKeyboard;
                self.boot_keyboard.send_report()?
            }
            ActiveKeyboard::Media => {
                use media::MediaKeyboard;
                self.media_keyboard.send_report()?
            }
            ActiveKeyboard::NKRO => {
                use nkro::NKROKeyboard;
                self.nkro_keyboard.send_report()?
            }
            ActiveKeyboard::System => {
                use system_control::SystemControlKeyboard;
                self.system_control_keyboard.send_report()?
            }
            _ => (),
        }

        Ok(())
    }

    /// Gets whether the provided key is in the current USB report.
    pub fn is_key_pressed(&self, key: &Key) -> bool {
        let key_code = key.key_code();

        match self.active_keyboard {
            ActiveKeyboard::Boot => self.boot_keyboard().is_key_pressed(key_code),
            ActiveKeyboard::NKRO => self.nkro_keyboard().is_key_pressed(key_code),
            ActiveKeyboard::Media => self.consumer_control().is_key_pressed(key_code),
            ActiveKeyboard::System => self.system_control().is_key_pressed(key_code),
            _ => false,
        }
    }
}

impl<'k> Keyboard<'k> for Keyboardio<'k> {
    type BootKeyboard = HIDKeyboard<'k>;
    type NKROKeyboard = HIDKeyboard<'k>;
    type ConsumerControl = HIDKeyboard<'k>;
    type SystemControl = HIDKeyboard<'k>;

    fn keyboard(&'k self) -> &'k HIDKeyboard {
        match self.active_keyboard {
            ActiveKeyboard::Boot => &self.boot_keyboard,
            ActiveKeyboard::NKRO => &self.nkro_keyboard,
            ActiveKeyboard::Media => &self.media_keyboard,
            ActiveKeyboard::System => &self.system_control_keyboard,
            _ => &self.boot_keyboard,
        }
    }

    fn keyboard_mut(&'k mut self) -> &'k mut HIDKeyboard {
        match self.active_keyboard {
            ActiveKeyboard::Boot => self.boot_keyboard.as_mut(),
            ActiveKeyboard::NKRO => self.nkro_keyboard.as_mut(),
            ActiveKeyboard::Media => self.media_keyboard.as_mut(),
            ActiveKeyboard::System => self.system_control_keyboard.as_mut(),
            _ => self.boot_keyboard.as_mut(),
        }
    }

    fn active_keyboard(&self) -> ActiveKeyboard {
        self.active_keyboard
    }

    fn boot_keyboard(&'k self) -> &'k dyn boot::BootKeyboard {
        &self.boot_keyboard
    }

    fn boot_keyboard_mut(&'k mut self) -> &'k mut dyn boot::BootKeyboard {
        &mut self.boot_keyboard
    }

    fn nkro_keyboard(&'k self) -> &'k dyn nkro::NKROKeyboard {
        &self.nkro_keyboard
    }

    fn nkro_keyboard_mut(&'k mut self) -> &'k mut dyn nkro::NKROKeyboard {
        &mut self.nkro_keyboard
    }

    fn consumer_control(&'k self) -> &'k dyn media::MediaKeyboard {
        &self.media_keyboard
    }

    fn consumer_control_mut(&'k mut self) -> &'k mut dyn media::MediaKeyboard {
        &mut self.media_keyboard
    }

    fn system_control(&'k self) -> &'k dyn system_control::SystemControlKeyboard {
        &self.system_control_keyboard
    }

    fn system_control_mut(&'k mut self) -> &'k mut dyn system_control::SystemControlKeyboard {
        &mut self.system_control_keyboard
    }

    fn set_active_keyboard(&mut self, active_keyboard: ActiveKeyboard) {
        self.active_keyboard = active_keyboard;
    }

    fn last_system_control_keycode(&self) -> u8 {
        self.last_system_control_keycode
    }

    fn set_last_system_control_keycode(&mut self, key_code: u8) {
        self.last_system_control_keycode = key_code;
    }

    fn press_system_control(&'k mut self, mapped_key: Key) {
        use system_control::SystemControlKeyboard;

        let keycode = mapped_key.key_code();

        self.system_control_keyboard.press(keycode);
        self.last_system_control_keycode = keycode;
    }

    fn press_key(&'k mut self, pressed_key: Key) {
        crate::press_modifiers!(self, pressed_key);
        crate::press_raw_key!(self, pressed_key);
    }

    fn release_key(&'k mut self, released_key: Key) {
        crate::release_modifiers!(self, released_key);
        crate::release_raw_key!(self, released_key);
    }

    fn press_modifiers(&'k mut self, pressed_key: Key) {
        crate::press_modifiers!(self, pressed_key);
    }

    fn release_modifiers(&'k mut self, released_key: Key) {
        crate::release_modifiers!(self, released_key);
    }

    fn clear_modifiers(&'k mut self) {
        if self.active_keyboard == ActiveKeyboard::Boot {
            use boot::BootKeyboard;
            self.boot_keyboard.release(Key_LeftShift.key_code());
            self.boot_keyboard.release(Key_LeftControl.key_code());
            self.boot_keyboard.release(Key_LeftAlt.key_code());
            self.boot_keyboard.release(Key_RightAlt.key_code());
            self.boot_keyboard.release(Key_LeftGui.key_code());
        } else {
            use nkro::NKROKeyboard;
            self.nkro_keyboard.release(Key_LeftShift.key_code());
            self.nkro_keyboard.release(Key_LeftControl.key_code());
            self.nkro_keyboard.release(Key_LeftAlt.key_code());
            self.nkro_keyboard.release(Key_RightAlt.key_code());
            self.nkro_keyboard.release(Key_LeftGui.key_code());
        }
    }

    fn press_raw_key(&'k mut self, pressed_key: Key) {
        crate::press_raw_key!(self, pressed_key);
    }

    fn release_raw_key(&'k mut self, released_key: Key) {
        crate::release_raw_key!(self, released_key);
    }
}

//
// Implement the following trait functions as macros to avoid borrow checker complaining
// about a double mutable borrow that is actually a single mutable borrow...
//

#[macro_export]
macro_rules! press_modifiers {
    ($keyboard:tt, $key:tt) => {
        let flags = $key.flags();

        if flags & KeyFlags::SHIFT_HELD != KeyFlags::NONE {
            $crate::press_raw_key!($keyboard, Key_LeftShift);
        }
        if flags & KeyFlags::CTRL_HELD != KeyFlags::NONE {
            $crate::press_raw_key!($keyboard, Key_LeftControl);
        }
        if flags & KeyFlags::LALT_HELD != KeyFlags::NONE {
            $crate::press_raw_key!($keyboard, Key_LeftAlt);
        }
        if flags & KeyFlags::RALT_HELD != KeyFlags::NONE {
            $crate::press_raw_key!($keyboard, Key_RightAlt);
        }
        if flags & KeyFlags::GUI_HELD != KeyFlags::NONE {
            $crate::press_raw_key!($keyboard, Key_LeftGui);
        }
    }
}

#[macro_export]
macro_rules! release_modifiers {
    ($keyboard:tt, $key:tt) => {
        let flags = $key.flags();

        if flags & KeyFlags::SHIFT_HELD != KeyFlags::NONE {
            $crate::release_raw_key!($keyboard, Key_LeftShift);
        }
        if flags & KeyFlags::CTRL_HELD != KeyFlags::NONE {
            $crate::release_raw_key!($keyboard, Key_LeftControl);
        }
        if flags & KeyFlags::LALT_HELD != KeyFlags::NONE {
            $crate::release_raw_key!($keyboard, Key_LeftAlt);
        }
        if flags & KeyFlags::RALT_HELD != KeyFlags::NONE {
            $crate::release_raw_key!($keyboard, Key_RightAlt);
        }
        if flags & KeyFlags::GUI_HELD != KeyFlags::NONE {
            $crate::release_raw_key!($keyboard, Key_LeftGui);
        }
    }
}

#[macro_export]
macro_rules! press_raw_key {
    ($keyboard:tt, $key:tt) => {
        if $keyboard.active_keyboard == ActiveKeyboard::Boot {
            use boot::BootKeyboard;
            $keyboard.boot_keyboard.press($key.key_code());
        } else {
            use nkro::NKROKeyboard;
            $keyboard.nkro_keyboard.press($key.key_code());
        }
    }
}

#[macro_export]
macro_rules! release_raw_key {
    ($keyboard:tt, $key:tt) => {
        if $keyboard.active_keyboard == ActiveKeyboard::Boot {
            use boot::BootKeyboard;
            $keyboard.boot_keyboard.release($key.key_code());
        } else {
            use nkro::NKROKeyboard;
            $keyboard.nkro_keyboard.release($key.key_code());
        }
    }
}
