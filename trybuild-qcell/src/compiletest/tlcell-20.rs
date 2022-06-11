extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    struct Marker;
    type ACell<T> = TLCell<Marker, T>;
    type ACellOwner = TLCellOwner<Marker>;
    let mut owner = ACellOwner::new();
    let mut cell = ACell::new(100);
    let cell_ref = owner.rw(&cell);
    *cell.get_mut() = 50; // Compile fail
    assert_eq!(100, *cell_ref);
}
