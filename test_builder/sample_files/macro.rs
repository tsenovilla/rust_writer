// SPDX-License-Identifier: GPL-3.0

my_macro! {
    type Type1 = From<String>;
    type Type2 = AsRef<Path>;

    enum SomeEnum{
        A,
        B,
        C(u8, String)
    }
}
