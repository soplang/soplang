# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali language, making programming more accessible to Somali speakers. It uniquely combines static and dynamic typing systems in one elegant language with a focus on clarity and ease of use.

**New to Soplang?** Check out our [comprehensive Installation Guide](docs/installation.md) for detailed setup instructions on all platforms.

## Features

- **Powerful type system** - Combines both static typing (`tiro`, `qoraal`, etc.) and dynamic typing (`door`) in one language
- **Somali-based syntax** - programming concepts expressed in Somali
- **Modern paradigms** - supports functional, procedural, and object-oriented programming
- **Type safety** - enforces type checking at runtime
- **Interpreter-based** - easy to run on any platform with Python
- **Interactive shell** - REPL for quick experimentation
- **Cross-platform** - dedicated installers for Windows, Linux, and macOS
- **Docker support** - run anywhere with Docker without installation
- **File extensions** - uses `.sop` (primary) and `.so` (secondary) file extensions

## Example

```js
// Hello World
bandhig("Salaan, Adduunka!")  // Prints: Hello, World!

// Variables with dual type system in action
door magac = "Sharafdin"  // Dynamic typing - can change type later
door num = 10             // Also dynamic
num = "new value"         // Valid: dynamic variables can change types

// Static typing examples
qoraal name = "Sharafdin" // Static typing - string only
tiro age = 25             // Static typing - number only

// Type safety enforcement
// age = "25"             // Would cause runtime error - type mismatch

// Control flow
haddii (age > 18) {
    bandhig("Waa qof weyn")   // Is an adult
} ugudambeyn {
    bandhig("Waa qof yar")    // Is a child
}

// Functions
hawl salaam(qof) {
    celi "Salaan, " + qof + "!"
}
bandhig(salaam(magac))        // Prints: Salaan, Sharafdin!
```

## Documentation

- [Getting Started](docs/index.md)
- [Language Reference](docs/language/keywords.md)
- [Examples](examples/)
- [Contributing Guide](docs/CONTRIBUTING.md)
- [Docker Guide](docs/docker.md)

## Installation

For detailed installation instructions, see the [Installation Guide](docs/installation.md).

### Download Installers

The easiest way to get started is to download a pre-built installer from our [releases page](https://github.com/sharafdin/soplang/releases).

### Using Docker (No Installation Required)

You can run Soplang using Docker without installing anything:

```bash
# Run the interactive shell
docker run -it --rm soplang/soplang

# Run a Soplang script
docker run -it --rm -v $(pwd):/scripts soplang/soplang my_script.sop
```

See the [Docker Guide](docs/docker.md) for more details.

### Building from Source

If you want to build Soplang from source, refer to the platform-specific build guides:

- **Windows**: See the [Windows Build Guide](windows/WINDOWS_BUILD_GUIDE.md)
- **Linux**: See the [Linux Build Guide](linux/README.md)
- **macOS**: See the [macOS Build Guide](macos/README.md)

You can also use our universal build script that automatically detects your platform:

```bash
# Clone the repository
git clone https://github.com/sharafdin/soplang.git
cd soplang

# Run the universal build script
./build.sh  # (may need chmod +x build.sh on Unix systems)
```

## Development

We provide a Makefile to simplify common development tasks:

```bash
# Setup your development environment
make install

# Run the interactive shell
make shell

# Run tests
make test

# Format code
make format

# See all commands
make help
```

## Releases

Soplang uses an automated release process powered by GitHub Actions. For details on creating releases, see our [Release Process Documentation](docs/RELEASE_PROCESS.md).

## Contributing

Contributions are welcome! See our [Contributing Guide](docs/CONTRIBUTING.md) for details on how to get started.

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. Please read our [Code of Conduct](CODE_OF_CONDUCT.md) to understand the community standards we uphold.

## License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT) - see the [LICENSE](LICENSE) file for details.
