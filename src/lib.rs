//! Statically-checked alternatives to [`RefCell`].
//!
//! This crate provides four alternatives to [`RefCell`], each of
//! which checks borrows from the cell at compile-time (statically)
//! instead of checking them at runtime as [`RefCell`] does.  The
//! mechasism for checks is the same for all four.  They only differ
//! in how ownership is represented: [`QCell`] uses an integer ID,
//! [`TCell`] and [`TLCell`] use a marker type, and [`LCell`] uses a
//! Rust lifetime.  Each approach has its advantages and
//! disadvantages.
//!
//! Taking [`QCell`] as an example: [`QCell`] is a cell type where the
//! cell contents are logically 'owned' for borrowing purposes by an
//! instance of an owner type, [`QCellOwner`].  So the cell contents
//! can only be accessed by making borrowing calls on that owner.
//! This behaves similarly to borrowing fields from a structure, or
//! borrowing elements from a `Vec`.  However actually the only link
//! between the objects is that a reference to the owner instance was
//! provided when the cell was created.  Effectively the
//! borrowing-owner and dropping-owner are separated.
//!
//! This enables a pattern where the compiler can statically check
//! mutable access to data stored behind `Rc` references (or other
//! reference types) at compile-time.  This pattern works as follows:
//! The owner is kept on the stack and a mutable reference to it is
//! passed down the stack to calls (for example as part of a context
//! structure).  This is fully checked at compile-time by the borrow
//! checker.  Then this static borrow checking is extended to the cell
//! contents (behind `Rc`s) through using borrowing calls on the owner
//! instance to access the cell contents.  This gives a compile-time
//! guarantee that access to the cell contents is safe.
//!
//! The alternative would be to use [`RefCell`], which panics if two
//! mutable references to the same data are attempted.  With
//! [`RefCell`] there are no warnings or errors to detect the problem
//! at compile-time.  On the other hand, using [`QCell`] the error is
//! detected at compile-time, but the restrictions are much stricter
//! than they really need to be.  For example it's not possible to
//! borrow from more than a few different cells at the same time if
//! they are protected by the same owner, which [`RefCell`] would
//! allow (correctly).  However if you are able to work within these
//! restrictions (e.g. by keeping borrows active only for a short
//! time), then the advantage is that there can never be a panic due
//! to erroneous use of borrowing, because everything is checked by
//! the compiler.
//!
//! Apart from [`QCell`] and [`QCellOwner`], this crate also provides
//! [`TCell`] and [`TCellOwner`] which work the same but use a marker
//! type instead of owner IDs, [`TLCell`] and [`TLCellOwner`] which
//! also use a marker type but which are thread-local, and [`LCell`]
//! and [`LCellOwner`] which use lifetimes.  See the ["Comparison of
//! cell types"](#comparison-of-cell-types) below.
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
//! let iref = owner.rw(&item);
//! test(&mut owner, &item);    // Compile error
//! iref.push(1);
//!
//! fn test(owner: &mut QCellOwner, item: &Rc<QCell<Vec<u8>>>) {
//!     owner.rw(&item).push(2);
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
//! let item = Rc::new(ACell::new(Vec::<u8>::new()));
//! let iref = owner.rw(&item);
//! iref.push(1);
//! test(&mut owner, &item);
//!
//! fn test(owner: &mut ACellOwner, item: &Rc<ACell<Vec<u8>>>) {
//!     owner.rw(&item).push(2);
//! }
//! ```
//!
//! And the same thing again using [`LCell`]:
//!
//! ```
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!   let item = Rc::new(LCell::new(Vec::<u8>::new()));
//!   let iref = owner.rw(&item);
//!   iref.push(1);
//!   test(&mut owner, &item);
//! });
//!
//! fn test<'id>(owner: &mut LCellOwner<'id>, item: &Rc<LCell<'id, Vec<u8>>>) {
//!     owner.rw(&item).push(2);
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
//! `&` cell reference, or a `&mut` on the owner to return a `&mut`
//! cell reference.  So this is the same kind of borrow on both sides.
//! The only borrow we allow for the cell is the borrow that Rust
//! allows for the borrow-owner, and while that borrow is active, the
//! borrow-owner and the cell's reference are blocked from further
//! incompatible borrows.  The contents of the cells act as if they
//! were owned by the borrow-owner, just like elements within a `Vec`.
//! So Rust's guarantees are maintained.
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
//! - Pro: Dynamic owner creation, not limited in any way
//! - Pro: No lifetime annotations or type parameters required
//! - Con: Can only borrow up to 3 objects at a time
//! - Con: Runtime owner checks and some cell space overhead
//!
//! [`TCell`] and [`TLCell`] pros and cons:
//!
//! - Pro: Compile-time borrowing checks
//! - Pro: No overhead at runtime for borrowing or ownership checks
//! - Pro: No cell space overhead
//! - Con: Can only borrow up to 3 objects at a time
//! - Con: Uses singletons, either per-process (TCell) or per-thread
//! (TLCell), meaning only one owner is allowed per thread or process
//! per marker type.  Code intended to be nested on the call stack
//! must be parameterised with an external marker type.
//!
//! [`LCell`] pros and cons:
//!
//! - Pro: Compile-time borrowing checks
//! - Pro: No overhead at runtime for borrowing or ownership checks
//! - Pro: No cell space overhead
//! - Pro: No need for singletons, meaning that one use does not limit other nested uses
//! - Con: Can only borrow up to 3 objects at a time
//! - Con: Requires lifetime annotations on calls and structures
//!
//! Cell | Owner ID | Cell overhead | Borrow check | Owner check | Owner creation check
//! ---|---|---|---|---|---
//! `RefCell` | n/a | `usize` | Runtime | n/a | n/a
//! `QCell` | integer | `u32` | Compile-time | Runtime | Runtime
//! `TCell` or `TLCell` | marker type | none | Compile-time | Compile-time | Runtime
//! `LCell` | lifetime | none | Compile-time | Compile-time | Compile-time
//!
//! Owner ergonomics:
//!
//! Cell | Owner type | Owner creation
//! ---|---|---
//! `RefCell` | n/a | n/a
//! `QCell` | `QCellOwner` | `QCellOwner::new()`
//! `TCell` or<br/>`TLCell` | `ACellOwner`<br/>(or `BCellOwner` or `CCellOwner` etc) | `struct MarkerA;`<br/>`type ACell<T> = TCell<MarkerA, T>;`<br/>`type ACellOwner = TCellOwner<MarkerA>;`<br/>`ACellOwner::new()`
//! `LCell` | `LCellOwner<'id>` | `LCellOwner::scope(`\|`owner`\|` { ... })`
//!
//! Cell ergonomics:
//!
//! Cell | Cell type | Cell creation
//! ---|---|---
//! `RefCell` | `RefCell<T>` | `RefCell::new(v)`
//! `QCell` | `QCell<T>` | `owner.cell(v)` or `QCell::new(&owner, v)`
//! `TCell` or `TLCell` | `ACell<T>` | `owner.cell(v)` or `ACell::new(v)`
//! `LCell` | `LCell<'id, T>` | `owner.cell(v)` or `LCell::new(v)`
//!
//! Borrowing ergonomics:
//!
//! Cell | Cell immutable borrow | Cell mutable borrow
//! ---|---|---
//! `RefCell` | `cell.borrow()` | `cell.borrow_mut()`
//! `QCell` | `owner.ro(&cell)` | `owner.rw(&cell)`
//! `TCell` or `TLCell` | `owner.ro(&cell)` | `owner.rw(&cell)`
//! `LCell` | `owner.ro(&cell)` | `owner.rw(&cell)`
//!
//! # Multi-threaded use: Send and Sync
//!
//! Most often the cell-owner will be held by just one thread, and all
//! access to cells will be made within that thread.  However it is
//! still safe to pass or share these objects between threads in some
//! cases, where permitted by the contained type:
//!
//! Cell | Owner type | Cell type
//! ---|---|---
//! `RefCell` | n/a | Send
//! `QCell` | Send + Sync | Send + Sync
//! `TCell` | Send + Sync | Send + Sync
//! `TLCell` |  | Send
//! `LCell` | Send + Sync | Send + Sync
//!
//! I believe that the reasoning behind enabling Send and/or Sync is
//! sound, but I welcome review of this in case anything has been
//! overlooked.  I am grateful for contributions from Github users
//! [**Migi**] and [**pythonesque**].  `GhostCell` by
//! [**pythonesque**] is a lifetime-based cell that predated `LCell`.
//! It appears that he has proved some properties of `GhostCell` using
//! Coq, and I hope that that research becomes public in due course.
//!
//! Here's an overview of the reasoning:
//!
//! - Unlike `RefCell` these cell types may be `Sync` because mutable
//! access is protected by the cell owner.  You can get mutable access
//! to the cell contents only if you have mutable access to the cell
//! owner.  (Note that `Sync` is only available where the contained
//! type is `Send + Sync`.)
//!
//! - The cell owner may be `Sync` because `Sync` only allows shared
//! immutable access to the cell owner across threads.  So there may
//! exist `&QCell` and `&QCellOwner` references in two threads, but
//! only immutable access to the cell contents is possible like that,
//! so there is no soundness issue.
//!
//! - In general `Send` is safe because that is a complete transfer of
//! some right from one thread to another (assuming the contained type
//! is also `Send`).
//!
//! - `TLCell` is the exception because there can be a different owner
//! with the same marker type in each thread, so owners must not be
//! sent or shared.  Also if two threads have `&TLCell` references to
//! the same cell then mutable references to the contained data could
//! be created in both threads which would break Rust's guarantees.
//! So `TLCell` cannot be `Sync`.  However it can be `Send` because in
//! that case the right to access the data is being transferred
//! completely from one thread to another.
//!
//! # Origin of names
//!
//! "Q" originally referred to quantum entanglement, the idea being
//! that this is a kind of remote ownership.  "T" refers to it being
//! type system based, "TL" thread-local, "L" to lifetime-based.
//!
//! # Unsafe code patterns blocked
//!
//! See the [`doctest_qcell`], [`doctest_tcell`], [`doctest_tlcell`]
//! and [`doctest_lcell`] modules
//!
//! [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
//! [`QCell`]: struct.QCell.html
//! [`QCellOwner`]: struct.QCellOwner.html
//! [`TCell`]: struct.TCell.html
//! [`TCellOwner`]: struct.TCellOwner.html
//! [`TLCell`]: struct.TLCell.html
//! [`TLCellOwner`]: struct.TLCellOwner.html
//! [`LCell`]: struct.LCell.html
//! [`LCellOwner`]: struct.LCellOwner.html
//! [`doctest_qcell`]: doctest_qcell/index.html
//! [`doctest_tcell`]: doctest_tcell/index.html
//! [`doctest_tlcell`]: doctest_tlcell/index.html
//! [`doctest_lcell`]: doctest_lcell/index.html
//! [**Migi**]: https://github.com/Migi
//! [**pythonesque**]: https://github.com/pythonesque

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate lazy_static;

#[allow(unused)]
mod lcell;
#[allow(unused)]
mod qcell;
#[allow(unused)]
mod tcell;
#[allow(unused)]
mod tlcell;
mod value_cell;

mod qcell_impl;
mod tcell_impl;
mod tlcell_impl;
mod lcell_impl;

pub mod doctest_lcell;
pub mod doctest_qcell;
pub mod doctest_tcell;
pub mod doctest_tlcell;

pub use crate::value_cell::{ValueCell, ValueCellOwner};
pub use crate::qcell_impl::{RuntimeOwner as QCellOwner, RuntimeProxy, QCell};
pub use crate::tcell_impl::{SingletonOwner as TCellOwner, SingletonProxy, TCell};
pub use crate::tlcell_impl::{ThreadLocalSingletonOwner as TLCellOwner, ThreadLocalSingletonProxy, TLCell};
pub use crate::lcell_impl::{LifetimeOwner as LCellOwner, LifetimeProxy, LCell};

// The compile-tests double-check that the compile_fail tests in the
// doctests actually fail for the reason intended, not for some other
// reason.  This is most useful to check when making changes to the
// crate.  However since the compiler error messages may change from
// one release to the next, the tests only remain valid for a certain
// range of compiler versions.
#[cfg(test)]
pub mod compiletest {
    #[rustversion::all(stable, since(1.39), before(1.40))]
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("src/compiletest/*.rs");
    }
}
