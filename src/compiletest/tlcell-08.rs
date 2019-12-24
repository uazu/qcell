extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{TLCell, TLCellOwner};
    use std::rc::Rc;
    struct Marker;
    type ACellOwner = TLCellOwner<Marker>;
    type ACell<T> = TLCell<Marker, T>;
    let mut owner = ACellOwner::new();
    let c1 = Rc::new(ACell::new(100u32));
    let c2 = Rc::new(ACell::new(200u32));

    fn test(o: &mut ACellOwner) {}

    let c1ref = owner.ro(&c1);
    test(&mut owner);    // Compile error
    println!("{}", *c1ref);
}
