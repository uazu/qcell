extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TCell, TCellOwner};
    use std::rc::Rc;
    struct Marker;
    type ACellOwner = TCellOwner<Marker>;
    type ACell = TCell<Marker, Rc<i32>>;
   
    let owner = ACellOwner::new();
    let cell = ACell::new(Rc::new(100));
   
    // We aren't permitted to move the Rc to another thread
    std::thread::spawn(move || {    // Compile fail
        assert_eq!(100, **owner.ro(&cell));
    }).join();
}
