extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{QCell, QCellOwner};
    use std::rc::Rc;
    let mut owner = QCellOwner::new();
    let c1 = Rc::new(QCell::new(&owner, 100u32));
    let c2 = Rc::new(QCell::new(&owner, 200u32));
    fn test(o: &mut QCellOwner) {}
   
    let c1ref = owner.ro(&c1);
    test(&mut owner);    // Compile error
    println!("{}", *c1ref);
}
