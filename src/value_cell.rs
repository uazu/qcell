use core::cell::UnsafeCell;

pub unsafe trait ValueCellOwner: Sized {
    type Proxy;

    fn validate_proxy(&self, proxy: &Self::Proxy) -> bool;

    fn make_proxy(&self) -> Self::Proxy;

    fn cell<T>(&self, value: T) -> ValueCell<Self, T> {
        ValueCell {
            proxy: self.make_proxy(),
            value: UnsafeCell::new(value)
        }
    }

    fn owns<T: ?Sized>(&self, cell: &ValueCell<Self, T>) -> bool {
        self.validate_proxy(cell.proxy())
    }

    fn ro<'a, T: ?Sized>(&'a self, cell: &'a ValueCell<Self, T>) -> &'a T {
        assert!(self.owns(cell), "You cannot borrow from a `ValueCell` using a different owner!");
        unsafe { &*cell.as_ptr() }
    }

    fn rw<'a, T: ?Sized>(&'a mut self, cell: &'a ValueCell<Self, T>) -> &'a mut T {
        assert!(self.owns(cell), "You cannot borrow from a `ValueCell` using a different owner!");
        unsafe { &mut *cell.as_ptr() }
    }

    fn rw2<'a, T: ?Sized, U: ?Sized>(
        &'a mut self,
        c1: &'a ValueCell<Self, T>,
        c2: &'a ValueCell<Self, U>,
    ) -> (&'a mut T, &'a mut U) {
        assert!(self.owns(c1), "You cannot borrow from a `ValueCell` using a different owner!");
        assert!(self.owns(c2), "You cannot borrow from a `ValueCell` using a different owner!");
        assert_ne!(c1 as *const _ as *const () as usize, c2 as *const _ as *const () as usize, "You cannot uniquely borrow the same cell multiple times");
        unsafe { (&mut *c1.as_ptr(), &mut *c2.as_ptr()) }
    }

    fn rw3<'a, T: ?Sized, U: ?Sized, V: ?Sized>(
        &'a mut self,
        c1: &'a ValueCell<Self, T>,
        c2: &'a ValueCell<Self, U>,
        c3: &'a ValueCell<Self, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert!(self.owns(c1), "You cannot borrow from a `ValueCell` using a different owner!");
        assert!(self.owns(c2), "You cannot borrow from a `ValueCell` using a different owner!");
        assert_ne!(c1 as *const _ as *const () as usize, c2 as *const _ as *const () as usize, "You cannot uniquely borrow the same cell multiple times");
        assert_ne!(c1 as *const _ as *const () as usize, c3 as *const _ as *const () as usize, "You cannot uniquely borrow the same cell multiple times");
        assert_ne!(c2 as *const _ as *const () as usize, c3 as *const _ as *const () as usize, "You cannot uniquely borrow the same cell multiple times");
        unsafe { (&mut *c1.as_ptr(), &mut *c2.as_ptr(), &mut *c3.as_ptr()) }
    }
}

pub struct ValueCell<O: ValueCellOwner, T: ?Sized> {
    proxy: O::Proxy,
    value: UnsafeCell<T>
}

unsafe impl<O: ValueCellOwner, T: ?Sized> Sync for ValueCell<O, T>
where
    O::Proxy: Sync,
    T: Send + Sync {}

impl<O: ValueCellOwner, T> ValueCell<O, T> {
    pub fn into_value(self) -> T {
        self.value.into_inner()
    }
}

impl<O: ValueCellOwner, T: ?Sized> ValueCell<O, T> {
    pub fn as_ptr(&self) -> *mut T {
        self.value.get()
    }

    pub fn proxy(&self) -> &O::Proxy {
        &self.proxy
    }

    pub fn value_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.value.get()
        }
    }
}