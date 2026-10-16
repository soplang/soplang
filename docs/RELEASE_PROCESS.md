# Soplang release process

## Version policy

Releases remain on `3.x.y`; version 4 is neither approved nor scheduled. Package
versions come from Cargo.toml and must match a `release/v3.x.y` branch. A release
candidate uses a version such as `3.0.0-rc.1` in Cargo.toml and Cargo.lock and a
matching branch. Choose that version before creating the release branch.

## Verification before drafting

1. Review the [verification record](testing/RELEASE_VERIFICATION.md) and
   [release notes](RELEASE_NOTES_V3.md). Finish pending platform gates.
2. Run `cargo test --locked` and `cargo test --release --locked`, including AOT.
3. Build packages on their native systems:
   - Linux: `bash linux/build_linux.sh` (install dpkg-dev for a Debian package).
   - Windows: `./windows/build_windows.ps1` (Inno Setup is optional).
   - macOS: `bash macos/build_macos.sh` (create-dmg is optional).
4. Extract each archive/package and run `python3 scripts/verify_package.py
   /path/to/extracted/soplang` (use `python` / `soplang.exe` on Windows).
   Test the actual installer on a clean native machine too; archive checks do
   not establish installer PATH, file association, signing, or notarization.
5. Verify each generated `SHA256SUMS` beside its assets using `sha256sum -c`
   (Linux) or `shasum -a 256 -c` (macOS). Each platform checksum file lists only
   that platform's generated assets. Keep the platform directories separate.
6. Build and test Docker without publishing:

   ```sh
   docker build -t soplang:release-check .
   docker run --rm --entrypoint python3 \
     -v "$PWD/scripts:/verify/scripts:ro" \
     -v "$PWD/examples:/verify/examples:ro" \
     soplang:release-check /verify/scripts/verify_package.py /usr/local/bin/soplang
   ```

The `Verify release packages` workflow runs the same native and container gates
on pull requests and manual dispatch. Native results must be green; an uploaded
package alone is not evidence of a successful JIT or LLVM ABI.

## Draft and publication

After the maintainer commits and pushes a matching `release/v3.x.y` branch,
release automation first calls the full verification workflow. Only successful
Linux, Windows, macOS, and container jobs allow a draft. The draft contains the
reviewed release notes, real package assets, and their platform checksum files,
and targets the exact tested source revision. Automation does not rewrite the
changelog or add a commit. Update and commit the changelog during review.

Prerelease versions produce a prerelease draft. Review every download, signing
requirement, platform result, and note before publishing manually. A release
candidate is recommended before 3.0.0 final. The package version remains 3.0.0
until a maintainer chooses a candidate version; this document does not select
or publish a version.

Publishing triggers the existing Docker registry workflow. Stable releases also
trigger AUR publication. Prereleases must not replace stable `latest`, `3`, or
`3.0` Docker tags or update the stable AUR package. Verify registry images and
AUR installation after publication; the local image check cannot verify a
registry upload. No publication is performed by the local packaging scripts.

## Installed build requirements

See [Installing packages](INSTALLING_PACKAGES.md). Native builds require clang,
a system linker, and the exact Rust toolchain recorded inside the package.
The image includes these tools. Built programs retain ordinary system-library
requirements; build Linux assets against the oldest supported distribution and
test them there before advertising compatibility.
