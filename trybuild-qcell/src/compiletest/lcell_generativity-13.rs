extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::LCell;
    use std::rc::Rc;
    fn is_sync<T: Sync>() {}
    is_sync::<LCell<'_, Rc<()>>>();  // Compile fail
}
