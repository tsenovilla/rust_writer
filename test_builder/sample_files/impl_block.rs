// SPDX-License-Identifier: GPL-3.0

impl SomeTrait for SomeImplementor{
    type Type1 = u8;

    fn some_func(&self) -> bool{
        true
    }
}

impl SomeImplementor{
    fn some_super_func(&self) -> bool{
        true
    }
}
