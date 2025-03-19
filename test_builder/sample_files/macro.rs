// SPDX-License-Identifier: GPL-3.0

my_macro! {
    type Type = From<String>;

    enum SomeEnum{
        A,
        B,
        C(u8, String)
    }
}
