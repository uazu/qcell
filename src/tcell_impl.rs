use core::marker::PhantomData;
use core::any::{Any, TypeId};

use std::sync::Mutex;
use std::collections::HashSet;

use crate::{ValueCell, ValueCellOwner};

pub type TCell<Mark, T> = ValueCell<SingletonOwner<Mark>, T>;

pub struct SingletonOwner<Mark>(PhantomData<Mark>);

pub struct SingletonMarker<Mark>(PhantomData<Mark>);

lazy_static! {
    static ref OWNERS: Mutex<HashSet<TypeId>> = Mutex::default();
}

impl<Mark: Any> Default for SingletonOwner<Mark> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Mark: Any> SingletonOwner<Mark> {
    pub fn new() -> Self {
        assert!(
            OWNERS.lock().unwrap().insert(TypeId::of::<Mark>()),
            "Invalid `SingletonOwner` was attempted to be created"
        );

        unsafe { Self::new_unchecked() }
    }
}

impl<Mark> SingletonOwner<Mark> {
    #[inline]
    pub const unsafe fn new_unchecked() -> Self {
        Self(PhantomData)
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

impl<Mark, T> TCell<Mark, T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self::from_marker(SingletonMarker(PhantomData), value)
    }
}

unsafe impl<Mark> ValueCellOwner for SingletonOwner<Mark> {
    type Marker = SingletonMarker<Mark>;

    #[inline]
    fn validate_marker(&self, &SingletonMarker(PhantomData): &Self::Marker) -> bool {
        true
    }

    #[inline]
    fn make_marker(&self) -> Self::Marker {
        SingletonMarker(PhantomData)
    }
}