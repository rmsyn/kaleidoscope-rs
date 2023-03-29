use crate::{
    key_defs::Key,
    key_event::{KeyEvent, KeyEventOps},
    keyswitch_state::KeyswitchState,
    runtime::Runtime,
};

pub trait Base {
    fn handle_keyswitch_event(
        &self,
        runtime: &mut Runtime,
        _key: Key,
        key_addr: <KeyEvent as KeyEventOps>::KeyAddr,
        key_state: KeyswitchState,
    ) {
        if key_state.key_toggled_on() || key_state.key_toggled_off() {
            let event = KeyEvent::next(key_addr, key_state);
            runtime.handle_keyswitch_event(event);
        }
    }
}
