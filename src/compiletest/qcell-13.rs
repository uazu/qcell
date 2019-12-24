extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    use std::rc::Rc;
    let owner = QCellOwner::new();
    let cell = QCell::new(&owner, Rc::new(100));

    // We aren't permitted to move the Rc to another thread
    std::thread::spawn(move || {    // Compile fail
        assert_eq!(100, **owner.ro(&cell));
    }).join();
}
