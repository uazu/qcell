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

    fn test(o: &mut ACellOwner) {}

    let c1ref = owner.ro(&c1);
    test(&mut owner);    // Compile error
    println!("{}", *c1ref);
}
