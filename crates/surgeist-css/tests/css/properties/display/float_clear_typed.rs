//! Exact authored float/clear values from CSS2 §§9.5.1–2 and Logical 1 §2.2.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::normal_declaration as parsed;
use surgeist_css::*;

fn ordinary(source: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("one completed longhand")
    };
    let [item] = values.items() else {
        panic!("one contribution")
    };
    item.ordinary_value().unwrap().clone()
}

#[test]
fn every_float_keyword_serializes_with_exact_single_keyword_budgets() {
    for (value, text) in [
        (CssFloat::None, "none"),
        (CssFloat::Left, "left"),
        (CssFloat::Right, "right"),
        (CssFloat::InlineStart, "inline-start"),
        (CssFloat::InlineEnd, "inline-end"),
    ] {
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), text);
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, text.len(),))
                .unwrap(),
            text
        );
        for (limits, kind) in [
            (
                CssSerializationLimits::new(0, 1, text.len()),
                SerializationFailure::InputNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 0, text.len()),
                SerializationFailure::ProjectionNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 1, text.len() - 1),
                SerializationFailure::ByteLimit,
            ),
        ] {
            assert_eq!(
                serialization_failure(&value.serialize_specified(limits).unwrap_err()),
                kind
            );
        }
    }
}

#[test]
fn every_clear_keyword_serializes_and_both_expands_as_its_own_value() {
    for (value, text) in [
        (CssClear::None, "none"),
        (CssClear::Left, "left"),
        (CssClear::Right, "right"),
        (CssClear::Both, "both"),
        (CssClear::InlineStart, "inline-start"),
        (CssClear::InlineEnd, "inline-end"),
    ] {
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), text);
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, text.len(),))
                .unwrap(),
            text
        );
        assert!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, text.len() - 1,))
                .unwrap_err()
                .is_byte_limit()
        );
    }
    assert_eq!(
        ordinary(&parsed("clear", "both")).view(),
        CssLonghandValueRef::Clear(&CssClear::Both)
    );
}

#[test]
fn wrappers_keep_physical_and_flow_relative_keywords() {
    for (text, current) in [
        ("none", CssFloat::None),
        ("left", CssFloat::Left),
        ("right", CssFloat::Right),
        ("INLINE-START", CssFloat::InlineStart),
        ("inline-end", CssFloat::InlineEnd),
    ] {
        let source = parsed("float", text);
        let Some(CssKnownPropertyValueRef::Float(value)) = source.known().unwrap().property_value()
        else {
            panic!("float wrapper")
        };
        assert_eq!(*value.value(), current);
        assert_eq!(value.as_css(), text);
        assert_eq!(
            ordinary(&source).view(),
            CssLonghandValueRef::Float(&current)
        );
    }
    for (text, current) in [
        ("none", CssClear::None),
        ("left", CssClear::Left),
        ("right", CssClear::Right),
        ("both", CssClear::Both),
        ("inline-start", CssClear::InlineStart),
        ("INLINE-END", CssClear::InlineEnd),
    ] {
        let source = parsed("clear", text);
        let Some(CssKnownPropertyValueRef::Clear(value)) = source.known().unwrap().property_value()
        else {
            panic!("clear wrapper")
        };
        assert_eq!(*value.value(), current);
        assert_eq!(value.as_css(), text);
        assert_eq!(
            ordinary(&source).view(),
            CssLonghandValueRef::Clear(&current)
        );
    }
}

#[test]
fn substituted_inline_end_reenters_to_exact_typed_keyword() {
    for name in ["float", "clear"] {
        let source = parsed(name, "var(--side)");
        let CssExpansion::Pending(pending) = source.expand().unwrap() else {
            panic!("pending substitution")
        };
        let replacement = CssComponentValues::parse(&CssParser::default(), "inline-end").unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("reentered longhand")
        };
        let [item] = values.items() else {
            panic!("one contribution")
        };
        let actual = item.ordinary_value().unwrap().view();
        match name {
            "float" => assert_eq!(actual, CssLonghandValueRef::Float(&CssFloat::InlineEnd)),
            "clear" => assert_eq!(actual, CssLonghandValueRef::Clear(&CssClear::InlineEnd)),
            _ => unreachable!(),
        }
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}
