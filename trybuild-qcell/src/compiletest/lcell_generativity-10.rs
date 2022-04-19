extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCellOwner, LCell, generativity::make_guard};
    make_guard!(guard);
    let mut owner = LCellOwner::new(guard);
    let cell = LCell::new(100);
    let val_ref = owner.ro(&cell);
    crossbeam::scope(move |s| {
        s.spawn(move |_| assert_eq!(*owner.ro(&cell), 100)).join().unwrap(); // Compile fail
    }).unwrap();
    assert_eq!(*val_ref, 100);
}
