# Keep the runtime rlibs and installed rustc on the same exact toolchain.
FROM rust:1.94.0-bookworm AS toolchain
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang-16 python3 ca-certificates && \
    ln -s /usr/bin/clang-16 /usr/local/bin/clang && rm -rf /var/lib/apt/lists/*

FROM toolchain AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src ./src
COPY benches ./benches
COPY scripts/package.py ./scripts/package.py
COPY README.md LICENSE ./
COPY licenses ./licenses
COPY docs/INSTALLING_PACKAGES.md ./docs/INSTALLING_PACKAGES.md
RUN python3 scripts/package.py --output /package

FROM toolchain AS runtime
ARG VERSION=dev
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown
COPY --from=builder /package/soplang /usr/local/bin/soplang
COPY --from=builder /package/lib/soplang /usr/local/lib/soplang
COPY --from=builder /package/LICENSE /package/INSTALLING.md /usr/local/share/doc/soplang/
COPY --from=builder /package/licenses /usr/local/share/doc/soplang/licenses
RUN groupadd -r soplang && useradd -r -g soplang -d /home/soplang -m soplang
VOLUME /scripts
WORKDIR /scripts
USER soplang
ENTRYPOINT ["soplang"]
CMD []
LABEL org.opencontainers.image.title="Soplang" \
    org.opencontainers.image.description="The Somali Programming Language" \
    org.opencontainers.image.url="https://www.soplang.org/" \
    org.opencontainers.image.source="https://github.com/soplang/soplang" \
    org.opencontainers.image.version="${VERSION}" \
    org.opencontainers.image.created="${BUILD_DATE}" \
    org.opencontainers.image.revision="${VCS_REF}" \
    org.opencontainers.image.licenses="MIT" \
    org.opencontainers.image.vendor="Soplang Software Foundation" \
    org.opencontainers.image.authors="info@soplang.org"
