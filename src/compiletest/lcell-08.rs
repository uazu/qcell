extern crate qcell;

#[allow(warnings)]
fn main() {
    use qcell::{LCell, LCellOwner};
    use std::rc::Rc;
    LCellOwner::scope(|mut owner| {
        let c1 = Rc::new(LCell::new(100u32));
        fn test(o: &mut LCellOwner) {}
   
        let c1ref = owner.ro(&c1);
        test(&mut owner);    // Compile error
        println!("{}", *c1ref);
    });
}
