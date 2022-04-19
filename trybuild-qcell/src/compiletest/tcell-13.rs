extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::TCell;
    use std::rc::Rc;
    struct Marker;
    fn is_sync<T: Sync>() {}
    is_sync::<TCell<Marker, Rc<()>>>();  // Compile fail
}
