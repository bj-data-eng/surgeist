use surgeist_html::*;

pub fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    raw: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(raw), l, cancel).unwrap()
}
