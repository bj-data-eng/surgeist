//! Every single-property `<position>` consumer except `background-position` and
//! `transform-origin` takes the Values 5 grammar, flow-relative keywords included
//! (`references/css-values-5--WD-css-values-5-20240917--9b60eb26cb8f.md`, `<position>`).
//! The keywords stay unresolved in the specified value.

use crate::css::support::sole_declaration;
use surgeist_css::*;

fn clean(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

fn written(css: &str) -> String {
    sole_declaration(css)
        .serialize_specified(Default::default())
        .unwrap()
}

#[test]
fn object_position_and_perspective_origin_accept_flow_relative_keywords() {
    for (authored, expected) in [
        ("object-position: start end", "object-position: start end;"),
        (
            "object-position: x-start y-end",
            "object-position: x-start y-end;",
        ),
        (
            "object-position: block-start inline-end",
            "object-position: block-start inline-end;",
        ),
        (
            "object-position: inline-start",
            "object-position: center inline-start;",
        ),
        (
            "perspective-origin: x-start y-end",
            "perspective-origin: x-start y-end;",
        ),
        (
            "perspective-origin: block-end",
            "perspective-origin: block-end center;",
        ),
    ] {
        assert_eq!(written(authored), expected, "{authored}");
    }
}

#[test]
fn flow_relative_object_position_is_retained_as_its_coordinate_family() {
    let declaration = clean("object-position: block-start inline-end");
    let Some(CssKnownPropertyValueRef::ObjectPosition(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("object-position value");
    };
    let CssPositionRef::NamedFlow(axes) = value.value().view() else {
        panic!("a named-flow position");
    };
    assert_eq!(axes.block(), &CssBlockPosition::Start);
    assert_eq!(axes.inline(), &CssInlinePosition::End);
}

#[test]
fn mask_position_layers_accept_flow_relative_keywords() {
    assert_eq!(
        written("mask-position: block-start inline-end, start end, left top"),
        "mask-position: block-start inline-end, start end, left top;"
    );
    // The `mask` shorthand takes the same position inside a layer.
    assert_eq!(
        written("mask: url(mask.png) x-start y-start / contain no-repeat"),
        "mask: url(\"mask.png\") x-start y-start / contain no-repeat;"
    );
    let declaration = clean("mask-position: start end, left top");
    let Some(CssKnownPropertyValueRef::MaskPosition(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("mask-position value");
    };
    let [relative, physical] = value.value().positions() else {
        panic!("two layers");
    };
    assert!(matches!(relative.view(), CssPositionRef::RelativeFlow(_)));
    assert!(matches!(physical.view(), CssPositionRef::Cartesian(_)));
}

#[test]
fn a_mask_layer_holds_the_full_position() {
    let position =
        CssPosition::try_from_named_axes(CssBlockPosition::End, CssInlinePosition::Start).unwrap();
    let layer =
        CssMaskLayer::try_new(None, Some(position.clone()), None, None, None, None, None).unwrap();
    assert_eq!(layer.position(), Some(&position));
    let list = CssPositionList::try_new(vec![position]).unwrap();
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        "block-end inline-start"
    );
    assert!(CssPositionList::try_new(Vec::new()).is_none());
}

#[test]
fn background_position_and_transform_origin_stay_physical() {
    // `background-position` stays physical until csswg-drafts#9690 settles how its longhands
    // are set; `transform-origin` takes the older physical grammar.
    for rejected in [
        "background-position: x-start y-start",
        "background-position: block-start",
        "transform-origin: x-start top",
        "transform-origin: start end",
    ] {
        let report = parse_style_attribute(rejected);
        assert!(!report.is_clean(), "{rejected}");
        assert!(report.syntax().is_empty(), "{rejected}");
    }
}

#[test]
fn a_physical_position_lifts_into_the_full_position_without_changing_its_axes() {
    let physical =
        CssPhysicalPosition::try_new(CssHorizontalPosition::Right, CssVerticalPosition::Bottom)
            .unwrap();
    let lifted = CssPosition::from(physical.clone());
    let CssPositionRef::Cartesian(view) = lifted.view() else {
        panic!("a Cartesian lift");
    };
    assert_eq!(view.horizontal(), physical.horizontal());
    assert_eq!(view.vertical(), physical.vertical());
}
