# Soplang scripts

- `package.py --output dist/soplang`: locked release build and relocatable
  distribution. Uses Cargo's JSON artifact list, not every old rlib in target.
- `verify_package.py /path/to/soplang`: relocates a distribution away from the
  checkout and checks all public examples, error snapshots, REPL, concurrent
  native builds, runtime lookup, toolchain validation, and standalone outputs.
- `verify_installation.py <debian|windows> <package>`: install on a disposable
  runner, execute the installed CLI through JIT/LLVM, and verify removal. The
  Windows check also covers PATH and file associations.
- `release_assets.py <artifact-directory>`: validate each platform checksum and
  create one combined manifest with unique asset names for publication.
- `checksums.py <assets...> --output SHA256SUMS`: hashes actual files and writes
  SHA-256 records by filename. Run verification beside the assets.

These tools require Python 3.9+. Platform builders call the packaging and
checksum tools. See [release verification](../docs/testing/RELEASE_VERIFICATION.md)
and the [release process](../docs/RELEASE_PROCESS.md). None of these scripts
commits, tags, pushes, or publishes.

The [benchmark comparison tooling](../benchmarks/comparison/README.md) is separate
from packaging. It requires Python 3.12+ and exports the original v2 interpreter
for comparison with Python and the v3 compiler. Current results are in
[benchmarks/RESULTS.md](../benchmarks/RESULTS.md).

## Unicode titlecase data

`generate_unicode_case.py` reads the official Unicode 17.0.0 `UnicodeData.txt`
and `SpecialCasing.txt` files and writes the sparse titlecase exceptions used
by text methods. It requires Python 3.9+ and `rustfmt`; ordinary Soplang builds
use the checked-in table and require neither input downloads nor Python.

```sh
python3 scripts/generate_unicode_case.py /path/to/UnicodeData.txt /path/to/SpecialCasing.txt src/runtime/unicode_case.rs
```

Input links and the Unicode license are recorded in the script and
[licenses/UNICODE.txt](../licenses/UNICODE.txt). Regenerate the table and review
casing conformance when the toolchain's Unicode version changes. Tests keep
casing data and word segmentation on the same Unicode version.
