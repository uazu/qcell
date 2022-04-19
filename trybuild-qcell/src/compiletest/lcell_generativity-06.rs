extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner, generativity::make_guard};
    use std::rc::Rc;
    make_guard!(guard);
    let mut owner = LCellOwner::new(guard);
    let c1 = Rc::new(LCell::new(100u32));
    let c2 = Rc::new(LCell::new(200u32));
    let c1mutref = owner.rw(&c1);
    let c2ref = owner.ro(&c2);    // Compile error
    *c1mutref += 1;
}
