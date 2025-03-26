// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use syn::{visit::Visit, visit_mut::VisitMut, File, Item};

/// This implementor targets any item inside a complete AST.
///
/// When it's used with [`Finder`], it doesn't take attributes into account, this is, if the
/// following is contained in the target AST
///
/// ```no_compile
/// /// Some nice docs
/// #[some_attr]
/// type Type = ();
/// ```
///
/// and the target item is `type Type = ();`, the [`find`] method will return true. A major update
/// will change this in the future, allowing to include attributes in the lookup if needed.
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
		let self_item_no_docs =
			rustilities::parsing::attrs_mut::tt_without_attrs(&self.finder.item);
		if file.items.iter().any(|item| {
			rustilities::parsing::attrs_mut::tt_without_attrs(item) == self_item_no_docs
		}) {
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
