use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticLimits,
    HtmlSemanticSession,
};

pub fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    raw: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(raw), l, cancel).unwrap()
}
