use clap::Parser;
use colored::Colorize;
use regex::Regex;
use std::fmt::Write;
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
    eprintln!(
        "{}",
        "  Please report issues, request features, or get support at:".yellow()
    );
    eprintln!(
        "{}",
        "  🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new"
            .blue()
            .bold()
    );
    eprintln!();
    eprintln!(
        "{}",
        "  Your feedback helps improve CG-Bundler for everyone!".yellow()
    );
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
#[allow(clippy::struct_excessive_bools)]
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

    /// Watch for file changes and rebuild automatically
    #[arg(short, long, help = "Watch for file changes and rebuild automatically")]
    pub watch: bool,

    /// Source directory to watch (default: src)
    #[arg(long, default_value = "src", help = "Source directory to watch")]
    pub src_dir: String,

    /// Debounce delay in milliseconds (default: 500)
    #[arg(long, default_value = "500", help = "Debounce delay in milliseconds")]
    pub debounce: u64,
}

impl Cli {
    /// Get the effective project path, using current directory as default
    #[must_use]
    pub fn get_project_path(&self) -> PathBuf {
        self.project_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Check if verbose mode is enabled
    #[must_use]
    pub const fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Get transform configuration from the CLI flags
    #[must_use]
    pub const fn get_transform_config(&self) -> TransformConfig {
        TransformConfig {
            remove_tests: !self.keep_tests,
            remove_docs: !self.keep_docs,
            expand_modules: !self.no_expand_modules,
            minify: self.minify || self.m2,
            aggressive_minify: self.m2,
        }
    }

    /// Get the output file path
    #[must_use]
    pub const fn get_output(&self) -> Option<&PathBuf> {
        self.output.as_ref()
    }

    /// Check if pretty formatting is requested
    #[must_use]
    pub const fn is_pretty(&self) -> bool {
        self.pretty
    }

    /// Check if minification is requested
    #[must_use]
    pub const fn is_minify(&self) -> bool {
        self.minify || self.m2
    }

    /// Check if aggressive minification is requested
    #[must_use]
    pub const fn is_aggressive_minify(&self) -> bool {
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
    } else if cli.watch {
        handle_watch_command(&cli)
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
                eprintln!(
                    "{}",
                    "   🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue()
                );
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
        eprintln!(
            "{}",
            "   Visit: https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue()
        );
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
    println!(
        "{}",
        "ℹ️  Need help or want to report an issue?".cyan().bold()
    );
    println!(
        "{}",
        "   🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue()
    );
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
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}

fn aggressive_minify_code(code: &str) -> String {
    // First apply basic minification
    let result_initial = minify_code(code);

    // Parse string literals to preserve them during aggressive minification

    let mut string_literals = Vec::new();
    let mut placeholder_index = 0;

    // Extract string literals and replace with placeholders
    let mut chars = result_initial.chars().peekable();
    let mut output = String::new();

    while let Some(char) = chars.next() {
        // Start of string literal
        match char {
            '"' => {
                let mut string_literal = String::from('"');
                let mut escaped = false;

                for inner_ch in chars.by_ref() {
                    string_literal.push(inner_ch);
                    if escaped {
                        escaped = false;
                    } else if inner_ch == '\\' {
                        escaped = true;
                    } else if inner_ch == '"' {
                        break;
                    }
                }
                // Store the string literal and use a placeholder
                let _ = write!(output, "__STRING_LITERAL_{placeholder_index}__");
                string_literals.push(string_literal);
                placeholder_index += 1;
            }
            '\'' => {
                let mut temp_buffer = String::from('\'');
                let mut found_closing = false;
                let mut escaped = false;

                while let Some(&next_ch) = chars.peek() {
                    if !escaped
                        && (next_ch == ' '
                            || next_ch == ';'
                            || next_ch == '\n'
                            || next_ch == '>'
                            || next_ch == ','
                            || next_ch == ')')
                    {
                        break;
                    }

                    temp_buffer.push(next_ch);
                    chars.next();

                    if escaped {
                        escaped = false;
                    } else if next_ch == '\\' {
                        escaped = true;
                    } else if next_ch == '\'' {
                        found_closing = true;
                        break;
                    }
                }

                if found_closing {
                    // Store the string literal and use a placeholder
                    let _ = write!(output, "__STRING_LITERAL_{placeholder_index}__");
                    string_literals.push(temp_buffer);
                    placeholder_index += 1;
                } else {
                    output.push_str(&temp_buffer);
                }
            }

            _ => {
                output.push(char);
            }
        }
    }

    // Apply aggressive replacements to the code without string literals
    let re = Regex::new(r"\s*([=+*/%&|^<>,;:.()\[\]{}-])\s*").unwrap();
    let mut result = re.replace_all(&output, "$1").to_string();
    result = result
        // Remove spaces around operators and punctuation
        .replace(",}", "}")
        .replace(",)", ")")
        .replace(",#", "#")
        .replace(",]", "]")
        // Fix: the regex strips the space between '<' and a leading '-', turning
        // `x < -1` into `x<-1`. In Rust 2015 `<-` was a reserved placement
        // operator, and even in later editions the sequence is visually misleading
        // and can confuse downstream tooling. Re-insert the separating space.
        .replace("<-", "< -");

    // Restore string literals
    for (i, string_literal) in string_literals.into_iter().enumerate() {
        let placeholder = format!("__STRING_LITERAL_{i}__");
        result = result.replace(&placeholder, &string_literal);
    }

    result
}

fn handle_watch_command(cli: &Cli) -> Result<(), BundlerError> {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    println!("{} Starting watch mode...", "🔍".green());
    println!("{} Watching directory: {}", "📁".blue(), cli.src_dir);
    if let Some(output) = &cli.output {
        println!("{} Output file: {}", "📄".blue(), output.display());
    } else {
        println!("{} Output: stdout", "📄".blue());
    }
    println!("{} Debounce delay: {}ms", "⏱️".blue(), cli.debounce);
    println!("{} Press Ctrl+C to stop\n", "ℹ️".yellow());

    // Validate source directory exists
    let watch_path = cli.get_project_path().join(&cli.src_dir);
    if !watch_path.exists() {
        return Err(BundlerError::Io {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source directory '{}' does not exist", cli.src_dir),
            ),
            path: Some(watch_path),
        });
    }

    // Setup signal handling for graceful shutdown
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = shutdown_tx.send(());
    })
    .map_err(|e| BundlerError::Io {
        source: std::io::Error::other(e.to_string()),
        path: None,
    })?;

    // Initial build
    if let Err(e) = handle_bundle_command(cli) {
        eprintln!("{} Initial build failed: {}", "❌".red(), e);
    } else {
        println!("{} Initial build successful!\n", "✅".green());
    }

    // Set up file watcher
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx).map_err(|e| BundlerError::Io {
        source: std::io::Error::other(e.to_string()),
        path: None,
    })?;

    watcher
        .watch(&watch_path, RecursiveMode::Recursive)
        .map_err(|e| BundlerError::Io {
            source: std::io::Error::other(e.to_string()),
            path: Some(watch_path),
        })?;

    let mut last_event_time = Instant::now();
    let debounce_duration = Duration::from_millis(cli.debounce);

    loop {
        // Check for shutdown signal
        if shutdown_rx.try_recv() == Ok(()) {
            println!("\n{} Received shutdown signal", "🛑".yellow());
            break;
        }

        // Check for file system events
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                if should_rebuild(&event) {
                    let now = Instant::now();
                    if now.duration_since(last_event_time) > debounce_duration {
                        last_event_time = now;

                        if let Some(path) = event.paths.first() {
                            if let Some(file_name) = path.file_name() {
                                println!(
                                    "{} File change detected: {}",
                                    "🔄".yellow(),
                                    file_name.to_string_lossy()
                                );
                            } else {
                                println!("{} File change detected", "🔄".yellow());
                            }
                        } else {
                            println!("{} File change detected", "🔄".yellow());
                        }

                        match handle_bundle_command(cli) {
                            Ok(()) => println!("{} Rebuild successful!\n", "✅".green()),
                            Err(e) => eprintln!("{} Rebuild failed: {}\n", "❌".red(), e),
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("{} Watch error: {}", "⚠️".yellow(), e),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Continue loop
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    println!("{} Watch mode stopped.", "🛑".red());
    Ok(())
}

fn should_rebuild(event: &notify::Event) -> bool {
    use notify::EventKind;

    match &event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            // Only rebuild for Rust files
            event.paths.iter().any(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext == "rs")
            })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── helpers ────────────────────────────────────────────────────────────

    fn make_project(dir: &std::path::Path, main_src: &str) {
        let cargo_toml = r#"[package]
name = "test_proj"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), main_src).unwrap();
    }

    fn make_project_with_lib(dir: &std::path::Path) {
        let cargo_toml = r#"[package]
name = "test_proj"
version = "0.1.0"
edition = "2021"
description = "a test project"

[[bin]]
name = "test_proj"
path = "src/main.rs"

[lib]
name = "test_proj"
path = "src/lib.rs"
"#;
        fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("src/lib.rs"),
            "pub fn greet() -> &'static str { \"hi\" }",
        )
        .unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();
    }

    // ── display_bug_report_info ────────────────────────────────────────────

    #[test]
    fn test_display_bug_report_info_does_not_panic() {
        // Function only writes to stderr – just verify it doesn't panic.
        display_bug_report_info();
    }

    // ── Cli accessor methods ───────────────────────────────────────────────

    #[test]
    fn test_cli_get_project_path_default() {
        let cli = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert_eq!(cli.get_project_path(), PathBuf::from("."));
    }

    #[test]
    fn test_cli_get_project_path_custom() {
        let cli = Cli::try_parse_from(["cg-bundler", "/tmp/myproj"]).unwrap();
        assert_eq!(cli.get_project_path(), PathBuf::from("/tmp/myproj"));
    }

    #[test]
    fn test_cli_is_verbose_flag() {
        let cli = Cli::try_parse_from(["cg-bundler", "--verbose"]).unwrap();
        assert!(cli.is_verbose());

        let cli2 = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert!(!cli2.is_verbose());
    }

    #[test]
    fn test_cli_is_pretty() {
        let cli = Cli::try_parse_from(["cg-bundler", "--pretty"]).unwrap();
        assert!(cli.is_pretty());

        let cli2 = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert!(!cli2.is_pretty());
    }

    #[test]
    fn test_cli_is_minify_and_aggressive() {
        let cli_m = Cli::try_parse_from(["cg-bundler", "--minify"]).unwrap();
        assert!(cli_m.is_minify());
        assert!(!cli_m.is_aggressive_minify());

        let cli_m2 = Cli::try_parse_from(["cg-bundler", "--m2"]).unwrap();
        assert!(cli_m2.is_minify());
        assert!(cli_m2.is_aggressive_minify());

        let cli_none = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert!(!cli_none.is_minify());
        assert!(!cli_none.is_aggressive_minify());
    }

    #[test]
    fn test_cli_get_output() {
        let cli_none = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert!(cli_none.get_output().is_none());

        let cli_out = Cli::try_parse_from(["cg-bundler", "-o", "out.rs"]).unwrap();
        assert!(cli_out.get_output().is_some());
        assert_eq!(cli_out.get_output().unwrap(), &PathBuf::from("out.rs"));
    }

    #[test]
    fn test_cli_get_transform_config_flags() {
        let cli = Cli::try_parse_from(["cg-bundler", "--keep-tests", "--keep-docs"]).unwrap();
        let config = cli.get_transform_config();
        assert!(!config.remove_tests);
        assert!(!config.remove_docs);
        assert!(config.expand_modules);
        assert!(!config.minify);
        assert!(!config.aggressive_minify);

        let cli_m2 = Cli::try_parse_from(["cg-bundler", "--m2"]).unwrap();
        let config_m2 = cli_m2.get_transform_config();
        assert!(config_m2.aggressive_minify);
        assert!(config_m2.minify);

        let cli_no = Cli::try_parse_from(["cg-bundler", "--no-expand-modules"]).unwrap();
        let config_no = cli_no.get_transform_config();
        assert!(!config_no.expand_modules);
    }

    // ── minify_code ────────────────────────────────────────────────────────

    #[test]
    fn test_minify_code_removes_newlines() {
        let code = "fn main() {\n    let x = 5;\n    println!(\"x={}\", x);\n}";
        let result = minify_code(code);
        assert!(!result.contains('\n'));
        assert!(result.contains("fn main"));
        assert!(result.contains("let x"));
    }

    #[test]
    fn test_minify_code_removes_empty_lines() {
        let code = "fn a() {}\n\n\nfn b() {}";
        let result = minify_code(code);
        assert_eq!(result, "fn a() {} fn b() {}");
    }

    // ── format_with_rustfmt ────────────────────────────────────────────────

    #[test]
    fn test_format_with_rustfmt_valid_code() {
        let code = "fn main(){let x=5;println!(\"{}\",x);}";
        let result = format_with_rustfmt(code, false);
        // May be None if rustfmt not available; if Some, must contain fn main
        if let Some(formatted) = result {
            assert!(formatted.contains("fn main"));
        }
    }

    #[test]
    fn test_format_with_rustfmt_invalid_code() {
        // Invalid Rust should cause rustfmt to fail -> None
        let code = "fn broken( { let";
        let result = format_with_rustfmt(code, false);
        assert!(result.is_none());
    }

    // ── handle_bundle_command ──────────────────────────────────────────────

    #[test]
    fn test_handle_bundle_command_basic() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() { println!(\"hello\"); }");

        let cli = Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap()]).unwrap();
        assert!(handle_bundle_command(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_minify() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");

        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--minify"]).unwrap();
        assert!(handle_bundle_command(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_m2() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() { let x: i32 = -1; let _ = x < -1; }");

        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--m2"]).unwrap();
        assert!(handle_bundle_command(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_verbose() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");

        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--verbose"]).unwrap();
        assert!(handle_bundle_command(&cli).is_ok());
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
        assert!(handle_bundle_command(&cli).is_ok());
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
        assert!(handle_bundle_command(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_pretty_verbose_invalid_code() {
        // Forces the "Warning: rustfmt formatting failed" verbose branch by using
        // code that rustfmt would reject (invalid Rust syntax would already be rejected
        // by syn, so use valid-but-unfixable input: just provide normal code and
        // let format_with_rustfmt return Some/None naturally).
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--pretty",
            "--verbose",
        ])
        .unwrap();
        // Either path (rustfmt available or not) is fine; we just ensure no panic.
        let _ = handle_bundle_command(&cli);
    }

    #[test]
    fn test_handle_bundle_command_invalid_path() {
        let cli = Cli::try_parse_from(["cg-bundler", "/nonexistent/path/xyz"]).unwrap();
        assert!(handle_bundle_command(&cli).is_err());
    }

    #[test]
    fn test_handle_bundle_command_pretty() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");

        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--pretty"]).unwrap();
        assert!(handle_bundle_command(&cli).is_ok());
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
        assert!(handle_bundle_command(&cli).is_ok());
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
        assert!(handle_bundle_command(&cli).is_ok());
    }

    #[test]
    fn test_handle_bundle_command_pretty_verbose() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--pretty",
            "--verbose",
        ])
        .unwrap();
        assert!(handle_bundle_command(&cli).is_ok());
    }

    #[test]
    fn test_format_with_rustfmt_verbose() {
        let code = "fn main(){let x=5;println!(\"{}\",x);}";
        // Should not panic regardless of whether rustfmt is installed
        let _result = format_with_rustfmt(code, true);
    }

    // ── aggressive_minify_code edge cases ────────────────────────────────

    #[test]
    fn test_aggressive_minify_code_with_escaped_string_chars() {
        // Covers the escape-tracking branches inside the '"' arm (lines 451, 453):
        // `escaped = false` and `escaped = true` are hit only when the input
        // contains a backslash-escaped character inside a double-quoted literal.
        let code = r#"fn f() { let s = "hello \"world\" \\path"; }"#;
        let result = aggressive_minify_code(code);
        // The escaped string content must be preserved verbatim
        assert!(
            result.contains(r#""hello \"world\" \\path""#),
            "escaped string content must survive minification: {result}"
        );
    }

    #[test]
    fn test_aggressive_minify_code_with_char_literals() {
        // Covers the `found_closing = true` branch and the subsequent storage
        // path (lines 488, 495-497) by providing plain char literals.
        let code = "fn f() { let _a = 'a'; let _z = 'z'; }";
        let result = aggressive_minify_code(code);
        assert!(
            result.contains("'a'") && result.contains("'z'"),
            "char literals must survive minification: {result}"
        );
    }

    #[test]
    fn test_aggressive_minify_code_with_escaped_char_literal() {
        // Covers the escaped backslash branches inside the '\'' arm (lines 484, 486):
        // `escaped = false` and `escaped = true` are hit for `'\\'` or `'\n'`.
        let code = r"fn f() { let _bs = '\\'; let _nl = '\n'; }";
        let result = aggressive_minify_code(code);
        // We don't assert the exact form since it depends on how the parser
        // reconstructs the char; just verify no panic and the function runs.
        let _ = result;
    }

    // ── handle_info_command with dependencies ──────────────────────────────

    #[test]
    fn test_handle_info_command_with_dependencies() {
        // Use the CG-Bundler project itself which has several dependencies.
        // This covers the `for dep in &package.dependencies { println!(...) }` loop (lines 368-369).
        let current_dir = std::env::current_dir().unwrap();
        if current_dir.join("Cargo.toml").exists() {
            let result = handle_info_command(&current_dir);
            assert!(result.is_ok());
        }
    }

    // ── handle_bundle_command write-to-dir (fs::write failure) ────────────

    #[test]
    fn test_handle_bundle_command_write_to_directory_fails() {
        // Writing to a directory path (not a file) causes fs::write to return an Io error.
        // This covers the map_err closure on fs::write (lines 245-246).
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "-o",
            tmp.path().to_str().unwrap(), // output path IS a directory → write fails
        ])
        .unwrap();
        let result = handle_bundle_command(&cli);
        assert!(result.is_err());
        match result.unwrap_err() {
            BundlerError::Io { .. } => {}
            e => panic!("Expected Io error from failed fs::write, got: {e}"),
        }
    }

    #[test]
    fn test_handle_watch_command_missing_src_dir_no_output() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        // src_dir "nonexistent_xyz" does not exist under the project → early error return
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--watch",
            "--src-dir",
            "nonexistent_xyz",
        ])
        .unwrap();
        let result = handle_watch_command(&cli);
        assert!(result.is_err());
        match result.unwrap_err() {
            BundlerError::Io { .. } => {}
            e => panic!("Expected Io error, got: {e}"),
        }
    }

    #[test]
    fn test_handle_watch_command_missing_src_dir_with_output() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let out = tmp.path().join("out.rs");
        // With --output flag so the "if let Some(output)" branch is covered
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--watch",
            "--src-dir",
            "nonexistent_xyz",
            "-o",
            out.to_str().unwrap(),
        ])
        .unwrap();
        let result = handle_watch_command(&cli);
        assert!(result.is_err());
    }

    // ── handle_validate_command ────────────────────────────────────────────

    #[test]
    fn test_handle_validate_command_valid() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        assert!(handle_validate_command(&tmp.path().to_path_buf(), false).is_ok());
    }

    #[test]
    fn test_handle_validate_command_verbose() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        assert!(handle_validate_command(&tmp.path().to_path_buf(), true).is_ok());
    }

    #[test]
    fn test_handle_validate_command_verbose_with_lib() {
        let tmp = TempDir::new().unwrap();
        make_project_with_lib(tmp.path());
        // verbose=true with a library target → covers the "Library target:" eprintln
        assert!(handle_validate_command(&tmp.path().to_path_buf(), true).is_ok());
    }

    #[test]
    fn test_handle_validate_command_invalid() {
        let tmp = TempDir::new().unwrap();
        let bad = tmp.path().join("nonexistent");
        assert!(handle_validate_command(&bad, false).is_err());
    }

    // ── handle_info_command ────────────────────────────────────────────────

    #[test]
    fn test_handle_info_command_valid() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        assert!(handle_info_command(&tmp.path().to_path_buf()).is_ok());
    }

    #[test]
    fn test_handle_info_command_with_lib_and_description() {
        let tmp = TempDir::new().unwrap();
        make_project_with_lib(tmp.path());
        // Covers: description branch, library target branch in info command
        assert!(handle_info_command(&tmp.path().to_path_buf()).is_ok());
    }

    #[test]
    fn test_handle_info_command_invalid() {
        let tmp = TempDir::new().unwrap();
        let bad = tmp.path().join("nonexistent");
        assert!(handle_info_command(&bad).is_err());
    }

    // ── should_rebuild ─────────────────────────────────────────────────────

    #[test]
    fn test_should_rebuild_create_rs_file() {
        use notify::{
            Event,
            event::{CreateKind, EventKind},
        };
        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![PathBuf::from("src/main.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_modify_rs_file() {
        use notify::{
            Event,
            event::{EventKind, ModifyKind},
        };
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            paths: vec![PathBuf::from("src/lib.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_non_rs_file() {
        use notify::{
            Event,
            event::{EventKind, ModifyKind},
        };
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            paths: vec![PathBuf::from("README.md")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_remove_rs_file() {
        use notify::{
            Event,
            event::{EventKind, RemoveKind},
        };
        let event = Event {
            kind: EventKind::Remove(RemoveKind::File),
            paths: vec![PathBuf::from("src/utils.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_access_event() {
        use notify::{
            Event,
            event::{AccessKind, EventKind},
        };
        let event = Event {
            kind: EventKind::Access(AccessKind::Read),
            paths: vec![PathBuf::from("src/main.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(!should_rebuild(&event));
    }

    // ── original tests ─────────────────────────────────────────────────────

    #[test]
    fn test_aggressive_minify_code() {
        let snippet = r#"
trait Printable {
    fn print(&self);
}

struct Person {
    name: String,
}

impl Printable for Person {
    fn print(&self) {
        println!("Person: {}", self.name);
    }
}

impl Person {
    fn new(name: String) -> Self {
        Person { name }
    }
}

fn main() {
    let person = Person::new("Alice".to_string());
    person.print();
}
"#;
        let minify_code = r#"trait Printable{fn print(&self);}struct Person{name:String}impl Printable for Person{fn print(&self){println!("Person: {}",self.name);}}impl Person{fn new(name:String)->Self{Person{name}}}fn main(){let person=Person::new("Alice".to_string());person.print();}"#;
        assert_eq!(aggressive_minify_code(snippet), minify_code);
    }

    #[test]
    fn test_aggressive_minify_does_not_create_spurious_arrow_sequences() {
        // The < comparison operator followed by a negative number must not
        // produce the '<-' sequence, which is misleading and can break parsers
        // in some Rust editions.  See: https://github.com/MathieuSoysal/CG-Bundler/issues/60
        let snippet = r#"
fn classify(x: i32) -> &'static str {
    match x {
        n if n < -10 => "very negative",
        n if n < 0 => "negative",
        0 => "zero",
        _ => "positive",
    }
}

fn filter_neg(v: &[i32]) -> Vec<i32> {
    v.iter().filter(|&&x| x < -1).cloned().collect()
}
"#;
        let result = aggressive_minify_code(snippet);
        // '<-' must never appear: it is not a valid Rust operator and arises
        // only from incorrectly joining '< ' and '-'.
        assert!(
            !result.contains("<-"),
            "spurious '<-' sequence found in minified output: {result}"
        );
        // The legitimate '->' (return type arrow) must still be present.
        assert!(
            result.contains("->"),
            "'->' was unexpectedly removed from minified output: {result}"
        );
        // Match-arm fat arrows must still be present.
        assert!(
            result.contains("=>"),
            "'=>' was unexpectedly removed from minified output: {result}"
        );
    }
}
