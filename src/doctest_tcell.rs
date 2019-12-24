// Run ./update-compiletest-from-doctest.pl in crate base directory
// after making any modification to compile_fail tests here.

//! This tests the `TCell` implementation.
//!
//! It's not possible to have two simultaneous owners for the same
//! marker type:
//!
//! ```should_panic
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//! struct Marker;
//! type ACellOwner = TCellOwner<Marker>;
//! let mut owner1 = ACellOwner::new();
//! let mut owner2 = ACellOwner::new();  // Panics here
//! ```
//!
//! It should be impossible to copy a `TCellOwner`:
//!
//! ```compile_fail
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//! type ACell<T> = TCell<Marker, T>;
//! type ACellOwner = TCellOwner<Marker>;
//! let mut owner1 = ACellOwner::new();
//! let mut owner2 = owner1;
//! let rc = Rc::new(owner1.cell(100u32));  // Compile fail
//! ```
//!
//! It should be impossible to clone a `TCellOwner`:
//!
//! ```compile_fail
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//! let mut owner1 = ACellOwner::new();
//! let owner2 = owner1.clone();  // Compile fail
//! ```
//!
//! Two different owners can't borrow each other's cells immutably:
//!
//! ```compile_fail
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//! struct MarkerA;
//! type ACellOwner = TCellOwner<MarkerA>;
//! type ACell<T> = TCell<MarkerA, T>;
//! struct MarkerB;
//! type BCellOwner = TCellOwner<MarkerB>;
//! type BCell<T> = TCell<MarkerB, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct MarkerA;
//!# type ACellOwner = TCellOwner<MarkerA>;
//!# type ACell<T> = TCell<MarkerA, T>;
//!# struct MarkerB;
//!# type BCellOwner = TCellOwner<MarkerB>;
//!# type BCell<T> = TCell<MarkerB, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell<T> = TCell<Marker, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell<T> = TCell<Marker, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell<T> = TCell<Marker, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell<T> = TCell<Marker, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell<T> = TCell<Marker, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell<T> = TCell<Marker, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell<T> = TCell<Marker, T>;
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
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell<T> = TCell<Marker, T>;
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
//! `TCellOwner` and `TCell` should be both `Send` and `Sync` by default:
//!
//! ```
//!# use qcell::{TCellOwner, TCell};
//! struct Marker;
//! fn is_send_sync<T: Send + Sync>() {}
//! is_send_sync::<TCellOwner<Marker>>();
//! is_send_sync::<TCell<Marker, ()>>();
//! ```
//!
//! So for example we can share a cell ref between threads (Sync), and
//! pass an owner back and forth (Send):
//!
//! ```
//!# use qcell::{TCellOwner, TCell};
//!# struct Marker;
//! type ACellOwner = TCellOwner<Marker>;
//! type ACell = TCell<Marker, i32>;
//!
//! let mut owner = ACellOwner::new();
//! let cell = ACell::new(100);
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
//!# use qcell::{TCellOwner, TCell};
//!# struct Marker;
//!# type ACellOwner = TCellOwner<Marker>;
//!# type ACell = TCell<Marker, i32>;
//! let owner = ACellOwner::new();
//! let cell = ACell::new(100);
//! let val_ref = owner.ro(&cell);
//! std::thread::spawn(move || {
//!     assert_eq!(*owner.ro(&cell), 100);
//! }).join();
//! assert_eq!(*val_ref, 100);
//! ```
//!
//! If the contained type isn't `Sync`, though, then `TCell` shouldn't
//! be `Sync` either:
//!
//! ```compile_fail
//!# use qcell::TCell;
//!# use std::cell::Cell;
//!# struct Marker;
//! fn is_sync<T: Sync>() {}
//! is_sync::<TCell<Marker, Cell<i32>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::{TCell, TCellOwner};
//!# use std::cell::Cell;
//!# struct Marker;
//! type ACellOwner = TCellOwner<Marker>;
//! type ACell = TCell<Marker, Cell<i32>>;
//!
//! let owner = ACellOwner::new();
//! let cell = ACell::new(Cell::new(100));
//!
//! // This would be a data race if the compiler permitted it, but it doesn't
//! std::thread::spawn(|| owner.ro(&cell).set(200));  // Compile fail
//! owner.ro(&cell).set(300);
//! ```
//!
//! If the contained type isn't `Send`, the `TCell` should be neither
//! `Sync` nor `Send`:
//!
//! ```compile_fail
//!# use qcell::TCell;
//!# use std::rc::Rc;
//!# struct Marker;
//! fn is_sync<T: Sync>() {}
//! is_sync::<TCell<Marker, Rc<()>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::TCell;
//!# use std::rc::Rc;
//!# struct Marker;
//! fn is_send<T: Send>() {}
//! is_send::<TCell<Marker, Rc<()>>>();  // Compile fail
//! ```
//!
//! ```compile_fail
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//!# struct Marker;
//! type ACellOwner = TCellOwner<Marker>;
//! type ACell = TCell<Marker, Rc<i32>>;
//!
//! let owner = ACellOwner::new();
//! let cell = ACell::new(Rc::new(100));
//!
//! // We aren't permitted to move the Rc to another thread
//! std::thread::spawn(move || {    // Compile fail
//!     assert_eq!(100, **owner.ro(&cell));
//! }).join();
//! ```
