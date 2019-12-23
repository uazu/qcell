//! This tests the `QCell` implementation.
//!
//! It should be impossible to copy a QCellOwner:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//! let mut owner1 = QCellOwner::new();
//! let mut owner2 = owner1;
//! let rc = Rc::new(QCell::new(&owner1, 100u32));  // Compile fail
//! ```
//!
//! It should be impossible to clone a QCellOwner:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//! let mut owner1 = QCellOwner::new();
//! let owner2 = owner1.clone();  // Compile fail
//! ```
//!
//! Two different owners can't borrow each other's cells immutably:
//!
//! ```should_panic
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//! let mut owner1 = QCellOwner::new();
//! let mut owner2 = QCellOwner::new();
//! let c1 = Rc::new(QCell::new(&owner1, 100u32));
//!
//! let c1ref = owner2.ro(&c1);   // Panics here
//! println!("{}", *c1ref);
//! ```
//!
//! Or mutably:
//!
//! ```should_panic
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//! let mut owner1 = QCellOwner::new();
//! let mut owner2 = QCellOwner::new();
//! let c1 = Rc::new(QCell::new(&owner1, 100u32));
//!
//! let c1mutref = owner2.rw(&c1);    // Panics here
//! println!("{}", *c1mutref);
//! ```
//!
//! You can't have two separate mutable borrows active on the same
//! owner at the same time:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//! let mut owner = QCellOwner::new();
//! let c1 = Rc::new(QCell::new(&owner, 100u32));
//! let c2 = Rc::new(QCell::new(&owner, 200u32));
//!
//! let c1mutref = owner.rw(&c1);
//! let c2mutref=  owner.rw(&c2);  // Compile error
//! *c1mutref += 1;
//! *c2mutref += 2;
//! ```
//!
//! However with `rw2()` you can do two mutable borrows at the
//! same time, since this call checks at runtime that the two
//! references don't refer to the same memory:
//!
//! ```
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//!# let mut owner = QCellOwner::new();
//!# let c1 = Rc::new(QCell::new(&owner, 100u32));
//!# let c2 = Rc::new(QCell::new(&owner, 200u32));
//! let (c1mutref, c2mutref) = owner.rw2(&c1, &c2);
//! *c1mutref += 1;
//! *c2mutref += 2;
//! assert_eq!(303, owner.ro(&c1) + owner.ro(&c2));   // Success!
//! ```
//!
//! You can't have a mutable borrow at the same time as an immutable
//! borrow:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//!# let mut owner = QCellOwner::new();
//!# let c1 = Rc::new(QCell::new(&owner, 100u32));
//!# let c2 = Rc::new(QCell::new(&owner, 200u32));
//! let c1ref = owner.ro(&c1);
//! let c1mutref = owner.rw(&c1);    // Compile error
//! println!("{}", *c1ref);
//! ```
//!
//! Not even if it's borrowing a different object:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//!# let mut owner = QCellOwner::new();
//!# let c1 = Rc::new(QCell::new(&owner, 100u32));
//!# let c2 = Rc::new(QCell::new(&owner, 200u32));
//! let c1mutref = owner.rw(&c1);
//! let c2ref = owner.ro(&c2);    // Compile error
//! *c1mutref += 1;
//! ```
//!
//! Many immutable borrows at the same time is fine:
//!
//! ```
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//!# let mut owner = QCellOwner::new();
//!# let c1 = Rc::new(QCell::new(&owner, 100u32));
//!# let c2 = Rc::new(QCell::new(&owner, 200u32));
//! let c1ref = owner.ro(&c1);
//! let c2ref = owner.ro(&c2);
//! let c1ref2 = owner.ro(&c1);
//! let c2ref2 = owner.ro(&c2);
//! assert_eq!(600, *c1ref + *c2ref + *c1ref2 + *c2ref2);   // Success!
//! ```
//!
//! Whilst a reference is active, it's impossible to drop the `Rc`:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//!# let mut owner = QCellOwner::new();
//!# let c1 = Rc::new(QCell::new(&owner, 100u32));
//!# let c2 = Rc::new(QCell::new(&owner, 200u32));
//! let c1ref = owner.ro(&c1);
//! drop(c1);    // Compile error
//! println!("{}", *c1ref);
//! ```
//!
//! Also, whilst a reference is active, it's impossible to call
//! anything else that uses the `owner` in an incompatible way,
//! e.g. `&mut` when there's a `&` reference:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//!# let mut owner = QCellOwner::new();
//!# let c1 = Rc::new(QCell::new(&owner, 100u32));
//!# let c2 = Rc::new(QCell::new(&owner, 200u32));
//! fn test(o: &mut QCellOwner) {}
//!
//! let c1ref = owner.ro(&c1);
//! test(&mut owner);    // Compile error
//! println!("{}", *c1ref);
//! ```
//!
//! Or `&` when there's a `&mut` reference:
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//!# let mut owner = QCellOwner::new();
//!# let c1 = Rc::new(QCell::new(&owner, 100u32));
//!# let c2 = Rc::new(QCell::new(&owner, 200u32));
//! fn test(o: &QCellOwner) {}
//!
//! let c1mutref = owner.rw(&c1);
//! test(&owner);    // Compile error
//! *c1mutref += 1;
//! ```
//!
//! `QCellOwner` and `QCell` should be both `Send` and `Sync` by default:
//!
//! ```
//!# use qcell::{QCellOwner, QCell};
//! fn is_send_sync<T: Send + Sync>() {}
//! is_send_sync::<QCellOwner>();
//! is_send_sync::<QCell<()>>();
//! ```
//!
//! So for example we can share a cell ref between threads (Sync), and
//! pass an owner back and forth (Send):
//!
//! ```
//!# use qcell::{QCellOwner, QCell};
//! let mut owner = QCellOwner::new();
//! let cell = QCell::new(&owner, 100_i32);
//!
//! *owner.rw(&cell) += 1;
//! let cell_ref = &cell;
//! let mut owner = crossbeam::scope(move |s| {
//!     s.spawn(move |_| {
//!         *owner.rw(cell_ref) += 2;
//!         owner
//!     }).join().unwrap()
//! }).unwrap();
//! *owner.rw(&cell) += 4;
//! assert_eq!(*owner.ro(&cell), 107);
//! ```
//!
//! However you can't send a cell that's still borrowed:
//!
//! ```compile_fail
//!# use qcell::{QCellOwner, QCell};
//! let owner = QCellOwner::new();
//! let cell = QCell::new(&owner, 100);
//! let val_ref = owner.ro(&cell);
//! std::thread::spawn(move || {
//!     assert_eq!(*owner.ro(&cell), 100);
//! }).join();
//! assert_eq!(*val_ref, 100);
//! ```
//!
//! If the contained type isn't `Sync`, though, then `QCell` shouldn't
//! be `Sync` either:
//!
//! ```compile_fail
//!# use qcell::QCell;
//!# use std::cell::Cell;
//! fn is_sync<T: Sync>() {}
//! is_sync::<QCell<Cell<i32>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::cell::Cell;
//! let owner = QCellOwner::new();
//! let cell = QCell::new(&owner, Cell::new(100));
//!
//! // This would be a data race if the compiler permitted it, but it doesn't
//! std::thread::spawn(|| owner.ro(&cell).set(200));  // Compile fail
//! owner.ro(&cell).set(300);
//! ```
//!
//! If the contained type isn't `Send`, the `QCell` should be neither
//! `Sync` nor `Send`:
//!
//! ```compile_fail
//!# use qcell::QCell;
//!# use std::rc::Rc;
//! fn is_sync<T: Sync>() {}
//! is_sync::<QCell<Rc<()>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::QCell;
//!# use std::rc::Rc;
//! fn is_send<T: Send>() {}
//! is_send::<QCell<Rc<()>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::{QCell, QCellOwner};
//!# use std::rc::Rc;
//! let owner = QCellOwner::new();
//! let cell = QCell::new(&owner, Rc::new(100));
//!
//! // We aren't permitted to move the Rc to another thread
//! std::thread::spawn(move || {    // Compile fail
//!     assert_eq!(100, **owner.ro(&cell));
//! }).join();
//! ```
