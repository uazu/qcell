extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    let owner = QCellOwner::new();
    let mut cell = QCell::new(&owner, 100);
    let cell_ref = owner.ro(&cell);
    *cell.get_mut() = 50; // Compile fail
    assert_eq!(100, *cell_ref);
}
