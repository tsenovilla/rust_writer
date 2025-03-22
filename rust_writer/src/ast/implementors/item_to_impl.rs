// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use syn::{visit::Visit, visit_mut::VisitMut, ImplItem, ItemImpl, PathSegment};

/// This implementor targets an element inside an `impl block`
#[derive(Debug, Clone)]
pub struct ItemToImpl<'a> {
	/// The trait's name lookup. If specified, the implementor will look inside `impl` blocks
	/// implementing this trait.  
	pub trait_name: Option<&'a str>,
	/// The type being implemented by the `impl` block.
	pub implementor_name: &'a str,
	/// The target item.
	pub impl_item: ImplItem,
}

impl<'a> From<(Option<&'a str>, &'a str, ImplItem)> for ItemToImpl<'a> {
	fn from(tuple: (Option<&'a str>, &'a str, ImplItem)) -> Self {
		Self { trait_name: tuple.0, implementor_name: tuple.1, impl_item: tuple.2 }
	}
}

struct PathSegmentFinder<'a> {
	found: [bool; 2],
	trait_name: Option<&'a str>,
	implementor_name: &'a str,
}

impl<'a> PathSegmentFinder<'a> {
	fn find_impl_paths(&mut self, item_impl: &'a ItemImpl) {
		match item_impl.trait_ {
			Some((_, ref path, _)) => self.visit_path(path),
			None if self.trait_name.is_none() => self.found[0] = true,
			_ => (),
		}
		self.visit_type(&item_impl.self_ty);
	}
}

impl<'a> Visit<'a> for PathSegmentFinder<'a> {
	fn visit_path_segment(&mut self, path_segment: &'a PathSegment) {
		match self.trait_name {
			Some(trait_name) if path_segment.ident == trait_name => self.found[0] = true,
			_ => (),
		}
		if path_segment.ident == self.implementor_name {
			self.found[1] = true;
		}
	}
}

impl<'a> ToFind<'a, ItemToImpl<'a>, 1> for Finder<'a, EmptyFinder, 1> {
	fn to_find(self, finder: &'a ItemToImpl<'a>) -> Finder<'a, ItemToImpl<'a>, 1> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, ItemToImpl<'a>, 1> {
	fn visit_item_impl(&mut self, item_impl: &'a ItemImpl) {
		let mut path_segment_finder = PathSegmentFinder {
			found: [false, false],
			trait_name: self.finder.trait_name,
			implementor_name: self.finder.implementor_name,
		};
		path_segment_finder.find_impl_paths(item_impl);
		if path_segment_finder.found.iter().all(|&x| x) &&
			item_impl.items.contains(&self.finder.impl_item)
		{
			self.found[0] = true;
		}
	}
}

impl<'a> ToMutate<'a, ItemToImpl<'a>, 1> for Mutator<'_, EmptyMutator, 1> {
	fn to_mutate(self, mutator: &'a ItemToImpl<'a>) -> Mutator<'a, ItemToImpl<'a>, 1> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl<'a> VisitMut for Mutator<'a, ItemToImpl<'a>, 1> {
	fn visit_item_impl_mut(&mut self, item_impl: &mut ItemImpl) {
		let mut path_segment_finder = PathSegmentFinder {
			found: [false, false],
			trait_name: self.mutator.trait_name,
			implementor_name: self.mutator.implementor_name,
		};
		path_segment_finder.find_impl_paths(item_impl);
		if path_segment_finder.found.iter().all(|&x| x) {
			self.mutated[0] = true;
			item_impl.items.push(self.mutator.impl_item.clone());
		}
	}
}
