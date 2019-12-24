extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner1| {
        let owner2 = owner1.clone(); // Compile fail
    });
}
