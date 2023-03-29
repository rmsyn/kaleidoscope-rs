const KEYSWITCH_STATE_MASK: u8 = 0b1000_0011;

bitfield! {
    /// Switch debouncing and status
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct KeyswitchState(u8);
    u8;
    pub injected, set_injected: 7;
    pub was_pressed, set_was_pressed: 0;
    pub is_pressed, set_is_pressed: 1;
}

impl From<u8> for KeyswitchState {
    fn from(k: u8) -> Self {
        Self(k & KEYSWITCH_STATE_MASK)
    }
}

impl KeyswitchState {
    /// Create a default KeyswitchState
    pub const fn default() -> Self {
        Self(0)
    }

    /// This is true if the key is pressed during this scan cycle.
    ///
    /// This will be true for several consecutive cycles even for a single tap of the key.
    ///
    /// Use this for events which should fire every scan cycle the key is held.
    /// If you want an event which fires only once when a key is pressed, use
    ///   [key_toggled_on](Self::key_toggled_on) or [key_toggled_off](Self::key_toggled_off) (defined below).
    pub fn key_is_pressed(&self) -> bool {
        self.is_pressed()
    }

    /// This is true if the key was pressed during the previous
    ///   scan cycle, regardless of whether it is pressed or not in this scan cycle.
    pub fn key_was_pressed(&self) -> bool {
        self.was_pressed()
    }

    /// This is true if the key is newly pressed during this scan
    ///   cycle, i.e. was not pressed in the previous scan cycle but is now.
    ///
    /// Use this for events which should fire exactly once per keypress, on a
    ///   "key-down" event.
    pub fn key_toggled_on(&self) -> bool {
        self.key_is_pressed() && !self.key_was_pressed()
    }

    /// This is true if the key is newly not-pressed during this
    ///   scan cycle, i.e. is not pressed now but was in the previous scan cycle.
    ///
    /// Use this for events which should fire exactly once per keypress, on a
    ///   "key-up" event.
    pub fn key_toggled_off(&self) -> bool {
        self.key_was_pressed() && !self.key_is_pressed()
    }

    /// This is true if the key was marked as injected by another
    /// plugin, i.e. it was generated artificially instead of corresponding to a
    ///   "real" keypress.
    pub fn key_is_injected(&self) -> bool {
        self.injected()
    }
}
