# Contributing to Soplang

Thank you for your interest in contributing to Soplang, the Somali programming language! This document provides guidelines and instructions for contributing to the project.

## Development Setup

There are two recommended ways to set up your development environment:

### Option 1: Docker (Recommended)

Using Docker provides a consistent development environment for all contributors and eliminates "works on my machine" issues.

1. **Install Docker and Docker Compose**:
   - [Docker Installation Guide](https://docs.docker.com/get-docker/)
   - [Docker Compose Installation Guide](https://docs.docker.com/compose/install/)

2. **Clone the repository**:
   ```bash
   git clone https://github.com/sharafdin/soplang.git
   cd soplang
   ```

3. **Build and start the Docker container**:
   ```bash
   docker-compose build
   docker-compose up -d
   ```

4. **Run commands inside the container**:
   ```bash
   # Run the shell
   docker-compose exec soplang python main.py

   # Run a Soplang file
   docker-compose exec soplang python main.py examples/hello_world.so

   # Run tests
   docker-compose exec soplang python -m unittest discover tests
   ```

5. **Stop the container when done**:
   ```bash
   docker-compose down
   ```

### Option 2: Local Setup

If you prefer working on your local machine:

#### Prerequisites

- Python 3.6 or higher
- Git
- A text editor or IDE

#### Setting up the Local Environment

1. **Clone the repository**:
   ```bash
   git clone https://github.com/sharafdin/soplang.git
   cd soplang
   ```

2. **Create a virtual environment** (optional but recommended):
   ```bash
   python -m venv env
   source env/bin/activate  # On Windows: env\Scripts\activate
   ```

3. **Install development dependencies**:
   ```bash
   pip install -r requirements-dev.txt  # If available
   ```

### Using the Makefile

We provide a Makefile to simplify common development tasks. To use it, you need:

#### Prerequisites

- GNU Make (comes pre-installed on most Linux distributions and macOS)
  - On Windows, you can install it via [Windows Subsystem for Linux](https://docs.microsoft.com/en-us/windows/wsl/install), [Chocolatey](https://chocolatey.org/packages/make), or [MSYS2](https://www.msys2.org/)

#### Common Commands

```bash
# Install all dependencies
make install

# Run tests
make test

# Format code
make format

# Run linting
make lint

# Run pre-commit hooks on all files
make precommit

# Build Docker container
make docker-build

# See all available commands
make help
```

Use `make help` to see all available commands and their descriptions.

## Development Workflow

1. **Choose an issue or feature** to work on, or create a new issue describing the problem or enhancement.

2. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**, following the coding style and best practices.

4. **Write or update tests** for your changes. Ensure all tests pass:
   ```bash
   make test
   # or the traditional way
   python -m unittest discover tests
   ```

5. **Update documentation** if necessary.

6. **Commit your changes** with clear, descriptive commit messages:
   ```bash
   git commit -m "Feature: Add support for new loop syntax"
   ```

7. **Create a pull request** against the main branch with a clear description of your changes.

## Coding Guidelines

- Follow PEP 8 style guidelines for Python code
- Write docstrings for all functions, classes, and modules
- Maintain consistent error messages in Somali language
- Add appropriate tests for new features and bug fixes

### Pre-commit Hooks

We use pre-commit hooks to ensure code quality and consistency. To set up pre-commit in your development environment:

1. **Install pre-commit**:
   ```bash
   pip install pre-commit
   # or using Make
   make install-dev
   ```

   Note: This is already included if you installed `requirements-dev.txt`.

2. **Install the git hooks**:
   ```bash
   pre-commit install
   # or using Make
   make install
   ```

3. **Run the hooks manually** (optional):
   ```bash
   pre-commit run --all-files
   # or using Make
   make precommit
   ```

The pre-commit configuration includes:
- Black for code formatting
- Flake8 for linting
- isort for import sorting
- Various file checks (trailing whitespace, file endings, etc.)

Pre-commit will run automatically on `git commit`, but you can also run it manually as shown above.

## Testing

- Write unit tests for all new functionality
- Update existing tests when modifying functionality
- Ensure all tests pass before submitting a pull request

Run tests using:
```bash
make test
# or
python -m unittest discover tests
```

## Documentation

- Update documentation for any changes to the language or tools
- Document new features with examples
- Keep the keyword reference up to date
- Write clear, concise explanations

## Pull Request Process

1. Ensure all tests pass
2. Update relevant documentation
3. Explain what your change does and why it should be included
4. Request a review from a maintainer
5. Address any feedback or requested changes

## Code of Conduct

Please be respectful and inclusive in your interactions with other contributors. We aim to foster an open and welcoming community. For detailed guidelines, please read our [Code of Conduct](../CODE_OF_CONDUCT.md).
