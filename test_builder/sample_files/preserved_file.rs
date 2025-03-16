///TEMP_DOC// SPDX-License-Identifier: GPL-3.0
///TEMP_DOC
///TEMP_DOC#![global_attr]
///TEMP_DOC
///TEMP_DOC//! Some superusefuldocs
///TEMP_DOC//! This file is just for testing!!! :)))
///TEMP_DOC
///TEMP_DOC// A simple function
///TEMP_DOCfn my_function() {
///TEMP_DOC    println!("Hello from a function!");
///TEMP_DOC}
///TEMP_DOC
///TEMP_DOC// A macro invocation (built-in macro)
///TEMP_DOCmacro_rules! my_macro {
///TEMP_DOC    ($stmt:stmt; $enum:item) => {
///TEMP_DOC        $stmt;
///TEMP_DOC        $enum
///TEMP_DOC    };
///TEMP_DOC}
///TEMP_DOC
///TEMP_DOC// A struct
struct MyStruct {
    field1: i32,
    field2: String,
}
///TEMP_DOC
///TEMP_DOC// A trait
///TEMP_DOCtrait MyTrait {
///TEMP_DOC    fn trait_method(&self);
///TEMP_DOC}
///TEMP_DOC
///TEMP_DOC// Implementing the trait for the struct
impl MyTrait for MyStruct {
    fn trait_method(&self) {
        println!("Trait method called!");
    }
///TEMP_DOC
///TEMP_DOC    fn other_method(&self) {
///TEMP_DOC        println!("Trait method called!");
///TEMP_DOC    }
///TEMP_DOC
type temp_marker = ();
}
///TEMP_DOC
///TEMP_DOC// An impl block with an associated function
///TEMP_DOCimpl MyStruct {
///TEMP_DOC    fn new(val: i32, text: &str) -> Self {
///TEMP_DOC        Self {
///TEMP_DOC            field1: val,
///TEMP_DOC            field2: text.to_string(),
///TEMP_DOC        }
///TEMP_DOC    }
///TEMP_DOC}
///TEMP_DOC
fn main() {
    my_function();
///TEMP_DOC
type temp_marker = ();
    my_macro!(
///TEMP_DOC        // Nice comment
type temp_marker = ();
        let a = 1;
        enum A {
            B,
            C,
            D(u8, String)
        }
    );
///TEMP_DOC
type temp_marker = ();
    let instance = MyStruct::new(42, "Hello");
    instance.trait_method();
}
type temp_marker = ();
