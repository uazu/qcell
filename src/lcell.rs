use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;

type Id<'id> = PhantomData<Cell<&'id mut ()>>;

/// Borrowing-owner of zero or more [`LCell`](struct.LCell.html)
/// instances.
///
/// Use `LCellOwner::scope(|owner| ...)` to create an instance of this
/// type.
///
/// This based around creating an invariant lifetime within the
/// closure, which is different to any other Rust lifetime thanks to
/// the techniques explained in 2015 in [this Reddit
/// post](https://www.reddit.com/r/rust/comments/3oo0oe/sound_unchecked_indexing_with_lifetimebased_value/),
/// and [this Rust playground
/// example](https://play.rust-lang.org/?gist=21a00b0e181a918f8ca4&version=stable).
/// Also see [this Reddit
/// comment](https://www.reddit.com/r/rust/comments/3aahl1/outside_of_closures_what_are_some_other_uses_for/csavac5/)
/// and its linked playground code.
///
/// This works in a similar way to a cell type known as `GhostCell` or
/// `ghost_cell`, but the invariant lifetime discussion above that
/// this code is based on predates the `GhostCell` implementation.
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

    /// Borrow contents of a `LCell` immutably.  Many `LCell`
    /// instances can be borrowed immutably at the same time from the
    /// same owner.
    #[inline]
    pub fn get<'a, T>(&'a self, lc: &'a LCell<'id, T>) -> &'a T {
        unsafe { &*lc.value.get() }
    }

    /// Borrow contents of a `LCell` mutably.  Only one `LCell` at a
    /// time can be borrowed from the owner using this call.  The
    /// returned reference must go out of scope before another can be
    /// borrowed.
    #[inline]
    pub fn get_mut<'a, T>(&'a mut self, lc: &'a LCell<'id, T>) -> &'a mut T {
        unsafe { &mut *lc.value.get() }
    }

    /// Borrow contents of two `LCell` instances mutably.  Panics if
    /// the two `LCell` instances point to the same memory.
    #[inline]
    pub fn get_mut2<'a, T, U>(
        &'a mut self,
        lc1: &'a LCell<'id, T>,
        lc2: &'a LCell<'id, U>,
    ) -> (&'a mut T, &'a mut U) {
        assert!(
            lc1 as *const _ as usize != lc2 as *const _ as usize,
            "Illegal to borrow same LCell twice with get_mut2()"
        );
        unsafe { (&mut *lc1.value.get(), &mut *lc2.value.get()) }
    }

    /// Borrow contents of three `LCell` instances mutably.  Panics if
    /// any pair of `LCell` instances point to the same memory.
    #[inline]
    pub fn get_mut3<'a, T, U, V>(
        &'a mut self,
        lc1: &'a LCell<'id, T>,
        lc2: &'a LCell<'id, U>,
        lc3: &'a LCell<'id, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert!(
            (lc1 as *const _ as usize != lc2 as *const _ as usize)
                && (lc2 as *const _ as usize != lc3 as *const _ as usize)
                && (lc3 as *const _ as usize != lc1 as *const _ as usize),
            "Illegal to borrow same LCell twice with get_mut3()"
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

#[cfg(test)]
mod tests {
    use super::{LCell, LCellOwner};
    use std::rc::Rc;

    #[test]
    fn lcell() {
        LCellOwner::scope(|mut owner| {
            let c1 = LCell::new(100u32);
            let c2 = owner.cell(200u32);
            (*owner.get_mut(&c1)) += 1;
            (*owner.get_mut(&c2)) += 2;
            let c1ref = owner.get(&c1);
            let c2ref = owner.get(&c2);
            let total = *c1ref + *c2ref;
            assert_eq!(total, 303);
        });
    }

    #[test]
    #[should_panic]
    fn lcell_get_mut2() {
        LCellOwner::scope(|mut owner| {
            let c1 = Rc::new(LCell::new(100u32));
            let (mutref1, mutref2) = owner.get_mut2(&c1, &c1);
            *mutref1 += 1;
            *mutref2 += 1;
        });
    }

    #[test]
    #[should_panic]
    fn lcell_get_mut3_1() {
        LCellOwner::scope(|mut owner| {
            let c1 = Rc::new(LCell::new(100u32));
            let c2 = Rc::new(LCell::new(200u32));
            let (mutref1, mutref2, mutref3) = owner.get_mut3(&c1, &c1, &c2);
            *mutref1 += 1;
            *mutref2 += 1;
            *mutref3 += 1;
        });
    }

    #[test]
    #[should_panic]
    fn lcell_get_mut3_2() {
        LCellOwner::scope(|mut owner| {
            let c1 = Rc::new(LCell::new(100u32));
            let c2 = Rc::new(LCell::new(200u32));
            let (mutref1, mutref2, mutref3) = owner.get_mut3(&c1, &c2, &c1);
            *mutref1 += 1;
            *mutref2 += 1;
            *mutref3 += 1;
        });
    }

    #[test]
    #[should_panic]
    fn lcell_get_mut3_3() {
        LCellOwner::scope(|mut owner| {
            let c1 = Rc::new(LCell::new(100u32));
            let c2 = Rc::new(LCell::new(200u32));
            let (mutref1, mutref2, mutref3) = owner.get_mut3(&c2, &c1, &c1);
            *mutref1 += 1;
            *mutref2 += 1;
            *mutref3 += 1;
        });
    }
}
