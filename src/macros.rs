/// A macro to generically uniquely borrow from any number of `*Cell` in this crate
/// You must pass references to cells directly to this macro, for example `&QCell<i32>`
/// works, but `&Rc<QCell<i32>>` does not (however you can call `Rc::as_ref` to convert it
/// to a reference).
/// 
/// The macro syntax follows the form
/// 
/// ```rust ignore
/// let owner: QCellOwner;
/// let cell_1: QCell<_>;
/// let cell_2: QCell<_>;
/// let cell_3: QCell<_>;
/// ...
/// let cell_n: QCell<_>;
/// 
/// let (rw_1, rw_2, rw_3, ..., rw_n) = qcell::rw!(owner => &cell_1, &cell_2, &cell_3, ..., &cell_n);
/// ```
/// 
/// As an example,
/// 
/// ```rust
///# use qcell::{QCell, QCellOwner};
///# let mut owner = QCellOwner::new();
///# let c1 = QCell::new(&owner, 100u32);
///# let c2 = QCell::new(&owner, 200u32);
/// let (c1mutref, c2mutref) = qcell::rw!(owner => &c1, &c2);
/// *c1mutref += 1;
/// *c2mutref += 2;
/// assert_eq!(303, owner.ro(&c1) + owner.ro(&c2));   // Success!
/// ```
#[macro_export]
macro_rules! rw {
    ($owner:expr => $($value:expr),+ $(,)?) => {{
        let output = $owner.rw_generic($crate::rw!(@tuple $($value),*));

        $crate::rw!(@destruct [$($value),*] [] output)
    }};
    (@tuple) => {
        $crate::tuple::Nil
    };
    (@tuple $value:expr $(, $rest:expr)*) => {
        $crate::tuple::Cons {
            value: $value,
            rest: $crate::rw!(@tuple $($rest),*)
        }
    };
    (@destruct [] [$($tup:expr),* $(,)?] $value:expr) => {{
        let $crate::tuple::Nil = $value;
        ($($tup),*)
    }};
    (@destruct [$a:expr $(, $rest:expr)*] [$($tup:tt)*] $value:expr) => {{
        let $crate::tuple::Cons { value, rest } = $value;

        $crate::rw!(@destruct [$($rest),*] [$($tup)* value,] rest)
    }};
}