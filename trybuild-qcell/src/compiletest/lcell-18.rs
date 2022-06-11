extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    LCellOwner::scope(|owner| {
        let mut cell = LCell::new(100);
        let cell_ref = owner.ro(&cell);
        *cell.get_mut() = 50; // Compile fail
        assert_eq!(100, *cell_ref);
    });
}
