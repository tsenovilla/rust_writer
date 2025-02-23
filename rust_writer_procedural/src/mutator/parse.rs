// SPDX-License-Identifier: GPL-3.0

use crate::parse_attrs::{InnerAttr, MacroAttrs};
use syn::{ItemStruct, Path, Result};

pub(crate) struct MutatorDef {
	pub(crate) implementors: Vec<Path>,
	pub(crate) struct_: ItemStruct,
	pub(crate) unit_struct: bool,
	pub(crate) impl_from: bool,
}

impl MutatorDef {
	pub(crate) fn try_from(attrs: MacroAttrs, mut struct_: ItemStruct) -> Result<Self> {
		let (unit_struct, impl_from) = match attrs.validate_struct(&mut struct_)? {
			InnerAttr::Unit => (true, false),
			InnerAttr::NotUnit => (false, false),
			InnerAttr::ImplFrom => (true, true),
			InnerAttr::ImplFromNotUnit => (false, true),
		};

		let implementors: Vec<Path> = attrs.0.into_iter().collect();

		Ok(Self { implementors, struct_, unit_struct, impl_from })
	}
}
