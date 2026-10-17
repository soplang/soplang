# Soplang 3 with Docker

The public Linux amd64 image is available as
[`soplang/soplang:3.0.0`](https://hub.docker.com/r/soplang/soplang).
It includes Rust 1.94.0, clang 16, the system linker, and the matching Soplang
runtime libraries for `--build`. Programs run as the unprivileged `soplang` user.

## Run a program or the REPL

```sh
docker run --rm soplang/soplang:3.0.0 --version
docker run --rm -v "$PWD:/scripts:ro" soplang/soplang:3.0.0 program.sop
docker run -it --rm soplang/soplang:3.0.0
```

Paths are relative to the mounted `/scripts` directory. From the repository
root, an example can be run as follows:

```sh
docker run --rm -v "$PWD:/scripts:ro" soplang/soplang:3.0.0 examples/01_hello.sop
```

Use `-i` for piped input and `-it` for an interactive terminal:

```sh
printf 'TestUser\n' | docker run --rm -i -v "$PWD:/scripts:ro" \
  soplang/soplang:3.0.0 examples/13_input.sop
```

## Build a native executable

Mount a writable directory for the output. On Linux, matching the host user ID
helps preserve ownership of generated files:

```sh
docker run --rm --user "$(id -u):$(id -g)" -v "$PWD:/scripts" \
  soplang/soplang:3.0.0 --build program.sop -o program
```

A container-built executable targets Linux amd64 and the container's system
libraries. It does not become a macOS or Windows executable. See
[package requirements](INSTALLING_PACKAGES.md) for native toolchain details.

## Registry tags and availability

Stable release publication sets these Docker Hub tags:

| Tag | Meaning |
| --- | --- |
| `3.0.0` | This release |
| `3.0` | Latest stable patch in the 3.0 line |
| `3` | Latest stable release in the 3.x line |
| `latest` | Latest stable release |
| `sha-<commit>` | Release source revision |

Pin a version for repeatable use. Prereleases do not replace stable aliases.
The published `3.0.0` and `latest` manifests matched in the release checks;
the pulled image passed version, JIT, and LLVM native-build smoke tests.

The workflow also uploads to `ghcr.io/soplang/soplang`. Anonymous pulls were
still rejected in the 2026-10-17 documentation check; GitHub package visibility
must be Public before it can serve as an unauthenticated mirror. Docker Hub
is the verified public download. See the
[release verification record](testing/RELEASE_VERIFICATION.md).

## Build and verify the image locally

```sh
git clone https://github.com/soplang/soplang.git
cd soplang
docker build -t soplang:local .
docker run --rm soplang:local --version
```

The image entrypoint is `soplang`; arguments after the image name are CLI flags
or source paths. Use `--entrypoint` to run other tools. The complete package
checker is documented in the [release process](RELEASE_PROCESS.md).

For compiler development and tests, use the source workflow in
[Contributing](CONTRIBUTING.md). Release images contain the installed compiler
and toolchain; compiling a changed checkout requires building that source.
