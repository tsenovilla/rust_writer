// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{finder, local_finder, implementors::ItemToTrait, finder::ToFind};
use syn::{visit::Visit, File};

#[local_finder('a)]
#[derive(Debug, Clone)]
struct LocalImplementor<T: std::fmt::Debug + Clone>{
  found: [bool; 4],
  something: T 
}

impl<'a, T> Visit<'a> for LocalImplementor<T> where T: std::fmt::Debug + Clone{
  fn visit_file(&mut self, _file: &'a File){
    // Dummy implementation
    self.found = [true; 4];
  }
}

#[finder(ItemToTrait<'a>, ItemToTrait<'a>, LocalImplementor<T: std::fmt::Debug + Clone>)]
struct NewImplementor;

fn main(){}
