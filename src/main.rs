use clap::Parser;
use colored::*;
use std::fs;
use std::process;

use rust_singler::{Bundler, CargoProject};
use rust_singler::cli::{Cli, Commands};
use rust_singler::error::BundlerError;

fn main() {
    let cli = Cli::parse();
    
    let result = match &cli.command {
        Commands::Bundle {
            output,
            pretty,
            verbose,
            ..
        } => handle_bundle_command(&cli.command, *verbose, *pretty, output.as_ref()),
        Commands::Validate { project_path, verbose } => {
            handle_validate_command(project_path, *verbose)
        }
        Commands::Info { project_path } => handle_info_command(project_path),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

fn handle_bundle_command(
    command: &Commands,
    verbose: bool,
    pretty: bool,
    output_file: Option<&std::path::PathBuf>,
) -> Result<(), BundlerError> {
    let project_path = command.project_path();
    let transform_config = command.get_transform_config().unwrap();

    if verbose {
        eprintln!("{} {}", "Bundling project:".green().bold(), project_path.display());
        eprintln!("Configuration:");
        eprintln!("  Remove tests: {}", transform_config.remove_tests);
        eprintln!("  Remove docs: {}", transform_config.remove_docs);
        eprintln!("  Expand modules: {}", transform_config.expand_modules);
    }

    let bundler = Bundler::with_config(transform_config);
    let mut bundled_code = bundler.bundle(project_path)?;

    // Format with rustfmt if requested and available
    if pretty {
        if verbose {
            eprintln!("{}", "Formatting with rustfmt...".yellow());
        }
        
        bundled_code = format_with_rustfmt(&bundled_code, verbose)
            .unwrap_or_else(|| {
                if verbose {
                    eprintln!("{}", "Warning: rustfmt formatting failed, using unformatted output".yellow());
                }
                bundled_code
            });
    }

    // Write output
    match output_file {
        Some(output_path) => {
            if verbose {
                eprintln!("{} {}", "Writing to file:".green(), output_path.display());
            }
            fs::write(output_path, &bundled_code)
                .map_err(|e| BundlerError::Io {
                    source: e,
                    path: Some(output_path.clone()),
                })?;
            
            if verbose {
                eprintln!("{}", "Bundle complete!".green().bold());
            }
        }
        None => {
            print!("{}", bundled_code);
        }
    }

    Ok(())
}

fn handle_validate_command(
    project_path: &std::path::PathBuf,
    verbose: bool,
) -> Result<(), BundlerError> {
    if verbose {
        eprintln!("{} {}", "Validating project:".green().bold(), project_path.display());
    }

    // Try to load the project
    let project = CargoProject::new(project_path)?;
    
    if verbose {
        eprintln!("{}", "✓ Project structure is valid".green());
        eprintln!("  Crate name: {}", project.crate_name());
        eprintln!("  Binary target: {}", project.binary_target().name);
        if let Some(lib) = project.library_target() {
            eprintln!("  Library target: {}", lib.name);
        }
    }

    // Try to bundle without writing output
    let bundler = Bundler::new();
    let _bundled_code = bundler.bundle_project(&project)?;

    if verbose {
        eprintln!("{}", "✓ Project can be bundled successfully".green());
    }

    // Try to parse the bundled code
    let bundled_code = bundler.bundle_project(&project)?;
    syn::parse_file(&bundled_code)
        .map_err(|e| BundlerError::Parsing {
            message: format!("Generated code is not valid Rust: {}", e),
            file_path: None,
        })?;

    if verbose {
        eprintln!("{}", "✓ Generated code is syntactically valid".green());
    }

    println!("{}", "✓ Project validation successful".green().bold());
    Ok(())
}

fn handle_info_command(project_path: &std::path::PathBuf) -> Result<(), BundlerError> {
    let project = CargoProject::new(project_path)?;
    
    println!("{}", "Project Information".blue().bold());
    println!("{}", "=".repeat(20));
    
    let package = project.root_package();
    println!("{}: {}", "Name".bold(), package.name);
    println!("{}: {}", "Version".bold(), package.version);
    
    if let Some(description) = &package.description {
        println!("{}: {}", "Description".bold(), description);
    }
    
    println!("{}: {}", "Manifest Path".bold(), package.manifest_path);
    println!("{}: {}", "Source Base Path".bold(), project.base_path().display());
    
    println!();
    println!("{}", "Targets".blue().bold());
    println!("{}", "-".repeat(10));
    
    let binary = project.binary_target();
    println!("{}: {} ({})", "Binary".bold(), binary.name, binary.src_path);
    
    if let Some(library) = project.library_target() {
        println!("{}: {} ({})", "Library".bold(), library.name, library.src_path);
    }
    
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

    Ok(())
}

fn format_with_rustfmt(code: &str, verbose: bool) -> Option<String> {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(if verbose { Stdio::inherit() } else { Stdio::null() })
        .spawn()
        .ok()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(code.as_bytes()).ok()?;
    }

    let output = child.wait_with_output().ok()?;
    
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}