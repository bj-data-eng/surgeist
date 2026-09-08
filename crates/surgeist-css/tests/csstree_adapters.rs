use std::collections::BTreeSet;

#[path = "support/csstree_adapters.rs"]
mod adapters;

use adapters::{
    EntryPoint, Extractor, FontFaceDescriptorKind, OptionsProfile, PropertyOrDescriptor,
    TopLevelRuleKind, UnsupportedPolicy, UnsupportedReason,
};
use surgeist_css::{CssDeclarationList, CssSheet};

#[test]
fn registry_covers_every_fixture_path() {
    let registry = adapters::REGISTRY;
    assert!(adapters::validate_closed_model());
    assert_eq!(registry.len(), 74);

    let paths = registry
        .iter()
        .map(|entry| entry.fixture_path())
        .collect::<BTreeSet<_>>();
    assert_eq!(paths.len(), registry.len());
    let oracle: serde_json::Value = serde_json::from_str(include_str!("csstree/oracle.json"))
        .expect("committed oracle should deserialize");
    let fixture_paths = oracle["records"]
        .as_array()
        .expect("oracle records")
        .iter()
        .map(|record| record["path"].as_str().expect("oracle fixture path"))
        .collect::<BTreeSet<_>>();
    assert_eq!(paths, fixture_paths);
    assert!(
        registry.iter().all(|entry| {
            entry.has_legal_context_combination()
                && !entry.context().is_empty()
                && entry.accepts_options(OptionsProfile::Empty)
                && !entry.adapter_name().is_empty()
                && !entry.entry_point_name().is_empty()
                && !entry.extractor_name().is_empty()
                && entry.unsupported_policy_name().is_some()
        }),
        "every fixture path must explicitly name its adapter, entry point, extractor, and unsupported policy"
    );
}

#[test]
fn truthful_complete_input_adapters_require_full_observation() {
    let truthful_adapters = [
        adapters::Adapter::Stylesheet,
        adapters::Adapter::TopLevelRule,
        adapters::Adapter::TopLevelAtRule,
        adapters::Adapter::StyleDeclarationList,
    ];

    for adapter in truthful_adapters {
        let entries = adapters::REGISTRY
            .iter()
            .filter(|entry| entry.adapter() == adapter)
            .collect::<Vec<_>>();
        assert!(!entries.is_empty(), "missing {adapter:?} registry coverage");
        assert!(
            entries.iter().all(|entry| {
                entry.unsupported_policy() == Some(UnsupportedPolicy::FullObservation)
            }),
            "truthful {adapter:?} adapters must retain a full observation"
        );
    }

    assert!(
        adapters::REGISTRY.iter().any(|entry| {
            entry.unsupported_policy() == Some(UnsupportedPolicy::FullObservation)
        })
    );
    assert!(
        adapters::REGISTRY.iter().any(|entry| {
            entry.unsupported_policy() == Some(UnsupportedPolicy::PanicFreedomOnly)
        })
    );
    assert!(adapters::REGISTRY.iter().all(|entry| {
        if entry.adapter() == adapters::Adapter::CustomPropertyContainment {
            !entry.has_truthful_complete_input()
                && entry.unsupported_policy() == Some(UnsupportedPolicy::PanicFreedomOnly)
                && entry.unsupported_reason()
                    == Some(
                        UnsupportedReason::GenericFragmentWithoutTruthfulSupportedPropertyOrDescriptor,
                    )
        } else {
            entry.has_truthful_complete_input()
                && entry.unsupported_policy() == Some(UnsupportedPolicy::FullObservation)
                && entry.unsupported_reason().is_none()
        }
    }));
}

#[test]
fn fixed_wrappers_preserve_one_contiguous_payload_and_exact_span() {
    let input = "λ/*surgeist*/λ";
    for entry in adapters::REGISTRY {
        let complete = entry
            .adapter()
            .wrap(input, entry.property_or_descriptor())
            .unwrap_or_else(|mismatch| panic!("{}: {mismatch:?}", entry.fixture_path()));
        let span = complete.payload_span();
        assert_eq!(span, span.start..span.start + input.len());
        assert_eq!(&complete.source()[span], input);

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/corpus/csstree")
            .join(entry.fixture_path());
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let fixture: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("failed to deserialize {}: {error}", path.display()));
        for case in fixture["cases"].as_array().expect("neutral cases") {
            let input = case["input"].as_str().expect("neutral input");
            let complete = entry
                .adapter()
                .wrap(input, entry.property_or_descriptor())
                .unwrap_or_else(|mismatch| {
                    panic!("{} input {input:?}: {mismatch:?}", entry.fixture_path())
                });
            let span = complete.payload_span();
            assert_eq!(span, span.start..span.start + input.len());
            assert_eq!(&complete.source()[span.clone()], input);
            if !input.is_empty() {
                assert_eq!(complete.source().match_indices(input).count(), 1);
            }
        }
    }
}

#[test]
fn extractor_inventory_uses_only_public_accessors() {
    let sheet = CssSheet::new();
    let rule_kinds = [
        TopLevelRuleKind::Import,
        TopLevelRuleKind::Namespace,
        TopLevelRuleKind::CounterStyle,
        TopLevelRuleKind::Page,
        TopLevelRuleKind::LayerStatement,
        TopLevelRuleKind::LayerBlock,
        TopLevelRuleKind::FontFace,
        TopLevelRuleKind::Keyframes,
        TopLevelRuleKind::Style,
        TopLevelRuleKind::Media,
        TopLevelRuleKind::Supports,
        TopLevelRuleKind::Container,
        TopLevelRuleKind::Scope,
    ];
    let descriptors = [
        FontFaceDescriptorKind::FontFamily,
        FontFaceDescriptorKind::Src,
        FontFaceDescriptorKind::FontWeight,
        FontFaceDescriptorKind::FontStyle,
        FontFaceDescriptorKind::FontStretch,
        FontFaceDescriptorKind::FontDisplay,
        FontFaceDescriptorKind::UnicodeRange,
        FontFaceDescriptorKind::FontFeatureSettings,
    ];
    let extractors = [
        Extractor::SheetRules,
        Extractor::StyleDeclarations,
        Extractor::StyleSelector,
        Extractor::MediaQueries,
        Extractor::MediaChildren,
        Extractor::SupportsChildren,
        Extractor::ContainerChildren,
        Extractor::ScopeChildren,
        Extractor::LayerBlockChildren,
    ];

    for extractor in extractors
        .into_iter()
        .chain(rule_kinds.map(Extractor::TopLevelRuleKind))
        .chain(descriptors.map(Extractor::FontFaceDescriptor))
    {
        assert_eq!(extractor.extract_sheet(&sheet), Ok(0));
    }
    assert!(rule_kinds.iter().all(|kind| !kind.name().is_empty()));
    assert!(
        descriptors
            .iter()
            .all(|descriptor| !descriptor.name().is_empty())
    );
    let _: fn(Extractor, &CssDeclarationList) -> Result<usize, adapters::Mismatch> =
        Extractor::extract_declaration_list;
    assert_eq!(
        UnsupportedPolicy::FullObservation.name(),
        "full_observation"
    );

    assert!(adapters::REGISTRY.iter().all(|entry| {
        !matches!(
            (entry.adapter(), entry.extractor()),
            (
                adapters::Adapter::FontFaceDescriptorValue,
                Extractor::TopLevelRuleKind(TopLevelRuleKind::FontFace)
            )
        )
    }));
    assert!(adapters::REGISTRY.iter().all(|entry| {
        entry
            .property_or_descriptor()
            .is_none_or(|owner| !owner.name().is_empty())
            && entry.unsupported_policy().is_some()
            && entry
                .unsupported_reason()
                .is_none_or(|reason| !reason.name().is_empty())
    }));
    assert!(adapters::REGISTRY.iter().all(|entry| {
        entry.property_or_descriptor()
            != Some(PropertyOrDescriptor::FontFaceDescriptor(
                FontFaceDescriptorKind::UnicodeRange,
            ))
            || entry.entry_point() == EntryPoint::Sheet
    }));
}
