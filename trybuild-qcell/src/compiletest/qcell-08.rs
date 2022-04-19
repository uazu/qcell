extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCellOwner, QCell};
    let owner = QCellOwner::new();
    let cell = QCell::new(&owner, 100);
    let val_ref = owner.ro(&cell);
    std::thread::spawn(move || {
        assert_eq!(*owner.ro(&cell), 100);
    }).join();
    assert_eq!(*val_ref, 100);
}
