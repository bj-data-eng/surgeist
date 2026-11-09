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

/// What a limits field maps to in the schema.
#[derive(Debug, PartialEq)]
enum Mapped {
    Work(HtmlWork),
    Resource(HtmlResource),
    /// A cap that is not a ledger cap.
    Local,
}

/// The field names of a limits value, read from its `Debug` text.
fn field_names(limits: &impl std::fmt::Debug) -> Vec<String> {
    let text = format!("{limits:?}");
    let body = text
        .split_once('{')
        .and_then(|(_, rest)| rest.rsplit_once('}'))
        .map_or("", |(body, _)| body);
    body.split(',')
        .filter_map(|field| field.split_once(':'))
        .map(|(name, _)| name.trim().to_owned())
        .collect()
}

/// Checks that the fields of `limits` map to exactly the dimensions `scope`
/// caps, so a field added to the owner type fails here until the schema lists it.
fn pin(scope: HtmlCapScope, limits: impl std::fmt::Debug, map: fn(&str) -> Mapped) {
    let mut work = Vec::new();
    let mut resources = Vec::new();
    for name in field_names(&limits) {
        match map(&name) {
            Mapped::Work(dimension) => work.push(dimension),
            Mapped::Resource(dimension) => resources.push(dimension),
            Mapped::Local => {}
        }
    }
    let mut listed_work = scope.capped_work().to_vec();
    let mut listed_resources = scope.capped_resources().to_vec();
    work.sort();
    resources.sort();
    listed_work.sort();
    listed_resources.sort();
    assert_eq!(listed_work, work, "{scope:?} work");
    assert_eq!(listed_resources, resources, "{scope:?} resources");
}

/// The fact families share the record, code unit and validation dimensions.
fn fact(name: &str) -> Mapped {
    match name {
        "records" | "record_capacity" => Mapped::Resource(HtmlResource::FactRecords),
        "utf16_units" | "code_units" => Mapped::Resource(HtmlResource::FactCodeUnits),
        "bytes" => Mapped::Resource(HtmlResource::FactBytes),
        "assigned_nodes" => Mapped::Resource(HtmlResource::AssignedNodes),
        "validation_work" | "work" => Mapped::Work(HtmlWork::SemanticStep),
        other => panic!("unmapped fact limit {other}"),
    }
}

#[test]
fn each_fact_family_scope_caps_the_fields_of_its_limits() {
    use crate::{
        HtmlBaseFactsLimits, HtmlControlFactsLimits, HtmlElementReferenceFactsLimits,
        HtmlGlobalInheritanceLimits, HtmlImageDimensionFactsLimits, HtmlInertFactLimits,
        HtmlLanguageDirectionLimits, HtmlMediaFactsLimits, HtmlNaturalDimensionLimits,
        HtmlOutputDefaultLimits, HtmlRelFactsLimits,
    };
    pin(
        HtmlCapScope::MediaFacts,
        HtmlMediaFactsLimits::new(1, 2, 3),
        fact,
    );
    pin(
        HtmlCapScope::LinkRelationshipFacts,
        HtmlRelFactsLimits::new(1, 2, 3),
        fact,
    );
    pin(
        HtmlCapScope::ImageDimensionFacts,
        HtmlImageDimensionFactsLimits::new(1, 2, 3),
        fact,
    );
    pin(
        HtmlCapScope::NaturalDimensionFacts,
        HtmlNaturalDimensionLimits::new(1, 2, 3),
        fact,
    );
    pin(
        HtmlCapScope::BaseFacts,
        HtmlBaseFactsLimits::new(1, 2, 3),
        fact,
    );
    pin(
        HtmlCapScope::ElementReferenceFacts,
        HtmlElementReferenceFactsLimits::new(1, 2),
        fact,
    );
    pin(
        HtmlCapScope::ControlFacts,
        HtmlControlFactsLimits::new(1, 2),
        fact,
    );
    pin(
        HtmlCapScope::GlobalInheritance,
        HtmlGlobalInheritanceLimits::new(1, 2),
        fact,
    );
    pin(
        HtmlCapScope::OutputDefault,
        HtmlOutputDefaultLimits::new(1, 2, 3),
        fact,
    );
    pin(
        HtmlCapScope::InertFacts,
        HtmlInertFactLimits::new(1, 2),
        fact,
    );
    pin(
        HtmlCapScope::LanguageDirection,
        HtmlLanguageDirectionLimits::new(1, 2, 3, 4),
        fact,
    );
}

#[test]
fn the_query_capture_and_import_scopes_cap_the_fields_of_their_limits() {
    use crate::{HtmlCaptureLimits, HtmlConstructionCaptureLimits, HtmlSemanticLimits};
    pin(
        HtmlCapScope::Query,
        HtmlSemanticLimits::new(1, 2, 3, 4, 5, 6),
        |name| match name {
            "work" => Mapped::Work(HtmlWork::SemanticStep),
            "output_records" => Mapped::Resource(HtmlResource::OutputRecords),
            "scratch_records" => Mapped::Resource(HtmlResource::ScratchRecords),
            "integer_digits" => Mapped::Local,
            "allocated_bytes" => Mapped::Resource(HtmlResource::AllocatedBytes),
            "nested_output_records" => Mapped::Resource(HtmlResource::NestedOutputRecords),
            other => panic!("unmapped query limit {other}"),
        },
    );
    pin(
        HtmlCapScope::Capture,
        HtmlCaptureLimits::new(1, 2, 3, 4, 5, 6, 7, 8),
        |name| match name {
            "subjects" => Mapped::Resource(HtmlResource::CapturedSubjects),
            "relationships" => Mapped::Resource(HtmlResource::CapturedRelationships),
            "attributes" => Mapped::Resource(HtmlResource::CapturedAttributes),
            "utf16_units" => Mapped::Resource(HtmlResource::CapturedUtf16Units),
            "depth" => Mapped::Resource(HtmlResource::CapturedDepth),
            "registries" => Mapped::Resource(HtmlResource::CapturedRegistries),
            "definitions" => Mapped::Resource(HtmlResource::CapturedDefinitions),
            "element_value_bytes" => Mapped::Resource(HtmlResource::ElementValueBytes),
            other => panic!("unmapped capture limit {other}"),
        },
    );
    pin(
        HtmlCapScope::CaptureImport,
        HtmlConstructionCaptureLimits::new(1, 2, 3),
        |name| match name {
            "mapping_entries" => Mapped::Resource(HtmlResource::MappingEntries),
            "scratch_entries" => Mapped::Resource(HtmlResource::ScratchRecords),
            "work" => Mapped::Work(HtmlWork::SemanticStep),
            other => panic!("unmapped import limit {other}"),
        },
    );
}

#[test]
fn the_option_value_scope_caps_the_fields_of_its_envelope() {
    pin(
        HtmlCapScope::OptionValue,
        crate::HtmlOptionValueEnvelope::new(1, 2, 3),
        |name| match name {
            "bytes" => Mapped::Resource(HtmlResource::RetainedBytes),
            "units" => Mapped::Resource(HtmlResource::RetainedUtf16Units),
            "work" => Mapped::Work(HtmlWork::SemanticStep),
            other => panic!("unmapped envelope field {other}"),
        },
    );
}
