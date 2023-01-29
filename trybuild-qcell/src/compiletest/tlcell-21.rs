extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    struct Marker;
    type ACell<T> = TLCell<Marker, T>;
    type ACellOwner = TLCellOwner<Marker>;
    struct NoDefault(i32);
    let mut owner = ACellOwner::new();
    let mut cell: ACell<NoDefault> = ACell::default(); // Compile fail
    assert_eq!(0, owner.ro(&cell).0);
}
