extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TCell, TCellOwner};
    use std::cell::Cell;
    struct Marker;
    type ACellOwner = TCellOwner<Marker>;
    type ACell = TCell<Marker, Cell<i32>>;
   
    let owner = ACellOwner::new();
    let cell = ACell::new(Cell::new(100));
   
    // This would be a data race if the compiler permitted it, but it doesn't
    std::thread::spawn(|| owner.ro(&cell).set(200));  // Compile fail
    owner.ro(&cell).set(300);
}
