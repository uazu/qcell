extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner1| {
        LCellOwner::scope(|mut owner2| {
            let c1 = Rc::new(LCell::new(100u32));
            let c1ref1 = owner1.ro(&c1);
            let c1ref2 = owner2.ro(&c1);   // Compile error
            println!("{}", *c1ref2);
        });
    });
}
