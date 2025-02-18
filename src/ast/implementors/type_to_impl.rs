// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use std::fmt::Debug;
use syn::{visit::Visit, visit_mut::VisitMut, ItemTrait, TraitItem};

#[derive(Debug)]
pub struct TypeToImpl {
	trait_name: String,
	type_: TraitItem,
}

impl From<(String, TraitItem)> for TypeToImpl {
	fn from(tuple: (String, TraitItem)) -> Self {
		Self { trait_name: tuple.0, type_: tuple.1 }
	}
}

impl<'a> ToFind<'a, TypeToImpl> for Finder<'a, EmptyFinder> {
	fn to_find(self, finder: &'a TypeToImpl) -> Finder<'a, TypeToImpl> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, TypeToImpl> {
	fn visit_item_trait(&mut self, item_trait: &'a ItemTrait) {
		if item_trait.ident == self.finder.trait_name {
			self.found = true;
		}
	}
}

impl ToMutate<TypeToImpl> for Mutator<EmptyMutator> {
	fn to_mutate(self, mutator: TypeToImpl) -> Mutator<TypeToImpl> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl VisitMut for Mutator<TypeToImpl> {
	fn visit_item_trait_mut(&mut self, item_trait: &mut ItemTrait) {
		if item_trait.ident == self.mutator.trait_name {
			self.mutated = true;
			item_trait.items.push(self.mutator.type_.clone());
		}
	}
}
