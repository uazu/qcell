extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner| {
        let c1 = Rc::new(LCell::new(100u32));
        let c2 = Rc::new(LCell::new(200u32));

        let c1mutref = owner.rw(&c1);
        let c2mutref = owner.rw(&c2);  // Compile error
        *c1mutref += 1;
        *c2mutref += 2;
    });
}
