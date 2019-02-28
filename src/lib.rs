//! Statically-checked alternatives to [`RefCell`].
//!
//! [`QCell`] is a cell type where the cell contents are logically
//! 'owned' for borrowing purposes by an instance of an owner type,
//! [`QCellOwner`].  So the cell contents can only be accessed by
//! making borrowing calls on that owner.  This behaves similarly to
//! borrowing fields from a structure, or borrowing elements from a
//! `Vec`.  However actually the only link between the objects is that
//! a reference to the owner instance was provided when the cell was
//! created.  Effectively the borrowing-owner and dropping-owner are
//! separated.
//!
//! This enables a pattern where the compiler can statically check
//! mutable access to data stored behind `Rc` references.  This
//! pattern works as follows: The owner is kept on the stack and a
//! mutable reference to it is passed to calls (for example as part of
//! a context structure).  This is fully checked at compile-time by
//! the borrow checker.  Then this static borrow checking is extended
//! to the cell contents (behind `Rc`s) through using borrowing calls
//! on the owner instance to access the cell contents.  This gives a
//! compile-time guarantee that access to the cell contents is safe.
//!
//! The alternative would be to use [`RefCell`], which panics if two
//! mutable references to the same data are attempted.  With [`RefCell`]
//! there are no warnings or errors to detect the problem at
//! compile-time.  On the other hand, using [`QCell`] the error is
//! detected, but the restrictions are much stricter than they really
//! need to be.  For example it's not possible to borrow from more
//! than a few different cells at the same time if they are protected
//! by the same owner, which [`RefCell`] would allow (correctly).
//! However if you are able to work within these restrictions (e.g. by
//! keeping borrows active only for a short time), then the advantage
//! is that there can never be a panic due to erroneous use of
//! borrowing, because everything is checked by the compiler.
//!
//! Apart from [`QCell`] and [`QCellOwner`], this crate also provides
//! [`TCell`] and [`TCellOwner`] which work the same but use the type
//! system instead of owner IDs.  See the ["Comparison of cell
//! types"](#comparison-of-cell-types) below.
//!
//! # Examples
//!
//! With [`RefCell`], this compiles but panics:
//!
//! ```should_panic
//!# use std::rc::Rc;
//!# use std::cell::RefCell;
//! let item = Rc::new(RefCell::new(Vec::<u8>::new()));
//! let mut iref = item.borrow_mut();
//! test(&item);
//! iref.push(1);
//!
//! fn test(item: &Rc<RefCell<Vec<u8>>>) {
//!     item.borrow_mut().push(2);    // Panics here
//! }
//! ```
//!
//! With [`QCell`], it refuses to compile:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//! let mut owner = QCellOwner::new();
//!
//! let item = Rc::new(QCell::new(&owner, Vec::<u8>::new()));
//! let iref = owner.get_mut(&item);
//! test(&mut owner, &item);    // Compile error
//! iref.push(1);
//!
//! fn test(owner: &mut QCellOwner, item: &Rc<QCell<Vec<u8>>>) {
//!     owner.get_mut(&item).push(2);
//! }
//! ```
//!
//! The solution in both cases is to make sure that the `iref` is not
//! active when the call is made, but [`QCell`] uses standard
//! compile-time borrow-checking to force the bug to be fixed.  This
//! is the main advantage of using these types.
//!
//! Here's a working version using [`TCell`] instead:
//!
//! ```
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//! struct Marker;
//! type ACell<T> = TCell<Marker, T>;
//! type ACellOwner = TCellOwner<Marker>;
//! let mut owner = ACellOwner::new();
//!
//! let item = Rc::new(ACell::new(&owner, Vec::<u8>::new()));
//! let iref = owner.get_mut(&item);
//! iref.push(1);
//! test(&mut owner, &item);
//!
//! fn test(owner: &mut ACellOwner, item: &Rc<ACell<Vec<u8>>>) {
//!     owner.get_mut(&item).push(2);
//! }
//! ```
//!
//! # Why this is safe
//!
//! This is the reasoning behind declaring this crate's interface
//! safe:
//!
//! - Between the cell creation and destruction, the only way to
//! access the contents (for read or write) is through the
//! borrow-owner instance.  So the borrow-owner is the exclusive
//! gatekeeper of this data.
//!
//! - The borrowing calls require a `&` owner reference to return a
//! `&` cell reference, or a `&mut` on the owner to return a `&mut`.
//! So this is the same kind of borrow on both sides.  The only borrow
//! we allow for the cell is the borrow that Rust allows for the
//! borrow-owner, and while that borrow is active, the borrow-owner
//! and the cell's reference are blocked from further incompatible
//! borrows.  The contents of the cells act as if they were owned by
//! the borrow-owner, just like elements within a `Vec`.  So Rust's
//! guarantees are maintained.
//!
//! - The borrow-owner has no control over when the cell's contents
//! are dropped, so the borrow-owner cannot act as a gatekeeper to the
//! data at that point.  However this cannot clash with any active
//! borrow on the data because whilst a borrow is active, the
//! reference to the cell is effectively locked by Rust's borrow
//! checking.  If this is behind an `Rc`, then it's impossible for the
//! last strong reference to be released until that borrow is
//! released.
//!
//! If you can see a flaw in this reasoning or in the code, please
//! raise an issue, preferably with test code which demonstrates the
//! problem.  MIRI in the Rust playground can report on some kinds of
//! unsafety.
//!
//! # Comparison of cell types
//!
//! This includes the Ghost Cell which can be found in
//! [ghost_cell.rs](https://github.com/ppedrot/kravanenn/blob/master/src/util/ghost_cell.rs)
//! or alternatively
//! [ghost_cell.rs](https://github.com/pythonesque/kravanenn/blob/wip/src/util/ghost_cell.rs).
//! This is based around lifetimes and looks neat, but seems to
//! involve a lot of lifetime annotations, for example
//! [HERE](https://github.com/ppedrot/kravanenn/blob/master/src/coq/checker/closure.rs).
//! This needs further investigation.  Possibly it could be
//! incorporated into this crate later.
//!
//! [`RefCell`] pros and cons:
//!
//! - Pro: Simple
//! - Pro: Allows very flexible borrowing patterns
//! - Con: No compile-time borrowing checks
//! - Con: Can panic due to distant code changes
//! - Con: Runtime borrow checks and some cell space overhead
//!
//! [`QCell`] pros and cons:
//!
//! - Pro: Simple
//! - Pro: Compile-time borrowing checks
//! - Pro: Dynamic owner creation
//! - Con: Can only borrow up to 3 objects at a time
//! - Con: Runtime owner checks and some cell space overhead
//!
//! [`TCell`] pros and cons:
//!
//! - Pro: Compile-time borrowing checks
//! - Pro: No overhead at runtime for borrowing or ownership checks
//! - Pro: No cell space overhead
//! - Con: Can only borrow up to 3 objects at a time
//! - Con: Uses singletons, so reusable code must be parameterised
//! with an external marker type
//!
//! [`GhostCell`] pros and cons:
//! - Pro: Compile-time borrowing checks
//! - Pro: No overhead at runtime for borrowing or ownership checks
//! - Pro: No cell space overhead
//! - Pro: No need for singletons
//! - Con: Can only borrow one object at a time (could be extended to 3 like `TCell`)
//! - Con: Uses lifetimes, so perhaps requires a lot of lifetime annotations (needs investigating)
//!
//! Cell | Owner ID | Cell overhead | Borrow check | Owner check
//! ---|---|---|---|---
//! `RefCell` | n/a | `usize` | Runtime | n/a
//! `QCell` | integer | `u32` | Compile-time | Runtime
//! `TCell` | marker type | none | Compile-time | Compile-time
//! `GhostCell` | lifetime | none | Compile-time | Compile-time
//!
//! Owner ergonomics:
//!
//! Cell | Owner type | Owner creation
//! ---|---|---
//! `RefCell` | n/a | n/a
//! `QCell` | `QCellOwner` | `QCellOwner::new()`
//! `TCell` | `ACellOwner`<br/>(or `BCellOwner` or `CCellOwner` etc) | `struct MarkerA;`<br/>`type ACell<T> = TCell<MarkerA, T>;`<br/>`type ACellOwner = TCellOwner<MarkerA>;`<br/>`ACellOwner::new()`
//! `GhostCell` | `Set<'id>` | `Set::new(`\|`set`\|` { ... })`
//!
//! Cell ergonomics:
//!
//! Cell | Cell type | Cell creation
//! ---|---|---
//! `RefCell` | `RefCell<T>` | `RefCell::new(v)`
//! `QCell` | `QCell<T>` | `QCell::new(&owner, v)`
//! `TCell` | `ACell<T>` | `ACell::new(&owner, v)`
//! `GhostCell` | `Cell<'id, T>` | `Cell::new(v)` in a context with 'id
//!
//! Borrowing ergonomics:
//!
//! Cell | Cell immutable borrow | Cell mutable borrow
//! ---|---|---
//! `RefCell` | `cell.borrow()` | `cell.borrow_mut()`
//! `QCell` | `owner.get(&cell)` | `owner.get_mut(&cell)`
//! `TCell` | `owner.get(&cell)` | `owner.get_mut(&cell)`
//! `GhostCell` | `set.get(&cell)` | `set.get_mut(&cell)`
//!
//! # Origin of names
//!
//! "Q" originally referred to quantum entanglement, the idea being
//! that this is a kind of remote ownership.  "T" refers to it being
//! type system based.
//!
//! # Unsafe code patterns blocked
//!
//! See the [`doctest_qcell`] and [`doctest_tcell`] modules
//!
//! [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
//! [`QCell`]: struct.QCell.html
//! [`QCellOwner`]: struct.QCellOwner.html
//! [`TCell`]: struct.TCell.html
//! [`TCellOwner`]: struct.TCellOwner.html
//! [`GhostCell`]: https://github.com/pythonesque/kravanenn/blob/wip/src/util/ghost_cell.rs
//! [`doctest_qcell`]: doctest_qcell/index.html
//! [`doctest_tcell`]: doctest_tcell/index.html

#[macro_use]
extern crate lazy_static;

use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::Mutex;

type QCellOwnerID = u32;

/// Borrowing-owner of zero or more [`QCell`](struct.QCell.html)
/// instances.
///
/// See [crate documentation](index.html).
pub struct QCellOwner {
    id: QCellOwnerID,
}

// Used to generate a unique QCellOwnerID number for each QCellOwner
// (until it wraps the u32).  The only purpose is as a code
// correctness check, though.
static QCELLOWNER_ID: AtomicUsize = ATOMIC_USIZE_INIT;

impl QCellOwner {
    /// Create an owner that can be used for creating many `QCell`
    /// instances.  It will have a unique(ish) ID associated with it
    /// to detect coding errors at runtime.
    pub fn new() -> Self {
        Self {
            id: QCELLOWNER_ID.fetch_add(1, Ordering::SeqCst) as u32,
        }
    }

    /// Borrow contents of a `QCell` immutably.  Many `QCell`
    /// instances can be borrowed immutably at the same time from the
    /// same owner.  Panics if the `QCell` is not owned by this
    /// `QCellOwner`.
    pub fn get<'a, T>(&'a self, qc: &'a QCell<T>) -> &'a T {
        assert_eq!(qc.owner, self.id, "QCell accessed with incorrect owner");
        unsafe { &*qc.value.get() }
    }

    /// Borrow contents of a `QCell` mutably.  Only one `QCell` at a
    /// time can be borrowed from the owner using this call.  The
    /// returned reference must go out of scope before another can be
    /// borrowed.  Panics if the `QCell` is not owned by this
    /// `QCellOwner`.
    pub fn get_mut<'a, T>(&'a mut self, qc: &'a QCell<T>) -> &'a mut T {
        assert_eq!(qc.owner, self.id, "QCell accessed with incorrect owner");
        unsafe { &mut *qc.value.get() }
    }

    /// Borrow contents of two `QCell` instances mutably.  Panics if
    /// the two `QCell` instances point to the same memory.  Panics if
    /// either `QCell` is not owned by this `QCellOwner`.
    pub fn get_mut2<'a, T, U>(
        &'a mut self,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
    ) -> (&'a mut T, &'a mut U) {
        assert_eq!(qc1.owner, self.id, "QCell accessed with incorrect owner");
        assert_eq!(qc2.owner, self.id, "QCell accessed with incorrect owner");
        assert_ne!(
            qc1 as *const _ as usize, qc2 as *const _ as usize,
            "Illegal to borrow same QCell twice with get_mut2()"
        );
        unsafe { (&mut *qc1.value.get(), &mut *qc2.value.get()) }
    }

    /// Borrow contents of three `QCell` instances mutably.  Panics if
    /// any pair of `QCell` instances point to the same memory.
    /// Panics if any `QCell` is not owned by this `QCellOwner`.
    pub fn get_mut3<'a, T, U, V>(
        &'a mut self,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
        qc3: &'a QCell<V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert_eq!(qc1.owner, self.id, "QCell accessed with incorrect owner");
        assert_eq!(qc2.owner, self.id, "QCell accessed with incorrect owner");
        assert_eq!(qc3.owner, self.id, "QCell accessed with incorrect owner");
        assert!(
            (qc1 as *const _ as usize != qc2 as *const _ as usize)
                && (qc2 as *const _ as usize != qc3 as *const _ as usize)
                && (qc3 as *const _ as usize != qc1 as *const _ as usize),
            "Illegal to borrow same QCell twice with get_mut3()"
        );
        unsafe {
            (
                &mut *qc1.value.get(),
                &mut *qc2.value.get(),
                &mut *qc3.value.get(),
            )
        }
    }
}

/// Cell whose contents is owned (for borrowing purposes) by a
/// [`QCellOwner`].
///
/// To borrow from this cell, use the borrowing calls on
/// [`QCellOwner`] instance that was used to create it.  See [crate
/// documentation](index.html).
///
/// [`QCellOwner`]: struct.QCellOwner.html
pub struct QCell<T> {
    owner: QCellOwnerID,
    value: UnsafeCell<T>,
}

impl<T> QCell<T> {
    /// Create a new `QCell` owned for borrowing purposes by the given
    /// `QCellOwner`
    #[inline]
    pub const fn new(owner: &QCellOwner, value: T) -> QCell<T> {
        QCell {
            value: UnsafeCell::new(value),
            owner: owner.id,
        }
    }
}

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
    /// type `Q`.  This may be used for creating many `TCell`
    /// instances.
    pub fn new() -> Self {
        assert!(
            SINGLETON_CHECK.lock().unwrap().insert(TypeId::of::<Q>()),
            "Illegal to create two TCellOwner instances with the same marker type parameter"
        );
        Self { typ: PhantomData }
    }

    /// Borrow contents of a `TCell` immutably.  Many `TCell`
    /// instances can be borrowed immutably at the same time from the
    /// same owner.
    #[inline]
    pub fn get<'a, T>(&'a self, qc: &'a TCell<Q, T>) -> &'a T {
        unsafe { &*qc.value.get() }
    }

    /// Borrow contents of a `TCell` mutably.  Only one `TCell` at a
    /// time can be borrowed from the owner using this call.  The
    /// returned reference must go out of scope before another can be
    /// borrowed.
    #[inline]
    pub fn get_mut<'a, T>(&'a mut self, qc: &'a TCell<Q, T>) -> &'a mut T {
        unsafe { &mut *qc.value.get() }
    }

    /// Borrow contents of two `TCell` instances mutably.  Panics if
    /// the two `TCell` instances point to the same memory.
    #[inline]
    pub fn get_mut2<'a, T, U>(
        &'a mut self,
        qc1: &'a TCell<Q, T>,
        qc2: &'a TCell<Q, U>,
    ) -> (&'a mut T, &'a mut U) {
        assert!(
            qc1 as *const _ as usize != qc2 as *const _ as usize,
            "Illegal to borrow same TCell twice with get_mut2()"
        );
        unsafe { (&mut *qc1.value.get(), &mut *qc2.value.get()) }
    }

    /// Borrow contents of three `TCell` instances mutably.  Panics if
    /// any pair of `TCell` instances point to the same memory.
    #[inline]
    pub fn get_mut3<'a, T, U, V>(
        &'a mut self,
        qc1: &'a TCell<Q, T>,
        qc2: &'a TCell<Q, U>,
        qc3: &'a TCell<Q, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert!(
            (qc1 as *const _ as usize != qc2 as *const _ as usize)
                && (qc2 as *const _ as usize != qc3 as *const _ as usize)
                && (qc3 as *const _ as usize != qc1 as *const _ as usize),
            "Illegal to borrow same TCell twice with get_mut3()"
        );
        unsafe {
            (
                &mut *qc1.value.get(),
                &mut *qc2.value.get(),
                &mut *qc3.value.get(),
            )
        }
    }
}

/// Cell whose contents is owned (for borrowing purposes) by a
/// [`TCellOwner`].
///
/// To borrow from this cell, use the borrowing calls on the
/// [`TCellOwner`] instance that was used to create it.  See [crate
/// documentation](index.html).
///
/// [`TCellOwner`]: struct.TCellOwner.html
pub struct TCell<Q, T> {
    owner: PhantomData<Q>,
    value: UnsafeCell<T>,
}

impl<Q, T> TCell<Q, T> {
    /// Create a new `TCell` owned for borrowing purposes by the given
    /// `TCellOwner<Q>`
    #[inline]
    pub const fn new(_owner: &TCellOwner<Q>, value: T) -> TCell<Q, T> {
        TCell {
            owner: PhantomData,
            value: UnsafeCell::new(value),
        }
    }
}

pub mod doctest_qcell;
pub mod doctest_tcell;

#[cfg(test)]
mod tests {
    use super::{QCell, QCellOwner, TCell, TCellOwner};
    #[test]
    fn qcell() {
        let mut owner = QCellOwner::new();
        let c1 = QCell::new(&owner, 100u32);
        let c2 = QCell::new(&owner, 200u32);
        // Fails at compile-time
        //let c1mutref = owner.get_mut(&c1);
        //let c2mutref = owner.get_mut(&c2);
        //*c1mutref += 1;
        //*c2mutref += 2;
        (*owner.get_mut(&c1)) += 1;
        (*owner.get_mut(&c2)) += 2;
        let c1ref = owner.get(&c1);
        let c2ref = owner.get(&c2);
        let total = *c1ref + *c2ref;
        assert_eq!(total, 303);
    }

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
        let c1 = ACell::new(&owner, 100u32);
        let c2 = ACell::new(&owner, 200u32);
        // Fails at compile-time
        //let c1mutref = owner.get_mut(&c1);
        //let c2mutref = owner.get_mut(&c2);
        //*c1mutref += 1;
        //*c2mutref += 2;

        (*owner.get_mut(&c1)) += 1;
        (*owner.get_mut(&c2)) += 2;
        let c1ref = owner.get(&c1);
        let c2ref = owner.get(&c2);
        // Fails at compile-time
        //let c1mutref = owner.get_mut(&c1);
        let total = *c1ref + *c2ref;
        assert_eq!(total, 303);
    }
}
