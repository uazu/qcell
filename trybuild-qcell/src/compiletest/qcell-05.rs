extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    use std::rc::Rc;
    let mut owner = QCellOwner::new();
    let c1 = Rc::new(QCell::new(&owner, 100u32));
    let c2 = Rc::new(QCell::new(&owner, 200u32));
    let c1ref = owner.ro(&c1);
    drop(c1);    // Compile error
    println!("{}", *c1ref);
}
