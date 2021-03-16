# Sirena

Audio signal processing primitives, instruments and Pure Data externals.

The API is experimental and should not be expected to be stable.

## Development

``` sh
# run formatting, linter and unit tests
make

# run slow tests too
make SLOW=1

# run benchmark
cargo bench --bench bench

# profiling example
rm -f target/release/deps/bench-*
rm -f callgrind.out.*
RUSTFLAGS="-g" cargo bench --bench bench --no-run
BENCH=$(find target/release/deps -type f -executable -name 'bench-*')
TEST=osc2
valgrind \
    --tool=callgrind \
    --dump-instr=yes \
    --collect-jumps=yes \
    --simulate-cache=yes \
    ${BENCH} --bench --profile-time 10 ${TEST}
kcachegrind callgrind.out.*
```
