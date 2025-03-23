// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{mutator, local_mutator, implementors::ItemToTrait, mutator::ToMutate};
use syn::{visit_mut::VisitMut, File};

#[local_mutator]
#[derive(Debug, Clone)]
struct LocalImplementor<T: std::fmt::Debug + Clone>{
  mutated: [bool; 4],
  something: T 
}

impl<T> VisitMut for LocalImplementor<T> where T: std::fmt::Debug + Clone{
  fn visit_file_mut(&mut self, _file: &mut File){
    // Dummy implementation
    self.mutated = [true; 4];
  }
}

#[mutator(ItemToTrait<'a>, ItemToTrait<'a>, LocalImplementor<T: std::fmt::Debug + Clone>)]
struct NewImplementor;

fn main(){}
