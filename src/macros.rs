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
            value: &$value,
            rest: $crate::rw!(@tuple $($rest),*)
        }
    };
    (@destruct [] [$($tup:tt)*] $value:expr) => {{
        let $crate::tuple::Nil = $value;
        ($($tup)*)
    }};
    (@destruct [$a:expr $(, $rest:expr)*] [$($tup:tt)*] $value:expr) => {{
        let $crate::tuple::Cons { value, rest } = $value;

        $crate::rw!(@destruct [$($rest),*] [$($tup)* value,] rest)
    }};
}