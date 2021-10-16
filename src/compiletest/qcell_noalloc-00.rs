extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwnerPinned};
    let mut owner1 = QCellOwnerPinned::new();
    let id = owner1.id();
}
