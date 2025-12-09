use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{AtomicU64, Ordering, fence},
};

use ash;

/// Atomically ref-counting indexed container for T
struct ArcArray<const N: usize, T> {
    ref_counts: [AtomicU64; N],
    items: [UnsafeCell<MaybeUninit<T>>; N],
}

// Safety: Can only be modified using unsafe
unsafe impl<const N: usize, T: Sync> Sync for ArcArray<N, T> {}

impl<const N: usize, T> ArcArray<N, T> {
    /// Initializes the first free element with the given function and returns its index if such
    /// element is present
    fn accuire_and_init(&self, init: impl FnOnce() -> T) -> Option<usize> {
        for (idx, rc) in self.ref_counts.iter().enumerate() {
            if rc
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                // Safety: The item did not have references exept this one
                unsafe {
                    (&mut *self.items[idx].get()).write(init());
                }
                return Some(idx);
            }
        }
        None
    }

    /// Returns a reference to the item with the given index  
    /// # Safety
    /// should only call this with an index of an initialized and not dropped item
    unsafe fn get_ref(&self, index: usize) -> &T {
        unsafe { (&*self.items[index].get()).assume_init_ref() }
    }

    /// Decrements item's ref count and drops if no more references are left  
    /// # Safety
    /// dec_count should be called no more that once for each corresponding inc_count
    unsafe fn dec_count(&self, index: usize) {
        assert!(index < N);

        let prev_count =
            unsafe { self.ref_counts.get_unchecked(index) }.fetch_sub(1, Ordering::Relaxed);
        debug_assert!(prev_count > 0);
        if prev_count == 1 {
            // Safety: if no more references are left, the item should be dropped.
            fence(Ordering::Acquire);
            unsafe {
                drop(
                    std::mem::replace(
                        &mut *self.items.get_unchecked(index).get(),
                        MaybeUninit::uninit(),
                    )
                    .assume_init(),
                )
            };
        }
    }

    /// Increments item's ref count
    fn inc_count(&self, index: usize) {
        self.ref_counts[index].fetch_add(1, Ordering::Relaxed);
    }

    const fn new() -> Self {
        Self {
            ref_counts: [const { AtomicU64::new(0) }; N],
            items: [const { UnsafeCell::new(MaybeUninit::uninit()) }; N],
        }
    }
}

impl<const N: usize, T> Default for ArcArray<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

/// ash::Instance wrapper that destroys the Instance when dropped
struct RawInstance(ash::Instance);

impl Drop for RawInstance {
    fn drop(&mut self) {
        unsafe {
            self.0.destroy_instance(None);
        }
    }
}

impl Deref for RawInstance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

const MAX_INSTANCES: usize = 1;
static RAW_INSTANCES: ArcArray<MAX_INSTANCES, RawInstance> = ArcArray::new();

/// A handle to a RawInstance
pub struct Instance {
    id: usize,
}

impl Drop for Instance {
    fn drop(&mut self) {
        // Safety: Instance's existence guarantees that the RawInstance is valid
        unsafe {
            RAW_INSTANCES.dec_count(self.id);
        }
    }
}

impl Instance {
    /// # Safety
    /// The ash::Instance should not be destroyed
    pub unsafe fn get_raw_ref(&self) -> &ash::Instance {
        // Safety: Instance's existence guarantees that the RawInstance under its index is initialized
        unsafe { RAW_INSTANCES.get_ref(self.id) }
    }

    /// # Safety
    /// The ash::Instance should not be destroyed  
    /// # Panics
    /// Panics if the instance limit is reached
    pub unsafe fn from_raw(raw_instance: ash::Instance) -> Self {
        Self {
            id: RAW_INSTANCES
                .accuire_and_init(|| RawInstance(raw_instance))
                .expect("Failed to initialize instance (no free space)"),
        }
    }
}

impl Clone for Instance {
    fn clone(&self) -> Self {
        RAW_INSTANCES.inc_count(self.id);
        Self { id: self.id }
    }
}
