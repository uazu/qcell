extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner, generativity::make_guard};
    use std::rc::Rc;
    make_guard!(guard1);
    make_guard!(guard2);
    let mut owner1 = LCellOwner::new(guard1);
    let mut owner2 = LCellOwner::new(guard2);
    let c1 = Rc::new(owner1.cell(100u32));
    let c1mutref2 = owner2.rw(&c1);    // Compile error
    println!("{}", *c1mutref2);
}
