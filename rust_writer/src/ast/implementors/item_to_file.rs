// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	helpers,
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use syn::{visit::Visit, visit_mut::VisitMut, File, Item};

/// This implementor targets any item inside a complete AST.
#[derive(Debug, Clone)]
pub struct ItemToFile {
	pub item: Item,
}

impl From<Item> for ItemToFile {
	fn from(item: Item) -> Self {
		Self { item }
	}
}

impl<'a> ToFind<'a, ItemToFile, 1> for Finder<'a, EmptyFinder, 1> {
	fn to_find(self, finder: &'a ItemToFile) -> Finder<'a, ItemToFile, 1> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, ItemToFile, 1> {
	fn visit_file(&mut self, file: &'a File) {
		let self_item_no_docs = helpers::item_without_docs(&self.finder.item);
		if file
			.items
			.iter()
			.any(|item| helpers::item_without_docs(item) == self_item_no_docs)
		{
			self.found[0] = true;
		}
	}
}

impl<'a> ToMutate<'a, ItemToFile, 1> for Mutator<'_, EmptyMutator, 1> {
	fn to_mutate(self, mutator: &'a ItemToFile) -> Mutator<'a, ItemToFile, 1> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl VisitMut for Mutator<'_, ItemToFile, 1> {
	fn visit_file_mut(&mut self, file: &mut File) {
		file.items.push(self.mutator.item.clone());
		self.mutated[0] = true;
	}
}
