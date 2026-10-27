# Soplang 3.6.0 verification record

## Release source and checks

[Soplang 3.6.0](https://github.com/soplang/soplang/releases/tag/v3.6.0)
was published on 2026-10-27 from commit
`1f1756d57ae9fdf11bd5983f39f4fa863ceec243`.
The [release verification run](https://github.com/soplang/soplang/actions/runs/40433861139)
passed its native package jobs on Linux, Windows, and macOS, its container
package job, and its toolchain-free package jobs on all three platforms before
creating the draft. The published tag points to the tested commit.

The release contains a Linux x86_64 archive and Debian package, a macOS Apple
Silicon archive, a Windows x64 archive and installer, and `SHA256SUMS`. All five
downloaded packages matched that checksum file. The extracted Linux package
reported `soplang 3.6.0`, ran a JIT program, and built a native executable
that produced the expected output without an installed Rust toolchain.

Local checks on the release source passed the 61 LLVM IR tests, 11 task tests,
and `cargo test --locked -- --skip test_aot`. The editor's dependency audit,
Node 22 check, and VSIX packaging passed locally. The
[editor CI run](https://github.com/soplang/soplang/actions/runs/40433828726)
passed on Linux, Windows, and macOS after rerunning a transient Rust toolchain
download failure.

## Documentation and distribution

The Soplang website's 3.6 downloads and documentation changes passed its five
unit tests, type check, lint, and production build. Its documentation checker
verified 106 code examples, including four expected errors, against the
downloaded Linux release package through the JIT and LLVM. The site changes
are committed in `soplang-site` at `a68009f`; the Next.js security update is
committed there at `297050c`.

The [Docker publication workflow](https://github.com/soplang/soplang/actions/runs/40452306458)
passed and pushed `soplang/soplang:3.6.0` with stable aliases. Its Linux amd64
manifest was publicly readable without Docker Hub credentials. The local Docker
daemon was unavailable, so an independent container runtime smoke test was not
performed; the release verification run's container package job passed.

The optional AUR upload was skipped because its credentials were not configured.
The `ghcr.io/soplang/soplang:3.6.0` manifest rejected an anonymous request;
the GHCR package is not yet verified for public pulls.

The site source is updated, but production still served 3.0.1 content at
`https://www.soplang.org/downloads` on 2026-10-27 and returned 404 for
`/docs/21_soplang-3-6`. A Vercel deployment of the new site commit is needed.
Docker Hub's README update is separate from image publishing; its existing
workflow received `Forbidden` from Docker Hub and did not update the listing
text.
