extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TCell, TCellOwner};
    struct Marker;
    type ACell<T> = TCell<Marker, T>;
    type ACellOwner = TCellOwner<Marker>;
    let mut owner = ACellOwner::new();
    let mut cell = ACell::new(100);
    let cell_ref = owner.rw(&cell);
    *cell.get_mut() = 50; // Compile fail
    assert_eq!(100, *cell_ref);
}
