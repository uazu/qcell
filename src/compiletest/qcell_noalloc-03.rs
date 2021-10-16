extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwnerPinned};
    use std::rc::Rc;
    use pin_utils::pin_mut;
    let mut owner1 = QCellOwnerPinned::new();
    pin_mut!(owner1);
    let mut owner2 = owner1;
    let rc = Rc::new(owner1.as_ref().cell(100u32));  // Compile fail
}
