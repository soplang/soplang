# Installing Soplang 3 packages

Keep the whole extracted package together. The executable discovers its runtime
at `lib/soplang` beside itself, or at `../lib/soplang` for a Unix `bin` directory.
Moving only the executable supports JIT execution but cannot support `--build`.
`SOPLANG_RUNTIME_DIR` can override the runtime location; an invalid override is
an error and never silently selects another installation.

## Running programs

Add the directory containing `soplang` (`soplang.exe` on Windows) to PATH, or use
its full path. Run `soplang --version`, `soplang program.sop`, or `soplang` for the
interactive shell. These operations do not need Rust, Cargo, or clang.

## Building standalone programs

`--build` requires clang 15 or newer (with opaque pointer support) and the **exact Rust compiler used to build the package**,
including its host target. Read `lib/soplang/rustc-version.txt` for the compiler
version, commit, and target. For official packages the toolchain is Rust 1.94.0;
install it through rustup, then select it in the shell where you run Soplang:

```sh
rustup toolchain install 1.94.0 --profile minimal
rustup default 1.94.0
soplang --build program.sop -o program
```

A directory-specific `rustup override set 1.94.0` or `RUSTUP_TOOLCHAIN=1.94.0`
can be used instead of changing your default. Rebuild a source package with
`scripts/package.py` if you need another compiler. Cargo is needed to build a
package from source; installed `--build` never invokes Cargo.

- Linux: install clang and a C linker / development toolchain (for example,
  `clang-16 build-essential`; ensure `clang` selects that version on Debian/Ubuntu).
- macOS: install Xcode Command Line Tools, including Apple clang and the linker.
- Windows: install LLVM clang and Visual Studio C++ Build Tools / Windows SDK;
  use a developer shell with the linker on PATH. Native Windows validation is a
  release gate; packaging alone does not establish ABI support.

The Docker image includes the matching Rust compiler, clang, and runtime files.
A built executable needs neither Soplang nor the original source files, but
retains its target platform's ordinary system-library requirements. It is not a
cross-platform executable.

## Verifying an extracted package

From the source checkout, with Python 3 and the build toolchain available:

```sh
python3 scripts/verify_package.py /absolute/path/to/soplang
```

The checker copies the executable and runtime into a temporary directory with
spaces, changes away from the checkout, runs all public examples through JIT
and LLVM, checks every diagnostic snapshot, and exercises imports and the REPL.
It also checks missing runtime files, mismatched Rust metadata, and standalone
execution after removing the source and installed runtime.
