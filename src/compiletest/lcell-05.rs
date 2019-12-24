extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner| {
        let c1 = Rc::new(LCell::new(100u32));
        let c2 = Rc::new(LCell::new(200u32));
        let c1ref = owner.ro(&c1);
        let c1mutref = owner.rw(&c1);    // Compile error
        println!("{}", *c1ref);
    });
}
