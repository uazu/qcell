extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner, generativity::make_guard};
    use std::rc::Rc;
    make_guard!(guard);
    let mut owner = LCellOwner::new(guard);
    let cell = LCell::new(Rc::new(100));
   
    // We aren't permitted to move the Rc to another thread
    crossbeam::scope(move |s| {
        s.spawn(move |_| assert_eq!(100, **owner.ro(&cell))).join().unwrap(); // Compile fail
    }).unwrap();
}
