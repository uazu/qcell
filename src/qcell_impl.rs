use core::sync::atomic::{AtomicU64, Ordering::Relaxed};
use crate::{ValueCell, ValueCellOwner};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub type QCell<T> = ValueCell<QCellOwner, T>;
type QCellIDValue = [u8; 6];
const MAX_ID_VALUE: u64 = 1 << (std::mem::size_of::<QCellIDValue>() * 8);

/// Borrowing-owner of zero or more [`QCell`](struct.QCell.html)
/// instances.
///
/// See [crate documentation](index.html).
pub struct QCellOwner {
    id: QCellIDValue,
}

pub struct QCellOwnerID(QCellIDValue);

impl<T> QCell<T> {
    #[inline]
    pub fn new(owner: &QCellOwner, value: T) -> Self {
        owner.cell(value)
    }
}

impl QCellOwnerID {
    /// Create a new cell owned by this owner-ID.  See also
    /// [`QCell::new`].
    ///
    /// [`QCell::new`]: struct.QCell.html
    pub fn cell<T>(&self, value: T) -> ValueCell<QCellOwner, T> {
        QCell::from_marker(QCellOwnerID(self.0), value)
    }
}

impl Default for QCellOwner {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl QCellOwner {
    /// Create an owner that can be used for creating many `QCell`
    /// instances.  It will have a unique ID associated with it to 
    /// detect using the wrong owner to access a cell at runtime, 
    /// which is a programming error.  This call will panic if the 
    /// limit of 2^48 owners ever created in the same process.
    #[inline]
    #[allow(clippy::many_single_char_names)]
    pub fn new() -> Self {
        let mut id = NEXT_ID.load(Relaxed);

        loop {
            core::sync::atomic::spin_loop_hint();

            let next_id = id + 1;
            
            assert!(next_id < MAX_ID_VALUE, "Tried to create too many `QCellOwner`s");
            
            match NEXT_ID.compare_exchange_weak(id, next_id, Relaxed, Relaxed) {
                Ok(_) => break,
                Err(next_id) => id = next_id
            }
        }

        let [a, b, c, d, e, f, _, _]: [u8; 8] = id.to_le_bytes();

        Self { id: [a, b, c, d, e, f] }
    }

    /// Get the internal owner ID.  This may be used to create `QCell`
    /// instances without needing a borrow on this structure, which is
    /// useful if this structure is already borrowed.
    pub fn id(&self) -> QCellOwnerID {
        QCellOwnerID(self.id)
    }
    
    /// Create a new cell owned by this owner instance.  See also
    /// [`QCell::new`].
    ///
    /// [`QCell::new`]: struct.QCell.html
    pub fn cell<T>(&self, value: T) -> ValueCell<Self, T> {
        ValueCellOwner::cell(self, value)
    }

    /// Borrow contents of a `QCell` immutably (read-only).  Many
    /// `QCell` instances can be borrowed immutably at the same time
    /// from the same owner.  Panics if the `QCell` is not owned by
    /// this `QCellOwner`.
    #[inline]
    pub fn ro<'a, T: ?Sized>(&'a self, cell: &'a ValueCell<Self, T>) -> &'a T {
        ValueCellOwner::ro(self, cell)
    }

    /// Borrow contents of a `QCell` mutably (read-write).  Only one
    /// `QCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  Panics if the `QCell` is not owned
    /// by this `QCellOwner`.
    #[inline]
    pub fn rw<'a, T: ?Sized>(&'a mut self, cell: &'a ValueCell<Self, T>) -> &'a mut T {
        ValueCellOwner::rw(self, cell)
    }

    /// Borrow contents of two `QCell` instances mutably.  Panics if
    /// the two `QCell` instances point to the same memory.  Panics if
    /// either `QCell` is not owned by this `QCellOwner`.
    #[inline]
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        c1: &'a ValueCell<Self, T>,
        c2: &'a ValueCell<Self, U>,
    ) -> (&'a mut T, &'a mut U) {
        ValueCellOwner::rw2(self, c1 ,c2)
    }

    /// Borrow contents of three `QCell` instances mutably.  Panics if
    /// any pair of `QCell` instances point to the same memory.
    /// Panics if any `QCell` is not owned by this `QCellOwner`.
    #[inline]
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        c1: &'a ValueCell<Self, T>,
        c2: &'a ValueCell<Self, U>,
        c3: &'a ValueCell<Self, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        ValueCellOwner::rw3(self, c1 ,c2, c3)
    }
}

unsafe impl ValueCellOwner for QCellOwner {
    type Marker = QCellOwnerID;

    // #[inline]
    fn validate_marker(&self, &QCellOwnerID(id): &Self::Marker) -> bool {
        self.id == id
    }

    #[inline]
    fn make_marker(&self) -> Self::Marker {
        QCellOwnerID(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::{QCell, QCellOwner};
    use std::sync::Mutex;
    lazy_static! {
        // Really we need the QCellOwner tests to always run with --test-threads=1 because
        // they all access the same pool of IDs, but there's no way to specify that in
        // Cargo.toml.  So use a lock instead.
        static ref LOCK: Mutex<()> = Mutex::new(());
    }
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
        let total   = *c1ref + *c2ref;
        assert_eq!(total, 303);
    }

    #[test]
    #[ignore] // TODO: ID reclamation
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
        assert_eq!(id4, id1, "Expected ID 1 to be reused");
        assert_eq!(id5, id3, "Expected ID 3 to be reused");
        assert_ne!(id4, id5, "Expected ID 4/5 to be different");
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
}
