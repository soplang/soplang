# Installing Soplang 3

[Soplang 3.0.0](https://github.com/soplang/soplang/releases/tag/v3.0.0) is available
for Linux x86_64, Windows x64, and macOS Apple Silicon. A Linux amd64 Docker
image includes the compiler and its native build toolchain.

## Release packages

Download the package for your platform and the release's combined `SHA256SUMS`.

| Platform | Package | Requirements |
| --- | --- | --- |
| Linux x86_64 | `soplang_3.0.0_amd64.deb` or `soplang-3.0.0-linux-x86_64.tar.gz` | glibc 2.39 or later; libgcc |
| Windows x64 | `soplang-setup.exe` or `soplang-3.0.0-windows-AMD64.zip` | Native Windows x64 environment |
| macOS Apple Silicon | `soplang-3.0.0-macos-arm64.tar.gz` | The app bundle declares macOS 14.8.9 or later |

The Windows installer is unsigned; the macOS archive is not notarized.
This release provides archives and a Debian package; RPM and DMG assets are
not attached to v3.0.0. The macOS builder can optionally generate a DMG locally.

All native platforms passed the [release verification gates](testing/RELEASE_VERIFICATION.md).

### Verify downloads

Keep `SHA256SUMS` next to your downloads. On Linux/macOS, check a selected
file against its entry, or use the full manifest after downloading all assets:

```sh
# Linux, when all listed assets are present:
sha256sum -c SHA256SUMS

# macOS, when all listed assets are present:
shasum -a 256 -c SHA256SUMS
```

On Windows, compare `Get-FileHash <download> -Algorithm SHA256` with the matching
line in `SHA256SUMS`. The manifest covers packages for all platforms.

## Linux

Install the Debian package:

```sh
sudo dpkg -i soplang_3.0.0_amd64.deb
soplang --version
```

Or extract the archive:

```sh
tar -xzf soplang-3.0.0-linux-x86_64.tar.gz
./soplang/soplang --version
```

Keep the archive's executable and `lib/soplang` together. Add the extracted
`soplang` directory to PATH if desired. Debian installs the executable at
`/usr/bin/soplang` and runtime libraries at `/usr/lib/soplang`.

## Windows

Run `soplang-setup.exe`. Its optional tasks add Soplang to PATH and associate
`.sop` and `.so` source files. Open a new terminal after installation:

```powershell
soplang --version
soplang -c 'qor("Salaan!")'
```

Alternatively, extract the ZIP and run `soplang.exe` inside the extracted
`soplang` directory. Keep its `lib` directory beside the executable.

## macOS

Extract the Apple Silicon archive:

```sh
tar -xzf soplang-3.0.0-macos-arm64.tar.gz
./Soplang.app/Contents/MacOS/soplang --version
```

This is a command-line application. Keep the complete app bundle and add its
`Contents/MacOS` directory to PATH if desired. Intel Mac packages are not part
of this release; [build from source](BUILD_GUIDE.md) for other targets.

## Docker

The public image is available from Docker Hub:

```sh
docker run --rm soplang/soplang:3.0.0 --version
docker run --rm -v "$PWD:/scripts:ro" soplang/soplang:3.0.0 program.sop
docker run -it --rm soplang/soplang:3.0.0
```

See [Docker usage](docker.md) for native builds, writable mounts, and registry
availability.

## Run programs and build executables

```sh
soplang program.sop
soplang -c 'qor("Salaan!")'
soplang
soplang --build program.sop -o program
```

JIT execution and the REPL do not need Rust or clang. Native `--build` requires
clang, a platform linker/SDK, and the exact Rust toolchain recorded in the
package. Official v3.0.0 packages use Rust 1.94.0. See
[installed build requirements](INSTALLING_PACKAGES.md) before building programs.
The Docker image includes these tools.

To build the compiler itself, follow the [source build guide](BUILD_GUIDE.md).
For language examples and source diagnostics, start with the
[example catalog](../examples/README.md) and [language reference](language/keywords.md).
