// SPDX-License-Identifier: GPL-3.0

mod SomeMod{
    enum A{
        A,
        B,
        C
    }

    fn some_super_func(&self) -> bool{
        true
    }

    /// Doc
    #[some_attr]
    trait SomeTrait{}
}
