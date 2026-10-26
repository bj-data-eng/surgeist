#![forbid(unsafe_code)]

//! A separate-crate demonstration of checked, owned CSS component values.
//!
//! Run with `cargo run --offline --locked -p surgeist-css --example component_value_contract`.
//! The accompanying integration test checks that this public consumer compiles
//! and that its independently expected token and provenance contracts hold.

use surgeist_css::{
    CssBlockKind, CssComponentValue, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssNumericTokenKind, CssParser, CssParserContext, CssSerializedOrigin,
    CssValueOrigin, CssValueTokenRef,
};

fn identifier(value: &CssComponentValue) -> &str {
    match value.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => name,
        other => panic!("expected an identifier, received {other:?}"),
    }
}

fn token_boundaries() {
    let number = CssComponentValue::try_number("10").expect("number token");
    let unit = CssComponentValue::try_ident("px").expect("identifier token");
    let values = CssComponentValues::try_new(vec![number, unit], Default::default())
        .expect("two separate tokens");
    let serialized = values
        .serialize_tokens(Default::default())
        .expect("serializable tokens");
    assert_eq!(serialized.as_css(), "10/**/px");
    assert!(matches!(
        serialized.origin_at(2),
        Some(CssSerializedOrigin::Separator {
            before: CssValueOrigin::Programmatic,
            after: CssValueOrigin::Programmatic
        })
    ));
    let reparsed = CssComponentValues::parse(&CssParser::default(), serialized.as_css())
        .expect("token-preserving output");
    assert!(matches!(reparsed.items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Number(number))
            if number.representation() == "10" && number.kind() == CssNumericTokenKind::Integer));
    assert_eq!(
        identifier(reparsed.items().last().expect("identifier after separator")),
        "px"
    );

    for (left, right, expected) in [
        ("foo", "bar", "foo/**/bar"),
        ("foo", "()", "foo/**/()"),
        ("10", "%", "10/**/%"),
        ("/", "*", "//**/*"),
    ] {
        let mut items = CssComponentValues::parse(&CssParser::default(), left)
            .expect("left component")
            .items()
            .to_vec();
        items.extend_from_slice(
            CssComponentValues::parse(&CssParser::default(), right)
                .expect("right component")
                .items(),
        );
        let result = CssComponentValues::try_new(items, Default::default())
            .expect("valid joined components")
            .serialize_tokens(Default::default())
            .expect("separated serialization");
        assert_eq!(result.as_css(), expected);
    }

    // The empty replacement contributes no token and must not reset separation.
    let mut items = CssComponentValues::parse(&CssParser::default(), "a")
        .expect("left identifier")
        .items()
        .to_vec();
    items.extend_from_slice(
        CssComponentValues::parse(&CssParser::default(), "")
            .expect("empty stream")
            .items(),
    );
    items.extend_from_slice(
        CssComponentValues::parse(&CssParser::default(), "b")
            .expect("right identifier")
            .items(),
    );
    assert_eq!(
        CssComponentValues::try_new(items, Default::default())
            .expect("joined stream")
            .serialize_tokens(Default::default())
            .expect("serialization")
            .as_css(),
        "a/**/b"
    );
    println!("token boundaries: ok");
}

fn checked_construction() {
    let escaped =
        CssComponentValues::parse(&CssParser::default(), r"\31 a").expect("escaped identifier");
    assert_eq!(identifier(&escaped.items()[0]), "1a");
    assert_eq!(
        escaped
            .serialize_tokens(Default::default())
            .expect("preserved spelling")
            .as_css(),
        r"\31 a"
    );
    let constructed = CssComponentValue::try_ident("1a").expect("escaped construction");
    assert_eq!(identifier(&constructed), "1a");
    assert!(matches!(constructed.origin(), CssValueOrigin::Programmatic));
    let empty_identifier = CssComponentValue::try_ident("").expect_err("empty identifier");
    assert!(
        empty_identifier.is_invalid_identifier(),
        "{empty_identifier:?}"
    );
    assert!(matches!(
        empty_identifier.origin(),
        CssValueOrigin::Programmatic
    ));
    assert!(
        CssComponentValue::try_ident("a\0b")
            .expect_err("NUL changes identity")
            .is_invalid_identifier()
    );
    assert!(
        CssComponentValue::try_number("10px")
            .expect_err("dimension is not a number")
            .is_invalid_number()
    );

    for (spelling, kind, signed) in [
        ("+0010", CssNumericTokenKind::Integer, true),
        ("-0.0", CssNumericTokenKind::Number, true),
        ("1e999", CssNumericTokenKind::Number, false),
        (
            "123456789012345678901234567890",
            CssNumericTokenKind::Integer,
            false,
        ),
    ] {
        let value = CssComponentValue::try_number(spelling).expect("syntactic number");
        match value.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
                assert_eq!(number.representation(), spelling);
                assert_eq!(number.kind(), kind);
                assert_eq!(number.has_sign(), signed);
            }
            other => panic!("number constructor returned {other:?}"),
        }
        assert_eq!(
            CssComponentValues::try_new(vec![value], Default::default())
                .expect("number stream")
                .serialize_tokens(Default::default())
                .expect("numeric spelling preserved")
                .as_css(),
            spelling
        );
    }

    let dimension = CssComponentValue::try_dimension("100", "E1m").expect("escaped unit");
    let css = CssComponentValues::try_new(vec![dimension], Default::default())
        .expect("dimension stream")
        .serialize_tokens(Default::default())
        .expect("unambiguous dimension");
    let roundtrip =
        CssComponentValues::parse(&CssParser::default(), css.as_css()).expect("dimension reparse");
    assert!(matches!(roundtrip.items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
            if number.representation() == "100" && unit == "E1m"));

    let string = CssComponentValue::try_string("a\"\\\nb").expect("escaped string");
    assert!(
        matches!(string.view(), CssComponentValueRef::Token(CssValueTokenRef::String(value))
        if value == "a\"\\\nb")
    );
    let url = CssComponentValue::try_url("a b)c").expect("escaped URL token");
    assert!(
        matches!(url.view(), CssComponentValueRef::Token(CssValueTokenRef::Url(value))
        if value == "a b)c")
    );
    let quoted_url = CssComponentValue::try_function(
        "url",
        CssComponentValues::try_new(
            vec![CssComponentValue::try_string("x").expect("URL string")],
            Default::default(),
        )
        .expect("URL arguments"),
    )
    .expect("quoted URL function");
    assert!(
        matches!(quoted_url.view(), CssComponentValueRef::Function(function)
        if function.name() == "url")
    );
    let invalid_url = CssComponentValue::try_function(
        "url",
        CssComponentValues::parse(&CssParser::default(), "x").expect("identifier argument"),
    );
    assert!(
        invalid_url
            .expect_err("function must not turn into URL token")
            .is_invalid_function()
    );

    assert!(
        CssComponentValues::parse(&CssParser::default(), "\"a\nb")
            .expect_err("bad string token")
            .is_bad_string()
    );
    assert!(
        CssComponentValues::parse(&CssParser::default(), "url(a b)")
            .expect_err("bad URL token")
            .is_bad_url()
    );
    assert!(
        CssComponentValues::parse(&CssParser::default(), "]")
            .expect_err("unmatched closer")
            .is_unmatched_closing_delimiter()
    );
    println!("checked construction: ok");
}

fn whitespace_and_closures() {
    for css in ["calc(1 + 2)", "calc(1/**/+/**/2)", "calc(1 +2)", "\\\na"] {
        assert_eq!(
            CssComponentValues::parse(&CssParser::default(), css)
                .expect("component syntax")
                .serialize_tokens(Default::default())
                .expect("representation-preserving serialization")
                .as_css(),
            css
        );
    }
    let values = CssComponentValues::parse(&CssParser::default(), "calc(1 + 2")
        .expect("EOF closes the function");
    let serialized = values
        .serialize_tokens(Default::default())
        .expect("explicit closing serialization");
    assert_eq!(serialized.as_css(), "calc(1 + 2)");
    assert!(matches!(serialized.origin_at(10),
        Some(CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { at, .. }))
            if at.span().start().byte_offset().value() == 10));
    assert!(matches!(
        serialized.origin_at(serialized.as_css().len()),
        Some(CssSerializedOrigin::End(_))
    ));
    println!("whitespace and closures: ok");
}

fn origins_and_limits() {
    let source_a =
        CssComponentValues::parse(&CssParser::default(), "10").expect("replacement source");
    let source_b = CssComponentValues::parse(&CssParser::default(), "px").expect("use-site source");
    let joined = CssComponentValues::try_new(
        vec![source_a.items()[0].clone(), source_b.items()[0].clone()],
        Default::default(),
    )
    .expect("independent source tokens");
    let serialized = joined
        .serialize_tokens(Default::default())
        .expect("mapped serialization");
    for (offset, expected_source, expected_end) in [(0, "10", 2), (6, "px", 2)] {
        match serialized.origin_at(offset).expect("mapped token") {
            CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) => {
                assert_eq!(origin.source().as_str(), expected_source);
                assert_eq!(origin.span().start().byte_offset().value(), 0);
                assert_eq!(origin.span().end().byte_offset().value(), expected_end);
            }
            other => panic!("expected original parsed token origin, received {other:?}"),
        }
    }
    let source = "😀\r\npx";
    let unicode = CssComponentValues::parse(&CssParser::default(), source).expect("UTF-8 source");
    match unicode.items().last().expect("last identifier").origin() {
        CssValueOrigin::Parsed(origin) => {
            assert_eq!(origin.span().start().byte_offset().value(), 6);
            assert_eq!(origin.span().start().line().value(), 1);
            assert_eq!(origin.span().start().column().value(), 0);
        }
        other => panic!("expected parsed source origin, received {other:?}"),
    }

    let exact = CssComponentValueLimits::try_new(1, 3, 3).expect("valid limits");
    assert!(
        CssComponentValues::parse(&CssParser::new(CssParserContext::default(), exact), "a b")
            .is_ok()
    );
    assert!(
        CssComponentValues::parse(&CssParser::new(CssParserContext::default(), exact), "abcd")
            .expect_err("input byte limit")
            .is_byte_limit()
    );
    let fewer = CssComponentValueLimits::try_new(1, 2, 3).expect("valid limits");
    assert!(
        CssComponentValues::parse(&CssParser::new(CssParserContext::default(), fewer), "a b")
            .expect_err("component limit")
            .is_component_limit()
    );
    assert!(
        joined
            .serialize_tokens(
                CssComponentValueLimits::try_new(256, usize::MAX, 7).expect("valid limits")
            )
            .expect_err("separator counts toward bytes")
            .is_byte_limit()
    );
    assert!(CssComponentValueLimits::try_new(257, usize::MAX, usize::MAX).is_none());

    let at_limit = format!("{}x{}", "(".repeat(256), ")".repeat(256));
    assert_eq!(
        CssComponentValues::parse(&CssParser::default(), &at_limit)
            .expect("256 blocks supported")
            .nesting_depth(),
        256
    );
    let excessive = format!("({at_limit})");
    assert!(
        CssComponentValues::parse(&CssParser::default(), &excessive)
            .expect_err("257 blocks rejected")
            .is_nesting_limit()
    );
    let mut constructed = CssComponentValues::parse(&CssParser::default(), "x").expect("leaf");
    for _ in 0..256 {
        constructed = CssComponentValues::try_new(
            vec![
                CssComponentValue::try_block(CssBlockKind::Parenthesis, constructed)
                    .expect("checked block"),
            ],
            Default::default(),
        )
        .expect("checked stream");
    }
    assert!(
        CssComponentValue::try_block(CssBlockKind::Parenthesis, constructed)
            .expect_err("constructed nesting obeys the same ceiling")
            .is_nesting_limit()
    );
    println!("origins and limits: ok");
}

fn main() {
    token_boundaries();
    checked_construction();
    whitespace_and_closures();
    origins_and_limits();
}
