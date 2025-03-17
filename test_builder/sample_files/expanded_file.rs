// SPDX-License-Identifier: GPL-3.0

#![global_attr]

//! Some superusefuldocs
//! This file is just for testing!!! :)))

// A simple function
fn my_function() {
    println!("Hello from a function!");
}

// A macro invocation (built-in macro)
macro_rules! my_macro {
    ($stmt:stmt; $enum:item) => {
        $stmt;
        $enum
    };
}

// A struct
struct MyStruct {
    field1: i32,
    field2: String,
}

// A trait
trait MyTrait {
    fn trait_method(&self);
}

// Implementing the trait for the struct
impl MyTrait for MyStruct {
    fn trait_method(&self) {
        println!("Trait method called!");
    }

    fn other_method(&self) {
        println!("Trait method called!");
    }

    fn func(&self) -> bool {
        false
    }
}

// An impl block with an associated function
impl MyStruct {
    fn new(val: i32, text: &str) -> Self {
        Self {
            field1: val,
            field2: text.to_string(),
        }
    }
}

fn main() {
    my_function();

    my_macro!(

           // Nice comment
    let a = 1; enum
           A { B, C, D(u8, String) } struct SomeStruct { field : u8, string : String }
       );

    let instance = MyStruct::new(42, "Hello");
    instance.trait_method();
}

use std::path::Path;
