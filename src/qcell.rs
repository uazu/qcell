use core::cell::UnsafeCell;
use core::marker::PhantomPinned;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

// Ensure the alignment is 2 so we can use odd-numbered IDs for those
// created via `QCellOwnerSeq`.
#[repr(align(2))]
#[derive(Clone, Copy)]
struct OwnerIDTarget {
    _data: u16,
}

const MAGIC_OWNER_ID_TARGET: OwnerIDTarget = OwnerIDTarget { _data: 0xCE11 };

#[cold]
#[inline(never)]
fn bad_owner_panic() -> ! {
    panic!("QCell accessed with incorrect owner");
}

macro_rules! owner_check {
    ($owner:expr $(, $qcell:expr)+) => {
        $(
            if $qcell.owner.0 != $owner.id().0 {
                bad_owner_panic();
            }
        )+
    }
}

#[cold]
#[inline(never)]
fn not_distinct_panic() -> ! {
    panic!("Illegal to borrow same QCell twice with rw2() or rw3()");
}

macro_rules! distinct_check {
    ($qc1:expr, $qc2:expr) => {{
        let qc1 = $qc1 as *const _ as *const () as usize;
        let qc2 = $qc2 as *const _ as *const () as usize;
        if qc1 == qc2 {
            not_distinct_panic();
        }
    }};
    ($qc1:expr, $qc2:expr, $qc3:expr) => {{
        let qc1 = $qc1 as *const _ as *const () as usize;
        let qc2 = $qc2 as *const _ as *const () as usize;
        let qc3 = $qc3 as *const _ as *const () as usize;
        if qc1 == qc2 || qc2 == qc3 || qc3 == qc1 {
            not_distinct_panic();
        }
    }};
}

/// Internal ID associated with a [`QCell`] owner.
///
/// The only purpose of this is to create [`QCell`] instances without
/// requiring a borrow on the owner.  A [`QCellOwnerID`] can be passed
/// to any code that needs to create [`QCell`] instances.  However to
/// access those [`QCell`] instances after creation will still require
/// a borrow on the original owner.  Create a [`QCellOwnerID`] from an
/// owner using `owner.into()` or `owner.id()`.
///
/// # Safety
///
/// Whilst the existence of this type does mean that an ID can exist
/// longer than than the owner, all that allows is new [`QCell`]
/// instances to be created after the owner has gone.  But [`QCell`]
/// instances can outlive the owner in any case, so this makes no
/// difference to safety.
#[derive(Clone, Copy)]
pub struct QCellOwnerID(usize);

impl QCellOwnerID {
    /// Create a new cell owned by this owner-ID.  See also
    /// [`QCell::new`].
    ///
    /// [`QCell::new`]: struct.QCell.html
    pub fn cell<T>(self, value: T) -> QCell<T> {
        QCell {
            value: UnsafeCell::new(value),
            owner: self,
        }
    }
}

#[cfg(feature = "alloc")]
impl From<&QCellOwner> for QCellOwnerID {
    fn from(owner: &QCellOwner) -> Self {
        owner.id()
    }
}

impl From<&QCellOwnerSeq> for QCellOwnerID {
    fn from(owner: &QCellOwnerSeq) -> Self {
        owner.id()
    }
}

impl From<Pin<&QCellOwnerPinned>> for QCellOwnerID {
    fn from(owner: Pin<&QCellOwnerPinned>) -> Self {
        owner.id()
    }
}

/// Cell whose contents is owned (for borrowing purposes) by a
/// [`QCellOwner`], a [`QCellOwnerSeq`] or a [`QCellOwnerPinned`].
///
/// To borrow from this cell, use the borrowing calls on the owner
/// instance that was used to create it.  For [`QCellOwner`], there
/// are also convenience methods [`QCell::ro`] and [`QCell::rw`].  See
/// also [crate documentation](index.html).
///
/// [`QCellOwner`]: struct.QCellOwner.html
/// [`QCell::ro`]: struct.QCell.html#method.ro
/// [`QCell::rw`]: struct.QCell.html#method.rw
pub struct QCell<T: ?Sized> {
    owner: QCellOwnerID,
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
    /// Create a new [`QCell`] owned for borrowing purposes by the
    /// owner with the given [`QCellOwnerID`], or a type that can be
    /// converted into a [`QCellOwnerID`], such as `&owner`.  So calls
    /// will typically take the form `QCell::new(&owner, value)` or
    /// `QCell::new(owner_id, value)`.
    #[inline]
    pub fn new(id: impl Into<QCellOwnerID>, value: T) -> QCell<T> {
        QCell {
            value: UnsafeCell::new(value),
            owner: id.into(),
        }
    }

    /// Destroy the cell and return the contained value
    ///
    /// Safety: Since this consumes the cell, there can be no other
    /// references to the cell or the data at this point.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

#[cfg(feature = "alloc")]
impl<T: ?Sized> QCell<T> {
    /// Convenience method to borrow a cell immutably when the owner
    /// is a [`QCellOwner`].  Equivalent to [`QCellOwner::ro`].  See
    /// [`QCellOwnerSeq::ro`] or [`QCellOwnerPinned::ro`] to borrow
    /// for other owner types.
    #[inline]
    pub fn ro<'a>(&'a self, owner: &'a QCellOwner) -> &'a T {
        owner.ro(self)
    }

    /// Convenience method to borrow a cell mutably when the owner is
    /// a [`QCellOwner`].  Equivalent to [`QCellOwner::rw`].  See
    /// [`QCellOwnerSeq::rw`] or [`QCellOwnerPinned::rw`] to borrow
    /// for other owner types.
    #[inline]
    pub fn rw<'a>(&'a self, owner: &'a mut QCellOwner) -> &'a mut T {
        owner.rw(self)
    }

    /// Returns a mutable reference to the underlying data
    ///
    /// Note that this is only useful at the beginning-of-life or
    /// end-of-life of the cell when you have exclusive access to it.
    /// Normally you'd use [`QCell::rw`] or [`QCellOwner::rw`] to get
    /// a mutable reference to the contents of the cell.
    ///
    /// Safety: This call borrows `QCell` mutably which guarantees
    /// that we possess the only reference.  This means that there can
    /// be no active borrows of other forms, even ones obtained using
    /// an immutable reference.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

/// Borrowing-owner of zero or more [`QCell`] instances.
///
/// The owner will have a temporally unique ID associated with it to
/// detect use of the wrong owner to access a cell at runtime, which
/// is a programming error.  Temporally unique means that at any one
/// time, only one owner will hold that ID.  This type derives the
/// owner ID from the address of an internal memory allocation which
/// this owner holds until it is dropped, which ensures that the ID is
/// temporally unique.  The allocation is aligned to ensure that its
/// ID cannot collide with those created using [`QCellOwnerSeq`].
///
/// In a `no_std` environment this requires the `alloc` feature
/// because it allocates memory.  For a `no_std` environment without
/// `alloc`, consider using [`QCellOwnerSeq`] or [`QCellOwnerPinned`].
///
/// # Safety
///
/// This should successfully defend against all malicious and unsafe
/// use.  If not, please raise an issue.  The same unique ID may later
/// be allocated to someone else once you drop the returned owner, but
/// this cannot be abused to cause unsafe access to cells because
/// there will still be only one owner active at any one time with
/// that ID.  Also it cannot be used maliciously to access cells which
/// don't belong to the new caller, because you also need a reference
/// to the cells.  So for example if you have a graph of cells that is
/// only accessible through a private structure, then if someone else
/// gets the same owner ID later, it makes no difference, because they
/// have no way to get a reference to those cells.  In any case, you
/// are probably going to drop all those cells at the same time as
/// dropping the owner, because they are no longer of any use without
/// the owner ID.
///
/// See [crate documentation](index.html).
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct QCellOwner {
    // `Box` should be enough to ensure that the address is unique and
    // stable, but add `Pin` as a safeguard against any future
    // optimisation of `Box`.
    handle: Pin<Box<OwnerIDTarget>>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Default for QCellOwner {
    fn default() -> Self {
        QCellOwner::new()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl QCellOwner {
    /// Create an owner that can be used for creating many [`QCell`]
    /// instances.
    #[inline]
    pub fn new() -> Self {
        let handle = Box::pin(MAGIC_OWNER_ID_TARGET);
        Self { handle }
    }

    /// Get the internal owner ID.  This may be used to create [`QCell`]
    /// instances without needing a borrow on this structure, which is
    /// useful if this structure is already borrowed.
    #[inline]
    pub fn id(&self) -> QCellOwnerID {
        let raw_ptr: *const OwnerIDTarget = &*self.handle;
        QCellOwnerID(raw_ptr as usize)
    }

    /// Create a new cell owned by this owner instance.  See also
    /// [`QCell::new`].
    #[inline]
    pub fn cell<T>(&self, value: T) -> QCell<T> {
        let id: QCellOwnerID = self.into();
        id.cell(value)
    }

    /// Borrow contents of a [`QCell`] immutably (read-only).  Many
    /// [`QCell`] instances can be borrowed immutably at the same time
    /// from the same owner.  Panics if the [`QCell`] is not owned by
    /// this [`QCellOwner`].
    #[inline]
    pub fn ro<'a, T: ?Sized>(&'a self, qc: &'a QCell<T>) -> &'a T {
        owner_check!(self, qc);
        unsafe { &*qc.value.get() }
    }

    /// Borrow contents of a [`QCell`] mutably (read-write).  Only one
    /// [`QCell`] at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  Panics if the [`QCell`] is not owned
    /// by this [`QCellOwner`].
    #[inline]
    pub fn rw<'a, T: ?Sized>(&'a mut self, qc: &'a QCell<T>) -> &'a mut T {
        owner_check!(self, qc);
        unsafe { &mut *qc.value.get() }
    }

    /// Borrow contents of two [`QCell`] instances mutably.  Panics if
    /// the two [`QCell`] instances point to the same memory.  Panics
    /// if either [`QCell`] is not owned by this [`QCellOwner`].
    #[inline]
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
    ) -> (&'a mut T, &'a mut U) {
        owner_check!(self, qc1, qc2);
        distinct_check!(qc1, qc2);
        unsafe { (&mut *qc1.value.get(), &mut *qc2.value.get()) }
    }

    /// Borrow contents of three [`QCell`] instances mutably.  Panics
    /// if any pair of [`QCell`] instances point to the same memory.
    /// Panics if any [`QCell`] is not owned by this [`QCellOwner`].
    #[inline]
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
        qc3: &'a QCell<V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        owner_check!(self, qc1, qc2, qc3);
        distinct_check!(qc1, qc2, qc3);
        unsafe {
            (
                &mut *qc1.value.get(),
                &mut *qc2.value.get(),
                &mut *qc3.value.get(),
            )
        }
    }
}

// Used to generate a unique QCellOwnerID number for each
// QCellOwnerSeq.  Start at index 1 and increment by 2 each time so
// the number is always odd to ensure it will never conflict with the
// address of a OwnerIDTarget.
static FAST_QCELLOWNER_ID: AtomicUsize = AtomicUsize::new(1);

/// Borrowing-owner of zero or more [`QCell`] instances, using an ID
/// sequence.
///
/// The owner will have a unique(-ish) ID associated with it to detect
/// use of the wrong owner to access a cell at runtime, which is a
/// programming error.  This type allocates the owner ID from a
/// wrapping sequence sourced from a global atomic variable, so it is
/// very fast to allocate.
///
/// # Safety
///
/// A malicious coder could cause an intentional ID collision, e.g. by
/// creating 2^63 owners on a 64-bit build (or 2^31 on 32-bit, etc),
/// which would cause the ID to wrap.  This would allow that coder to
/// cause undefined behaviour in their own code.  So at a stretch this
/// could allow a coder to hide unsound code from a safety review.
/// Because of that the [`QCellOwnerSeq::new`] method is marked as
/// `unsafe`.  However it is not possible to use it unsafely by
/// accident, only through making an intentional, determined and
/// CPU-intensive effort to exploit it.
///
/// See [crate documentation](index.html).
pub struct QCellOwnerSeq {
    id: QCellOwnerID,
}

// Default implementation not possible, due to `unsafe`

impl QCellOwnerSeq {
    /// Create an owner that can be used for creating many [`QCell`]
    /// instances.
    ///
    /// # Safety
    ///
    /// The contract with the caller is that the caller must not
    /// intentionally create an owner-ID collision and exploit it to
    /// create undefined behaviour.  The caller could do this by
    /// creating 2^63 more owners on a 64-bit build (or 2^31 on
    /// 32-bit, etc), causing the ID to wrap, and then using two
    /// owners that they know to have the same ID to access the same
    /// memory mutably from two references at the same time.  This is
    /// totally impossible to do by accident, so any normal use of
    /// this call will be 100% safe.
    ///
    /// To get unsound behaviour requires both an owner ID collision
    /// (which might just about happen by accident in very unusual
    /// situations), and then also intentionally using the wrong owner
    /// to access a cell.  Usually using the wrong owner to access a
    /// cell would cause an immediate panic because it is a
    /// programming error.  It is extremely unlikely that there would
    /// always be the same ID collision in testing, so this panic
    /// would soon be fixed.  Once it is fixed, there is absolutely no
    /// way that even an accidental collision could cause any unsound
    /// behaviour, because the bug has been eliminated, and the
    /// correct owner is always used to access each cell.
    #[inline]
    pub unsafe fn new() -> Self {
        // Must increment by 2 to ensure we never collide with an ID
        // derived from the address of an `OwnerIDTarget`.  Use
        // `Relaxed` ordering because we don't care who gets which ID,
        // just that they are different.
        Self {
            id: QCellOwnerID(FAST_QCELLOWNER_ID.fetch_add(2, Ordering::Relaxed)),
        }
    }

    /// Get the internal owner ID.  This may be used to create
    /// [`QCell`] instances without needing a borrow on this
    /// structure, which is useful if this structure is already
    /// borrowed.
    #[inline]
    pub fn id(&self) -> QCellOwnerID {
        self.id
    }

    /// Create a new cell owned by this owner instance.  See also
    /// [`QCell::new`].
    ///
    /// [`QCell::new`]: struct.QCell.html
    #[inline]
    pub fn cell<T>(&self, value: T) -> QCell<T> {
        self.id.cell(value)
    }

    /// Borrow contents of a [`QCell`] immutably (read-only).  Many
    /// [`QCell`] instances can be borrowed immutably at the same time
    /// from the same owner.  Panics if the [`QCell`] is not owned by
    /// this [`QCellOwnerSeq`].
    #[inline]
    pub fn ro<'a, T: ?Sized>(&'a self, qc: &'a QCell<T>) -> &'a T {
        owner_check!(self, qc);
        unsafe { &*qc.value.get() }
    }

    /// Borrow contents of a [`QCell`] mutably (read-write).  Only one
    /// [`QCell`] at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  Panics if the [`QCell`] is not owned
    /// by this [`QCellOwnerSeq`].
    #[inline]
    pub fn rw<'a, T: ?Sized>(&'a mut self, qc: &'a QCell<T>) -> &'a mut T {
        owner_check!(self, qc);
        unsafe { &mut *qc.value.get() }
    }

    /// Borrow contents of two [`QCell`] instances mutably.  Panics if
    /// the two [`QCell`] instances point to the same memory.  Panics
    /// if either [`QCell`] is not owned by this [`QCellOwnerSeq`].
    #[inline]
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
    ) -> (&'a mut T, &'a mut U) {
        owner_check!(self, qc1, qc2);
        distinct_check!(qc1, qc2);
        unsafe { (&mut *qc1.value.get(), &mut *qc2.value.get()) }
    }

    /// Borrow contents of three [`QCell`] instances mutably.  Panics
    /// if any pair of [`QCell`] instances point to the same memory.
    /// Panics if any [`QCell`] is not owned by this
    /// [`QCellOwnerSeq`].
    #[inline]
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
        qc3: &'a QCell<V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        owner_check!(self, qc1, qc2, qc3);
        distinct_check!(qc1, qc2, qc3);
        unsafe {
            (
                &mut *qc1.value.get(),
                &mut *qc2.value.get(),
                &mut *qc3.value.get(),
            )
        }
    }
}

/// Borrowing-owner of zero or more [`QCell`] instances, based on a
/// pinned struct
///
/// This type uses its own memory address to provide a unique owner
/// ID, which requires no allocations and only 2 bytes of storage.  So
/// this is suitable for a `no_std` environment without an allocator.
/// The owner can be created on the stack, or on the heap, as
/// required.  To ensure its memory address cannot change while cells
/// exist that are owned by it, it requires itself to be pinned before
/// any operation interacting with the ID is attempted.
///
/// There are many ways to safely pin a value, such as
/// [`Box::pin`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.pin),
/// [`pin-utils::pin_mut!`](https://docs.rs/pin-utils/latest/pin_utils/macro.pin_mut.html),
/// [`tokio::pin!`](https://docs.rs/tokio/latest/tokio/macro.pin.html),
/// or the [`pin-project`](https://github.com/taiki-e/pin-project)
/// crate.
///
/// The following example uses the `pin_mut!` macro from the
/// `pin-utils` crate:
///
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
///
/// This example incorporates the [`QCellOwnerPinned`] into a larger
/// structure kept on the stack, and accesses it using the
/// [`pin-project`](https://github.com/taiki-e/pin-project) crate:
///
/// ```
/// use crate::qcell::{QCell, QCellOwnerPinned};
/// use pin_project::pin_project;
/// use pin_utils::pin_mut;
///# use std::pin::Pin;
///# use std::rc::Rc;
///
/// #[pin_project]
/// struct MyStruct {
///     _misc: usize,  // Unpinned value
///     #[pin]
///     owner: QCellOwnerPinned,
/// }
///
/// let mystruct = MyStruct {
///     _misc: 0,
///     owner: QCellOwnerPinned::new(),
/// };
///
/// pin_mut!(mystruct);
///
/// let item = Rc::new(
///     mystruct.as_mut().project().owner.as_ref().cell(Vec::<u8>::new())
/// );
/// mystruct.as_mut().project().owner.rw(&item).push(1);
/// test(mystruct.as_mut().project().owner, &item);
///
/// fn test(owner: Pin<&mut QCellOwnerPinned>, item: &Rc<QCell<Vec<u8>>>) {
///     owner.rw(&item).push(2);
/// }
/// ```
///
/// # Safety
///
/// After the owner is pinned, its address is used as a temporally
/// unique ID.  This detects use of the wrong owner to access a cell
/// at runtime, which is a programming error.
///
/// Note that even without `Pin`, this would still be sound, because
/// there would still be only one owner valid at any one time with the
/// same ID, because two owners cannot occupy the same memory.
/// However `Pin` is useful because it helps the coder avoid
/// accidentally moving an owner from one address to another without
/// realizing it, and causing panics due to the changed owner ID.
///
/// The ID generated from this type cannot clash with IDs generated by
/// [`QCellOwner`] (which is also based on the addresses of occupied
/// memory, but always on the heap), or [`QCellOwnerSeq`] (which only
/// allocates odd IDs, which cannot clash with addresses from this
/// type which always have an alignment of 2).  So this should
/// successfully defend against all malicious and unsafe use.  If not,
/// please raise an issue.
///
/// The same unique ID may later be allocated to someone else once you
/// drop the returned owner, but this cannot be abused to cause unsafe
/// access to cells because there will still be only one owner active
/// at any one time with that ID.  Also it cannot be used maliciously
/// to access cells which don't belong to the new caller, because you
/// also need a reference to the cells.  So for example if you have a
/// graph of cells that is only accessible through a private
/// structure, then someone else getting the same owner ID makes no
/// difference, because they have no way to get a reference to those
/// cells.  In any case, you are probably going to drop all those
/// cells at the same time as dropping the owner, because they are no
/// longer of any use without the owner ID.
///
/// [`QCellOwner`]: struct.QCellOwner.html
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
    /// Create an owner that can be used for creating many [`QCell`]
    /// instances.
    #[inline]
    pub fn new() -> Self {
        Self {
            target: MAGIC_OWNER_ID_TARGET,
            _marker: PhantomPinned,
        }
    }

    /// Get the internal owner ID.  This may be used to create
    /// [`QCell`] instances without needing a borrow on this
    /// structure, which is useful if this structure is already
    /// borrowed.
    ///
    /// Requires this owner to be pinned before use.
    pub fn id(self: Pin<&Self>) -> QCellOwnerID {
        // Pin guarantees that our address will not change until we
        // are dropped, so we can use it as a unique ID.
        let raw_ptr: *const OwnerIDTarget = &self.target;
        QCellOwnerID(raw_ptr as usize)
    }

    /// Create a new cell owned by this owner instance.
    ///
    /// Requires this owner to be pinned before use.
    #[inline]
    pub fn cell<T>(self: Pin<&Self>, value: T) -> QCell<T> {
        let id: QCellOwnerID = self.into();
        id.cell(value)
    }

    /// Borrow contents of a [`QCell`] immutably (read-only).  Many
    /// [`QCell`] instances can be borrowed immutably at the same time
    /// from the same owner.  Panics if the [`QCell`] is not owned by
    /// this [`QCellOwnerPinned`].
    ///
    /// Requires this owner to be pinned before use.
    #[inline]
    pub fn ro<'a, T: ?Sized>(self: Pin<&'a Self>, qc: &'a QCell<T>) -> &'a T {
        owner_check!(self, qc);
        unsafe { &*qc.value.get() }
    }

    /// Borrow contents of a [`QCell`] mutably (read-write).  Only one
    /// [`QCell`] at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  Panics if the [`QCell`] is not owned
    /// by this [`QCellOwnerPinned`].
    ///
    /// Requires this owner to be pinned before use.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub fn rw<'a, T: ?Sized>(self: Pin<&'a mut Self>, qc: &'a QCell<T>) -> &'a mut T {
        owner_check!(self.as_ref(), qc);
        unsafe { &mut *qc.value.get() }
    }

    /// Borrow contents of two [`QCell`] instances mutably.  Panics if
    /// the two [`QCell`] instances point to the same memory.  Panics
    /// if either [`QCell`] is not owned by this [`QCellOwnerPinned`].
    ///
    /// Requires this owner to be pinned before use.
    #[inline]
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        self: Pin<&'a mut Self>,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
    ) -> (&'a mut T, &'a mut U) {
        owner_check!(self.as_ref(), qc1, qc2);
        distinct_check!(qc1, qc2);
        unsafe { (&mut *qc1.value.get(), &mut *qc2.value.get()) }
    }

    /// Borrow contents of three [`QCell`] instances mutably.  Panics
    /// if any pair of [`QCell`] instances point to the same memory.
    /// Panics if any [`QCell`] is not owned by this
    /// [`QCellOwnerPinned`].
    ///
    /// Requires this owner to be pinned before use.
    #[inline]
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        self: Pin<&'a mut Self>,
        qc1: &'a QCell<T>,
        qc2: &'a QCell<U>,
        qc3: &'a QCell<V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        owner_check!(self.as_ref(), qc1, qc2, qc3);
        distinct_check!(qc1, qc2, qc3);
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

    use super::{QCell, QCellOwnerPinned, QCellOwnerSeq};

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
        let id1 = owner1.as_ref().id();
        let owner2 = unsafe { QCellOwnerSeq::new() };
        let id2 = owner2.id;
        assert_ne!(id1.0, id2.0, "Expected ID 1/2 to be different");
        let owner3 = unsafe { QCellOwnerSeq::new() };
        let id3 = owner3.id;
        assert_ne!(id2.0, id3.0, "Expected ID 2/3 to be different");
        drop(owner2);
        drop(owner3);
        let owner4 = QCellOwnerPinned::new();
        pin_mut!(owner4);
        let id4 = owner4.as_ref().id();
        assert_ne!(id1.0, id4.0, "Expected ID 1/4 to be different");
        assert_ne!(id2.0, id4.0, "Expected ID 2/4 to be different");
        assert_ne!(id3.0, id4.0, "Expected ID 3/4 to be different");
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
    use super::{QCell, QCellOwner, QCellOwnerSeq};

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
        let id1 = owner1.id();
        let owner2 = QCellOwner::new();
        let id2 = owner2.id();
        assert_ne!(id1.0, id2.0, "Expected ID 1/2 to be different");
        drop(owner2);
        let owner3 = QCellOwner::new();
        let id3 = owner3.id();
        assert_ne!(id1.0, id3.0, "Expected ID 1/3 to be different");
        drop(owner3);
        drop(owner1);
        let owner4 = QCellOwner::new();
        let id4 = owner4.id();
        let owner5 = QCellOwner::new();
        let id5 = owner5.id();
        assert_ne!(id4.0, id5.0, "Expected ID 4/5 to be different");
    }

    #[test]
    fn qcell_fast_ids() {
        let owner1 = QCellOwner::new();
        let id1 = owner1.id();
        let owner2 = unsafe { QCellOwnerSeq::new() };
        let id2 = owner2.id();
        assert_ne!(id1.0, id2.0, "Expected ID 1/2 to be different");
        let owner3 = unsafe { QCellOwnerSeq::new() };
        let id3 = owner3.id();
        assert_ne!(id2.0, id3.0, "Expected ID 2/3 to be different");
        drop(owner2);
        drop(owner3);
        let owner4 = QCellOwner::new();
        let id4 = owner4.id();
        assert_ne!(id1.0, id4.0, "Expected ID 1/4 to be different");
        assert_ne!(id2.0, id4.0, "Expected ID 2/4 to be different");
        assert_ne!(id3.0, id4.0, "Expected ID 3/4 to be different");
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
    fn qcell_get_mut() {
        let owner = QCellOwner::new();
        let mut cell = QCell::new(&owner, 100u32);
        let mut_ref = cell.get_mut();
        *mut_ref = 50;
        let cell_ref = owner.ro(&cell);
        assert_eq!(*cell_ref, 50);
    }

    #[test]
    fn qcell_into_inner() {
        let owner = QCellOwner::new();
        let cell = QCell::new(&owner, 100u32);
        assert_eq!(cell.into_inner(), 100);
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
