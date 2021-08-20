extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::cell::Cell;
    LCellOwner::scope(|owner| {
        let cell = LCell::new(Cell::new(100));
   
        // This would likely be a data race if it compiled
        crossbeam::scope(|s| {   // Compile fail
            let handle = s.spawn(|_| owner.ro(&cell).set(200));
            owner.ro(&cell).set(300);
            handle.join().unwrap();
        }).unwrap();
    });
}
