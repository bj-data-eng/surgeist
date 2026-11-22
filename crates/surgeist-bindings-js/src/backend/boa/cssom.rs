use boa_engine::{JsData, JsObject};
use boa_gc::{Finalize, Trace};

#[derive(Trace, Finalize, JsData)]
pub(super) struct CssomPrototypes(pub(super) Vec<JsObject>);
