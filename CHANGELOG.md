# Significant feature changes and additions

This project follows Rust semantic versioning.

<!-- see keepachangelog.com for format ideas -->

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
