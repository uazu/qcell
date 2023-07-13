# Significant feature changes and additions

This project follows Rust semantic versioning.

<!-- see keepachangelog.com for format ideas -->

## 0.5.4 (2023-07-13)

### Added

- `no_std` support for `TCell`, contributed by [Violet
  Leonard](https://github.com/geeklint).
- `Default` for `TCell`, `TLCell` and `LCell`
- Formalize MSRV of 1.60, and test


## 0.5.3 (2023-01-15)

### Added

- `Hash`, `Eq` and `PartialEq` implementations for QCellOwnerID.
  These changes were contributed by [Simon
  Ask](https://github.com/simonask) and
  [LegionMammal978](https://github.com/LegionMammal978)


## 0.5.2 (2022-06-11)

### Added

- `#[repr(transparent)]` added to `LCell`, `TCell` and `TLCell`
- `get_mut` and `into_inner` added to all cell types which better
  supports beginning/end-of-life of the cell

[Troy Hinckley](https://github.com/CeleritasCelery) contributed to
this point release.


## 0.5.1 (2022-04-19)

### Added

- `LCellOwner` can now be created from a
  [**generativity**](https://crates.io/crates/generativity) guard,
  which makes `LCellOwner` creation more convenient.  This change was
  contributed by [Troy Hinckley](https://github.com/CeleritasCelery).


## 0.5.0 (2022-01-23)

### Changed

- `QCellOwner` is now based on IDs derived from the addresses of
  heap-allocated objects.  This change was contributed by [Violet
  Leonard](https://github.com/geeklint).  This offloads maintaining
  lists of in-use IDs to the allocator, which improves `no_std`
  compatibility.

### Added

- `no_std` support in `QCell` and `LCell`, contributed by [Violet
  Leonard](https://github.com/geeklint).

- `QCellOwnerPinned` (contributed by [Violet
  Leonard](https://github.com/geeklint)), and `QCellOwnerSeq`.  These
  now form a family of ID-based owners along with `QCellOwner`.

### Breaking

- `QCellOwner::fast_new` is replaced by `QCellOwnerSeq::new`


## 0.4.3 (2021-09-20)

### Fixed

- It is now no longer possible for a malicious coder to cheat the
  singleton check in `TCellOwner` and `TLCellOwner` by using covariant
  subtypes.  Thanks to [Frank Steffahn](https://github.com/steffahn)
  for noticing the bug, and for providing the PR to fix the issue.

### Testing

- Check common traits using `static_assertions`


## 0.4.2 (2021-08-20)

### Added

- `?Sized` / unsized types / DST support, e.g. `QCell<dyn Trait>`
- `cell.rw(owner)` and `cell.ro(owner)`
- `TCellOwner::try_new` and `TCellOwner::wait_for_new`, contributed by
  [Michiel De Muycnk](https://github.com/Migi)


## 0.4.1 (2020-06-25)

(internal changes)


## 0.4.0 (2019-12-24)

### Added

- `Send` and `Sync` support, with full reasoning, thanks to a
  contribution from [Michiel de Muycnk](https://github.com/Migi)

### Breaking

- `TCell` and `TLCell` are now split into separate types and the
  previous 'no-thread-local' feature has been removed

### Testing

- `trybuild` testing of all compile_fail tests, to ensure that they're
  failing for the right reasons


## 0.3.0 (2019-07-24)

### Breaking

- `TCell` is now thread-local-based by default, with a cargo feature
  to switch on old global behaviour


## 0.2.0 (2019-03-20)

### Breaking

- Switch from `get()` and `get_mut()` to `ro()` and `rw()` to access
  cell contents

- No longer require the owner to be specified when creating `TCell`
  and `LCell` instances.

### Added

- `QCellOwnerID` for creating cells when the owner is not available


## 0.1.2 (2019-03-18)

### Added

- `LCell` lifetime-based cell


## 0.1.1 (2019-03-07)

### Fixed

- Make `QCellOwner::new()` completely safe.  Old code that could be
  used maliciously to create undefined behaviour is moved to
  `QCellOwner::fast_new()` and marked as `unsafe`.


## 0.1.0 (2019-02-28)

(first public release)


<!-- Local Variables: -->
<!-- mode: markdown -->
<!-- End: -->
