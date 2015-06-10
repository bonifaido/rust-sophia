extern crate gcc;

fn main() {
    gcc::compile_library("libsophia.a", &["sophia/sophia.c"]);
}
