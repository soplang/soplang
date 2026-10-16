# Build stage
FROM rust:1-bookworm AS builder

ARG VERSION=dev
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown

WORKDIR /app

# Copy manifests and source
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

ARG VERSION=dev
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/soplang /usr/local/bin/soplang

# Create a non-root user
RUN groupadd -r soplang && useradd -r -g soplang -d /home/soplang -m soplang

VOLUME /scripts
WORKDIR /scripts
USER soplang

ENTRYPOINT ["soplang"]
CMD []

LABEL org.opencontainers.image.title="Soplang"
LABEL org.opencontainers.image.description="The Somali Programming Language"
LABEL org.opencontainers.image.url="https://www.soplang.org/"
LABEL org.opencontainers.image.source="https://github.com/soplang/soplang"
LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.created="${BUILD_DATE}"
LABEL org.opencontainers.image.revision="${VCS_REF}"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.vendor="Soplang Software Foundation"
LABEL org.opencontainers.image.authors="info@soplang.org"
