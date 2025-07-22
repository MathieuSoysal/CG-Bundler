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

## 📋 Requirements

- **Rust 1.75.0** or later
- **Cargo** (comes with Rust)
- Compatible with **all Rust editions** (2015, 2018, 2021)

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

---

<div align="center">

**Made with ❤️ by the Rust community**

[⭐ Star us on GitHub](https://github.com/MathieuSoysal/cg-bundler) | [🐛 Report Bug](https://github.com/MathieuSoysal/cg-bundler/issues) | [💡 Request Feature](https://github.com/MathieuSoysal/cg-bundler/issues)

</div>
