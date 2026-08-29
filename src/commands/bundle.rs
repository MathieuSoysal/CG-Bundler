//! Default `bundle` command: produce a single-file Rust source from a Cargo project.

use std::fs;
use std::io::{self, IsTerminal as _, Write as _};
use std::path::Path;

use colored::Colorize;

use cg_bundler::{Bundler, BundlerError, TransformConfig};

use crate::cli::Cli;
use crate::minify::{aggressive_minify_code, minify_code};
use crate::rustfmt_fmt::format_with_rustfmt;

/// Run the bundle command described by `cli`.
///
/// # Errors
/// Returns any [`BundlerError`] raised by bundling, post-processing, or
/// writing the output file.
pub fn run(cli: &Cli) -> Result<(), BundlerError> {
    let project_path = cli.get_project_path();
    let transform_config = cli.get_transform_config();
    let verbose = cli.is_verbose();

    if verbose {
        log_configuration(&project_path, &transform_config);
    }

    let bundler = Bundler::with_config(transform_config);
    let raw_output = bundler.bundle(&project_path)?;
    let processed = post_process(raw_output, cli);

    write_output(&processed, cli.get_output(), verbose)
}

fn log_configuration(project_path: &Path, config: &TransformConfig) {
    eprintln!(
        "{} {}",
        "Bundling project:".green().bold(),
        project_path.display()
    );
    eprintln!("Configuration:");
    eprintln!("  Remove tests: {}", config.remove_tests);
    eprintln!("  Remove docs: {}", config.remove_docs);
    eprintln!("  Expand modules: {}", config.expand_modules);
    eprintln!("  Minify: {}", config.minify);
    eprintln!("  Aggressive minify: {}", config.aggressive_minify);
}

fn post_process(code: String, cli: &Cli) -> String {
    let verbose = cli.is_verbose();

    if cli.is_aggressive_minify() {
        if verbose {
            eprintln!("{}", "Applying aggressive minification...".yellow());
        }
        return aggressive_minify_code(&code);
    }

    if cli.is_minify() {
        if verbose {
            eprintln!("{}", "Minifying output to single line...".yellow());
        }
        return minify_code(&code);
    }

    if cli.is_pretty() {
        if verbose {
            eprintln!("{}", "Formatting with rustfmt...".yellow());
        }
        return format_with_rustfmt(&code, verbose).unwrap_or_else(|| {
            // Silently ignoring --pretty makes the flag look broken; say so once.
            eprintln!(
                "{}",
                "Warning: --pretty needs rustfmt, which is unavailable or failed here; emitting unformatted output. Install it with `rustup component add rustfmt`."
                    .yellow()
            );
            code
        });
    }

    code
}

fn write_output(
    code: &str,
    output: Option<&std::path::PathBuf>,
    verbose: bool,
) -> Result<(), BundlerError> {
    let Some(path) = output else {
        write_stdout(code)?;
        if io::stdout().is_terminal() {
            eprintln!(
                "{}",
                "Tip: that was the bundle on stdout. Use `-o <FILE>` to save it -- a redirect such as `> out.rs` is emptied before bundling, so a failed run destroys the previous one."
                    .yellow()
            );
        }
        return Ok(());
    };

    if verbose {
        eprintln!("{} {}", "Writing to file:".green(), path.display());
    }
    fs::write(path, code).map_err(|e| BundlerError::Io {
        source: e,
        path: Some(path.clone()),
    })?;
    if verbose {
        print_completion_banner();
    }

    Ok(())
}

/// Write the bundle to stdout.
///
/// A consumer that stops reading -- `cg-bundler | head`, or quitting `less`
/// early -- closes the pipe, which is an ordinary way to end a shell pipeline
/// rather than a failure. `print!` turns that into a panic, so write explicitly
/// and treat a broken pipe as success.
fn write_stdout(code: &str) -> Result<(), BundlerError> {
    let mut stdout = io::stdout().lock();

    match stdout
        .write_all(code.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(BundlerError::Io {
            source: e,
            path: None,
        }),
        Ok(()) => Ok(()),
    }
}

fn print_completion_banner() {
    eprintln!("{}", "Bundle complete!".green().bold());
    eprintln!();
    eprintln!("{}", "ℹ️  Issues or feedback? Visit:".cyan());
    eprintln!(
        "{}",
        "   🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::make_project;
    use clap::Parser;
    use tempfile::TempDir;

    #[test]
    fn test_handle_bundle_command_basic() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() { println!(\"hello\"); }");

        let cli = Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap()]).unwrap();
        assert!(run(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_minify() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");

        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--minify"]).unwrap();
        assert!(run(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_m2() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() { let x: i32 = -1; let _ = x < -1; }");

        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--m2"]).unwrap();
        assert!(run(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_verbose() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");

        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--verbose"]).unwrap();
        assert!(run(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_to_file() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let out = tmp.path().join("out.rs");

        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .unwrap();
        assert!(run(&cli).is_ok());
        assert!(out.exists());
    }

    #[test]
    fn test_handle_bundle_command_to_file_verbose() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let out = tmp.path().join("out.rs");

        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--verbose",
            "-o",
            out.to_str().unwrap(),
        ])
        .unwrap();
        assert!(run(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_pretty_verbose_invalid_code() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--pretty",
            "--verbose",
        ])
        .unwrap();
        let _ = run(&cli);
    }

    #[test]
    fn test_handle_bundle_command_invalid_path() {
        let cli = Cli::try_parse_from(["cg-bundler", "/nonexistent/path/xyz"]).unwrap();
        assert!(run(&cli).is_err());
    }

    #[test]
    fn test_handle_bundle_command_pretty() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");

        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--pretty"]).unwrap();
        assert!(run(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_m2_verbose() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() { let x: i32 = -1; let _ = x < -1; }");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--m2",
            "--verbose",
        ])
        .unwrap();
        assert!(run(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_minify_verbose() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--minify",
            "--verbose",
        ])
        .unwrap();
        assert!(run(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_write_to_directory_fails() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "-o",
            tmp.path().to_str().unwrap(),
        ])
        .unwrap();
        let result = run(&cli);
        assert!(result.is_err());
        match result.unwrap_err() {
            BundlerError::Io { .. } => {}
            e => panic!("Expected Io error from failed fs::write, got: {e}"),
        }
    }
}
