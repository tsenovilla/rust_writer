// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::finder;

#[finder(rust_writer::ast::implementors::ItemToTrait<'a>, local = path)]
#[rust_writer_procedural::already_impl_from]
#[impl_from]
struct SomeStruct;

fn main() {}
