// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use std::fmt::Debug;
use syn::{visit::Visit, visit_mut::VisitMut, ItemTrait, TraitItem};

#[derive(Debug, Clone)]
pub struct TypeToTrait {
	trait_name: String,
	type_: TraitItem,
}

impl From<(String, TraitItem)> for TypeToTrait {
	fn from(tuple: (String, TraitItem)) -> Self {
		Self { trait_name: tuple.0, type_: tuple.1 }
	}
}

impl<'a> ToFind<'a, TypeToTrait> for Finder<'a, EmptyFinder> {
	fn to_find(self, finder: &'a TypeToTrait) -> Finder<'a, TypeToTrait> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, TypeToTrait> {
	fn visit_item_trait(&mut self, item_trait: &'a ItemTrait) {
		if item_trait.ident == self.finder.trait_name &&
			item_trait.items.contains(&self.finder.type_)
		{
			self.found = true;
		}
	}
}

impl ToMutate<TypeToTrait> for Mutator<EmptyMutator> {
	fn to_mutate(self, mutator: TypeToTrait) -> Mutator<TypeToTrait> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl VisitMut for Mutator<TypeToTrait> {
	fn visit_item_trait_mut(&mut self, item_trait: &mut ItemTrait) {
		if item_trait.ident == self.mutator.trait_name {
			self.mutated = true;
			item_trait.items.push(self.mutator.type_.clone());
		}
	}
}
