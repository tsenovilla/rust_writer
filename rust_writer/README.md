[<img alt="CI Workflow" src="https://img.shields.io/github/actions/workflow/status/tsenovilla/rust_writer/ci.yml?branch=main&style=for-the-badge&logo=github&label=CI" height="20">](https://github.com/tsenovilla/rust_writer/actions/workflows/ci.yml)
[<img alt="Codecov" src="https://img.shields.io/codecov/c/github/tsenovilla/rust_writer?style=for-the-badge&logo=codecov" height="20">](https://codecov.io/gh/tsenovilla/rust_writer)
[<img alt="Crates.io" src="https://img.shields.io/crates/v/rust_writer.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/rust_writer)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rust_writer-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/rust_writer)

# Description 📖📚

[The crate docs](https://docs.rs/rust_writer/latest/rust_writer) should be considered the only source of truth for this crate usage.

`rust_writer` is a crate designed to simplify meta programming in Rust—but in a different way than the typical procedural macro crates (such as [syn](https://docs.rs/syn/latest/syn/), [quote](https://docs.rs/quote/latest/quote/), and [proc_macro2](https://docs.rs/proc-macro2/latest/proc_macro2/)).  

While those crates excel at writing procedural macros, `rust_writer` leverages their capabilities to modify Rust source files. This makes it ideal for tasks that require in-place source code modifications rather than merely generating new code visible only to the compiler.

The crate is divided into two modules: the [preserver](https://docs.rs/rust_writer/latest/rust_writer/preserver/) module and the [ast](https://docs.rs/rust_writer/latest/rust_writer/ast/) module. Although these modules can be used separately and even for purposes other than the crate's primary objective, using them together unlocks the full potential of the crate.

- The [preserver](https://docs.rs/rust_writer/latest/rust_writer/preserver/) module ensures that the original structure of the source code is maintained when it is parsed into an AST.
- The [ast](https://docs.rs/rust_writer/latest/rust_writer/ast/) module provides various tools to simplify AST interactions, allowing precise modifications exactly where needed.

For further details, please refer to the individual module documentation. A complete example is often the best way to illustrate the functionality:

```rust
use quote::quote;
use rust_writer::{
    ast::{
        implementors::{ItemToFile, ItemToImpl, TokenStreamToMacro},
        mutator,
        mutator::{Mutator, ToMutate},
    },
    preserver::Preserver,
};
use syn::{parse_quote, ImplItem, Item, visit_mut::VisitMut};
use test_builder::TestBuilder;

// A mutator defined with the given implementors.
#[mutator(TokenStreamToMacro, ItemToFile, ItemToImpl<'a>)]
#[impl_from]
struct TestMutator;

// In this example, complete file can be found at https://github.com/tsenovilla/rust_writer/blob/main/test_builder/sample_files/complete_file.rs,
// while expanded_file can be found at https://github.com/tsenovilla/rust_writer/blob/main/test_builder/sample_files/expanded_file.rs
TestBuilder::default()
    .with_complete_file()
    .with_expanded_file()
    .execute(|builder| {
        let complete_file_path = builder.tempfile_path("complete_file.rs")
            .expect("This file exists");
        let expanded_file_path = builder.tempfile_path("expanded_file.rs")
            .expect("This file exists");
        let expected_code = std::fs::read_to_string(&expanded_file_path)
            .expect("File should be readable");

        // Preserve an impl block and the invocation of `my_macro` in the code.
        // The rest of the file remains unchanged, preserving its original AST structure.
        let preserver1 = Preserver::new("impl MyTrait for MyStruct");
        let mut preserver2 = Preserver::new("fn main");
        preserver2.add_inners(&["my_macro"]);

        let mut ast = rust_writer::preserver::preserve_and_parse(
            complete_file_path,
            &[&preserver1, &preserver2],
        )
        .expect("Preservation should be applied");

        // Add a function to the trait implementation.
        // Note the TEMP_DOC comment: it will become an empty line in the final code,
        // which is a neat trick to insert an empty line at the start of the function.
        let item_to_impl: ItemToImpl = (
            Some("MyTrait"),
            "MyStruct",
            ImplItem::Fn(parse_quote! {
                ///TEMP_DOC
                fn func(&self) -> bool {
                    false
                }
            }),
        )
        .into();

        // Add a TokenStream to the `my_macro` invocation.
        let token_stream_to_macro: TokenStreamToMacro = (
            parse_quote!(my_macro),
            None,
            quote! {
                struct SomeStruct {
                    field: u8,
                    string: String
                }
            },
        )
        .into();

        // Insert an item into the file so that `Path` is in scope.
        let item: Item = parse_quote!( use std::path::Path; );
        let item_to_file: ItemToFile = item.into();

        // Create a mutator for the given implementors.
        let test_mutator: TestMutator =
            (token_stream_to_macro, item_to_file, item_to_impl).into();
        let mut mutator: TestMutatorMutatorWrapper =
            Mutator::default().to_mutate(&test_mutator).into();

        // Mutate the AST.
        assert!(mutator.mutate(&mut ast, None).is_ok());

        // Unparse the AST and restore the preserved code.
        assert!(rust_writer::preserver::resolve_preserved(&ast, complete_file_path).is_ok());

        let actual_code = std::fs::read_to_string(complete_file_path)
            .expect("File should be readable");
        assert_eq!(actual_code, expected_code);
    });
```

# Contributing 🤝🚀

Any contribution is more than welcome! 🤝🦾 Just open a PR with your changes and it'll be considered 😸
