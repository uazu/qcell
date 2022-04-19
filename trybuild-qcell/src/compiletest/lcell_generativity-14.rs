extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::LCell;
    use std::rc::Rc;
    fn is_send<T: Send>() {}
    is_send::<LCell<'_, Rc<()>>>();  // Compile fail
}
