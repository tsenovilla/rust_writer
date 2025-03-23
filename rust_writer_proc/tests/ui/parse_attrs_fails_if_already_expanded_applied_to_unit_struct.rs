// SPDX-License-Identifier: GPL-3.0

use rust_writer_proc::mutator;

#[mutator(rust_writer::ast::implementors::ItemToTrait<'a>, rust_writer::ast::implementors::ItemToImpl<'a>)]
#[rust_writer_proc::already_expanded]
struct SomeStruct;

fn main() {}
