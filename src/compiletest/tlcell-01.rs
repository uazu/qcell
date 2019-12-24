extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    use std::rc::Rc;
    struct Marker;
    type ACellOwner = TLCellOwner<Marker>;
    let mut owner1 = ACellOwner::new();
    let owner2 = owner1.clone();  // Compile fail
}
