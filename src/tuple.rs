//! This module holds the lion's share of the magic that makes var-args emulation possible
//! 
//! This is heavily inspired by [`frunk`](https://crates.io/crates/frunk/),
//! so please look into that for more details about how type-lists work.
//! 
//! # Types and Traits
//! 
//! `Cons` and `Nil` are the backbone of the type-lists. They form a sort of
//! compile time managed linked list. `Nil` signifies the end of the type-list,
//! and `Cons` contains the current node's value, and the rest of the type-list.
//! 
//! `GenericCell` is a generalization of `QCell`, `TCell`, `TLCell`, and `LCell`.
//! This allows `LoadValues` to be written generically.
//! 
//! `LoadValues` gets a iterates through the type-list using `impl` for `Cons`
//! and `Nil` and extracts a unique reference to each element in the type-list.
//! `Nil` yields `Nil` because `Nil` means the empty list, so nothing to extract
//! `Cons<&C, R>` yields `Cons<&mut C::Value, R::Output>` because `C` is one of
//! the cells listed above, and `R` is a recusion, going through `LoadValues` again
//! 
//! `ValidateUniqueness` works in a similar way. It uses the property `IterAddresses`
//! to extract out the addresses of the cells. It then recusively iterates through the
//! cells in the list and checks that all of the addresses are unique.
//! 
//! `IterAddresses` will yield the addresses (as usizes) of all items in the type-list.
//! 
//! The strategy for checking uniqueness is, ensure that all cells have a different address from 
//! all cells after them. This triangle strategy is guaranteed to pair-wise check each
//! and every possible pair of addresses exactly once.
//! 
//! ```text
//! For example, let's check the addresses [0, 1, 2, 3]
//! 
//! 0 will be checked against [1, 2, 3]
//! 1 will be checked against [2, 3]
//! 2 will be checked against [3]
//! 3 will be checked against []
//! ```
//! 
//! This finite number of deterministic const-propogation compatible checks
//! are really easy for LLVM to optimize away, and in the happy path, LLVM 
//! will completely eliminate all checks. In fact, all of the type-list shennanigans 
//! are super optimzable because they are `inline`, fully deterministic, and
//! play really well with const-propogation.
//! 
//! 

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
