extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    use std::rc::Rc;
    let mut owner1 = QCellOwner::new();
    let owner2 = owner1.clone();  // Compile fail
}
