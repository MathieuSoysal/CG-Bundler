# Rust Singler - Code Compression Tool

## Project Overview
This project builds a Rust program that compresses entire Rust codebases into single-line format by removing whitespace, newlines, comments, and test code while maintaining valid Rust syntax.

## Core Requirements

### Input/Output Specifications
- **Input**: Process entire `src/` directories and `.rs` files
- **Output**: Single-line minified Rust code (similar to cargo-minify)
- **Compression**: Remove all unnecessary whitespace and newlines while preserving code validity

### Code Preservation Rules
- ✅ **Preserve**: Function names, variable names, string literals and their formatting
- ❌ **Remove**: All comments (`//`, `/* */`), documentation comments (`///`, `//!`), test code, benchmark code, example code in doc comments
- ❌ **Exclude**: Conditional compilation attributes like `#[cfg(test)]`, macro definitions

### Error Handling Requirements
- Handle files that cannot be read gracefully
- Do not handle invalid Rust syntax (assume input is valid)
- No special handling for very large files required

### Technical Stack Preferences
- **CLI Framework**: Use `clap` for command-line interface
- **Rust Parsing**: Use `syn` and `proc-macro2` for AST parsing and manipulation
- **File Handling**: Implement robust file system operations
- **Architecture**: Object-Oriented Programming (OOP) patterns preferred
- **Performance**: Prioritize performance optimizations

### Code Style Guidelines
- Use proper Result types for error handling (avoid unwrap in production code)
- Organize code using modules, traits, and structs following OOP principles
- Implement performance-conscious algorithms and data structures
- Write clean, maintainable code with clear separation of concerns

### Test-Driven Development (TDD) Guidelines
- **Red-Green-Refactor**: Write failing tests first, implement minimal code to pass, then refactor
- **Test Coverage**: Ensure comprehensive test coverage for all public methods and functions
- **Unit Tests**: Write focused unit tests for individual components and functions
- **Integration Tests**: Create integration tests for end-to-end functionality
- **Test Organization**: Place unit tests in `#[cfg(test)]` modules within source files
- **Integration Tests**: Place integration tests in `tests/` directory
- **Test Data**: Use realistic test cases with actual Rust code snippets
- **Mocking**: Use traits and dependency injection to enable proper testing
- **Property-Based Testing**: Consider using `proptest` for complex parsing scenarios
- **Performance Tests**: Include benchmarks for performance-critical operations

### Key Features to Implement
1. **File Discovery**: Recursively find all `.rs` files in `src/` directories
2. **AST Processing**: Parse Rust code and remove unwanted elements
3. **Code Minification**: Compress valid code into single-line format
4. **CLI Interface**: Provide user-friendly command-line options
5. **Error Reporting**: Clear error messages for file access issues

### Example Usage Pattern
```bash
rust-singler --input ./src --output compressed.rs
rust-singler --file main.rs --output minified.rs
```

## Development Notes
- Focus on correctness: ensure output compiles and runs identically to input
- Maintain string literal integrity and semantic meaning
- Exclude all testing infrastructure and documentation
- Optimize for processing speed and memory efficiency