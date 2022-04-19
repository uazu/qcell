extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TCell, TCellOwner};
    use std::rc::Rc;
    struct Marker;
    type ACellOwner = TCellOwner<Marker>;
    let mut owner1 = ACellOwner::new();
    let owner2 = owner1.clone();  // Compile fail
}
