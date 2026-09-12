#![forbid(unsafe_code)]
//! Original-token Unicode-range expectations from pinned Syntax 3 section 7.1.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#urange-syntax
use surgeist_css::{
    CssFontFaceDescriptorKind, CssFontFaceDescriptorValue, CssRecoveryAction, ErrorKind,
    parse_font_face_descriptor_value,
};

type RangeExpectation<'a> = (&'a str, &'a [(u32, u32)], bool);

#[test]
fn raw_unicode_range_corpus_preserves_values_and_original_diagnostics() {
    let expectations: &[RangeExpectation<'_>] = &[
        (
            "value/UnicodeRange.json#/a number ends with a minus",
            &[],
            false,
        ),
        (
            "value/UnicodeRange.json#/a number with a negative exponent",
            &[(302, 304)],
            false,
        ),
        (
            "value/UnicodeRange.json#/a single question mark",
            &[(0, 15)],
            false,
        ),
        ("value/UnicodeRange.json#/errors/0", &[], true),
        ("value/UnicodeRange.json#/errors/1", &[], false),
        ("value/UnicodeRange.json#/errors/10", &[], false),
        ("value/UnicodeRange.json#/errors/11", &[], false),
        ("value/UnicodeRange.json#/errors/12", &[], false),
        ("value/UnicodeRange.json#/errors/13", &[], false),
        ("value/UnicodeRange.json#/errors/14", &[], false),
        ("value/UnicodeRange.json#/errors/15", &[], false),
        ("value/UnicodeRange.json#/errors/16", &[], false),
        ("value/UnicodeRange.json#/errors/17", &[], false),
        ("value/UnicodeRange.json#/errors/2", &[], false),
        ("value/UnicodeRange.json#/errors/3", &[], false),
        ("value/UnicodeRange.json#/errors/4", &[], false),
        ("value/UnicodeRange.json#/errors/5", &[], false),
        ("value/UnicodeRange.json#/errors/6", &[], false),
        ("value/UnicodeRange.json#/errors/7", &[], false),
        ("value/UnicodeRange.json#/errors/8", &[], false),
        ("value/UnicodeRange.json#/errors/9", &[], false),
        ("value/UnicodeRange.json#/not an unicode range", &[], true),
        (
            "value/UnicodeRange.json#/unicode range hex pair",
            &[(3840, 4095)],
            false,
        ),
        (
            "value/UnicodeRange.json#/unicode range hex pair #2",
            &[(37, 255)],
            false,
        ),
        (
            "value/UnicodeRange.json#/unicode range hex pair number~1letters",
            &[(4660, 43981)],
            false,
        ),
        (
            "value/UnicodeRange.json#/unicode range hex pair only numbers",
            &[(4660, 9029)],
            false,
        ),
        (
            "value/UnicodeRange.json#/unicode range hex pair starts with letters",
            &[(65280, 65296)],
            false,
        ),
        (
            "value/UnicodeRange.json#/unicode range hex with ?",
            &[(983040, 983295)],
            false,
        ),
        (
            "value/UnicodeRange.json#/unicode range one hex",
            &[(3840, 3840)],
            false,
        ),
        (
            "value/UnicodeRange.json#/unicode range short hex pair",
            &[(0, 127)],
            false,
        ),
    ];
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "corpus/csstree/expectations/value/UnicodeRange.json"
    ))
    .unwrap();
    let classes: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    let cases = source["cases"].as_array().unwrap();
    assert_eq!(cases.len(), expectations.len());
    for (id, ranges, eof) in expectations {
        let matching: Vec<_> = cases.iter().filter(|case| case["id"] == *id).collect();
        assert_eq!(matching.len(), 1, "one original input for {id}");
        let input = matching[0]["input"].as_str().unwrap();
        let report =
            parse_font_face_descriptor_value(input, CssFontFaceDescriptorKind::UnicodeRange);
        let clean = !ranges.is_empty();
        assert_eq!(report.is_clean(), clean, "{id}: {report:?}");
        let mut expected_class = serde_json::json!({
            "kind": if clean {"clean"} else {"strict_rejected"},
            "retained_syntax": {
                "extractor": {"kind": "font_face_descriptor", "descriptor": "unicode_range"},
                "predicate": {"relation": if clean {"nonempty"} else {"empty"}}
            }
        });
        if clean {
            let Some(CssFontFaceDescriptorValue::UnicodeRange(value)) = report.syntax() else {
                panic!("retained typed range for {id}: {report:?}");
            };
            let actual: Vec<_> = value
                .ranges()
                .iter()
                .map(|r| (r.start(), r.end()))
                .collect();
            assert_eq!(actual.as_slice(), *ranges, "{id}");
        } else {
            assert!(report.syntax().is_none(), "{id}");
            let [diagnostic] = report.diagnostics() else {
                panic!("one atomic value rejection for {id}: {report:?}");
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput, "{id}");
            assert_eq!(diagnostic.span().start().byte_offset().value(), 0, "{id}");
            assert_eq!(
                diagnostic.span().end().byte_offset().value(),
                input.len(),
                "{id}"
            );
            let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
                panic!("descriptor grammar error for {id}");
            };
            if *eof {
                assert_eq!(
                    diagnostic.error().position().byte_offset().value(),
                    input.len(),
                    "{id}"
                );
                assert!(detail.encountered().is_none(), "{id}");
            } else {
                assert!(
                    diagnostic.error().position().byte_offset().value() < input.len(),
                    "{id}"
                );
                assert!(detail.encountered().is_some(), "{id}");
            }
            expected_class["diagnostics"] = serde_json::json!([{
                "code": "invalid_descriptor_value", "action": "reject_input",
                "payload_relation": if *eof {"recovery_ends_at"} else {"intersects"}
            }]);
        }
        let matching: Vec<_> = classes["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"] == *id)
            .collect();
        assert_eq!(matching.len(), 1, "one expected class for {id}");
        assert_eq!(matching[0]["class"], expected_class, "{id}");
    }
}
