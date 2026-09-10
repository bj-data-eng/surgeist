use std::collections::BTreeSet;

#[path = "support/csstree_adapters.rs"]
mod adapters;

use adapters::{
    EntryPoint, Extractor, FontFaceDescriptorKind, OptionsProfile, PropertyOrDescriptor,
    TopLevelRuleKind, UnsupportedPolicy, UnsupportedReason,
};
use surgeist_css::{CssDeclarationList, CssRule, CssSheet, parse_sheet};

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

struct GroupFixture {
    path: &'static str,
    rule_kind: TopLevelRuleKind,
    source: &'static str,
    neutral: &'static str,
    cases: &'static [(&'static str, &'static str, usize)],
}

// Conditional 3 section 6 and Cascade 6 section 2.5.2 accept a <rule-list>;
// Syntax 3 section 5.4.1 permits that list to be empty. Conditional 4 section 2
// supplies selector() in the two supports cases. These are the catalog's pinned
// 2024-08-15, 2024-09-06, 2021-12-24, and 2025-09-04 publications, respectively.
// The case labels, inputs, and child counts below come from the independently
// imported CSSTree fixtures, not from the Surgeist parser or captured oracle.
const GROUP_FIXTURES: [GroupFixture; 2] = [
    GroupFixture {
        path: "expectations/atrule/atrule/supports.json",
        rule_kind: TopLevelRuleKind::Supports,
        source: include_str!("corpus/csstree/source/atrule/atrule/supports.json"),
        neutral: include_str!("corpus/csstree/expectations/atrule/atrule/supports.json"),
        cases: &[
            (
                "@supports with selector()",
                "@supports selector(.example) {}",
                0,
            ),
            (
                "@supports with selector() case-insensitive",
                "@supports SELECTOR(.example) {}",
                0,
            ),
            (
                "base test with comments",
                "@supports not /*0*/(/*1*/flex :/*3*/1/*4*/)/*5*/{}",
                0,
            ),
            (
                "base test with spaces",
                "@supports  (  flex  :  1  )  {}",
                0,
            ),
            (
                "complex prelude",
                "@supports (not (flex: 1)) or (grid: support) {}",
                0,
            ),
            ("custom property", "@supports (--custom: 1){}", 0),
            ("should be case insensitive", "@SuppOrts (flex:1){}", 0),
            (
                "simple supports with negation",
                "@supports not (flex: 1) {}",
                0,
            ),
            (
                "using !important",
                "@supports (box-shadow: something var(--complex) !important) {}",
                0,
            ),
            ("using function", "@supports func(flex: 1){}", 0),
            ("vendor property", "@supports (-vendor-name:1){}", 0),
            ("base test", "@supports (flex:1){selector{color:green}}", 1),
        ],
    },
    GroupFixture {
        path: "expectations/atrule/atrule/scope.json",
        rule_kind: TopLevelRuleKind::Scope,
        source: include_str!("corpus/csstree/source/atrule/atrule/scope.json"),
        neutral: include_str!("corpus/csstree/expectations/atrule/atrule/scope.json"),
        cases: &[
            ("only limit", "@scope to (limit){}", 0),
            ("only root", "@scope (root){}", 0),
            ("base syntax", "@scope (a) to (b) { c {} }", 1),
        ],
    },
];

fn group_case_id(fixture: &GroupFixture, label: &str) -> String {
    let path = fixture
        .path
        .strip_prefix("expectations/")
        .expect("neutral expectation path");
    format!("{path}#/{label}")
}

#[test]
fn csstree_group_rules_retain_empty_and_nonempty_blocks() {
    for fixture in &GROUP_FIXTURES {
        let source: serde_json::Value =
            serde_json::from_str(fixture.source).expect("imported fixture source");
        let neutral: serde_json::Value =
            serde_json::from_str(fixture.neutral).expect("neutral fixture");
        let neutral_cases = neutral["cases"].as_array().expect("neutral cases");

        for &(label, input, child_count) in fixture.cases {
            let id = group_case_id(fixture, label);
            let case = neutral_cases
                .iter()
                .find(|case| case["id"] == id)
                .unwrap_or_else(|| panic!("missing neutral case {id}"));
            assert_eq!(case["input"], input, "{id}: frozen neutral input");
            assert_eq!(source[label]["source"], input, "{id}: imported input");
            assert_eq!(
                source[label]["ast"]["block"]["children"]
                    .as_array()
                    .expect("imported block children")
                    .len(),
                child_count,
                "{id}: independently frozen child count"
            );

            let report = parse_sheet(input);
            assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
            let [rule] = report.syntax().rules() else {
                panic!("{id}: expected exactly one retained outer rule");
            };
            let actual_children = match (fixture.rule_kind, rule) {
                (TopLevelRuleKind::Supports, CssRule::Supports(rule)) => rule.rules().len(),
                (TopLevelRuleKind::Scope, CssRule::Scope(rule)) => rule.rules().rules().len(),
                _ => panic!("{id}: incorrect retained outer rule: {rule:?}"),
            };
            assert_eq!(actual_children, child_count, "{id}: retained children");
        }
    }
}

#[test]
fn csstree_group_registry_counts_retained_outer_rules() {
    for fixture in &GROUP_FIXTURES {
        let entry = adapters::REGISTRY
            .iter()
            .find(|entry| entry.fixture_path() == fixture.path)
            .expect("group fixture registry entry");
        assert_eq!(
            entry.extractor(),
            Extractor::TopLevelRuleKind(fixture.rule_kind),
            "{}: empty group blocks still retain their outer rule",
            fixture.path
        );
        for &(label, input, _) in fixture.cases {
            let report = parse_sheet(input);
            assert_eq!(
                entry.extractor().extract_sheet(report.syntax()),
                Ok(1),
                "{}: registry must observe the outer rule",
                group_case_id(fixture, label)
            );
        }
    }
}

#[test]
fn csstree_group_expected_classes_bind_retained_outer_rules() {
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json"))
            .expect("CSS-owned expected-class registry");
    let records = expected["records"]
        .as_array()
        .expect("expected-class records");
    for fixture in &GROUP_FIXTURES {
        let expected_class = serde_json::json!({
            "kind": "clean",
            "retained_syntax": {
                "extractor": {
                    "kind": "top_level_rule_kind",
                    "rule_kind": fixture.rule_kind.name(),
                },
                "predicate": { "relation": "nonempty" },
            },
        });
        for &(label, _, _) in fixture.cases {
            let id = group_case_id(fixture, label);
            let record = records
                .iter()
                .find(|record| record["id"] == id)
                .unwrap_or_else(|| panic!("missing expected class {id}"));
            assert_eq!(
                record["class"], expected_class,
                "{id}: cleanliness and outer-rule retention are independent of child count"
            );
        }
    }
}
