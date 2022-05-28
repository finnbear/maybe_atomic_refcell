# maybe_atomic_refcell

[`AtomicRefCell`](https://github.com/bholley/atomic_refcell) for `debug` mode and [`UnsafeCell`](https://doc.rust-lang.org/stable/std/cell/struct.UnsafeCell.html) in `release` mode.

## Motivation

`AtomicRefCell` performs an atomic memory access at runtime to validate borrowing. While
this is an excellent way to validate code and ensure safety, it is an expensive
operation. This crate delegates to `AtomicRefCell` in `debug` mode and uses `UnsafeCell` to
emulate the same interface in `release` mode, minus the runtime overhead.

## Features

- `safe` enables unconditional runtime checks, good for validating in `release` mode

## Limitations

- No try-borrows, as they are impossible to (properly) implement without overhead
- `borrow` and `borrow_mut` are `unsafe` (despite being safe in `debug` mode)
- No `PartialEq`, `Eq`, `PartialOrd`, `Ord`, etc. due to the above

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.