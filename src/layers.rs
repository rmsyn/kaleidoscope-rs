use core::sync::atomic::{AtomicU8, Ordering};

use crate::driver::keyscanner::KeyScannerProps;
use crate::{Error, EventHandler, Hooks, Key, KeyAddr, KeyEvent, Key_NoKey, Key_Transparent, Result, shift_to_layer};
use crate::{KEYMAP_NEXT, KEYMAP_PREVIOUS, LAYER_MOVE_OFFSET, LAYER_SHIFT_OFFSET, LIVE_KEYS};
#[cfg(feature = "atreus")]
use crate::plugins::atreus::DeviceProps;

#[cfg(feature = "atreus")]
mod atreus;
#[cfg(feature = "atreus")]
pub use atreus::*;

pub const MAX_ACTIVE_LAYERS: usize = 16;
pub const NUM_KEYS: usize = DeviceProps::ROWS * DeviceProps::COLS;
pub const ZERO_LAYER_KEYMAP: [u8; NUM_KEYS] = [0u8; NUM_KEYS];

pub static LAYER_COUNT: AtomicU8 = AtomicU8::new(1);

/// Macro for defining the keymap. This should be used in the sketch
/// file (*.ino) to define the keymap[] array that holds the user's
/// layers. It also computes the number of layers in that keymap.
#[macro_export]
macro_rules! keymaps {
    {$keymap:ident, $keymaps:tt, $layers:tt} => {
        avr_progmem::progmem! {
            pub static progmem $keymap: [[$crate::key_defs::Key; $crate::layers::NUM_KEYS]; $layers] = $keymaps;
        }
    }
}

pub type ForEachHandler = fn(index: usize, layer: u8);

/// Represents active keymap layers.
///
/// Used to perform layer activation/deactivation, and other layer management functions.
///
/// For details on Kaleidoscope layer theory, see [Layers: kaleidoscope.readthedocs.io](https://kaleidoscope.readthedocs.io/en/latest/layers.html).
pub struct Layer {
    active_layer_count: usize,
    active_layers: [u8; MAX_ACTIVE_LAYERS],
    active_layer_keymap: [u8; NUM_KEYS],
}

impl Layer {
    /// Creates a new [Layer].
    pub const fn new() -> Self {
        Self {
            active_layer_count: 1,
            active_layers: [0u8; MAX_ACTIVE_LAYERS],
            active_layer_keymap: ZERO_LAYER_KEYMAP,
        }
    }

    /// Setup the active layers.
    pub fn setup(&mut self) {
        self.update_active_layers();
    }

    /// There are two lookup functions here, for historical reasons. Previously,
    /// Kaleidoscope would need to look up a value for each active keyswitch in
    /// every cycle, and pass that value on to the "event" handlers. Most of these
    /// lookups were for keys that were being held, not toggled on or off. Because
    /// these lookups were so frequent, a cache was used to speed them up.
    ///
    /// We no longer need to look up these values every cycle for keys that are
    /// held, because Kaleidoscope now only acts on key events that are actual
    /// toggle-on or toggle-off events, so the speed of the lookups here is not so
    /// critical. However, the old "live composite keymap" cache was also used by
    /// some plugins (and certain parts of Kaleidoscope itself) to override values
    /// in the keymap, and these plugins might use calls to `Layer.lookup()`,
    /// expecting to get the override values.
    ///
    /// Therefore, the `lookup()` function below first checks the `live_keys` array
    /// (the keyboard state array that has replaced the keymap cache). This should
    /// allow old code to continue working, until all the associated code (mostly
    /// the `onKeyswitchEvent()` handlers) is replaced, at which point we can
    /// remove dependence on `live_keys` entirely from this class.
    ///
    /// The `Runtime.lookup_key()` function replaces this one, for plugins that
    /// still want to do this same check.
    pub fn lookup_on_active_layer(&self, key_addr: &KeyAddr) -> Key {
        let layer = self.active_layer_keymap[key_addr.index()];
        self.key(layer as usize, key_addr)
    }

    /// Gets the active layer associated with the provided [KeyAddr].
    pub fn lookup_active_layer(&self, key_addr: &KeyAddr) -> u8 {
        self.active_layer_keymap[key_addr.index()]
    }

    /// Get a keymap [Key] from the PROGMEM keymap 2D-array.
    pub fn key(&self, layer: usize, key_addr: &KeyAddr) -> Key {
        if layer >= NUM_LAYERS || !key_addr.is_valid() {
            Key_NoKey
        } else {
            KEYMAP_LINEAR.load_at(layer)[key_addr.index()]
        }
    }

    /// Gets the current global layer count.
    pub fn layer_count(&self) -> usize {
        LAYER_COUNT.load(Ordering::Relaxed) as usize
    }

    /// Sets the global layer count.
    pub fn set_layer_count(&self, count: usize) {
        LAYER_COUNT.store(count as u8, Ordering::SeqCst);
    }

    /// Update the active layer keymap with all non-transparent keys 
    pub fn update_active_layers(&mut self) {
        // First, set every entry in the active layer keymap to point to the default
        // layer (layer 0).
        self.active_layer_keymap.copy_from_slice(ZERO_LAYER_KEYMAP.as_ref());

        // For each key address, set its entry in the active layer keymap to the value
        // of the top active layer that has a non-transparent entry for that address.
        for key_addr in KeyAddr::iter() {
            for i in (0..self.active_layer_count).rev() {
                let layer = self.unshifted(self.active_layers[i - 1]);
                let key = self.key(layer as usize, &key_addr);

                if key != Key_Transparent {
                    self.active_layer_keymap[key_addr.index()] = layer;
                    break;
                }
            }
        }
        // Even if there are no active layers (a situation that should be prevented by
        // `deactivate()`), each key will be mapped from the base layer (layer
        // 0). Likewise, for any address where all active layers have a transparent
        // entry, that key will be mapped from the base layer, even if the base layer
        // has been deactivated.
    }

    /// Handles layer key events.
    ///
    /// The caller is responsible for checking that the [KeyEvent] is for a [Layer] [Key],
    /// so we avoid checking it here.
    pub fn handle_layer_key_event(&mut self, event: KeyEvent) -> Result<()> {
        let mut key_code = event.key().key_code();

        if event.key().is_mod_layer_key() {
            key_code = (key_code / 8) + LAYER_SHIFT_OFFSET;
        }

        if key_code >= LAYER_MOVE_OFFSET {
            if event.state().key_toggled_on() {
                let target_layer = key_code - LAYER_MOVE_OFFSET;
                self.move_layer(target_layer)?;
            } else if key_code >= LAYER_SHIFT_OFFSET {
                let mut target_layer = key_code - LAYER_SHIFT_OFFSET;

                match target_layer {
                    KEYMAP_NEXT | KEYMAP_PREVIOUS => {
                        if event.state().key_toggled_on() {
                            let top_layer = self.unshifted(self.last_layer());

                            if target_layer == KEYMAP_NEXT {
                                target_layer = top_layer + 1;
                            } else {
                                target_layer = top_layer - 1;
                            }

                            if target_layer as usize >= self.layer_count() {
                                LIVE_KEYS.write().mask(*event.addr());
                                return Ok(());
                            }

                            let target_layer_shifted = target_layer + LAYER_SHIFT_OFFSET;
                            self.activate(target_layer_shifted)?;

                            // We can't just change `event.key` here because `LIVE_KEYS` has
                            // already been updated by the time `handle_layer_key_event` gets called.
                            LIVE_KEYS.write()[*event.addr()] = shift_to_layer(target_layer);
                        }
                    }
                    _ => {
                        target_layer += LAYER_SHIFT_OFFSET;

                        if event.state().key_toggled_on() {
                            if self.stack_position(target_layer).is_err() {
                                self.activate(target_layer)?;
                            }
                        } else {
                            self.deactivate(target_layer)?;
                        }
                    }

                }
            } else if event.state().key_toggled_on() {
                let target_layer = key_code;

                let mut top_locked_layer = (MAX_ACTIVE_LAYERS + 1) as u8;

                for &active_layer in self.active_layers[..self.active_layer_count].iter() {
                    if active_layer < LAYER_SHIFT_OFFSET {
                        top_locked_layer = active_layer;
                    }
                }

                // If the top locked layer is the target layer, we remove it from the stack.
                // Otherwise, we activate it.  We disregard shifted layers so that it's
                // possible to set up a layer toggle key on a shifted layer that will
                // actually deactivate the target layer as expected, with a single tap.
                if top_locked_layer == target_layer {
                    self.deactivate(target_layer)?;
                } else {
                    self.activate(target_layer)?;
                }
            }
        }

        Ok(())
    }

    /// Does pretty much what `activate` does, except we do everything
    /// unconditionally, to make sure all parts of the firmware are aware of the
    /// layer change.
    pub fn move_layer(&mut self, layer: u8) -> Result<()> {
        if layer as usize > self.layer_count() {
            return Ok(());
        }

        self.active_layer_count = 1;
        self.active_layers[0] = layer;

        self.update_active_layers();

        Hooks::on_layer_change()?;

        Ok(())
    }

    /// Activates the provided layer.
    pub fn activate(&mut self, layer: u8) -> Result<()> {
        // If we're trying to turn on a layer that doesn't exist, abort (but
        // if the keymap wasn't defined using the [keymaps](crate::keymaps) macro, proceed anyway
        let layer_unshifted = self.unshifted(layer);

        if layer_unshifted as usize >= self.layer_count() {
            return Ok(());
        }

        if let Ok(old_pos) = self.stack_position(layer) {
            self.remove(old_pos);
        }

        // Guarantee that we don't overflow by removing layers from the bottom if
        // we're about to exceed the size of the active layers array.
        while self.active_layer_count >= MAX_ACTIVE_LAYERS {
            self.remove(0);
        }

        // Otherwise, push it onto the active layer stack
        self.active_layers[self.active_layer_count] = layer;
        self.active_layer_count += 1;

        // Update the keymap cache (but not live_composite_keymap_; that gets
        // updated separately, when keys toggle on or off. See layers.h)
        self.update_active_layers();

        Hooks::on_layer_change()?;

        Ok(())
    }

    /// Deactivates the provided layer.
    ///
    /// Always leaves at least one layer active.
    pub fn deactivate(&mut self, layer: u8) -> Result<()> {
        let current_pos = self.stack_position(layer)?;

        // If the sole active layer is being deactivated, turn on the base layer and
        // return so we always have at least one layer active.
        if self.active_layer_count <= 1 {
            self.move_layer(0)?;
            return Ok(());
        }

        self.remove(current_pos);

        self.update_active_layers();

        Hooks::on_layer_change()?;

        Ok(())
    }

    /// Tests whether the provided layer is active.
    pub fn is_active(&self, layer: u8) -> bool {
        self.stack_position(layer).is_ok() && self.stack_position(layer + LAYER_SHIFT_OFFSET).is_ok()
    }

    /// Activates the next layer.
    pub fn activate_next(&mut self) -> Result<()> {
        self.activate(self.last_layer() + 1)
    }

    /// Deactivates the most recently activated layer.
    pub fn deactivate_most_recent(&mut self) -> Result<()> {
        let layer = self.last_layer();
        self.deactivate(layer)
    }

    /// Applies the provided handler to each active layer.
    pub fn for_each_active_layer(&self, h: ForEachHandler) {
        for i in 0..self.active_layer_count {
            let layer = self.unshifted(self.active_layers[i]);
            h(i, layer);
        }
    }

    fn last_layer(&self) -> u8 {
        self.active_layers[self.active_layer_count - 1]
    }

    fn unshifted(&self, mut layer: u8) -> u8 {
        if layer >= LAYER_SHIFT_OFFSET {
            layer -= LAYER_SHIFT_OFFSET;
        }
        layer
    }

    fn remove(&mut self, i: usize) {
        self.active_layers.copy_within((i+1)..(self.active_layer_count - (i + 1)), i);
        self.active_layer_count -= 1;
    }

    fn stack_position(&self, layer: u8) -> Result<usize> {
        for i in 0..self.active_layer_count {
            if self.active_layers[i] == layer {
                return Ok(i);
            }
        }

        Err(Error::Layer)
    }
}
