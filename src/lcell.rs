use core::cell::UnsafeCell;
use core::marker::PhantomData;

use super::Invariant;
type Id<'id> = PhantomData<Invariant<&'id ()>>;

/// Borrowing-owner of zero or more [`LCell`](struct.LCell.html)
/// instances.
///
/// Use `LCellOwner::scope(|owner| ...)` to create an instance of this
/// type.
///
/// If the `generativity` feature is enabled, the
/// [`generativity`](https://crates.io/crates/generativity) crate can be used to
/// create a `Guard` for [`LCellOwner::new`]. This removes the requirment of
/// accessing an `LCellOwner` only inside closures.
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
    ///
    /// ```rust
    /// use qcell::{LCellOwner, LCell};
    /// LCellOwner::scope(|owner| {
    ///     let cell = LCell::new(100);
    ///     assert_eq!(cell.ro(&owner), &100);
    /// })
    /// ```
    pub fn scope<F>(f: F)
    where
        F: for<'scope_id> FnOnce(LCellOwner<'scope_id>),
    {
        f(Self { _id: PhantomData })
    }

    /// Create a new `LCellOwner` with a unique lifetime from a `Guard`.
    ///
    /// ```rust
    /// use qcell::{generativity::make_guard, LCellOwner, LCell};
    /// make_guard!(guard);
    /// let mut owner = LCellOwner::new(guard);
    /// let cell = LCell::new(100);
    /// assert_eq!(cell.ro(&owner), &100);
    /// ```
    #[cfg(feature = "generativity")]
    #[cfg_attr(docsrs, doc(cfg(feature = "generativity")))]
    pub fn new(_guard: generativity::Guard<'id>) -> Self {
        Self { _id: PhantomData }
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
    pub fn ro<'a, T: ?Sized>(&'a self, lc: &'a LCell<'id, T>) -> &'a T {
        unsafe { &*lc.value.get() }
    }

    /// Borrow contents of a `LCell` mutably (read-write).  Only one
    /// `LCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.
    #[inline]
    pub fn rw<'a, T: ?Sized>(&'a mut self, lc: &'a LCell<'id, T>) -> &'a mut T {
        unsafe { &mut *lc.value.get() }
    }

    /// Borrow contents of two `LCell` instances mutably.  Panics if
    /// the two `LCell` instances point to the same memory.
    #[inline]
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        lc1: &'a LCell<'id, T>,
        lc2: &'a LCell<'id, U>,
    ) -> (&'a mut T, &'a mut U) {
        assert!(
            lc1 as *const _ as *const () as usize != lc2 as *const _ as *const () as usize,
            "Illegal to borrow same LCell twice with rw2()"
        );
        unsafe { (&mut *lc1.value.get(), &mut *lc2.value.get()) }
    }

    /// Borrow contents of three `LCell` instances mutably.  Panics if
    /// any pair of `LCell` instances point to the same memory.
    #[inline]
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        lc1: &'a LCell<'id, T>,
        lc2: &'a LCell<'id, U>,
        lc3: &'a LCell<'id, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert!(
            (lc1 as *const _ as *const () as usize != lc2 as *const _ as *const () as usize)
                && (lc2 as *const _ as *const () as usize != lc3 as *const _ as *const () as usize)
                && (lc3 as *const _ as *const () as usize != lc1 as *const _ as *const () as usize),
            "Illegal to borrow same LCell twice with rw3()"
        );
        unsafe {
            (
                &mut *lc1.value.get(),
                &mut *lc2.value.get(),
                &mut *lc3.value.get(),
            )
        }
    }
}

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
pub struct LCell<'id, T: ?Sized> {
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

impl<'id, T: ?Sized> LCell<'id, T> {
    /// Borrow contents of this cell immutably (read-only).  Many
    /// `LCell` instances can be borrowed immutably at the same time
    /// from the same owner.
    #[inline]
    pub fn ro<'a>(&'a self, owner: &'a LCellOwner<'id>) -> &'a T {
        owner.ro(self)
    }

    /// Borrow contents of this cell mutably (read-write).  Only one
    /// `LCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  To mutably borrow from two or three
    /// cells at the same time, see [`LCellOwner::rw2`] or
    /// [`LCellOwner::rw3`].
    #[inline]
    pub fn rw<'a>(&'a self, owner: &'a mut LCellOwner<'id>) -> &'a mut T {
        owner.rw(self)
    }
}

// LCell already automatically implements Send, but not
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
unsafe impl<'id, T: Send + Sync + ?Sized> Sync for LCell<'id, T> {}

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
    #[cfg(feature = "generativity")]
    fn generativity() {
        generativity::make_guard!(guard);
        let mut owner = LCellOwner::new(guard);
        let c1 = LCell::new(100_u32);
        let c2 = LCell::new(200_u32);
        (*owner.rw(&c1)) += 1;
        (*owner.rw(&c2)) += 2;
        let c1ref = owner.ro(&c1);
        let c2ref = owner.ro(&c2);
        let total = *c1ref + *c2ref;
        assert_eq!(total, 303);
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

    #[test]
    fn lcell_unsized() {
        LCellOwner::scope(|mut owner| {
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
            fn series<'id>(init: u32, is_squares: bool) -> Box<LCell<'id, dyn Series>> {
                if is_squares {
                    Box::new(LCell::new(Squares(init)))
                } else {
                    Box::new(LCell::new(Integers(init as u64)))
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
        });
    }
}
