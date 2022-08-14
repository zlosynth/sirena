# Development

This project utilizes [cargo make](https://github.com/sagiegurari/cargo-make).
Start by installing it:

```sh
cargo install --force cargo-make
```

## Formatting, linting, unit tests

Run formatting, linter and unit tests:

```sh
cargo make dev

# run slow tests too
cargo make dev-slow
```

Read the Makefile.toml to learn more.

## Benchmark

If a package has a benchmark defined, `cd` into its directory and run it via:

``` sh
cargo bench --bench bench
```

Use the benchmark for profiling:

``` sh
rm -f target/release/deps/bench-*
rm -f callgrind.out.*
RUSTFLAGS="-g" cargo bench --bench bench --no-run
BENCH=$(find target/release/deps -type f -executable -name 'bench-*')
TEST=log_taper
valgrind \
    --tool=callgrind \
    --dump-instr=yes \
    --collect-jumps=yes \
    --simulate-cache=yes \
    ${BENCH} --bench --profile-time 10 ${TEST}
kcachegrind callgrind.out.*
```
