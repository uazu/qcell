extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwnerPinned};
    use std::rc::Rc;
    use pin_utils::pin_mut;
    let mut owner1 = QCellOwnerPinned::new();
    pin_mut!(owner1);
    let owner2 = owner1.clone();  // Compile fail
}
