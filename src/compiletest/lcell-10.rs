extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCellOwner, LCell};
    LCellOwner::scope(|mut owner| {
        let cell = LCell::new(100);
        let val_ref = owner.ro(&cell);
        crossbeam::scope(move |s| {
            s.spawn(move |_| assert_eq!(*owner.ro(&cell), 100)).join().unwrap(); // Compile fail
        }).unwrap();
        assert_eq!(*val_ref, 100);
    });
}
