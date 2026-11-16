//! Specified serialization leaves out a member that the grammar supplies when
//! it is omitted: CSSOM 1 (WD 2021-08-26) §6.7.2 omits or replaces component
//! values with a shorter representation when the meaning of the value does not
//! change. The omitted value of each case comes from the property's grammar,
//! never from the serializer:
//!
//! - `mask-border`: Masking 1 (CRD 2021-08-05) §8.7 writes the shorthand as
//!   `<'mask-border-source'> || <'mask-border-slice'> [ / <'mask-border-width'>?
//!   [ / <'mask-border-outset'> ]? ]? || <'mask-border-repeat'> ||
//!   <'mask-border-mode'>`, with the initial values of the property index:
//!   source `none`, slice `0`, width `auto`, outset `0`, repeat `stretch` and
//!   mode `alpha`.
//! - `shape-outside`: Shapes 1 (CRD 2025-06-12) §6.1 gives a `<basic-shape>`
//!   without a `<shape-box>` the reference box `margin-box`.

use crate::css::support::sole_declaration as declaration;
use crate::css::support::value_assertions::specified_property_value as read;
use surgeist_css::CssParser;
use surgeist_css::declaration::{CssDeclaration, CssSpecifiedDeclarationBlock};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssKnownProperty, CssKnownPropertyValueRef,
    CssLonghandContributions,
};
use surgeist_css::values::shape::{CssBasicShape, CssShapeBox, CssShapeOutside};

fn specified(property: &str, value: &str) -> String {
    let text = declaration(&format!("{property}: {value}"))
        .serialize_specified(Default::default())
        .unwrap();
    text.strip_prefix(&format!("{property}: "))
        .and_then(|text| text.strip_suffix(';'))
        .unwrap()
        .to_owned()
}

fn border_members(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) = source.expand().unwrap()
    else {
        panic!("complete intrinsic expansion")
    };
    items
}

/// The same six longhand values: omitted members expand to their initial
/// values, so equal expansions are equal meaning.
#[track_caller]
fn assert_same_border_meaning(source: &CssDeclaration, reparsed: &CssDeclaration, note: &str) {
    let (left, right) = (border_members(source), border_members(reparsed));
    assert_eq!(left.items().len(), 6, "{note}");
    for (left, right) in left.items().iter().zip(right.items()) {
        assert_eq!(left.property(), right.property(), "{note}");
        assert_eq!(
            left.ordinary_value().unwrap(),
            right.ordinary_value().unwrap(),
            "{note}: {}",
            left.property().canonical_name()
        );
    }
}

/// Serializes each authored `mask-border`, checks the shortest form, and
/// checks that the serialized form reparses to the same six longhand values and
/// is its own fixed point.
#[track_caller]
fn assert_mask_border_cases(cases: &[(&str, &str)]) {
    for (authored, expected) in cases {
        let source = declaration(&format!("mask-border: {authored}"));
        assert_eq!(
            specified("mask-border", authored),
            *expected,
            "mask-border: {authored}"
        );
        let serialized = source.serialize_specified(Default::default()).unwrap();
        let reparsed = declaration(&serialized);
        assert_same_border_meaning(
            &source,
            &reparsed,
            &format!("{authored} must keep its meaning as `{serialized}`"),
        );
        assert_eq!(
            reparsed.serialize_specified(Default::default()).unwrap(),
            serialized,
            "{authored} must reach a fixed point"
        );
    }
}

#[test]
fn mask_border_omits_the_initial_width_between_slashes_and_the_initial_mode() {
    assert_mask_border_cases(&[
        // The issue's spec form.
        (
            "url(a) 1 2 fill / auto / 2 round alpha",
            "url(\"a\") 1 2 fill / / 2 round",
        ),
        (
            "url(a) 1 2 fill / / 2 round",
            "url(\"a\") 1 2 fill / / 2 round",
        ),
        (
            "url(a) 1 2 fill / / 2 round alpha",
            "url(\"a\") 1 2 fill / / 2 round",
        ),
        ("url(a) 10 / auto", "url(\"a\") 10"),
        ("url(a) 10 / auto auto auto auto", "url(\"a\") 10"),
        ("url(a) 10 /", "url(\"a\") 10"),
    ]);
}

#[test]
fn mask_border_omits_every_member_at_its_initial_value() {
    assert_mask_border_cases(&[
        // All members initial: the grammar needs a token, and `none` is the
        // initial source.
        ("none", "none"),
        ("alpha", "none"),
        ("none alpha", "none"),
        ("stretch", "none"),
        ("stretch stretch", "none"),
        ("0", "none"),
        ("0 0 0 0", "none"),
        ("0.0", "none"),
        ("0 / auto / 0", "none"),
        ("none 0 / auto / 0 stretch alpha", "none"),
        ("none 0.0 / auto / 0.00 stretch stretch alpha", "none"),
        ("alpha stretch 0 / auto / 0 none", "none"),
        // One non-initial member keeps only itself.
        ("url(a)", "url(\"a\")"),
        ("url(a) alpha", "url(\"a\")"),
        ("none 10", "10"),
        ("none luminance", "luminance"),
        ("luminance", "luminance"),
        ("LUMINANCE", "luminance"),
        ("round", "round"),
        ("round stretch", "round stretch"),
        ("stretch round", "stretch round"),
        ("repeat space", "repeat space"),
        ("none 0 / auto / 0 round alpha", "round"),
        ("none 0 / auto / 0 stretch luminance", "luminance"),
    ]);
}

#[test]
fn mask_border_keeps_a_repeat_pair_unless_both_keywords_are_stretch() {
    assert_mask_border_cases(&[
        ("round round", "round"),
        ("space space", "space"),
        ("repeat repeat", "repeat"),
        ("stretch space", "stretch space"),
        ("space stretch", "space stretch"),
        ("url(a) stretch", "url(\"a\")"),
        (
            "url(a) round space luminance",
            "url(\"a\") round space luminance",
        ),
    ]);
}

#[test]
fn mask_border_keeps_the_slice_unless_it_is_the_initial_zero_alone() {
    assert_mask_border_cases(&[
        ("10", "10"),
        ("10 20% fill", "10 20% fill"),
        ("0 fill", "0 fill"),
        ("0 1", "0 1"),
        ("0 0 0 1", "0 0 0 1"),
        // Percentage zero is another literal than the initial `0`.
        ("0%", "0%"),
        ("url(a) 0%", "url(\"a\") 0%"),
        // A following width or outset needs its slice, even at zero.
        ("0 / 2", "0 / 2"),
        ("0 / 2 / 3", "0 / 2 / 3"),
        ("0 / / 3", "0 / / 3"),
        ("0 / auto / 3", "0 / / 3"),
        ("url(a) 0 / 2 round", "url(\"a\") 0 / 2 round"),
    ]);
}

#[test]
fn mask_border_keeps_the_slash_groups_the_outset_or_width_needs() {
    assert_mask_border_cases(&[
        ("10 / 2", "10 / 2"),
        ("10 / 2 / 3", "10 / 2 / 3"),
        ("10 / / 3", "10 / / 3"),
        ("10 / auto / 3", "10 / / 3"),
        // An initial outset leaves its slash out, a retained width keeps its own.
        ("10 / 2 / 0", "10 / 2"),
        ("10 / / 0", "10"),
        ("10 / auto / 0", "10"),
        ("10 / auto 2", "10 / auto 2"),
        ("10 / 2 auto", "10 / 2 auto"),
        ("10 / 1", "10 / 1"),
        ("10 / 5px / 1em", "10 / 5px / 1em"),
        ("10 / 5% / 1.5", "10 / 5% / 1.5"),
        ("10 / auto auto 2 3", "10 / auto auto 2 3"),
        // A zero length is another literal than the initial number `0`.
        ("10 / / 0px", "10 / / 0px"),
        ("10 / 2 / 0 0 0 1", "10 / 2 / 0 0 0 1"),
    ]);
}

#[test]
fn mask_border_keeps_every_member_in_grammar_order_when_none_is_initial() {
    assert_mask_border_cases(&[
        (
            "luminance round space url(frame.png) 1 2 3 4 fill / 2 3 4 5 / 1 2 3 4",
            "url(\"frame.png\") 1 2 3 4 fill / 2 3 4 5 / 1 2 3 4 round space luminance",
        ),
        (
            "url(a) 10 / 2 / 3 repeat luminance",
            "url(\"a\") 10 / 2 / 3 repeat luminance",
        ),
        (
            "linear-gradient(red, blue) 1 fill / auto / 2px round",
            "linear-gradient(red, blue) 1 fill / / 2px round",
        ),
    ]);
}

#[test]
fn mask_border_keeps_symbolic_members_unresolved() {
    // Only a literal initial is omitted; math that happens to be the initial
    // value stays, as it does for `border-image`.
    assert_mask_border_cases(&[("calc(0)", "calc(0)"), ("10 / / calc(0)", "10 / / calc(0)")]);
    // Calculations print their canonical sum order, so only the text is compared.
    for (authored, expected) in [
        ("calc(1 - 1)", "calc(0)"),
        ("10 / / calc(1px - 1px)", "10 / / calc(0px)"),
        ("10 / calc(1px + 2%)", "10 / calc(2% + 1px)"),
        (
            "url(a) calc(0) / calc(1px + 2%) round",
            "url(\"a\") calc(0) / calc(2% + 1px) round",
        ),
    ] {
        assert_eq!(specified("mask-border", authored), expected, "{authored}");
    }
}

#[test]
fn mask_border_shares_border_images_omission_of_the_members_they_have_in_common() {
    // `border-image` already omits its initial members through the same
    // writer; `mask-border` differs only in its own initial slice and width and
    // its mode, so a shared member that is not initial prints the same way.
    for tail in ["1 / 2 / 3 round", "1 fill / / 4", "7 / 8 space round", "1"] {
        let border = specified("border-image", &format!("url(a) {tail}"));
        let mask = specified("mask-border", &format!("url(a) {tail}"));
        assert_eq!(mask, border, "{tail}");
    }
}

#[test]
fn mask_border_declaration_block_inverse_and_authored_value_print_one_text() {
    // The specified declaration block rebuilds the shorthand from its six
    // longhands, which already omit their initials; the authored value now
    // omits them too, so both paths print the same shortest text.
    let block = |source: &str| {
        let report = CssParser::default().parse_block_contents(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax(), Default::default())
            .unwrap()
    };
    for (authored, expected) in [
        (
            "url(a) 1 2 fill / auto / 2 round alpha",
            "url(\"a\") 1 2 fill / / 2 round",
        ),
        (
            "url(a) 1 2 fill / / 2 round",
            "url(\"a\") 1 2 fill / / 2 round",
        ),
        ("none 0 / auto / 0 stretch alpha", "none"),
        ("alpha", "none"),
        ("none 10", "10"),
        ("none 0 / 2", "0 / 2"),
        ("none 0 / auto / 2", "0 / / 2"),
        ("none 0 fill", "0 fill"),
        (
            "luminance round space url(frame.png) 1 2 3 4 fill / 2 3 4 5 / 1 2 3 4",
            "url(\"frame.png\") 1 2 3 4 fill / 2 3 4 5 / 1 2 3 4 round space luminance",
        ),
    ] {
        assert_eq!(
            read(
                &block(&format!("mask-border: {authored}")),
                CssKnownProperty::MaskBorder
            )
            .as_deref(),
            Some(expected),
            "block: {authored}"
        );
        assert_eq!(
            specified("mask-border", authored),
            expected,
            "authored: {authored}"
        );
    }
}

/// The meaning of a parsed `shape-outside`: its shape text and the reference
/// box that Shapes 1 §6.1 gives an omitted `<shape-box>`.
fn outside_meaning(source: &CssDeclaration) -> (Option<String>, Option<&'static str>) {
    let CssKnownPropertyValueRef::ShapeOutside(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("shape-outside wrapper")
    };
    match value.value() {
        CssShapeOutside::BasicShape(shape) => (
            Some(
                shape
                    .shape()
                    .serialize_specified(Default::default())
                    .unwrap(),
            ),
            Some(
                shape
                    .reference_box()
                    .unwrap_or(CssShapeBox::MarginBox)
                    .as_str(),
            ),
        ),
        CssShapeOutside::ShapeBox(shape_box) => (None, Some(shape_box.as_str())),
        other => (
            Some(other.serialize_specified(Default::default()).unwrap()),
            None,
        ),
    }
}

#[track_caller]
fn assert_outside_cases(cases: &[(&str, &str)]) {
    for (authored, expected) in cases {
        assert_eq!(
            specified("shape-outside", authored),
            *expected,
            "shape-outside: {authored}"
        );
        let source = declaration(&format!("shape-outside: {authored}"));
        let reparsed = declaration(&format!("shape-outside: {expected}"));
        assert_eq!(
            outside_meaning(&reparsed),
            outside_meaning(&source),
            "{authored} must keep its meaning as `{expected}`"
        );
        assert_eq!(
            specified("shape-outside", expected),
            *expected,
            "{authored} must reach a fixed point"
        );
    }
}

#[test]
fn shape_outside_omits_the_margin_box_next_to_a_basic_shape() {
    assert_outside_cases(&[
        // The issue's spec form.
        ("margin-box circle(50%)", "circle(50%)"),
        ("circle(50%) margin-box", "circle(50%)"),
        ("circle(50%)", "circle(50%)"),
        ("MARGIN-BOX circle()", "circle()"),
        ("inset(1px) margin-box", "inset(1px)"),
        ("margin-box inset(1px 2px)", "inset(1px 2px)"),
        (
            "ellipse(10% closest-side) margin-box",
            "ellipse(10% closest-side)",
        ),
        (
            "polygon(nonzero, 0 0, 100% 0) margin-box",
            "polygon(nonzero, 0 0, 100% 0)",
        ),
        ("rect(1px auto 2px 0) margin-box", "rect(1px auto 2px 0)"),
        ("xywh(0 0 3px 4px) margin-box", "xywh(0 0 3px 4px)"),
        (
            "path(evenodd, 'M0 0 L1 1 Z') margin-box",
            "path(evenodd, \"M0 0 L1 1 Z\")",
        ),
        (
            "shape(from 0px 0px, line to 1px 1px) margin-box",
            "shape(from 0px 0px, line to 1px 1px)",
        ),
    ]);
}

#[test]
fn shape_outside_keeps_every_box_that_is_not_the_default() {
    assert_outside_cases(&[
        ("circle(50%) border-box", "circle(50%) border-box"),
        ("border-box circle(50%)", "circle(50%) border-box"),
        ("circle(50%) padding-box", "circle(50%) padding-box"),
        ("padding-box circle(50%)", "circle(50%) padding-box"),
        ("circle(50%) content-box", "circle(50%) content-box"),
        ("content-box inset(1px)", "inset(1px) content-box"),
        ("inset(1px) border-box", "inset(1px) border-box"),
    ]);
}

#[test]
fn shape_outside_keeps_a_lone_box_and_the_exclusive_alternatives() {
    // Without a basic shape the box is the whole value: `margin-box` alone
    // sets the shape from the margin edge, and is not a default to leave out.
    assert_outside_cases(&[
        ("margin-box", "margin-box"),
        ("border-box", "border-box"),
        ("padding-box", "padding-box"),
        ("content-box", "content-box"),
        ("none", "none"),
        ("url(\"asset.png\")", "url(\"asset.png\")"),
        ("linear-gradient(red, blue)", "linear-gradient(red, blue)"),
    ]);
}

#[test]
fn shape_outside_keeps_symbolic_shapes_unresolved_while_omitting_the_box() {
    assert_outside_cases(&[
        (
            "circle(calc(1px + 2%)) margin-box",
            "circle(calc(2% + 1px))",
        ),
        (
            "margin-box circle(calc(1px + 2%))",
            "circle(calc(2% + 1px))",
        ),
        (
            "circle(calc(1px + 2%)) border-box",
            "circle(calc(2% + 1px)) border-box",
        ),
    ]);
    assert_eq!(specified("shape-outside", "var(--shape)"), "var(--shape)");
}

#[test]
fn shape_outside_declaration_block_reads_the_same_shortest_form() {
    let report = CssParser::default().parse_block_contents("shape-outside: margin-box circle(50%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let block =
        CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax(), Default::default())
            .unwrap();
    assert_eq!(
        read(&block, CssKnownProperty::ShapeOutside).as_deref(),
        Some("circle(50%)")
    );
}

#[test]
fn shape_outside_typed_construction_omits_an_explicit_margin_box_but_keeps_the_others() {
    use surgeist_css::values::shape::{CssCircleRadius, CssCircleShape, CssShapeOutsideShape};
    let circle = || CssBasicShape::Circle(CssCircleShape::new(CssCircleRadius::Default, None));
    for (shape_box, expected) in [
        (None, "circle()"),
        (Some(CssShapeBox::MarginBox), "circle()"),
        (Some(CssShapeBox::BorderBox), "circle() border-box"),
        (Some(CssShapeBox::PaddingBox), "circle() padding-box"),
        (Some(CssShapeBox::ContentBox), "circle() content-box"),
    ] {
        let value = CssShapeOutsideShape::new(circle(), shape_box);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected,
            "{shape_box:?}"
        );
        assert_eq!(
            CssShapeOutside::BasicShape(value)
                .serialize_specified(Default::default())
                .unwrap(),
            expected,
            "{shape_box:?}"
        );
    }
    assert_eq!(
        CssShapeOutside::ShapeBox(CssShapeBox::MarginBox)
            .serialize_specified(Default::default())
            .unwrap(),
        "margin-box"
    );
}
