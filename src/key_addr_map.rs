use core::ops::{Index, IndexMut};

use crate::{Key, key_addr::KeyAddr};

/// A `KeyAddrMap` is a collection of objects, indexed by [KeyAddr], with one
/// entry per key on the keyboard.
pub struct KeyAddrMap<const N: usize> {
    values: [Key; N],
}

impl<const N: usize> KeyAddrMap<N> {
    pub const fn new() -> Self {
        Self {
            values: [Key::default(); N],
        }
    }

    /// Returns the number of [`Key`](crate::Key) entries in the array.
    pub const fn len() -> usize {
        N
    }

    /// Creates an iterator over the [`KeyAddrMap`].
    pub fn iter<'m>(&'m self) -> KeyAddrMapIter<'m> {
        self.into_iter()
    }

    /// Creates a mutable iterator over the [`KeyAddrMap`].
    pub fn iter_mut<'m>(&'m mut self) -> KeyAddrMapIterMut<'m> {
        self.into_iter()
    }
}

impl<const N: usize> Index<KeyAddr> for KeyAddrMap<N> {
    type Output = Key;

    /// To get the value of an entry:
    ///
    /// Example:
    ///
    /// ```rust
    /// use kaleidoscope::{key_addr::KeyAddr, key_addr_map::KeyAddrMap};
    ///
    /// let key_array: KeyAddrMap<10> = KeyAddrMap::new();
    /// let key_addr = KeyAddr::default();
    /// let _key = key_array[key_addr];
    /// ```
    fn index(&self, addr: KeyAddr) -> &Self::Output {
        let index: usize = addr.into();
        &self.values[index]
    }
}

impl<const N: usize> IndexMut<KeyAddr> for KeyAddrMap<N> {
    /// To get the value of an entry:
    ///
    /// Example:
    ///
    /// ```rust
    /// use kaleidoscope::{key_addr::KeyAddr, key_addr_map::KeyAddrMap};
    ///
    /// let key_array: KeyAddrMap<10> = KeyAddrMap::new();
    /// let key_addr = KeyAddr::default();
    /// key_array[key_addr] = Key::from_raw(0x1111);
    /// ```
    fn index_mut(&mut self, addr: KeyAddr) -> &mut Self::Output {
        let index: usize = addr.into();
        &mut self.values[index]
    }
}

impl<'m, const N: usize> Index<KeyAddr> for &'m KeyAddrMap<N> {
    type Output = Key;

    /// To get the value of an entry:
    ///
    /// Example:
    ///
    /// ```rust
    /// use kaleidoscope::{key_addr::KeyAddr, key_addr_map::KeyAddrMap};
    ///
    /// let key_array: KeyAddrMap<10> = KeyAddrMap::new();
    /// let key_addr = KeyAddr::default();
    /// let _key = key_array[key_addr];
    /// ```
    fn index(&self, addr: KeyAddr) -> &'m Self::Output {
        let index: usize = addr.into();
        &self.values[index]
    }
}

impl<'m, const N: usize> Index<KeyAddr> for &'m mut KeyAddrMap<N> {
    type Output = Key;

    /// To get the value of an entry:
    ///
    /// Example:
    ///
    /// ```rust
    /// use kaleidoscope::{key_addr::KeyAddr, key_addr_map::KeyAddrMap};
    ///
    /// let key_array: KeyAddrMap<10> = KeyAddrMap::new();
    /// let key_addr = KeyAddr::default();
    /// let _key = key_array[key_addr];
    /// ```
    fn index(&self, addr: KeyAddr) -> &Self::Output {
        let index: usize = addr.into();
        &self.values[index]
    }
}

impl<'m, const N: usize> IndexMut<KeyAddr>
    for &'m mut KeyAddrMap<N>
{
    fn index_mut(&mut self, addr: KeyAddr) -> &mut Self::Output {
        let index: usize = addr.into();
        &mut self.values[index]
    }
}

pub struct KeyAddrMapIter<'m> {
    index: KeyAddr,
    map_iter: core::slice::Iter<'m, Key>,
}

impl<'m> Iterator for KeyAddrMapIter<'m> {
    type Item = &'m Key;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;

        if usize::from(index) < self.map_iter.len() {
            self.index += 1;
            self.map_iter.nth(index.into())
        } else {
            None
        }
    }
}

impl<'m, const N: usize> IntoIterator for &'m KeyAddrMap<N> {
    type Item = &'m Key;
    type IntoIter = KeyAddrMapIter<'m>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            index: KeyAddr::new(0),
            map_iter: self.values.iter(),
        }
    }
}

pub struct KeyAddrMapIterMut<'m> {
    index: KeyAddr,
    map_iter: core::slice::IterMut<'m, Key>,
}

impl<'m> Iterator for KeyAddrMapIterMut<'m> {
    type Item = &'m mut Key;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;

        if usize::from(index) < self.map_iter.len() {
            self.index += 1;
            self.map_iter.nth(index.into())
        } else {
            None
        }
    }
}

impl<'m, const N: usize> IntoIterator for &'m mut KeyAddrMap<N> {
    type Item = &'m mut Key;
    type IntoIter = KeyAddrMapIterMut<'m>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            index: KeyAddr::new(0),
            map_iter: self.values.iter_mut(),
        }
    }
}
