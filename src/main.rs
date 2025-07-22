use clap::Parser;
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process;

use cg_bundler::{Bundler, BundlerError, CargoProject, TransformConfig};

/// Display bug report information to the user
fn display_bug_report_info() {
    eprintln!();
    eprintln!("{}", "━".repeat(60).bright_yellow());
    eprintln!("{}", "💡 Need help or found a bug?".bright_yellow().bold());
    eprintln!();
    eprintln!("{}", "  Please report issues, request features, or get support at:".yellow());
    eprintln!("{}", "  🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue().bold());
    eprintln!();
    eprintln!("{}", "  Your feedback helps improve CG-Bundler for everyone!".yellow());
    eprintln!("{}", "━".repeat(60).bright_yellow());
}

/// A Rust code bundler that combines multiple source files into a single file
#[derive(Parser, Debug)]
#[command(name = "cg-bundler")]
#[command(about = "Bundle Rust projects into single files")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "CG Bundler Contributors")]
#[command(
    long_about = "A Rust code bundler that combines multiple source files into a single file.\nBy default, bundles the current directory or the specified project path.\n\n🐛 Found a bug or need help?\n   Report issues: https://github.com/MathieuSoysal/CG-Bundler/issues/new\n\n📖 Documentation:\n   https://docs.rs/cg-bundler"
)]
pub struct Cli {
    /// Path to the Cargo project directory (defaults to current directory)
    #[arg(
        value_name = "PROJECT_PATH",
        help = "Path to bundle (defaults to current directory)"
    )]
    pub project_path: Option<PathBuf>,

    /// Output file path (stdout if not specified)
    #[arg(short, long, value_name = "FILE", help = "Output file path")]
    pub output: Option<PathBuf>,

    /// Keep test code in the bundled output
    #[arg(long, help = "Keep test code in the bundled output")]
    pub keep_tests: bool,

    /// Keep documentation comments in the bundled output
    #[arg(long, help = "Keep documentation comments")]
    pub keep_docs: bool,

    /// Disable module expansion (keep module declarations)
    #[arg(long, help = "Disable module expansion")]
    pub no_expand_modules: bool,

    /// Pretty print the output (format with rustfmt if available)
    #[arg(long, help = "Pretty print the output")]
    pub pretty: bool,

    /// Minify the output to a single line
    #[arg(short, long, help = "Minify the output")]
    pub minify: bool,

    /// Aggressive minify with whitespace replacements (implies -m)
    #[arg(long, help = "Aggressive minify")]
    pub m2: bool,

    /// Verbose output
    #[arg(short, long, help = "Verbose output")]
    pub verbose: bool,

    /// Validate that the project can be bundled without errors (instead of bundling)
    #[arg(long, help = "Validate that the project can be bundled without errors")]
    pub validate: bool,

    /// Show information about the Cargo project structure (instead of bundling)
    #[arg(long, help = "Show information about the Cargo project structure")]
    pub info: bool,
}

impl Cli {
    /// Get the effective project path, using current directory as default
    pub fn get_project_path(&self) -> PathBuf {
        self.project_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Get transform configuration from the CLI flags
    pub fn get_transform_config(&self) -> TransformConfig {
        TransformConfig {
            remove_tests: !self.keep_tests,
            remove_docs: !self.keep_docs,
            expand_modules: !self.no_expand_modules,
            minify: self.minify || self.m2,
            aggressive_minify: self.m2,
        }
    }

    /// Get the output file path
    pub fn get_output(&self) -> Option<&PathBuf> {
        self.output.as_ref()
    }

    /// Check if pretty formatting is requested
    pub fn is_pretty(&self) -> bool {
        self.pretty
    }

    /// Check if minification is requested
    pub fn is_minify(&self) -> bool {
        self.minify || self.m2
    }

    /// Check if aggressive minification is requested
    pub fn is_aggressive_minify(&self) -> bool {
        self.m2
    }
}

fn main() {
    let cli = Cli::parse();

    // Handle the different operations based on flags
    let result = if cli.validate {
        handle_validate_command(&cli.get_project_path(), cli.is_verbose())
    } else if cli.info {
        handle_info_command(&cli.get_project_path())
    } else {
        // Default behavior: bundle the project
        handle_bundle_command(&cli)
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        display_bug_report_info();
        process::exit(1);
    }
}

fn handle_bundle_command(cli: &Cli) -> Result<(), BundlerError> {
    let project_path = cli.get_project_path();
    let transform_config = cli.get_transform_config();
    let verbose = cli.is_verbose();
    let pretty = cli.is_pretty();
    let minify = cli.is_minify();
    let aggressive_minify = cli.is_aggressive_minify();
    let output_file = cli.get_output();

    if verbose {
        eprintln!(
            "{} {}",
            "Bundling project:".green().bold(),
            project_path.display()
        );
        eprintln!("Configuration:");
        eprintln!("  Remove tests: {}", transform_config.remove_tests);
        eprintln!("  Remove docs: {}", transform_config.remove_docs);
        eprintln!("  Expand modules: {}", transform_config.expand_modules);
        eprintln!("  Minify: {}", transform_config.minify);
        eprintln!(
            "  Aggressive minify: {}",
            transform_config.aggressive_minify
        );
    }

    let bundler = Bundler::with_config(transform_config);
    let mut bundled_code = bundler.bundle(&project_path)?;

    // Apply minification if requested
    if aggressive_minify {
        if verbose {
            eprintln!("{}", "Applying aggressive minification...".yellow());
        }
        bundled_code = aggressive_minify_code(&bundled_code);
    } else if minify {
        if verbose {
            eprintln!("{}", "Minifying output to single line...".yellow());
        }
        bundled_code = minify_code(&bundled_code);
    }
    // Format with rustfmt if requested and available (only if not minifying)
    else if pretty {
        if verbose {
            eprintln!("{}", "Formatting with rustfmt...".yellow());
        }

        bundled_code = format_with_rustfmt(&bundled_code, verbose).unwrap_or_else(|| {
            if verbose {
                eprintln!(
                    "{}",
                    "Warning: rustfmt formatting failed, using unformatted output".yellow()
                );
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
            fs::write(output_path, &bundled_code).map_err(|e| BundlerError::Io {
                source: e,
                path: Some(output_path.clone()),
            })?;

            if verbose {
                eprintln!("{}", "Bundle complete!".green().bold());
                eprintln!();
                eprintln!("{}", "ℹ️  Issues or feedback? Visit:".cyan());
                eprintln!("{}", "   🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue());
            }
        }
        None => {
            print!("{bundled_code}");
        }
    }

    Ok(())
}

fn handle_validate_command(
    project_path: &std::path::PathBuf,
    verbose: bool,
) -> Result<(), BundlerError> {
    if verbose {
        eprintln!(
            "{} {}",
            "Validating project:".green().bold(),
            project_path.display()
        );
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
    syn::parse_file(&bundled_code).map_err(|e| BundlerError::Parsing {
        message: format!("Generated code is not valid Rust: {e}"),
        file_path: None,
    })?;

    if verbose {
        eprintln!("{}", "✓ Generated code is syntactically valid".green());
    }

    println!("{}", "✓ Project validation successful".green().bold());
    
    if verbose {
        eprintln!();
        eprintln!("{}", "ℹ️  Need help or want to report an issue?".cyan());
        eprintln!("{}", "   Visit: https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue());
    }
    
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
    println!(
        "{}: {}",
        "Source Base Path".bold(),
        project.base_path().display()
    );

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

    println!();
    println!("{}", "━".repeat(50).bright_blue());
    println!("{}", "ℹ️  Need help or want to report an issue?".cyan().bold());
    println!("{}", "   🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue());
    println!("{}", "━".repeat(50).bright_blue());

    Ok(())
}

fn format_with_rustfmt(code: &str, verbose: bool) -> Option<String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(if verbose {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
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

fn minify_code(code: &str) -> String {
    code.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}

fn aggressive_minify_code(code: &str) -> String {
    // First apply basic minification
    let mut result = minify_code(code);

    // Parse string literals to preserve them during aggressive minification
    let mut string_literals = Vec::new();
    let mut placeholder_index = 0;

    // Extract string literals and replace with placeholders
    let mut chars = result.chars().peekable();
    let mut output = String::new();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            // Start of string literal
            let mut string_literal = String::from('"');
            let mut escaped = false;

            for str_ch in chars.by_ref() {
                string_literal.push(str_ch);
                if str_ch == '\\' && !escaped {
                    escaped = true;
                } else if str_ch == '"' && !escaped {
                    break;
                } else {
                    escaped = false;
                }
            }

            // Store the string literal and use a placeholder
            let placeholder = format!("__STRING_LITERAL_{placeholder_index}__");
            string_literals.push(string_literal);
            output.push_str(&placeholder);
            placeholder_index += 1;
        } else {
            output.push(ch);
        }
    }

    // Apply aggressive replacements to the code without string literals
    result = output
        // Remove spaces around operators and punctuation
        .replace(" = ", "=")
        .replace(" + ", "+")
        .replace(" - ", "-")
        .replace(" * ", "*")
        .replace(" / ", "/")
        .replace(" % ", "%")
        .replace(" & ", "&")
        .replace(" | ", "|")
        .replace(" ^ ", "^")
        .replace(" < ", "<")
        .replace(" > ", ">")
        .replace(" == ", "==")
        .replace(" != ", "!=")
        .replace(" <= ", "<=")
        .replace(" >= ", ">=")
        .replace(" && ", "&&")
        .replace(" || ", "||")
        .replace(" -> ", "->")
        .replace(" => ", "=>")
        // Remove spaces around punctuation
        .replace(" , ", ",")
        .replace(" ; ", ";")
        .replace(" : ", ":")
        .replace(" :: ", "::")
        .replace(" . ", ".")
        // Remove spaces around brackets and parentheses
        .replace(" ( ", "(")
        .replace(" ) ", ")")
        .replace(" [ ", "[")
        .replace(" ] ", "]")
        .replace(" { ", "{")
        .replace(" } ", "}")
        // Remove spaces before punctuation
        .replace(" ,", ",")
        .replace(" ;", ";")
        .replace(" :", ":")
        .replace(" .", ".")
        .replace(" (", "(")
        .replace(" )", ")")
        .replace(" [", "[")
        .replace(" ]", "]")
        .replace(" {", "{")
        .replace(" }", "}")
        // Remove spaces after punctuation
        .replace(", ", ",")
        .replace("; ", ";")
        .replace("( ", "(")
        .replace("[ ", "[")
        .replace("{ ", "{");

    // Restore string literals
    for (i, string_literal) in string_literals.into_iter().enumerate() {
        let placeholder = format!("__STRING_LITERAL_{i}__");
        result = result.replace(&placeholder, &string_literal);
    }

    // Final cleanup: remove any remaining multiple spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    result
}
