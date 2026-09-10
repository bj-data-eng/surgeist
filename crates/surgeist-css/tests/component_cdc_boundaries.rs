#![forbid(unsafe_code)]

//! Component composition promises to preserve token identity and lexical numbers.
//! The leading `--` in a CDC must not be consumed by its preceding token.

use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssNumericTokenKind, CssSerializedOrigin, CssValueOrigin, CssValueTokenRef,
    parse_component_values,
};

const HAZARDOUS_LEFT_SPELLINGS: [&str; 7] = ["-", "@", "#", "1", "1e3", "+1", "-1"];

fn parsed_pair(left: &str) -> CssComponentValues {
    let mut items = parse_component_values(left).unwrap().items().to_vec();
    items.extend_from_slice(parse_component_values("-->").unwrap().items());
    CssComponentValues::try_new(items).unwrap()
}

fn assert_left_token(value: &CssComponentValue, spelling: &str) {
    match (spelling, value.view()) {
        ("-", CssComponentValueRef::Token(CssValueTokenRef::Delim('-')))
        | ("@", CssComponentValueRef::Token(CssValueTokenRef::Delim('@')))
        | ("#", CssComponentValueRef::Token(CssValueTokenRef::Delim('#'))) => {}
        (
            "1" | "1e3" | "+1" | "-1",
            CssComponentValueRef::Token(CssValueTokenRef::Number(number)),
        ) => {
            assert_eq!(number.representation(), spelling);
            assert_eq!(
                number.kind(),
                if spelling == "1e3" {
                    CssNumericTokenKind::Number
                } else {
                    CssNumericTokenKind::Integer
                }
            );
            assert_eq!(number.has_sign(), matches!(spelling, "+1" | "-1"));
        }
        (_, actual) => panic!("preserve the independently supplied {spelling:?} token: {actual:?}"),
    }
}

fn assert_pair_survives(values: &CssComponentValues, left: &str) {
    let output = values.serialize().unwrap();
    let reparsed = parse_component_values(output.as_css()).unwrap();
    assert_left_token(&reparsed.items()[0], left);
    let [_, separator, cdc] = reparsed.items() else {
        panic!("retain the left token, separating comment, and CDC: {output:?}");
    };
    assert!(matches!(
        separator.view(),
        CssComponentValueRef::Comment("")
    ));
    assert!(matches!(
        cdc.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Cdc)
    ));
    assert_eq!(output.as_css(), format!("{left}/**/-->"));
}

#[test]
fn cdc_remains_separate_after_a_minus_delimiter() {
    assert_pair_survives(&parsed_pair("-"), "-");
}

#[test]
fn cdc_remains_separate_after_an_at_delimiter() {
    assert_pair_survives(&parsed_pair("@"), "@");
}

#[test]
fn cdc_remains_separate_after_a_hash_delimiter() {
    assert_pair_survives(&parsed_pair("#"), "#");
}

#[test]
fn cdc_remains_separate_after_numbers_with_each_lexical_form() {
    for spelling in ["1", "1e3", "+1", "-1"] {
        assert_pair_survives(&parsed_pair(spelling), spelling);
    }
}

#[test]
fn checked_rust_tokens_preserve_the_same_cdc_boundaries() {
    for spelling in HAZARDOUS_LEFT_SPELLINGS {
        let values = CssComponentValues::try_new(vec![
            CssComponentValue::try_token(spelling).unwrap(),
            CssComponentValue::try_token("-->").unwrap(),
        ])
        .unwrap();
        assert_pair_survives(&values, spelling);
        let output = values.serialize().unwrap();
        assert!(matches!(
            output.origin_at(spelling.len()),
            Some(CssSerializedOrigin::Separator {
                before: CssValueOrigin::Programmatic,
                after: CssValueOrigin::Programmatic,
            })
        ));
    }
}

#[test]
fn inserted_cdc_separators_map_both_original_parse_inputs() {
    for spelling in HAZARDOUS_LEFT_SPELLINGS {
        let values = parsed_pair(spelling);
        let before = values.items()[0].origin();
        let after = values.items()[1].origin();
        let output = values.serialize().unwrap();
        for offset in 0..spelling.len() {
            assert!(matches!(output.origin_at(offset),
                Some(CssSerializedOrigin::Token(origin)) if origin == before));
        }
        for offset in spelling.len()..spelling.len() + 4 {
            assert!(matches!(output.origin_at(offset),
                Some(CssSerializedOrigin::Separator { before: actual_before, after: actual_after })
                    if actual_before == before && actual_after == after));
        }
        for offset in spelling.len() + 4..spelling.len() + 7 {
            assert!(matches!(output.origin_at(offset),
                Some(CssSerializedOrigin::Token(origin)) if origin == after));
        }
        assert!(matches!(output.origin_at(spelling.len() + 7),
            Some(CssSerializedOrigin::End(Some(origin))) if origin == after));
    }
}

#[test]
fn cdc_separator_bytes_count_in_construction_and_serialization_limits() {
    for spelling in HAZARDOUS_LEFT_SPELLINGS {
        let values = parsed_pair(spelling);
        let required_bytes = spelling.len() + 4 + 3;
        for limit in [spelling.len(), spelling.len() + 3, required_bytes - 1] {
            let error = values.serialize_with_limit(limit).unwrap_err();
            assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
            assert_eq!(error.origin(), values.items()[1].origin());
            let error = CssComponentValues::try_new_with_limits(
                values.items().to_vec(),
                CssComponentValueLimits::try_new(0, 2, limit).unwrap(),
            )
            .unwrap_err();
            assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
            assert_eq!(error.origin(), values.items()[1].origin());
        }
        let exact = CssComponentValues::try_new_with_limits(
            values.items().to_vec(),
            CssComponentValueLimits::try_new(0, 2, required_bytes).unwrap(),
        )
        .unwrap();
        assert_eq!(exact.component_count(), 2);
        assert_eq!(
            exact.serialize_with_limit(required_bytes).unwrap().as_css(),
            format!("{spelling}/**/-->")
        );
    }
}

#[test]
fn harmless_delimiters_before_cdc_do_not_gain_separators() {
    for delimiter in ['+', '.', '/', '%', '<'] {
        let spelling = delimiter.to_string();
        let output = parsed_pair(&spelling).serialize().unwrap();
        assert_eq!(output.as_css(), format!("{delimiter}-->"));
        let reparsed = parse_component_values(output.as_css()).unwrap();
        let [left, right] = reparsed.items() else {
            panic!("retain exactly the delimiter and CDC: {output:?}");
        };
        assert!(matches!(left.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Delim(actual)) if actual == delimiter));
        assert!(matches!(
            right.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Cdc)
        ));
    }
}

#[test]
fn authored_whitespace_and_comments_already_separate_cdc() {
    for source in ["- -->", "-/*kept*/-->", "1 -->", "1/*kept*/-->"] {
        let values = parse_component_values(source).unwrap();
        let output = values.serialize().unwrap();
        assert_eq!(output.as_css(), source);
        let reparsed = parse_component_values(output.as_css()).unwrap();
        let [left, separator, right] = reparsed.items() else {
            panic!("retain the original three components: {output:?}");
        };
        assert_left_token(left, &source[..1]);
        assert!(matches!(
            separator.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace(" "))
                | CssComponentValueRef::Comment("kept")
        ));
        assert!(matches!(
            right.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Cdc)
        ));
        assert!(
            output
                .segments()
                .iter()
                .all(|segment| matches!(segment.origin(), CssSerializedOrigin::Token(_)))
        );
    }
}
