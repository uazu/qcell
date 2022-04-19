extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::TLCellOwner;
    struct Marker;
    fn is_send<T: Send>() {}
    is_send::<TLCellOwner<Marker>>();  // Compile fail
}
