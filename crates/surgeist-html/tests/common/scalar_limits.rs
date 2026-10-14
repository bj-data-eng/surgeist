use surgeist_html::*;

pub fn scalar_limits(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(
        if resource == HtmlSemanticResource::Work {
            cap
        } else {
            20000000
        },
        if resource == HtmlSemanticResource::OutputRecords {
            cap
        } else {
            100
        },
        0,
        0,
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            1000000
        },
        0,
    )
}
