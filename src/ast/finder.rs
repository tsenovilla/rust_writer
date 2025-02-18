// SPDX-License-Identifier: GPL-3.0

use std::fmt::Debug;
use syn::{visit::Visit, File};

#[derive(Debug)]
pub struct EmptyFinder;

pub struct Finder<'a, T: Debug> {
	pub found: bool,
	pub finder: &'a T,
}

pub trait ToFind<'a, T: Debug> {
	fn to_find(self, finder: &'a T) -> Finder<'a, T>;
}

impl<'a> Default for Finder<'a, EmptyFinder> {
	fn default() -> Self {
		Self { found: false, finder: &EmptyFinder }
	}
}

impl<'a, T> Finder<'a, T>
where
	T: Debug,
	Finder<'a, T>: Visit<'a>,
{
	pub fn find(&'a mut self, ast: &'a File) -> bool {
		self.visit_file(ast);
		self.found
	}
}
