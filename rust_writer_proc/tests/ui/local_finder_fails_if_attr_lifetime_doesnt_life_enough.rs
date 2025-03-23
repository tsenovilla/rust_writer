// SPDX-License-Identifier: GPL-3.0

use rust_writer_proc::local_finder;
use syn::visit::Visit;

// A custom finder emulating ItemToTrait
#[local_finder('b)]
#[derive(Debug)]
struct SomeStruct<'a, T: Clone + std::fmt::Debug> {
	found: [bool; 1],
	trait_name: &'a str,
	#[allow(dead_code)]
	just_extra_data: T,
}

impl<'a, T: Clone + std::fmt::Debug> Visit<'a> for SomeStruct<'a, T> {}

fn main() {}
