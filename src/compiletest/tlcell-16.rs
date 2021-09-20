extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    type MarkerA = fn(&());
    type MarkerB = fn(&'static ());

    let mut owner1 = TLCellOwner::<MarkerA>::new() as TLCellOwner<MarkerB>;  // Compile fail
    let mut owner2 = TLCellOwner::<MarkerB>::new();
    let cell = TLCell::<MarkerB, u32>::new(1234);
    let ref1 = owner1.rw(&cell);
    let ref2 = owner2.rw(&cell);
    *ref1 = 1;  // Two mutable refs at the same time!  Unsound!
    *ref2 = 2;
}
