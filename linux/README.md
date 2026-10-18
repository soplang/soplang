# Soplang 3 for Linux

The [v3.0.1 release](https://github.com/soplang/soplang/releases/tag/v3.0.1)
provides x86_64 archives and Debian packages requiring glibc 2.39 or later.
Native debug/release suites, package checks, and Debian install/removal are
covered by the [release gates](../docs/testing/LOCALIZATION_3_0_1.md). For download and
installation commands, see [Installing Soplang](../docs/installation.md).

## Build packages locally

Install Rust via rustup, Python 3.9+, and a C development toolchain. Install
clang for Soplang native builds and dpkg-dev for Debian packaging.

```sh
bash linux/build_linux.sh
```

The locked builder stages the executable and matching runtime libraries in
`dist/soplang`, creates `linux/soplang-<version>-linux-<arch>.tar.gz`, and creates
`linux/soplang_<version>_<deb-arch>.deb` when dpkg-deb is available. Checksums are
in `linux/SHA256SUMS`. Debian versions map prerelease `-` to `~` so a
candidate sorts below the corresponding final release.

Extract the archive and keep its directory intact, or install the Debian
package. Debian installs the executable at `/usr/bin/soplang` and runtime at
`/usr/lib/soplang`. Verify an extracted package before publishing:

```sh
python3 scripts/verify_package.py /path/to/extracted/soplang/soplang
(cd linux && sha256sum -c SHA256SUMS)
```

[Installed native-build requirements](../docs/INSTALLING_PACKAGES.md) include
clang and the exact Rust compiler recorded inside the package. Build against
the oldest supported distribution and test there before announcing a minimum
Linux version; a successful build on a newer host does not establish it.
