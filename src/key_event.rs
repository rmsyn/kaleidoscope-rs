use core::{
    ops::Add,
    sync::atomic::{AtomicI8, Ordering},
};

use crate::{key_addr::KeyAddr, key_defs::Key, keyswitch_state::KeyswitchState};

static LAST_ID: AtomicI8 = AtomicI8::new(0);

/// It's important that this is a signed integer, not unsigned.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyEventId(i8);

impl KeyEventId {
    pub const fn default() -> Self {
        Self(0)
    }
}

impl Add for &KeyEventId {
    type Output = KeyEventId;
    fn add(self, oth: Self) -> Self::Output {
        KeyEventId(self.0 + oth.0)
    }
}

impl Add<i8> for &KeyEventId {
    type Output = KeyEventId;

    fn add(self, oth: i8) -> Self::Output {
        KeyEventId(self.0 + oth)
    }
}

impl Add for KeyEventId {
    type Output = Self;
    fn add(self, oth: Self) -> Self::Output {
        KeyEventId(self.0 + oth.0)
    }
}

impl Add<i8> for KeyEventId {
    type Output = Self;

    fn add(self, oth: i8) -> Self {
        KeyEventId(self.0 + oth)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyEvent {
    addr: KeyAddr,
    state: KeyswitchState,
    key: Key,
    last_id: KeyEventId,
    id: KeyEventId,
}

impl KeyEvent {
    pub fn new() -> Self {
        Self {
            addr: KeyAddr::default(),
            state: KeyswitchState::default(),
            key: Key::default(),
            last_id: KeyEventId::default(),
            id: KeyEventId(LAST_ID.load(Ordering::Relaxed)),
        }
    }

    /// For use by keyscanner creating a new event from a physical keyswitch toggle on or off.
    pub fn next(addr: KeyAddr, state: KeyswitchState) -> Self {
        let id = LAST_ID.load(Ordering::Relaxed) + 1;
        LAST_ID.store(id, Ordering::SeqCst);

        Self {
            addr,
            state,
            key: Key::default(),
            last_id: KeyEventId::default(),
            id: KeyEventId(id),
        }
    }

    /// Get the key address
    pub fn addr(&self) -> &KeyAddr {
        &self.addr
    }

    /// Get the keyswitch state
    pub fn state(&self) -> KeyswitchState {
        self.state
    }

    /// Gets the [Key].
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Sets the [Key].
    pub fn set_key(&mut self, key: Key) {
        self.key = key;
    }

    /// Get the last key event ID
    pub fn last_id(&self) -> KeyEventId {
        self.last_id
    }

    /// Get the current key event ID
    pub fn id(&self) -> KeyEventId {
        self.id
    }
}

impl Default for KeyEvent {
    fn default() -> Self {
        Self::new()
    }
}

pub trait KeyEventOps {
    type Output;
    type KeyAddr;

    fn next_event(&self, addr: Self::KeyAddr, state: KeyswitchState) -> Self::Output;
}

impl KeyEventOps for KeyEvent {
    type Output = KeyEvent;
    type KeyAddr = KeyAddr;

    fn next_event(&self, addr: Self::KeyAddr, state: KeyswitchState) -> Self::Output {
        Self::next(addr, state)
    }
}
