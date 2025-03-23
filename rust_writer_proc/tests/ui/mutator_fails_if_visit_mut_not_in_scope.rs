// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::mutator::ToMutate;
use rust_writer_proc::mutator;

#[mutator(rust_writer::ast::implementors::ItemToTrait<'a>, rust_writer::ast::implementors::ItemToImpl<'a>)]
struct SomeStruct;

fn main() {}
