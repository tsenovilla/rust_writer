// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::finder;

#[finder(rust_writer::ast::implementors::ItemToTrait<'a>, rust_writer::ast::implementors::ItemToImpl<'a>)]
struct SomeStruct;

fn main() {}
