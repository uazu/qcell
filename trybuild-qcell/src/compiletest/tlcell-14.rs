extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::TLCell;
    use std::rc::Rc;
    struct Marker;
    fn is_send<T: Send>() {}
    is_send::<TLCell<Marker, Rc<()>>>();  // Compile fail
}
