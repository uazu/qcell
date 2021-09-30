use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

type OwnerID = u32;

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
    // Reserve first half of range for safe version, second half for
    // unsafe version
    id: OwnerID,
}

// Used to generate a unique QCellOwnerID number for each QCellOwner
// with the `fast_new()` call.
static FAST_QCELLOWNER_ID: AtomicUsize = AtomicUsize::new(0);
const FAST_FIRST_ID: OwnerID = 0x8000_0000;

// Used to allocate temporally unique QCellOwnerID numbers for each
// QCellOwner created with the slower `new()` call.  Expected pattern
// of allocation is to have just a few owners active at any one time
// (let's say 1-4 for each component using QCell), but then perhaps
// very many components created and destroyed over the lifetime of the
// process, possibly way more than 2^32.  So a free list suits this
// pattern.
static SAFE_QCELLOWNER_FREE_LIST: crossbeam_queue::SegQueue<OwnerID> =
    crossbeam_queue::SegQueue::new();
static SAFE_QCELLOWNER_NEXT: AtomicUsize = AtomicUsize::new(0);

impl Drop for QCellOwner {
    fn drop(&mut self) {
        // Re-use safe IDs
        if self.id < FAST_FIRST_ID {
            SAFE_QCELLOWNER_FREE_LIST.push(self.id);
        }
    }
}

impl Default for QCellOwner {
    fn default() -> Self {
        QCellOwner::new()
    }
}

impl QCellOwner {
    /// Create an owner that can be used for creating many `QCell`
    /// instances.  It will have a temporally unique ID associated
    /// with it to detect using the wrong owner to access a cell at
    /// runtime, which is a programming error.  This call will panic
    /// if the limit of 2^31 owners active at the same time is
    /// reached.  This is the slow and safe version that uses a mutex
    /// and a free list to allocate IDs.  If speed of this call
    /// matters, then consider using [`fast_new()`](#method.fast_new)
    /// instead.
    ///
    /// This safe version does successfully defend against all
    /// malicious and unsafe use, as far as I am aware.  If not,
    /// please raise an issue.  The same unique ID will later be
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
    pub fn new() -> Self {
        if let Some(id) = SAFE_QCELLOWNER_FREE_LIST.pop() {
            return Self { id };
        }
        match SAFE_QCELLOWNER_NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            use std::convert::TryFrom;
            let max_id = usize::try_from(FAST_FIRST_ID).unwrap_or(usize::MAX);
            if next < max_id {
                Some(next + 1)
            } else {
                None
            }
        }) {
            Ok(id) => Self { id: id as u32 },
            Err(_) => {
                panic!("More than 2^31 QCellOwner instances are active at the same time")
            }
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
    /// create exactly 2^31 more owners to get a duplicate ID and
    /// you'd also have to have a bug in your code where you try to
    /// use the wrong owner to access a cell (which should normally be
    /// rejected with a panic).  Already this is vanishingly
    /// improbable, but then if that happened by accident on one run
    /// but not on another, your code would still panic and you would
    /// fix your bug.  So once that bug in your code is fixed, the
    /// risk is zero.  No amount of fuzz-testing could ever cause
    /// unsafe behaviour once that bug is fixed.  So whilst
    /// strictly-speaking this call is unsafe, in practice there is no
    /// risk unless you really try hard to exploit it.
    pub unsafe fn fast_new() -> Self {
        Self {
            // Range 0x80000000 to 0xFFFFFFFF reserved for fast
            // version.  Use `Relaxed` ordering because we don't care
            // who gets which ID, just that they are different.
            id: FAST_QCELLOWNER_ID.fetch_add(1, Ordering::Relaxed) as u32 | FAST_FIRST_ID,
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

#[cfg(test)]
mod tests {
    use super::{QCell, QCellOwner};
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    // Really we need the QCellOwner tests to always run with
    // --test-threads=1 because they all access the same pool of IDs,
    // but there's no way to specify that in Cargo.toml.  So use a
    // lock instead.
    static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn qcell() {
        let _lock = LOCK.lock().unwrap();
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
        let _lock = LOCK.lock().unwrap();
        let owner1 = QCellOwner::new();
        let id1 = owner1.id;
        let owner2 = QCellOwner::new();
        let id2 = owner2.id;
        assert_ne!(id1, id2, "Expected ID 1/2 to be different");
        drop(owner2);
        let owner3 = QCellOwner::new();
        let id3 = owner3.id;
        assert_eq!(id3, id2, "Expected ID 2 to be reused");
        assert_ne!(id1, id3, "Expected ID 1/3 to be different");
        drop(owner3);
        drop(owner1);
        let owner4 = QCellOwner::new();
        let id4 = owner4.id;
        let owner5 = QCellOwner::new();
        let id5 = owner5.id;
        assert!((id1 == id4 || id1 == id5), "Expected ID 1 to be reused");
        assert!((id3 == id4 || id3 == id5), "Expected ID 3 to be reused");
        assert_ne!(id4, id5, "Expected ID 4/5 to be different");
    }

    #[test]
    fn qcell_fast_ids() {
        let _lock = LOCK.lock().unwrap();
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
        let _lock = LOCK.lock().unwrap();
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
        let _lock = LOCK.lock().unwrap();
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
