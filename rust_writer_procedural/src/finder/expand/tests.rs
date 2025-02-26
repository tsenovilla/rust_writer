// SPDX-License-Identifier: GPL-3.0

use super::*;

// The UI test covers almost the whole macro code, but they don't cover the case when struct_ is a
// tuple struct (should panic), as that's an unreachable branch due to the parse flow. This test is
// just to show that expand_finder panics in that case for completeness
#[test]
#[should_panic]
fn expand_finder_panics_if_tuple_struct() {
	let def = MacroParsed {
		crate_implementors_idents: vec![],
		local_implementors_idents: vec![],
		struct_: parse_quote!(
			struct SomeStruct(u8);
		),
		already_expanded: false,
		impl_from: false,
		one: Index::from(1),
		implementors_count: Index::from(1),
		crate_implementors_indexes: vec![],
		local_implementors_indexes: vec![],
		generics_idents: Punctuated::new(),
		where_clause: parse_quote!(where),
		new_struct_fields: Punctuated::new(),
	};

	expand_finder(def);
}
