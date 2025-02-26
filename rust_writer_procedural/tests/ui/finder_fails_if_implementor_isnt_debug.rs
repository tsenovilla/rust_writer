// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::finder::ToFind;
use rust_writer_procedural::{impl_finder, finder};
use syn::visit::Visit;

#[impl_finder]
#[derive(Clone)]
struct A {
	found: [bool; 1],
}

impl Visit<'_> for A {}

#[finder(rust_writer::ast::implementors::ItemToTrait<'a>, local = A)]
struct SomeStruct;

fn main() {}
