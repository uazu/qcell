extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner1| {
        LCellOwner::scope(|mut owner2| {
            let c1 = Rc::new(owner1.cell(100u32));
            let c1mutref2 = owner2.rw(&c1);    // Compile error
            println!("{}", *c1mutref2);
        });
    });
}
