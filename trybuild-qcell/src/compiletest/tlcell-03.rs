extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    use std::rc::Rc;
    struct MarkerA;
    type ACellOwner = TLCellOwner<MarkerA>;
    type ACell<T> = TLCell<MarkerA, T>;
    struct MarkerB;
    type BCellOwner = TLCellOwner<MarkerB>;
    type BCell<T> = TLCell<MarkerB, T>;
    let mut owner_a = ACellOwner::new();
    let mut owner_b = BCellOwner::new();
    let c1 = Rc::new(ACell::new(100u32));
   
    let c1mutref = owner_b.rw(&*c1);    // Compile error
    println!("{}", *c1mutref);
}
