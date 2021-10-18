extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::QCellOwnerPinned;
    fn is_unpin<T: Unpin>() {}
    is_unpin::<QCellOwnerPinned>();
}
