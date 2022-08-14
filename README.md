# Sirena

Digital audio signal processing primitives.

Components of the project include:

- Signal abstraction used to abstract samples into infinite-iterator-like
  objects. It also includes basic signal generators and trait allowing
  implementation of custom signal processors.
- Ring buffer of arbitrary size, allowing to read discrete or interpolated
  samples.
- Spectral analyzer, using FFT to measure harmonic spectrum of given signal.
  Mostly meant for testing.
- State variable filter, can be used for low/high/band pass or band reject.

# Razor

* `#[no_std]`, targetted for embedded systems.

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
