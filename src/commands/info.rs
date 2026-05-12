//! `--info` command: print a high-level summary of the Cargo project.

use std::path::Path;

use cargo_metadata::Package;
use colored::Colorize;

use cg_bundler::{BundlerError, CargoProject};

/// Run the info command for the project at `project_path`.
///
/// # Errors
/// Returns any [`BundlerError`] raised while loading the Cargo project.
pub fn run(project_path: &Path) -> Result<(), BundlerError> {
    let project = CargoProject::new(project_path)?;
    let package = project.root_package();

    print_header();
    print_package_info(package, &project);
    print_targets(&project);
    print_dependencies(package);
    print_help_footer();

    Ok(())
}

fn print_header() {
    println!("{}", "Project Information".blue().bold());
    println!("{}", "=".repeat(20));
}

fn print_package_info(package: &Package, project: &CargoProject) {
    println!("{}: {}", "Name".bold(), package.name);
    println!("{}: {}", "Version".bold(), package.version);

    if let Some(description) = &package.description {
        println!("{}: {}", "Description".bold(), description);
    }

    println!("{}: {}", "Manifest Path".bold(), package.manifest_path);
    println!(
        "{}: {}",
        "Source Base Path".bold(),
        project.base_path().display()
    );
}

fn print_targets(project: &CargoProject) {
    println!();
    println!("{}", "Targets".blue().bold());
    println!("{}", "-".repeat(10));

    let binary = project.binary_target();
    println!("{}: {} ({})", "Binary".bold(), binary.name, binary.src_path);

    if let Some(library) = project.library_target() {
        println!(
            "{}: {} ({})",
            "Library".bold(),
            library.name,
            library.src_path
        );
    }
}

fn print_dependencies(package: &Package) {
    println!();
    println!("{}", "Dependencies".blue().bold());
    println!("{}", "-".repeat(15));

    if package.dependencies.is_empty() {
        println!("No dependencies");
    } else {
        for dep in &package.dependencies {
            println!("  {} {}", dep.name, dep.req);
        }
    }
}

fn print_help_footer() {
    println!();
    println!("{}", "━".repeat(50).bright_blue());
    println!(
        "{}",
        "ℹ️  Need help or want to report an issue?".cyan().bold()
    );
    println!(
        "{}",
        "   🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue()
    );
    println!("{}", "━".repeat(50).bright_blue());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{make_project, make_project_with_lib};
    use tempfile::TempDir;

    #[test]
    fn test_handle_info_command_valid() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        assert!(run(tmp.path()).is_ok());
    }

    #[test]
    fn test_handle_info_command_with_lib_and_description() {
        let tmp = TempDir::new().unwrap();
        make_project_with_lib(tmp.path());
        assert!(run(tmp.path()).is_ok());
    }

    #[test]
    fn test_handle_info_command_invalid() {
        let tmp = TempDir::new().unwrap();
        let bad = tmp.path().join("nonexistent");
        assert!(run(&bad).is_err());
    }

    #[test]
    fn test_handle_info_command_with_dependencies() {
        let current_dir = std::env::current_dir().unwrap();
        if current_dir.join("Cargo.toml").exists() {
            assert!(run(&current_dir).is_ok());
        }
    }
}
