//! Tests for the macros in this crate.

#![allow(unused_crate_dependencies)]

use surgeist_js_macros::utf16;

#[test]
fn literal() {
    let utf16 = utf16!("hello!");
    let manual = "hello!".encode_utf16().collect::<Vec<_>>();
    assert_eq!(manual, utf16);
}

#[test]
fn utf16() {
    let utf16 = utf16!("hello!😁😁😁");
    let manual = "hello!😁😁😁".encode_utf16().collect::<Vec<_>>();
    assert_eq!(manual, utf16);
}
