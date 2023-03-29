use keyboardio_hid::{boot, media, nkro, system_control, Keyboard as HIDKeyboard};

use crate::{Key, Result};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ActiveKeyboard {
    #[default]
    Boot,
    NKRO,
    Media,
    System,
    None,
}

/// Generic keyboard trait
pub trait Keyboard<'k> {
    type BootKeyboard: boot::BootKeyboard;
    type NKROKeyboard: nkro::NKROKeyboard;
    type ConsumerControl: media::MediaKeyboard;
    type SystemControl: system_control::SystemControlKeyboard;

    /// Gets a reference to the keyboard as a [KeyboardOps] object.
    ///
    /// Returns an error if the implementation does not have a set keyboard.
    fn keyboard(&'k self) -> &'k HIDKeyboard;

    /// Gets a mutable reference to the keyboard as a [KeyboardOps] object.
    ///
    /// Returns an error if the implementation does not have a set keyboard.
    fn keyboard_mut(&'k mut self) -> &'k mut HIDKeyboard;

    /// Gets the currently [ActiveKeyboard].
    fn active_keyboard(&self) -> ActiveKeyboard;

    /// Sets the currently [ActiveKeyboard].
    fn set_active_keyboard(&mut self, active_keyboard: ActiveKeyboard);

    /// Gets the last processed system control keycode.
    fn last_system_control_keycode(&self) -> u8;

    /// Sets the last processed system control keycode.
    fn set_last_system_control_keycode(&mut self, key_code: u8);

    /// Gets an optional reference to the boot keyboard.
    fn boot_keyboard(&'k self) -> &'k dyn boot::BootKeyboard;

    /// Gets an optional mutable reference to the boot keyboard.
    fn boot_keyboard_mut(&'k mut self) -> &'k mut dyn boot::BootKeyboard;

    /// Gets an optional reference to the NKRO keyboard.
    fn nkro_keyboard(&'k self) -> &'k dyn nkro::NKROKeyboard;

    /// Gets an optional mutable reference to the NKRO keyboard.
    fn nkro_keyboard_mut(&'k mut self) -> &'k mut dyn nkro::NKROKeyboard;

    /// Gets an optional reference to the consumer control / media keyboard.
    fn consumer_control(&'k self) -> &'k dyn media::MediaKeyboard;

    /// Gets an optional mutable reference to the consumer control / media keyboard.
    fn consumer_control_mut(&'k mut self) -> &'k mut dyn media::MediaKeyboard;

    /// Gets an optional reference to the system control keyboard.
    fn system_control(&'k self) -> &'k dyn system_control::SystemControlKeyboard;

    /// Gets an optional mutable reference to the system control keyboard.
    fn system_control_mut(&'k mut self) -> &'k mut dyn system_control::SystemControlKeyboard; 

    fn setup(&'k mut self) -> Result<()> {
        self.keyboard().begin();
        Ok(())
    }

    /// Releases all currently held keys.
    fn release_all_keys(&'k mut self) -> Result<()> {
        self.keyboard_mut().release_all();

        Ok(())
    }

    fn press_consumer_control(&'k mut self, mapped_key: Key) {
        self.consumer_control_mut().press(mapped_key.consumer() as u8);
    }

    fn release_consumer_control(&'k mut self, mapped_key: Key) {
        self.consumer_control_mut().release(mapped_key.consumer() as u8);
    }

    fn press_system_control(&'k mut self, mapped_key: Key);

    fn release_system_control(&'k mut self, mapped_key: Key) {
        let keycode = mapped_key.key_code();
        if keycode == self.last_system_control_keycode() {
            self.system_control_mut().release(keycode);
        }
    }

    fn press_key(&'k mut self, pressed_key: Key);

    fn release_key(&'k mut self, released_key: Key);

    fn press_modifiers(&'k mut self, pressed_key: Key);

    fn release_modifiers(&'k mut self, released_key: Key);

    fn clear_modifiers(&'k mut self);

    fn press_raw_key(&'k mut self, pressed_key: Key);

    fn release_raw_key(&'k mut self, released_key: Key);
}
