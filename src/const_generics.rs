use crate::LCell;
use crate::LCellOwner;
use crate::QCell;
use crate::QCellOwner;
#[cfg(feature = "std")]
use crate::{TCell, TCellOwner, TLCell, TLCellOwner};

impl<'id> LCellOwner<'id> {
    /// Borrow the contents of many `LCell` instances mutably.  Panics if
    /// any pair of `LCell` instances point to the same memory.
    #[cfg_attr(docsrs, doc(cfg(feature = "const_generics")))]
    pub fn rw_array<'a, T: ?Sized, const N: usize>(
        &'a mut self,
        lcells: [&'a LCell<'id, T>; N],
    ) -> [&'a mut T; N]
    where
        ValueOf<N>: LessThan256,
    {
        assert_array_unique(&lcells);
        lcells.map(|lc| unsafe { &mut *lc.value.get() })
    }
}

impl QCellOwner {
    /// Borrow the contents of many `QCell` instances mutably.  Panics if
    /// any pair of `QCell` instances point to the same memory.
    #[cfg_attr(docsrs, doc(cfg(feature = "const_generics")))]
    pub fn rw_array<'a, T: ?Sized, const N: usize>(
        &'a mut self,
        qcells: [&'a QCell<T>; N],
    ) -> [&'a mut T; N]
    where
        ValueOf<N>: LessThan256,
    {
        assert_array_unique(&qcells);
        qcells.map(|qc| unsafe { &mut *qc.value.get() })
    }
}

#[cfg(feature = "std")]
impl<Q: 'static> TCellOwner<Q> {
    /// Borrow the contents of many `TCell` instances mutably.  Panics if
    /// any pair of `TCell` instances point to the same memory.
    #[cfg_attr(docsrs, doc(cfg(feature = "const_generics")))]
    pub fn rw_array<'a, T: ?Sized, const N: usize>(
        &'a mut self,
        tcells: [&'a TCell<Q, T>; N],
    ) -> [&'a mut T; N]
    where
        ValueOf<N>: LessThan256,
    {
        assert_array_unique(&tcells);
        tcells.map(|tc| unsafe { &mut *tc.value.get() })
    }
}

#[cfg(feature = "std")]
impl<Q: 'static> TLCellOwner<Q> {
    /// Borrow the contents of many `TLCell` instances mutably.  Panics if
    /// any pair of `TLCell` instances point to the same memory.
    #[cfg_attr(docsrs, doc(cfg(feature = "const_generics")))]
    pub fn rw_array<'a, T: ?Sized, const N: usize>(
        &'a mut self,
        tlcells: [&'a TLCell<Q, T>; N],
    ) -> [&'a mut T; N]
    where
        ValueOf<N>: LessThan256,
    {
        assert_array_unique(&tlcells);
        tlcells.map(|tc| unsafe { &mut *tc.value.get() })
    }
}

fn assert_array_unique<C: ?Sized, const N: usize>(array: &[&C; N])
where
    ValueOf<N>: LessThan256,
{
    // threshold of 60 chosen by prototype benchmark
    if N < 60 {
        assert_array_unique_nested_loop(array);
    } else {
        assert_array_unique_sort(array);
    }
}

// This function uses a nested loop to check for duplicates.  This has O(n^2)
// complexity, but very low overhead and tends to outperform other functions on
// small arrays
fn assert_array_unique_nested_loop<C: ?Sized, const N: usize>(array: &[&C; N]) {
    let mut window = &array[..];
    // "consume" slice one element at a time
    while let Some(elem) = window.split_first().map(|(first, remain)| {
        window = remain;
        first
    }) {
        // iterate over whats left of the slice, checking for duplicates
        for other in window {
            if core::ptr::eq(elem, other) {
                panic!("Illegal to borrow same cell twice with rw_array");
            }
        }
    }
}

// This function uses a sort, and then iterates through the sorted list to
// check for adjacent duplicates.  This has O(n log n) complexity, but is still
// very fast for small arrays, and can be implemented with very minimal
// additional memory usage for arrays smaller than 256 elements.  Better
// time-complexity solutions are often slower for small arrays, and/or require
// more memory.
fn assert_array_unique_sort<C: ?Sized, const N: usize>(array: &[&C; N])
where
    ValueOf<N>: LessThan256,
{
    use std::convert::TryInto;
    let mut indecies = [0_u8; N];
    // N fits in a u8, asserted by the where clause, so this unwrap
    // should get optimized out
    for i in 0_u8..(N.try_into().unwrap()) {
        indecies[usize::from(i)] = i;
    }
    // now we sort the u8 indecies, so if there are any duplicates
    // they will be adjacent
    indecies.sort_unstable_by_key(|&i| array[usize::from(i)] as *const C as *const () as usize);
    // check for adjacent duplicates
    for window in indecies.windows(2) {
        let a = array[usize::from(window[0])];
        let b = array[usize::from(window[1])];
        if core::ptr::eq(a, b) {
            panic!("Illegal to borrow same cell twice with rw_array");
        }
    }
}

#[allow(dead_code)]
pub struct ValueOf<const N: usize> {}

pub trait LessThan256 {}

macro_rules! impl_less_than_256 {
    ( $( $value:literal )* ) => {
        $(
            impl LessThan256 for ValueOf<{ $value }> {}
        )*
    };
}

impl_less_than_256! {
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27
    28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52
    53 54 55 56 57 58 59 60 61 62 63 64 65 66 67 68 69 70 71 72 73 74 75 76 77
    78 79 80 81 82 83 84 85 86 87 88 89 90 91 92 93 94 95 96 97 98 99 100 101
    102 103 104 105 106 107 108 109 110 111 112 113 114 115 116 117 118 119 120
    121 122 123 124 125 126 127 128 129 130 131 132 133 134 135 136 137 138 139
    140 141 142 143 144 145 146 147 148 149 150 151 152 153 154 155 156 157 158
    159 160 161 162 163 164 165 166 167 168 169 170 171 172 173 174 175 176 177
    178 179 180 181 182 183 184 185 186 187 188 189 190 191 192 193 194 195 196
    197 198 199 200 201 202 203 204 205 206 207 208 209 210 211 212 213 214 215
    216 217 218 219 220 221 222 223 224 225 226 227 228 229 230 231 232 233 234
    235 236 237 238 239 240 241 242 243 244 245 246 247 248 249 250 251 252 253
    254 255
}
