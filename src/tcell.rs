use core::any::TypeId;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
#[cfg(all(feature = "std", not(feature = "exclusion-set")))]
use once_cell::sync::Lazy;
#[cfg(all(feature = "std", not(feature = "exclusion-set")))]
use std::{
    collections::HashSet,
    sync::{Condvar, Mutex},
};

use super::Invariant;

#[cfg(all(feature = "std", not(feature = "exclusion-set")))]
static SINGLETON_CHECK: Lazy<Mutex<HashSet<TypeId>>> = Lazy::new(|| Mutex::new(HashSet::new()));
#[cfg(all(feature = "std", not(feature = "exclusion-set")))]
static SINGLETON_CHECK_CONDVAR: Lazy<Condvar> = Lazy::new(Condvar::new);
#[cfg(feature = "exclusion-set")]
static SINGLETON_CHECK_SET: exclusion_set::Set<TypeId> = exclusion_set::Set::new();

/// Borrowing-owner of zero or more [`TCell`](struct.TCell.html)
/// instances.
///
/// See [crate documentation](index.html).
pub struct TCellOwner<Q: 'static> {
    // Allow Send and Sync, and Q is invariant
    typ: PhantomData<Invariant<Q>>,
}

impl<Q: 'static> Drop for TCellOwner<Q> {
    #[cfg(all(not(feature = "exclusion-set"), feature = "std"))]
    fn drop(&mut self) {
        // Remove the TypeId of Q from the HashSet, indicating that
        // no more instances of TCellOwner<Q> exist.
        SINGLETON_CHECK.lock().unwrap().remove(&TypeId::of::<Q>());

        // Wake up all threads waiting in TCellOwner::wait_for_new()
        // to check if their Q was removed from the HashSet.
        SINGLETON_CHECK_CONDVAR.notify_all();
    }

    #[cfg(feature = "exclusion-set")]
    fn drop(&mut self) {
        // Remove the TypeId of Q from the Set, indicating that
        // no more instances of TCellOwner<Q> exist.
        // SAFETY: the precondition of remove is satisfied since
        // this can be the only TCellOwner for a given Q.
        unsafe {
            SINGLETON_CHECK_SET.remove(&TypeId::of::<Q>());
        }
    }

    #[cfg(not(any(feature = "std", feature = "exclusion-set")))]
    fn drop(&mut self) {
        // constructors should be unavailable with this feature set, so the
        // destructor should be unreachable
        unreachable!()
    }
}

#[cfg(any(feature = "std", feature = "exclusion-set"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "exclusion-set"))))]
impl<Q: 'static> Default for TCellOwner<Q> {
    fn default() -> Self {
        TCellOwner::new()
    }
}

impl<Q: 'static> TCellOwner<Q> {
    /// Create the singleton owner instance.  Each owner may be used
    /// to create many `TCell` instances.  There may be only one
    /// instance of this type per process at any given time for each
    /// different marker type `Q`.  This call panics if a second
    /// simultaneous instance is created.
    ///
    /// Keep in mind that in Rust, tests are run in parallel unless
    /// specified otherwise (using e.g. `RUST_TEST_THREADS`), so
    /// this panic may be more easy to trigger than you might think.
    /// To avoid this panic, consider using the methods
    /// [`TCellOwner::wait_for_new`] or [`TCellOwner::try_new`] instead.
    #[cfg(any(feature = "std", feature = "exclusion-set"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "exclusion-set"))))]
    pub fn new() -> Self {
        if let Some(owner) = TCellOwner::try_new() {
            owner
        } else {
            panic!("Illegal to create two TCellOwner instances with the same marker type parameter")
        }
    }

    /// Same as [`TCellOwner::new`], except if another `TCellOwner`
    /// of this type `Q` already exists, this returns `None` instead
    /// of panicking.
    #[cfg(all(not(feature = "exclusion-set"), feature = "std"))]
    pub fn try_new() -> Option<Self> {
        if SINGLETON_CHECK.lock().unwrap().insert(TypeId::of::<Q>()) {
            Some(Self { typ: PhantomData })
        } else {
            None
        }
    }

    /// Same as [`TCellOwner::new`], except if another `TCellOwner`
    /// of this type `Q` already exists, this returns `None` instead
    /// of panicking.
    #[cfg(feature = "exclusion-set")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "exclusion-set"))))]
    pub fn try_new() -> Option<Self> {
        if SINGLETON_CHECK_SET.try_insert(TypeId::of::<Q>()) {
            Some(Self { typ: PhantomData })
        } else {
            None
        }
    }

    /// Same as [`TCellOwner::new`], except if another `TCellOwner`
    /// of this type `Q` already exists, this function blocks the thread
    /// until that other instance is dropped.  This will of course deadlock
    /// if that other instance is owned by the same thread.
    ///
    /// Note that owners are expected to be relatively long-lived.  If
    /// you need to access cells associated with a given marker type
    /// from several different threads, the most efficient pattern is
    /// to have a single long-lived owner shared between threads, with
    /// a `Mutex` or `RwLock` to control access.  This call is
    /// intended to help when several independent tests need to run
    /// which use the same marker type internally.
    #[cfg(all(not(feature = "exclusion-set"), feature = "std"))]
    pub fn wait_for_new() -> Self {
        // Lock the HashSet mutex.
        let hashset_guard = SINGLETON_CHECK.lock().unwrap();

        // If the HashSet already contains the TypeId of Q, there is
        // another TCellOwner. Block the thread until it gets dropped.
        // (the HashSet mutex is unlocked while waiting)
        let mut hashset_guard = SINGLETON_CHECK_CONDVAR
            .wait_while(hashset_guard, |hashset| {
                hashset.contains(&TypeId::of::<Q>())
            })
            .unwrap();

        // If we get here, no other TCellOwner of this type exists.
        // Return a new TCellOwner.  When dropped, it will remove the
        // TypeId of Q from the HashSet, and notify all waiting threads.
        let inserted = hashset_guard.insert(TypeId::of::<Q>());
        assert!(inserted);
        Self { typ: PhantomData }
    }

    /// Same as [`TCellOwner::new`], except if another `TCellOwner`
    /// of this type `Q` already exists, this function blocks the thread
    /// until that other instance is dropped.  This will of course deadlock
    /// if that other instance is owned by the same thread.
    ///
    /// Note that owners are expected to be relatively long-lived.  If
    /// you need to access cells associated with a given marker type
    /// from several different threads, the most efficient pattern is
    /// to have a single long-lived owner shared between threads, with
    /// a `Mutex` or `RwLock` to control access.  This call is
    /// intended to help when several independent tests need to run
    /// which use the same marker type internally.
    #[cfg(all(feature = "std", feature = "exclusion-set"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "std", all(feature = "exclusion-set", feature = "std"))))
    )]
    pub fn wait_for_new() -> Self {
        SINGLETON_CHECK_SET.wait_to_insert(TypeId::of::<Q>());
        Self { typ: PhantomData }
    }

    /// Create a new cell owned by this owner instance.  See also
    /// [`TCell::new`].
    ///
    /// [`TCell::new`]: struct.TCell.html
    pub fn cell<T>(&self, value: T) -> TCell<Q, T> {
        TCell::<Q, T>::new(value)
    }

    /// Borrow contents of a `TCell` immutably (read-only).  Many
    /// `TCell` instances can be borrowed immutably at the same time
    /// from the same owner.
    #[inline]
    pub fn ro<'a, T: ?Sized>(&'a self, tc: &'a TCell<Q, T>) -> &'a T {
        unsafe { &*tc.value.get() }
    }

    /// Borrow contents of a `TCell` mutably (read-write).  Only one
    /// `TCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.
    #[inline]
    pub fn rw<'a, T: ?Sized>(&'a mut self, tc: &'a TCell<Q, T>) -> &'a mut T {
        unsafe { &mut *tc.value.get() }
    }

    /// Borrow contents of two `TCell` instances mutably.  Panics if
    /// the two `TCell` instances point to the same memory.
    #[inline]
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        tc1: &'a TCell<Q, T>,
        tc2: &'a TCell<Q, U>,
    ) -> (&'a mut T, &'a mut U) {
        assert!(
            tc1 as *const _ as *const () as usize != tc2 as *const _ as *const () as usize,
            "Illegal to borrow same TCell twice with rw2()"
        );
        unsafe { (&mut *tc1.value.get(), &mut *tc2.value.get()) }
    }

    /// Borrow contents of three `TCell` instances mutably.  Panics if
    /// any pair of `TCell` instances point to the same memory.
    #[inline]
    pub fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        tc1: &'a TCell<Q, T>,
        tc2: &'a TCell<Q, U>,
        tc3: &'a TCell<Q, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert!(
            (tc1 as *const _ as *const () as usize != tc2 as *const _ as *const () as usize)
                && (tc2 as *const _ as *const () as usize != tc3 as *const _ as *const () as usize)
                && (tc3 as *const _ as *const () as usize != tc1 as *const _ as *const () as usize),
            "Illegal to borrow same TCell twice with rw3()"
        );
        unsafe {
            (
                &mut *tc1.value.get(),
                &mut *tc2.value.get(),
                &mut *tc3.value.get(),
            )
        }
    }
}

/// Cell whose contents is owned (for borrowing purposes) by a
/// [`TCellOwner`].
///
/// To borrow from this cell, use the borrowing calls on the
/// [`TCellOwner`] instance that shares the same marker type.
///
/// See also [crate documentation](index.html).
///
/// [`TCellOwner`]: struct.TCellOwner.html
#[repr(transparent)]
pub struct TCell<Q, T: ?Sized> {
    // Use Invariant<Q> for invariant parameter
    owner: PhantomData<Invariant<Q>>,

    // It's fine to Send a TCell to a different thread if the contained
    // type is Send, because you can only send something if nothing
    // borrows it, so nothing can be accessing its contents.
    //
    // `UnsafeCell` disables `Sync` and already gives the right `Send` implementation.
    // `Sync` is re-enabled below under certain conditions.
    value: UnsafeCell<T>,
}

impl<Q, T> TCell<Q, T> {
    /// Create a new `TCell` owned for borrowing purposes by the
    /// `TCellOwner` derived from the same marker type `Q`.
    #[inline]
    pub const fn new(value: T) -> TCell<Q, T> {
        TCell {
            owner: PhantomData,
            value: UnsafeCell::new(value),
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

impl<Q, T: ?Sized> TCell<Q, T> {
    /// Borrow contents of this cell immutably (read-only).  Many
    /// `TCell` instances can be borrowed immutably at the same time
    /// from the same owner.
    #[inline]
    pub fn ro<'a>(&'a self, owner: &'a TCellOwner<Q>) -> &'a T {
        owner.ro(self)
    }

    /// Borrow contents of this cell mutably (read-write).  Only one
    /// `TCell` at a time can be borrowed from the owner using this
    /// call.  The returned reference must go out of scope before
    /// another can be borrowed.  To mutably borrow from two or three
    /// cells at the same time, see [`TCellOwner::rw2`] or
    /// [`TCellOwner::rw3`].
    #[inline]
    pub fn rw<'a>(&'a self, owner: &'a mut TCellOwner<Q>) -> &'a mut T {
        owner.rw(self)
    }

    /// Returns a mutable reference to the underlying data
    ///
    /// Note that this is only useful at the beginning-of-life or
    /// end-of-life of the cell when you have exclusive access to it.
    /// Normally you'd use [`TCell::rw`] or [`TCellOwner::rw`] to get
    /// a mutable reference to the contents of the cell.
    ///
    /// Safety: This call borrows `TCell` mutably which guarantees
    /// that we possess the only reference.  This means that there can
    /// be no active borrows of other forms, even ones obtained using
    /// an immutable reference.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

impl<Q: 'static, T: Default + ?Sized> Default for TCell<Q, T> {
    fn default() -> Self {
        TCell::new(T::default())
    }
}

// We can add a Sync implementation, since it's fine to send a &TCell
// to another thread, and even mutably borrow the value there, as long
// as T is Send and Sync.
//
// The reason why TCell<T>'s impl of Sync requires T: Send + Sync
// instead of just T: Sync is that TCell provides interior mutability.
// If you send a &TCell<T> (and its owner) to a different thread, you
// can call .rw() to get a &mut T, and use std::mem::swap() to move
// the T, effectively sending the T to that other thread. That's not
// allowed if T: !Send.
//
// Note that the bounds on T for TCell<T>'s impl of Sync are the same
// as those of std::sync::RwLock<T>. That's not a coincidence.
// The way these types let you access T concurrently is the same,
// even though the locking mechanisms are different.
unsafe impl<Q, T: Send + Sync + ?Sized> Sync for TCell<Q, T> {}

#[cfg(test)]
mod tests {
    use super::{TCell, TCellOwner};
    #[test]
    #[should_panic]
    fn tcell_singleton_1() {
        struct Marker;
        let _owner1 = TCellOwner::<Marker>::new();
        let _owner2 = TCellOwner::<Marker>::new(); // Panic here
    }

    #[test]
    fn tcell_singleton_2() {
        struct Marker;
        let owner1 = TCellOwner::<Marker>::new();
        drop(owner1);
        let _owner2 = TCellOwner::<Marker>::new();
    }

    #[test]
    fn tcell_singleton_3() {
        struct Marker1;
        struct Marker2;
        let _owner1 = TCellOwner::<Marker1>::new();
        let _owner2 = TCellOwner::<Marker2>::new();
    }

    #[test]
    fn tcell_singleton_try_new() {
        struct Marker;
        let owner1 = TCellOwner::<Marker>::try_new();
        assert!(owner1.is_some());
        let owner2 = TCellOwner::<Marker>::try_new();
        assert!(owner2.is_none());
    }

    #[test]
    fn tcell() {
        struct Marker;
        type ACellOwner = TCellOwner<Marker>;
        type ACell<T> = TCell<Marker, T>;
        let mut owner = ACellOwner::new();
        let c1 = ACell::new(100u32);
        let c2 = owner.cell(200u32);
        (*owner.rw(&c1)) += 1;
        (*owner.rw(&c2)) += 2;
        let c1ref = owner.ro(&c1);
        let c2ref = owner.ro(&c2);
        let total = *c1ref + *c2ref;
        assert_eq!(total, 303);
    }

    #[test]
    #[should_panic]
    fn tcell_threads() {
        struct Marker;
        type ACellOwner = TCellOwner<Marker>;
        // Do it this way around to make the panic appear in the main
        // thread, to avoid spurious messages in the test output.
        let (tx, rx) = std::sync::mpsc::sync_channel(0);
        std::thread::spawn(move || {
            let mut _owner = ACellOwner::new();
            tx.send(()).unwrap();
            // Delay long enough for the panic to occur; this will
            // fail if the main thread panics, so ignore that
            let _ = tx.send(());
        });
        rx.recv().unwrap();
        let mut _owner = ACellOwner::new(); // Panics here
        let _ = rx.recv();
    }

    #[cfg(feature = "std")]
    #[test]
    fn tcell_wait_for_new_in_100_threads() {
        use rand::Rng;
        use std::sync::Arc;
        struct Marker;
        type ACellOwner = TCellOwner<Marker>;
        type ACell = TCell<Marker, i32>;
        let cell_arc = Arc::new(ACell::new(0));
        let mut handles = vec![];
        for _ in 0..100 {
            let cell_arc_clone = cell_arc.clone();
            let handle = std::thread::spawn(move || {
                // wait a bit
                let mut rng = rand::thread_rng();
                std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(0..10)));
                // create a new owner
                let mut owner = ACellOwner::wait_for_new();
                // read the cell's current value
                let current_cell_val = *owner.ro(&*cell_arc_clone);
                // wait a bit more
                std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(0..10)));
                // write the old cell value + 1 to the cell
                // (no other thread should have been able to modify the cell in the
                // meantime because we still hold on to the owner)
                *owner.rw(&*cell_arc_clone) = current_cell_val + 1;
            });
            handles.push(handle);
        }
        for handle in handles {
            assert!(handle.join().is_ok());
        }
        let owner = ACellOwner::wait_for_new();
        assert_eq!(*owner.ro(&*cell_arc), 100);
    }

    #[cfg(feature = "std")]
    #[test]
    fn tcell_wait_for_new_timeout() {
        fn assert_time_out<F>(d: std::time::Duration, f: F)
        where
            F: FnOnce(),
            F: Send + 'static,
        {
            let (done_tx, done_rx) = std::sync::mpsc::channel();
            let _handle = std::thread::spawn(move || {
                let val = f();
                done_tx.send(()).unwrap();
                val
            });

            assert!(
                done_rx.recv_timeout(d).is_err(),
                "ACellOwner::wait_for_new completed (but it shouldn't have)"
            );
        }

        assert_time_out(std::time::Duration::from_millis(1000), || {
            struct Marker;
            type ACellOwner = TCellOwner<Marker>;

            let _owner1 = ACellOwner::new();
            let _owner2 = ACellOwner::wait_for_new();
        });
    }

    #[test]
    fn tcell_get_mut() {
        struct Marker;
        type ACellOwner = TCellOwner<Marker>;
        type ACell<T> = TCell<Marker, T>;
        let owner = ACellOwner::new();
        let mut cell = ACell::new(100u32);
        let mut_ref = cell.get_mut();
        *mut_ref = 50;
        let cell_ref = owner.ro(&cell);
        assert_eq!(*cell_ref, 50);
    }

    #[test]
    fn tcell_into_inner() {
        struct Marker;
        type ACell<T> = TCell<Marker, T>;
        let cell = ACell::new(100u32);
        assert_eq!(cell.into_inner(), 100);
    }

    #[test]
    fn tcell_unsized() {
        struct Marker;
        type ACellOwner = TCellOwner<Marker>;
        type ACell<T> = TCell<Marker, T>;
        let mut owner = ACellOwner::new();
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
        fn series(init: u32, is_squares: bool) -> Box<ACell<dyn Series>> {
            if is_squares {
                Box::new(ACell::new(Squares(init)))
            } else {
                Box::new(ACell::new(Integers(init as u64)))
            }
        }

        let own = &mut owner;
        let cell1 = series(4, false);
        let cell2 = series(7, true);
        let cell3 = series(3, true);
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
