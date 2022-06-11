extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TCell, TCellOwner};
    struct Marker;
    type ACell<T> = TCell<Marker, T>;
    type ACellOwner = TCellOwner<Marker>;
    let mut owner = ACellOwner::new();
    let mut cell = ACell::new(100);
    let cell_ref = cell.get_mut();
    assert_eq!(100, *owner.rw(&cell)); // Compile fail
    *cell_ref = 50;
}
