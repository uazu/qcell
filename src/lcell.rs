use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;

use crate::tuple::{ValidateUniqueness, LoadValues};

type Id<'id> = PhantomData<Cell<&'id mut ()>>;

/// Borrowing-owner of zero or more [`LCell`](struct.LCell.html)
/// instances.
///
/// Use `LCellOwner::scope(|owner| ...)` to create an instance of this
/// type.
///
/// The key piece of Rust syntax that enables this is `for<'id>`.
/// This allows creating an invariant lifetime within a closure, which
/// is different to any other Rust lifetime thanks to the techniques
/// explained in various places: section 6.3 of [this thesis from
/// Gankro](https://raw.githubusercontent.com/Gankro/thesis/master/thesis.pdf),
/// [this Reddit
/// post](https://www.reddit.com/r/rust/comments/3oo0oe/sound_unchecked_indexing_with_lifetimebased_value/),
/// and [this Rust playground
/// example](https://play.rust-lang.org/?gist=21a00b0e181a918f8ca4&version=stable).
/// Also see [this Reddit
/// comment](https://www.reddit.com/r/rust/comments/3aahl1/outside_of_closures_what_are_some_other_uses_for/csavac5/)
/// and its linked playground code.
///
/// `LCellOwner` uses a closure to contain the invariant lifetime.
/// However it's also worth noting the alternative approach used in
/// crate [`generativity`](https://crates.io/crates/generativity) that
/// uses a macro instead.
///
/// Some history: `GhostCell` by
/// [**pythonesque**](https://github.com/pythonesque) predates the
/// creation of `LCell`, and inspired it.  Discussion of `GhostCell`
/// on Reddit showed that a lifetime-based approach to cells was
/// feasible, but unfortunately the `ghost_cell.rs` source didn't seem
/// to be available under a community-friendly licence.  So I went
/// back to first principles and created `LCell` from `TCell` code,
/// combined with invariant lifetime code derived from the various
/// community sources that predate `GhostCell`.  Later `Send` and
/// `Sync` support for `LCell` was contributed independently.
///
/// See also [crate documentation](index.html).
pub struct LCellOwner<'id> {
    _id: Id<'id>,
}

impl<'id> LCellOwner<'id> {
    /// Create a new `LCellOwner`, with a new lifetime, that exists
    /// only within the scope of the execution of the given closure
    /// call.  If two scope calls are nested, then the two owners get
    /// different lifetimes.
    pub fn scope<F>(f: F)
    where
        F: for<'scope_id> FnOnce(LCellOwner<'scope_id>),
    {
        f(Self { _id: PhantomData })
    }

    /// Create a new cell owned by this owner instance.  See also
    /// [`LCell::new`].
    ///
    /// [`LCell::new`]: struct.LCell.html
    pub fn cell<T>(&self, value: T) -> LCell<'id, T> {
        LCell::<T>::new(value)
    }

    /// Borrow contents of a `LCell` immutably (read-only).  Many
    /// `LCell` instances can be borrowed immutably at the same time
    /// from the same owner.
    #[inline]
    pub fn ro<'a, T>(&'a self, lc: &'a LCell<'id, T>) -> &'a T {
        unsafe { &*lc.value.get() }
    }

    /// Borrow contents of a `LCell` mutably (read-write).  Only one
    /// `LCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.
    #[inline]
    pub fn rw<'a, T>(&'a mut self, lc: &'a LCell<'id, T>) -> &'a mut T {
        unsafe { &mut *lc.value.get() }
    }

    /// Borrow contents of two `LCell` instances mutably.  Panics if
    /// the two `LCell` instances point to the same memory.
    #[inline]
    pub fn rw2<'a, T, U>(
        &'a mut self,
        lc1: &'a LCell<'id, T>,
        lc2: &'a LCell<'id, U>,
    ) -> (&'a mut T, &'a mut U) {
        crate::rw!(self => lc1, lc2)
    }

    /// Borrow contents of three `LCell` instances mutably.  Panics if
    /// any pair of `LCell` instances point to the same memory.
    #[inline]
    pub fn rw3<'a, T, U, V>(
        &'a mut self,
        lc1: &'a LCell<'id, T>,
        lc2: &'a LCell<'id, U>,
        lc3: &'a LCell<'id, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        crate::rw!(self => lc1, lc2, lc3)
    }

    /// Borrow the contents of any number of `LCell` instances mutably.  Panics if
    /// any pair of `LCell` instances point to the same memory.
    #[inline]
    pub fn rw_generic<'a, T>(&'a mut self, lcells: T) -> T::Output
    where
        T: GenericLCellList<'id> + LoadValues<'a> + ValidateUniqueness
    {
        assert!(lcells.all_unique(), "Illegal to borrow same LCell multiple times");

        unsafe {
            lcells.load_values()
        }
    }
}

impl<T> crate::Sealed for LCell<'_, T> {}
unsafe impl<T> crate::tuple::GenericCell for LCell<'_, T> {
    type Value = T;

    fn rw_ptr(&self) -> *mut Self::Value {
        self.value.get()
    }
}

/// # Safety
/// 
/// Must only be implemented for type-lists of &LCell
pub unsafe trait GenericLCellList<'id>: crate::Sealed {}

unsafe impl GenericLCellList<'_> for crate::tuple::Nil {}
unsafe impl<'id, T, R> GenericLCellList<'id> for crate::tuple::Cons<&LCell<'id, T>, R>
where
    R: GenericLCellList<'id>
{}

/// Cell whose contents are owned (for borrowing purposes) by a
/// [`LCellOwner`].
///
/// To borrow from this cell, use the borrowing calls on the
/// [`LCellOwner`] instance that owns it, i.e. that shares the same
/// Rust lifetime.
///
/// See also [crate documentation](index.html).
///
/// [`LCellOwner`]: struct.LCellOwner.html
pub struct LCell<'id, T> {
    _id: Id<'id>,
    value: UnsafeCell<T>,
}

impl<'id, T> LCell<'id, T> {
    /// Create a new `LCell`.  The owner of this cell is inferred by
    /// Rust from the context.  So the owner lifetime is whatever
    /// lifetime is required by the first use of the new `LCell`.
    #[inline]
    pub fn new(value: T) -> LCell<'id, T> {
        LCell {
            _id: PhantomData,
            value: UnsafeCell::new(value),
        }
    }
}

// LCellOwner and LCell already automatically implement Send, but not
// Sync. We can add these implementations though, since it's fine to
// send a &LCell to another thread, and even mutably borrow the value
// there, as long as T is Send and Sync.
//
// The reason why LCell<T>'s impl of Sync requires T: Send + Sync
// instead of just T: Sync is that LCell provides interior mutability.
// If you send a &LCell<T> (and its owner) to a different thread, you
// can call .rw() to get a &mut T, and use std::mem::swap() to move
// the T, effectively sending the T to that other thread. That's not
// allowed if T: !Send.
//
// Note that the bounds on T for LCell<T>'s impl of Sync are the same
// as those of std::sync::RwLock<T>. That's not a coincidence.
// The way these types let you access T concurrently is the same,
// even though the locking mechanisms are different.
unsafe impl<'id> Sync for LCellOwner<'id> {}
unsafe impl<'id, T: Send + Sync> Sync for LCell<'id, T> {}

#[cfg(test)]
mod tests {
    use super::{LCell, LCellOwner};
    use std::rc::Rc;

    #[test]
    fn lcell() {
        LCellOwner::scope(|mut owner| {
            let c1 = LCell::new(100u32);
            let c2 = owner.cell(200u32);
            (*owner.rw(&c1)) += 1;
            (*owner.rw(&c2)) += 2;
            let c1ref = owner.ro(&c1);
            let c2ref = owner.ro(&c2);
            let total = *c1ref + *c2ref;
            assert_eq!(total, 303);
        });
    }

    #[test]
    #[should_panic]
    fn lcell_rw2() {
        LCellOwner::scope(|mut owner| {
            let c1 = Rc::new(LCell::new(100u32));
            let (mutref1, mutref2) = owner.rw2(&c1, &c1);
            *mutref1 += 1;
            *mutref2 += 1;
        });
    }

    #[test]
    #[should_panic]
    fn lcell_rw3_1() {
        LCellOwner::scope(|mut owner| {
            let c1 = Rc::new(LCell::new(100u32));
            let c2 = Rc::new(LCell::new(200u32));
            let (mutref1, mutref2, mutref3) = owner.rw3(&c1, &c1, &c2);
            *mutref1 += 1;
            *mutref2 += 1;
            *mutref3 += 1;
        });
    }

    #[test]
    #[should_panic]
    fn lcell_rw3_2() {
        LCellOwner::scope(|mut owner| {
            let c1 = Rc::new(LCell::new(100u32));
            let c2 = Rc::new(LCell::new(200u32));
            let (mutref1, mutref2, mutref3) = owner.rw3(&c1, &c2, &c1);
            *mutref1 += 1;
            *mutref2 += 1;
            *mutref3 += 1;
        });
    }

    #[test]
    #[should_panic]
    fn lcell_rw3_3() {
        LCellOwner::scope(|mut owner| {
            let c1 = Rc::new(LCell::new(100u32));
            let c2 = Rc::new(LCell::new(200u32));
            let (mutref1, mutref2, mutref3) = owner.rw3(&c2, &c1, &c1);
            *mutref1 += 1;
            *mutref2 += 1;
            *mutref3 += 1;
        });
    }
}
