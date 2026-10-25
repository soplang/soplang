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

Run Soplang with no arguments in a terminal for the command browser. Use
`soplang repl` for the REPL. See the [example catalog](../examples/README.md)
and [test methodology](testing/TEST_METHODOLOGY.md).

## Packages

Python 3.9+ is required for packaging. Native packages include embedded LLVM,
a linker, the Soplang runtime archive, licenses, and installation instructions.
Keep the whole package together. Linux packaging needs LLVM and LLD development
libraries, a C++ compiler, and `patchelf`. macOS packaging needs `llvm@22` and
`lld@22` from Homebrew. Windows packaging needs the LLVM SDK, Zig, and Visual
Studio C++ Build Tools; set `SOPLANG_LLVM_SDK` and `SOPLANG_ZIG_SDK` to their
extracted directories before running the package builder.

| Platform | Command | Assets |
| --- | --- | --- |
| Linux | `bash linux/build_linux.sh` | Tar archive; Debian package when dpkg-deb is installed |
| Windows | `./windows/build_windows.ps1` or `windows\\build_windows.bat` | ZIP; installer when Inno Setup is installed |
| macOS | `bash macos/build_macos.sh` | App archive; DMG when create-dmg is installed |

Each builder writes a platform-local `SHA256SUMS`. See
[package requirements](INSTALLING_PACKAGES.md) for installed build behavior and
[release verification](testing/RELEASE_VERIFICATION.md) for the platform gates.
Every new release must repeat those checks.

## Docker

```sh
docker build -t soplang:release-check .
docker run --rm soplang:release-check --version
docker run --rm -v "$PWD/examples:/scripts:ro" soplang:release-check 01_hello.sop
```

The image includes the installed native package and needs no Rust or Clang to
build Soplang programs. Mount a writable output directory when using `--build`;
programs run as the unprivileged `soplang` user. See
[release checks](RELEASE_PROCESS.md) for the complete image verifier.

## Performance measurements

See the [current Python/v2/v3 results](../benchmarks/RESULTS.md) and
[comparison setup](../benchmarks/comparison/README.md). Timings use release
compiler builds and validate program output. `cargo bench` runs the separate
internal Criterion suite.
