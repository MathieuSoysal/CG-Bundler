# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Items guarded by `#[cfg(not(test))]` are no longer deleted from the bundle
- `cfg` predicates that merely contain the text `test` (e.g. `feature = "fastest"`) are no longer mistaken for test markers
- Test code nested inside inline `mod { .. }` blocks is now removed
- `#[cfg(any(test, ...))]` items are no longer deleted: the predicate also holds
  outside a test build, unlike `#[cfg(all(test, ...))]`
- Test-only `use`, `const`, `static`, `type`, `union`, `macro_rules!` and
  `extern crate` items are now stripped alongside test functions and modules. A
  surviving `#[cfg(test)] use some_dep::X;` made dependency detection inline the
  whole of `some_dep` into a bundle that never used it
- Crate expansion honours a custom `[lib] path` instead of assuming `<src>/lib.rs`
- Imports of the crate's own library from a submodule are retargeted to `crate::` instead of being left dangling
- Path dependencies referenced only from a submodule are now inlined; previously the bundle referenced a crate that no longer existed
- `crate::` paths inside an inlined dependency are requalified to its new module
- Registry dependencies are no longer inlined: their sources rely on build scripts and `#[path]` modules that cannot be flattened
- Module expansion failures are reported as errors instead of being printed as warnings while exiting successfully
- `--m2` no longer deletes the comma separating a field or variant from a following attribute
- `--m2` no longer collapses `a & &b` into the logical `a && b`
- `--m2` no longer collapses `a / *b` into `/*`, which opened a block comment and
  swallowed the rest of the bundle
- `--m2` no longer collapses `x < <T as Trait>::CONST` into the shift operator `<<`
- `--m2` no longer rewrites the one-element tuple `(x,)` into `(x)`; the bundle still
  compiled but the tuple had silently become a parenthesised expression
- Minifying with `--keep-docs` no longer emits a bundle that fails to compile: doc
  comments are re-encoded as `#[doc = "..."]` attributes instead of `///` and
  `/** ... */` comments. A line comment used to comment out everything after it
  once the lines were joined, and a block comment had its interior punctuation
  rewritten -- which could move its `*/` terminator -- and string placeholders
  substituted inside it
- The library no longer prints progress messages to stderr
- `--validate` uses the requested transform options
- Paths inside a macro are no longer left un-rewritten when preceded by a single `:`,
  as in the `v:` of a struct literal; only a `::` that continues a longer path
  suppresses the rewrite now
- A leading `::`, as in `use ::dep::X` or `::dep::X`, is dropped when the path is
  retargeted; it used to survive and produce the un-parseable `::crate::dep::X`
- A module declared locally now shadows a dependency of the same name instead of
  having its own references retargeted at the inlined dependency
- A dependency whose name collides with a module defined by this crate is reported as
  an error instead of silently resolving every such path to the wrong code
- `--watch` no longer rebuilds forever when `-o` writes inside the watched directory:
  the bundle's own output is not treated as a source change
- `cg-bundler | head` no longer panics with a broken-pipe error: a consumer that
  stops reading ends the pipeline normally
- A path that does not exist is reported as such, instead of being resolved against
  a parent directory and bundling an unrelated project
- README: the download instructions were missing `chmod +x` and ran the binary via
  `bash`, which cannot execute it; all five published platform builds are now listed
- `--watch` writes status messages to stderr (keeping stdout clean) and uses a trailing debounce so a burst of edits rebuilds the final state

### Changed
- `cg-bundler` now searches parent directories for `Cargo.toml`, so it runs from
  anywhere inside a project rather than only from its root
- Pointing it at a workspace root names the members to bundle instead of reporting
  that a root package is missing
- A problem in the user's own project is no longer presented as a bug in the bundler.
  The issue-tracker link stays, but the "found a bug?" banner is now reserved for
  unexpected failures
- `--pretty` warns when `rustfmt` is unavailable instead of silently emitting
  unformatted output
- Writing the bundle to a terminal adds a one-line hint about `-o` and redirection;
  redirected and piped output is unchanged

### Added
- `CodeTransformer::with_library_path` for projects with a non-default library root
- `tests/regression_tests.rs` covering all of the above

### Removed
- Unused dependencies: `anyhow`, `thiserror`, `walkdir`, `proptest`

## [Previous Unreleased]

### Added
- Enhanced open source best practices implementation following opensource.guide
- Comprehensive security policy (SECURITY.md) with vulnerability reporting
- Code of Conduct (Contributor Covenant v2.1)
- Issue and pull request templates for better community engagement
- Dependabot configuration for automated dependency updates
- Enhanced CI/CD with multi-platform support (Windows, macOS, Linux)
- Code coverage reporting with Codecov integration
- Security audit workflow with cargo-audit and cargo-deny
- Homebrew formula and Chocolatey package distribution scripts
- Comprehensive examples directory with competitive programming samples

### Changed
- Expanded Cargo.toml metadata with better keywords, categories, and rust-version
- Enhanced README.md with installation verification, troubleshooting, and benchmarks
- Improved inline code documentation for better developer experience
- Stricter clippy lints for higher code quality
- Updated CI workflows for better security and reliability
- Enhanced CONTRIBUTING.md with development setup and guidelines
- Improved code comments and documentation coverage
- Added troubleshooting section and performance benchmarks
- Added IMPLEMENTATION.md summarizing all open source best practices

### Fixed
- Fixed code scanning alert no. 10: Added proper permissions to GitHub workflow (July 24, 2025)
- Streamlined README.md to remove outdated watch mode examples and improve CLI documentation

### Security
- Added security audit workflow with cargo-audit and cargo-deny
- Implemented comprehensive security policy (SECURITY.md) with vulnerability reporting
- Fixed GitHub workflow permissions to improve security posture (automated security fix via GitHub Advanced Security)

## [1.1.0] - 2025-07-22

### Added
- **Watch mode** (`--watch`/`-w`) for automatic rebuilding on file changes
- **Custom source directory** (`--src-dir`) option for watch mode
- **Debounce delay** (`--debounce`) configuration for file watching (default: 500ms)
- **Enhanced CLI help** with bug reporting and documentation links
- **Project validation** (`--validate`) mode to check bundling capability
- **Project information** (`--info`) command to display project metadata
- **Aggressive minification** (`--m2`) with whitespace optimization
- **Pretty printing** (`--pretty`) with rustfmt integration
- **Enhanced error messages** with emoji indicators and GitHub issue links
- **Graceful shutdown** for watch mode with Ctrl+C handling
- **File change detection** with smart filtering for Rust source files

### Changed
- **Improved CLI structure** with better command organization and help text
- **Enhanced verbose output** with detailed progress information
- **Better error handling** with contextual information and user guidance
- **Comprehensive test coverage** with CLI, integration, and unit tests
- **Code organization** improvements for better maintainability

### Fixed
- **File watching stability** with proper debouncing and error handling
- **Module resolution** improvements for complex project structures
- **Error reporting** consistency across different command modes
- **CLI flag validation** and better user feedback for invalid inputs

## [1.0.1] - 2025-07-22

### Fixed
- Fixed module resolution for nested directory structures
- Improved error messages for malformed Cargo.toml files
- Fixed CLI argument parsing edge cases

### Security
- Updated dependencies to patch security vulnerabilities
- Enhanced input validation for file paths

## [1.0.0] - 2025-07-22

### Added
- **Core bundling functionality** for Rust projects
- **Smart module expansion** with automatic dependency resolution
- **Code optimization** with test and documentation removal options
- **Minification support** for size optimization
- **CLI interface** with comprehensive options and flags
- **Library API** for programmatic usage (`bundle()` function)
- **Cargo project integration** with manifest parsing
- **Multi-target support** for both binary and library projects
- **Cross-platform compatibility** (Windows, macOS, Linux)
- **Error handling** with detailed messages and context

### Features
- **Transform configuration** with customizable options:
  - `--keep-tests`: Preserve test code in output
  - `--keep-docs`: Preserve documentation comments
  - `--no-expand-modules`: Disable module expansion
  - `--minify`: Single-line minification
  - `--verbose`: Detailed operation logging
- **Output options**: File output or stdout
- **Project validation**: Ensure bundling compatibility
- **Rust edition support**: Compatible with 2015, 2018, and 2021 editions

---

## Release Notes

### Installation

#### From Crates.io (Recommended)
```bash
cargo install cg-bundler
```

#### Pre-built Binaries
Download from [GitHub Releases](https://github.com/MathieuSoysal/cg-bundler/releases/latest):
```bash
# Linux
curl -L https://github.com/MathieuSoysal/cg-bundler/releases/latest/download/cg-bundler-linux-amd64 -o cg-bundler

# macOS
curl -L https://github.com/MathieuSoysal/cg-bundler/releases/latest/download/cg-bundler-macos-amd64 -o cg-bundler

# Windows
curl -L https://github.com/MathieuSoysal/cg-bundler/releases/latest/download/cg-bundler-windows-amd64.exe -o cg-bundler.exe
```

#### Package Managers
- **Homebrew**: `brew install cg-bundler` (pending publication)
- **Chocolatey**: `choco install cg-bundler` (pending publication)

### How to Upgrade

When upgrading between versions:
1. Check the **Changed** and **Fixed** sections for any breaking changes
2. Update using your preferred installation method
3. Test with `cg-bundler --version` to verify the new version
4. Run `cg-bundler --validate` on your projects to ensure compatibility

### Migration Guide

#### From 1.0.x to 1.1.x
- No breaking changes
- New features are opt-in through CLI flags
- All existing workflows continue to work unchanged

### Support & Community

- **Bug Reports**: [GitHub Issues](https://github.com/MathieuSoysal/cg-bundler/issues/new?template=bug_report.md)
- **Feature Requests**: [GitHub Issues](https://github.com/MathieuSoysal/cg-bundler/issues/new?template=feature_request.md)
- **Documentation**: [docs.rs/cg-bundler](https://docs.rs/cg-bundler)
- **Discussions**: [GitHub Discussions](https://github.com/MathieuSoysal/cg-bundler/discussions)
- **Security**: See [SECURITY.md](SECURITY.md) for vulnerability reporting

### Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup
- Code style guidelines  
- Testing requirements
- Pull request process

---

[Unreleased]: https://github.com/MathieuSoysal/cg-bundler/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/MathieuSoysal/cg-bundler/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/MathieuSoysal/cg-bundler/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/MathieuSoysal/cg-bundler/releases/tag/v1.0.0
