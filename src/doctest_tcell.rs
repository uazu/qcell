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
//! It should be impossible to copy a TCellOwner:
//!
//! ```compile_fail
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//! struct Marker;
//! type ACell<T> = TCell<Marker, T>;
//! type ACellOwner = TCellOwner<Marker>;
//! let mut owner1 = ACellOwner::new();
//! let mut owner2 = owner1;
//! let rc = Rc::new(ACell::new(&owner1, 100u32));  // Compile fail
//! ```
//!
//! It should be impossible to clone a TCellOwner:
//!
//! ```compile_fail
//!# use qcell::{TCell, TCellOwner};
//!# use std::rc::Rc;
//! struct Marker;
//! type ACellOwner = TCellOwner<Marker>;
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
//! let c1 = Rc::new(ACell::new(&owner_a, 100u32));
//!
//! let c1ref = owner_b.get(&c1);   // Compile error
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
//!
//! let mut owner_a = ACellOwner::new();
//! let mut owner_b = BCellOwner::new();
//! let c1 = Rc::new(ACell::new(&owner_a, 100u32));
//!
//! let c1mutref = owner_b.get_mut(&c1);    // Compile error
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
//! let c1 = Rc::new(ACell::new(&owner, 100u32));
//! let c2 = Rc::new(ACell::new(&owner, 200u32));
//!
//! let c1mutref = owner.get_mut(&c1);
//! let c2mutref=  owner.get_mut(&c2);  // Compile error
//! *c1mutref += 1;
//! *c2mutref += 2;
//! ```
//!
//! However with `get_mut2()` you can do two mutable borrows at the
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
//!# let c1 = Rc::new(ACell::new(&owner, 100u32));
//!# let c2 = Rc::new(ACell::new(&owner, 200u32));
//! let (c1mutref, c2mutref) = owner.get_mut2(&c1, &c2);
//! *c1mutref += 1;
//! *c2mutref += 2;
//! assert_eq!(303, owner.get(&c1) + owner.get(&c2));   // Success!
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
//!# let c1 = Rc::new(ACell::new(&owner, 100u32));
//!# let c2 = Rc::new(ACell::new(&owner, 200u32));
//! let c1ref = owner.get(&c1);
//! let c1mutref = owner.get_mut(&c1);    // Compile error
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
//!# let c1 = Rc::new(ACell::new(&owner, 100u32));
//!# let c2 = Rc::new(ACell::new(&owner, 200u32));
//! let c1mutref = owner.get_mut(&c1);
//! let c2ref = owner.get(&c2);    // Compile error
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
//!# let c1 = Rc::new(ACell::new(&owner, 100u32));
//!# let c2 = Rc::new(ACell::new(&owner, 200u32));
//! let c1ref = owner.get(&c1);
//! let c2ref = owner.get(&c2);
//! let c1ref2 = owner.get(&c1);
//! let c2ref2 = owner.get(&c2);
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
//!# let c1 = Rc::new(ACell::new(&owner, 100u32));
//!# let c2 = Rc::new(ACell::new(&owner, 200u32));
//! let c1ref = owner.get(&c1);
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
//!# let c1 = Rc::new(ACell::new(&owner, 100u32));
//!# let c2 = Rc::new(ACell::new(&owner, 200u32));
//! fn test(o: &mut ACellOwner) {}
//!
//! let c1ref = owner.get(&c1);
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
//!# let c1 = Rc::new(ACell::new(&owner, 100u32));
//!# let c2 = Rc::new(ACell::new(&owner, 200u32));
//! fn test(o: &ACellOwner) {}
//!
//! let c1mutref = owner.get_mut(&c1);
//! test(&owner);    // Compile error
//! *c1mutref += 1;
//! ```
