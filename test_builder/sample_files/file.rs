// SPDX-License-Identifier: GPL-3.0

use std::path::PathBuf;

/// Doc
use std::fs;

mod SomeMod{
    enum A{
        A,
        B,
        C
    }

    fn some_super_func() -> bool{
        true
    }
}


some_macro! { that uses some tokens }

trait A{
    fn some_func(&self);
}

impl A for u8{
    fn some_func(&self){}
}
