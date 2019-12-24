extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TCell, TCellOwner};
    use std::rc::Rc;
    struct Marker;
    type ACellOwner = TCellOwner<Marker>;
    type ACell<T> = TCell<Marker, T>;
    let mut owner = ACellOwner::new();
    let c1 = Rc::new(ACell::new(100u32));
    let c2 = Rc::new(ACell::new(200u32));

    let c1ref = owner.ro(&c1);
    let c1mutref = owner.rw(&c1);    // Compile error
    println!("{}", *c1ref);
}
