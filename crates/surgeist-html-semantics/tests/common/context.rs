use surgeist_html_semantics::{HtmlCapture, HtmlSemanticContext, HtmlSemanticContextInput};

pub fn context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap()
}
