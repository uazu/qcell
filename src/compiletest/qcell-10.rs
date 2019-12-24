extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    use std::cell::Cell;
    let owner = QCellOwner::new();
    let cell = QCell::new(&owner, Cell::new(100));

    // This would be a data race if the compiler permitted it, but it doesn't
    std::thread::spawn(|| owner.ro(&cell).set(200));  // Compile fail
    owner.ro(&cell).set(300);
}
