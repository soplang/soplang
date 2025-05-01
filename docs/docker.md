# Using Soplang with Docker

Soplang is available as a Docker image, making it easy to run on any platform with Docker installed without dealing with installation or dependencies.

## Docker Images

Soplang Docker images are automatically built and published to:

1. **Docker Hub**: [soplang/soplang](https://hub.docker.com/r/soplang/soplang)
2. **GitHub Container Registry**: [ghcr.io/soplang/soplang](https://github.com/soplang/soplang/pkgs/container/soplang)

## Available Tags

- `latest` - Always points to the latest stable release
- `x.y.z` - Specific version (e.g., `0.1.0`)
- `x.y` - Latest patch version of a minor release (e.g., `0.1`)
- `x` - Latest minor version of a major release (e.g., `0`)
- `main` - Built from the main branch (may be unstable)
- `<commit-sha>` - Specific commit (for debugging)

## Usage

### Running the Interactive Shell

To start the Soplang interactive shell:

```bash
docker run -it --rm soplang/soplang
```

### Running a Script

To run a Soplang script from your current directory:

```bash
docker run -it --rm -v $(pwd):/scripts soplang/soplang my_script.sop
```

Note that the file path should be relative to your mounted directory. For example, if your script is in the examples folder:

```bash
# If running from the project root
docker run -it --rm -v $(pwd):/scripts soplang/soplang examples/01_dynamic_typing.sop

# If examples/ is in your current directory
cd examples
docker run -it --rm -v $(pwd):/scripts soplang/soplang 01_dynamic_typing.sop
```

### Mounting a Volume for Persistent Scripts

To maintain a persistent workspace:

```bash
docker run -it --rm -v /path/to/soplang/scripts:/scripts soplang/soplang
```

### Setting a Custom Working Directory

```bash
docker run -it --rm -v $(pwd):/scripts -w /scripts/my_project soplang/soplang
```

## Examples

### Hello World

```bash
# Create a hello.sop file
echo 'qor("Salaan, Adduunka!")' > hello.sop

# Run it with Docker
docker run -it --rm -v $(pwd):/scripts soplang/soplang hello.sop
```

### Interactive Development

For an interactive development experience, you can use a more complex setup:

```bash
# Create a development container with your scripts mounted
docker run -it --rm --name soplang-dev \
  -v $(pwd):/scripts \
  soplang/soplang
```

## Building the Docker Image Locally

If you want to build the Docker image locally:

```bash
# Clone the repository
git clone https://github.com/sharafdin/soplang.git
cd soplang

# Build the image
docker build -t soplang:local .

# Run with your local build
docker run -it --rm soplang:local
```

## Environment Variables

The Soplang Docker image supports the following environment variables:

- `PYTHONPATH`: Customize the Python path (default: `/app`)
- `PYTHONUNBUFFERED`: Control Python output buffering

## Security

The Soplang Docker image:

- Runs as a non-root user `soplang` for better security
- Uses a multi-stage build to minimize image size
- Keeps dependencies updated with each build

## Troubleshooting

If you encounter issues with the Docker image:

1. **Permission Issues**: Make sure you have proper permissions for mounted volumes
2. **Interactive Shell Not Working**: Ensure you're using the `-it` flags
3. **Script Not Found**: Check that your volume is mounted correctly and the path is relative to your mounted directory
