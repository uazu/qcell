These `trybuild` compile-tests double-check that the `compile_fail`
tests in the doctests in the main crate actually fail for the reason
intended, not for some other reason.  This is most useful to check
when making changes to the crate that may have broken something.
However since the compiler error messages change from one Rust release
to the next, the test output only remains valid for a certain range of
compiler versions.

Procedure:

- Get a clean git status by checking in any in-progress changes
- Update the Rust version numbers in `src/lib.rs` to the current version
- Run `TRYBUILD=overwrite cargo test`
- Examine any modified files noticed by git

Any error output that has changed will show up as modified files under
`src/compiletest`.  Check through these manually to see that the
failure is the same as before.  Mostly the top line of the error
message will be the same and there will be changes in the formatting
or hints provided by the compiler.  If all is okay, check in the
changes.

The reason for having this in a separate crate is to run `trybuild` in
an environment with all `qcell` features enabled, since `trybuild`
seems to have problems running tests that depend on optional features.
