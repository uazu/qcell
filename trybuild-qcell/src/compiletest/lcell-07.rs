extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner| {
        let c1 = Rc::new(LCell::new(100u32));
        let c1ref = owner.ro(&c1);
        drop(c1);    // Compile error
        println!("{}", *c1ref);
    });
}
