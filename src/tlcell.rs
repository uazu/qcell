use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashSet;
use std::marker::PhantomData;

use super::Invariant;

std::thread_local! {
    static SINGLETON_CHECK: std::cell::RefCell<HashSet<TypeId>> = std::cell::RefCell::new(HashSet::new());
}

struct NotSendOrSync(*const ());

/// Borrowing-owner of zero or more [`TLCell`](struct.TLCell.html)
/// instances.
///
/// See [crate documentation](index.html).
pub struct TLCellOwner<Q: 'static> {
    // Use NotSendOrSync to disable Send and Sync,
    not_send_or_sync: PhantomData<NotSendOrSync>,
    // use Invariant<Q> for invariant parameter, not influencing
    // other auto-traits, e.g. UnwindSafe (unlike other solutions like `*mut Q` or `Cell<Q>`)
    typ: PhantomData<Invariant<Q>>,
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
        Self {
            not_send_or_sync: PhantomData,
            typ: PhantomData,
        }
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
    pub fn ro<'a, T: ?Sized>(&'a self, tc: &'a TLCell<Q, T>) -> &'a T {
        unsafe { &*tc.value.get() }
    }

    /// Borrow contents of a `TLCell` mutably (read-write).  Only one
    /// `TLCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.
    #[inline]
    pub fn rw<'a, T: ?Sized>(&'a mut self, tc: &'a TLCell<Q, T>) -> &'a mut T {
        unsafe { &mut *tc.value.get() }
    }

    /// Borrow contents of two `TLCell` instances mutably.  Panics if
    /// the two `TLCell` instances point to the same memory.
    #[inline]
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        tc1: &'a TLCell<Q, T>,
        tc2: &'a TLCell<Q, U>,
    ) -> (&'a mut T, &'a mut U) {
        assert!(
            tc1 as *const _ as *const () as usize != tc2 as *const _ as *const () as usize,
            "Illegal to borrow same TLCell twice with rw2()"
        );
        unsafe { (&mut *tc1.value.get(), &mut *tc2.value.get()) }
    }

    /// Borrow contents of three `TLCell` instances mutably.  Panics if
    /// any pair of `TLCell` instances point to the same memory.
    #[inline]
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        tc1: &'a TLCell<Q, T>,
        tc2: &'a TLCell<Q, U>,
        tc3: &'a TLCell<Q, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert!(
            (tc1 as *const _ as *const () as usize != tc2 as *const _ as *const () as usize)
                && (tc2 as *const _ as *const () as usize != tc3 as *const _ as *const () as usize)
                && (tc3 as *const _ as *const () as usize != tc1 as *const _ as *const () as usize),
            "Illegal to borrow same TLCell twice with rw3()"
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
pub struct TLCell<Q, T: ?Sized> {
    // Use NotSendOrSync to disable Send
    // (and Sync, but `UnsafeCell` would already do that, too)
    not_send_or_sync: PhantomData<NotSendOrSync>,
    // use Invariant<Q> for invariant parameter, not influencing
    // other auto-traits, e.g. UnwindSafe (unlike other solutions like `*mut Q` or `Cell<Q>`)
    owner: PhantomData<Invariant<Q>>,
    value: UnsafeCell<T>,
}

impl<Q, T> TLCell<Q, T> {
    /// Create a new `TLCell` owned for borrowing purposes by the
    /// `TLCellOwner` derived from the same marker type `Q`.
    #[inline]
    pub const fn new(value: T) -> TLCell<Q, T> {
        TLCell {
            not_send_or_sync: PhantomData,
            owner: PhantomData,
            value: UnsafeCell::new(value),
        }
    }
}

impl<Q, T: ?Sized> TLCell<Q, T> {
    /// Borrow contents of this cell immutably (read-only).  Many
    /// `TLCell` instances can be borrowed immutably at the same time
    /// from the same owner.
    #[inline]
    pub fn ro<'a>(&'a self, owner: &'a TLCellOwner<Q>) -> &'a T {
        owner.ro(self)
    }

    /// Borrow contents of this cell mutably (read-write).  Only one
    /// `TLCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  To mutably borrow from two or three
    /// cells at the same time, see [`TLCellOwner::rw2`] or
    /// [`TLCellOwner::rw3`].
    #[inline]
    pub fn rw<'a>(&'a self, owner: &'a mut TLCellOwner<Q>) -> &'a mut T {
        owner.rw(self)
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
unsafe impl<Q, T: Send + ?Sized> Send for TLCell<Q, T> {}

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

    #[test]
    fn tlcell_unsized() {
        struct Marker;
        type ACellOwner = TLCellOwner<Marker>;
        type ACell<T> = TLCell<Marker, T>;
        let mut owner = ACellOwner::new();
        struct Squares(u32);
        struct Integers(u64);
        trait Series {
            fn step(&mut self);
            fn value(&self) -> u64;
        }
        impl Series for Squares {
            fn step(&mut self) {
                self.0 += 1;
            }
            fn value(&self) -> u64 {
                (self.0 as u64) * (self.0 as u64)
            }
        }
        impl Series for Integers {
            fn step(&mut self) {
                self.0 += 1;
            }
            fn value(&self) -> u64 {
                self.0
            }
        }
        fn series(init: u32, is_squares: bool) -> Box<ACell<dyn Series>> {
            if is_squares {
                Box::new(ACell::new(Squares(init)))
            } else {
                Box::new(ACell::new(Integers(init as u64)))
            }
        }

        let own = &mut owner;
        let cell1 = series(4, false);
        let cell2 = series(7, true);
        let cell3 = series(3, true);
        assert_eq!(cell1.ro(own).value(), 4);
        cell1.rw(own).step();
        assert_eq!(cell1.ro(own).value(), 5);
        assert_eq!(own.ro(&cell2).value(), 49);
        own.rw(&cell2).step();
        assert_eq!(own.ro(&cell2).value(), 64);
        let (r1, r2, r3) = own.rw3(&cell1, &cell2, &cell3);
        r1.step();
        r2.step();
        r3.step();
        assert_eq!(cell1.ro(own).value(), 6);
        assert_eq!(cell2.ro(own).value(), 81);
        assert_eq!(cell3.ro(own).value(), 16);
        let (r1, r2) = own.rw2(&cell1, &cell2);
        r1.step();
        r2.step();
        assert_eq!(cell1.ro(own).value(), 7);
        assert_eq!(cell2.ro(own).value(), 100);
    }
}
