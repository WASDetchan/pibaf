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
    }

    /// Increments item's ref count
    pub fn inc_count(&self, index: usize) {
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
        cell: Rc<Cell<i64>>,
    }
    impl DropSet {
        fn set(&self, v: i64) {
            self.cell.set(v);
        }
    }
    impl Drop for DropSet {
        fn drop(&mut self) {
            self.cell.set(self.v)
        }
    }

    impl DropSet {
        fn new(v: i64) -> (Self, Rc<Cell<i64>>) {
            let cell = Rc::new(Cell::new(0));
            (
                Self {
                    v,
                    cell: cell.clone(),
                },
                cell,
            )
        }
    }
    use std::{cell::Cell, rc::Rc};

    use super::*;
    #[test]
    fn basic() {
        let arr = UnsafeArcArray::<11, DropSet>::default();
        let (ds, cell) = DropSet::new(11);
        let idx = arr.acquire_and_init(|| ds).unwrap();
        assert_eq!(cell.get(), 0);
        unsafe { arr.get_ref(idx).set(6) };
        assert_eq!(cell.get(), 6);
        unsafe {
            arr.dec_count(idx);
        }
        assert_eq!(cell.get(), 11);
    }

    #[test]
    fn fill_capacity() {
        let arr = UnsafeArcArray::<3, i64>::default();
        let idx1 = arr.acquire_and_init(|| 1);
        let idx2 = arr.acquire_and_init(|| 2);
        let idx3 = arr.acquire_and_init(|| 3);
        let idx4 = arr.acquire_and_init(|| 4);

        assert!(idx1.is_some());
        assert!(idx2.is_some());
        assert!(idx3.is_some());
        assert!(idx4.is_none());
    }

    // TODO: test better: multiple items, parallel access
}
