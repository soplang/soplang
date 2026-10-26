# Installing Soplang 3.6.0

[Soplang 3.6.0](https://github.com/soplang/soplang/releases/tag/v3.6.0) provides
packages for Linux x86_64, Windows x64, and macOS Apple Silicon. A Linux amd64
Docker image is also available.

## Release packages

Download your platform's package and the release's combined `SHA256SUMS`.

| Platform | Package | Requirements |
| --- | --- | --- |
| Linux x86_64 | `soplang_3.6.0_amd64.deb` or `soplang-3.6.0-linux-x86_64.tar.gz` | glibc 2.39 or later; libgcc |
| Windows x64 | `soplang-setup.exe` or `soplang-3.6.0-windows-AMD64.zip` | Windows 10 or later on x64 |
| macOS Apple Silicon | `soplang-3.6.0-macos-arm64.tar.gz` | macOS 15.7.9 or later |

The Windows installer is unsigned and the macOS archive is not notarized.
The release includes archives and a Debian package. The macOS builder may
produce a DMG when `create-dmg` is available.

### Verify a download

Keep `SHA256SUMS` beside your downloads. On Linux or macOS, check the selected
file against its entry, or check the full manifest after downloading all assets:

```sh
sha256sum -c SHA256SUMS   # Linux
shasum -a 256 -c SHA256SUMS   # macOS
```

On Windows, compare `Get-FileHash <download> -Algorithm SHA256` with the matching
line in `SHA256SUMS`.

## Linux

Install the Debian package:

```sh
sudo dpkg -i soplang_3.6.0_amd64.deb
soplang nooc
```

Or extract the archive:

```sh
tar -xzf soplang-3.6.0-linux-x86_64.tar.gz
./soplang/soplang nooc
```

Keep the archive's executable and `lib/soplang` directory together. Debian
installs the executable at `/usr/bin/soplang` and its runtime files at
`/usr/lib/soplang`.

## Windows

Run `soplang-setup.exe`. Its optional tasks add Soplang to PATH and associate
`.sop` and `.so` source files. Open a new terminal after installation:

```powershell
soplang nooc
soplang -c 'qor("Salaan!")'
```

Alternatively, extract the ZIP and run `soplang.exe` inside its `soplang`
directory. Keep the `lib` directory beside the executable.

## macOS

Extract the Apple Silicon archive:

```sh
tar -xzf soplang-3.6.0-macos-arm64.tar.gz
./Soplang.app/Contents/MacOS/soplang nooc
```

This is a command-line application. Keep the complete app bundle and add its
`Contents/MacOS` directory to PATH if desired. Intel Mac packages are not part
of this release; follow the [source build guide](BUILD_GUIDE.md) for other
targets.

## Docker

```sh
docker run --platform linux/amd64 --rm soplang/soplang:3.6.0 nooc
docker run --platform linux/amd64 --rm -v "$PWD:/scripts:ro" soplang/soplang:3.6.0 program.sop
docker run --platform linux/amd64 --rm -it soplang/soplang:3.6.0 isdhexgal
```

See [Docker usage](docker.md) for writable mounts and building executables.

## Run and build programs

```sh
soplang fuli program.sop
soplang -c 'qor("Salaan!")'
soplang isdhexgal
soplang dhis program.sop
soplang build program.sop -o program
```

The 3.6 packages include the LLVM and linker components needed for native
builds. Running Soplang and building a program need no separate Rust, Cargo,
Clang, or system linker installation. A built program runs without Soplang or
its source files beside it. It targets the operating system and architecture
of the package that built it. See [installed package details](INSTALLING_PACKAGES.md).

To build the compiler itself, follow the [source build guide](BUILD_GUIDE.md).
For language examples and diagnostics, see the [example catalog](../examples/README.md)
and [keyword reference](language/keywords.md).
