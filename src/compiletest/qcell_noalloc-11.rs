extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwnerPinned};
    use std::{rc::Rc, pin::Pin};
    use pin_utils::pin_mut;
    let mut owner = QCellOwnerPinned::new();
    pin_mut!(owner);
    let c1 = Rc::new(owner.as_ref().cell(100u32));
    let c2 = Rc::new(owner.as_ref().cell(200u32));
    fn test(o: Pin<&QCellOwnerPinned>) {}
   
    let c1mutref = owner.as_mut().rw(&c1);
    test(owner.as_ref());    // Compile error
    *c1mutref += 1;
}
