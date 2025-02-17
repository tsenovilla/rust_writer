use std::path::Path;

trait MyTrait{
    type Type1: From<String>;
    type Type2: AsRef<Path>;

    fn my_fun() -> Self;
}
