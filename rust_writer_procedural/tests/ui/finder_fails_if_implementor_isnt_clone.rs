// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::finder::ToFind;
use rust_writer_procedural::{finder, impl_finder};
use syn::visit::Visit;

#[impl_finder('a)]
#[derive(Debug)]
struct A {
	found: [bool; 1],
}

impl<'a> Visit<'a> for A {}

#[finder(rust_writer::ast::implementors::ItemToTrait<'a>, local = A)]
struct SomeStruct;

fn main() {}
