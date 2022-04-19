extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::LCell;
    use std::cell::Cell;
    fn is_sync<T: Sync>() {}
    is_sync::<LCell<'_, Cell<i32>>>();  // Compile fail
}
