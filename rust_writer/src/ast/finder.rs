// SPDX-License-Identifier: GPL-3.0

use std::fmt::Debug;
use syn::{visit::Visit, File};

#[derive(Debug)]
pub struct EmptyFinder;

pub struct Finder<'a, T: Debug, const N: usize> {
	pub found: [bool; N],
	pub finder: &'a T,
}

pub trait ToFind<'a, T: Debug, const N: usize> {
	fn to_find(self, finder: &'a T) -> Finder<'a, T, N>;
}

impl<'a> Default for Finder<'a, EmptyFinder, 1> {
	fn default() -> Self {
		Self { found: [false], finder: &EmptyFinder }
	}
}

impl<'a, T, const N: usize> Finder<'a, T, N>
where
	T: Debug,
	Finder<'a, T, N>: Visit<'a>,
{
	pub fn find(&'a mut self, ast: &'a File) -> bool {
		self.visit_file(ast);
		self.found.iter().all(|&x| x)
	}
}
