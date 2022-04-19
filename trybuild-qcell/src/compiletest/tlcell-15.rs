extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    use std::rc::Rc;
    struct Marker;
    type ACellOwner = TLCellOwner<Marker>;
    type ACell = TLCell<Marker, Rc<i32>>;
   
    let owner = ACellOwner::new();
    let cell = ACell::new(Rc::new(100));
   
    // We aren't permitted to move the Rc to another thread
    std::thread::spawn(move || {    // Compile fail
        assert_eq!(100, **owner.ro(&cell));
    }).join();
}
