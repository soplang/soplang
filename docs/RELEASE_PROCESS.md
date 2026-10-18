# Soplang release process

## Version policy

Releases remain on `3.x.y`; version 4 is neither approved nor scheduled. Package
versions come from Cargo.toml and must match a `release/v3.x.y` branch. A release
candidate uses a version such as `3.0.0-rc.1` in Cargo.toml and Cargo.lock and a
matching branch. Choose that version before creating the release branch.

## Verification before drafting

1. Review the [verification record](testing/RELEASE_VERIFICATION.md) and
   [version-specific release notes](releases/3.0.1.md). Finish pending platform gates.
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
reviewed release notes, real package assets, and one combined `SHA256SUMS`,
and targets the exact tested source revision. Automation does not rewrite the
changelog or add a commit. Update and commit the changelog during review.

Prerelease versions produce a prerelease draft. Review every download, signing
requirement, platform result, and note before publishing manually. Keep the package version in Cargo.toml and Cargo.lock synchronized with the
release branch. Future 3.x releases can use candidate versions before final
publication. Store version-specific notes in `docs/releases/<version>.md`;
the original 3.0.0 notes remain in `docs/RELEASE_NOTES_V3.md`.

Publishing triggers the existing Docker registry workflow. Stable releases also
trigger AUR publication when its credentials are configured. Prereleases must not replace stable `latest`, `3`, or
`3.0` Docker tags or update the stable AUR package. Verify registry images and
AUR installation after publication; the local image check cannot verify a
registry upload. No publication is performed by the local packaging scripts.

## Docker Hub README

`docs/docker.md` is the source for the public Docker Hub README. The
`Update Docker Hub README` workflow publishes it when that file or its workflow
changes on `main`. It can also be run manually. It uses the existing
`DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` secrets and does not rebuild or push
images.

The Docker Hub account must have repository Admin access, and the personal
access token must have the read/write/delete scope required by the description
action. Keep links absolute so they work on both GitHub and Docker Hub. Check
the workflow result and the public repository description after an update.

## Installed build requirements

See [Installing packages](INSTALLING_PACKAGES.md). Native builds require clang,
a system linker, and the exact Rust toolchain recorded inside the package.
The image includes these tools. Built programs retain ordinary system-library
requirements; build Linux assets against the oldest supported distribution and
test them there before advertising compatibility.

## Performance documentation

For compiler or runtime changes, use the [comparison procedure](../benchmarks/comparison/README.md)
to measure equivalent Python/v2/v3 workloads, preserve every sample, and validate
outputs. Review the complete report and LLVM build costs before summarizing it.
Refresh `README.md`, [benchmarks/RESULTS.md](../benchmarks/RESULTS.md), and the
chart from the recorded data. Keep internal Criterion results in their separate
report. Documentation-only changes can use the existing dataset when the
measured compiler source is unchanged.
