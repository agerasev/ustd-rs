# ustd-rs

[![Crates.io][crates_badge]][crates]
[![Docs.rs][docs_badge]][docs]
[![Github Actions][github_badge]][github]
[![License][license_badge]][license]

[crates_badge]: https://img.shields.io/crates/v/ustd.svg
[docs_badge]: https://docs.rs/ustd/badge.svg
[github_badge]: https://github.com/agerasev/ustd-rs/actions/workflows/test.yml/badge.svg
[license_badge]: https://img.shields.io/crates/l/ustd.svg

[crates]: https://crates.io/crates/ustd
[docs]: https://docs.rs/ustd
[github]: https://github.com/agerasev/ustd-rs/actions/workflows/test.yml
[license]: #license

Micro-stdlib for Rust (primarily for embedded purposes).

## Backends

+ `std` (uses Rust stdlib, for testing)
+ `freertos` (uses [`freertos-rust`](https://github.com/lobaro/FreeRTOS-rust) crate)

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
