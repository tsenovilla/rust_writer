// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::mutator;

#[mutator(a,b,c,d(e,f,g))]
struct SomeStruct;

fn main(){}
