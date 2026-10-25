# Installing Soplang packages

Keep the complete package together. The `soplang` executable discovers its
runtime at `lib/soplang` beside itself, or at `../lib/soplang` when installed in
a Unix `bin` directory. Moving only the executable breaks native builds.
`SOPLANG_RUNTIME_DIR` can select another runtime directory; an invalid value is
an error.

## Soplang 3.6 native builds

The 3.6 packages include LLVM object generation, a linker, and the Soplang
runtime archive. An installed package can build a program without Rust, Cargo,
Clang, or an external linker on `PATH`:

```sh
soplang dhis program.sop
# English command and flag forms also work:
soplang build program.sop
soplang --build program.sop -o program
```

The output is a standalone executable for the same operating system and
architecture as the compiler package. It does not need Soplang or the source
files to run, but it uses ordinary system libraries on its target platform.
Database servers, network services, and certificates remain external resources
for programs that use them.

The Linux x86_64 package includes LLVM, LLD, a startup object, and its required
shared libraries. The macOS Apple Silicon package includes LLVM, LLD, and link
stubs for system frameworks. The Windows x64 package includes LLVM and Zig's
MinGW runtime and linker. Build the compiler itself from source using Rust and
the native dependencies in the [build guide](BUILD_GUIDE.md).

## Run and inspect an installed package

Add the directory containing `soplang` (`soplang.exe` on Windows) to `PATH`, or
use its full path. Try `soplang nooc`, `soplang program.sop`, or `soplang` for
the interactive command browser. Use `soplang isdhexgal` for the REPL.

Verify an extracted package from a source checkout with Python 3:

```sh
python3 scripts/verify_package.py /absolute/path/to/soplang
```

The verifier relocates the executable and runtime to a path with spaces. It
runs the public examples through JIT and LLVM, checks diagnostic snapshots,
tests imports and the REPL, and builds with no compiler tools on `PATH`. It
also checks that the built executable runs after the source and Soplang runtime
are removed.

## Earlier 3.0.1 packages

Published v3.0.1 packages use the earlier Rust and Clang build path. Their
installed `--build` needs Clang 15 or newer, a platform linker/SDK, and the
exact Rust compiler recorded in `lib/soplang/rustc-version.txt`. JIT execution
and the REPL do not need those tools. See the
[v3.0.1 installation guide](https://github.com/soplang/soplang/blob/v3.0.1/docs/INSTALLING_PACKAGES.md)
when using an earlier package.

Set `SOPLANG_ERROR_DETAILS=1` to include original tool and operating-system
details with Somali diagnostics when troubleshooting a build.
