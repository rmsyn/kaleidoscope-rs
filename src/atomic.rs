use core::sync::atomic::{AtomicU8, Ordering};

pub struct AtomicU32 {
    inner: [AtomicU8; 4],
}

impl AtomicU32 {
    pub const fn new(n: u32) -> Self {
        let b = n.to_le_bytes();
        Self { inner: [AtomicU8::new(b[0]), AtomicU8::new(b[1]), AtomicU8::new(b[2]), AtomicU8::new(b[3])] }
    }

    pub fn load(&self, ordering: Ordering) -> u32 {
        u32::from_le_bytes([
                           self.inner[0].load(ordering),
                           self.inner[1].load(ordering),
                           self.inner[2].load(ordering),
                           self.inner[3].load(ordering),
        ])
    }

    pub fn store(&self, n: u32, ordering: Ordering) {
        let b = n.to_le_bytes();

        for (dst, &src) in self.inner.iter().zip(b.iter()) {
            dst.store(src, ordering);
        }
    }
}
