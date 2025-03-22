// SPDX-License-Identifier: GPL-3.0

//! This module provides utilities to “preserve” portions of a Rust source file
//! during parsing so that empty lines, comments, and certain attributes are not lost.
//!
//! The preservation process converts these elements into temporary doc comments (starting with
//! ///TEMP_DOC) and injects helper markers. This approach better preserves the original source code
//! structure, ensuring that all parts of the source (including non-code elements) are present in
//! the parsed AST.
//!
//! The public API consists of two functions:
//!
//! - [`preserve_and_parse`]: Reads a source file, applies preservation via a list of provided [`Preserver`], and
//!   parses the resulting code into a [`syn::File`](https://docs.rs/syn/latest/syn/struct.File.html).
//!   This function ensures that the full source code structure is retained during parsing.
//!
//! - [`resolve_preserved`]: Takes a preserved AST, un-parses it back to source code using [`prettyplease::unparse`](https://docs.rs/prettyplease/latest/prettyplease/fn.unparse.html),
//!   and then restores the preserved comments and removes temporary markers.
//!
//! # Terminology
//!
//! The `Preserver` type specifies which parts of the code should remain unchanged, those parts are
//! called "preserved code". All the rest is considered non-preserved code.
//!
//! Preserved code is the code that will be parse in the AST in its original form, ie, it's
//! preserved for the AST.
//!
//! The naming may be a bit confusing, cause in practice, non-preserved code becomes a doc comment
//! in the way described above, so non-preserved code may be considered preserved in the sense that
//! its structure will be untouched when parsing the AST, but this isn't the sense in which this
//! term is applied in this crate.
//!
//! # Example
//!
//! ```rust
//! use test_builder::TestBuilder;
//! use rust_writer::preserver::Preserver;
//!
//! TestBuilder::default()
//!     .with_complete_file()
//!     .with_resolved_file()
//!     .with_preserved_file_ast()
//!     .execute(|builder| {
//!         let complete_file_path = builder.tempfile_path("complete_file.rs")
//!             .expect("This exists; qed;");
//!
//!         let preserved_ast = builder.get_ref_ast_file("preserved_file.rs")
//!             .expect("This exists; qed;");
//!
//!         let preserver1 = Preserver::new("struct MyStruct");
//!         let mut preserver2 = Preserver::new("impl MyTrait for MyStruct");
//!         preserver2.add_inners(&["fn trait_method"]);
//!         let preserver3 = Preserver::new("fn main");
//!
//!         assert_eq!(
//!             *preserved_ast,
//!             rust_writer::preserver::preserve_and_parse(
//!                 builder.tempfile_path("complete_file.rs").expect("This exists; qed;"),
//!                 &[&preserver1, &preserver2, &preserver3]
//!             )
//!             .expect("This should be Ok; qed;")
//!         );
//!
//!         // While the code in "resolved_file" and "complete_file" is the same, the formatting
//!         // isn't the same cause there's a preserved declarative macro with enough
//!         // complexity, so we cannot compare the resolved directly with the original.
//!         // Check both files in the repo to see their formatting differences.
//!         let resolved_file_path = builder.tempfile_path("resolved_file.rs")
//!             .expect("This exists; qed;");
//!
//!         let expected_code = std::fs::read_to_string(&resolved_file_path)
//!             .expect("File should be readable");
//!
//!         assert!(rust_writer::preserver::resolve_preserved(
//!             builder.get_ref_ast_file("preserved_file.rs")
//!                 .expect("This exists; qed;"),
//!             complete_file_path
//!         )
//!         .is_ok());
//!
//!         let actual_code = std::fs::read_to_string(complete_file_path)
//!             .expect("File should be readable");
//!
//!         assert_eq!(actual_code, expected_code);
//!     });
//! ```

mod types;

#[cfg(test)]
mod tests;

use crate::Error;
use regex::{Captures, Regex};
use std::path::Path;
use syn::File;
use types::DelimitersCount;
pub use types::Preserver;

/// Reads the Rust source file at the given `code` path, applies the specified
/// preservation strategies (via the list of [`Preserver`]), and parses the resulting
/// code into a [`syn::File`]. All the preserved code becomes a doc comment starting by
/// ///TEMP_DOC.
///
/// All the code that isn't inside a preserved block (identified by a `Preserved`) becomes
/// non-preserved code, which means that this code becomes a doc comment in the way described above.
///
/// This function is useful to ensure that non-code elements such as comments,
/// empty lines, and global attributes are not lost during parsing. By converting these parts
/// into doc comment tokens, the overall source code structure is better preserved, which can
/// simplify later processing and transformations.
///
/// # Non preservable code
///
/// Inside preserved code, empty lines and comments become doc comments in order to keep them in
/// the AST. Those doc comments must document something to stay in the AST, so a temporal marker
/// `type temp_marker = ();` is added to achieve that. While this is enough in many cases, it has a
/// counterpart.
///
/// Imagine that this struct is preserved code:
///
/// ```no_compile
/// struct MyStruct {
/// // Invalid comment
/// field1: i32,
/// field2: String,
/// }
/// ```
///
/// It will result in the following struct:
///
/// ```no_compile
/// struct MyStruct {
/// ///TEMP_DOC // Invalid comment
/// type temp_marker = ();
/// field1: i32,
/// field2: String,
/// }
/// ```
///
/// which is invalid Rust code, impossible to be parsed into an AST. A file containing this lines
/// would be deemed as "non preservable code" if those lines are preserved. But it's perfectly
/// valid if those lines are non-preserved, so just pay attention to preserved code.
///
/// The following snippet shows illustrates it:
///
/// ```rust
/// use test_builder::TestBuilder;
/// use rust_writer::{preserver::Preserver, Error};
/// TestBuilder::default().with_non_preservable_file().execute(|builder| {
///  let preserver1 = Preserver::new("struct MyStruct");
///  let mut preserver2 = Preserver::new("impl MyTrait for MyStruct");
///  preserver2.add_inners(&["fn trait_method"]);
///  let preserver3 = Preserver::new("fn main");
///
///  assert!(matches!(
///   rust_writer::preserver::preserve_and_parse(
///      builder.tempfile_path("non_preservable_file.rs").expect("This exists; qed;"),
///      &[&preserver1, &preserver2, &preserver3]
///   ),
///   Err(Error::NonPreservableCode)
///  ));
///
///  assert!(rust_writer::preserver::preserve_and_parse(
///    builder.tempfile_path("non_preservable_file.rs").expect("This exists;qed"),
///    &[&preserver2, &preserver3]
///  ).is_ok());
/// });
/// ```
pub fn preserve_and_parse(code: &Path, preservers: &[&Preserver]) -> Result<File, Error> {
	let preserved_code = apply_preservers(&std::fs::read_to_string(code)?, preservers);
	syn::parse_file(&preserved_code).map_err(|_| Error::NonPreservableCode)
}

/// Resolves a previously preserved and parsed AST back into source code and writes it to the
/// specified `path`.
///
/// This function un-parses the given AST using [`prettyplease::unparse`] and resolves all the
/// changes applied by [`preserve_and_parse`], cleaning up all temporary doc comments and markers.
///
/// # Known limitations
///
/// The resolution process uses [`prettyplease::unparse`] to convert
/// the AST back into source code. While this approach generally preserves the overall structure of
/// the code, unparsing preserved declarative macro invocations (especially those that are
/// complex) can sometimes lead to formatting differences from the original source. This is a
/// well-known challenge in the Rust parsing ecosystem, and something to keep in mind.
pub fn resolve_preserved(ast: &File, path: &Path) -> Result<(), Error> {
	let code = prettyplease::unparse(ast);
	// Inside preserved declarative macros invocations, everything is a token so the doc
	// comments became #[doc] in order to preserve them (tokens doesn't accept doc comments).
	// ///TEMP_DOC comments became #[doc = "///TEMP_DOC"] which are 4 tokens in the AST. When the
	// AST is converted to a String, new line characters can appear in the middle of any of those
	// tokens, so to properly unpreserve them we can use regex.
	// Attention, before TEMP_DOCs may appear some literal space character (\\s, \\t, \\n). They
	// must be skipped to avoid having invalid rust code!
	let re = Regex::new(r#"#\s*\[\s*doc\s*=\s*"TEMP_DOC([\\s\\t\\n]*)(.*?)"\s*\]"#)
		.expect("The regex is valid; qed;");
	let code = re.replace_all(&code, |caps: &Captures| format!("\n{}\n", &caps[2])).to_string();
	// Same happens with 'type temp_marker = ();'. This lines also delete them from everywhere, not
	// just inside declarative macros
	let re = Regex::new(r"(?m)^\s*type\s*temp_marker\s*=\s*\(\);[ \t]*\n?")
		.expect("The regex is valid; qed;");
	let code = re.replace_all(&code, "").to_string();
	// Delete all TEMP_DOCS present in the rest of the code and return the result.
	let re = Regex::new(r"(?m)^\s*///TEMP_DOC").expect("The regex is valid; qed;");
	let code = re.replace_all(&code, "").to_string();

	std::fs::write(path, &code)?;

	Ok(())
}

fn apply_preservers(code: &str, preservers: &[&Preserver]) -> String {
	let mut delimiters_counts = DelimitersCount::new();

	let mut lines = code.lines();

	// Non-preserved lines are pushed to the Vec in bundles, whule preserved lines are pushed
	// together with a '\n' character, so the bound #lines * 2 is an upper
	// bound of the final capacity (probably far from the real length of the Vec).
	let mut result: Vec<String> = Vec::with_capacity(code.lines().count() * 2);

	while let Some(line) = lines.next() {
		let trimmed_line = line.trim_start();
		if let Some(index) = preservers
			.iter()
			.position(|preserver| trimmed_line.starts_with(preserver.lookup()))
		{
			delimiters_counts.count(line);
			result.push(line.to_owned());
			result.push("\n".to_owned());

			let inner_preserver = preservers[index].get_inner();

			if let Some(inner_preserver_pointer) = inner_preserver {
				let mut inner_code = String::new();
				for line in lines.by_ref() {
					delimiters_counts.count(line);

					if delimiters_counts.is_complete() {
						result.push(apply_preservers(&inner_code, &[inner_preserver_pointer]));
						result.push(line.to_owned());
						result.push("\n".to_owned());
						break;
					} else {
						inner_code.push_str(line);
						inner_code.push('\n');
					}
				}
			}
		} else if delimiters_counts.is_complete() {
			result.push(format!("///TEMP_DOC{}\n", line));
		} else {
			if (trimmed_line.starts_with("//") &&
				!trimmed_line.starts_with("///") &&
				!trimmed_line.starts_with("//!")) ||
				trimmed_line.starts_with("#![")
			{
				// Preserve comments and global attributes.
				// Global attributes may be hard to parse with syn, so we comment them to solve
				// potential issues related to them.
				result.push(format!("///TEMP_DOC{}\ntype temp_marker = ();\n", line));
			} else if trimmed_line.is_empty() {
				// Preserve empty lines inside a non-preserved block
				result.push("///TEMP_DOC\ntype temp_marker = ();\n".to_owned());
			} else {
				result.push(line.to_owned());
				result.push("\n".to_owned());
			}

			delimiters_counts.count(line);
		}
	}

	result.push("type temp_marker = ();\n".to_owned());

	result.join("")
}
