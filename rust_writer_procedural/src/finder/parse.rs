// SPDX-License-Identifier: GPL-3.0

use crate::parse_attrs::{MacroAttr, MacroAttrs};
use syn::{Error, Fields, FieldsNamed, ItemStruct, Path, Result, Type, TypeArray};

pub(crate) struct FinderDef {
	pub(crate) crate_implementors: Vec<Path>,
	pub(crate) local_implementors: Vec<Path>,
	pub(crate) struct_: ItemStruct,
	pub(crate) already_expanded: bool,
	pub(crate) impl_from: bool,
}

pub(crate) struct ImplFinderDef {
	pub(crate) struct_: ItemStruct,
}

impl FinderDef {
	pub(crate) fn try_from(attrs: MacroAttrs, mut struct_: ItemStruct) -> Result<Self> {
		let (already_expanded, impl_from) = attrs.validate_struct(&mut struct_)?.parse();

		let mut crate_implementors: Vec<Path> = Vec::with_capacity(attrs.0.len());
		let mut local_implementors: Vec<Path> = Vec::with_capacity(attrs.0.len());
		attrs.0.into_iter().for_each(|macro_attr| match macro_attr {
			MacroAttr::CrateImplementor(path) => crate_implementors.push(path),
			MacroAttr::LocalImplementor(path) => local_implementors.push(path),
		});

		Ok(Self { crate_implementors, local_implementors, struct_, already_expanded, impl_from })
	}
}

impl ImplFinderDef {
	pub(crate) fn try_from(struct_: ItemStruct) -> Result<Self> {
		match &struct_.fields {
			Fields::Named(FieldsNamed { named, .. })
				if named.iter().any(|field| {
					field.ident.as_ref().expect("Named fields have ident; qed;") == "found" &&
						matches!(&field.ty, Type::Array(TypeArray { elem, .. })
							if matches!(&**elem, Type::Path(path) if path.path.is_ident("bool"))
						)
				}) =>
				Ok(Self { struct_ }),
			_ => Err(Error::new(
				struct_.ident.span(),
				"Expected a file named found being [bool;N] inside struct.",
			)),
		}
	}
}
