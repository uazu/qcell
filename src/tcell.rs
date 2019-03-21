use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Mutex;

lazy_static! {
    static ref SINGLETON_CHECK: Mutex<HashSet<TypeId>> = Mutex::new(HashSet::new());
}

/// Borrowing-owner of zero or more [`TCell`](struct.TCell.html)
/// instances.
///
/// See [crate documentation](index.html).
pub struct TCellOwner<Q: 'static> {
    typ: PhantomData<Q>,
}

impl<Q: 'static> Drop for TCellOwner<Q> {
    fn drop(&mut self) {
        SINGLETON_CHECK.lock().unwrap().remove(&TypeId::of::<Q>());
    }
}

impl<Q: 'static> TCellOwner<Q> {
    /// Create the singleton owner instance.  There may only be one
    /// instance of this type at any time for each different marker
    /// type `Q`.  This call panics if another instance is created.
    /// This may be used for creating many `TCell` instances.
    pub fn new() -> Self {
        assert!(
            SINGLETON_CHECK.lock().unwrap().insert(TypeId::of::<Q>()),
            "Illegal to create two TCellOwner instances with the same marker type parameter"
        );
        Self { typ: PhantomData }
    }

    /// Create a new cell owned by this owner instance.  See also
    /// [`TCell::new`].
    ///
    /// [`TCell::new`]: struct.TCell.html
    pub fn cell<T>(&self, value: T) -> TCell<Q, T> {
        TCell::<Q, T>::new(value)
    }

    /// Borrow contents of a `TCell` immutably (read-only).  Many
    /// `TCell` instances can be borrowed immutably at the same time
    /// from the same owner.
    #[inline]
    pub fn ro<'a, T>(&'a self, tc: &'a TCell<Q, T>) -> &'a T {
        unsafe { &*tc.value.get() }
    }

    /// Borrow contents of a `TCell` mutably (read-write).  Only one
    /// `TCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.
    #[inline]
    pub fn rw<'a, T>(&'a mut self, tc: &'a TCell<Q, T>) -> &'a mut T {
        unsafe { &mut *tc.value.get() }
    }

    /// Borrow contents of two `TCell` instances mutably.  Panics if
    /// the two `TCell` instances point to the same memory.
    #[inline]
    pub fn rw2<'a, T, U>(
        &'a mut self,
        tc1: &'a TCell<Q, T>,
        tc2: &'a TCell<Q, U>,
    ) -> (&'a mut T, &'a mut U) {
        assert!(
            tc1 as *const _ as usize != tc2 as *const _ as usize,
            "Illegal to borrow same TCell twice with rw2()"
        );
        unsafe { (&mut *tc1.value.get(), &mut *tc2.value.get()) }
    }

    /// Borrow contents of three `TCell` instances mutably.  Panics if
    /// any pair of `TCell` instances point to the same memory.
    #[inline]
    pub fn rw3<'a, T, U, V>(
        &'a mut self,
        tc1: &'a TCell<Q, T>,
        tc2: &'a TCell<Q, U>,
        tc3: &'a TCell<Q, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert!(
            (tc1 as *const _ as usize != tc2 as *const _ as usize)
                && (tc2 as *const _ as usize != tc3 as *const _ as usize)
                && (tc3 as *const _ as usize != tc1 as *const _ as usize),
            "Illegal to borrow same TCell twice with rw3()"
        );
        unsafe {
            (
                &mut *tc1.value.get(),
                &mut *tc2.value.get(),
                &mut *tc3.value.get(),
            )
        }
    }
}

/// Cell whose contents is owned (for borrowing purposes) by a
/// [`TCellOwner`].
///
/// To borrow from this cell, use the borrowing calls on the
/// [`TCellOwner`] instance that shares the same marker type.
///
/// See also [crate documentation](index.html).
///
/// [`TCellOwner`]: struct.TCellOwner.html
pub struct TCell<Q, T> {
    owner: PhantomData<Q>,
    value: UnsafeCell<T>,
}

impl<Q, T> TCell<Q, T> {
    /// Create a new `TCell` owned for borrowing purposes by the
    /// `TCellOwner` derived from the same marker type `Q`.
    #[inline]
    pub const fn new(value: T) -> TCell<Q, T> {
        TCell {
            owner: PhantomData,
            value: UnsafeCell::new(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TCell, TCellOwner};
    #[test]
    #[should_panic]
    fn tcell_singleton_1() {
        struct Marker;
        let _owner1 = TCellOwner::<Marker>::new();
        let _owner2 = TCellOwner::<Marker>::new(); // Panic here
    }

    #[test]
    fn tcell_singleton_2() {
        struct Marker;
        let owner1 = TCellOwner::<Marker>::new();
        drop(owner1);
        let _owner2 = TCellOwner::<Marker>::new();
    }

    #[test]
    fn tcell_singleton_3() {
        struct Marker1;
        struct Marker2;
        let _owner1 = TCellOwner::<Marker1>::new();
        let _owner2 = TCellOwner::<Marker2>::new();
    }

    #[test]
    fn tcell() {
        struct Marker;
        type ACellOwner = TCellOwner<Marker>;
        type ACell<T> = TCell<Marker, T>;
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
}
