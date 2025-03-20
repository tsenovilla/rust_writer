// SPDX-License-Identifier: GPL-3.0

use std::path::Path;

trait MyTrait{
    type Type1: From<String>;
    type Type2: AsRef<Path>;

    fn my_fun() -> Self;
}

impl SomeTrait for SomeImplementor{
    type Type = u8;

    fn some_func(&self) -> bool{
        true
    }
}
