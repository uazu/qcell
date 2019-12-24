extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::QCell;
    use std::rc::Rc;
    fn is_send<T: Send>() {}
    is_send::<QCell<Rc<()>>>();  // Compile fail
}
