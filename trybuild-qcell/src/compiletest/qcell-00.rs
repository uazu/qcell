extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    use std::rc::Rc;
    let mut owner1 = QCellOwner::new();
    let mut owner2 = owner1;
    let rc = Rc::new(QCell::new(&owner1, 100u32));  // Compile fail
}
