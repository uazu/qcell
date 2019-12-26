use core::marker::PhantomData;
use core::cell::UnsafeCell;

use crate::{ValueCell, ValueCellOwner};

struct Invariant<'mark>(PhantomData<UnsafeCell<&'mark ()>>);

unsafe impl Send for Invariant<'_> {}
unsafe impl Sync for Invariant<'_> {}

pub type LCell<'mark, T> = ValueCell<LifetimeOwner<'mark>, T>;

pub struct LifetimeOwner<'mark>(Invariant<'mark>);

pub struct LifetimeMarker<'mark>(Invariant<'mark>);

impl LifetimeOwner<'_> {
    pub fn scope<F: FnOnce(LifetimeOwner<'_>) -> R, R>(f: F) -> R {
        f(LifetimeOwner(Invariant(PhantomData)))
    }
    
    #[inline]
    pub const unsafe fn new_unchecked() -> Self {
        Self(Invariant(PhantomData))
    }
    
    #[inline]
    pub fn ro<'a, T: ?Sized>(&'a self, cell: &'a ValueCell<Self, T>) -> &'a T {
        ValueCellOwner::ro(self, cell)
    }

    #[inline]
    pub fn rw<'a, T: ?Sized>(&'a mut self, cell: &'a ValueCell<Self, T>) -> &'a mut T {
        ValueCellOwner::rw(self, cell)
    }

    #[inline]
    pub fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        c1: &'a ValueCell<Self, T>,
        c2: &'a ValueCell<Self, U>,
    ) -> (&'a mut T, &'a mut U) {
        ValueCellOwner::rw2(self, c1 ,c2)
    }

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

impl<T> LCell<'_, T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self::from_marker(LifetimeMarker(Invariant(PhantomData)), value)
    }
}

unsafe impl<'mark> ValueCellOwner for LifetimeOwner<'mark> {
    type Marker = LifetimeMarker<'mark>;

    #[inline]
    fn validate_marker(&self, &LifetimeMarker(Invariant(PhantomData)): &Self::Marker) -> bool {
        true
    }

    #[inline]
    fn make_marker(&self) -> Self::Marker {
        LifetimeMarker(Invariant(PhantomData))
    }
}