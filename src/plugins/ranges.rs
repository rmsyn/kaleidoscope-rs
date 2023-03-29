///! NOTE: We need to keep the ranges stable, and backwards compatible!
///!
///! When adding, removing, or changing ranges, make sure that existing ranges are
///! never accidentally moved. If migrating keycodes that weren't previously using
///! the range system, make sure you don't change the `Key` values. If an existing
///! `Key` value is changed, it won't be a problem for the Kaleidoscope sketch
///! itself, but it will cause problems for keymap entries stored in EEPROM
///! (i.e. Chrysalis keymap layers), which will not get updated by flashing the
///! firmware.
///!
///! When adding new `Key` values for plugins, to keep them backwards-compatible,
///! they must be added at the end of the range below (just before `SAFE_START`),
///! even if those values are logically related to existing ones. This is
///! important for compatibility with existing Chrysalis keymaps, despite the fact
///! that it makes the code more obtuse here.
use crate::key_defs::SYNTHETIC;

pub const MAX_CS_KEYS: u8 = 64;

pub const MACRO_FIRST: u16 = (SYNTHETIC as u16 | 0b0010_0000) << 8;
pub const MACRO_LAST: u16 = MACRO_FIRST + 255;

// Macro ranges pre-date Kaleidoscope-Ranges, so they're coming before
// ranges::FIRST, because we want to keep the keycodes backwards compatible.
// This is undesirable, because it prevents us from making a clear distinction
// between plugin key values and core key values. The magic number
// `0b00100000` is the old `IS_MACRO` key flags bit.
pub const FIRST: u16 = 0xc000;
pub const KALEIDOSCOPE_FIRST: u16 = FIRST;
pub const OS_FIRST: u16 = FIRST + 1;
pub const OSM_FIRST: u16 = OS_FIRST;
pub const OSM_LAST: u16 = OS_FIRST + 7;
pub const OSL_FIRST: u16 = OSM_LAST + 1;
pub const OSL_LAST: u16 = OSL_FIRST + 7;
pub const OS_LAST: u16 = OSL_LAST;
pub const DU_FIRST: u16 = OS_LAST + 1;
pub const DUM_FIRST: u16 = DU_FIRST;
pub const DUM_LAST: u16 = DUM_FIRST + (8 << 8);
pub const DUL_FIRST: u16 = DUM_LAST + 1;
pub const DUL_LAST: u16 = DUL_FIRST + (8 << 8);
pub const DU_LAST: u16 = DUL_LAST;
pub const TD_FIRST: u16 = DU_LAST + 1;
pub const TD_LAST: u16 = TD_FIRST + 15;
pub const LEAD_FIRST: u16 = TD_LAST + 1;
pub const LEAD_LAST: u16 = LEAD_FIRST + 7;
pub const CYCLE: u16 = LEAD_LAST + 1;
pub const SYSTER: u16 = CYCLE + 1;
pub const TT_FIRST: u16 = SYSTER + 1;
pub const TT_LAST: u16 = TT_FIRST + 255;
pub const STENO_FIRST: u16 = TT_LAST + 1;
pub const STENO_LAST: u16 = STENO_FIRST + 42;
pub const SC_FIRST: u16 = STENO_LAST + 1;
pub const SC_LAST: u16 = SC_FIRST + 1;
pub const REDIAL: u16 = SC_LAST + 1;
pub const TURBO: u16 = REDIAL + 1;
pub const DYNAMIC_MACRO_FIRST: u16 = TURBO + 1;
pub const DYNAMIC_MACRO_LAST: u16 = DYNAMIC_MACRO_FIRST + 31;
pub const OS_META_STICKY: u16 = DYNAMIC_MACRO_LAST + 1;
pub const OS_ACTIVE_STICKY: u16 = OS_META_STICKY + 1;
pub const OS_CANCEL: u16 = OS_ACTIVE_STICKY + 1;
pub const CS_FIRST: u16 = OS_CANCEL + 1;
pub const CS_LAST: u16 = CS_FIRST + MAX_CS_KEYS as u16;
pub const SAFE_START: u16 = CS_LAST + 1;
pub const KALEIDOSCOPE_SAFE_START: u16 = SAFE_START;
