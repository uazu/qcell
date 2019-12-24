
pub struct Nil;

#[derive(Clone, Copy)]
pub struct Cons<T, R> {
    pub value: T,
    pub rest: R
}

pub unsafe trait GenericCell {
    type Value;

    fn rw_ptr(&self) -> *mut Self::Value;
}

pub unsafe trait IterAddresses {
    type Iter: Iterator<Item = usize>;

    fn iter_addr(&self) -> Self::Iter;
}

unsafe impl IterAddresses for Nil {
    type Iter = std::iter::Empty<usize>;

    #[inline]
    fn iter_addr(&self) -> Self::Iter {
        std::iter::empty()
    }
}

unsafe impl<T, R> IterAddresses for Cons<&T, R>
where
    R: IterAddresses
{
    type Iter = std::iter::Chain<std::iter::Once<usize>, R::Iter>;

    #[inline]
    fn iter_addr(&self) -> Self::Iter {
        std::iter::once(self.value as *const _ as usize).chain(self.rest.iter_addr())
    }
}

pub unsafe trait ValidateUniqueness {
    fn all_unique(&self) -> bool;
}

// nil is trivially unique
unsafe impl ValidateUniqueness for Nil {
    #[inline]
    fn all_unique(&self) -> bool { true }
}

// Cons is unique if the rest of the list is unique, 
// and if the current address is not the same as any other address
unsafe impl<T, R> ValidateUniqueness for Cons<&T, R>
where
    R: IterAddresses + ValidateUniqueness
{
    #[inline]
    fn all_unique(&self) -> bool {
        if self.rest.all_unique() {
            let addr = self.value as *const _ as usize;

            self.rest.iter_addr().all(move |a| a != addr)
        } else {
            false
        }
    }
}

pub unsafe trait LoadValues<'a> {
    type Output;

    unsafe fn load_values(self) -> Self::Output;
}

unsafe impl LoadValues<'_> for Nil {
    type Output = Self;

    #[inline]
    unsafe fn load_values(self) -> Self::Output { Self }
}

unsafe impl<'a, T, R> LoadValues<'a> for Cons<&'a T, R>
where
    T: GenericCell,
    R: LoadValues<'a>
{
    type Output = Cons<&'a mut T::Value, R::Output>;

    #[inline]
    unsafe fn load_values(self) -> Self::Output {
        Cons {
            value: &mut *self.value.rw_ptr(),
            rest: self.rest.load_values()
        }
    }
}
