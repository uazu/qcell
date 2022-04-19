extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwnerPinned};
    use std::rc::Rc;
    use pin_utils::pin_mut;
    let mut owner = QCellOwnerPinned::new();
    pin_mut!(owner);
    let c1 = Rc::new(owner.as_ref().cell(100u32));
    let c2 = Rc::new(owner.as_ref().cell(200u32));
    let c1ref = owner.as_ref().ro(&c1);
    let c1mutref = owner.as_mut().rw(&c1);    // Compile error
    println!("{}", *c1ref);
}
