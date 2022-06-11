extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    LCellOwner::scope(|owner| {
        let mut cell = LCell::new(100);
        let cell_ref = cell.get_mut();
        assert_eq!(100, *owner.ro(&cell)); // Compile fail
        *cell_ref = 50;
    });
}
