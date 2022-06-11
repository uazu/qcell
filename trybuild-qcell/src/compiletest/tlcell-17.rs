extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    struct Marker;
    type ACell<T> = TLCell<Marker, T>;
    type ACellOwner = TLCellOwner<Marker>;
    let owner = ACellOwner::new();
    let mut cell = ACell::new(100);
    let cell_ref = cell.get_mut();
    assert_eq!(100, *owner.ro(&cell)); // Compile fail
    *cell_ref = 50;
}
