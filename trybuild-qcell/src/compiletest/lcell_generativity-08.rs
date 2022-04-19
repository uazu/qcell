extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner, generativity::make_guard};
    use std::rc::Rc;
    make_guard!(guard);
    let mut owner = LCellOwner::new(guard);
    let c1 = Rc::new(LCell::new(100u32));
    fn test(o: &mut LCellOwner) {}
   
    let c1ref = owner.ro(&c1);
    test(&mut owner);    // Compile error
    println!("{}", *c1ref);
}
