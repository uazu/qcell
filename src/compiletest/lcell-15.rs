extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|owner| {
        let cell = LCell::new(Rc::new(100));

        // We aren't permitted to move the Rc to another thread
        crossbeam::scope(move |s| {
            s.spawn(move |_| assert_eq!(100, **owner.ro(&cell))).join().unwrap(); // Compile fail
        }).unwrap();
    });
}
