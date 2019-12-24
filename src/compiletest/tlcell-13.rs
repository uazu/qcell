extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCellOwner, TLCell};
    struct Marker;
    type ACellOwner = TLCellOwner<Marker>;
    type ACell = TLCell<Marker, i32>;
    let owner = ACellOwner::new();
    let cell = ACell::new(100);
    let val_ref = owner.ro(&cell);
    std::thread::spawn(move || {
        assert_eq!(*owner.ro(&cell), 100);
    }).join();
    assert_eq!(*val_ref, 100);
}
