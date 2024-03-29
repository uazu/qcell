# Statically-checked alternatives to RefCell or RwLock

Cell types that instead of panicking at runtime as with `RefCell` will
give compilation errors instead, or that exchange fine-grained locking
with `RwLock` for coarser-grained locking of a separate owner object.

### Documentation

See the [crate documentation](http://docs.rs/qcell).

# License

This project is licensed under either the Apache License version 2 or
the MIT license, at your option.  (See
[LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT)).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
