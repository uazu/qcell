extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner| {
        let c1 = Rc::new(LCell::new(100u32));
        fn test(o: &LCellOwner) {}

        let c1mutref = owner.rw(&c1);
        test(&owner);    // Compile error
        *c1mutref += 1;
    });
}
