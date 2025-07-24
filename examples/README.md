# CG Bundler Examples

This directory contains practical examples demonstrating various use cases for CG Bundler.

## 📁 Examples Overview

- [`basic.rs`](basic.rs) - Simple hello world example
- [`competitive-programming/`](competitive-programming/) - CodinGame/Codeforces project structure
- [`library-with-binary/`](library-with-binary/) - Project with both lib.rs and main.rs
- [`complex-modules/`](complex-modules/) - Deep module hierarchy example
- [`watch-mode/`](watch-mode/) - Live development workflow

## 🚀 Running Examples

```bash
# Basic bundling
cg-bundler examples/basic.rs

# Bundle and output to file
cg-bundler examples/competitive-programming/ -o bundled.rs

# Watch mode for development
cg-bundler examples/competitive-programming/ --watch -o output.rs
```

## 💡 Tips

- Use `--validate` to check if your project can be bundled before actual bundling
- Add `--verbose` for detailed output during bundling
- Use `--keep-docs` for educational/reference purposes
- Try `--minify` for size-optimized output
