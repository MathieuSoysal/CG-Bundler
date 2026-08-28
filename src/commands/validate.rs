//! `--validate` command: ensure the project bundles and parses cleanly.

use colored::Colorize;

use cg_bundler::{Bundler, BundlerError, CargoProject};

use crate::cli::Cli;

/// Run the validate command for the project described by `cli`.
///
/// # Errors
/// Returns any [`BundlerError`] from project loading, bundling, or
/// re-parsing the generated source.
pub fn run(cli: &Cli) -> Result<(), BundlerError> {
    let project_path = cli.get_project_path();
    let verbose = cli.is_verbose();

    if verbose {
        eprintln!(
            "{} {}",
            "Validating project:".green().bold(),
            project_path.display()
        );
    }

    let project = CargoProject::new(&project_path)?;
    if verbose {
        log_project_structure(&project);
    }

    let bundler = Bundler::with_config(cli.get_transform_config());
    let bundled_code = bundler.bundle_project(&project)?;
    if verbose {
        eprintln!("{}", "✓ Project can be bundled successfully".green());
    }

    syn::parse_file(&bundled_code).map_err(|e| BundlerError::Parsing {
        message: format!("Generated code is not valid Rust: {e}"),
        file_path: None,
    })?;

    if verbose {
        eprintln!("{}", "✓ Generated code is syntactically valid".green());
    }

    println!("{}", "✓ Project validation successful".green().bold());

    if verbose {
        print_help_footer();
    }

    Ok(())
}

fn log_project_structure(project: &CargoProject) {
    eprintln!("{}", "✓ Project structure is valid".green());
    eprintln!("  Crate name: {}", project.crate_name());
    eprintln!("  Binary target: {}", project.binary_target().name);
    if let Some(lib) = project.library_target() {
        eprintln!("  Library target: {}", lib.name);
    }
}

fn print_help_footer() {
    eprintln!();
    eprintln!("{}", "ℹ️  Need help or want to report an issue?".cyan());
    eprintln!(
        "{}",
        "   Visit: https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{make_project, make_project_with_lib};
    use clap::Parser;
    use tempfile::TempDir;

    fn cli_for(path: &std::path::Path, verbose: bool) -> Cli {
        let mut args = vec!["cg-bundler", path.to_str().unwrap(), "--validate"];
        if verbose {
            args.push("--verbose");
        }
        Cli::try_parse_from(args).unwrap()
    }

    #[test]
    fn test_handle_validate_command_valid() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        assert!(run(&cli_for(tmp.path(), false)).is_ok());
    }

    #[test]
    fn test_handle_validate_command_verbose() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        assert!(run(&cli_for(tmp.path(), true)).is_ok());
    }

    #[test]
    fn test_handle_validate_command_verbose_with_lib() {
        let tmp = TempDir::new().unwrap();
        make_project_with_lib(tmp.path());
        assert!(run(&cli_for(tmp.path(), true)).is_ok());
    }

    #[test]
    fn test_handle_validate_command_invalid() {
        let tmp = TempDir::new().unwrap();
        let bad = tmp.path().join("nonexistent");
        assert!(run(&cli_for(&bad, false)).is_err());
    }
}
