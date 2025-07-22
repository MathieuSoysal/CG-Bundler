# CG Bundler 🔧

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/cg-bundler.svg)](https://crates.io/crates/cg-bundler)
[![Documentation](https://docs.rs/cg-bundler/badge.svg)](https://docs.rs/cg-bundler)
[![Build Status](https://github.com/MathieuSoysal/cg-bundler/workflows/CI/badge.svg)](https://github.com/MathieuSoysal/cg-bundler/actions)
[![Security Rating](https://sonarcloud.io/api/project_badges/measure?project=MathieuSoysal_CG-Bundler&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=MathieuSoysal_CG-Bundler)
[![Quality gate](https://sonarcloud.io/api/project_badges/quality_gate?project=MathieuSoysal_CG-Bundler)](https://sonarcloud.io/summary/new_code?id=MathieuSoysal_CG-Bundler)

**A powerful Rust code bundler that combines multiple source files into a single, optimized file for competitive programming and code distribution.**

[📖 Documentation](https://docs.rs/cg-bundler) | [🚀 Getting Started](#getting-started) | [💡 Examples](#examples) | [🤝 Contributing](#contributing)

</div>

---

### Quick use

_With cargo :_
```bash
cargo install cg-bundler
cg-bundler > output.rs

# Or with watch mode for live updates
cg-bundler --watch -o output.rs
```

_Without cargo :_
```bash
curl -L https://github.com/MathieuSoysal/cg-bundler/releases/latest/download/cg-bundler-linux-amd64 -o cg-bundler
bash cg-bundler > output.rs
```

## ✨ Features

- 🚀 **Fast bundling** - Efficiently combines Rust projects into single files
- 🔍 **Smart module expansion** - Automatically resolves and inlines module dependencies
- 🧹 **Code optimization** - Removes tests, documentation, and unused code
- 🎛️ **Configurable transformation** - Customize what gets included/excluded
- 📦 **Cargo integration** - Works seamlessly with standard Cargo projects
- 🔧 **CLI & Library** - Use as command-line tool or integrate into your workflow
- ⚡ **Minification** - Optional code minification for size optimization
- 🛡️ **Error handling** - Comprehensive error reporting with context
- 🔄 **Watch mode** - **NEW** Automatic rebuilding on file changes for live development

## 🚀 Getting Started

### Installation

#### From Crates.io (Recommended)
```bash
cargo install cg-bundler
```

#### From Source
```bash
git clone https://github.com/MathieuSoysal/cg-bundler.git
cd cg-bundler
cargo install --path .
```

#### As Library Dependency
Add to your `Cargo.toml`:
```toml
[dependencies]
cg-bundler = "1.1.0"
```

### Quick Start

#### Command Line Usage
```bash
# Bundle current directory
cg-bundler

# Bundle specific project
cg-bundler /path/to/rust/project

# Output to file
cg-bundler -o bundled.rs

# Minify output
cg-bundler --minify -o compressed.rs

# Keep documentation and tests
cg-bundler --keep-docs --keep-tests

# NEW: Watch mode for live development
cg-bundler --watch -o output.rs

# Watch with verbose output and fast response
cg-bundler --watch -o output.rs --verbose --debounce 200
```

#### Library Usage
```rust
use cg_bundler::{bundle, Bundler, TransformConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple bundling
    let bundled_code = bundle("./my_project")?;
    println!("{}", bundled_code);

    // Advanced configuration
    let config = TransformConfig {
        remove_tests: true,
        remove_docs: true,
        expand_modules: true,
        minify: false,
        aggressive_minify: false,
    };
    
    let bundler = Bundler::with_config(config);
    let result = bundler.bundle("./my_project")?;
    
    Ok(())
}
```

## 📋 Requirements

- **Rust 1.75.0** or later
- **Cargo** (comes with Rust)
- Compatible with **all Rust editions** (2015, 2018, 2021)

## 🎯 Use Cases

### Competitive Programming
Perfect for platforms like Codeforces, AtCoder, or Codingame where you need to submit a single source file:

```bash
# Bundle your modular solution into a single file
cg-bundler --minify --output solution.rs ./my-contest-solution/
```

### Code Distribution
Share your Rust libraries as single files for easy integration:

```bash
# Create a distributable single-file version
cg-bundler --keep-docs --output my-lib-standalone.rs ./my-library/
```

### CI/CD Integration
Integrate into your build pipeline:

```bash
# Validate that your project can be bundled
cg-bundler --validate

# Get project information
cg-bundler --info
```

## 💡 Examples

### Basic Project Structure
```
my_project/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs          # Optional
│   ├── utils.rs
│   ├── game/
│   │   ├── mod.rs
│   │   └── engine.rs
│   └── ai/
│       ├── mod.rs
│       └── strategy.rs
```

### Input: Modular Code
```rust
// src/main.rs
use my_project::game::GameEngine;
use my_project::ai::Strategy;

fn main() {
    let engine = GameEngine::new();
    let strategy = Strategy::default();
    engine.run_with_strategy(strategy);
}
```

```rust
// src/game/mod.rs
pub mod engine;
pub use engine::GameEngine;
```

### Output: Bundled Code
```rust
// All modules expanded and combined
pub mod game {
    pub struct GameEngine { /* ... */ }
    impl GameEngine {
        pub fn new() -> Self { /* ... */ }
        pub fn run_with_strategy(&self, strategy: Strategy) { /* ... */ }
    }
}

pub mod ai {
    pub struct Strategy { /* ... */ }
    impl Default for Strategy { /* ... */ }
}

use game::GameEngine;
use ai::Strategy;

fn main() {
    let engine = GameEngine::new();
    let strategy = Strategy::default();
    engine.run_with_strategy(strategy);
}
```

## 🎛️ CLI Options

| Option | Short | Description |
|--------|-------|-------------|
| `--output` | `-o` | Output file path (stdout if not specified) |
| `--keep-tests` | | Keep test code in the bundled output |
| `--keep-docs` | | Keep documentation comments |
| `--no-expand-modules` | | Disable module expansion |
| `--pretty` | | Pretty print the output (format with rustfmt) |
| `--minify` | `-m` | Minify the output to a single line |
| `--m2` | | Aggressive minify with whitespace replacements |
| `--verbose` | `-v` | Verbose output |
| `--validate` | | Validate project can be bundled without errors |
| `--info` | | Show project structure information |
| `--watch` | `-w` | **NEW** Watch for file changes and rebuild automatically |
| `--src-dir` | | Source directory to watch (default: src) |
| `--debounce` | | Debounce delay in milliseconds (default: 500) |
| `--help` | `-h` | Print help information |
| `--version` | `-V` | Print version information |

## 🔄 Watch Mode

**NEW in v1.0.1**: CG Bundler now supports automatic rebuilding when source files change!

### What's New
- 🆕 **`--watch` flag**: Monitor source files and rebuild automatically  
- 🆕 **`--src-dir` option**: Specify custom directory to watch
- 🆕 **`--debounce` option**: Control rebuild timing for optimal performance
- 🛠️ **Built-in file monitoring**: No external tools needed
- 🎨 **Rich feedback**: Colored output with status indicators

### Basic Watch Usage
```bash
# Watch the current project and rebuild on changes
cg-bundler --watch -o output.rs

# Watch with verbose output to see what's happening
cg-bundler --watch -o output.rs --verbose

# Watch with faster response time
cg-bundler --watch -o output.rs --debounce 200
```

### Advanced Watch Configuration
```bash
# Watch a specific project directory
cg-bundler my-project --watch -o bundled.rs

# Watch custom source directory
cg-bundler --watch --src-dir lib -o output.rs

# Watch with minification enabled
cg-bundler --watch -o output.rs --minify

# Watch with all optimizations
cg-bundler --watch -o output.rs --pretty --keep-docs --verbose
```

### Watch Mode Features
- 🔍 **Smart File Monitoring** - Only rebuilds on `.rs` file changes
- ⚡ **Debounced Rebuilding** - Prevents rapid consecutive rebuilds
- 🎯 **Selective Watching** - Configure which directory to monitor
- 🛡️ **Error Resilience** - Continues watching even if builds fail
- 🎨 **Rich Output** - Colored status messages with emojis
- 🛑 **Graceful Shutdown** - Clean exit with Ctrl+C

### Perfect for Development Workflows
```bash
# Competitive programming: watch your solution as you code
cg-bundler contest-solution --watch -o solution.rs --minify

# Library development: watch with documentation preserved
cg-bundler my-lib --watch -o dist/my-lib.rs --keep-docs --pretty

# Real-time feedback during coding
cg-bundler --watch -o output.rs --verbose
```

Watch mode is perfect for:
- **Competitive Programming**: Instant feedback as you develop solutions
- **Rapid Prototyping**: See bundled results immediately
- **CI/CD Development**: Test bundling behavior continuously
- **Learning**: Understand how your code transforms in real-time

> 📖 **For more detailed watch mode documentation, see [WATCH_MODE.md](WATCH_MODE.md)**

## 🏗️ Project Structure

```
cg-bundler/
├── src/
│   ├── main.rs           # CLI application entry point
│   ├── lib.rs            # Library root with public API
│   ├── bundler.rs        # Main bundling orchestration
│   ├── cargo_project.rs  # Cargo project analysis
│   ├── transformer.rs    # AST transformation logic
│   ├── file_manager.rs   # File I/O operations
│   ├── error.rs          # Error types and handling
│   └── cli.rs            # CLI argument parsing
├── tests/
│   ├── integration_tests.rs  # End-to-end tests
│   └── unit_tests.rs         # Unit tests
├── test_project/         # Sample project for testing
└── target/              # Build artifacts
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test categories
cargo test integration
cargo test unit_tests

# Test with different configurations
cargo test test_bundle_contains_expected_structures
```

## 🐛 Troubleshooting

### Common Issues

#### "No binary target found"
Ensure your `Cargo.toml` has a binary target defined:
```toml
[[bin]]
name = "my_program"
path = "src/main.rs"
```

#### "Multiple binary targets found"
CG Bundler currently supports only single binary targets. Use `--target` flag if your cargo supports it, or temporarily remove extra binary targets.

#### "Module not found"
Ensure your module structure follows Rust conventions:
- `mod.rs` files for directory modules
- `.rs` files for simple modules
- Proper `mod` declarations in parent modules

#### "Parsing errors"
Make sure your code compiles successfully with `cargo check` before bundling.

### Debug Mode
Enable verbose output for detailed debugging:
```bash
cg-bundler --verbose ./my_project

# For watch mode debugging
cg-bundler --watch --verbose -o output.rs
```

### Watch Mode Troubleshooting

#### Watch not detecting changes
- Ensure you're editing files in the watched directory (default: `src/`)
- Use `--src-dir` to specify a different directory to watch
- Only `.rs` files trigger rebuilds

#### Too many rebuilds
- Increase debounce delay: `--debounce 1000` (1 second)
- Check if your editor creates temporary files in the source directory

#### Watch mode not starting
- Verify the source directory exists
- Check file permissions for the project directory
- Ensure you have the latest version with watch support

## 🤝 Contributing

We welcome contributions! Here's how to get started:

### Development Setup
```bash
# Clone the repository
git clone https://github.com/MathieuSoysal/cg-bundler.git
cd cg-bundler

# Install dependencies and build
cargo build

# Run tests
cargo test

# Check formatting and linting
cargo fmt --check
cargo clippy -- -D warnings
```

### Contributing Guidelines

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes with tests
4. **Ensure** all tests pass (`cargo test`)
5. **Format** your code (`cargo fmt`)
6. **Commit** your changes (`git commit -m 'Add amazing feature'`)
7. **Push** to the branch (`git push origin feature/amazing-feature`)
8. **Open** a Pull Request

### Code Standards
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Add tests for new functionality
- Update documentation for public APIs
- Ensure backward compatibility

### Reporting Issues
Found a bug? Have a feature request? Please [create an issue](https://github.com/MathieuSoysal/cg-bundler/issues) with:
- Clear description of the problem/feature
- Steps to reproduce (for bugs)
- Expected vs actual behavior
- Your environment (OS, Rust version, etc.)

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2024 CG Bundler Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
```

## 🙏 Acknowledgments

- The **Rust Community** for excellent tooling and ecosystem
- **Syn crate** for powerful Rust AST parsing
- **Cargo** for providing metadata APIs
- **Contributors** who help improve this project

## 📊 Project Stats

- **Language**: Rust 🦀
- **Dependencies**: Minimal, focused on core functionality
- **Performance**: Optimized for large codebases
- **Compatibility**: Works with all major Rust project structures
- **Test Coverage**: Comprehensive test suite with integration and unit tests

---

<div align="center">

**Made with ❤️ by the Rust community**

[⭐ Star us on GitHub](https://github.com/MathieuSoysal/cg-bundler) | [🐛 Report Bug](https://github.com/MathieuSoysal/cg-bundler/issues) | [💡 Request Feature](https://github.com/MathieuSoysal/cg-bundler/issues)

</div>
