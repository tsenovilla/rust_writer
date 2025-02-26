// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::mutator;

#[mutator(rust_writer::ast::implementors::ItemToTrait<'a>, local = "hello")]
struct SomeStruct;

fn main() {}
