use static_assertions::assert_impl_all;
use static_assertions::assert_not_impl_any;

use std::cell::Cell;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use std::rc::Rc;

use crate::{LCell, LCellOwner, QCell, QCellOwner, TCell, TCellOwner, TLCell, TLCellOwner};

// Doesn't do anything, but shows up in list to prove that this file
// has compiled
#[test]
fn test_static_assertions() {}

#[allow(dead_code)]
struct Q;

// Check owners
assert_impl_all!(QCellOwner: Send, Sync, Unpin, UnwindSafe, RefUnwindSafe);
assert_impl_all!(TCellOwner<Q>: Send, Sync, Unpin, UnwindSafe, RefUnwindSafe);
assert_impl_all!(LCellOwner<'_>: Send, Sync, Unpin, UnwindSafe, RefUnwindSafe);
assert_impl_all!(TLCellOwner<Q>: Unpin, UnwindSafe, RefUnwindSafe);
assert_not_impl_any!(TLCellOwner<Q>: Send, Sync);

// Check cells for simple type: i32
assert_impl_all!(QCell<i32>: Send, Sync, Unpin, UnwindSafe);
assert_impl_all!(TCell<Q, i32>: Send, Sync, Unpin, UnwindSafe);
assert_impl_all!(LCell<'_, i32>: Send, Sync, Unpin, UnwindSafe);
assert_impl_all!(TLCell<Q, i32>: Send, Unpin, UnwindSafe);
assert_not_impl_any!(TLCell<Q, i32>: Sync);

// Check cells for a !Send !Sync type: Rc<i32>
assert_impl_all!(QCell<Rc<i32>>: Unpin, UnwindSafe);
assert_impl_all!(TCell<Q, Rc<i32>>: Unpin, UnwindSafe);
assert_impl_all!(LCell<'_, Rc<i32>>: Unpin, UnwindSafe);
assert_impl_all!(TLCell<Q, Rc<i32>>: Unpin, UnwindSafe);
assert_not_impl_any!(QCell<Rc<i32>>: Send, Sync);
assert_not_impl_any!(TCell<Q, Rc<i32>>: Send, Sync);
assert_not_impl_any!(LCell<'_, Rc<i32>>: Send, Sync);
assert_not_impl_any!(TLCell<Q, Rc<i32>>: Send, Sync);

// Check cells for a Send !Sync type: Cell<i32>
assert_impl_all!(QCell<Cell<i32>>: Send, Unpin, UnwindSafe);
assert_impl_all!(TCell<Q, Cell<i32>>: Send, Unpin, UnwindSafe);
assert_impl_all!(LCell<'_, Cell<i32>>: Send, Unpin, UnwindSafe);
assert_impl_all!(TLCell<Q, Cell<i32>>: Send, Unpin, UnwindSafe);
assert_not_impl_any!(QCell<Cell<i32>>: Sync);
assert_not_impl_any!(TCell<Q, Cell<i32>>: Sync);
assert_not_impl_any!(LCell<'_, Cell<i32>>: Sync);
assert_not_impl_any!(TLCell<Q, Cell<i32>>: Sync);

// Check cells for a !Send Sync type
struct Test(*const i32);
unsafe impl Sync for Test {}
assert_impl_all!(QCell<Test>: Unpin, UnwindSafe);
assert_impl_all!(TCell<Q, Test>: Unpin, UnwindSafe);
assert_impl_all!(LCell<'_, Test>: Unpin, UnwindSafe);
assert_impl_all!(TLCell<Q, Test>: Unpin, UnwindSafe);
assert_not_impl_any!(QCell<Test>: Send, Sync);
assert_not_impl_any!(TCell<Q, Test>: Send, Sync);
assert_not_impl_any!(LCell<'_, Test>: Send, Sync);
assert_not_impl_any!(TLCell<Q, Test>: Send, Sync);
