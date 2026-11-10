//! Tests for optional values in `TryFromJs` derive.

#![allow(unused_crate_dependencies)]

use surgeist_js::Source;
use surgeist_js::value::TryFromJs;

#[derive(PartialEq, Eq, TryFromJs)]
struct Deserialize {
    required: String,
    optional: Option<String>,
}

#[test]
fn optional_missing_try_from_js() {
    let mut context = surgeist_js::Context::builder().build().unwrap();
    let value = context
        .eval(Source::from_bytes(
            r#"
            let empty = {
                "required":"foo",
            };
            empty
        "#,
        ))
        .unwrap();

    let deserialized: Deserialize = Deserialize::try_from_js(&value, context.access()).unwrap();
    assert_eq!(deserialized.required, "foo");
    assert_eq!(deserialized.optional, None);
}

#[test]
fn optional_try_from_js() {
    let mut context = surgeist_js::Context::builder().build().unwrap();
    let value = context
        .eval(Source::from_bytes(
            r#"
            let empty = {
                "required": "foo",
                "optional": "bar",
            };
            empty
        "#,
        ))
        .unwrap();

    let deserialized: Deserialize = Deserialize::try_from_js(&value, context.access()).unwrap();
    assert_eq!(deserialized.required, "foo");
    assert_eq!(deserialized.optional, Some("bar".to_string()));
}

#[test]
fn required_missing_try_from_js() {
    let mut context = surgeist_js::Context::builder().build().unwrap();
    let value = context
        .eval(Source::from_bytes(
            r"
            let value = {};
            value
        ",
        ))
        .unwrap();

    assert!(
        Deserialize::try_from_js(&value, context.access()).is_err(),
        "foo"
    );
}
