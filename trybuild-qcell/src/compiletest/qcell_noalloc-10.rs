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
    fn test(o: Pin<&mut QCellOwnerPinned>) {}
   
    let c1ref = owner.as_ref().ro(&c1);
    test(owner.as_mut());    // Compile error
    println!("{}", *c1ref);
}
