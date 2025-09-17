#[cfg(test)]
pub mod compiletest {
    #[rustversion::all(stable, since(1.89), before(1.90))]
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("src/compiletest/*.rs");
    }
}
