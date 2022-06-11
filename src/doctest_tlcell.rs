// Run ./update-compiletest-from-doctest.pl in crate base directory
// after making any modification to compile_fail tests here.

//! This tests the `TLCell` implementation.
//!
//! It's not possible to have two simultaneous owners for the same
//! marker type:
//!
//! ```should_panic
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//! struct Marker;
//! type ACellOwner = TLCellOwner<Marker>;
//! let mut owner1 = ACellOwner::new();
//! let mut owner2 = ACellOwner::new();  // Panics here
//! ```
//!
//! It should be impossible to copy a `TLCellOwner`:
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//! type ACell<T> = TLCell<Marker, T>;
//! type ACellOwner = TLCellOwner<Marker>;
//! let mut owner1 = ACellOwner::new();
//! let mut owner2 = owner1;
//! let rc = Rc::new(owner1.cell(100u32));  // Compile fail
//! ```
//!
//! It should be impossible to clone a `TLCellOwner`:
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//! let mut owner1 = ACellOwner::new();
//! let owner2 = owner1.clone();  // Compile fail
//! ```
//!
//! Two different owners can't borrow each other's cells immutably:
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//! struct MarkerA;
//! type ACellOwner = TLCellOwner<MarkerA>;
//! type ACell<T> = TLCell<MarkerA, T>;
//! struct MarkerB;
//! type BCellOwner = TLCellOwner<MarkerB>;
//! type BCell<T> = TLCell<MarkerB, T>;
//!
//! let mut owner_a = ACellOwner::new();
//! let mut owner_b = BCellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//!
//! let c1ref = owner_b.ro(&*c1);   // Compile error
//! println!("{}", *c1ref);
//! ```
//!
//! Or mutably:
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct MarkerA;
//!# type ACellOwner = TLCellOwner<MarkerA>;
//!# type ACell<T> = TLCell<MarkerA, T>;
//!# struct MarkerB;
//!# type BCellOwner = TLCellOwner<MarkerB>;
//!# type BCell<T> = TLCell<MarkerB, T>;
//! let mut owner_a = ACellOwner::new();
//! let mut owner_b = BCellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//!
//! let c1mutref = owner_b.rw(&*c1);    // Compile error
//! println!("{}", *c1mutref);
//! ```
//!
//! You can't have two separate mutable borrows active on the same
//! owner at the same time:
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell<T> = TLCell<Marker, T>;
//! let mut owner = ACellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//! let c2 = Rc::new(ACell::new(200u32));
//!
//! let c1mutref = owner.rw(&c1);
//! let c2mutref = owner.rw(&c2);  // Compile error
//! *c1mutref += 1;
//! *c2mutref += 2;
//! ```
//!
//! However with `rw2()` you can do two mutable borrows at the
//! same time, since this call checks at runtime that the two
//! references don't refer to the same memory:
//!
//! ```
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell<T> = TLCell<Marker, T>;
//!# let mut owner = ACellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//! let c2 = Rc::new(ACell::new(200u32));
//!
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
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell<T> = TLCell<Marker, T>;
//!# let mut owner = ACellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//! let c2 = Rc::new(ACell::new(200u32));
//!
//! let c1ref = owner.ro(&c1);
//! let c1mutref = owner.rw(&c1);    // Compile error
//! println!("{}", *c1ref);
//! ```
//!
//! Not even if it's borrowing a different object:
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell<T> = TLCell<Marker, T>;
//!# let mut owner = ACellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//! let c2 = Rc::new(ACell::new(200u32));
//!
//! let c1mutref = owner.rw(&c1);
//! let c2ref = owner.ro(&c2);    // Compile error
//! *c1mutref += 1;
//! ```
//!
//! Many immutable borrows at the same time is fine:
//!
//! ```
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell<T> = TLCell<Marker, T>;
//!# let mut owner = ACellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//! let c2 = Rc::new(ACell::new(200u32));
//!
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
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell<T> = TLCell<Marker, T>;
//!# let mut owner = ACellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//! let c2 = Rc::new(ACell::new(200u32));
//!
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
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell<T> = TLCell<Marker, T>;
//!# let mut owner = ACellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//! let c2 = Rc::new(ACell::new(200u32));
//!
//! fn test(o: &mut ACellOwner) {}
//!
//! let c1ref = owner.ro(&c1);
//! test(&mut owner);    // Compile error
//! println!("{}", *c1ref);
//! ```
//!
//! Or `&` when there's a `&mut` reference:
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell<T> = TLCell<Marker, T>;
//!# let mut owner = ACellOwner::new();
//! let c1 = Rc::new(ACell::new(100u32));
//! let c2 = Rc::new(ACell::new(200u32));
//!
//! fn test(o: &ACellOwner) {}
//!
//! let c1mutref = owner.rw(&c1);
//! test(&owner);    // Compile error
//! *c1mutref += 1;
//! ```
//!
//! `TLCellOwner` should be neither `Send` nor `Sync`, because it must
//! not escape the thread in which it was created:
//!
//! ```compile_fail
//!# use qcell::TLCellOwner;
//! struct Marker;
//! fn is_send<T: Send>() {}
//! is_send::<TLCellOwner<Marker>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::TLCellOwner;
//! struct Marker;
//! fn is_sync<T: Sync>() {}
//! is_sync::<TLCellOwner<Marker>>();  // Compile fail
//! ```
//!
//! `TLCell` should be `Send` by default, but never `Sync`:
//!
//! ```
//!# use qcell::TLCell;
//! struct Marker;
//! fn is_send<T: Send>() {}
//! is_send::<TLCell<Marker, ()>>();
//! ```
//!
//! ```compile_fail
//!# use qcell::TLCell;
//! struct Marker;
//! fn is_sync<T: Sync>() {}
//! is_sync::<TLCell<Marker, ()>>(); // Compile fail
//! ```
//!
//! A practical example is sending a `TLCell` to another thread for
//! modification and receiving it back again:
//!
//! ```
//!# use qcell::{TLCellOwner, TLCell};
//!# struct Marker;
//! type ACellOwner = TLCellOwner<Marker>;
//! type ACell = TLCell<Marker, i32>;
//!
//! let mut owner = ACellOwner::new();
//! let cell = ACell::new(100);
//!
//! *owner.rw(&cell) += 1;
//! let cell = std::thread::spawn(move || {
//!     let mut owner = ACellOwner::new();  // A different owner
//!     *owner.rw(&cell) += 2;
//!     cell
//! }).join().unwrap();
//! *owner.rw(&cell) += 4;
//! assert_eq!(*owner.ro(&cell), 107);
//! ```
//!
//! However you can't send a cell that's still borrowed:
//!
//! ```compile_fail
//!# use qcell::{TLCellOwner, TLCell};
//!# struct Marker;
//!# type ACellOwner = TLCellOwner<Marker>;
//!# type ACell = TLCell<Marker, i32>;
//! let owner = ACellOwner::new();
//! let cell = ACell::new(100);
//! let val_ref = owner.ro(&cell);
//! std::thread::spawn(move || {
//!     assert_eq!(*owner.ro(&cell), 100);
//! }).join();
//! assert_eq!(*val_ref, 100);
//! ```
//!
//! If the contained type isn't `Send`, the `TLCell` shouldn't be
//! `Send` either:
//!
//! ```compile_fail
//!# use qcell::TLCell;
//!# use std::rc::Rc;
//!# struct Marker;
//! fn is_send<T: Send>() {}
//! is_send::<TLCell<Marker, Rc<()>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//! type ACellOwner = TLCellOwner<Marker>;
//! type ACell = TLCell<Marker, Rc<i32>>;
//!
//! let owner = ACellOwner::new();
//! let cell = ACell::new(Rc::new(100));
//!
//! // We aren't permitted to move the Rc to another thread
//! std::thread::spawn(move || {    // Compile fail
//!     assert_eq!(100, **owner.ro(&cell));
//! }).join();
//! ```
//!
//! Covariant subtypes can't be used to cheat the owner singleton
//! check.  (This code incorrectly succeeds before qcell version
//! 0.4.3.)
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//! type MarkerA = fn(&());
//! type MarkerB = fn(&'static ());
//!
//! let mut owner1 = TLCellOwner::<MarkerA>::new() as TLCellOwner<MarkerB>;  // Compile fail
//! let mut owner2 = TLCellOwner::<MarkerB>::new();
//! let cell = TLCell::<MarkerB, u32>::new(1234);
//! let ref1 = owner1.rw(&cell);
//! let ref2 = owner2.rw(&cell);
//! *ref1 = 1;  // Two mutable refs at the same time!  Unsound!
//! *ref2 = 2;
//! ```
//!
//! A reference obtained using `get_mut` should exclude any other kind
//! of borrowing.
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# struct Marker;
//!# type ACell<T> = TLCell<Marker, T>;
//!# type ACellOwner = TLCellOwner<Marker>;
//! let owner = ACellOwner::new();
//! let mut cell = ACell::new(100);
//! let cell_ref = cell.get_mut();
//! assert_eq!(100, *owner.ro(&cell)); // Compile fail
//! *cell_ref = 50;
//! ```
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# struct Marker;
//!# type ACell<T> = TLCell<Marker, T>;
//!# type ACellOwner = TLCellOwner<Marker>;
//! let mut owner = ACellOwner::new();
//! let mut cell = ACell::new(100);
//! let cell_ref = cell.get_mut();
//! assert_eq!(100, *owner.rw(&cell)); // Compile fail
//! *cell_ref = 50;
//! ```
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# struct Marker;
//!# type ACell<T> = TLCell<Marker, T>;
//!# type ACellOwner = TLCellOwner<Marker>;
//! let owner = ACellOwner::new();
//! let mut cell = ACell::new(100);
//! let cell_ref = owner.ro(&cell);
//! *cell.get_mut() = 50; // Compile fail
//! assert_eq!(100, *cell_ref);
//! ```
//!
//! ```compile_fail
//!# use qcell::{TLCell, TLCellOwner};
//!# struct Marker;
//!# type ACell<T> = TLCell<Marker, T>;
//!# type ACellOwner = TLCellOwner<Marker>;
//! let mut owner = ACellOwner::new();
//! let mut cell = ACell::new(100);
//! let cell_ref = owner.rw(&cell);
//! *cell.get_mut() = 50; // Compile fail
//! assert_eq!(100, *cell_ref);
//! ```
