extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    let mut owner = QCellOwner::new();
    let mut cell = QCell::new(&owner, 100);
    let cell_ref = owner.rw(&cell);
    *cell.get_mut() = 50; // Compile fail
    assert_eq!(100, *cell_ref);
}
