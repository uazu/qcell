extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::TCell;
    use std::rc::Rc;
    struct Marker;
    fn is_send<T: Send>() {}
    is_send::<TCell<Marker, Rc<()>>>();  // Compile fail
}
