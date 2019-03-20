//! Statically-checked alternatives to [`RefCell`].
//!
//! This crate provides three alternatives to [`RefCell`], each of
//! which checks borrows from the cell at compile-time (statically)
//! instead of checking them at runtime as [`RefCell`] does.  The
//! mechasism for checks is the same for all three.  They only differ
//! in how ownership is represented: [`QCell`] uses an integer ID,
//! [`TCell`] uses a marker type, and [`LCell`] uses a Rust lifetime.
//! Each approach has its advantages and disadvantages.
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
//! type instead of owner IDs, and [`LCell`] and [`LCellOwner`] which
//! use lifetimes.  See the ["Comparison of cell
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
//! let item = Rc::new(ACell::new(Vec::<u8>::new()));
//! let iref = owner.get_mut(&item);
//! iref.push(1);
//! test(&mut owner, &item);
//!
//! fn test(owner: &mut ACellOwner, item: &Rc<ACell<Vec<u8>>>) {
//!     owner.get_mut(&item).push(2);
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
//!   let iref = owner.get_mut(&item);
//!   iref.push(1);
//!   test(&mut owner, &item);
//! });
//!
//! fn test<'id>(owner: &mut LCellOwner<'id>, item: &Rc<LCell<'id, Vec<u8>>>) {
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
//! [`TCell`] pros and cons:
//!
//! - Pro: Compile-time borrowing checks
//! - Pro: No overhead at runtime for borrowing or ownership checks
//! - Pro: No cell space overhead
//! - Con: Can only borrow up to 3 objects at a time
//! - Con: Uses singletons, so reusable code must be parameterised
//! with an external marker type
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
//! `TCell` | marker type | none | Compile-time | Compile-time | Runtime
//! `LCell` | lifetime | none | Compile-time | Compile-time | Compile-time
//!
//! Owner ergonomics:
//!
//! Cell | Owner type | Owner creation
//! ---|---|---
//! `RefCell` | n/a | n/a
//! `QCell` | `QCellOwner` | `QCellOwner::new()`
//! `TCell` | `ACellOwner`<br/>(or `BCellOwner` or `CCellOwner` etc) | `struct MarkerA;`<br/>`type ACell<T> = TCell<MarkerA, T>;`<br/>`type ACellOwner = TCellOwner<MarkerA>;`<br/>`ACellOwner::new()`
//! `LCell` | `LCellOwner<'id>` | `LCellOwner::scope(`\|`owner`\|` { ... })`
//!
//! Cell ergonomics:
//!
//! Cell | Cell type | Cell creation
//! ---|---|---
//! `RefCell` | `RefCell<T>` | `RefCell::new(v)`
//! `QCell` | `QCell<T>` | `owner.cell(v)` or `QCell::new(&owner, v)`
//! `TCell` | `ACell<T>` | `owner.cell(v)` or `ACell::new(v)`
//! `LCell` | `LCell<'id, T>` | `owner.cell(v)` or `LCell::new(v)`
//!
//! Borrowing ergonomics:
//!
//! Cell | Cell immutable borrow | Cell mutable borrow
//! ---|---|---
//! `RefCell` | `cell.borrow()` | `cell.borrow_mut()`
//! `QCell` | `owner.get(&cell)` | `owner.get_mut(&cell)`
//! `TCell` | `owner.get(&cell)` | `owner.get_mut(&cell)`
//! `LCell` | `owner.get(&cell)` | `owner.get_mut(&cell)`
//!
//! # Origin of names
//!
//! "Q" originally referred to quantum entanglement, the idea being
//! that this is a kind of remote ownership.  "T" refers to it being
//! type system based, "L" to lifetime-based.
//!
//! # Unsafe code patterns blocked
//!
//! See the [`doctest_qcell`], [`doctest_tcell`] and [`doctest_lcell`] modules
//!
//! [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
//! [`QCell`]: struct.QCell.html
//! [`QCellOwner`]: struct.QCellOwner.html
//! [`TCell`]: struct.TCell.html
//! [`TCellOwner`]: struct.TCellOwner.html
//! [`LCell`]: struct.LCell.html
//! [`LCellOwner`]: struct.LCellOwner.html
//! [`doctest_qcell`]: doctest_qcell/index.html
//! [`doctest_tcell`]: doctest_tcell/index.html
//! [`doctest_lcell`]: doctest_lcell/index.html

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate lazy_static;

mod lcell;
mod qcell;
mod tcell;

pub mod doctest_lcell;
pub mod doctest_qcell;
pub mod doctest_tcell;

pub use crate::lcell::LCell;
pub use crate::lcell::LCellOwner;
pub use crate::qcell::QCell;
pub use crate::qcell::QCellOwner;
pub use crate::qcell::QCellOwnerID;
pub use crate::tcell::TCell;
pub use crate::tcell::TCellOwner;
