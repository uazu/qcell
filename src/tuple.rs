
pub struct Nil;

#[derive(Clone, Copy)]
pub struct Cons<T, R> {
    pub value: T,
    pub rest: R
}

impl crate::Sealed for Nil {}
impl<T, R> crate::Sealed for Cons<T, R> {}

/// This is a generic interface that all of `QCell`, `TCell`, `TLCell`, and `LCell`
/// implement in order to uniquely borrow an arbitrary number of arguments
/// 
/// # Safety
/// 
/// The pointer returned by `rw_ptr` must be valid to write to for as long as
/// the corresponding lock on the owner is held
pub unsafe trait GenericCell: crate::Sealed {
    type Value;

    fn rw_ptr(&self) -> *mut Self::Value;
}

/// Enumerate the addresses of the cells
/// 
/// # Safety
/// 
/// `iter_addr` must account for all cells in the type-list exactly once
pub unsafe trait IterAddresses: crate::Sealed {
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

/// Validates that each reference in the type-list is unique within the type-list
/// 
/// # Safety
/// 
/// `all_unique` must return false if there are any aliasing references in the type-list
pub unsafe trait ValidateUniqueness: crate::Sealed {
    fn all_unique(&self) -> bool;
}

// nil is trivially unique
unsafe impl ValidateUniqueness for Nil {
    #[inline]
    fn all_unique(&self) -> bool { true }
}

// Cons is unique if the rest of the type-list is unique, 
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

/// Gets a type-list of unique references into the type-list of cells
/// 
/// # Safety
/// 
/// Must only be implemented for type-lists of `QCell`, `TCell`, `TLCell`, `LCell`
pub unsafe trait LoadValues<'a>: crate::Sealed {
    type Output;

    /// # Safety
    /// 
    /// For all cells you must validate that all cells in the type-list are unique
    /// 
    /// For `QCell` you must also validate the owner
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
