use cg_bundler::{Bundler, TransformConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Re-import the functions from main.rs for testing
// Since they're not public, we'll include them in the test module

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

#[cfg(test)]
mod main_functionality_tests {
    use super::*;

    /// Tests for the minify_code function
    mod minification_tests {
        use super::*;

        #[test]
        fn test_minify_basic_code() {
            let input = r#"
fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x + 1;
}
"#;
            let result = minify_code(input);
            assert_eq!(result, r#"fn main() { println!("Hello, world!"); let x = 42; let y = x + 1; }"#);
        }

        #[test]
        fn test_minify_with_empty_lines() {
            let input = r#"
fn main() {

    println!("Hello");

    let x = 1;

}
"#;
            let result = minify_code(input);
            assert_eq!(result, r#"fn main() { println!("Hello"); let x = 1; }"#);
        }

        #[test]
        fn test_minify_preserves_content() {
            let input = "fn add(a: i32, b: i32) -> i32 { a + b }";
            let result = minify_code(input);
            assert_eq!(result, "fn add(a: i32, b: i32) -> i32 { a + b }");
        }

        #[test]
        fn test_minify_with_whitespace_only_lines() {
            let input = "   \n\t\nfn main() {}\n   \t   \n";
            let result = minify_code(input);
            assert_eq!(result, "fn main() {}");
        }

        #[test]
        fn test_minify_empty_input() {
            let input = "";
            let result = minify_code(input);
            assert_eq!(result, "");
        }

        #[test]
        fn test_minify_whitespace_only() {
            let input = "   \n\t\n   ";
            let result = minify_code(input);
            assert_eq!(result, "");
        }
    }

    /// Tests for the aggressive_minify_code function
    mod aggressive_minification_tests {
        use super::*;

        #[test]
        fn test_aggressive_minify_operators() {
            let input = "let x = a + b - c * d / e % f;";
            let result = aggressive_minify_code(input);
            assert_eq!(result, "let x=a+b-c*d/e%f;");
        }

        #[test]
        fn test_aggressive_minify_comparisons() {
            let input = "if x == y && a != b || c <= d && e >= f { }";
            let result = aggressive_minify_code(input);
            assert_eq!(result, "if x==y&&a!=b||c<=d&&e>=f{}");
        }

        #[test]
        fn test_aggressive_minify_arrows() {
            let input = "fn test() -> Option<i32> { Some(x) => y }";
            let result = aggressive_minify_code(input);
            assert_eq!(result, "fn test()->Option<i32>{Some(x)=>y}");
        }

        #[test]
        fn test_aggressive_minify_punctuation() {
            let input = "let arr = [1, 2, 3]; let tup = (a, b, c);";
            let result = aggressive_minify_code(input);
            assert_eq!(result, "let arr=[1,2,3];let tup=(a,b,c);");
        }

        #[test]
        fn test_aggressive_minify_preserves_string_literals() {
            let input = r#"println!("Hello, world!"); let msg = "Test string";"#;
            let result = aggressive_minify_code(input);
            assert_eq!(result, r#"println!("Hello, world!");let msg="Test string";"#);
        }

        #[test]
        fn test_aggressive_minify_string_with_spaces() {
            let input = r#"let msg = "This is a test string with spaces";"#;
            let result = aggressive_minify_code(input);
            assert_eq!(result, r#"let msg="This is a test string with spaces";"#);
        }

        #[test]
        fn test_aggressive_minify_escaped_quotes() {
            let input = r#"let msg = "He said \"Hello\" to me";"#;
            let result = aggressive_minify_code(input);
            assert_eq!(result, r#"let msg="He said \"Hello\" to me";"#);
        }

        #[test]
        fn test_aggressive_minify_multiple_strings() {
            let input = r#"let a = "First"; let b = "Second"; let c = a + b;"#;
            let result = aggressive_minify_code(input);
            assert_eq!(result, r#"let a="First";let b="Second";let c=a+b;"#);
        }

        #[test]
        fn test_aggressive_minify_complex_code() {
            let input = r#"
                fn fibonacci(n: u32) -> u32 {
                    if n <= 1 {
                        return n;
                    }
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            "#;
            let result = aggressive_minify_code(input);
            let expected = "fn fibonacci(n: u32)->u32{if n<=1{return n;}fibonacci(n-1)+fibonacci(n-2)}";
            assert_eq!(result, expected);
        }

        #[test]
        fn test_aggressive_minify_with_comments_removed() {
            // Note: Comments should be removed by the basic minification first
            let input = "let x = 5; let y = 10;";
            let result = aggressive_minify_code(input);
            assert_eq!(result, "let x=5;let y=10;");
        }
    }

    /// Tests for CLI argument handling and validation
    mod cli_argument_tests {
        use super::*;
        use clap::Parser;

        // Re-define the CLI struct for testing (since it's in main.rs)
        #[derive(Parser, Debug)]
        struct TestCli {
            pub project_path: Option<PathBuf>,
            #[arg(short, long)]
            pub output: Option<PathBuf>,
            #[arg(long)]
            pub keep_tests: bool,
            #[arg(long)]
            pub keep_docs: bool,
            #[arg(long)]
            pub no_expand_modules: bool,
            #[arg(long)]
            pub pretty: bool,
            #[arg(short, long)]
            pub minify: bool,
            #[arg(long)]
            pub m2: bool,
            #[arg(short, long)]
            pub verbose: bool,
            #[arg(long)]
            pub validate: bool,
            #[arg(long)]
            pub info: bool,
            #[arg(short, long)]
            pub watch: bool,
            #[arg(long, default_value = "src")]
            pub src_dir: String,
            #[arg(long, default_value = "500")]
            pub debounce: u64,
        }

        impl TestCli {
            pub fn get_project_path(&self) -> PathBuf {
                self.project_path
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("."))
            }

            pub fn get_transform_config(&self) -> TransformConfig {
                TransformConfig {
                    remove_tests: !self.keep_tests,
                    remove_docs: !self.keep_docs,
                    expand_modules: !self.no_expand_modules,
                    minify: self.minify || self.m2,
                    aggressive_minify: self.m2,
                }
            }
        }

        #[test]
        fn test_cli_default_values() {
            let cli = TestCli {
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
            assert_eq!(cli.src_dir, "src");
            assert_eq!(cli.debounce, 500);
        }

        #[test]
        fn test_transform_config_defaults() {
            let cli = TestCli {
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
            assert_eq!(config.remove_tests, true);
            assert_eq!(config.remove_docs, true);
            assert_eq!(config.expand_modules, true);
            assert_eq!(config.minify, false);
            assert_eq!(config.aggressive_minify, false);
        }

        #[test]
        fn test_transform_config_with_flags() {
            let cli = TestCli {
                project_path: None,
                output: None,
                keep_tests: true,
                keep_docs: true,
                no_expand_modules: true,
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
            assert_eq!(config.remove_tests, false);
            assert_eq!(config.remove_docs, false);
            assert_eq!(config.expand_modules, false);
            assert_eq!(config.minify, true);
            assert_eq!(config.aggressive_minify, false);
        }

        #[test]
        fn test_m2_implies_minify() {
            let cli = TestCli {
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
            assert_eq!(config.minify, true);
            assert_eq!(config.aggressive_minify, true);
        }

        #[test]
        fn test_custom_project_path() {
            let cli = TestCli {
                project_path: Some(PathBuf::from("/custom/path")),
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
                src_dir: "custom_src".to_string(),
                debounce: 1000,
            };

            assert_eq!(cli.get_project_path(), PathBuf::from("/custom/path"));
            assert_eq!(cli.src_dir, "custom_src");
            assert_eq!(cli.debounce, 1000);
        }
    }

    /// Integration tests for bundling functionality
    mod bundling_integration_tests {
        use super::*;

        fn create_test_project_structure(temp_dir: &TempDir, project_name: &str) -> PathBuf {
            let project_path = temp_dir.path().join(project_name);
            fs::create_dir_all(&project_path).unwrap();

            // Create Cargo.toml
            fs::write(
                project_path.join("Cargo.toml"),
                format!(
                    r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
                    project_name, project_name
                ),
            )
            .unwrap();

            // Create src directory
            let src_dir = project_path.join("src");
            fs::create_dir_all(&src_dir).unwrap();

            // Create main.rs
            fs::write(
                src_dir.join("main.rs"),
                r#"
mod utils;

use utils::greet;

fn main() {
    greet("World");
    let result = add(5, 3);
    println!("Result: {}", result);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#,
            )
            .unwrap();

            // Create utils.rs module
            fs::write(
                src_dir.join("utils.rs"),
                r#"
/// Greets the given name
pub fn greet(name: &str) {
    println!("Hello, {}!", name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        greet("Test");
    }
}
"#,
            )
            .unwrap();

            project_path
        }

        #[test]
        fn test_bundling_with_modules() {
            let temp_dir = TempDir::new().unwrap();
            let project_path = create_test_project_structure(&temp_dir, "test_project");

            let bundler = Bundler::new();
            let result = bundler.bundle(&project_path);

            assert!(result.is_ok());
            let bundled_code = result.unwrap();

            // Should contain the main function
            assert!(bundled_code.contains("fn main()"));
            // Should contain the add function
            assert!(bundled_code.contains("fn add("));
            // Should contain the greet function from utils
            assert!(bundled_code.contains("pub fn greet("));
        }

        #[test]
        fn test_bundling_removes_tests_by_default() {
            let temp_dir = TempDir::new().unwrap();
            let project_path = create_test_project_structure(&temp_dir, "test_project");

            let bundler = Bundler::new();
            let result = bundler.bundle(&project_path);

            assert!(result.is_ok());
            let bundled_code = result.unwrap();

            // Should not contain test functions
            assert!(!bundled_code.contains("fn test_add()"));
            assert!(!bundled_code.contains("fn test_greet()"));
            assert!(!bundled_code.contains("#[test]"));
        }

        #[test]
        fn test_bundling_keeps_tests_when_configured() {
            let temp_dir = TempDir::new().unwrap();
            let project_path = create_test_project_structure(&temp_dir, "test_project");

            let config = TransformConfig {
                remove_tests: false,
                remove_docs: true,
                expand_modules: true,
                minify: false,
                aggressive_minify: false,
            };

            let bundler = Bundler::with_config(config);
            let result = bundler.bundle(&project_path);

            assert!(result.is_ok());
            let bundled_code = result.unwrap();

            // Should contain test functions
            assert!(bundled_code.contains("#[test]"));
        }

        #[test]
        fn test_bundling_removes_docs_by_default() {
            let temp_dir = TempDir::new().unwrap();
            let project_path = create_test_project_structure(&temp_dir, "test_project");

            let bundler = Bundler::new();
            let result = bundler.bundle(&project_path);

            assert!(result.is_ok());
            let bundled_code = result.unwrap();

            // Should not contain doc comments
            assert!(!bundled_code.contains("/// Greets the given name"));
        }

        #[test]
        fn test_bundling_keeps_docs_when_configured() {
            let temp_dir = TempDir::new().unwrap();
            let project_path = create_test_project_structure(&temp_dir, "test_project");

            let config = TransformConfig {
                remove_tests: true,
                remove_docs: false,
                expand_modules: true,
                minify: false,
                aggressive_minify: false,
            };

            let bundler = Bundler::with_config(config);
            let result = bundler.bundle(&project_path);

            assert!(result.is_ok());
            let bundled_code = result.unwrap();

            // Should contain doc comments
            assert!(bundled_code.contains("/// Greets the given name"));
        }

        #[test]
        fn test_minified_bundling() {
            let temp_dir = TempDir::new().unwrap();
            let project_path = create_test_project_structure(&temp_dir, "test_project");

            let bundler = Bundler::new();
            let bundled_code = bundler.bundle(&project_path).unwrap();

            let minified = minify_code(&bundled_code);

            // Should be on fewer lines
            assert!(minified.lines().count() < bundled_code.lines().count());
            // Should still contain essential content
            assert!(minified.contains("fn main()"));
            assert!(minified.contains("fn add("));
        }

        #[test]
        fn test_aggressive_minified_bundling() {
            let temp_dir = TempDir::new().unwrap();
            let project_path = create_test_project_structure(&temp_dir, "test_project");

            let bundler = Bundler::new();
            let bundled_code = bundler.bundle(&project_path).unwrap();

            let aggressively_minified = aggressive_minify_code(&bundled_code);

            // Should be shorter than regular minification
            let regular_minified = minify_code(&bundled_code);
            assert!(aggressively_minified.len() < regular_minified.len());
            
            // Should still contain essential content but with no spaces around operators
            assert!(aggressively_minified.contains("fn main()"));
            assert!(aggressively_minified.contains("a+b"));
        }
    }

    /// Tests for error handling and edge cases
    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_minify_handles_unicode() {
            let input = "let message = \"Hello 世界\"; // Unicode comment\nfn test() {}";
            let result = minify_code(input);
            assert!(result.contains("Hello 世界"));
            assert!(result.contains("fn test()"));
        }

        #[test]
        fn test_aggressive_minify_handles_complex_strings() {
            let input = r#"let complex = "String with \"quotes\" and \\backslashes\\"; let x = a + b;"#;
            let result = aggressive_minify_code(input);
            assert!(result.contains(r#""String with \"quotes\" and \\backslashes\\""#));
            assert!(result.contains("x=a+b"));
        }

        #[test]
        fn test_minify_preserves_string_newlines() {
            let input = "let multiline = \"Line 1\\nLine 2\"; let x = 1;";
            let result = aggressive_minify_code(input);
            assert!(result.contains("\"Line 1\\nLine 2\""));
            assert!(result.contains("x=1"));
        }

        #[test]
        fn test_empty_string_handling() {
            let input = r#"let empty = ""; let not_empty = "test";"#;
            let result = aggressive_minify_code(input);
            assert!(result.contains(r#"empty="""#));
            assert!(result.contains(r#"not_empty="test""#));
        }
    }

    /// Performance and stress tests
    mod performance_tests {
        use super::*;

        #[test]
        fn test_minify_large_input() {
            // Create a large input string
            let mut large_input = String::new();
            for i in 0..1000 {
                large_input.push_str(&format!("fn function_{}() {{ let x = {}; }}\n", i, i));
            }

            let result = minify_code(&large_input);
            
            // Should still work correctly
            assert!(result.contains("fn function_0()"));
            assert!(result.contains("fn function_999()"));
            // Should be minified (no newlines)
            assert!(!result.contains('\n'));
        }

        #[test]
        fn test_aggressive_minify_many_strings() {
            let mut input = String::new();
            for i in 0..100 {
                input.push_str(&format!(r#"let str_{} = "String number {}"; "#, i, i));
            }

            let result = aggressive_minify_code(&input);
            
            // Should preserve all strings
            assert!(result.contains(r#""String number 0""#));
            assert!(result.contains(r#""String number 99""#));
            // Should be aggressively minified
            assert!(result.contains("str_0="));
            assert!(result.contains("str_99="));
        }
    }
}
