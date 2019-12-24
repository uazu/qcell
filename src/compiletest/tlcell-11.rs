extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::TLCellOwner;
    struct Marker;
    fn is_sync<T: Sync>() {}
    is_sync::<TLCellOwner<Marker>>();  // Compile fail
}
