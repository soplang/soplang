# Soplang Docker image

Soplang is a programming language with Somali keywords. Soplang 3 is written
in Rust and uses Cranelift JIT to run programs and LLVM to build native
executables.

The [`soplang/soplang:3.0.1`](https://hub.docker.com/r/soplang/soplang) image
targets Linux amd64. It includes Rust 1.94.0, clang 16, a system linker, and
the matching runtime libraries needed by `--build`. It runs as the
unprivileged `soplang` user.

## Quick start

Check the installed version:

```sh
docker run --platform linux/amd64 --rm soplang/soplang:3.0.1 --version
```

Start the interactive shell:

```sh
docker run --platform linux/amd64 --rm -it soplang/soplang:3.0.1
```

Run a code snippet:

```sh
docker run --platform linux/amd64 --rm soplang/soplang:3.0.1 \
  -c 'qor("Salaan, Adduunka!")'
```

The commands specify `linux/amd64` because that is the published image's
platform. Other host architectures require Docker's emulation support.

## Run a source file

Save this program as `hello.sop`:

```sop
qor("Salaan, Adduunka!")
```

Mount its directory at `/scripts` and pass the filename:

```sh
docker run --platform linux/amd64 --rm -v "$PWD:/scripts:ro" \
  soplang/soplang:3.0.1 hello.sop
```

Paths inside the container are relative to `/scripts`. Mount the complete
project directory when a program uses `keen` to import other files. The
`:ro` suffix makes the source mount read-only.

In PowerShell, use a bind mount with the current directory's full path:

```powershell
docker run --platform linux/amd64 --rm --mount "type=bind,source=$($PWD.Path),target=/scripts,readonly" soplang/soplang:3.0.1 hello.sop
```

For piped input, add `-i`:

```sh
printf 'Ayaan\n' | docker run --platform linux/amd64 --rm -i \
  soplang/soplang:3.0.1 -c 'qor(gelin())'
```

## Build a native executable

Use a writable mount so `--build` can save its output. On Linux, match the
host user's ID so the output belongs to that user:

```sh
docker run --platform linux/amd64 --rm --user "$(id -u):$(id -g)" \
  -v "$PWD:/scripts" \
  soplang/soplang:3.0.1 --build hello.sop -o hello
```

The selected user must have write access to the mounted directory. For
Docker Desktop on Windows, use the PowerShell mount syntax above without
`readonly`, omit `--user`, and pass `--build hello.sop -o hello`.

Run the built program in the same image:

```sh
docker run --platform linux/amd64 --rm -v "$PWD:/scripts:ro" \
  --entrypoint /scripts/hello soplang/soplang:3.0.1
```

The executable targets Linux amd64 and the container's system libraries.
It can run on a compatible Linux system without Soplang. Building for macOS
or Windows requires the corresponding native build environment. See the
[package requirements](https://github.com/soplang/soplang/blob/main/docs/INSTALLING_PACKAGES.md).

## Image tags

| Tag | Meaning |
| --- | --- |
| `3.0.1` | Soplang 3.0.1 |
| `3.0` | Latest stable patch in the 3.0 line |
| `3` | Latest stable release in the 3.x line |
| `latest` | Latest stable release |
| `sha-<commit>` | Release source revision |

Use a specific version in scripts and deployments. Prereleases do not replace
stable aliases. Published images are also uploaded to
`ghcr.io/soplang/soplang`; anonymous access depends on the GitHub package's
visibility. The commands above use Docker Hub.

## Build the image from source

```sh
git clone --branch v3.0.1 --depth 1 https://github.com/soplang/soplang.git
cd soplang
docker build -t soplang:local .
docker run --rm soplang:local --version
```

The image entrypoint is `soplang`. Arguments after the image name are compiler
options or source paths. Override `--entrypoint` to run another tool.

## Documentation

- [Language documentation](https://www.soplang.org/docs)
- [Source code](https://github.com/soplang/soplang)
- [Release packages](https://github.com/soplang/soplang/releases/tag/v3.0.1)
- [Release verification](https://github.com/soplang/soplang/blob/main/docs/testing/LOCALIZATION_3_0_1.md)
- [Bug reports](https://github.com/soplang/soplang/issues)
- [MIT license](https://github.com/soplang/soplang/blob/main/LICENSE)
