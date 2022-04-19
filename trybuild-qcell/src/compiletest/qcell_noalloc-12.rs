extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCellOwnerPinned, QCell};
    let mut owner = Box::pin(QCellOwnerPinned::new());
    let cell = owner.as_ref().cell(100);
    let val_ref = owner.as_ref().ro(&cell);
    std::thread::spawn(move || {
        assert_eq!(*owner.as_ref().ro(&cell), 100);
    }).join();
    assert_eq!(*val_ref, 100);
}
