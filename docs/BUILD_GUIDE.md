# Building Soplang 3

## Source build

Install Rust through rustup, Git, a system linker, and clang for native Soplang
builds. `rust-toolchain.toml` pins Rust 1.94.0 for the release. Windows also needs
Visual Studio C++ Build Tools and the Windows SDK. macOS needs Xcode Command
Line Tools. Linux needs its C development toolchain.

```sh
cargo build --release --locked
cargo test --locked
cargo test --release --locked
```

The executable is `target/release/soplang` (`soplang.exe` on Windows). Cargo
builds resolve their runtime artifacts beside that executable, including custom
`CARGO_TARGET_DIR` layouts. Installed native builds never run Cargo.

```sh
./target/release/soplang examples/01_hello.sop
./target/release/soplang --build examples/27_inheritance.sop -o program
./program
```

Run Soplang with no arguments for the REPL. See the [example catalog](../examples/README.md)
and [test methodology](testing/TEST_METHODOLOGY.md).

## Packages

Python 3.9+ is required for packaging. Each package contains the executable,
matching runtime/dependency rlibs, toolchain metadata, license, and installation
instructions. Keep the whole package together. Package builders use Cargo's
reported artifacts, including fresh dependencies, instead of globbing stale
libraries from a shared target directory.

| Platform | Command | Assets |
| --- | --- | --- |
| Linux | `bash linux/build_linux.sh` | Tar archive; Debian package when dpkg-deb is installed |
| Windows | `./windows/build_windows.ps1` or `windows\\build_windows.bat` | ZIP; installer when Inno Setup is installed |
| macOS | `bash macos/build_macos.sh` | App archive; DMG when create-dmg is installed |

Each builder writes a platform-local `SHA256SUMS`. See [package requirements](INSTALLING_PACKAGES.md)
for the matching Rust compiler needed by installed `--build`; JIT and REPL do
not require that compiler. See [release verification](testing/RELEASE_VERIFICATION.md)
for the completed v3.0.0 native/package gates and remaining distribution limits.
Every new release must repeat those checks.

## Docker

```sh
docker build -t soplang:release-check .
docker run --rm soplang:release-check --version
docker run --rm -v "$PWD/examples:/scripts:ro" soplang:release-check 01_hello.sop
```

The image includes Rust, clang, the system linker, and installed runtime
libraries. Mount a writable output directory when using `--build`; programs run
as the unprivileged `soplang` user. See [release checks](RELEASE_PROCESS.md) for
the complete image verifier.

## Performance measurements

See the [current Python/v2/v3 results](../benchmarks/RESULTS.md) and
[comparison setup](../benchmarks/comparison/README.md). Timings use release
compiler builds and validate program output. `cargo bench` runs the separate
internal Criterion suite.
