# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali language, making programming more accessible to Somali speakers. It uniquely combines static and dynamic typing systems in one elegant language with a focus on clarity and ease of use.

## Features

- **Powerful type system** - Combines both static typing (`tiro`, `qoraal`, etc.) and dynamic typing (`door`) in one language
- **Somali-based syntax** - programming concepts expressed in Somali
- **Modern paradigms** - supports functional, procedural, and object-oriented programming
- **Type safety** - enforces type checking at runtime
- **Interpreter-based** - easy to run on any platform with Python
- **Interactive shell** - REPL for quick experimentation

## Example

```js
// Hello World
qor("Salaan, Adduunka!")  // Prints: Hello, World!

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
    qor("Waa qof weyn")   // Is an adult
} haddii_kalena {
    qor("Waa qof yar")    // Is a child
}

// Functions
howl salaam(qof) {
    soo_celi "Salaan, " + qof + "!"
}
qor(salaam(magac))        // Prints: Salaan, Sharafdin!
```

## Documentation

- [Getting Started](docs/index.md)
- [Language Reference](docs/language/keywords.md)
- [Examples](examples/)
- [Contributing Guide](docs/CONTRIBUTING.md)

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

## Contributing

Contributions are welcome! See our [Contributing Guide](docs/CONTRIBUTING.md) for details on how to get started.

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. Please read our [Code of Conduct](CODE_OF_CONDUCT.md) to understand the community standards we uphold.

## License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT) - see the [LICENSE](LICENSE) file for details.
