// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use syn::{visit::Visit, visit_mut::VisitMut, File, Item};

#[derive(Debug, Clone)]
pub struct ItemToFile {
	pub to_file_end: bool,
	pub item: Item,
}

impl From<(bool, Item)> for ItemToFile {
	fn from(tuple: (bool, Item)) -> Self {
		Self { to_file_end: tuple.0, item: tuple.1 }
	}
}

impl<'a> ToFind<'a, ItemToFile, 1> for Finder<'a, EmptyFinder, 1> {
	fn to_find(self, finder: &'a ItemToFile) -> Finder<'a, ItemToFile, 1> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, ItemToFile, 1> {
	fn visit_file(&mut self, file: &'a File) {
		if file.items.contains(&self.finder.item) {
			self.found[0] = true;
		}
	}
}

impl<'a> ToMutate<'a, ItemToFile, 1> for Mutator<'_, EmptyMutator, 1> {
	fn to_mutate(self, mutator: &'a ItemToFile) -> Mutator<'a, ItemToFile, 1> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl<'a> VisitMut for Mutator<'a, ItemToFile, 1> {
	fn visit_file_mut(&mut self, file: &mut File) {
		if self.mutator.to_file_end {
			file.items.push(self.mutator.item.clone());
		} else {
			file.items.insert(0, self.mutator.item.clone());
		}
		self.mutated[0] = true;
	}
}
