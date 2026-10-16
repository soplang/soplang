# Soplang scripts

- `package.py --output dist/soplang`: locked release build and relocatable
  distribution. Uses Cargo's JSON artifact list, not every old rlib in target.
- `verify_package.py /path/to/soplang`: relocates a distribution away from the
  checkout and checks all public examples, error snapshots, REPL, concurrent
  native builds, runtime lookup, toolchain validation, and standalone outputs.
- `checksums.py <assets...> --output SHA256SUMS`: hashes actual files and writes
  SHA-256 records by filename. Run verification beside the assets.

These tools require Python 3.9+. Platform builders call the packaging and
checksum tools. See [release verification](../docs/testing/RELEASE_VERIFICATION.md)
and the [release process](../docs/RELEASE_PROCESS.md). None of these scripts
commits, tags, pushes, or publishes.
