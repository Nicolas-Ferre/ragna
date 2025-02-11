# Ragna

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Nicolas-Ferre/ragna#license)
[![CI](https://github.com/Nicolas-Ferre/ragna/actions/workflows/ci.yml/badge.svg)](https://github.com/Nicolas-Ferre/ragna/actions/workflows/ci.yml)
[![Coverage with grcov](https://img.shields.io/codecov/c/gh/Nicolas-Ferre/ragna)](https://app.codecov.io/gh/Nicolas-Ferre/ragna)

Ragna is a library for easily creating GPU-native applications, where most of the logic is run
directly on GPU side.

It is particularly well suited for graphics applications such as games.

## ⚠️ Warning ⚠️

Before you consider using this library, please keep in mind that:

- It is developed by a single person in his spare time.
- The library is highly experimental, so it shouldn't be used for production applications.

## Main characteristics

- 🔥 Maximize execution on GPU side
- 💪 Strong typing and compile-time checks performed with Rust
- 🔀 Data race free
- 🔄 Hot reloadable GPU code

## Supported platforms

- Windows
- Linux
- macOS (limited support because the maintainer doesn't have access to a physical device)
- Android
- Web

Ragna may also work on some other platforms, but they have not been tested.

## Getting started

You can use the following command to run examples located in the `examples` folder:

`cargo run --example buffers --release`

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or
conditions.
