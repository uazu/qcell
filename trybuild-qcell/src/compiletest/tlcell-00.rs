extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    use std::rc::Rc;
    struct Marker;
    type ACell<T> = TLCell<Marker, T>;
    type ACellOwner = TLCellOwner<Marker>;
    let mut owner1 = ACellOwner::new();
    let mut owner2 = owner1;
    let rc = Rc::new(owner1.cell(100u32));  // Compile fail
}
