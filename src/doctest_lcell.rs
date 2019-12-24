// Run ./update-compiletest-from-doctest.pl in crate base directory
// after making any modification to compile_fail tests here.

//! This tests the `LCell` implementation.
//!
//! It should be impossible to copy a `&mut LCellOwner`:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner1| {
//!     let owner2 = owner1;
//!     let rc = Rc::new(owner1.cell(100u32)); // Compile fail
//! });
//! ```
//!
//! It should be impossible to clone an LCellOwner:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner1| {
//!     let owner2 = owner1.clone(); // Compile fail
//! });
//! ```
//!
//! Two different owners can't borrow each other's cells immutably:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner1| {
//!     LCellOwner::scope(|mut owner2| {
//!         let c1 = Rc::new(LCell::new(100u32));
//!         let c1ref1 = owner1.ro(&c1);
//!         let c1ref2 = owner2.ro(&c1);   // Compile error
//!         println!("{}", *c1ref2);
//!     });
//! });
//! ```
//!
//! Or mutably:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner1| {
//!     LCellOwner::scope(|mut owner2| {
//!         let c1 = Rc::new(owner1.cell(100u32));
//!         let c1mutref2 = owner2.rw(&c1);    // Compile error
//!         println!("{}", *c1mutref2);
//!     });
//! });
//! ```
//!
//! You can't have two separate mutable borrows active on the same
//! owner at the same time:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     let c2 = Rc::new(LCell::new(200u32));
//!
//!     let c1mutref = owner.rw(&c1);
//!     let c2mutref = owner.rw(&c2);  // Compile error
//!     *c1mutref += 1;
//!     *c2mutref += 2;
//! });
//! ```
//!
//! However with `rw2()` you can do two mutable borrows at the
//! same time, since this call checks at runtime that the two
//! references don't refer to the same memory:
//!
//! ```
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     let c2 = Rc::new(LCell::new(200u32));
//!     let (c1mutref, c2mutref) = owner.rw2(&c1, &c2);
//!     *c1mutref += 1;
//!     *c2mutref += 2;
//!     assert_eq!(303, owner.ro(&c1) + owner.ro(&c2));   // Success!
//! });
//! ```
//!
//! You can't have a mutable borrow at the same time as an immutable
//! borrow:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     let c2 = Rc::new(LCell::new(200u32));
//!     let c1ref = owner.ro(&c1);
//!     let c1mutref = owner.rw(&c1);    // Compile error
//!     println!("{}", *c1ref);
//! });
//! ```
//!
//! Not even if it's borrowing a different object:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     let c2 = Rc::new(LCell::new(200u32));
//!     let c1mutref = owner.rw(&c1);
//!     let c2ref = owner.ro(&c2);    // Compile error
//!     *c1mutref += 1;
//! });
//! ```
//!
//! Many immutable borrows at the same time is fine:
//!
//! ```
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     let c2 = Rc::new(LCell::new(200u32));
//!     let c1ref = owner.ro(&c1);
//!     let c2ref = owner.ro(&c2);
//!     let c1ref2 = owner.ro(&c1);
//!     let c2ref2 = owner.ro(&c2);
//!     assert_eq!(600, *c1ref + *c2ref + *c1ref2 + *c2ref2);   // Success!
//! });
//! ```
//!
//! Whilst a reference is active, it's impossible to drop the `Rc`:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     let c1ref = owner.ro(&c1);
//!     drop(c1);    // Compile error
//!     println!("{}", *c1ref);
//! });
//! ```
//!
//! Also, whilst a reference is active, it's impossible to call
//! anything else that uses the `owner` in an incompatible way,
//! e.g. `&mut` when there's a `&` reference:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     fn test(o: &mut LCellOwner) {}
//!
//!     let c1ref = owner.ro(&c1);
//!     test(&mut owner);    // Compile error
//!     println!("{}", *c1ref);
//! });
//! ```
//!
//! Or `&` when there's a `&mut` reference:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     fn test(o: &LCellOwner) {}
//!
//!     let c1mutref = owner.rw(&c1);
//!     test(&owner);    // Compile error
//!     *c1mutref += 1;
//! });
//! ```
//!
//! Two examples of passing owners and cells in function arguments.
//! This needs lifetime annotations.
//!
//! ```
//! use qcell::{LCell, LCellOwner};
//! use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(100u32));
//!     fn test<'id>(o: &mut LCellOwner<'id>, rc: &Rc<LCell<'id, u32>>) {
//!        *o.rw(rc) += 1;
//!     }
//!
//!     test(&mut owner, &c1);
//!     let c1mutref = owner.rw(&c1);
//!     *c1mutref += 1;
//! });
//! ```
//!
//! ```
//! use qcell::{LCell, LCellOwner};
//! use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     struct Context<'id> { owner: LCellOwner<'id>, }
//!     let c1 = Rc::new(LCell::new(100u32));
//!     let mut ct = Context { owner };
//!     fn test<'id>(ct: &mut Context<'id>, rc: &Rc<LCell<'id, u32>>) {
//!        *ct.owner.rw(rc) += 1;
//!     }
//!
//!     test(&mut ct, &c1);
//!     let c1mutref = ct.owner.rw(&c1);
//!     *c1mutref += 2;
//! });
//! ```
//!
//! `LCellOwner` and `LCell` should be both `Send` and `Sync` by default:
//!
//! ```
//!# use qcell::{LCellOwner, LCell};
//! fn is_send_sync<T: Send + Sync>() {}
//! is_send_sync::<LCellOwner<'_>>();
//! is_send_sync::<LCell<'_, ()>>();
//! ```
//!
//! So for example we can share a cell ref between threads (Sync), and
//! pass an owner back and forth (Send):
//!
//! ```
//!# use qcell::{LCellOwner, LCell};
//!
//! LCellOwner::scope(|mut owner| {
//!     let cell = LCell::new(100);
//!
//!     *owner.rw(&cell) += 1;
//!     let cell_ref = &cell;
//!     let mut owner = crossbeam::scope(move |s| {
//!         s.spawn(move |_| {
//!             *owner.rw(cell_ref) += 2;
//!             owner
//!         }).join().unwrap()
//!     }).unwrap();
//!     *owner.rw(&cell) += 4;
//!     assert_eq!(*owner.ro(&cell), 107);
//! });
//! ```
//!
//! However you can't send a cell that's still borrowed:
//!
//! ```compile_fail
//!# use qcell::{LCellOwner, LCell};
//! LCellOwner::scope(|mut owner| {
//!     let cell = LCell::new(100);
//!     let val_ref = owner.ro(&cell);
//!     crossbeam::scope(move |s| {
//!         s.spawn(move |_| assert_eq!(*owner.ro(&cell), 100)).join().unwrap(); // Compile fail
//!     }).unwrap();
//!     assert_eq!(*val_ref, 100);
//! });
//! ```
//!
//! If the contained type isn't `Sync`, though, then `LCell` shouldn't
//! be `Sync` either:
//!
//! ```compile_fail
//!# use qcell::LCell;
//!# use std::cell::Cell;
//! fn is_sync<T: Sync>() {}
//! is_sync::<LCell<'_, Cell<i32>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::cell::Cell;
//! LCellOwner::scope(|owner| {
//!     let cell = LCell::new(Cell::new(100));
//!
//!     // This would likely be a data race if it compiled
//!     crossbeam::scope(|s| {   // Compile fail
//!         let handle = s.spawn(|_| owner.ro(&cell).set(200));
//!         owner.ro(&cell).set(300);
//!         handle.join().unwrap();
//!     }).unwrap();
//! });
//! ```
//!
//! If the contained type isn't `Send`, the `LCell` should be neither
//! `Sync` nor `Send`:
//!
//! ```compile_fail
//!# use qcell::LCell;
//!# use std::rc::Rc;
//!# struct Marker;
//! fn is_sync<T: Sync>() {}
//! is_sync::<LCell<'_, Rc<()>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::LCell;
//!# use std::rc::Rc;
//!# struct Marker;
//! fn is_send<T: Send>() {}
//! is_send::<LCell<'_, Rc<()>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|owner| {
//!     let cell = LCell::new(Rc::new(100));
//!
//!     // We aren't permitted to move the Rc to another thread
//!     crossbeam::scope(move |s| {
//!         s.spawn(move |_| assert_eq!(100, **owner.ro(&cell))).join().unwrap(); // Compile fail
//!     }).unwrap();
//! });
//! ```
