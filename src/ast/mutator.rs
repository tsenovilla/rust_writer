// SPDX-License-Identifier: GPL-3.0

use crate::Error;
use std::fmt::Debug;
use syn::{visit_mut::VisitMut, File};

#[derive(Debug)]
pub struct EmptyMutator;

pub struct Mutator<T: Debug> {
	pub mutated: bool,
	pub mutator: T,
}

pub trait ToMutate<T: Debug> {
	fn to_mutate(self, mutator: T) -> Mutator<T>;
}

impl Default for Mutator<EmptyMutator> {
	fn default() -> Self {
		Self { mutated: false, mutator: EmptyMutator }
	}
}

impl<T> Mutator<T>
where
	T: Debug,
	Mutator<T>: VisitMut,
{
	pub fn mutate(mut self, ast: &mut File) -> Result<(), Error> {
		self.visit_file_mut(ast);

		if self.mutated {
			Ok(())
		} else {
			Err(Error::Descriptive(format!("Cannot mutate using Mutator: {:?}", self.mutator)))
		}
	}
}
