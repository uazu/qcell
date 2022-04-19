extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::TLCell;
    struct Marker;
    fn is_sync<T: Sync>() {}
    is_sync::<TLCell<Marker, ()>>(); // Compile fail
}
