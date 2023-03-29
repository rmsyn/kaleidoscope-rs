use avr_device::interrupt;

use crate::{hid, hid_mut, LAYER, LIVE_KEYS, error::Result, event_handler::{EventHandler, EventHandlerError}, hooks::Hooks, key_addr::KeyAddr, key_defs::*, key_event::KeyEvent, millis::millis, return_on_err};
use crate::driver::{mcu::Mcu, hid::base::keyboard::Keyboard};

#[cfg(feature = "atreus")]
use crate::plugins::atreus::Device;

// FIXME: impl
pub struct Runtime {
    device: Device,
    millis_at_cycle_start: u32,
    last_addr_toggled_on: KeyAddr,
    has_leds: bool,
}

impl Runtime {
    /// Creates a new runtime.
    pub const fn new() -> Self {
        let device = Device::new();
        let has_leds = Device::led_count() > 0;

        Self {
            device,
            millis_at_cycle_start: 0,
            last_addr_toggled_on: KeyAddr::default(),
            has_leds,
        }
    }

    /// Handles all component setup necessary for the firmware runtime.
    pub fn setup(&mut self) -> Result<()> {
        Device::setup();

        Hooks::on_setup()?;

        LIVE_KEYS.write().clear_all();

        LAYER.write().setup();

        Ok(())
    }

    /// Main execution loop for scanning keyswitch events, and updating internal state.
    pub fn main_loop(&mut self) {
        // FIXME: implement millis for atmega32u4
        self.millis_at_cycle_start = millis();

        if Device::poll_usb_reset() {
            return_on_err!(hid_mut()).keyboard_mut().on_usb_reset();
        }

        return_on_err!(Hooks::before_each_cycle());

        // Next, we scan the keyswitches. Any toggle-on or toggle-off events will
        // trigger a call to `handleKeyswitchEvent()`, which in turn will
        // (conditionally) result in a HID report. Note that each event gets handled
        // (and any resulting HID report(s) sent) as soon as it is detected. It is
        // possible for more than one event to be handled like this in any given
        // cycle, resulting in multiple HID reports, but guaranteeing that only one
        // event is being handled at a time.
        self.device.scan_matrix();

        return_on_err!(Hooks::after_each_cycle());
    }

    /// Gets a reference to the runtime device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Gets a mutable reference to the runtime device.
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    /// Handle a physical keyswitch event
    ///
    /// This method is called in response to physical keyswitch state changes. Its
    /// job is to call the `onKeyswitchEvent()` plugin handler functions, used by
    /// plugins that are particularly concerned about the timing of those
    /// events. It takes only one parameter, of type `KeyEvent`, which encapsulates
    /// the information about that event:
    ///
    /// - `event.key_addr`: The address of the key that was pressed or released.
    /// - `event.state`: The state of the keyswitch event (toggled on or off).
    /// - `event.key`: The `Key` value for this event.
    /// - `event.id`: A semi-unique ID value for the event.
    ///
    /// The ID value is used to help plugins that delay events to coordinate with
    /// each other so that they can avoid re-processing the same event, possibly
    /// causing endless loops.
    pub fn handle_keyswitch_event(&mut self, mut event: KeyEvent) {
        // This function strictly handles physical key events. Any event without a
        // valid `KeyAddr` gets ignored.
        if !event.addr().is_valid() {
            return;
        }

        // Ignore any (non-)event that's not a state change. This check should be
        // unnecessary, as we shouldn't call this function otherwise.
        if !(event.state().key_toggled_on() || event.state().key_toggled_off()) {
            return;
        }

        // Set the `Key` value for this event.
        if event.state().key_toggled_off() {
            // When a key toggles off, set the event's key value to whatever the key's
            // current value is in the live keys state array.
            event.set_key(LIVE_KEYS.read()[*event.addr()]);
            // If that key was masked, unmask it and return.
            if *event.key() == Key_Masked {
                LIVE_KEYS.write().clear(*event.addr());
                return;
            }
        } else if event.key() == &Key_Undefined {
            // When a key toggles on, unless the event already has a key value (i.e. we
            // were called by a plugin rather than `actOnMatrixScan()`), we look up the
            // value from the current keymap (overridden by `live_keys`).
            event.set_key(Self::lookup_key(event.addr()));
        }

        // Run the plugin event handlers
        //
        // We check the result from the plugin event handlers, and stop processing
        // if it was anything other than `OK`.
        if Hooks::on_keyswitch_event(&mut event).is_err() {
            return;
        }

        // If all the plugin handlers returned OK, we proceed to the next step in
        // processing the event.
        self.handle_key_event(&mut event);
    }

    /// Handle a logical key event
    ///
    /// This method triggers the handling of a logical "key event". Ususally that
    /// event is the result of a call to `handle_keyswitch_event()`, but it can also
    /// be called by plugins that need to generate extra events without a 1:1
    /// mapping to physical keyswitch state transitions.
    pub fn handle_key_event(&mut self, event: &mut KeyEvent) {
        // For events that didn't begin with `handleKeyswitchEvent()`, we need to look
        // up the `Key` value from the keymap (maybe overridden by `live_keys`).
        if event.addr().is_valid() {
            if event.state().key_toggled_off() || event.key() == &Key_Undefined {
                event.set_key(Self::lookup_key(event.addr()));
            }
        }

        // If any `on_key_event()` handler returns `Error::EventAbort`, we return before updating
        // the Live Keys state array; as if the event didn't happen.
        let result = Hooks::on_key_event(event);
        if result == Err(EventHandlerError::Abort) {
            return;
        }

        // Update the live keys array based on the new event.
        if event.addr().is_valid() {
            if event.state().key_toggled_off() {
                LIVE_KEYS.write().clear(*event.addr());
            } else {
                LIVE_KEYS.write().activate(*event.addr(), *event.key());
            }
        }

        let key = *event.key();

        // If any `on_key_event()` handler returned a value other than `OK`, stop
        // processing now. Likewise if the event's `Key` value is a no-op.
        if result.is_err() ||
            key == Key_Masked ||
            key == Key_NoKey ||
            key == Key_Undefined ||
            key == Key_Transparent {
                return;
            }

        // Built-in layer change keys are handled by the Layer object.
        if key.is_layer_key() || key.is_mod_layer_key() {
            return_on_err!(LAYER.write().handle_layer_key_event(*event));
        }

        // If the event is for a layer change key, there's no need to send a HID
        // report, so we return early.
        if key.is_layer_key() {
            return;
        }

        // The System Control HID report contains only one keycode, and gets sent
        // immediately on `pressSystemControl()` or `releaseSystemControl()`. This is
        // significantly different from the way the other HID reports work, where held
        // keys remain in effect for subsequent reports.
        if key.is_system_control_key() {
            interrupt::free(|_cs| {
                if event.state().key_toggled_on() {
                    return_on_err!(hid_mut()).press_system_control(key);
                } else {
                    return_on_err!(hid_mut()).release_system_control(key);
                }
            });
            return;
        }

        // Until this point, the old report was still valid. Now we construct the new
        // one, based on the contents of the `LIVE_KEYS` state array.
        self.prepare_keyboard_report(event);

        // Finally, send the new keyboard report
        self.send_keyboard_report(event);

        // Now that the report has been sent, let plugins act on it after the fact.
        // This is useful for plugins that need to react to an event, but must wait
        // until after that event is processed to do so.
        return_on_err!(Hooks::after_reporting_state(event));
    }

    /// Prepare a new set of USB HID reports
    ///
    /// This method gets called when a key event results in at least one new HID
    /// report being sent to the host, usually as a result of a call to
    /// `handle_key_event()`. It clears the keyboard report (after plugins have
    /// already responded to the new event that triggered the forthcoming report),
    /// then populates the new report based on the values stored in the `LIVE_KEYS`
    /// state array.
    pub fn prepare_keyboard_report(&mut self, event: &mut KeyEvent) {
        // before building the new report, start clean
        return_on_err!(return_on_err!(hid_mut()).release_all_keys());

        // Build report from composite keymap cache. This can be much more efficient
        // with a bitfield. What we should be doing here is going through the array
        // and checking for HID values (Keyboard, Consumer, System) and directly
        // adding them to their respective reports. This comes before the old plugin
        // hooks are called for the new event so that the report will be full complete
        // except for that new event.
        for key_addr in KeyAddr::iter() {
            // Skip this event's key addr; we will deal with that later. This is most
            // important in the case of a key release, because we can't safely remove
            // any keycode(s) added to the report later.
            if &key_addr == event.addr() {
                continue;
            }

            let key = LIVE_KEYS.read()[key_addr];

            // If the key is idle or masked, we can ignore it.
            if key == Key_Inactive || key == Key_Masked {
                continue;
            }

            self.add_to_report(key);
        }
    }

    /// Add keycode(s) to a USB HID report
    ///
    /// This method gets called from `prepare_keyboard_report()` to add keycodes
    /// corresponding to active keys in the `LIVE_KEYS` state array to the Keyboard
    /// & Consumer Control HID reports. It calls the `on_add_to_report()` plugin
    /// handlers first to give them a chance to abort.
    pub fn add_to_report(&mut self, mut key: Key) {
        if let Err(err) = Hooks::on_add_to_report(key) {
            if err == EventHandlerError::Abort {
                return;
            }
        }

        if key.is_mod_layer_key() {
            let modifier = key.key_code() % 8;
            key = Key::from_raw(Key_LeftControl.raw() + modifier as u16);
        }

        if key.is_keyboard_key() {
            // The only incidental Keyboard modifiers that are allowed are the ones on
            // the key that generated the event, so we strip any others before adding
            // them. This might turn out to be too simple to cover all the corner cases,
            // but the OS should be helpful and do most of the masking we want for us.
            if !key.is_keyboard_modifier() {
                key.set_flags(KeyFlags::NONE);
            }

            return_on_err!(hid_mut()).press_key(key);
            return;
        }

        if key.is_consumer_control_key() {
            return_on_err!(hid_mut()).press_consumer_control(key);
        }
    }

    /// Send the new USB HID report(s)
    ///
    /// This method is called by `handle_key_event()` after `prepare_keyboard_report()`
    /// is done. It uses the information about the new event to guard against
    /// modifier and mod-flags rollover issues, and calls the
    /// `before_reporting_state()` plugin handler functions before sending the
    /// complete Keyboard and Consumer Control HID reports.
    pub fn send_keyboard_report(&mut self, event: &mut KeyEvent) {
        // If the keycode for this key is already in the report, we need to send an
        // extra report without that keycode in order to correctly process the
        // rollover. It might be better to exempt modifiers from this rule, but it's
        // not clear that would be better.
        if event.state().key_toggled_on() && event.key().is_keyboard_key() {
            // last keyboard key toggled on
            self.last_addr_toggled_on = *event.addr();

            if return_on_err!(hid()).is_key_pressed(event.key()) {
                // The keycode (flags ignored) for `event.key` is active in the current
                // report. Should this be `wasKeyPressed()` instead? I don't think so,
                // because (if I'm right) the new event hasn't been added yet.
                return_on_err!(hid_mut()).release_key(*event.key());
                return_on_err!(return_on_err!(hid_mut()).send_report());
            }

            if event.key().flags() != KeyFlags::NONE {
                // The keycode (flags ignored) for `event.key` is active in the current
                // report. Should this be `wasKeyPressed()` instead? I don't think so,
                // because (if I'm right) the new event hasn't been added yet.
                return_on_err!(hid_mut()).press_modifiers(*event.key());
                return_on_err!(return_on_err!(hid_mut()).send_report());
            }
        } else if event.addr() != self.last_addr_toggled_on() {
            // (not a keyboard key OR toggled off) AND not last keyboard key toggled on
            let last_key = LIVE_KEYS.read()[self.last_addr_toggled_on];
            if last_key.is_keyboard_key() {
                return_on_err!(hid_mut()).press_modifiers(last_key);
            }
        }

        if event.state().key_toggled_on() {
            self.add_to_report(*event.key());
        }

        // Call new pre-report handlers:
        if let Err(err) = Hooks::before_reporting_state(event) {
            if err == EventHandlerError::Abort {
                return;
            }
        }

        // Finally, send the report:
        return_on_err!(return_on_err!(hid_mut()).send_report());
    }

    /// Gets the current value of a keymap entry.
    ///
    /// Returns the `Key` value for a given `KeyAddr` entry in the current keymap,
    /// overridden by any active entry in the `live_keys` array.
    pub fn lookup_key(key_addr: &KeyAddr) -> Key {
        // First, check for an active key value in the `live_keys` array.
        let mut key = LIVE_KEYS.read()[*key_addr];

        // If that entry is clear, look up the entry from the active keymap layers.
        if key == Key_Transparent {
            key = LAYER.read().lookup_on_active_layer(key_addr);
        }

        key
    }

    /// Detaching from / attaching to the host.
    ///
    /// These two functions wrap the hardware plugin's similarly named functions.
    /// We wrap them, because we'd like plugins and user-code not having to use
    /// `Runtime.device()` directly.
    ///
    /// The methods themselves implement detaching from / attaching to the host,
    /// without rebooting the device, and remaining powered in between.
    ///
    /// Intended to be used in cases where we want to change some settings between
    /// detach and attach.
    pub fn detach_from_host() {
        return_on_err!(Device::detach_from_host());
    }

    /// Detaching from / attaching to the host.
    ///
    /// These two functions wrap the hardware plugin's similarly named functions.
    /// We wrap them, because we'd like plugins and user-code not having to use
    /// `Runtime.device()` directly.
    ///
    /// The methods themselves implement detaching from / attaching to the host,
    /// without rebooting the device, and remaining powered in between.
    ///
    /// Intended to be used in cases where we want to change some settings between
    /// detach and attach.
    pub fn attach_to_host() {
        return_on_err!(Device::attach_to_host());
    }

    /// Gets the milliseconds at cycle start.
    pub fn millis_at_cycle_start(&self) -> u32 {
        self.millis_at_cycle_start
    }

    /// Gets the last key address toggled on.
    pub fn last_addr_toggled_on(&self) -> &KeyAddr {
        &self.last_addr_toggled_on
    }

    /// Gets whether the device has LEDs.
    pub fn has_leds(&self) -> bool {
        self.has_leds
    }

    pub fn on_focus_event(input: &str) -> Result<()> {
        Hooks::on_focus_event(input).map_err(|err| err.into())
    }
}
