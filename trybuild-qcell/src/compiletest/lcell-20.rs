extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    LCellOwner::scope(|mut owner| {
        struct NoDefault(i32);
        let mut cell: LCell<NoDefault> = LCell::default(); // Compile fail
        assert_eq!(0, owner.ro(&cell).0);
    });
}
