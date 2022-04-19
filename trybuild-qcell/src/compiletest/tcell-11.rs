extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::TCell;
    use std::cell::Cell;
    struct Marker;
    fn is_sync<T: Sync>() {}
    is_sync::<TCell<Marker, Cell<i32>>>();  // Compile fail
}
