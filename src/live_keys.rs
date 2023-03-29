use core::ops::{Index, IndexMut};

use crate::{Key, KeyAddr, Key_Inactive, Key_Masked, KeyMap, KeyMapIter, KeyMapIterMut};

/// A representation of the "live" state of the keys on the keyboard
///
/// This is structure of [Key] values, indexed by `KeyAddr` values, with one
/// entry per key on the keyboard. These entries are meant to represent the
/// current state of the keys, with two special values:
///
/// - [KEY_INACTIVE] indicates that the given key is not active. Usually this
///   will correspond to the keyswitch being off (not pressed).
///
/// - [KEY_MASKED] indicates that the given key has been masked. When a key
///   release event is processed for that key, it will be reset to
///   [KEY_INACTIVE].
///
/// Any other value indicates that the key is active (i.e. pressed, though
/// plugins can set entries to active even when the physical keyswitches are not
/// engaged), and the `Key` value is what the that key is "sending" at the
/// time. At the end of its processing of a [KeyEvent](crate::key_event::KeyEvent), Kaleidoscope will use
/// the contents of this array to populate the Keyboard HID reports.
pub struct LiveKeys {
    key_map: KeyMap,
    dummy: Key,
}

impl LiveKeys {
    pub const fn new() -> Self {
        Self {
            key_map: KeyMap::new(),
            dummy: Key_Masked,
        }
    }

    /// Set an entry to "active" with a specified [Key] value.
    pub fn activate(&mut self, key_addr: KeyAddr, key: Key) {
        if key_addr.is_valid() {
            self.key_map[key_addr] = key;
        }
    }

    /// Deactivate an entry by setting its value to [KEY_INACTIVE].
    pub fn clear(&mut self, key_addr: KeyAddr) {
        if key_addr.is_valid() {
            self.key_map[key_addr] = Key_Inactive;
        }
    }

    /// Mask a key by setting its entry to [KEY_MASKED]. The key will become
    /// unmasked by Kaleidoscope on release (but not on a key press event).
    pub fn mask(&mut self, key_addr: KeyAddr) {
        if key_addr.is_valid() {
            self.key_map[key_addr] = Key_Masked;
        }
    }

    /// Clear the entire array by setting all values to [KEY_INACTIVE].
    pub fn clear_all(&mut self) {
        for key_addr in self.key_map.iter_mut() {
            *key_addr = Key_Inactive;
        }
    }

    /// Returns an iterator for use in range-based for loops.
    ///
    /// Example:
    ///
    /// ```rust
    /// use kaleidoscope::{Key, LiveKeys};
    ///
    /// let live_keys = LiveKeys::new();
    ///
    /// for key in live_keys.iter() {
    ///     assert_eq!(key, Key::default());
    /// }
    /// ```
    pub fn iter(&self) -> KeyMapIter {
        self.key_map.iter()
    }

    /// Returns a mutable iterator for use in range-based for loops.
    ///
    /// Example:
    ///
    /// ```rust
    /// use kaleidoscope::{Key, LiveKeys};
    ///
    /// let mut live_keys = LiveKeys::new();
    ///
    /// for key in live_keys.iter_mut() {
    ///     assert_eq!(key, Key::default());
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> KeyMapIterMut {
        self.key_map.iter_mut()
    }
}

impl Index<KeyAddr> for LiveKeys {
    type Output = Key;

    fn index(&self, key_addr: KeyAddr) -> &Self::Output {
        if key_addr.is_valid() {
            &self.key_map[key_addr]
        } else {
            &self.dummy
        }
    }
}

impl IndexMut<KeyAddr> for LiveKeys {
    fn index_mut(&mut self, key_addr: KeyAddr) -> &mut Self::Output {
        if key_addr.is_valid() {
            &mut self.key_map[key_addr]
        } else {
            self.dummy = Key_Masked;
            &mut self.dummy
        }
    }
}
