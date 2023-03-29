use crate::{key_defs::Key, key_event::KeyEvent};

/// This is the set of return values for event handlers. Event handlers for
/// plugins are called in sequence by the corresponding hook function, in plugin
/// initialization order. The interpretation of these return values can vary
/// based on the needs of the hook function, but should be as follows:
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventHandlerError {
    /// Stop processing event handlers. The calling hook function
    /// should not call any further handlers, but may continue to take some
    /// actions to finish processing the event. This should be used to indicate
    /// that the event has been successfully handled.
    EventConsumed,
    /// Ignore the event. The calling hook function should not call any
    /// further handlers, and should treat the event as if it didn't
    /// happen. This should be used by plugin handlers that need to either
    /// suppress an event or queue the event in order to delay it.
    Abort,
    /// Undefined error. The calling hook function should not call any
    /// further handlers. There is currently no specification for what should
    /// happen if this is returned.
    Error,
}

/// Continue processing the event. The calling hook function should
/// continue calling next event handler in the sequence. If all event
/// handlers return `OK`, finish processing the event.
pub type Result<T> = core::result::Result<T, EventHandlerError>;

pub trait EventHandler {
    /// Called by Focus, when handling the `plugins` command.
    /// Should send the plugin name if that makes sense,
    /// but can be no-op.
    fn on_name_query() -> Result<&'static str> {
        Ok("")
    }

    /// Called on the setup of the device.
    fn on_setup() -> Result<()> {
        Ok(())
    }

    /// Called at the very start of each cycle, before gathering
    /// events, before doing anything else.
    fn before_each_cycle() -> Result<()> {
        Ok(())
    }

    /// Function called for every physical keyswitch event (toggle on or
    /// off). The `event` parameter is passed by reference so its key
    /// value can be modified. If it returns `Ok(())`, the
    /// next handler will be passed the event; otherwise Kaleidoscope
    /// will stop processing the event. Plugins that implement this
    /// handler must not process the same event id twice in order to
    /// prevent handler loops. Events may be aborted or queued for later
    /// release (by calling
    /// [`Runtime::handle_keyswitch_event()`](crate::runtime::Runtime::handle_keyswitch_event), but any
    /// plugin that does so must release events in ascending order,
    /// counting by ones.
    fn on_keyswitch_event(event: &mut KeyEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }

    /// Function called for every logical key event, including ones that
    /// originate from a physical keyswitch and ones that are injected
    /// by plugins. The `event` parameter is passed by reference so its
    /// key value can be modified. If it returns EventHandlerResult::OK,
    /// the next handler will be passed the event; otherwise
    /// Kaleidoscope will stop processing the event.
    fn on_key_event(event: &mut KeyEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }

    /// Called when a new set of HID reports (Keyboard, Consumer
    /// Control, and System Control) is being constructed in response to
    /// a key event. This is mainly useful for plugins that need to add
    /// values to HID reports based on special `Key` values other than
    /// the builtin ones.
    fn on_add_to_report(key: Key) -> Result<()> {
        let _ = key;
        Ok(())
    }

    /// Called by an external plugin (such as Kaleidoscope-FocusSerial)
    /// via [kaleidoscope::on_focus_event](Self::on_focus_event). This is where Focus events can
    /// be handled. The function can return EventHandlerResult::OK, and
    /// allow other plugins to handle the same command (with the caveat
    /// that arguments can only be parsed once), or
    /// [EventHandlerError::EventConsumed], in which case no other
    /// plugin will have a chance to react to the event.
    fn on_focus_event(input: &str) -> Result<()> {
        let _ = input;
        Ok(())
    }

    /// Called when the layer state changes. Which layes changed are
    /// not passed as arguments. If one needs that info, they should
    /// track Layer.getState() themselves.
    fn on_layer_change() -> Result<()> {
        Ok(())
    }

    /// Called when the LED mode changes. If one needs to know what
    /// from and what to the mode changed, they should track that
    /// themselves.
    fn on_led_mode_change() -> Result<()> {
        Ok(())
    }

    /// Called immediately before the LEDs get updated. This is for
    /// plugins that override the current LED mode.
    fn before_syncing_leds() -> Result<()> {
        Ok(())
    }

    /// Called before reporting our state to the host. This is the
    /// last point in a cycle where a plugin can alter what gets
    /// reported to the host.
    fn before_reporting_state(event: &KeyEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }

    /// Called after reporting our state to the host. This is the last
    /// point at which a plugin can do something in response to an event
    /// before the next event is processed, if multiple events occur in
    /// and are processed in a single cycle (usually due to delayed
    /// events or generated events).
    fn after_reporting_state(event: &KeyEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }

    /// Called at the very end of a cycle, after everything's
    /// said and done.
    fn after_each_cycle() -> Result<()> {
        Ok(())
    }

    /// Called before setup to enable plugins at compile time
    /// to explore the sketch.
    fn explore_sketch<Sketch>(sketch: Sketch) -> Result<()> {
        let _ = sketch;
        Ok(())
    }
}
