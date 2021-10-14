use core::cell::UnsafeCell;
use core::marker::PhantomPinned;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

// Ensure the alignment is 2 so we can use odd-numbered pointers for
// those created via `fast_new`.
#[repr(align(2))]
#[derive(Clone, Copy)]
struct OwnerIDTarget {
    _data: u16,
}

const MAGIC_OWNER_ID_TARGET: OwnerIDTarget = OwnerIDTarget { _data: 0xCE11 };

type OwnerID = usize;

/// Internal ID associated with a [`QCellOwner`].
///
/// The only purpose of this is to create [`QCell`] instances without
/// requiring a borrow on the [`QCellOwner`].
///
/// Safety: Whilst the existence of this type does mean that an ID can
/// exist longer than than the `QCellOwner`, all that allows is new
/// `QCell` instances to be created after the `QCellOwner` has gone.
/// But `QCell` instances can outlive the owner in any case, so this
/// makes no difference to safety.
///
/// [`QCellOwner`]: struct.QCellOwner.html
/// [`QCell`]: struct.QCell.html
#[derive(Clone, Copy)]
pub struct QCellOwnerID {
    id: OwnerID,
}

impl QCellOwnerID {
    /// Create a new cell owned by this owner-ID.  See also
    /// [`QCell::new`].
    ///
    /// [`QCell::new`]: struct.QCell.html
    pub fn cell<T>(self, value: T) -> QCell<T> {
        QCell {
            value: UnsafeCell::new(value),
            owner: self.id,
        }
    }
}

/// Borrowing-owner of zero or more [`QCell`](struct.QCell.html)
/// instances.
///
/// See [crate documentation](index.html).
pub struct QCellOwner {
    #[cfg(feature = "alloc")]
    _handle: Option<Pin<Box<OwnerIDTarget>>>,
    id: OwnerID,
}

// Used to generate a unique QCellOwnerID number for each QCellOwner
// with the `fast_new()` call.  Start at index 1 and increment by 2 each time
// so the number is always odd - this ensures it will never conflict with
// a real pointer.
static FAST_QCELLOWNER_ID: AtomicUsize = AtomicUsize::new(1);

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Default for QCellOwner {
    fn default() -> Self {
        QCellOwner::new()
    }
}

impl QCellOwner {
    /// Create an owner that can be used for creating many `QCell`
    /// instances.  It will have a temporally unique ID associated
    /// with it to detect using the wrong owner to access a cell at
    /// runtime, which is a programming error.  This is the slow and
    /// safe version that uses memory allocation to ensure unique
    /// IDs.  If speed of this call matters, then consider using
    /// [`fast_new()`](#method.fast_new) instead.
    ///
    /// This safe version does successfully defend against all
    /// malicious and unsafe use, as far as I am aware.  If not,
    /// please raise an issue.  The same unique ID may later be
    /// allocated to someone else once you drop the returned owner,
    /// but this cannot be abused to cause unsafe access to cells
    /// because there will still be only one owner active at any one
    /// time with that ID.  Also it cannot be used maliciously to
    /// access cells which don't belong to the new caller, because you
    /// also need a reference to the cells.  So for example if you
    /// have a graph of cells that is only accessible through a
    /// private structure, then someone else getting the same owner ID
    /// makes no difference, because they have no way to get a
    /// reference to those cells.  In any case, you are probably going
    /// to drop all those cells at the same time as dropping the
    /// owner, because they are no longer of any use without the owner
    /// ID.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn new() -> Self {
        let handle = Box::pin(MAGIC_OWNER_ID_TARGET);
        let raw_ptr: *const OwnerIDTarget = &*handle;
        let id = raw_ptr as usize;
        Self {
            _handle: Some(handle),
            id,
        }
    }

    /// Create an owner that can be used for creating many `QCell`
    /// instances.  It will have a unique(-ish) ID associated with it
    /// to detect using the wrong owner to access a cell at runtime,
    /// which is a programming error.
    ///
    /// # Safety
    ///
    /// This call is much faster than [`new()`](#method.new) because
    /// it uses a simple atomic increment to get a new ID, but it
    /// could be used maliciously to obtain unsafe behaviour, so the
    /// call is marked as `unsafe`.
    ///
    /// If used non-maliciously the chance of getting unsafe behaviour
    /// in practice is zero -- not just close to zero but actually
    /// zero.  To get unsafe behaviour, you'd have to accidentally
    /// create exactly `usize::MAX / 2` more owners to get a duplicate
    /// ID and you'd also have to have a bug in your code where you
    /// try to use the wrong owner to access a cell (which should
    /// normally be rejected with a panic).  Already this is
    /// vanishingly improbable, but then if that happened by accident
    /// on one run but not on another, your code would still panic and
    /// you would fix your bug.  So once that bug in your code is
    /// fixed, the risk is zero.  No amount of fuzz-testing could ever
    /// cause unsafe behaviour once that bug is fixed.  So whilst
    /// strictly-speaking this call is unsafe, in practice there is no
    /// risk unless you really try hard to exploit it.
    pub unsafe fn fast_new() -> Self {
        // Must increment by 2 to ensure we never overlap with a
        // real pointer.
        // Use `Relaxed` ordering because we don't care
        // who gets which ID, just that they are different.
        let id = FAST_QCELLOWNER_ID.fetch_add(2, Ordering::Relaxed);
        Self {
            #[cfg(feature = "alloc")]
            _handle: None,
            id,
        }
    }

    /// Create a new cell owned by this owner instance.  See also
    /// [`QCell::new`].
    ///
    /// [`QCell::new`]: struct.QCell.html
    pub fn cell<T>(&self, value: T) -> QCell<T> {
        QCellOwnerID { id: self.id }.cell(value)
    }

    /// Get the internal owner ID.  This may be used to create `QCell`
    /// instances without needing a borrow on this structure, which is
    /// useful if this structure is already borrowed.
    pub fn id(&self) -> QCellOwnerID {
        QCellOwnerID { id: self.id }
    }

    /// Borrow contents of a `QCell` immutably (read-only).  Many
    /// `QCell` instances can be borrowed immutably at the same time
    /// from the same owner.  Panics if the `QCell` is not owned by
    /// this `QCellOwner`.
    pub fn ro<'a, T: ?Sized>(&'a self, qc: &'a QCell<T>) -> &'a T {
        assert_eq!(qc.owner, self.id, "QCell accessed with incorrect owner");
        unsafe { &*qc.value.get() }
    }

    /// Borrow contents of a `QCell` mutably (read-write).  Only one
    /// `QCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  Panics if the `QCell` is not owned
    /// by this `QCellOwner`.
    pub fn rw<'a, T: ?Sized>(&'a mut self, qc: &'a QCell<T>) -> &'a mut T {
        assert_eq!(qc.owner, self.id, "QCell accessed with incorrect owner");
        unsafe { &mut *qc.value.get() }
    }

    /// Borrow contents of two `QCell` instances mutably.  Panics if
    /// the two `QCell` instances point to the same memory.  Panics if
    /// either `QCell` is not owned by this `QCellOwner`.
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
    ) -> (&'a mut T, &'a mut U) {
        assert_eq!(qc1.owner, self.id, "QCell accessed with incorrect owner");
        assert_eq!(qc2.owner, self.id, "QCell accessed with incorrect owner");
        assert_ne!(
            qc1 as *const _ as *const () as usize, qc2 as *const _ as *const () as usize,
            "Illegal to borrow same QCell twice with rw2()"
        );
        unsafe { (&mut *qc1.value.get(), &mut *qc2.value.get()) }
    }

    /// Borrow contents of three `QCell` instances mutably.  Panics if
    /// any pair of `QCell` instances point to the same memory.
    /// Panics if any `QCell` is not owned by this `QCellOwner`.
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
        qc3: &'a QCell<V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert_eq!(qc1.owner, self.id, "QCell accessed with incorrect owner");
        assert_eq!(qc2.owner, self.id, "QCell accessed with incorrect owner");
        assert_eq!(qc3.owner, self.id, "QCell accessed with incorrect owner");
        assert!(
            (qc1 as *const _ as *const () as usize != qc2 as *const _ as *const () as usize)
                && (qc2 as *const _ as *const () as usize != qc3 as *const _ as *const () as usize)
                && (qc3 as *const _ as *const () as usize != qc1 as *const _ as *const () as usize),
            "Illegal to borrow same QCell twice with rw3()"
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
pub struct QCell<T: ?Sized> {
    owner: OwnerID,
    value: UnsafeCell<T>,
}

// QCell already automatically implements Send, but not Sync.
// We can add this implementation though, since it's fine to
// send a &QCell to another thread, and even mutably borrow the value
// there, as long as T is Send and Sync.
//
// The reason why QCell<T>'s impl of Sync requires T: Send + Sync
// instead of just T: Sync is that QCell provides interior mutability.
// If you send a &QCell<T> (and its owner) to a different thread, you
// can call .rw() to get a &mut T, and use std::mem::swap() to move
// the T, effectively sending the T to that other thread. That's not
// allowed if T: !Send.
//
// Note that the bounds on T for QCell<T>'s impl of Sync are the same
// as those of std::sync::RwLock<T>. That's not a coincidence.
// The way these types let you access T concurrently is the same,
// even though the locking mechanisms are different.
unsafe impl<T: Send + Sync + ?Sized> Sync for QCell<T> {}

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

impl<T: ?Sized> QCell<T> {
    /// Borrow contents of this cell immutably (read-only).  Many
    /// `QCell` instances can be borrowed immutably at the same time
    /// from the same owner.
    #[inline]
    pub fn ro<'a>(&'a self, owner: &'a QCellOwner) -> &'a T {
        owner.ro(self)
    }

    /// Borrow contents of this cell mutably (read-write).  Only one
    /// `QCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  To mutably borrow from two or three
    /// cells at the same time, see [`QCellOwner::rw2`] or
    /// [`QCellOwner::rw3`].
    #[inline]
    pub fn rw<'a>(&'a self, owner: &'a mut QCellOwner) -> &'a mut T {
        owner.rw(self)
    }
}

/// Borrowing-owner of zero or more [`QCell`](struct.QCell.html)
/// instances.
///
/// This type uses it's memory address to provide a unique owner ID.
/// To ensure it's memory address cannot change while cells exist
/// that are owned by it, it requires itself to be pinned before any
/// operation interacting with the ID is attempted.
///
/// The following example uses the `pin_mut` macro from the `pin-utils`
/// crate.  There are many ways to safely pin a value, such as
/// [`Box::pin`](std::boxed::Box::pin),
/// [`pin-utils::pin_mut!`](pin_utils::pin_mut),
/// [`tokio::pin!`](https://docs.rs/tokio/latest/tokio/macro.pin.html),
/// or the
/// [`pin-project`](https://github.com/taiki-e/pin-project) crate.
/// ```
/// use pin_utils::pin_mut;
/// use qcell::{QCell, QCellOwnerPinned};
///# use std::rc::Rc;
///# use std::pin::Pin;
/// let mut owner = QCellOwnerPinned::new();
/// pin_mut!(owner);
/// let item = Rc::new(owner.as_ref().cell(Vec::<u8>::new()));
/// owner.as_mut().rw(&item).push(1);
/// test(owner, &item);
///
/// fn test(owner: Pin<&mut QCellOwnerPinned>, item: &Rc<QCell<Vec<u8>>>) {
///     owner.rw(&item).push(2);
/// }
/// ```
pub struct QCellOwnerPinned {
    target: OwnerIDTarget,
    // ensure this type is !Unpin
    _marker: PhantomPinned,
}

impl Default for QCellOwnerPinned {
    fn default() -> Self {
        QCellOwnerPinned::new()
    }
}

impl QCellOwnerPinned {
    /// Create an owner that can be used for creating many `QCell`
    /// instances.  After this value is pinned it will have a temporally
    /// unique ID associated with it to detect using the wrong owner to
    /// access a cell at runtime, which is a programming error.
    ///
    /// Given the safety contract of Pin, this safe version does
    /// successfully defend against all malicious and unsafe use, as far
    /// as I am aware.  If not, please raise an issue.  The same unique
    /// ID may later be allocated to someone else once you drop the
    /// returned owner, but this cannot be abused to cause unsafe access
    /// to cells because there will still be only one owner active at
    /// any one time with that ID.  Also it cannot be used maliciously
    /// to access cells which don't belong to the new caller, because
    /// you also need a reference to the cells.  So for example if you
    /// have a graph of cells that is only accessible through a
    /// private structure, then someone else getting the same owner ID
    /// makes no difference, because they have no way to get a
    /// reference to those cells.  In any case, you are probably going
    /// to drop all those cells at the same time as dropping the
    /// owner, because they are no longer of any use without the owner
    /// ID.
    pub fn new() -> Self {
        Self {
            target: MAGIC_OWNER_ID_TARGET,
            _marker: PhantomPinned,
        }
    }

    fn raw_id(self: Pin<&Self>) -> OwnerID {
        // Pin guarentees that our address will not change until we are
        // dropped, so we can use it as a unique ID.
        let raw_ptr: *const OwnerIDTarget = &self.target;
        raw_ptr as usize
    }

    /// Get the internal owner ID.  This may be used to create `QCell`
    /// instances without needing a borrow on this structure, which is
    /// useful if this structure is already borrowed.
    ///
    /// Requires this owner to be pinned before use.
    pub fn id(self: Pin<&Self>) -> QCellOwnerID {
        let id = self.raw_id();
        QCellOwnerID { id }
    }

    /// Create a new cell owned by this owner instance.
    ///
    /// Requires this owner to be pinned before use.
    pub fn cell<T>(self: Pin<&Self>, value: T) -> QCell<T> {
        QCellOwnerID { id: self.raw_id() }.cell(value)
    }

    /// Borrow contents of a `QCell` immutably (read-only).  Many
    /// `QCell` instances can be borrowed immutably at the same time
    /// from the same owner.  Panics if the `QCell` is not owned by
    /// this `QCellOwnerPinned`.
    ///
    /// Requires this owner to be pinned before use.
    pub fn ro<'a, T: ?Sized>(self: Pin<&'a Self>, qc: &'a QCell<T>) -> &'a T {
        assert_eq!(
            qc.owner,
            self.raw_id(),
            "QCell accessed with incorrect owner"
        );
        unsafe { &*qc.value.get() }
    }

    /// Borrow contents of a `QCell` mutably (read-write).  Only one
    /// `QCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  Panics if the `QCell` is not owned
    /// by this `QCellOwnerPinned`.
    ///
    /// Requires this owner to be pinned before use.
    #[allow(clippy::mut_from_ref)]
    pub fn rw<'a, T: ?Sized>(self: Pin<&'a mut Self>, qc: &'a QCell<T>) -> &'a mut T {
        assert_eq!(
            qc.owner,
            self.as_ref().id().id,
            "QCell accessed with incorrect owner"
        );
        unsafe { &mut *qc.value.get() }
    }

    /// Borrow contents of two `QCell` instances mutably.  Panics if
    /// the two `QCell` instances point to the same memory.  Panics if
    /// either `QCell` is not owned by this `QCellOwnerPinned`.
    ///
    /// Requires this owner to be pinned before use.
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        self: Pin<&'a mut Self>,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
    ) -> (&'a mut T, &'a mut U) {
        assert_eq!(
            qc1.owner,
            self.as_ref().id().id,
            "QCell accessed with incorrect owner"
        );
        assert_eq!(
            qc2.owner,
            self.as_ref().id().id,
            "QCell accessed with incorrect owner"
        );
        assert_ne!(
            qc1 as *const _ as *const () as usize, qc2 as *const _ as *const () as usize,
            "Illegal to borrow same QCell twice with rw2()"
        );
        unsafe { (&mut *qc1.value.get(), &mut *qc2.value.get()) }
    }

    /// Borrow contents of three `QCell` instances mutably.  Panics if
    /// any pair of `QCell` instances point to the same memory.
    /// Panics if any `QCell` is not owned by this `QCellOwnerPinned`.
    ///
    /// Requires this owner to be pinned before use.
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        self: Pin<&'a mut Self>,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
        qc3: &'a QCell<V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert_eq!(
            qc1.owner,
            self.as_ref().id().id,
            "QCell accessed with incorrect owner"
        );
        assert_eq!(
            qc2.owner,
            self.as_ref().id().id,
            "QCell accessed with incorrect owner"
        );
        assert_eq!(
            qc3.owner,
            self.as_ref().id().id,
            "QCell accessed with incorrect owner"
        );
        assert!(
            (qc1 as *const _ as *const () as usize != qc2 as *const _ as *const () as usize)
                && (qc2 as *const _ as *const () as usize != qc3 as *const _ as *const () as usize)
                && (qc3 as *const _ as *const () as usize != qc1 as *const _ as *const () as usize),
            "Illegal to borrow same QCell twice with rw3()"
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

#[cfg(test)]
mod tests {
    use core::pin::Pin;

    use pin_utils::pin_mut;

    use super::{QCell, QCellOwner, QCellOwnerPinned};

    #[test]
    fn qcell_pinned() {
        let owner = QCellOwnerPinned::new();
        pin_mut!(owner);
        let c1 = owner.as_ref().cell(100u32);
        let c2 = owner.as_ref().cell(200u32);
        (*owner.as_mut().rw(&c1)) += 1;
        (*owner.as_mut().rw(&c2)) += 2;
        let c1ref = owner.as_ref().ro(&c1);
        let c2ref = owner.as_ref().ro(&c2);
        let total = *c1ref + *c2ref;
        assert_eq!(total, 303);
    }

    #[test]
    fn qcell_fast_ids_pinned() {
        let owner1 = QCellOwnerPinned::new();
        pin_mut!(owner1);
        let id1 = owner1.as_ref().raw_id();
        let owner2 = unsafe { QCellOwner::fast_new() };
        let id2 = owner2.id;
        assert_ne!(id1, id2, "Expected ID 1/2 to be different");
        let owner3 = unsafe { QCellOwner::fast_new() };
        let id3 = owner3.id;
        assert_ne!(id2, id3, "Expected ID 2/3 to be different");
        drop(owner2);
        drop(owner3);
        let owner4 = QCellOwnerPinned::new();
        pin_mut!(owner4);
        let id4 = owner4.as_ref().raw_id();
        assert_ne!(id1, id4, "Expected ID 1/4 to be different");
        assert_ne!(id2, id4, "Expected ID 2/4 to be different");
        assert_ne!(id3, id4, "Expected ID 3/4 to be different");
    }

    #[test]
    fn qcell_sep_ids_pinned() {
        let owner1 = QCellOwnerPinned::new();
        let owner2 = QCellOwnerPinned::new();
        pin_mut!(owner1);
        pin_mut!(owner2);
        let id1 = owner1.as_ref().id();
        let id2 = owner2.as_ref().id();
        let c11 = id1.cell(1u32);
        let c12 = id2.cell(2u32);
        let c21 = owner1.as_ref().cell(4u32);
        let c22 = owner2.as_ref().cell(8u32);
        assert_eq!(
            15,
            owner1.as_ref().ro(&c11)
                + owner2.as_ref().ro(&c12)
                + owner1.as_ref().ro(&c21)
                + owner2.as_ref().ro(&c22)
        );
    }

    #[test]
    fn qcell_unsized_pinned() {
        let owner = QCellOwnerPinned::new();
        struct Squares(u32);
        struct Integers(u64);
        trait Series {
            fn step(&mut self);
            fn value(&self) -> u64;
        }
        impl Series for Squares {
            fn step(&mut self) {
                self.0 += 1;
            }
            fn value(&self) -> u64 {
                (self.0 as u64) * (self.0 as u64)
            }
        }
        impl Series for Integers {
            fn step(&mut self) {
                self.0 += 1;
            }
            fn value(&self) -> u64 {
                self.0
            }
        }
        fn series(
            init: u32,
            is_squares: bool,
            owner: Pin<&QCellOwnerPinned>,
        ) -> Box<QCell<dyn Series>> {
            if is_squares {
                Box::new(owner.cell(Squares(init)))
            } else {
                Box::new(owner.cell(Integers(init as u64)))
            }
        }

        pin_mut!(owner);
        let cell1 = series(4, false, owner.as_ref());
        let cell2 = series(7, true, owner.as_ref());
        let cell3 = series(3, true, owner.as_ref());
        assert_eq!(owner.as_ref().ro(&cell1).value(), 4);
        owner.as_mut().rw(&cell1).step();
        assert_eq!(owner.as_ref().ro(&cell1).value(), 5);
        assert_eq!(owner.as_ref().ro(&cell2).value(), 49);
        owner.as_mut().rw(&cell2).step();
        assert_eq!(owner.as_ref().ro(&cell2).value(), 64);
        let (r1, r2, r3) = owner.as_mut().rw3(&cell1, &cell2, &cell3);
        r1.step();
        r2.step();
        r3.step();
        assert_eq!(owner.as_ref().ro(&cell1).value(), 6);
        assert_eq!(owner.as_ref().ro(&cell2).value(), 81);
        assert_eq!(owner.as_ref().ro(&cell3).value(), 16);
        let (r1, r2) = owner.as_mut().rw2(&cell1, &cell2);
        r1.step();
        r2.step();
        assert_eq!(owner.as_ref().ro(&cell1).value(), 7);
        assert_eq!(owner.as_ref().ro(&cell2).value(), 100);
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests_with_alloc {
    use super::{QCell, QCellOwner};

    #[test]
    fn qcell() {
        let mut owner = QCellOwner::new();
        let c1 = QCell::new(&owner, 100u32);
        let c2 = QCell::new(&owner, 200u32);
        (*owner.rw(&c1)) += 1;
        (*owner.rw(&c2)) += 2;
        let c1ref = owner.ro(&c1);
        let c2ref = owner.ro(&c2);
        let total = *c1ref + *c2ref;
        assert_eq!(total, 303);
    }

    #[test]
    fn qcell_ids() {
        let owner1 = QCellOwner::new();
        let id1 = owner1.id;
        let owner2 = QCellOwner::new();
        let id2 = owner2.id;
        assert_ne!(id1, id2, "Expected ID 1/2 to be different");
        drop(owner2);
        let owner3 = QCellOwner::new();
        let id3 = owner3.id;
        assert_ne!(id1, id3, "Expected ID 1/3 to be different");
        drop(owner3);
        drop(owner1);
        let owner4 = QCellOwner::new();
        let id4 = owner4.id;
        let owner5 = QCellOwner::new();
        let id5 = owner5.id;
        assert_ne!(id4, id5, "Expected ID 4/5 to be different");
    }

    #[test]
    fn qcell_fast_ids() {
        let owner1 = QCellOwner::new();
        let id1 = owner1.id;
        let owner2 = unsafe { QCellOwner::fast_new() };
        let id2 = owner2.id;
        assert_ne!(id1, id2, "Expected ID 1/2 to be different");
        let owner3 = unsafe { QCellOwner::fast_new() };
        let id3 = owner3.id;
        assert_ne!(id2, id3, "Expected ID 2/3 to be different");
        drop(owner2);
        drop(owner3);
        let owner4 = QCellOwner::new();
        let id4 = owner4.id;
        assert_ne!(id1, id4, "Expected ID 1/4 to be different");
        assert_ne!(id2, id4, "Expected ID 2/4 to be different");
        assert_ne!(id3, id4, "Expected ID 3/4 to be different");
    }

    #[test]
    fn qcell_sep_ids() {
        let owner1 = QCellOwner::new();
        let owner2 = QCellOwner::new();
        let id1 = owner1.id();
        let id2 = owner2.id();
        let c11 = id1.cell(1u32);
        let c12 = id2.cell(2u32);
        let c21 = owner1.cell(4u32);
        let c22 = owner2.cell(8u32);
        assert_eq!(
            15,
            owner1.ro(&c11) + owner2.ro(&c12) + owner1.ro(&c21) + owner2.ro(&c22)
        );
    }

    #[test]
    fn qcell_unsized() {
        let mut owner = QCellOwner::new();
        struct Squares(u32);
        struct Integers(u64);
        trait Series {
            fn step(&mut self);
            fn value(&self) -> u64;
        }
        impl Series for Squares {
            fn step(&mut self) {
                self.0 += 1;
            }
            fn value(&self) -> u64 {
                (self.0 as u64) * (self.0 as u64)
            }
        }
        impl Series for Integers {
            fn step(&mut self) {
                self.0 += 1;
            }
            fn value(&self) -> u64 {
                self.0
            }
        }
        fn series(init: u32, is_squares: bool, owner: &QCellOwner) -> Box<QCell<dyn Series>> {
            if is_squares {
                Box::new(QCell::new(owner, Squares(init)))
            } else {
                Box::new(QCell::new(owner, Integers(init as u64)))
            }
        }

        let own = &mut owner;
        let cell1 = series(4, false, own);
        let cell2 = series(7, true, own);
        let cell3 = series(3, true, own);
        assert_eq!(cell1.ro(own).value(), 4);
        cell1.rw(own).step();
        assert_eq!(cell1.ro(own).value(), 5);
        assert_eq!(own.ro(&cell2).value(), 49);
        own.rw(&cell2).step();
        assert_eq!(own.ro(&cell2).value(), 64);
        let (r1, r2, r3) = own.rw3(&cell1, &cell2, &cell3);
        r1.step();
        r2.step();
        r3.step();
        assert_eq!(cell1.ro(own).value(), 6);
        assert_eq!(cell2.ro(own).value(), 81);
        assert_eq!(cell3.ro(own).value(), 16);
        let (r1, r2) = own.rw2(&cell1, &cell2);
        r1.step();
        r2.step();
        assert_eq!(cell1.ro(own).value(), 7);
        assert_eq!(cell2.ro(own).value(), 100);
    }
}
