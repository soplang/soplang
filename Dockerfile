FROM python:3.10-slim AS builder

# Add build arguments for versioning
ARG PYTHON_VERSION=3.11
ARG VERSION=2.0.0-beta
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown

# Set working directory
WORKDIR /app

# Install only the necessary build dependencies
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir wheel

# Install core dependencies directly (in case requirements.txt isn't present)
RUN pip install --no-cache-dir colorama>=0.4.0 prompt_toolkit>=3.0.0

# Copy requirements if it exists and install any additional dependencies
COPY requirements.txt* ./
RUN if [ -f requirements.txt ]; then \
    pip wheel --no-cache-dir --wheel-dir /app/wheels -r requirements.txt; \
    fi

# Final stage - smaller image
FROM python:3.10-slim

# Add build arguments for versioning
ARG PYTHON_VERSION=3.11
ARG VERSION=2.0.0-beta
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown

# Set working directory
WORKDIR /app

# Install core dependencies directly
RUN pip install --no-cache-dir colorama>=0.4.0 prompt_toolkit>=3.0.0

# Copy wheels from builder stage if any were created
COPY --from=builder /app/wheels* /wheels
RUN if [ -d /wheels ]; then \
    pip install --no-cache-dir --no-index --find-links=/wheels /wheels/* && \
    rm -rf /wheels; \
    fi

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
