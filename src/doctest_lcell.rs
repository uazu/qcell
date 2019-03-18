//! This tests the `LCell` implementation.
//!
//! It should be impossible to copy a `&mut LCellOwner`:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner1| {
//!     let owner2 = owner1;
//!     let rc = Rc::new(LCell::new(&owner1, 100u32)); // Compile fail
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
//!         let c1 = Rc::new(LCell::new(&owner1, 100u32));
//!         let c1ref2 = owner2.get(&c1);   // Compile error
//!         println!("{}", *c1ref2);
//!     });
//! });
//!
//! ```
//!
//! Or mutably:
//!
//! ```compile_fail
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner1| {
//!     LCellOwner::scope(|mut owner2| {
//!         let c1 = Rc::new(LCell::new(&owner1, 100u32));
//!         let c1mutref = owner2.get_mut(&c1);    // Compile error
//!         println!("{}", *c1mutref);
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
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     let c2 = Rc::new(LCell::new(&owner, 200u32));
//!
//!     let c1mutref = owner.get_mut(&c1);
//!     let c2mutref = owner.get_mut(&c2);  // Compile error
//!     *c1mutref += 1;
//!     *c2mutref += 2;
//! });
//! ```
//!
//! However with `get_mut2()` you can do two mutable borrows at the
//! same time, since this call checks at runtime that the two
//! references don't refer to the same memory:
//!
//! ```
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     let c2 = Rc::new(LCell::new(&owner, 200u32));
//!     let (c1mutref, c2mutref) = owner.get_mut2(&c1, &c2);
//!     *c1mutref += 1;
//!     *c2mutref += 2;
//!     assert_eq!(303, owner.get(&c1) + owner.get(&c2));   // Success!
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
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     let c2 = Rc::new(LCell::new(&owner, 200u32));
//!     let c1ref = owner.get(&c1);
//!     let c1mutref = owner.get_mut(&c1);    // Compile error
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
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     let c2 = Rc::new(LCell::new(&owner, 200u32));
//!     let c1mutref = owner.get_mut(&c1);
//!     let c2ref = owner.get(&c2);    // Compile error
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
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     let c2 = Rc::new(LCell::new(&owner, 200u32));
//!     let c1ref = owner.get(&c1);
//!     let c2ref = owner.get(&c2);
//!     let c1ref2 = owner.get(&c1);
//!     let c2ref2 = owner.get(&c2);
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
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     let c1ref = owner.get(&c1);
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
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     fn test(o: &mut LCellOwner) {}
//!
//!     let c1ref = owner.get(&c1);
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
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     fn test(o: &LCellOwner) {}
//!
//!     let c1mutref = owner.get_mut(&c1);
//!     test(&owner);    // Compile error
//!     *c1mutref += 1;
//! });
//! ```
//!
//! Now a few tests that check that borrowing the same item twice or
//! more cause a panic:
//!
//! ```should_panic
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//!# LCellOwner::scope(|mut owner| {
//!#     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     let (mutref1, mutref2) = owner.get_mut2(&c1, &c1);
//!#     *mutref1 += 1;
//!#     *mutref2 += 1;
//!# });
//! ```
//!
//! ```should_panic
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//!# LCellOwner::scope(|mut owner| {
//!#     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!#     let c2 = Rc::new(LCell::new(&owner, 200u32));
//!     let (mutref1, mutref2, mutref3) = owner.get_mut3(&c1, &c1, &c2);
//!#     *mutref1 += 1;
//!#     *mutref2 += 1;
//!#     *mutref3 += 1;
//!# });
//! ```
//!
//! ```should_panic
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//!# LCellOwner::scope(|mut owner| {
//!#     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!#     let c2 = Rc::new(LCell::new(&owner, 200u32));
//!     let (mutref1, mutref2, mutref3) = owner.get_mut3(&c1, &c2, &c1);
//!#     *mutref1 += 1;
//!#     *mutref2 += 1;
//!#     *mutref3 += 1;
//!# });
//! ```
//!
//! ```should_panic
//!# use qcell::{LCell, LCellOwner};
//!# use std::rc::Rc;
//!# LCellOwner::scope(|mut owner| {
//!#     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!#     let c2 = Rc::new(LCell::new(&owner, 200u32));
//!     let (mutref1, mutref2, mutref3) = owner.get_mut3(&c2, &c1, &c1);
//!#     *mutref1 += 1;
//!#     *mutref2 += 1;
//!#     *mutref3 += 1;
//!# });
//! ```
//!
//! Two examples of passing owners and cells in function arguments.
//! This needs lifetime annotations.
//!
//! ```
//! use qcell::{LCell, LCellOwner};
//! use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     fn test<'id>(o: &mut LCellOwner<'id>, rc: &Rc<LCell<'id, u32>>) {
//!        *o.get_mut(rc) += 1;
//!     }
//!
//!     test(&mut owner, &c1);
//!     let c1mutref = owner.get_mut(&c1);
//!     *c1mutref += 1;
//! });
//! ```
//!
//! ```
//! use qcell::{LCell, LCellOwner};
//! use std::rc::Rc;
//! LCellOwner::scope(|mut owner| {
//!     struct Context<'id> { owner: LCellOwner<'id>, }
//!     let c1 = Rc::new(LCell::new(&owner, 100u32));
//!     let mut ct = Context { owner };
//!     fn test<'id>(ct: &mut Context<'id>, rc: &Rc<LCell<'id, u32>>) {
//!        *ct.owner.get_mut(rc) += 1;
//!     }
//!
//!     test(&mut ct, &c1);
//!     let c1mutref = ct.owner.get_mut(&c1);
//!     *c1mutref += 2;
//! });
//! ```
