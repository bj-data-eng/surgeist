#[path = "common/property_expectations.rs"]
mod property_expectations;

use property_expectations::{BoundaryOutcome, CASES, CatalogExpectation};
use surgeist_css::{
    CssErrorCode, CssFeatureKind, CssSupportStatus, ErrorKind, feature_metadata,
    parse_style_attribute, property_support_metadata,
};

const CSS_WIDE_KEYWORDS: &[&str] = &["inherit", "initial", "unset", "revert", "revert-layer"];

fn starts_with_css_wide_keyword(authored_value: &str) -> bool {
    authored_value
        .trim_start()
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .next()
        .is_some_and(|first| {
            CSS_WIDE_KEYWORDS
                .iter()
                .any(|keyword| first.eq_ignore_ascii_case(keyword))
        })
}

fn contains_substitution(authored_value: &str) -> bool {
    authored_value.to_ascii_lowercase().contains("var(")
}

#[test]
fn public_feature_catalog_exposes_declared_metadata_and_lookup() {
    for case in CASES {
        let Some(CatalogExpectation::Grammar {
            feature_id,
            production,
            ..
        }) = &case.catalog
        else {
            continue;
        };
        let metadata = property_support_metadata(case.name)
            .unwrap_or_else(|| panic!("missing metadata for `{}`", case.name));
        let feature = metadata.feature();
        assert_eq!(metadata.property(), case.property);
        assert_eq!(feature.id().as_str(), *feature_id);
        assert_eq!(feature.kind(), CssFeatureKind::Property);
        assert_eq!(feature.spelling(), case.name);
        if let Some(production) = production {
            assert_eq!(
                feature.production(),
                *production,
                "{} exact production",
                case.name
            );
        } else {
            assert!(
                feature.production().contains("#propdef-"),
                "{} exact property production",
                feature_id
            );
        }
        assert_eq!(feature.recognized_unsupported_code(), None);
        assert_ne!(
            feature.source().id().as_str(),
            "I01-BASE-PARSER",
            "{} retained generic baseline provenance",
            feature_id
        );
        assert_ne!(
            feature.source().url().is_some(),
            feature.source().repository_provenance().is_some(),
            "{} source provenance XOR",
            feature_id
        );
        assert_eq!(metadata.property().canonical_name(), case.name);
        assert_eq!(metadata.canonical_name(), case.name);
        assert_eq!(metadata.aliases(), case.aliases);
        assert_eq!(metadata.property().aliases(), case.aliases);
        if !case.aliases.is_empty() {
            assert_eq!(
                feature.source().id().as_str(),
                case.source_id.expect("alias provenance expectation")
            );
            assert_eq!(feature.status(), CssSupportStatus::Complete);
            assert_eq!(feature.supported_subset(), None);
            assert_eq!(feature.unsupported_remainder(), None);
        }
        assert!(std::ptr::eq(
            feature,
            feature_metadata(feature_id).expect("exact feature lookup")
        ));
        for alias in case.aliases {
            let alias_metadata = property_support_metadata(alias)
                .unwrap_or_else(|| panic!("missing alias metadata for `{alias}`"));
            assert!(std::ptr::eq(alias_metadata.feature(), feature));
            let folded = alias.to_ascii_uppercase();
            assert_eq!(
                property_support_metadata(&folded).map(|entry| entry.property()),
                Some(case.property)
            );
        }
        let folded = case.name.to_ascii_uppercase();
        assert_eq!(
            property_support_metadata(&folded).map(|entry| entry.property()),
            Some(case.property)
        );
    }
    for name in [
        "--display",
        "--custom",
        "definitely-unknown",
        "",
        " display",
    ] {
        assert!(
            property_support_metadata(name).is_none(),
            "unexpected metadata for `{name}`"
        );
    }
    assert!(feature_metadata("BASELINE.PROPERTY.DISPLAY").is_none());
    for case in CASES {
        if let Some(source_id) = case.source_id {
            let metadata = property_support_metadata(case.name)
                .expect("independently expected property provenance");
            assert_eq!(metadata.property(), case.property);
            assert_eq!(
                metadata.feature().source().id().as_str(),
                source_id,
                "{} provenance",
                case.name
            );
        }
    }
}

#[test]
fn authored_property_cases_exercise_public_parser_behavior() {
    for case in CASES {
        let Some(CatalogExpectation::Grammar {
            feature_id,
            positive,
            boundary,
            ..
        }) = &case.catalog
        else {
            continue;
        };
        if case.name == "all" {
            assert!(
                CSS_WIDE_KEYWORDS
                    .iter()
                    .any(|keyword| positive.trim().eq_ignore_ascii_case(keyword))
                    || contains_substitution(positive),
                "`all` positive must use its valid global/substitution contract"
            );
        } else {
            assert!(
                !starts_with_css_wide_keyword(positive),
                "{} positive must use an ordinary value",
                feature_id
            );
            assert!(
                !contains_substitution(positive),
                "{} positive must reach property-specific dispatch",
                feature_id
            );
        }
        let report = parse_style_attribute(&format!("{}: {}", case.name, positive));
        assert!(
            report.is_clean(),
            "{} positive diagnostics: {:?}",
            feature_id,
            report.diagnostics()
        );
        let [declaration] = report.syntax().as_slice() else {
            panic!(
                "{} positive must retain exactly one declaration",
                feature_id
            );
        };
        let known = declaration
            .known()
            .expect("ordinary known property declaration");
        assert_eq!(known.property(), case.property);
        assert_eq!(known.property().canonical_name(), case.name);
        assert_eq!(known.property().stable_id(), *feature_id);

        let negative = boundary.authored;
        if case.name == "all" {
            assert_eq!(negative, "block");
        } else {
            assert!(
                !negative.trim_end().ends_with('/'),
                "{} negative must use a property-specific rejection, not shared trailing syntax",
                feature_id
            );
        }
        assert!(
            !starts_with_css_wide_keyword(negative),
            "{} negative must reach property-specific dispatch",
            feature_id
        );
        assert!(
            !contains_substitution(negative),
            "{} negative must not use substitution-dependent parsing",
            feature_id
        );
        let report = parse_style_attribute(&format!("{}: {}", case.name, negative));
        if let BoundaryOutcome::Accepted(assertion) = boundary.outcome {
            // The three archived overflow stimuli remain provenance-bearing
            // cases whose current accepted outcome is explicit in their records.
            assert_eq!(negative, "auto");
            assert!(report.is_clean(), "{} current auto", feature_id);
            let [declaration] = report.syntax().as_slice() else {
                panic!("{} must retain one declaration", feature_id);
            };
            let known = declaration
                .known()
                .expect("known accepted boundary declaration");
            assert_eq!(known.property(), case.property);
            assert_eq!(known.property().stable_id(), *feature_id);
            assertion(known);
            continue;
        }
        assert!(
            report.syntax().is_empty(),
            "{} negative vector was retained",
            feature_id
        );
        let diagnostics = report.diagnostics();
        assert_eq!(diagnostics.len(), 1, "{} negative diagnostics", feature_id);
        let error = diagnostics[0].error();
        assert_eq!(
            error.code(),
            CssErrorCode::InvalidPropertyValue,
            "{} negative diagnostic root",
            feature_id
        );
        let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
            panic!("{} negative returned {error:?}", feature_id);
        };
        assert_eq!(detail.property(), case.property);
        assert_eq!(detail.property().canonical_name(), case.name);
        assert_eq!(detail.property().stable_id(), *feature_id);
    }
}

#[test]
fn added_fonts_property_rows_expose_complete_authored_metadata() {
    for case in CASES {
        let Some(CatalogExpectation::Complete {
            feature_id,
            authored,
            production,
        }) = &case.catalog
        else {
            continue;
        };
        let report = parse_style_attribute(&format!("{}: {}", case.name, authored));
        assert!(
            report.is_clean(),
            "{}: {:?}",
            feature_id,
            report.diagnostics()
        );
        let [declaration] = report.syntax().as_slice() else {
            panic!("{}: expected one retained declaration", feature_id);
        };
        let known = declaration.known().expect("known Fonts declaration");
        assert_eq!(known.property(), case.property);
        assert_eq!(known.property().stable_id(), *feature_id);
        let metadata =
            property_support_metadata(case.name).unwrap_or_else(|| panic!("missing {feature_id}"));
        assert_eq!(metadata.feature().id().as_str(), *feature_id);
        assert_eq!(
            metadata.feature().source().id().as_str(),
            case.source_id.expect("complete property provenance")
        );
        assert_eq!(metadata.feature().status(), CssSupportStatus::Complete);
        assert_eq!(metadata.feature().supported_subset(), None);
        assert_eq!(metadata.feature().unsupported_remainder(), None);
        assert_eq!(metadata.feature().production(), *production);
        assert_eq!(metadata.canonical_name(), case.name);
        assert_eq!(
            feature_metadata(feature_id).map(|feature| feature.production()),
            Some(*production)
        );
    }
}
