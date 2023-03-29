use crate::key_addr::KeyAddr;
use crate::key_addr_map::{KeyAddrMap, KeyAddrMapIter, KeyAddrMapIterMut};

pub const UPPER_LIMIT: usize = KeyAddr::UPPER_LIMIT as usize;

pub type KeyMap = KeyAddrMap<UPPER_LIMIT>;
pub type KeyMapIter<'m> = KeyAddrMapIter<'m>;
pub type KeyMapIterMut<'m> = KeyAddrMapIterMut<'m>;
