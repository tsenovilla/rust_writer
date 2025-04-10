// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use syn::{visit::Visit, visit_mut::VisitMut, ItemTrait, TraitItem};

/// This implementor target any item inside a trait definition.
///
/// When it's used with [`Finder`], it doesn't take attributes into account, this is, if the
/// following is contained in the target trait
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
pub struct ItemToTrait<'a> {
	/// The trait's name.
	pub trait_name: &'a str,
	/// The target item.
	pub item_trait: TraitItem,
}

impl<'a> From<(&'a str, TraitItem)> for ItemToTrait<'a> {
	fn from(tuple: (&'a str, TraitItem)) -> Self {
		Self { trait_name: tuple.0, item_trait: tuple.1 }
	}
}

impl<'a> ToFind<'a, ItemToTrait<'a>, 1> for Finder<'a, EmptyFinder, 1> {
	fn to_find(self, finder: &'a ItemToTrait<'a>) -> Finder<'a, ItemToTrait<'a>, 1> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, ItemToTrait<'a>, 1> {
	fn visit_item_trait(&mut self, item_trait: &'a ItemTrait) {
		let self_item_trait_no_docs =
			rustilities::parsing::attrs_mut::tt_without_attrs(&self.finder.item_trait);
		if item_trait.ident == self.finder.trait_name &&
			item_trait.items.iter().any(|trait_item| {
				rustilities::parsing::attrs_mut::tt_without_attrs(trait_item) ==
					self_item_trait_no_docs
			}) {
			self.found[0] = true;
		}
	}
}

impl<'a> ToMutate<'a, ItemToTrait<'a>, 1> for Mutator<'_, EmptyMutator, 1> {
	fn to_mutate(self, mutator: &'a ItemToTrait<'a>) -> Mutator<'a, ItemToTrait<'a>, 1> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl<'a> VisitMut for Mutator<'a, ItemToTrait<'a>, 1> {
	fn visit_item_trait_mut(&mut self, item_trait: &mut ItemTrait) {
		if item_trait.ident == self.mutator.trait_name {
			self.mutated[0] = true;
			item_trait.items.push(self.mutator.item_trait.clone());
		}
	}
}
