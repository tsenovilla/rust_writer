// SPDX-License-Identifier: GPL-3.0

mod finder;
pub(crate) mod helpers;
mod mutator;
pub(crate) mod parse;

use proc_macro::TokenStream;

/// The `#[mutator]` macro is used to define a new implementor which combines other implementors,
/// capable of mutating different parts of an AST with just one instruction, with the same level of
/// precision as if we had used each implementor separately.
///
/// The syntax to use the `#[mutator]` macro is pretty simple:
///
/// ```rust
/// use rust_writer::ast::{mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::visit_mut::VisitMut;
///
/// // The new implementor behaves as two `ItemToTrait` implementors.
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>)]
/// struct NewImplementor;
/// ```
///
/// From the snippet above we can already assert that the `#[mutator]` macro needs that the traits
/// [`ToMutate`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/trait.ToMutate.html)
/// and [`VisitMut`](https://docs.rs/syn/latest/syn/visit_mut/trait.VisitMut.html) have been
/// brought into scope.
///
/// The `#[mutator]` macro can be applied to either unit structs or structs with named
/// files, but it doesn't support tuple structs. Hence, this wouldn't compile:
///
/// ```compile_fail
/// use rust_writer::ast::{mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::visit_mut::VisitMut;
///
/// // The new implementor behaves as two `ItemToTrait` implementors.
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>)]
/// struct NewImplementor(u8);
/// ```
///
/// It's also possible to combine an implementor from the predefined set with a local implementor,
/// just by adding the keyword `local` before the local implementor.
///
/// ```rust
/// use rust_writer::ast::{mutator, local_mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::{visit_mut::VisitMut, File};
///
/// #[local_mutator]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone>{
///   mutated: [bool; 4],
///   something: T
/// }
///
/// impl<T> VisitMut for LocalImplementor<T> where T: std::fmt::Debug + Clone{
///   fn visit_file_mut(&mut self, _file: &mut File){
///     // Dummy implementation
///     self.mutated = [true; 4];
///   }
/// }
///
/// // The new implementor behaves as two `ItemToTrait` implementors + our local implementor.
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T: std::fmt::Debug + Clone>)]
/// struct NewImplementor{
///   number: u8
/// }
/// ```
///
/// When the `#[mutator]` macro is called, the struct is expanded an includes a new field for each
/// implementor, named as the implementor type in lowercase. If an implementor is included several
/// times in the same macro invocation, a suffix is added to the subsequent fields, such as
/// _1, _2, _3 and so on. For example, the struct from the snippet above will look like this after
/// the expansion:
///
/// ```no_compile
/// struct NewImplementor<'a, T: std::fmt::Debug + Clone>{
///   number: u8,
///   itemtotrait: ItemToTrait<'a>,
///   itemtotrait_1: ItemToTrait<'a>,
///   localimplementor: LocalImplementor<T>
/// }
/// ```
///
/// These snippets also show that if an implementor needs a generic, including it in the
/// macro invocation is enough. Particularly, the generics will be added to the `NewImplementor`
/// struct, so they'll be available inside it. It's also possible to include the implementor's
/// generics in the struct definition, which is specially useful if the struct contains a generic
/// field sharing its trait bound with an implementor's generic:
///
/// ```rust
/// use rust_writer::ast::{mutator, local_mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::{visit_mut::VisitMut, File};
///
/// #[local_mutator]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone, const N: usize>{
///   mutated: [bool; N],
///   something: T
/// }
///
/// impl<T, const N: usize> VisitMut for LocalImplementor<T, N> where T: std::fmt::Debug + Clone{
///   fn visit_file_mut(&mut self, _file: &mut File){
///     // Dummy implementation
///     self.mutated = [true; N];
///   }
/// }
///
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T,N>)]
/// struct NewImplementor<T: std::fmt::Debug + Clone, const N: usize>{
///   array: [T;N]
/// }
/// ```
///
/// There's two important points to remark here:
///
/// - An implementor's generic will be added to the struct iff there's not a generic with the same
///   name already defined. This is something to keep in mind to avoid situations like the
///   following:
///
/// ```compile_fail
/// use rust_writer::ast::{mutator, local_mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::{visit_mut::VisitMut, File};
///
/// #[local_mutator]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone, const N: usize>{
///   mutated: [bool; N],
///   something: T
/// }
///
/// impl<T, const N: usize> VisitMut for LocalImplementor<T, N> where T: std::fmt::Debug + Clone{
///   fn visit_file_mut(&mut self, _file: &mut File){
///     // Dummy implementation
///     self.mutated = [true; N];
///   }
/// }
///
/// // This cannot compile even if the macro receives the correct bounds, due to T is already
/// // defined in the struct.
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T: std::fmt::Debug +
/// Clone,N>)]
/// struct NewImplementor<T, const N: usize>{
///   array: [T;N]
/// }
/// ```
///
/// - Const generics **MUST** be defined in the struct definition, even if they are only needed for
///   the implementor due to a parsing reason.
///
/// ```compile_fail
/// use rust_writer::ast::{mutator, local_mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::{visit_mut::VisitMut, File};
///
/// #[local_mutator]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone, const N: usize>{
///   mutated: [bool; N],
///   something: T
/// }
///
/// impl<T, const N: usize> VisitMut for LocalImplementor<T, N> where T: std::fmt::Debug + Clone{
///   fn visit_file_mut(&mut self, _file: &mut File){
///     // Dummy implementation
///     self.mutated = [true; N];
///   }
/// }
///
/// // This cannot compile cause the const bound is defined in the macro.
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T,const N: usize>)]
/// struct NewImplementor<T: std::fmt::Debug + Clone>{
///   value: T
/// }
/// ```
///
/// # The `#[impl_from]` attribute
///
/// As soon as a new implementor is defined using the `#[mutator]` macro, it's ready to be used!
/// But constructing a variable of this new type may be a bit annoying:
///
/// ```rust
/// use rust_writer::ast::{mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::{parse_quote, TraitItem, visit_mut::VisitMut};
///
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>)]
/// struct NewImplementor;
///
/// let itemtotrait: ItemToTrait = ("MyTrait", parse_quote!(type Type1: From<String>;)).into();
/// let itemtotrait_1: ItemToTrait = ("MyTrait", parse_quote!(type Type2: From<String>;)).into();
///
/// let new_implementor = NewImplementor{ itemtotrait, itemtotrait_1 };
/// ```
///
/// The proccess, in this simple case, implies to create the variables for the implementors and
/// remember how the fields should be named inside `NewImplementor`. The `#[impl_from]` attribute
/// comes in handy to create the `NewImplementor` variable at once, as it implements the `From`
/// trait from a tuple consisting in the struct original fields + the implementors fields.  
///
/// ```rust
/// use rust_writer::ast::{mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::{parse_quote, visit_mut::VisitMut};
///
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>)]
/// #[impl_from]
/// struct NewImplementor;
///
/// let new_implementor: NewImplementor = (
///   ("MyTrait", parse_quote!(type Type1: From<String>;)).into(),
///   ("MyTrait", parse_quote!(type Type2: From<String>;)).into()
/// ).into();
/// ```
/// This is much more compact! If the new implementor combines a lot of implementors instead of
/// just two, this will be even more evident. On the other hand, it's less explicit, so just a
/// trade-off to consider.
///
/// # How to use the `#[mutator]` macro
///
/// Typically, an implementor from the predefined set of implementors is used to mutate an AST by
/// loading it into a [`Mutator`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/struct.Mutator.html).
/// Such `Mutator` instances implement `VisitMut`, and hence the
/// [`mutate`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/struct.Mutator.html#method.mutate)
/// method can be called on them.
///
/// However, when an implementor is created with the `#[mutator]` macro, `VisitMut` cannot be
/// implemented for `Mutator` instances loading that implementor due to the orphan rule. The
/// `#[mutator]` macro does the following to mimic that behavior:
///
/// 1. Implements the `ToMutate` trait for this implementor, so a `Mutator` instance can be loaded,
///    even if the `mutate` method isn't available for it. That instance's `mutated` array has an
///    entry for each inner implementor, reflecting if that implementor mutations succeed.
///
/// 1. Creates a wrapper for that `Mutator` called `implementor_name + MutatorWrapper`. So for an
///    implementor called `NewImplementor` this wrapper would be called
///    `NewImplementorMutatorWrapper`. The `From` trait is implemented for `Mutator`, so
///    constructing this wrappers is seamless.
///
/// 1. Implements a `mutate` method for the wrapper which works exactly as the `mutate` method works
///    for a regular `Mutator`, with the only difference that it accepts an extra parameter of the
///    type `Option<&[u32]>`. If it's `Some`, the inner slice would tell the wrapper which
///    implementors apply to the AST. This is specially useful when some elements are already in the
///    AST and duplication isn't desired (the `#[finder]` macro may help to identify such elements).
///    If the parameter is `None`, all the implementors are applied.
///
/// ```rust
/// use rust_writer::ast::{mutator, implementors::ItemToTrait, mutator::{ToMutate, Mutator}};
/// use syn::{parse_quote, visit_mut::VisitMut};
/// use test_builder::TestBuilder;
///
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>)]
/// #[impl_from]
/// struct NewImplementor;
///
/// TestBuilder::default().with_trait_ast().execute(|mut builder|{
///  let new_implementor: NewImplementor = (
///    ("MyTrait", parse_quote!(type Type3: From<String>;)).into(),
///    ("MyTrait", parse_quote!(type Type4: From<String>;)).into()
///  ).into();
///
///  let mut mutator: NewImplementorMutatorWrapper =
///   Mutator::default().to_mutate(&new_implementor).into();
///
///  let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");
///
///  // Apply both mutators
///  assert!(mutator.mutate(ast, None).is_ok());
/// });
/// ```
///
/// # Compatibility with [`#[finder]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.finder.html)
///
/// It's perfectly possible to use the `#[mutator]` macro in combination with the `#[finder]`
/// macro, even more, it's quite recommended as the `#[finder]` macro can help to avoid unwanted
/// duplications in the AST, thanks to the `fn get_missing_indexes(&self) -> Option<Vec<u32>>`
/// method. The order in which macros are applied doesn't matter.
///
/// **There's just one golden rule to combine both macros**: The list of implementors should be the
/// same and come in the same order in both macros.
///
/// So this is perfectly valid:
///
/// ```rust
/// use rust_writer::ast::{
///   mutator,
///   finder,
///   implementors::{ItemToTrait, ItemToImpl},
///   mutator::ToMutate,
///   finder::ToFind
/// };
/// use syn::visit_mut::VisitMut;
///
/// #[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
/// #[finder(ItemToTrait<'a>, ItemToImpl<'a>)]
/// struct NewImplementor;
/// ```
/// While this doesn't even compile:
///
/// ```compile_fail
/// use rust_writer::ast::{
///   mutator,
///   finder,
///   implementors::{ItemToTrait, ItemToImpl},
///   mutator::ToMutate,
///   finder::ToFind
/// };
/// use syn::visit_mut::VisitMut;
///
/// #[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
/// #[finder(ItemToImpl<'a>, ItemToTrait<'a>)]
/// struct NewImplementor;
/// ```
#[proc_macro_attribute]
pub fn mutator(attrs: TokenStream, item: TokenStream) -> TokenStream {
	mutator::mutator(attrs, item)
}

/// The `#[local_mutator]` macro is used to create a custom implementor mimicking the behavior of
/// a [`Mutator`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/struct.Mutator.html).
///
/// While creating an implementor may seem an easy task, the orphan rule can complicate matters.
/// If a custom implementor implements `ToMutate`, it's not possible to implement
/// [`VisitMut`](https://docs.rs/syn/latest/syn/visit_mut/trait.VisitMut.html) for `Mutator<CustomImplementor>`,
/// so the [`mutate`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/struct.Mutator.html#method.mutate)
/// method wouldn't be available and that `Mutator` type would be totally useless.
///
/// The workaround is to implement `VisitMut` for the custom implementor itself and use it as if it
/// were a `Mutator` variable. The `#[local_mutator]` macro takes such a type and implements the
/// `mutate` and `reset` methods directly on it, mimicking the behavior of a regular `Mutator`
/// instance.
///
/// ```rust
/// use rust_writer::ast::{local_mutator, mutator::ToMutate};
/// use syn::{visit_mut::VisitMut, File, ItemTrait, parse_quote};
/// use test_builder::TestBuilder;
///
/// // This local mutator will simply add `type Type3: From<String>` to an AST.
/// #[local_mutator]
/// #[derive(Debug, Clone)]
/// struct MutateTrait<'a> {
///   mutated: [bool; 1],
///   trait_name: &'a str,
/// }
///
/// impl<'a> VisitMut for MutateTrait<'a> {
///   fn visit_item_trait_mut(&mut self, item_trait: &mut ItemTrait) {
///     if item_trait.ident == self.trait_name {
///       self.mutated[0] = true;
///       item_trait.items.push(parse_quote!(type Type3: From<String>;));
///     }
///   }
/// }
///
/// TestBuilder::default().with_trait_ast().execute(|mut builder| {
///   let mut ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed");
///
///   let mut mutator = MutateTrait {
///     mutated: [false; 1],
///     trait_name: "MyTrait"
///   };
///
///   // The AST is mutated as expected
///   assert!(mutator.mutate(&mut ast).is_ok());
///
///   // The reset method works exactly as it does for `Mutator`
///   assert_eq!(mutator.mutated, [true]);
///   mutator.mutator_reset();
///   assert_eq!(mutator.mutated, [false]);
///
///   // If the trait isn't in the AST, no mutation occurs.
///   let mut mutator = MutateTrait {
///     mutated: [false; 1],
///     trait_name: "OtherTrait"
///   };
///
///   assert!(mutator.mutate(&mut ast).is_err());
/// });
/// ```
///
/// Note that the `reset` method is called `mutator_reset` in this case. This is because a local
/// mutator can also be a local finder, hence this distinction is needed.
#[proc_macro_attribute]
pub fn local_mutator(_: TokenStream, item: TokenStream) -> TokenStream {
	mutator::local_mutator(item)
}

/// The `#[finder]` macro is used to define a new implementor which combines other implementors,
/// capable of asserting if different elements are part of an AST with just one instruction, with
/// the same level of precision as if we had used each implementor separately.
///
/// The syntax to use the `#[finder]` macro is pretty simple:
///
/// ```rust
/// use rust_writer::ast::{finder, implementors::ItemToTrait, finder::ToFind};
///
/// // The new implementor behaves as two `ItemToTrait` implementors.
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>)]
/// struct NewImplementor;
/// ```
///
/// From the snippet above we can already assert that the `#[finder]` macro needs that the trait
/// [`ToFind`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/trait.ToFind.html)
/// has been brought into scope.
///
/// The `#[finder]` macro can be applied to either unit structs or structs with named
/// fields, but it doesn't support tuple structs. Hence, this wouldn't compile:
///
/// ```compile_fail
/// use rust_writer::ast::{finder, implementors::ItemToTrait, finder::ToFind};
///
/// // The new implementor behaves as two `ItemToTrait` implementors.
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>)]
/// struct NewImplementor(u8);
/// ```
///
/// It's also possible to combine an implementor from the predefined set with a local implementor,
/// just by adding the keyword `local` before the local implementor.
///
/// ```rust
/// use rust_writer::ast::{finder, local_finder, implementors::ItemToTrait, finder::ToFind};
/// use syn::{visit::Visit, File};
///
/// #[local_finder('a)]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone>{
///   found: [bool; 4],
///   something: T
/// }
///
/// impl<'a,T> Visit<'a> for LocalImplementor<T> where T: std::fmt::Debug + Clone{
///   fn visit_file(&mut self, _file: &'a File){
///     // Dummy implementation
///     self.found = [true; 4];
///   }
/// }
///
/// // The new implementor behaves as two `ItemToTrait` implementors + our local implementor.
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T: std::fmt::Debug + Clone>)]
/// struct NewImplementor{
///   number: u8
/// }
/// ```
///
/// When the `#[finder]` macro is called, the struct is expanded and includes a new field for each
/// implementor, named as the implementor type in lowercase. If an implementor is included several
/// times in the same macro invocation, a suffix is added to the subsequent fields, such as
/// _1, _2, _3 and so on. For example, the struct from the snippet above will look like this after
/// the expansion:
///
/// ```no_compile
/// struct NewImplementor<'a, T: std::fmt::Debug + Clone>{
///   number: u8,
///   itemtotrait: ItemToTrait<'a>,
///   itemtotrait_1: ItemToTrait<'a>,
///   localimplementor: LocalImplementor<T>
/// }
/// ```
///
/// These snippets also show that if an implementor needs a generic, including it in the
/// macro invocation is enough. Particularly, the generics will be added to the `NewImplementor`
/// struct, so they'll be available inside it. It's also possible to include the implementor's
/// generics in the struct definition, which is specially useful if the struct contains a generic
/// field sharing its trait bound with an implementor's generic:
///
/// ```rust
/// use rust_writer::ast::{finder, local_finder, implementors::ItemToTrait, finder::ToFind};
/// use syn::{visit::Visit, File};
///
/// #[local_finder('a)]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone, const N: usize>{
///   found: [bool; N],
///   something: T
/// }
///
/// impl<'a, T, const N: usize> Visit<'a> for LocalImplementor<T, N> where T: std::fmt::Debug + Clone{
///   fn visit_file(&mut self, _file: &'a File){
///     // Dummy implementation
///     self.found = [true; N];
///   }
/// }
///
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T,N>)]
/// struct NewImplementor<T: std::fmt::Debug + Clone, const N: usize>{
///   array: [T;N]
/// }
/// ```
///
/// There's two important points to remark here:
///
/// - An implementor's generic will be added to the struct iff there's not a generic with the same
///   name already defined. This is something to keep in mind to avoid situations like the
///   following:
///
/// ```compile_fail
/// use rust_writer::ast::{finder, local_finder, implementors::ItemToTrait, finder::ToFind};
/// use syn::{visit::Visit, File};
///
/// #[local_finder('a)]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone, const N: usize>{
///   found: [bool; N],
///   something: T
/// }
///
/// impl<'a, T, const N: usize> Visit<'a> for LocalImplementor<T, N> where T: std::fmt::Debug + Clone{
///   fn visit_file(&mut self, _file: &'a File){
///     // Dummy implementation
///     self.found = [true; N];
///   }
/// }
///
/// // This cannot compile even if the macro receives the correct bounds, due to T is already
/// // defined in the struct.
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T: std::fmt::Debug +
/// Clone,N>)]
/// struct NewImplementor<T, const N: usize>{
///   array: [T;N]
/// }
/// ```
///
/// - Const generics **MUST** be defined in the struct definition, even if they are only needed for
///   the implementor due to a parsing reason.
///
/// ```compile_fail
/// use rust_writer::ast::{finder, local_finder, implementors::ItemToTrait, finder::ToFind};
/// use syn::{visit::Visit, File};
///
/// #[local_finder('a)]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone, const N: usize>{
///   found: [bool; N],
///   something: T
/// }
///
/// impl<'a, T, const N: usize> Visit<'a> for LocalImplementor<T, N> where T: std::fmt::Debug + Clone{
///   fn visit_file(&mut self, _file: &'a File){
///     // Dummy implementation
///     self.found = [true; N];
///   }
/// }
///
/// // This cannot compile cause the const bound is defined in the macro.
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T,const N: usize>)]
/// struct NewImplementor<T: std::fmt::Debug + Clone>{
///   value: T
/// }
/// ```
///
/// # The `#[impl_from]` attribute
///
/// As soon as a new implementor is defined using the `#[finder]` macro, it's ready to be used!
/// But constructing a variable of this new type may be a bit annoying:
///
/// ```rust
/// use rust_writer::ast::{finder, implementors::ItemToTrait, finder::ToFind};
/// use syn::{parse_quote, TraitItem};
///
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>)]
/// struct NewImplementor;
///
/// let itemtotrait: ItemToTrait = ("MyTrait", parse_quote!(type Type1: From<String>;)).into();
/// let itemtotrait_1: ItemToTrait = ("MyTrait", parse_quote!(type Type2: From<String>;)).into();
///
/// let new_implementor = NewImplementor{ itemtotrait, itemtotrait_1 };
/// ```
///
/// The process, in this simple case, implies creating the variables for the implementors and
/// remembering how the fields should be named inside `NewImplementor`. The `#[impl_from]` attribute
/// comes in handy to create the `NewImplementor` variable at once, as it implements the `From`
/// trait from a tuple consisting in the struct original fields + the implementors fields.  
///
/// ```rust
/// use rust_writer::ast::{finder, implementors::ItemToTrait, finder::ToFind};
/// use syn::parse_quote;
///
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>)]
/// #[impl_from]
/// struct NewImplementor;
///
/// let new_implementor: NewImplementor = (
///   ("MyTrait", parse_quote!(type Type1: From<String>;)).into(),
///   ("MyTrait", parse_quote!(type Type2: From<String>;)).into()
/// ).into();
/// ```
/// This is much more compact! If the new implementor combines a lot of implementors instead of
/// just two, this will be even more evident. On the other hand, it's less explicit, so just a
/// trade-off to consider.
///
/// # How to use the `#[finder]` macro
///
/// Typically, an implementor from the predefined set of implementors is used to assert that an
/// element belongs to an AST by loading it into a
/// [`Finder`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html).
/// Such `Finder` instances implement [`Visit`](https://docs.rs/syn/latest/syn/visit/trait.Visit.html),
/// and hence the [`find`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html#method.find)
/// method can be called on them.
///
/// However, when an implementor is created with the `#[finder]` macro, `Visit` cannot be
/// implemented for `Finder` instances loading that implementor due to the orphan rule. The
/// `#[finder]` macro does the following to mimic that behavior:
///
/// 1. Implements the `ToFind` trait for this implementor, so a `Finder` instance can be loaded,
///    even if the `find` method isn't available for it. That instance's `found` array has an entry
///    for each inner implementor, reflecting if that implementor finds a match.
///
/// 1. Creates a wrapper for that `Finder` called `implementor_name + FinderWrapper`. So for an
///    implementor called `NewImplementor` this wrapper would be called
///    `NewImplementorFinderWrapper`. The `From` trait is implemented for `Finder`, so constructing
///    this wrapper is seamless.
///
/// 1. Implements a `find` method for the wrapper which works exactly as the `find` method works for
///    a regular `Finder`, with the only difference that it accepts an extra parameter of the type
///    `Option<&[u32]>`. If it's `Some`, the inner slice would tell the wrapper which implementors
///    apply to the AST. If the parameter is `None`, all the implementors are applied.
///
/// 1. Implements the method `fn get_missing_indexes(&self) -> Option<Vec<u32>>` which precisely
///    reflects which implementors have succeeded in their research up to the point of this call.
///
/// ```rust
/// use rust_writer::ast::{finder, implementors::ItemToTrait, finder::{ToFind, Finder}};
/// use syn::parse_quote;
/// use test_builder::TestBuilder;
///
/// #[finder(ItemToTrait<'a>, ItemToTrait<'a>)]
/// #[impl_from]
/// struct NewImplementor;
///
/// TestBuilder::default().with_trait_ast().execute(|mut builder|{
///  let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");
///
///  let new_implementor: NewImplementor = (
///    ("MyTrait", parse_quote!(type Type1: From<String>;)).into(),
///    ("MyTrait", parse_quote!(type Type3: From<String>;)).into()
///  ).into();
///
///  let mut finder: NewImplementorFinderWrapper =
///   Finder::default().to_find(&new_implementor).into();
///
///
///  // Apply both finders -> Cannot succeed as the `Type3` isn't in the AST
///  assert!(!finder.find(ast, None));
///
///  // The `get_missing_indexes` method tells that the second implementor didn't succeed in its
///  // research.
///  assert_eq!(finder.get_missing_indexes(), Some(vec![1]));
///
///  // If we look for two types which effectively belongs to `MyTrait`.
///  let new_implementor: NewImplementor = (
///    ("MyTrait", parse_quote!(type Type1: From<String>;)).into(),
///    ("MyTrait", parse_quote!(type Type2: AsRef<Path>;)).into()
///  ).into();
///
///  let mut finder: NewImplementorFinderWrapper =
///   Finder::default().to_find(&new_implementor).into();
///
///  assert!(finder.find(ast, None));
///  assert_eq!(finder.get_missing_indexes(), None);
/// });
/// ```
///
/// # Compatibility with [`#[mutator]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.mutator.html)
///
/// It's perfectly possible to use the `#[finder]` macro in combination with the `#[mutator]`
/// macro. The order in which macros are applied doesn't matter.
///
/// **There's just one golden rule to combine both macros**: The list of implementors should be the
/// same and come in the same order in both macros.
///
/// So this is perfectly valid:
///
/// ```rust
/// use rust_writer::ast::{
///   finder,
///   mutator,
///   implementors::{ItemToTrait, ItemToImpl},
///   finder::ToFind,
///   mutator::ToMutate
/// };
/// use syn::visit_mut::VisitMut;
///
/// #[finder(ItemToTrait<'a>, ItemToImpl<'a>)]
/// #[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
/// struct NewImplementor;
/// ```
/// While this doesn't even compile:
///
/// ```compile_fail
/// use rust_writer::ast::{
///   finder,
///   mutator,
///   implementors::{ItemToTrait, ItemToImpl},
///   finder::ToFind,
///   mutator::ToMutate
/// };
/// use syn::visit_mut::VisitMut;
///
/// #[finder(ItemToTrait<'a>, ItemToImpl<'a>)]
/// #[mutator(ItemToImpl<'a>, ItemToTrait<'a>)]
/// struct NewImplementor;
/// ```  
#[proc_macro_attribute]
pub fn finder(attrs: TokenStream, item: TokenStream) -> TokenStream {
	finder::finder(attrs, item)
}

/// The `#[local_finder]` macro is used to create a custom implementor mimicking the behavior of
/// [`Finder`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html).
///
/// While creating an implementor may seem an easy task, the orphan rule can complicate matters.
/// If a custom implementor implements `ToFind`, it's not possible to implement
/// [`Visit`](https://docs.rs/syn/latest/syn/visit/trait.Visit.html) for `Finder<CustomImplementor>`,
/// so the [`find`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html#method.find)
/// wouldn't be available and that `Finder` type would be totally useless.
///
/// The workaround is to implement `Visit` for the custom implementor itself and use it as if it was
/// a `Finder` variable. The `#[local_finder]` macro takes such a type and implements the `find`
/// and `reset` methods directly on it, mimicking the behavior of a regular `Finder` instance.
///
///```rust
/// use rust_writer::ast::{local_finder, finder::ToFind};
/// use syn::{visit::Visit, File, ItemTrait};
/// use test_builder::TestBuilder;
///
/// // This local finder will simply assert if a trait exists in the AST
/// #[local_finder('a)]
/// #[derive(Debug, Clone)]
/// struct FindTrait<'a>{
///   found: [bool; 1],
///   trait_name: &'a str
/// }
///
/// impl<'a> Visit<'a> for FindTrait<'a>{
///   fn visit_item_trait(&mut self, item_trait: &'a ItemTrait){
///     if item_trait.ident == self.trait_name{
///       self.found[0] = true;
///     }
///   }
/// }
///
/// TestBuilder::default().with_trait_ast().execute(|builder|{
///   let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed");
///
///   let mut finder = FindTrait{ found: [false; 1], trait_name: "MyTrait" };
///
///   // The trait is found in the AST
///   assert!(finder.find(&ast));
///
///   // The reset method works exactly as it does for `Finder`
///   assert_eq!(finder.found, [true]);
///   finder.finder_reset();
///   assert_eq!(finder.found, [false]);
///
///   // If the trait isn't in the AST, it's not found.
///   let mut finder = FindTrait{ found: [false; 1], trait_name: "OtherTrait" };
///   assert!(!finder.find(&ast));
/// });
/// ```
///
/// From the example we can learn a few things:
///
/// - The `reset` method is called `finder_reset` in this case. This is because a local finder can
///   also be a local mutator, hence this distinction is needed.
///
/// - The `#[local_finder]` macro needs an argument. This argument is a lifetime that must outlive
///   the lifetime used in the `Visit` impl block. So if the `Visit` impl block has a lifetime `'a`,
///   `#[local_finder('a)]`, `#[local_finder('b: 'a)]`,`#[local_finder('b: 'a + 'c + 'd)]` are all
///   valid invocations, while `#[local_finder('b)]` would be invalid.
#[proc_macro_attribute]
pub fn local_finder(attrs: TokenStream, item: TokenStream) -> TokenStream {
	finder::local_finder(attrs, item)
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn already_expanded(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}
