// SPDX-License-Identifier: GPL-3.0

use std::{collections::HashMap, fs::Permissions, os::unix::fs::PermissionsExt, path::Path};
use syn::File as syn_File;
use tempfile::{NamedTempFile, TempDir};

pub struct TestBuilder<'a> {
	tempdir: TempDir,
	with_read_only_temp_dir: bool,
	tempfiles: HashMap<&'a str, NamedTempFile>,
	ast_files: HashMap<&'a str, syn_File>,
}

impl Default for TestBuilder<'_> {
	fn default() -> Self {
		let tempdir = tempfile::tempdir().expect("Tempdir should be created; qed;");
		Self {
			tempdir,
			tempfiles: HashMap::new(),
			with_read_only_temp_dir: false,
			ast_files: HashMap::new(),
		}
	}
}

macro_rules! add_ast_from_sample_files{
 ($([$name: ident, $file: literal]),*) => {
   $(
     pub fn $name(mut self) -> Self{
       let sample_file_path = Path::new(&std::env::var("SAMPLE_FILES_PATH")
         .expect("SAMPLE_FILES_PATH must be defined; qed;"))
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

macro_rules! add_tempfiles_from_sample_files{
 ($([$name: ident, $file: literal]),*) => {
   $(
     pub fn $name(mut self) -> Self{
       let sample_file_path = Path::new(&std::env::var("SAMPLE_FILES_PATH")
         .expect("SAMPLE_FILES_PATH must be defined; qed;"))
         .join($file);
       let tempfile = NamedTempFile::new_in(self.tempdir.path()).expect("Tempfile should be created; qed;");
       std::fs::copy(sample_file_path, tempfile.path()).expect("Tempfile should be writable; qed;");
       self.tempfiles.insert($file, tempfile);
       self
      }
    )*
  };
}

impl<'a> TestBuilder<'a> {
	add_ast_from_sample_files! {
		[with_trait_ast, "trait.rs"],
		[with_impl_block_ast, "impl_block.rs"],
		[with_trait_and_impl_block_ast, "trait_and_impl_block.rs"],
		[with_mod_ast, "mod.rs"],
		[with_macro_ast, "macro.rs"],
		[with_file_ast, "file.rs"],
		[with_preserved_file_ast, "preserved_file.rs"]
	}

	add_tempfiles_from_sample_files! {
		[with_complete_file, "complete_file.rs"],
		[with_preserved_file, "preserved_file.rs"]
	}

	pub fn get_ref_ast_file(&self, key: &'a str) -> Option<&syn_File> {
		self.ast_files.get(key)
	}

	pub fn get_mut_ast_file(&mut self, key: &'a str) -> Option<&mut syn_File> {
		self.ast_files.get_mut(key)
	}

	pub fn with_read_only_temp_dir(self) -> Self {
		Self { with_read_only_temp_dir: true, ..self }
	}

	pub fn tempfile_path(&self, key: &'a str) -> Option<&Path> {
		self.tempfiles.get(key).map(|tempfile| tempfile.path())
	}

	pub fn execute<F>(self, test: F)
	where
		F: Fn(Self),
	{
		if self.with_read_only_temp_dir {
			std::fs::set_permissions(self.tempdir.path(), Permissions::from_mode(0o444))
				.expect("temp dir permissions should be configurable; qed;");
		}

		test(self);
	}
}
