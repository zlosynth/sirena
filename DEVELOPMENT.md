# Development

## Formatting, linting, unit tests

Run formatting, linter and unit tests:

```sh
make

# run slow tests too
make SLOW=1
```

Read the Makefile to learn more.

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
