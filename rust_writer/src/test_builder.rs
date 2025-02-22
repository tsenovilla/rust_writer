// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;
use syn::File as syn_File;
use tempfile::TempDir;

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
		[with_trait_ast, "trait.rs"],
	[with_impl_block_ast, "impl_block.rs"]
	}

	pub fn get_ref_ast_file(&self, key: &'a str) -> Option<&syn_File> {
		self.ast_files.get(key)
	}

	pub fn get_mut_ast_file(&mut self, key: &'a str) -> Option<&mut syn_File> {
		self.ast_files.get_mut(key)
	}

	pub fn get_ast_file(self, key: &'a str) -> Option<syn_File> {
		self.ast_files.get(key).map(|file| file.clone())
	}

	pub fn execute<F>(self, test: F)
	where
		F: Fn(Self),
	{
		test(self);
	}
}
