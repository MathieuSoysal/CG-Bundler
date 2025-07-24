# Contributing to CG Bundler

First off, thank you for considering contributing to CG Bundler! It's people like you that make CG Bundler such a great tool for the Rust community.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

### Issues

- **Bug reports**: Create a detailed issue with steps to reproduce
- **Feature requests**: Describe the feature and its use case
- **Questions**: Use GitHub Discussions for general questions

### Good First Issues

Look for issues labeled `good first issue` - these are perfect for newcomers!

## Development Setup

### Prerequisites

- **Rust 1.75.0+** (install via [rustup](https://rustup.rs/))
- **Git**

### Setup Instructions

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/cg-bundler.git
   cd cg-bundler
   ```

2. **Build the project**:
   ```bash
   cargo build
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

4. **Install development tools**:
   ```bash
   # For formatting
   rustup component add rustfmt
   
   # For linting
   rustup component add clippy
   
   # For coverage (optional)
   cargo install cargo-llvm-cov
   ```

## How to Contribute

### Types of Contributions

We welcome several types of contributions:

- 🐛 **Bug fixes**
- ✨ **New features**
- 📝 **Documentation improvements**
- 🧪 **Tests**
- 🔧 **Code refactoring**
- 🎨 **Performance improvements**

### Development Workflow

1. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

2. **Make your changes** following our code style guidelines

3. **Test your changes**:
   ```bash
   # Run all tests
   cargo test
   
   # Run specific test
   cargo test test_name
   
   # Run integration tests
   cargo test --test integration_tests
   ```

4. **Check formatting and lints**:
   ```bash
   # Format code
   cargo fmt
   
   # Check lints
   cargo clippy -- -D warnings
   ```

## Testing

### Running Tests

```bash
# All tests
cargo test --verbose

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration_tests

# Specific test
cargo test bundle_simple_project
```

### Writing Tests

- **Unit tests**: Place in the same file as the code being tested
- **Integration tests**: Place in the `tests/` directory
- **Example tests**: Test CLI functionality in `tests/cli_tests.rs`

### Test Guidelines

- Write descriptive test names
- Test both success and error cases
- Use `tempfile` for temporary files in tests
- Mock external dependencies when possible

## Code Style

### Formatting

We use `rustfmt` with default settings:

```bash
cargo fmt
```

### Linting

We enforce strict clippy lints:

```bash
cargo clippy -- -D warnings
```

### Code Guidelines

- **Error handling**: Use `anyhow` for application errors, `thiserror` for library errors
- **Documentation**: Document all public APIs with `///` comments
- **Naming**: Use clear, descriptive names
- **Performance**: Avoid unnecessary allocations in hot paths
- **Safety**: Prefer safe Rust; document any `unsafe` usage

### Commit Messages

Follow [Conventional Commits](https://conventionalcommits.org/):

```
feat: add watch mode with file system monitoring
fix: handle edge case in module resolution
docs: update README with new CLI options
test: add integration tests for bundling
refactor: simplify error handling in transformer
chore: update dependencies
```

## Submitting Changes

### Pull Request Process

1. **Update documentation** if needed
2. **Add tests** for new functionality
3. **Ensure CI passes** (formatting, lints, tests)
4. **Update CHANGELOG.md** with your changes
5. **Submit PR** with a clear description

### PR Guidelines

- **Title**: Use a descriptive title
- **Description**: Explain what and why, not just how
- **Link issues**: Reference related issues
- **Screenshots**: Include for UI changes
- **Breaking changes**: Clearly document any breaking changes

### Review Process

- All submissions require review
- We may suggest changes or improvements
- Once approved, maintainers will merge the PR

## Community

### Getting Help

- **GitHub Discussions**: General questions and ideas
- **Issues**: Bug reports and feature requests
- **Email**: For security issues only

### Recognition

Contributors are recognized in:
- Release notes
- `CHANGELOG.md`
- GitHub contributor list

## Development Tips

### Useful Commands

```bash
# Quick development cycle
cargo check     # Fast compilation check
cargo test      # Run tests
cargo clippy    # Lint check
cargo fmt       # Format code

# Release build
cargo build --release

# Install locally for testing
cargo install --path .

# Generate documentation
cargo doc --open
```

### Project Structure

```
cg-bundler/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library root
│   ├── bundler.rs        # Core bundling logic
│   ├── transformer.rs    # Code transformation
│   ├── file_manager.rs   # File operations
│   ├── cargo_project.rs  # Cargo project handling
│   └── error.rs          # Error types
├── tests/                # Integration tests
├── examples/             # Usage examples
└── docs/                 # Additional documentation
```

### Performance Considerations

- Profile with `cargo bench` for performance-critical changes
- Use `cargo flamegraph` for detailed profiling
- Consider memory usage in large projects

## Questions?

Don't hesitate to ask! We're here to help:

- Open a [GitHub Discussion](https://github.com/MathieuSoysal/cg-bundler/discussions)
- Comment on relevant issues
- Join our community discussions

**Thank you for contributing to CG Bundler!** 🎉
