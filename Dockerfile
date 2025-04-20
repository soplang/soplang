FROM python:3.10-slim

# Set working directory
WORKDIR /app

# Install only the necessary dependency
RUN pip install --no-cache-dir colorama>=0.4.0

# Copy only the necessary files
COPY main.py .
COPY src/ ./src/

# Create a simple executable script to run Soplang
RUN echo '#!/bin/bash\npython /app/main.py "$@"' > /usr/local/bin/soplang && \
    chmod +x /usr/local/bin/soplang

# Set Python path to include the current directory
ENV PYTHONPATH=/app

# Create a volume for user scripts
VOLUME /scripts
WORKDIR /scripts

# Set the entrypoint to the soplang command
ENTRYPOINT ["soplang"]

# By default, start the interactive shell
CMD []

# Metadata
LABEL maintainer="info@soplang.org"
LABEL version="0.1.0"
LABEL description="The Somali Programming Language"
