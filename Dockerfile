FROM python:3.10-slim AS builder

# Add build arguments for versioning
ARG VERSION=0.1.0
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown

# Set working directory
WORKDIR /app

# Install only the necessary build dependencies
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir wheel

# Copy requirements first for better layer caching
COPY requirements.txt .
RUN pip wheel --no-cache-dir --wheel-dir /app/wheels -r requirements.txt

# Final stage - smaller image
FROM python:3.10-slim

# Add build arguments for versioning
ARG VERSION=0.1.0
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown

# Set working directory
WORKDIR /app

# Copy wheels from builder stage
COPY --from=builder /app/wheels /wheels
RUN pip install --no-cache-dir --no-index --find-links=/wheels /wheels/* && \
    rm -rf /wheels

# Copy only the necessary files
COPY main.py .
COPY src/ ./src/

# Create a simple executable script to run Soplang
RUN echo '#!/bin/bash\npython /app/main.py "$@"' > /usr/local/bin/soplang && \
    chmod +x /usr/local/bin/soplang

# Set Python path to include the current directory
ENV PYTHONPATH=/app

# Create a non-root user to run the application
RUN groupadd -r soplang && \
    useradd -r -g soplang -d /home/soplang -m soplang && \
    chown -R soplang:soplang /app

# Create a volume for user scripts
VOLUME /scripts
WORKDIR /scripts

# Switch to non-root user
USER soplang

# Set the entrypoint to the soplang command
ENTRYPOINT ["soplang"]

# By default, start the interactive shell
CMD []

# Standard Docker labels from OCI Image Spec
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
