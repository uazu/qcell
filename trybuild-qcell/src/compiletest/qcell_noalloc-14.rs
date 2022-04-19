extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::QCell;
    use std::rc::Rc;
    fn is_sync<T: Sync>() {}
    is_sync::<QCell<Rc<()>>>();  // Compile fail
}
