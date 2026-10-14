use surgeist_html::*;

pub fn collection_limits(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
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
        if resource == HtmlSemanticResource::ScratchRecords {
            cap
        } else {
            1000
        },
        0,
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            1000000
        },
        if resource == HtmlSemanticResource::NestedOutputRecords {
            cap
        } else {
            100000
        },
    )
}
