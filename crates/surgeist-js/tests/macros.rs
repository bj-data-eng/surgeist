//! Tests for the macros in this crate.

#![allow(unused_crate_dependencies)]

use surgeist_js::value::TryFromJs;
use surgeist_js::{Context, JsResult, JsValue, Source, js_string};
use surgeist_js_string::JsString;

#[test]
fn try_from_js_derive() {
    #[derive(Debug, TryFromJs, Eq, PartialEq)]
    struct TryFromJsTest {
        a: JsString,
        #[boa(rename = "bBB")]
        b: i32,
        #[boa(from_js_with = "check_tfj_called")]
        c: i32,
    }

    fn check_tfj_called(
        value: &JsValue,
        mut context: surgeist_js::ContextAccess<'_>,
    ) -> JsResult<i32> {
        let v = value.to_i32(context.reborrow())?;
        Ok(v / 2)
    }

    let mut context = Context::builder().build().unwrap();
    let obj = context
        .eval(Source::from_bytes(br#"({ a: "hello", bBB: 42, c: 120 })"#))
        .unwrap();

    let result = TryFromJsTest::try_from_js(&obj, context.access()).unwrap();
    assert_eq!(
        result,
        TryFromJsTest {
            a: js_string!("hello"),
            b: 42,
            c: 60
        }
    );
}
