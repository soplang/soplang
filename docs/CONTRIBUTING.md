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

## Project Structure

- `src/` - Core implementation files
  - `lexer.py` - Tokenizer for Soplang code
  - `parser.py` - Parser that builds an AST
  - `interpreter.py` - Interpreter that executes the AST
  - `ast.py` - Abstract Syntax Tree definitions
  - `tokens.py` - Token type definitions
  - `errors.py` - Error handling and messages
  - `builtins.py` - Built-in functions
  - `shell.py` - Interactive shell implementation
  
- `examples/` - Example Soplang programs
  
- `tests/` - Unit and integration tests
  
- `docs/` - Documentation
  - `language/` - Language reference documentation
  - `build/` - Build and compilation documentation
  - `testing/` - Testing documentation
  - `examples/` - Example documentation

## Development Workflow

1. **Choose an issue or feature** to work on, or create a new issue describing the problem or enhancement.

2. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**, following the coding style and best practices.

4. **Write or update tests** for your changes. Ensure all tests pass:
   ```bash
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

## Testing

- Write unit tests for all new functionality
- Update existing tests when modifying functionality
- Ensure all tests pass before submitting a pull request

Run tests using:
```bash
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

Please be respectful and inclusive in your interactions with other contributors. We aim to foster an open and welcoming community. 