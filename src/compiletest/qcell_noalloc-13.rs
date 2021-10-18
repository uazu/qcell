extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::QCell;
    use std::cell::Cell;
    fn is_sync<T: Sync>() {}
    is_sync::<QCell<Cell<i32>>>();  // Compile fail
}
