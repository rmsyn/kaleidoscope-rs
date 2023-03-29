use core::sync::atomic::{AtomicBool, Ordering};

pub struct RawSpinLock(AtomicBool);

unsafe impl lock_api::RawRwLock for RawSpinLock {
    const INIT: RawSpinLock = RawSpinLock(AtomicBool::new(false));

    type GuardMarker = lock_api::GuardSend;

    fn lock_shared(&self) {
        while !self.try_lock_shared() {}
    }

    fn try_lock_shared(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    unsafe fn unlock_shared(&self) {
        self.0.store(false, Ordering::Release);
    }

    fn lock_exclusive(&self) {
        while !self.try_lock_shared() {}
    }

    fn try_lock_exclusive(&self) -> bool {
        self.0
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock_exclusive(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

pub type Spinlock<T> = lock_api::RwLock<RawSpinLock, T>;
pub type SpinlockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawSpinLock, T>;
pub type SpinlockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawSpinLock, T>;
