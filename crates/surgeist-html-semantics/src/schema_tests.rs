//! The public resource vocabularies of this crate, compared with the HTML
//! accounting schema's dimensions and cap scopes.

use crate::{HtmlCaptureResource, HtmlSemanticResource};
use surgeist_html::accounting::{HtmlCapScope, HtmlResource, HtmlWork};

/// The schema dimension of a semantic resource. The integer digit cap bounds
/// one integer and has no dimension.
fn semantic(resource: HtmlSemanticResource) -> Option<Result<HtmlResource, HtmlWork>> {
    match resource {
        HtmlSemanticResource::Work => Some(Err(HtmlWork::SemanticStep)),
        HtmlSemanticResource::OutputRecords => Some(Ok(HtmlResource::OutputRecords)),
        HtmlSemanticResource::ScratchRecords => Some(Ok(HtmlResource::ScratchRecords)),
        HtmlSemanticResource::IntegerDigits => None,
        HtmlSemanticResource::AllocatedBytes => Some(Ok(HtmlResource::AllocatedBytes)),
        HtmlSemanticResource::NestedOutputRecords => Some(Ok(HtmlResource::NestedOutputRecords)),
    }
}

fn capture(resource: HtmlCaptureResource) -> HtmlResource {
    match resource {
        HtmlCaptureResource::Subjects => HtmlResource::CapturedSubjects,
        HtmlCaptureResource::Relationships => HtmlResource::CapturedRelationships,
        HtmlCaptureResource::Attributes => HtmlResource::CapturedAttributes,
        HtmlCaptureResource::Utf16Units => HtmlResource::CapturedUtf16Units,
        HtmlCaptureResource::Depth => HtmlResource::CapturedDepth,
        HtmlCaptureResource::Registries => HtmlResource::CapturedRegistries,
        HtmlCaptureResource::Definitions => HtmlResource::CapturedDefinitions,
        HtmlCaptureResource::ElementValueBytes => HtmlResource::ElementValueBytes,
    }
}

#[test]
fn the_query_scope_caps_every_semantic_resource_but_the_integer_digits() {
    let all = [
        HtmlSemanticResource::Work,
        HtmlSemanticResource::OutputRecords,
        HtmlSemanticResource::ScratchRecords,
        HtmlSemanticResource::IntegerDigits,
        HtmlSemanticResource::AllocatedBytes,
        HtmlSemanticResource::NestedOutputRecords,
    ];
    let mut resources = 0;
    for resource in all {
        match semantic(resource) {
            Some(Ok(dimension)) => {
                resources += 1;
                assert!(HtmlCapScope::Query.capped_resources().contains(&dimension));
            }
            Some(Err(dimension)) => {
                assert!(HtmlCapScope::Query.capped_work().contains(&dimension));
            }
            None => assert_eq!(resource, HtmlSemanticResource::IntegerDigits),
        }
    }
    assert_eq!(HtmlCapScope::Query.capped_resources().len(), resources);
    assert_eq!(HtmlCapScope::Query.capped_work().len(), 1);
}

#[test]
fn the_capture_scope_caps_exactly_the_capture_resources() {
    let all = [
        HtmlCaptureResource::Subjects,
        HtmlCaptureResource::Relationships,
        HtmlCaptureResource::Attributes,
        HtmlCaptureResource::Utf16Units,
        HtmlCaptureResource::Depth,
        HtmlCaptureResource::Registries,
        HtmlCaptureResource::Definitions,
        HtmlCaptureResource::ElementValueBytes,
    ];
    let capped: Vec<HtmlResource> = all.into_iter().map(capture).collect();
    assert_eq!(HtmlCapScope::Capture.capped_resources(), capped.as_slice());
}
