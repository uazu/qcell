#[cfg(test)]
pub mod compiletest {
    #[rustversion::all(stable, since(1.60), before(1.61))]
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("src/compiletest/*.rs");
    }
}
