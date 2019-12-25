use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use crate::{ValueCell, ValueCellOwner};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

pub type QCell<T> = ValueCell<RuntimeOwner, T>;

pub struct RuntimeOwner {
    id: u32,
}

pub struct RuntimeProxy(u32);

impl Default for RuntimeOwner {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeOwner {
    #[inline]
    pub fn new() -> Self {
        let mut id = NEXT_ID.load(Relaxed);

        loop {
            let next_id = if let Some(next_id) = id.checked_add(1) {
                next_id
            } else {
                panic!("Tried to create too many `RuntimeOwner`s");
            };

            match NEXT_ID.compare_exchange_weak(id, next_id, Relaxed, Relaxed) {
                Ok(_) => break,
                Err(next_id) => id = next_id
            }
        }

        Self { id }
    }

    #[inline]
    pub unsafe fn new_unchecked() -> Self {
        Self::from_id_unchecked(NEXT_ID.fetch_add(1, Relaxed))
    }

    #[inline]
    pub const unsafe fn from_id_unchecked(id: u32) -> Self {
        Self { id }
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

impl<T> QCell<T> {
    #[inline]
    pub fn new(owner: &RuntimeOwner, value: T) -> Self {
        owner.cell(value)
    }
}

unsafe impl ValueCellOwner for RuntimeOwner {
    type Proxy = RuntimeProxy;

    #[inline]
    fn validate_proxy(&self, &RuntimeProxy(id): &Self::Proxy) -> bool {
        self.id == id
    }

    #[inline]
    fn make_proxy(&self) -> Self::Proxy {
        RuntimeProxy(self.id)
    }
}