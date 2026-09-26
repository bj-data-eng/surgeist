#![forbid(unsafe_code)]

//! Exact authored float/clear values from CSS2 §§9.5.1–2 and Logical 1 §2.2.

use surgeist_css::*;

fn parsed(name: &str, value: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{value}"));
    assert!(
        report.is_clean(),
        "{name}:{value}: {:?}",
        report.diagnostics()
    );
    report.syntax()[0].clone()
}

fn ordinary(source: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
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
        assert_eq!(value.serialize_specified().unwrap(), text);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len(),
                ))
                .unwrap(),
            text
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, text.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, text.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, text.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
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
        assert_eq!(value.serialize_specified().unwrap(), text);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len(),
                ))
                .unwrap(),
            text
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len() - 1,
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
    }
    assert_eq!(
        ordinary(&parsed("clear", "both")).view(),
        CssLonghandValueRef::Clear(&CssClear::Both)
    );
}

#[test]
fn wrappers_keep_current_keyword_and_only_project_original_i01_values() {
    for (text, current, legacy) in [
        ("none", CssFloat::None, true),
        ("left", CssFloat::Left, true),
        ("right", CssFloat::Right, true),
        ("INLINE-START", CssFloat::InlineStart, false),
        ("inline-end", CssFloat::InlineEnd, false),
    ] {
        let source = parsed("float", text);
        let Some(CssKnownPropertyValueRef::Float(value)) = source.known().unwrap().property_value()
        else {
            panic!("float wrapper")
        };
        assert_eq!(*value.current(), current);
        assert_eq!(value.i01_subset().is_some(), legacy);
        assert_eq!(value.as_css(), text);
        assert_eq!(
            ordinary(&source).view(),
            CssLonghandValueRef::Float(&current)
        );
    }
    for (text, current, legacy) in [
        ("none", CssClear::None, true),
        ("left", CssClear::Left, true),
        ("right", CssClear::Right, true),
        ("both", CssClear::Both, true),
        ("inline-start", CssClear::InlineStart, false),
        ("INLINE-END", CssClear::InlineEnd, false),
    ] {
        let source = parsed("clear", text);
        let Some(CssKnownPropertyValueRef::Clear(value)) = source.known().unwrap().property_value()
        else {
            panic!("clear wrapper")
        };
        assert_eq!(*value.current(), current);
        assert_eq!(value.i01_subset().is_some(), legacy);
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
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending substitution")
        };
        let replacement = parse_component_values("inline-end").unwrap();
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
