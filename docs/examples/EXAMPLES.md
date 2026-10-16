# Soplang example programs

The runnable programs are `examples/*.sop` at the repository root. Descriptions and the current `.expected` checks are in [testing/EXAMPLES.md](../testing/EXAMPLES.md).

```bash
cargo build --release
./target/release/soplang examples/01_hello.sop
```

`make run FILE=examples/01_hello.sop` runs the release binary on that file. Build it first with `make build`.
