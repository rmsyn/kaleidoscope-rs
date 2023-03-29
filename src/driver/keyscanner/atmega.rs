use crate::device::{pins_and_ports::*, F_CPU};
use crate::driver::keyscanner::{base::Base, KeyScannerProps};
use crate::{key_addr::KeyAddr, key_defs::Key, util::bits::bit_read};
use crate::{RUNTIME, return_on_err, tc1, wdt};

use kaleidoscope_internal::driver::keyscanner::{Atmega as AtmegaInner, MatrixScanner};

#[cfg(feature = "atreus")]
use crate::plugins::atreus::DeviceProps;

/// Keyscanner implementation for Atmega-based platforms.
pub struct Atmega {
    inner: AtmegaInner,
}

impl Atmega {
    /// Creates a new [Atmega] key scanner.
    pub const fn new() -> Self {
        Self {
            inner: AtmegaInner::new(),
        }
    }

    /// Gets whether the scanner should scan the keys.
    pub fn do_scan(&self) -> bool {
        self.inner.do_scan()
    }

    /// Sets whether the scanner should scan the keys.
    pub fn set_do_scan(&mut self, do_scan: bool) {
        self.inner.set_do_scan(do_scan);
    }

    /// Setup the row and column pins for the key scanner.
    pub fn setup(&self) {
        assert!(
            DeviceProps::MATRIX_ROW_PINS.len() > 0,
            "The key scanner description has an empty array of matrix row pins."
        );
        assert!(
            DeviceProps::MATRIX_COL_PINS.len() > 0,
            "The key scanner description has an empty array of matrix column pins."
        );

        let wdt_lock = return_on_err!(wdt());

        avr_device::interrupt::free(|cs| {
            let wdt = wdt_lock.borrow(cs);

            // Reset the watchdog timer
            avr_device::asm::wdr();

            // Enable watchdog configuration mode
            wdt.wdtcsr
                .modify(|_, w| w.wdce().set_bit().wde().set_bit());

            // Disable watchdog timer
            wdt.wdtcsr.reset();
        });

        for pin in DeviceProps::MATRIX_COL_PINS {
            ddr_input(pin.into());
            enable_pullup(pin.into());
        }

        for pin in DeviceProps::MATRIX_ROW_PINS {
            ddr_output(pin.into());
            drive_output_high(pin.into());
        }

        self.set_scan_cycle_time(DeviceProps::KEYSCAN_INTERVAL);
    }

    /// Takes a value of between 0 and 8192.
    ///
    /// This corresponds (roughly) to the number of microseconds to wait between scanning the key matrix.
    ///
    /// Our debouncing algorithm does four checks before deciding that a result is valid.
    ///
    /// Most normal mechanical switches specify a 5ms debounce period. On an ATMega32U4, 1700 gets you about 5ms of debouncing.
    ///
    /// Because keycanning is triggered by an interrupt but not run in that interrupt, the actual amount of time between scans is prone to a little bit of jitter.
    pub fn set_scan_cycle_time(&self, interval: u16) {
        let tc1_lock = return_on_err!(tc1());

        avr_device::interrupt::free(|cs| {
            let tc1 = tc1_lock.borrow(cs);

            tc1.tccr1b.modify(|_, w| w.wgm1().bits(0b01));
            tc1.tccr1a.modify(|_, w| unsafe { w.bits(0) });

            let cycles = (F_CPU / 2_000_000) * interval as u32;
            tc1.icr1.modify(|_, w| w.bits(cycles as u16));

            tc1.tccr1b
                .write(|w| w.wgm1().bits(0b01).cs1().bits(0b01));
            tc1.timsk1.modify(|_, w| w.toie1().bit(true));
        });
    }

    /// Read the key matrix.
    pub fn read_matrix(&mut self) {
        let mut any_debounced_changes = 0u16;

        for (i, &row) in DeviceProps::MATRIX_ROW_PINS.iter().enumerate() {
            output_toggle(row.into());
            let hot_pins = self.read_cols();
            output_toggle(row.into());

            any_debounced_changes |= self.debounce(hot_pins, i);

            if any_debounced_changes != 0 {
                let matrix_len = self.inner.matrix_state().len();
                for i in 0..matrix_len {
                    let dbn_state = self.inner.matrix_state()[i].debouncer.debounced_state;
                    self.inner.matrix_state_mut()[i].current = dbn_state;
                }
            }
        }
    }

    /// In the C++ library, no loop unrolling + a nop instruction is used to slow down
    /// how the scanner reads each column. Because Rust does not support fine-grained control
    /// over loop unrolling, here we use a very short delay instead. Hopefully, the performance
    /// is similar.
    ///
    /// From the original:
    ///
    /// ```nobuild
    /// This function has loop unrolling disabled on purpose: we want to give the
    /// hardware enough time to produce stable PIN reads for us. If we unroll the
    /// loop, we will not have that, because even with the NOP, the codepath is too
    /// fast. If we don't have stable reads, then entire rows or columns will behave
    /// erratically.
    ///
    /// For this reason, we ask the compiler to not unroll our loop, which in turn,
    /// gives hardware enough time to produce stable reads, at the cost of a little
    /// bit of speed.
    ///
    /// Do not remove the attribute!
    /// ```
    pub fn read_cols(&self) -> u16 {
        let mut hot_pins = 0u16;
        for (i, col) in DeviceProps::MATRIX_COL_PINS.iter().enumerate() {
            // Should be roughly equivalent to no loop unrolling + a nop instruction...
            arduino_hal::delay_us(1);

            hot_pins |= (!read_pin((col << i).into())) as u16;
        }
        hot_pins
    }

    pub fn act_on_matrix_scan(&mut self) {
        for row in 0..DeviceProps::ROWS {
            for col in 0..DeviceProps::COLS {
                let matrix_state = self.inner.matrix_state();
                let key_state = (bit_read(matrix_state[row].previous as u8, col as u8) << 0)
                    | (bit_read(matrix_state[row].current as u8, col as u8) << 1);
                if key_state != 0 {
                    self.handle_keyswitch_event(
                        &mut RUNTIME.write(),
                        Key::default(),
                        KeyAddr::create(row as u8, col as u8),
                        key_state.into(),
                    );
                }
            }
            let current = self.inner.matrix_state()[row].current;
            self.inner.matrix_state_mut()[row].previous = current;
        }
    }

    fn debounce(&mut self, sample: u16, row: usize) -> u16 {
        let debouncer = &mut self.inner.matrix_state_mut()[row].debouncer;

        // Use xor to detect changes from last stable state:
        // if a key has changed, it's bit will be 1, otherwise 0
        let delta = sample ^ debouncer.debounced_state;

        // Increment counters and reset any unchanged bits:
        // increment bit 1 for all changed keys
        debouncer.db1 = (debouncer.db1 ^ debouncer.db0) & delta;
        // increment bit 0 for all changed keys
        debouncer.db0 = !debouncer.db0 & delta;

        // Calculate returned change set: if delta is still true
        // and the counter has wrapped back to 0, the key is changed.
        let changes = !(!delta | (debouncer.db0 | debouncer.db1));
        // Update state: in this case use xor to flip any bit that is true in changes.
        debouncer.debounced_state ^= changes;

        changes
    }
}

impl Base for Atmega {}

impl MatrixScanner for Atmega {
    /// Scans the key matrix if the internal flag is set to perform a scan.
    fn scan_matrix(&mut self) {
        if self.do_scan() {
            self.set_do_scan(false);
            self.read_matrix();
        }
        self.act_on_matrix_scan();
    }
}
