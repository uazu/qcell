extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwnerPinned};
    use std::rc::Rc;
    let mut owner = Box::pin(QCellOwnerPinned::new());
    let cell = owner.as_ref().cell(Rc::new(100));
   
    // We aren't permitted to move the Rc to another thread
    std::thread::spawn(move || {    // Compile fail
        assert_eq!(100, **owner.as_ref().ro(&cell));
    }).join();
}
