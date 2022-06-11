extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    let mut owner = QCellOwner::new();
    let mut cell = QCell::new(&owner, 100);
    let cell_ref = cell.get_mut();
    assert_eq!(100, *owner.rw(&cell)); // Compile fail
    *cell_ref = 50;
}
