use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU64, Ordering, fence},
};

/// Atomically ref-counting indexed container for T
pub struct UnsafeArcArray<const N: usize, T> {
    ref_counts: [AtomicU64; N],
    items: [UnsafeCell<MaybeUninit<T>>; N],
}

// Safety: Can only be modified using unsafe
unsafe impl<const N: usize, T: Sync> Sync for UnsafeArcArray<N, T> {}

impl<const N: usize, T> UnsafeArcArray<N, T> {
    /// Initializes the first free element with the given function and returns its index if such
    /// element is present
    pub fn acquire_and_init(&self, init: impl FnOnce() -> T) -> Option<usize> {
        for (idx, rc) in self.ref_counts.iter().enumerate() {
            if rc
                .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                println!("new");
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
    pub unsafe fn get_ref(&self, index: usize) -> &T {
        unsafe { (&*self.items[index].get()).assume_init_ref() }
    }

    /// Decrements item's ref count and drops if no more references are left  
    /// # Safety
    /// dec_count should be called no more that once for each corresponding inc_count
    pub unsafe fn dec_count(&self, index: usize) {
        println!("dec_count");
        assert!(index < N);

        let prev_count =
            unsafe { self.ref_counts.get_unchecked(index) }.fetch_sub(1, Ordering::Relaxed);
        debug_assert!(prev_count > 0);
        if prev_count == 1 {
            // Safety: if no more references are left, the item should be dropped.
            fence(Ordering::Acquire);
            unsafe {
                (&mut *self.items.get_unchecked(index).get()).assume_init_drop();
            };
        }
        println!("dec_count finish");
    }

    /// Increments item's ref count
    pub fn inc_count(&self, index: usize) {
        println!("inc_count");
        self.ref_counts[index].fetch_add(1, Ordering::Relaxed);
    }

    pub const fn new() -> Self {
        Self {
            ref_counts: [const { AtomicU64::new(0) }; N],
            items: [const { UnsafeCell::new(MaybeUninit::uninit()) }; N],
        }
    }
}

impl<const N: usize, T> Default for UnsafeArcArray<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    struct DropSet {
        v: i64,
        cell: Rc<OnceCell<i64>>,
    }
    impl Drop for DropSet {
        fn drop(&mut self) {
            self.cell.set(self.v).unwrap()
        }
    }

    impl DropSet {
        fn new(v: i64) -> (Self, Rc<OnceCell<i64>>) {
            let cell = Rc::new(OnceCell::new());
            (
                Self {
                    v,
                    cell: cell.clone(),
                },
                cell,
            )
        }
    }
    use std::{cell::OnceCell, rc::Rc};

    use super::*;
    #[test]
    fn basic() {
        let arr = UnsafeArcArray::<11, DropSet>::new();
        let (ds, cell) = DropSet::new(11);
        let idx = arr.acquire_and_init(|| ds).unwrap();
        assert!(cell.get().is_none());
        unsafe {
            arr.dec_count(idx);
        }
        assert!(cell.get().is_some_and(|v| *v == 11));
    }

    // TODO: test better: multiple items, parallel access
}
