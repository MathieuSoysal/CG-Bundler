use cg_bundler::transformer::{CodeTransformer, TransformConfig};
use std::path::PathBuf;

#[test]
fn test_transform_config_default() {
    let config = TransformConfig::default();
    assert!(config.remove_tests);
    assert!(config.remove_docs);
    assert!(config.expand_modules);
    assert!(!config.minify);
    assert!(!config.aggressive_minify);
}

#[test]
fn test_is_doc_attribute() {
    let base_path = PathBuf::from("/tmp");
    let _transformer = CodeTransformer::new(&base_path, "test_crate", TransformConfig::default());

    // Test with a doc attribute
    let doc_attr: syn::Attribute = syn::parse_quote!(#[doc = "test"]);
    assert!(CodeTransformer::is_doc_attribute(&doc_attr));

    // Test with a non-doc attribute
    let non_doc_attr: syn::Attribute = syn::parse_quote!(#[test]);
    assert!(!CodeTransformer::is_doc_attribute(&non_doc_attr));
}

#[test]
fn test_has_test_attribute() {
    let base_path = PathBuf::from("/tmp");
    let _transformer = CodeTransformer::new(&base_path, "test_crate", TransformConfig::default());

    // Test function with test attribute
    let test_fn: syn::Item = syn::parse_quote! {
        #[test]
        fn test_function() {}
    };
    assert!(CodeTransformer::has_test_attribute(&test_fn));

    // Test regular function
    let regular_fn: syn::Item = syn::parse_quote! {
        fn regular_function() {}
    };
    assert!(!CodeTransformer::has_test_attribute(&regular_fn));
}

#[test]
fn test_expand_crate_path_single_segment_not_stripped() {
    // Regression test for issue #51:
    // A single-segment path whose name matches the crate name (e.g., a function
    // parameter named after the crate) must NOT be stripped, as that would produce
    // an empty path and cause `prettyplease` to panic.
    let base_path = PathBuf::from("/tmp");
    let transformer = CodeTransformer::new(&base_path, "mycrate", TransformConfig::default());

    // Single-segment path "mycrate" (local variable / parameter usage)
    let mut path: syn::Path = syn::parse_quote!(mycrate);
    transformer.expand_crate_path(&mut path);
    // Must still have one segment — not stripped to empty
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].ident.to_string(), "mycrate");

    // Multi-segment path "mycrate::SomeType" must be stripped to "SomeType"
    let mut path2: syn::Path = syn::parse_quote!(mycrate::SomeType);
    transformer.expand_crate_path(&mut path2);
    assert_eq!(path2.segments.len(), 1);
    assert_eq!(path2.segments[0].ident.to_string(), "SomeType");
}

#[test]
fn test_is_extern_crate() {
    let base_path = PathBuf::from("/tmp");
    let _transformer = CodeTransformer::new(&base_path, "test_crate", TransformConfig::default());

    // Test extern crate with matching name
    let extern_crate_item: syn::Item = syn::parse_quote! {
        extern crate test_crate;
    };
    assert!(CodeTransformer::is_extern_crate(
        &extern_crate_item,
        "test_crate"
    ));

    // Test extern crate with different name
    assert!(!CodeTransformer::is_extern_crate(
        &extern_crate_item,
        "other_crate"
    ));

    // Test non-extern-crate item
    let fn_item: syn::Item = syn::parse_quote! {
        fn test() {}
    };
    assert!(!CodeTransformer::is_extern_crate(&fn_item, "test_crate"));
}

// ── is_use_path (Name / Rename variants) ──────────────────────────────

#[test]
fn test_is_use_path_variants() {
    // Path variant: use test_crate::something;
    let use_path: syn::Item = syn::parse_quote! { use test_crate::something; };
    assert!(CodeTransformer::is_use_path(&use_path, "test_crate"));
    assert!(!CodeTransformer::is_use_path(&use_path, "other"));

    // Name variant: use test_crate;
    let use_name: syn::Item = syn::parse_quote! { use test_crate; };
    assert!(CodeTransformer::is_use_path(&use_name, "test_crate"));
    assert!(!CodeTransformer::is_use_path(&use_name, "other"));

    // Rename variant: use test_crate as tc;
    let use_rename: syn::Item = syn::parse_quote! { use test_crate as tc; };
    assert!(CodeTransformer::is_use_path(&use_rename, "test_crate"));
    assert!(!CodeTransformer::is_use_path(&use_rename, "other"));

    // Non-use item should return false
    let fn_item: syn::Item = syn::parse_quote! { fn foo() {} };
    assert!(!CodeTransformer::is_use_path(&fn_item, "test_crate"));
}

// ── has_test_attribute for struct/enum/trait/impl ──────────────────────

#[test]
fn test_has_test_attribute_various_item_kinds() {
    // cfg(test) on a struct
    let cfg_struct: syn::Item = syn::parse_quote! {
        #[cfg(test)]
        struct S {}
    };
    assert!(CodeTransformer::has_test_attribute(&cfg_struct));

    // cfg(test) on an enum
    let cfg_enum: syn::Item = syn::parse_quote! {
        #[cfg(test)]
        enum E { A }
    };
    assert!(CodeTransformer::has_test_attribute(&cfg_enum));

    // cfg(test) on a trait
    let cfg_trait: syn::Item = syn::parse_quote! {
        #[cfg(test)]
        trait T {}
    };
    assert!(CodeTransformer::has_test_attribute(&cfg_trait));

    // cfg(test) on an impl block
    let cfg_impl: syn::Item = syn::parse_quote! {
        #[cfg(test)]
        impl S {}
    };
    assert!(CodeTransformer::has_test_attribute(&cfg_impl));

    // Other item kind (const) – returns false
    let const_item: syn::Item = syn::parse_quote! { const X: i32 = 1; };
    assert!(!CodeTransformer::has_test_attribute(&const_item));
}

// ── filter_tests_and_docs ──────────────────────────────────────────────

#[test]
fn test_filter_tests_and_docs_with_keep_config() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        remove_tests: false,
        remove_docs: false,
        expand_modules: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);
    let code = "
        #[test] fn t() {}
        /// doc
        fn f() {}
    ";
    let mut file = syn::parse_file(code).unwrap();
    transformer.transform_file(&mut file).unwrap();
    let out = prettyplease::unparse(&file);
    // With remove_tests=false the test fn must survive
    assert!(out.contains("fn t"));
    // With remove_docs=false the doc comment survives
    assert!(out.contains("doc"));
}

// ── remove_doc_from_children (Trait, Impl, Fn, Mod, Enum variants) ─────

#[test]
fn test_remove_doc_from_children_trait_and_impl() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        remove_docs: true,
        expand_modules: false,
        remove_tests: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);

    // Trait with documented methods
    let code = "
        trait MyTrait {
            /// doc
            fn method(&self);
            /// const doc
            const C: i32;
            /// type doc
            type T;
        }
    ";
    let mut file = syn::parse_file(code).unwrap();
    transformer.transform_file(&mut file).unwrap();
    let out = prettyplease::unparse(&file);
    assert!(
        !out.contains("doc"),
        "Trait member docs should be removed: {out}"
    );

    // Impl with documented methods and associated consts/types
    let code2 = "
        struct S;
        impl S {
            /// method doc
            fn m(&self) {}
            /// const doc
            const X: i32 = 0;
            /// type doc
            type T = i32;
        }
    ";
    let mut file2 = syn::parse_file(code2).unwrap();
    transformer.transform_file(&mut file2).unwrap();
    let out2 = prettyplease::unparse(&file2);
    assert!(
        !out2.contains("doc"),
        "Impl member docs should be removed: {out2}"
    );
}

#[test]
fn test_remove_doc_from_children_fn_inputs() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        remove_docs: true,
        expand_modules: false,
        remove_tests: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);

    let code = "
        fn f(x: i32) -> i32 { x }
    ";
    let mut file = syn::parse_file(code).unwrap();
    transformer.transform_file(&mut file).unwrap();
    // If it doesn't panic it's fine – fn input doc removal ran
}

#[test]
fn test_remove_doc_from_children_mod_and_enum() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        remove_docs: true,
        expand_modules: false,
        remove_tests: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);

    // Inline mod with docs inside
    let code = "
        mod inner {
            /// inner fn doc
            fn g() {}
        }
    ";
    let mut file = syn::parse_file(code).unwrap();
    transformer.transform_file(&mut file).unwrap();
    let out = prettyplease::unparse(&file);
    assert!(!out.contains("inner fn doc"));

    // Enum with documented variants and tuple/struct fields
    let code2 = "
        enum Color {
            /// red doc
            Red,
            /// green doc
            Green(u8),
            /// blue doc
            Blue { intensity: u8 },
        }
    ";
    let mut file2 = syn::parse_file(code2).unwrap();
    transformer.transform_file(&mut file2).unwrap();
    let out2 = prettyplease::unparse(&file2);
    assert!(
        !out2.contains("red doc") && !out2.contains("green doc") && !out2.contains("blue doc"),
        "Enum variant docs should be removed: {out2}"
    );
}

// ── remove_doc_attributes item kinds (Type/Const/Static/Use/ExternCrate) ─

#[test]
fn test_remove_doc_attributes_all_item_kinds() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        remove_docs: true,
        expand_modules: false,
        remove_tests: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);

    let code = "
        /// type alias doc
        type Alias = i32;
        /// const doc
        const C: i32 = 0;
        /// static doc
        static S: i32 = 0;
    ";
    let mut file = syn::parse_file(code).unwrap();
    transformer.transform_file(&mut file).unwrap();
    let out = prettyplease::unparse(&file);
    assert!(
        !out.contains("type alias doc")
            && !out.contains("const doc")
            && !out.contains("static doc"),
        "Docs should be stripped for Type/Const/Static items: {out}"
    );
}

// ── expand_extern_crate path ───────────────────────────────────────────

#[test]
fn test_expand_extern_crate_finds_lib_rs() {
    use std::fs;
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    // Write a lib.rs that will be inlined
    fs::write(base.join("lib.rs"), "pub fn from_lib() {}").unwrap();

    let config = TransformConfig {
        expand_modules: true,
        remove_tests: true,
        remove_docs: true,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "mylib", config);

    let code = "extern crate mylib; fn main() {}";
    let mut file = syn::parse_file(code).unwrap();
    let result = transformer.transform_file(&mut file);
    assert!(
        result.is_ok(),
        "expand_extern_crate should succeed: {result:?}"
    );
    let out = prettyplease::unparse(&file);
    assert!(
        out.contains("from_lib"),
        "lib.rs content should be inlined: {out}"
    );
}

#[test]
fn test_expand_extern_crate_missing_lib_rs_returns_error() {
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    // No lib.rs → should return ProjectStructure error
    let config = TransformConfig {
        expand_modules: true,
        remove_tests: false,
        remove_docs: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(tmp.path(), "mylib", config);

    let code = "extern crate mylib;";
    let mut file = syn::parse_file(code).unwrap();
    let result = transformer.transform_file(&mut file);
    assert!(result.is_err(), "Should error when lib.rs is missing");
}

// ── expand_use_path (library inlined from use statement) ──────────────

#[test]
fn test_expand_use_path_inlines_lib() {
    use std::fs;
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    fs::write(base.join("lib.rs"), "pub fn helper() {}").unwrap();

    let config = TransformConfig {
        expand_modules: true,
        remove_tests: true,
        remove_docs: true,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "mylib", config);

    // Use statement referencing the crate should inline lib.rs
    let code = "use mylib::helper; fn main() { helper(); }";
    let mut file = syn::parse_file(code).unwrap();
    let result = transformer.transform_file(&mut file);
    assert!(result.is_ok(), "expand_use_path should succeed: {result:?}");
    let out = prettyplease::unparse(&file);
    assert!(out.contains("fn helper"), "lib.rs should be inlined: {out}");
}

#[test]
fn test_expand_use_path_missing_lib_returns_error() {
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let config = TransformConfig {
        expand_modules: true,
        remove_tests: false,
        remove_docs: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(tmp.path(), "mylib", config);

    let code = "use mylib::something;";
    let mut file = syn::parse_file(code).unwrap();
    let result = transformer.transform_file(&mut file);
    assert!(
        result.is_err(),
        "Should error when lib.rs is missing for use path"
    );
}

// ── expand_mods with pre-inlined content ──────────────────────────────

#[test]
fn test_expand_mods_already_has_content() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        expand_modules: true,
        remove_tests: false,
        remove_docs: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);

    // Inline mod body – expand_mods must return Ok immediately (content.is_some())
    let code = "
        mod inner {
            pub fn f() {}
        }
    ";
    let mut file = syn::parse_file(code).unwrap();
    let result = transformer.transform_file(&mut file);
    assert!(result.is_ok());
}

// ── visit_file_mut (VisitMut impl) ────────────────────────────────────

#[test]
fn test_visit_file_mut_removes_file_level_docs() {
    use syn::visit_mut::VisitMut;

    let base = PathBuf::from("/tmp");
    let mut transformer = CodeTransformer::new(
        &base,
        "c",
        TransformConfig {
            remove_docs: true,
            expand_modules: false,
            remove_tests: false,
            ..TransformConfig::default()
        },
    );

    // File-level inner doc attribute
    let mut file: syn::File = syn::parse_quote! {
        #![doc = "crate-level doc"]
        fn foo() {}
    };
    transformer.visit_file_mut(&mut file);
    let out = prettyplease::unparse(&file);
    assert!(!out.contains("crate-level doc"));
}

// ── expand_external_lib ───────────────────────────────────────────────

#[test]
fn test_expand_external_lib_inlines_dependency() {
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let ext_dir = tmp.path().join("ext_crate");
    fs::create_dir_all(&ext_dir).unwrap();
    fs::write(ext_dir.join("lib.rs"), "pub fn ext_fn() {}").unwrap();

    let mut external_libs = HashMap::new();
    external_libs.insert("ext_crate".to_string(), ext_dir.join("lib.rs"));

    let config = TransformConfig {
        expand_modules: false,
        remove_tests: true,
        remove_docs: true,
        ..TransformConfig::default()
    };
    let base = tmp.path().to_path_buf();
    let mut transformer =
        CodeTransformer::with_external_libs(&base, "myapp", config, external_libs);

    // Provide a use statement referencing the external crate to trigger expansion
    let code = "use ext_crate::ext_fn; fn main() {}";
    let mut file = syn::parse_file(code).unwrap();
    // expand_items is called inside transform_file
    let result = transformer.transform_file(&mut file);
    assert!(
        result.is_ok(),
        "External lib expansion should succeed: {result:?}"
    );
    let out = prettyplease::unparse(&file);
    assert!(
        out.contains("ext_fn"),
        "External lib content should be inlined: {out}"
    );
}

#[test]
fn test_expand_external_lib_missing_file_returns_error() {
    use std::collections::HashMap;
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let mut external_libs = HashMap::new();
    external_libs.insert(
        "ext_crate".to_string(),
        tmp.path().join("nonexistent/lib.rs"),
    );

    let config = TransformConfig {
        expand_modules: false,
        remove_tests: false,
        remove_docs: false,
        ..TransformConfig::default()
    };
    let mut transformer =
        CodeTransformer::with_external_libs(tmp.path(), "myapp", config, external_libs);

    let code = "use ext_crate::something;";
    let mut file = syn::parse_file(code).unwrap();
    let result = transformer.transform_file(&mut file);
    assert!(
        result.is_err(),
        "Should fail when external lib file is missing"
    );
}

// ── expand_mods (module file on disk) ─────────────────────────────────

#[test]
fn test_expand_mods_loads_from_file() {
    use std::fs;
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    fs::write(base.join("utils.rs"), "pub fn util_fn() {}").unwrap();

    let config = TransformConfig {
        expand_modules: true,
        remove_tests: false,
        remove_docs: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "myapp", config);

    let code = "mod utils; fn main() {}";
    let mut file = syn::parse_file(code).unwrap();
    let result = transformer.transform_file(&mut file);
    assert!(result.is_ok(), "Should expand module from file: {result:?}");
    let out = prettyplease::unparse(&file);
    assert!(
        out.contains("util_fn"),
        "Module content should be inlined: {out}"
    );
}

// ── filter_tests_and_docs: remove_tests=true removes test items ────────

#[test]
fn test_filter_tests_removes_test_annotated_items() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        remove_tests: true,
        remove_docs: false,
        expand_modules: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);
    let code = "
        #[test]
        fn my_test() {}
        fn regular_fn() {}
    ";
    let mut file = syn::parse_file(code).unwrap();
    transformer.transform_file(&mut file).unwrap();
    let out = prettyplease::unparse(&file);
    assert!(
        !out.contains("my_test"),
        "Test fn should be filtered: {out}"
    );
    assert!(
        out.contains("regular_fn"),
        "Regular fn should remain: {out}"
    );
}

// ── remove_docs_from_fields with actual field attributes ───────────────

#[test]
fn test_remove_docs_from_named_fields_with_attrs() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        remove_docs: true,
        expand_modules: false,
        remove_tests: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);
    // Struct with doc-commented named fields
    let code = "
        struct Foo {
            /// x doc
            x: i32,
            /// y doc
            y: String,
        }
    ";
    let mut file = syn::parse_file(code).unwrap();
    transformer.transform_file(&mut file).unwrap();
    let out = prettyplease::unparse(&file);
    assert!(
        !out.contains("x doc") && !out.contains("y doc"),
        "Named field docs should be removed: {out}"
    );
}

#[test]
fn test_remove_docs_from_unnamed_fields_with_attrs() {
    let base = PathBuf::from("/tmp");
    let config = TransformConfig {
        remove_docs: true,
        expand_modules: false,
        remove_tests: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(&base, "c", config);
    // Tuple struct with doc attribute on its unnamed field
    let code = r#"
        struct Bar(#[doc = "tuple field doc"] i32);
    "#;
    let mut file = syn::parse_file(code).unwrap();
    transformer.transform_file(&mut file).unwrap();
    let out = prettyplease::unparse(&file);
    assert!(
        !out.contains("tuple field doc"),
        "Unnamed field docs should be removed: {out}"
    );
}

// ── use_tree_references_crate: Glob and Group branches ─────────────────

#[test]
fn test_use_tree_references_crate_glob_and_group() {
    // Glob tree at top level should return false
    let glob = syn::UseTree::Glob(syn::UseGlob {
        star_token: syn::token::Star::default(),
    });
    assert!(!CodeTransformer::use_tree_references_crate(
        &glob,
        "any_crate"
    ));

    // Group tree at top level should return false
    let name_tree = syn::UseTree::Name(syn::UseName {
        ident: syn::Ident::new("foo", proc_macro2::Span::call_site()),
    });
    let group = syn::UseTree::Group(syn::UseGroup {
        brace_token: syn::token::Brace::default(),
        items: std::iter::once(name_tree).collect(),
    });
    assert!(!CodeTransformer::use_tree_references_crate(
        &group,
        "any_crate"
    ));
}

// ── visit_file_mut: error path (expand_items fails) ───────────────────

#[test]
fn test_visit_file_mut_expand_items_error_is_printed_not_panicked() {
    use syn::visit_mut::VisitMut;
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    // Create a transformer with expand_modules=true but no lib.rs,
    // and provide a file with a use statement for the crate so expand_items errors.
    let config = TransformConfig {
        expand_modules: true,
        remove_docs: false,
        remove_tests: false,
        ..TransformConfig::default()
    };
    let mut transformer = CodeTransformer::new(tmp.path(), "mylib", config);
    // This use statement causes expand_use_path to be attempted (lib.rs missing → error)
    let mut file: syn::File = syn::parse_quote! {
        use mylib::something;
    };
    // visit_file_mut catches errors from expand_items and eprints them instead of panicking
    transformer.visit_file_mut(&mut file);
    // If we reach here without panic, the error path was handled correctly
}
