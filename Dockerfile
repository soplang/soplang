FROM ubuntu:24.04 AS builder
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential ca-certificates cmake curl liblld-18-dev llvm-18-dev \
    libssl-dev libzstd-dev pkg-config python3 && \
    rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 --fail --silent --show-error https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain 1.94.0 --profile minimal
ENV PATH="/root/.cargo/bin:/usr/lib/llvm-18/bin:${PATH}"
ENV SOPLANG_LLD_SDK=/usr/lib/llvm-18
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml build.rs ./
COPY src ./src
COPY benches ./benches
COPY scripts/package_native.py ./scripts/package_native.py
COPY README.md LICENSE ./
COPY licenses ./licenses
COPY docs/INSTALLING_PACKAGES.md ./docs/INSTALLING_PACKAGES.md
RUN python3 scripts/package_native.py --output /package

FROM ubuntu:24.04 AS runtime
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libgcc-s1 python3 && rm -rf /var/lib/apt/lists/*
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
