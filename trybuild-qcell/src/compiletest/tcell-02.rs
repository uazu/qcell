extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TCell, TCellOwner};
    use std::rc::Rc;
    struct MarkerA;
    type ACellOwner = TCellOwner<MarkerA>;
    type ACell<T> = TCell<MarkerA, T>;
    struct MarkerB;
    type BCellOwner = TCellOwner<MarkerB>;
    type BCell<T> = TCell<MarkerB, T>;
   
    let mut owner_a = ACellOwner::new();
    let mut owner_b = BCellOwner::new();
    let c1 = Rc::new(ACell::new(100u32));
   
    let c1ref = owner_b.ro(&*c1);   // Compile error
    println!("{}", *c1ref);
}
