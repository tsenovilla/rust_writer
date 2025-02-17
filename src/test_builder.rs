// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;
use syn::{visit::Visit, File as syn_File, TraitItem};
use tempfile::TempDir;

pub trait WithItems<'a, T> {
	fn with_items(&mut self, items: &'a T);
}

pub struct Visitor<'a, T> {
	found: bool,
	items: Option<&'a T>,
}

impl<T> Default for Visitor<'_, T> {
	fn default() -> Self {
		Self { found: false, items: None }
	}
}

impl<T> Visitor<'_, T> {
	pub fn found(&self) -> bool {
		self.found
	}
}

impl<'a> WithItems<'a, TraitItem> for Visitor<'a, TraitItem> {
	fn with_items(&mut self, items: &'a TraitItem) {
		self.items = Some(items);
	}
}

impl<'ast> Visit<'ast> for Visitor<'ast, TraitItem> {
	fn visit_trait_item(&mut self, item: &'ast TraitItem) {
		if let Some(trait_item) = self.items {
			if item == trait_item {
				self.found = true;
			}
		}
	}
}

pub struct TestBuilder<'a> {
	tempdir: TempDir,
	ast_files: HashMap<&'a str, syn_File>,
}

impl Default for TestBuilder<'_> {
	fn default() -> Self {
		let tempdir = tempfile::tempdir().expect("Tempdir should be created; qed;");
		Self { tempdir, ast_files: HashMap::new() }
	}
}

macro_rules! add_ast_from_sample_files{
 ($([$name: ident, $file: literal]),*) => {
   $(
     pub fn $name(mut self) -> Self{
       let sample_file_path = std::env::current_dir()
         .expect("This should work; qed;")
         .join("sample_files")
         .join($file);
       let ast = syn::parse_file(
         &std::fs::read_to_string(&sample_file_path).expect("File should be readable; qed;")
       ).expect("File should be parsed; qed;");
       self.ast_files.insert($file, ast);
       self
      }
    )*
  };
}

impl<'a> TestBuilder<'a> {
	add_ast_from_sample_files! {
		[with_trait_ast, "trait.rs"]
	}

	pub fn get_ref_ast_file(&self, key: &'a str) -> Option<&syn_File> {
		self.ast_files.get(key)
	}

	pub fn get_mut_ast_file(&mut self, key: &'a str) -> Option<&mut syn_File> {
		self.ast_files.get_mut(key)
	}

	pub fn execute<F>(self, test: F)
	where
		F: Fn(Self),
	{
		test(self);
	}
}
