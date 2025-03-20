// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::Error;
use std::fmt::Debug;
use syn::{visit_mut::VisitMut, File};

#[derive(Debug, Clone)]
pub struct EmptyMutator;

#[derive(Debug, Clone)]
pub struct Mutator<'a, T: Debug + Clone, const N: usize> {
	pub mutated: [bool; N],
	pub mutator: &'a T,
}

pub trait ToMutate<'a, T: Debug + Clone, const N: usize> {
	fn to_mutate(self, mutator: &'a T) -> Mutator<'a, T, N>;
}

impl Default for Mutator<'_, EmptyMutator, 1> {
	fn default() -> Self {
		Self { mutated: [false], mutator: &EmptyMutator }
	}
}

impl<'a, T, const N: usize> Mutator<'a, T, N>
where
	T: Debug + Clone,
	Mutator<'a, T, N>: VisitMut,
{
	pub fn mutate(&mut self, ast: &mut File) -> Result<(), Error> {
		self.visit_file_mut(ast);

		if self.mutated.iter().all(|&x| x) {
			Ok(())
		} else {
			Err(Error::Descriptive(format!("Cannot mutate using Mutator: {:?}", self.mutator)))
		}
	}
}

impl<T, const N: usize> Mutator<'_, T, N>
where
	T: Debug + Clone,
{
	pub fn reset(&mut self) {
		self.mutated = [false; N];
	}
}
