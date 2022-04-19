extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TCellOwner, TCell};
    struct Marker;
    type ACellOwner = TCellOwner<Marker>;
    type ACell = TCell<Marker, i32>;
    let owner = ACellOwner::new();
    let cell = ACell::new(100);
    let val_ref = owner.ro(&cell);
    std::thread::spawn(move || {
        assert_eq!(*owner.ro(&cell), 100);
    }).join();
    assert_eq!(*val_ref, 100);
}
