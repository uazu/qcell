extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    use std::rc::Rc;
    let mut owner = QCellOwner::new();
    let c1 = Rc::new(QCell::new(&owner, 100u32));
    let c2 = Rc::new(QCell::new(&owner, 200u32));
    fn test(o: &QCellOwner) {}
   
    let c1mutref = owner.rw(&c1);
    test(&owner);    // Compile error
    *c1mutref += 1;
}
