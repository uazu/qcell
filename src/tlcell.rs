use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashSet;
use std::marker::PhantomData;

use crate::tuple::{ValidateUniqueness, LoadValues};

std::thread_local! {
    static SINGLETON_CHECK: std::cell::RefCell<HashSet<TypeId>> = std::cell::RefCell::new(HashSet::new());
}

/// Borrowing-owner of zero or more [`TLCell`](struct.TLCell.html)
/// instances.
///
/// See [crate documentation](index.html).
pub struct TLCellOwner<Q: 'static> {
    // Use *const to disable Send and Sync
    typ: PhantomData<*const Q>,
}

impl<Q: 'static> Drop for TLCellOwner<Q> {
    fn drop(&mut self) {
        SINGLETON_CHECK.with(|set| set.borrow_mut().remove(&TypeId::of::<Q>()));
    }
}

impl<Q: 'static> Default for TLCellOwner<Q> {
    fn default() -> Self {
        TLCellOwner::new()
    }
}

impl<Q: 'static> TLCellOwner<Q> {
    /// Create the singleton owner instance.  Each owner may be used
    /// to create many `TLCell` instances.  There may be only one
    /// instance of this type per thread at any given time for each
    /// different marker type `Q`.  This call panics if a second
    /// simultaneous instance is created.  Since the owner is only
    /// valid to use in the thread it is created in, it does not
    /// support `Send` or `Sync`.
    pub fn new() -> Self {
        SINGLETON_CHECK.with(|set| {
            assert!(set.borrow_mut().insert(TypeId::of::<Q>()),
                    "Illegal to create two TLCellOwner instances within the same thread with the same marker type parameter");
        });
        Self { typ: PhantomData }
    }

    /// Create a new cell owned by this owner instance.  See also
    /// [`TLCell::new`].
    ///
    /// [`TLCell::new`]: struct.TLCell.html
    pub fn cell<T>(&self, value: T) -> TLCell<Q, T> {
        TLCell::<Q, T>::new(value)
    }

    /// Borrow contents of a `TLCell` immutably (read-only).  Many
    /// `TLCell` instances can be borrowed immutably at the same time
    /// from the same owner.
    #[inline]
    pub fn ro<'a, T>(&'a self, tc: &'a TLCell<Q, T>) -> &'a T {
        unsafe { &*tc.value.get() }
    }

    /// Borrow contents of a `TLCell` mutably (read-write).  Only one
    /// `TLCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.
    #[inline]
    pub fn rw<'a, T>(&'a mut self, tc: &'a TLCell<Q, T>) -> &'a mut T {
        unsafe { &mut *tc.value.get() }
    }

    /// Borrow contents of two `TLCell` instances mutably.  Panics if
    /// the two `TLCell` instances point to the same memory.
    #[inline]
    pub fn rw2<'a, T, U>(
        &'a mut self,
        tc1: &'a TLCell<Q, T>,
        tc2: &'a TLCell<Q, U>,
    ) -> (&'a mut T, &'a mut U) {
        crate::rw!(self => tc1, tc2)
    }

    /// Borrow contents of three `TLCell` instances mutably.  Panics if
    /// any pair of `TLCell` instances point to the same memory.
    #[inline]
    pub fn rw3<'a, T, U, V>(
        &'a mut self,
        tc1: &'a TLCell<Q, T>,
        tc2: &'a TLCell<Q, U>,
        tc3: &'a TLCell<Q, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        crate::rw!(self => tc1, tc2, tc3)
    }

    /// Borrow the contents of any number of `LCell` instances mutably.  Panics if
    /// any pair of `LCell` instances point to the same memory.
    #[inline]
    pub fn rw_generic<'a, T>(&'a mut self, tcells: T) -> T::Output
    where
        T: GenericTLCellList<Q> + LoadValues<'a> + ValidateUniqueness
    {
        assert!(tcells.all_unique(), "Illegal to borrow same TLCell multiple times");

        unsafe {
            tcells.load_values()
        }
    }
}

impl<Q, T> crate::Sealed for TLCell<Q, T> {}
unsafe impl<Q, T> crate::tuple::GenericCell for TLCell<Q, T> {
    type Value = T;

    fn rw_ptr(&self) -> *mut Self::Value {
        self.value.get()
    }
}

/// A marker trait that ensures that only `TLCells` get accepted for `TLCellOwner::rw_generic`
/// 
/// # Safety
/// 
/// Must only be implemented for type-lists of `&TLCell`
pub unsafe trait GenericTLCellList<Q>: crate::Sealed {}

unsafe impl<Q> GenericTLCellList<Q> for crate::tuple::Nil {}
unsafe impl<Q, T, R> GenericTLCellList<Q> for crate::tuple::Cons<&TLCell<Q, T>, R>
where
    R: GenericTLCellList<Q>
{}

/// Cell whose contents is owned (for borrowing purposes) by a
/// [`TLCellOwner`].
///
/// To borrow from this cell, use the borrowing calls on the
/// [`TLCellOwner`] instance that shares the same marker type.  Since
/// there may be another indistinguishable [`TLCellOwner`] in another
/// thread, `Sync` is not supported for this type.  However it *is*
/// possible to send the cell to another thread, which then allows its
/// contents to be borrowed using the owner in that thread.
///
/// See also [crate documentation](index.html).
///
/// [`TLCellOwner`]: struct.TLCellOwner.html
pub struct TLCell<Q, T> {
    // Use *const to disable Send and Sync
    owner: PhantomData<*const Q>,
    value: UnsafeCell<T>,
}

impl<Q, T> TLCell<Q, T> {
    /// Create a new `TLCell` owned for borrowing purposes by the
    /// `TLCellOwner` derived from the same marker type `Q`.
    #[inline]
    pub const fn new(value: T) -> TLCell<Q, T> {
        TLCell {
            owner: PhantomData,
            value: UnsafeCell::new(value),
        }
    }
}

// TLCell absolutely cannot be Sync, since otherwise you could send
// two &TLCell's to two different threads, that each have their own
// TLCellOwner<Q> instance and that could therefore both give out
// a &mut T to the same T.
//
// However, it's fine to Send a TLCell to a different thread, because
// you can only send something if nothing borrows it, so nothing can
// be accessing its contents. After sending the TLCell, the original
// TLCellOwner can no longer give access to the TLCell's contents since
// TLCellOwner is !Send + !Sync. Only the TLCellOwner of the new thread
// can give access to this TLCell's contents now.
unsafe impl<Q, T: Send> Send for TLCell<Q, T> {}

#[cfg(test)]
mod tests {
    use super::{TLCell, TLCellOwner};

    #[test]
    #[should_panic]
    fn tlcell_singleton_1() {
        struct Marker;
        let _owner1 = TLCellOwner::<Marker>::new();
        let _owner2 = TLCellOwner::<Marker>::new(); // Panic here
    }

    #[test]
    fn tlcell_singleton_2() {
        struct Marker;
        let owner1 = TLCellOwner::<Marker>::new();
        drop(owner1);
        let _owner2 = TLCellOwner::<Marker>::new();
    }

    #[test]
    fn tlcell_singleton_3() {
        struct Marker1;
        struct Marker2;
        let _owner1 = TLCellOwner::<Marker1>::new();
        let _owner2 = TLCellOwner::<Marker2>::new();
    }

    #[test]
    fn tlcell() {
        struct Marker;
        type ACellOwner = TLCellOwner<Marker>;
        type ACell<T> = TLCell<Marker, T>;
        let mut owner = ACellOwner::new();
        let c1 = ACell::new(100u32);
        let c2 = owner.cell(200u32);
        (*owner.rw(&c1)) += 1;
        (*owner.rw(&c2)) += 2;
        let c1ref = owner.ro(&c1);
        let c2ref = owner.ro(&c2);
        let total = *c1ref + *c2ref;
        assert_eq!(total, 303);
    }

    #[test]
    fn tlcell_threads() {
        struct Marker;
        type ACellOwner = TLCellOwner<Marker>;
        let mut _owner1 = ACellOwner::new();
        std::thread::spawn(|| {
            let mut _owner2 = ACellOwner::new();
        })
        .join()
        .unwrap();
    }
}
