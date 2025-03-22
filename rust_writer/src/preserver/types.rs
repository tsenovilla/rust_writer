// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

/// The `Preserver` type specifies which fragments of code should be preserved. It uses a lookup
/// that identifies the beginning of a line that should be preserved. If such a line doesn't start
/// a new block (eg, `let something = false;`), that line is the only preserved thing by this type.
/// If the line opens a new block (eg, `impl MyTrait{`), the whole block is preserved.
///
/// It's possible to preserve only a part of a preserved block, while keeping that block itself
/// preserved by using inner preservers, that is, a `Preserver` contained inside another one (and so
/// on).
///
/// ```no_compile
/// trait External{
///   type A: From<String>;
///
///   fn inner_function(){
///     let a = false;
///     fn super_inner_function(){
///
///     }
///   }
/// }
///
/// // This preserves just super_inner_function.
/// let preserver = Preserver::new("super_inner_function");
///
/// // This preserves inner_function inside the trait, but doesn't preserve the line
/// // `type A: From<String>`.
/// let mut preserver = Preserver::new("trait External");
/// preserver.add_inners(&["fn inner_function"]);
///
/// // This preserves super_inner_function inside inner_function inside the trait, but doesn't preserve the lines
/// // `let a = false;` and `type A: From<String>;`.
/// let mut preserver = Preserver::new("trait External");
/// preserver.add_inners(&["fn inner_function", "fn super_inner_function"]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Preserver {
	lookup: String,
	inner: Option<Box<Preserver>>,
}

impl Preserver {
	/// Creates a new preserver using the provided lookup.  
	pub fn new(lookup: &str) -> Self {
		Self { lookup: lookup.to_owned(), inner: None }
	}

	/// Add inner preservers, in order to preserve just an inner block of a preserved block, keeping
	/// the outer block itself preserved. Inner preserver will be composed sequentially as they are
	/// defined in the input slice.
	pub fn add_inners(&mut self, lookups: &[&str]) {
		let mut current = self;
		for lookup in lookups {
			current.inner = Some(Box::new(Self::new(lookup)));
			current = current.inner.as_mut().expect("Inner is Some due to the previous line; qed");
		}
	}

	/// Gets the lookup for a `Preserver`.
	pub fn lookup(&self) -> &str {
		&self.lookup
	}

	/// Gets the outermost inner preserver for a `Preserver`, if any.
	pub fn get_inner(&self) -> Option<&Preserver> {
		self.inner.as_deref()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DelimitersCount {
	counts: [u8; 6],
}

impl DelimitersCount {
	pub(crate) fn new() -> Self {
		Self { counts: [0; 6] }
	}

	pub(crate) fn is_complete(&self) -> bool {
		self.counts[0] == self.counts[1] && // `{` and `}`
        self.counts[2] == self.counts[3] && // `(` and `)`
        self.counts[4] == self.counts[5] // `[` and `]`
	}

	pub(crate) fn count(&mut self, line: &str) {
		self.counts[0] += line.matches('{').count() as u8;
		self.counts[1] += line.matches('}').count() as u8;
		self.counts[2] += line.matches('(').count() as u8;
		self.counts[3] += line.matches(')').count() as u8;
		self.counts[4] += line.matches('[').count() as u8;
		self.counts[5] += line.matches(']').count() as u8;
	}
}
