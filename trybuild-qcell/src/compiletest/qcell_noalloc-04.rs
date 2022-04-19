extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwnerPinned};
    use std::rc::Rc;
    let mut owner1 = QCellOwnerPinned::new();
    let owner2 = owner1.clone();  // Compile fail
}
