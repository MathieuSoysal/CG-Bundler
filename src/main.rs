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
        .replace(",]", "]");

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
                                println!("{} File change detected: {:?}", "🔄".yellow(), file_name);
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
    fn test_minify_code_removes_empty_lines() {
        let code = "fn main() {\n    println!(\"hello\");\n}\n";
        let result = minify_code(code);
        assert!(!result.contains('\n'));
        assert!(result.contains("fn main()"));
        assert!(result.contains("println!"));
    }

    #[test]
    fn test_minify_code_trims_whitespace() {
        let code = "   fn main() {   \n   let x = 5;   \n}   ";
        let result = minify_code(code);
        // All lines should be trimmed and joined with spaces
        assert!(!result.starts_with(' '));
        assert!(!result.ends_with(' '));
    }

    #[test]
    fn test_minify_code_filters_empty_lines() {
        let code = "fn main() {\n\n\n    println!(\"hello\");\n\n}";
        let result = minify_code(code);
        // Empty lines should be removed
        assert!(!result.contains("  "));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_minify_code_empty_input() {
        let result = minify_code("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_minify_code_single_line() {
        let code = "fn main() { println!(\"hello\"); }";
        let result = minify_code(code);
        assert_eq!(result, "fn main() { println!(\"hello\"); }");
    }

    #[test]
    fn test_aggressive_minify_preserves_string_literals() {
        let code = "fn main() {\n    let s = \"hello world\";\n    println!(\"{}\", s);\n}\n";
        let result = aggressive_minify_code(code);
        assert!(result.contains("\"hello world\""));
        assert!(result.contains("\"{}\""));
    }

    #[test]
    fn test_aggressive_minify_preserves_char_literals() {
        let code = "fn main() {\n    let c = 'a';\n    println!(\"{}\", c);\n}\n";
        let result = aggressive_minify_code(code);
        assert!(result.contains("'a'"));
    }

    #[test]
    fn test_aggressive_minify_empty_input() {
        let result = aggressive_minify_code("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_cli_get_project_path_default() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        assert_eq!(cli.get_project_path(), PathBuf::from("."));
    }

    #[test]
    fn test_cli_get_project_path_custom() {
        let cli = Cli {
            project_path: Some(PathBuf::from("/my/project")),
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        assert_eq!(cli.get_project_path(), PathBuf::from("/my/project"));
    }

    #[test]
    fn test_cli_is_verbose() {
        let mut cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        assert!(!cli.is_verbose());
        cli.verbose = true;
        assert!(cli.is_verbose());
    }

    #[test]
    fn test_cli_get_output_none() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        assert!(cli.get_output().is_none());
    }

    #[test]
    fn test_cli_get_output_some() {
        let output_path = PathBuf::from("/output.rs");
        let cli = Cli {
            project_path: None,
            output: Some(output_path.clone()),
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        assert_eq!(cli.get_output(), Some(&output_path));
    }

    #[test]
    fn test_cli_is_pretty() {
        let mut cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        assert!(!cli.is_pretty());
        cli.pretty = true;
        assert!(cli.is_pretty());
    }

    #[test]
    fn test_cli_is_minify() {
        let mut cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        assert!(!cli.is_minify());
        cli.minify = true;
        assert!(cli.is_minify());
    }

    #[test]
    fn test_cli_is_minify_via_m2() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: true,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        // m2 implies minify
        assert!(cli.is_minify());
    }

    #[test]
    fn test_cli_is_aggressive_minify() {
        let mut cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        assert!(!cli.is_aggressive_minify());
        cli.m2 = true;
        assert!(cli.is_aggressive_minify());
    }

    #[test]
    fn test_cli_get_transform_config_defaults() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        let config = cli.get_transform_config();
        // By default: remove_tests = !keep_tests = true, remove_docs = !keep_docs = true
        assert!(config.remove_tests);
        assert!(config.remove_docs);
        assert!(config.expand_modules);
        assert!(!config.minify);
        assert!(!config.aggressive_minify);
    }

    #[test]
    fn test_cli_get_transform_config_with_keep_tests() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: true,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        let config = cli.get_transform_config();
        assert!(!config.remove_tests);
        assert!(config.remove_docs);
    }

    #[test]
    fn test_cli_get_transform_config_with_keep_docs() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: true,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        let config = cli.get_transform_config();
        assert!(config.remove_tests);
        assert!(!config.remove_docs);
    }

    #[test]
    fn test_cli_get_transform_config_with_no_expand_modules() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: true,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        let config = cli.get_transform_config();
        assert!(!config.expand_modules);
    }

    #[test]
    fn test_cli_get_transform_config_with_minify() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: true,
            m2: false,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        let config = cli.get_transform_config();
        assert!(config.minify);
        assert!(!config.aggressive_minify);
    }

    #[test]
    fn test_cli_get_transform_config_with_m2() {
        let cli = Cli {
            project_path: None,
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: true,
            verbose: false,
            validate: false,
            info: false,
            watch: false,
            src_dir: "src".to_string(),
            debounce: 500,
        };
        let config = cli.get_transform_config();
        assert!(config.minify);
        assert!(config.aggressive_minify);
    }

    #[test]
    fn test_should_rebuild_create_rs_file() {
        use notify::{Event, EventKind};
        use notify::event::CreateKind;

        let event = Event::new(EventKind::Create(CreateKind::File))
            .add_path(PathBuf::from("/some/path/main.rs"));
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_modify_rs_file() {
        use notify::{Event, EventKind};
        use notify::event::{DataChange, ModifyKind};

        let event = Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Content)))
            .add_path(PathBuf::from("/some/path/lib.rs"));
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_remove_rs_file() {
        use notify::{Event, EventKind};
        use notify::event::RemoveKind;

        let event = Event::new(EventKind::Remove(RemoveKind::File))
            .add_path(PathBuf::from("/some/path/module.rs"));
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_non_rs_file() {
        use notify::{Event, EventKind};
        use notify::event::ModifyKind;

        let event = Event::new(EventKind::Modify(ModifyKind::Any))
            .add_path(PathBuf::from("/some/path/Cargo.toml"));
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_access_event() {
        use notify::{Event, EventKind};
        use notify::event::AccessKind;

        let event = Event::new(EventKind::Access(AccessKind::Read))
            .add_path(PathBuf::from("/some/path/main.rs"));
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_other_event() {
        use notify::{Event, EventKind};

        let event =
            Event::new(EventKind::Other).add_path(PathBuf::from("/some/path/main.rs"));
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_any_event() {
        use notify::{Event, EventKind};

        let event =
            Event::new(EventKind::Any).add_path(PathBuf::from("/some/path/main.rs"));
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_no_paths() {
        use notify::{Event, EventKind};
        use notify::event::ModifyKind;

        let event = Event::new(EventKind::Modify(ModifyKind::Any));
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_non_rs_extension() {
        use notify::{Event, EventKind};
        use notify::event::CreateKind;

        let event = Event::new(EventKind::Create(CreateKind::File))
            .add_path(PathBuf::from("/some/path/file.txt"));
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_display_bug_report_info_does_not_panic() {
        // Just ensure the function runs without panicking
        display_bug_report_info();
    }

    #[test]
    fn test_format_with_rustfmt_valid_code() {
        let code = "fn main(){println!(\"hello\");}";
        // format_with_rustfmt may return None if rustfmt is not available
        // but should not panic
        let _result = format_with_rustfmt(code, false);
    }

    #[test]
    fn test_format_with_rustfmt_invalid_code() {
        let code = "this is not valid rust code !!!";
        // Should return None for invalid code without panicking
        let result = format_with_rustfmt(code, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_aggressive_minify_code_with_escaped_quotes() {
        let code = "fn main() {\n    let s = \"he said \\\"hello\\\"\";\n}\n";
        let result = aggressive_minify_code(code);
        assert!(result.contains("\"he said \\\"hello\\\"\""));
    }

    #[test]
    fn test_aggressive_minify_code_with_lifetime() {
        let code = "fn foo<'a>(x: &'a str) -> &'a str {\n    x\n}\n";
        let result = aggressive_minify_code(code);
        assert!(result.contains("'a"));
    }

    #[test]
    fn test_aggressive_minify_code_trailing_comma_cleanup() {
        // Verify trailing comma cleanup in various contexts
        let code = "fn main() {\n    let v: Vec<u32> = vec![1, 2, 3];\n}\n";
        let result = aggressive_minify_code(code);
        assert!(result.contains("vec![1,2,3]") || result.contains("vec ! [1,2,3]"));
    }
}
