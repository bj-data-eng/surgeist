use surgeist_js::{ContextAccess, JsResult, JsValue, value::TryFromJs};

struct Escaping<'a>(ContextAccess<'a>);

impl<'a> TryFromJs for Escaping<'a> {
    fn try_from_js(_: &JsValue, access: ContextAccess<'_>) -> JsResult<Self> {
        Ok(Self(access))
    }
}

fn main() {}
