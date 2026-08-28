//! Regression tests for defects found during the codebase audit.
//!
//! Each test maps to a concrete bug where bundling used to either silently delete
//! live code, silently emit code that does not compile, or exit successfully with
//! a broken bundle.

use assert_cmd::cargo::cargo_bin_cmd;
use cg_bundler::{Bundler, TransformConfig, bundle};
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Write a Cargo project. `files` are `(relative path, contents)` pairs.
fn write_project(root: &Path, manifest: &str, files: &[(&str, &str)]) {
    fs::create_dir_all(root).expect("create project root");
    fs::write(root.join("Cargo.toml"), manifest).expect("write Cargo.toml");

    for (relative, contents) in files {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("file has a parent")).expect("create parent dir");
        fs::write(&path, contents).expect("write source file");
    }
}

fn simple_manifest(name: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
}

/// Bundle and assert the result parses as Rust, returning it for further assertions.
fn bundle_and_parse(project: &Path) -> String {
    let bundled = bundle(project).expect("bundling should succeed");
    syn::parse_file(&bundled)
        .unwrap_or_else(|e| panic!("bundle is not valid Rust: {e}\n{bundled}"));
    bundled
}

mod test_detection {
    use super::*;

    /// `#[cfg(not(test))]` items belong to the *non-test* build and must survive.
    #[test]
    fn keeps_cfg_not_test_items() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("cfg_not_test"),
            &[(
                "src/main.rs",
                "fn main() { println!(\"{}\", compute()); }\n\
                 #[cfg(not(test))]\nfn compute() -> i32 { 42 }\n\
                 #[cfg(test)]\nfn compute() -> i32 { 0 }\n",
            )],
        );

        let bundled = bundle_and_parse(temp.path());

        assert!(
            bundled.contains("#[cfg(not(test))]"),
            "cfg(not(test)) item must be kept:\n{bundled}"
        );
        assert!(
            !bundled.contains("#[cfg(test)]"),
            "cfg(test) item must be removed:\n{bundled}"
        );
        assert!(
            bundled.contains("42"),
            "the non-test implementation must survive:\n{bundled}"
        );
    }

    /// A `cfg` predicate that merely *contains* the substring "test" is not a test marker.
    #[test]
    fn keeps_predicates_whose_text_contains_test() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("cfg_substring"),
            &[(
                "src/main.rs",
                "#[cfg(feature = \"fastest\")]\nfn fast_path() {}\n\
                 #[cfg(target_os = \"latest_os\")]\nfn other() {}\n\
                 fn main() {}\n",
            )],
        );

        let bundled = bundle_and_parse(temp.path());

        assert!(
            bundled.contains("fast_path"),
            "feature `fastest` is unrelated to tests:\n{bundled}"
        );
        assert!(
            bundled.contains("fn other"),
            "target_os predicate is unrelated to tests:\n{bundled}"
        );
    }

    /// `any(test, X)` holds whenever `X` holds, so it is not a test-only marker.
    /// `all(test, X)` cannot hold outside a test build, so it is.
    #[test]
    fn distinguishes_any_from_all_in_cfg_predicates() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            "[package]\nname = \"any_all\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\ndebug = []\n",
            &[(
                "src/main.rs",
                "#[cfg(any(test, feature = \"debug\"))]\npub fn kept() -> i32 { 7 }\n\
                 #[cfg(all(test, feature = \"debug\"))]\npub fn dropped() -> i32 { 8 }\n\
                 fn main() { println!(\"ok\"); }\n",
            )],
        );

        let bundled = bundle_and_parse(temp.path());

        assert!(
            bundled.contains("fn kept"),
            "any(test, feature = ..) can hold in a non-test build:\n{bundled}"
        );
        assert!(
            !bundled.contains("fn dropped"),
            "all(test, ..) is test-only:\n{bundled}"
        );
    }

    /// Test code nested inside an inline `mod` used to survive bundling.
    #[test]
    fn removes_tests_inside_inline_modules() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("inline_tests"),
            &[(
                "src/main.rs",
                "mod outer {\n    pub fn keep() {}\n\n    #[cfg(test)]\n    mod tests {\n        #[test]\n        fn nested() {}\n    }\n}\nfn main() { outer::keep(); }\n",
            )],
        );

        let bundled = bundle_and_parse(temp.path());

        assert!(bundled.contains("fn keep"), "real code must survive");
        assert!(
            !bundled.contains("#[cfg(test)]") && !bundled.contains("#[test]"),
            "nested test module must be removed:\n{bundled}"
        );
    }

    /// `--keep-tests` must still keep nested test code.
    #[test]
    fn keep_tests_config_preserves_nested_tests() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("keep_nested"),
            &[(
                "src/main.rs",
                "mod outer {\n    #[cfg(test)]\n    mod tests {\n        #[test]\n        fn nested() {}\n    }\n}\nfn main() {}\n",
            )],
        );

        let bundled = Bundler::with_config(TransformConfig {
            remove_tests: false,
            ..TransformConfig::default()
        })
        .bundle(temp.path())
        .expect("bundling should succeed");

        assert!(bundled.contains("#[test]"), "{bundled}");
    }
}

mod library_resolution {
    use super::*;

    /// The library root was hard-coded to `<src>/lib.rs`, breaking custom `[lib] path`.
    #[test]
    fn expands_library_with_custom_path() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            concat!(
                "[package]\nname = \"custom_lib\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
                "\n[lib]\nname = \"custom_lib\"\npath = \"src/library.rs\"\n",
                "\n[[bin]]\nname = \"custom_lib\"\npath = \"src/main.rs\"\n",
            ),
            &[
                ("src/library.rs", "pub fn hello() -> i32 { 7 }\n"),
                (
                    "src/main.rs",
                    "use custom_lib::hello;\nfn main() { println!(\"{}\", hello()); }\n",
                ),
            ],
        );

        let bundled = bundle_and_parse(temp.path());

        assert!(bundled.contains("fn hello"), "{bundled}");
        assert!(!bundled.contains("use custom_lib"), "{bundled}");
    }

    /// Imports of the crate's own library from a submodule must go through `crate::`.
    #[test]
    fn submodule_importing_own_crate_is_retargeted() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("self_import"),
            &[
                ("src/lib.rs", "pub struct Marker;\npub mod helper;\n"),
                (
                    "src/helper.rs",
                    "use self_import::Marker;\npub fn take(_: Marker) {}\n",
                ),
                (
                    "src/main.rs",
                    "use self_import::Marker;\nfn main() { helper::take(Marker); }\n",
                ),
            ],
        );

        let bundled = bundle_and_parse(temp.path());

        assert_eq!(
            bundled.matches("struct Marker").count(),
            1,
            "library must be inlined exactly once:\n{bundled}"
        );
        assert!(
            bundled.contains("pub fn take"),
            "submodule must be expanded:\n{bundled}"
        );
        assert!(
            !bundled.contains("use self_import"),
            "crate-relative import must be retargeted:\n{bundled}"
        );
    }
}

mod external_dependencies {
    use super::*;

    /// `helper` is a path dependency exposing `crate::util` internally.
    fn write_workspace(root: &Path, app_main: &str, extra_app_files: &[(&str, &str)]) {
        write_project(
            &root.join("helper"),
            "[package]\nname = \"helper\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            &[
                (
                    "src/lib.rs",
                    "pub mod util;\npub fn help() -> i32 { crate::util::v() }\n",
                ),
                ("src/util.rs", "pub fn v() -> i32 { 9 }\n"),
            ],
        );

        let mut files: Vec<(&str, &str)> = vec![("src/main.rs", app_main)];
        files.extend_from_slice(extra_app_files);

        write_project(
            &root.join("app"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nhelper = { path = \"../helper\" }\n",
            &files,
        );
    }

    /// A dependency used only from a submodule used to be left un-inlined.
    #[test]
    fn inlines_dependency_referenced_from_submodule() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            temp.path(),
            "mod inner;\nfn main() { println!(\"{}\", inner::run()); }\n",
            &[(
                "src/inner.rs",
                "use helper::help;\npub fn run() -> i32 { help() }\n",
            )],
        );

        let bundled = bundle_and_parse(&temp.path().join("app"));

        assert!(
            bundled.contains("mod helper"),
            "dependency must be inlined:\n{bundled}"
        );
        assert!(
            bundled.contains("use crate::helper::help"),
            "submodule import must be retargeted:\n{bundled}"
        );
    }

    /// A dependency reached only from a `#[cfg(test)]` import must not be inlined:
    /// the import is test-only code that bundling strips anyway.
    #[test]
    fn test_only_import_does_not_inline_dependency() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            temp.path(),
            "mod inner;\nfn main() { println!(\"{}\", inner::real()); }\n",
            &[(
                "src/inner.rs",
                "#[cfg(test)]\nuse helper::help;\n\npub fn real() -> i32 { 1 }\n",
            )],
        );

        let bundled = bundle_and_parse(&temp.path().join("app"));

        assert!(
            !bundled.contains("mod helper"),
            "a test-only import must not drag in the whole dependency:\n{bundled}"
        );
        assert!(
            !bundled.contains("use helper::help") && !bundled.contains("use crate::helper::help"),
            "the test-only import itself must be stripped:\n{bundled}"
        );
        assert!(
            bundled.contains("fn real"),
            "live code must survive:\n{bundled}"
        );
    }

    /// `crate::` inside an inlined dependency must be requalified to its new module.
    #[test]
    fn requalifies_crate_paths_inside_inlined_dependency() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            temp.path(),
            "use helper::help;\nfn main() { println!(\"{}\", help()); }\n",
            &[],
        );

        let bundled = bundle_and_parse(&temp.path().join("app"));

        assert!(
            bundled.contains("crate::helper::util::v()"),
            "dependency-internal `crate::` must point at the inlined module:\n{bundled}"
        );
    }

    /// Registry dependencies cannot be inlined; bundling must still succeed.
    #[test]
    fn registry_dependencies_are_not_inlined() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            concat!(
                "[package]\nname = \"registry_dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
                "\n[dependencies]\nserde = \"1.0\"\n",
            ),
            &[(
                "src/main.rs",
                "use serde::Serialize;\n#[derive(Serialize)]\nstruct S;\nfn main() {}\n",
            )],
        );

        let bundled = bundle_and_parse(temp.path());

        assert!(
            !bundled.contains("mod serde"),
            "registry crates must not be inlined:\n{bundled}"
        );
        assert!(bundled.contains("use serde::Serialize"), "{bundled}");
    }

    /// A dependency referenced *only* from inside a macro must still be inlined.
    #[test]
    fn detects_dependency_referenced_only_inside_a_macro() {
        let temp = TempDir::new().expect("temp dir");
        write_workspace(
            temp.path(),
            "mod inner;\nfn main() { println!(\"{}\", inner::show()); }\n",
            &[(
                "src/inner.rs",
                "pub fn show() -> String { format!(\"{}\", helper::help()) }\n",
            )],
        );

        let bundled = bundle_and_parse(&temp.path().join("app"));

        assert!(
            bundled.contains("mod helper"),
            "dependency used only inside a macro must be inlined:\n{bundled}"
        );
        assert!(
            bundled.contains("crate :: helper :: help") || bundled.contains("crate ::helper::help"),
            "macro-internal dependency path must be anchored at the bundle root:\n{bundled}"
        );
    }
}

mod failure_reporting {
    use super::*;

    /// A module that cannot be resolved used to produce a broken bundle and exit 0.
    #[test]
    fn unresolvable_module_is_an_error() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("missing_mod"),
            &[("src/main.rs", "mod missing;\nfn main() {}\n")],
        );

        assert!(
            bundle(temp.path()).is_err(),
            "unresolvable module must be reported as an error"
        );
    }

    #[test]
    fn cli_fails_on_unresolvable_module() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("missing_mod_cli"),
            &[("src/main.rs", "mod missing;\nfn main() {}\n")],
        );

        cargo_bin_cmd!("cg-bundler")
            .current_dir(temp.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn validate_fails_on_unresolvable_module() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("missing_mod_validate"),
            &[("src/main.rs", "mod missing;\nfn main() {}\n")],
        );

        cargo_bin_cmd!("cg-bundler")
            .current_dir(temp.path())
            .arg("--validate")
            .assert()
            .failure();
    }

    /// The library must stay silent; progress noise used to be printed unconditionally.
    #[test]
    fn bundling_is_silent_without_verbose() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("silent"),
            &[
                ("src/lib.rs", "pub fn hello() {}\n"),
                (
                    "src/main.rs",
                    "use silent::hello;\nfn main() { hello(); }\n",
                ),
            ],
        );

        cargo_bin_cmd!("cg-bundler")
            .current_dir(temp.path())
            .assert()
            .success()
            .stderr(predicate::str::is_empty());
    }
}

mod minification {
    use super::*;

    /// `,#` was rewritten to `#`, deleting a required separator before an attribute.
    #[test]
    fn aggressive_minify_keeps_comma_before_attribute() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("attr_comma"),
            &[(
                "src/main.rs",
                "pub enum Kind {\n    Alpha,\n    #[allow(dead_code)]\n    Beta,\n}\n\
                 pub struct Cfg {\n    pub a: i32,\n    #[allow(dead_code)]\n    pub b: i32,\n}\n\
                 fn main() { let _ = (Kind::Alpha, Cfg { a: 1, b: 2 }); }\n",
            )],
        );

        cargo_bin_cmd!("cg-bundler")
            .current_dir(temp.path())
            .args(["--m2", "-o", "out.rs"])
            .assert()
            .success();

        let bundled = fs::read_to_string(temp.path().join("out.rs")).expect("read minified bundle");

        assert!(bundled.contains("Alpha,#[allow"), "{bundled}");
        syn::parse_file(&bundled)
            .unwrap_or_else(|e| panic!("minified bundle is not valid Rust: {e}\n{bundled}"));
    }

    /// `x & &y` must not collapse into the logical `&&`, and `a && b` must be kept.
    #[test]
    fn aggressive_minify_distinguishes_bitand_from_logical_and() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("amp"),
            &[(
                "src/main.rs",
                "fn main() {\n    let (x, y) = (6u8, 3u8);\n    let (a, b) = (true, false);\n    println!(\"{} {}\", x & &y, a && b);\n}\n",
            )],
        );

        cargo_bin_cmd!("cg-bundler")
            .current_dir(temp.path())
            .args(["--m2", "-o", "out.rs"])
            .assert()
            .success();

        let bundled = fs::read_to_string(temp.path().join("out.rs")).expect("read minified bundle");

        assert!(
            bundled.contains("x & & y"),
            "bitwise and of a reference must not become `&&`:\n{bundled}"
        );
        assert!(
            bundled.contains("a&&b"),
            "logical and must stay collapsed:\n{bundled}"
        );
        syn::parse_file(&bundled)
            .unwrap_or_else(|e| panic!("minified bundle is not valid Rust: {e}\n{bundled}"));
    }

    /// `--keep-docs` plus minification used to emit a `///` comment on the single
    /// output line, commenting out everything that followed it.
    #[test]
    fn minify_keeps_doc_comments_without_swallowing_the_file() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("docs_minified"),
            &[(
                "src/main.rs",
                "//! Crate level docs.\n\
                 /// Doubles `value`.\n\
                 pub fn double(value: i32) -> i32 { value * 2 }\n\
                 fn main() { println!(\"{}\", double(21)); }\n",
            )],
        );

        for flags in [["-m", "--keep-docs"], ["--m2", "--keep-docs"]] {
            cargo_bin_cmd!("cg-bundler")
                .current_dir(temp.path())
                .args(flags)
                .args(["-o", "out.rs"])
                .assert()
                .success();

            let bundled =
                fs::read_to_string(temp.path().join("out.rs")).expect("read minified bundle");

            assert!(
                !bundled.contains("///"),
                "doc comments must be re-encoded as attributes for {flags:?}:\n{bundled}"
            );
            assert!(
                bundled.contains("Doubles"),
                "doc text must survive for {flags:?}:\n{bundled}"
            );
            assert!(
                bundled.contains("fn main"),
                "code after a doc comment must not be commented out for {flags:?}:\n{bundled}"
            );
            syn::parse_file(&bundled).unwrap_or_else(|e| {
                panic!("minified bundle is not valid Rust for {flags:?}: {e}\n{bundled}")
            });
        }
    }

    /// `a / *b` must not collapse into `/*`, which opens a block comment.
    #[test]
    fn aggressive_minify_keeps_division_by_dereference_apart() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("slash_star"),
            &[(
                "src/main.rs",
                "fn divide(a: u64, b: &u64) -> u64 { a / *b }\n\
                 fn main() { println!(\"{}\", divide(8, &4)); }\n",
            )],
        );

        cargo_bin_cmd!("cg-bundler")
            .current_dir(temp.path())
            .args(["--m2", "-o", "out.rs"])
            .assert()
            .success();

        let bundled = fs::read_to_string(temp.path().join("out.rs")).expect("read minified bundle");

        assert!(
            !bundled.contains("/*"),
            "division by a dereference must not open a block comment:\n{bundled}"
        );
        syn::parse_file(&bundled)
            .unwrap_or_else(|e| panic!("minified bundle is not valid Rust: {e}\n{bundled}"));
    }

    /// `(x,)` is a one-element tuple; rewriting `,)` to `)` silently turned it
    /// into a parenthesised expression, which still compiles but means something
    /// different.
    #[test]
    fn aggressive_minify_preserves_one_element_tuples() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("one_tuple"),
            &[(
                "src/main.rs",
                "fn pair() -> (i32,) { (42,) }\n\
                 fn main() { println!(\"{:?}\", pair()); }\n",
            )],
        );

        cargo_bin_cmd!("cg-bundler")
            .current_dir(temp.path())
            .args(["--m2", "-o", "out.rs"])
            .assert()
            .success();

        let bundled = fs::read_to_string(temp.path().join("out.rs")).expect("read minified bundle");

        assert!(
            bundled.contains("(42,)") && bundled.contains("(i32,)"),
            "one-element tuple must keep its comma:\n{bundled}"
        );
        syn::parse_file(&bundled)
            .unwrap_or_else(|e| panic!("minified bundle is not valid Rust: {e}\n{bundled}"));
    }

    /// `x < <T as Trait>::CONST` must not collapse into the shift operator `<<`.
    #[test]
    fn aggressive_minify_keeps_qualified_path_after_less_than() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("lt_qpath"),
            &[(
                "src/main.rs",
                "trait Limit { const MAX: i32; }\n\
                 struct S;\n\
                 impl Limit for S { const MAX: i32 = 10; }\n\
                 fn under(x: i32) -> bool { x < <S as Limit>::MAX }\n\
                 fn main() { println!(\"{} {}\", under(3), 1i32 << 2); }\n",
            )],
        );

        cargo_bin_cmd!("cg-bundler")
            .current_dir(temp.path())
            .args(["--m2", "-o", "out.rs"])
            .assert()
            .success();

        let bundled = fs::read_to_string(temp.path().join("out.rs")).expect("read minified bundle");

        assert!(
            bundled.contains("x < <S as Limit>::MAX"),
            "comparison against a qualified path must not become a shift:\n{bundled}"
        );
        assert!(
            bundled.contains("1i32<<2"),
            "a genuine shift must stay collapsed:\n{bundled}"
        );
        syn::parse_file(&bundled)
            .unwrap_or_else(|e| panic!("minified bundle is not valid Rust: {e}\n{bundled}"));
    }

    /// Block comments survive line joining, but their interior used to be
    /// rewritten by punctuation tightening -- which could move the `*/`
    /// terminator -- and to have string placeholders substituted inside it.
    #[test]
    fn minify_reencodes_block_doc_comments() {
        let temp = TempDir::new().expect("temp dir");
        write_project(
            temp.path(),
            &simple_manifest("block_docs"),
            &[(
                "src/main.rs",
                "/** Block docs spanning\n * a line break with a tricky * / sequence\n */\n\
                 pub fn tricky() -> i32 { 7 }\n\
                 fn main() { println!(\"{}\", tricky()); }\n",
            )],
        );

        for flags in [["-m", "--keep-docs"], ["--m2", "--keep-docs"]] {
            cargo_bin_cmd!("cg-bundler")
                .current_dir(temp.path())
                .args(flags)
                .args(["-o", "out.rs"])
                .assert()
                .success();

            let bundled =
                fs::read_to_string(temp.path().join("out.rs")).expect("read minified bundle");

            assert!(
                !bundled.contains("/*"),
                "block comments must be re-encoded as attributes for {flags:?}:\n{bundled}"
            );
            assert!(
                bundled.contains("Block docs spanning"),
                "doc text must survive for {flags:?}:\n{bundled}"
            );
            syn::parse_file(&bundled).unwrap_or_else(|e| {
                panic!("minified bundle is not valid Rust for {flags:?}: {e}\n{bundled}")
            });
        }
    }
}
