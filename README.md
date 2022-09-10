# Sirena

Digital audio signal processing primitives.

Documentation:

* [API reference (docs.rs)](https://docs.rs/sirena)
* [Repository (github.com)](https://github.com/zlosynth/sirena)
* [Crate (crates.io)](https://crates.io/crates/sirena)

The library is compatible with `#[no_std]` and targetted for embedded systems.

# Target optimizations

Some of the functions have multiple implementations with platform specific
optimizations. The target platform can be selected using features:

* Default: No optimizations.
* `cortexm7lfdp`: Cortex-M7, double-precission FPU.

# Development

See [DEVELOPMENT.md](DEVELOPMENT.md) to find some basic commands to interact
with the project.

# License

Software of Sirena is distributed under the terms of the General Public
License version 3. See [LICENSE](LICENSE) for details.

# Changelog

Read the [CHANGELOG.md](CHANGELOG.md) to learn about changes introduced in each
release.

# Versioning

The project adheres to [Semantic Versioning](https://semver.org/). Note that the
API is unstable and should be expected to change.
