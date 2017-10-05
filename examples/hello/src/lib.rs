#![feature(plugin)]
#![plugin(rfi_codegen)]

extern crate rfi;

#[rfi]
fn hello(name: String) -> String {
    format!("hello, {}", name)
}
