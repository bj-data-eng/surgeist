#![forbid(unsafe_code)]

//! Authored Overflow 3 keywords and the Cascade 5 overlay alias.

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

fn ordinary(source: &CssDeclaration) -> Vec<CssLonghandValue> {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed overflow contributions")
    };
    values
        .items()
        .iter()
        .map(|item| item.ordinary_value().unwrap().clone())
        .collect()
}

#[test]
fn every_keyword_serializes_canonically_with_one_node_and_exact_bytes() {
    for (value, text) in [
        (CssOverflow::Visible, "visible"),
        (CssOverflow::Hidden, "hidden"),
        (CssOverflow::Clip, "clip"),
        (CssOverflow::Scroll, "scroll"),
        (CssOverflow::Auto, "auto"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), text);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len()
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
fn shorthand_retains_authored_arity_and_charges_outer_and_child_nodes() {
    let single = CssOverflowValue::new(CssOverflow::Clip, None);
    assert_eq!(
        (single.x(), single.authored_y(), single.y()),
        (CssOverflow::Clip, None, CssOverflow::Clip)
    );
    assert_eq!(single.serialize_specified().unwrap(), "clip");
    assert_eq!(
        single
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 4))
            .unwrap(),
        "clip"
    );
    let pair = CssOverflowValue::new(CssOverflow::Visible, Some(CssOverflow::Auto));
    assert_eq!(
        (pair.x(), pair.authored_y(), pair.y()),
        (
            CssOverflow::Visible,
            Some(CssOverflow::Auto),
            CssOverflow::Auto
        )
    );
    assert_ne!(
        CssOverflowValue::new(CssOverflow::Visible, None),
        CssOverflowValue::new(CssOverflow::Visible, Some(CssOverflow::Visible))
    );
    assert_eq!(pair.serialize_specified().unwrap(), "visible auto");
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 12))
            .unwrap(),
        "visible auto"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 12),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 12),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 11),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            pair.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}

#[test]
fn parsed_aliases_and_wrappers_keep_semantics_separate_from_authored_spelling() {
    for name in [
        "overflow-x",
        "overflow-y",
        "overflow-block",
        "overflow-inline",
    ] {
        for text in ["auto", "overlay", "OVERLAY", "ov\\65 rlay"] {
            let source = parsed(name, text);
            let value = source.known().unwrap().property_value().unwrap();
            match (name, value) {
                ("overflow-x", CssKnownPropertyValueRef::OverflowX(value)) => {
                    assert_eq!(*value.current(), CssOverflow::Auto);
                    assert!(value.i01_subset().is_none());
                    assert_eq!(value.as_css(), text);
                }
                ("overflow-y", CssKnownPropertyValueRef::OverflowY(value)) => {
                    assert_eq!(*value.current(), CssOverflow::Auto);
                    assert!(value.i01_subset().is_none());
                    assert_eq!(value.as_css(), text);
                }
                ("overflow-block", CssKnownPropertyValueRef::OverflowBlock(value)) => {
                    assert_eq!(*value.current(), CssOverflow::Auto);
                    assert_eq!(value.as_css(), text);
                }
                ("overflow-inline", CssKnownPropertyValueRef::OverflowInline(value)) => {
                    assert_eq!(*value.current(), CssOverflow::Auto);
                    assert_eq!(value.as_css(), text);
                }
                other => panic!("matching overflow wrapper: {other:?}"),
            }
            let contribution_values = ordinary(&source);
            let [ordinary] = contribution_values.as_slice() else {
                panic!("one overflow contribution")
            };
            match ordinary.view() {
                CssLonghandValueRef::OverflowX(value)
                | CssLonghandValueRef::OverflowY(value)
                | CssLonghandValueRef::OverflowBlock(value)
                | CssLonghandValueRef::OverflowInline(value) => {
                    assert_eq!(*value, CssOverflow::Auto)
                }
                other => panic!("typed overflow contribution: {other:?}"),
            }
        }
    }
    for text in ["overlay", "OVERLAY", "ov\\65 rlay"] {
        let source = parsed("overflow", text);
        let Some(CssKnownPropertyValueRef::Overflow(value)) =
            source.known().unwrap().property_value()
        else {
            panic!("overflow shorthand wrapper")
        };
        assert_eq!(
            (
                value.current().x(),
                value.current().authored_y(),
                value.current().y()
            ),
            (CssOverflow::Auto, None, CssOverflow::Auto)
        );
        assert!(value.i01_subset().is_none());
        assert_eq!(value.as_css(), text);
        assert_eq!(value.current().serialize_specified().unwrap(), "auto");
    }
}

#[test]
fn exact_axes_project_only_original_i01_values_and_reenter_to_auto() {
    for (text, expected) in [
        (
            "clip",
            CssOverflowI01PropertyValue::Single(CssOverflow::Clip),
        ),
        (
            "hidden scroll",
            CssOverflowI01PropertyValue::Pair(CssOverflowAxes::new(
                CssOverflow::Hidden,
                CssOverflow::Scroll,
            )),
        ),
    ] {
        let source = parsed("overflow", text);
        let Some(CssKnownPropertyValueRef::Overflow(value)) =
            source.known().unwrap().property_value()
        else {
            panic!("overflow shorthand wrapper")
        };
        assert_eq!(value.i01_subset(), Some(&expected));
    }
    let source = parsed("overflow", "visible overlay");
    let Some(CssKnownPropertyValueRef::Overflow(value)) = source.known().unwrap().property_value()
    else {
        panic!("overflow shorthand wrapper")
    };
    assert_eq!(
        (
            value.current().x(),
            value.current().authored_y(),
            value.current().y()
        ),
        (
            CssOverflow::Visible,
            Some(CssOverflow::Auto),
            CssOverflow::Auto
        )
    );
    assert!(value.i01_subset().is_none());
    let contribution_values = ordinary(&source);
    let [x, y] = contribution_values.as_slice() else {
        panic!("two axis contributions")
    };
    assert_eq!(
        x.view(),
        CssLonghandValueRef::OverflowX(&CssOverflow::Visible)
    );
    assert_eq!(y.view(), CssLonghandValueRef::OverflowY(&CssOverflow::Auto));

    let pending = parsed("overflow", "var(--axes)");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending overflow")
    };
    let replacement = parse_component_values("overlay clip").unwrap();
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("reentered axis contributions")
    };
    let [x, y] = values.items() else {
        panic!("two reentered axes")
    };
    assert_eq!(
        x.ordinary_value().unwrap().view(),
        CssLonghandValueRef::OverflowX(&CssOverflow::Auto)
    );
    assert_eq!(
        y.ordinary_value().unwrap().view(),
        CssLonghandValueRef::OverflowY(&CssOverflow::Clip)
    );
    for item in values.items() {
        assert!(item.source().same_occurrence(&pending));
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}
