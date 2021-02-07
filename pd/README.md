# Sirena Pure Data

Sirena modules distributed as Pure Data external.

Build and install on Linux with:

```sh
cargo build --release

mkdir -p ~/.local/lib/pd/extra/sirena
cp ../target/release/libsirena_pd.so ~/.local/lib/pd/extra/sirena/sirena.pd_linux
```

