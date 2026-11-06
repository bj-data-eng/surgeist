#![allow(unused)]

use surgeist_js::value::TryFromJs;

#[derive(TryFromJs)]
struct TestStruct {
    inner: bool,
}

fn main() {}
