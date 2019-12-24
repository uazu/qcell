extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner1| {
        let owner2 = owner1;
        let rc = Rc::new(owner1.cell(100u32)); // Compile fail
    });
}
