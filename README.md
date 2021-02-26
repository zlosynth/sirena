# Sirena

Audio signal processing primitives, instruments and Pure Data externals.

## Development

``` sh
# run formatting, linter and unit tests
make

# run slow tests too
make SLOW=1

# run benchmark
cargo bench

# profiling example
rm -f target/release/deps/bench-* callgrind.out.*
cargo bench --no-run
BENCH=$(find target/release/deps -type f -executable -name 'bench-*')
valgrind \
    --tool=callgrind \
    --dump-instr=yes \
    --collect-jumps=yes \
    --simulate-cache=yes \
    ${BENCH} --bench --profile-time 10
kcachegrind callgrind.out.*
```

