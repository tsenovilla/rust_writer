// SPDX-License-Identifier: GPL-3.0

use std::path::Path;

trait MyTrait{
    type Type1: From<String>;
    type Type2: AsRef<Path>;

    /// Doc
    #[some_attr]
    type CommentedType: From<String>;

    fn my_fun() -> Self;
}
