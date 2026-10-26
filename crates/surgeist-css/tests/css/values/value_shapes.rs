//! The shared value shapes: `logical? <T>{1,4}`, `<T>{1,2}`, `auto | <T>` and the productions that
//! several properties reuse.
//!
//! Expected texts follow CSS Box Model 3 (the one-to-four side assignment) and CSSOM 1 §6.7.2
//! "serialize a CSS value" step 2 ("If component values can be omitted or replaced with a
//! shorter representation without changing the meaning of the value, omit/replace them"),
//! both pinned under `references/`.

use crate::css::support::sole_declaration;
use surgeist_css::*;

fn written(css: &str) -> String {
    sole_declaration(css)
        .serialize_specified(CssSerializationLimits::default())
        .unwrap()
}

/// The smallest input-node limit at which the declaration still serializes.
fn input_nodes(css: &str) -> usize {
    let declaration = sole_declaration(css);
    (0..)
        .find(|nodes| {
            declaration
                .serialize_specified(CssSerializationLimits::new(*nodes, usize::MAX, usize::MAX))
                .is_ok()
        })
        .unwrap()
}

/// The smallest projection-node limit at which the declaration still serializes.
fn projection_nodes(css: &str) -> usize {
    let declaration = sole_declaration(css);
    (0..)
        .find(|nodes| {
            declaration
                .serialize_specified(CssSerializationLimits::new(usize::MAX, *nodes, usize::MAX))
                .is_ok()
        })
        .unwrap()
}

fn property_value(css: &str) -> CssDeclaration {
    sole_declaration(css)
}

#[test]
fn four_side_assignment_follows_the_one_to_four_rule() {
    let sides = |count: usize| {
        let values = ["1px", "2px", "3px", "4px"];
        let text = format!("padding: {}", values[..count].join(" "));
        let declaration = property_value(&text);
        let Some(CssKnownPropertyValueRef::Padding(value)) =
            declaration.known().unwrap().property_value()
        else {
            panic!("padding value");
        };
        value
            .value()
            .assigned()
            .map(|side| side.serialize_specified(Default::default()).unwrap())
    };
    // Box Model 3: one value applies to all sides; two set top/bottom then right/left; three set
    // top, left/right, bottom; four apply to top, right, bottom and left.
    assert_eq!(sides(1), ["1px", "1px", "1px", "1px"]);
    assert_eq!(sides(2), ["1px", "2px", "1px", "2px"]);
    assert_eq!(sides(3), ["1px", "2px", "3px", "2px"]);
    assert_eq!(sides(4), ["1px", "2px", "3px", "4px"]);
}

#[test]
fn four_side_counts_other_than_one_to_four_are_refused() {
    let side = CssPaddingValue::new(
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
            CssComponentValue::try_dimension("1", "px").unwrap(),
        )
        .unwrap(),
    );
    assert!(CssFourSides::<CssPaddingValue>::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(
        CssFourSides::try_new(CssBoxSideKind::Physical, vec![side.clone(); 5]).is_none(),
        "five values"
    );
    let four = CssFourSides::try_new(CssBoxSideKind::Logical, vec![side; 4]).unwrap();
    assert_eq!(four.kind(), CssBoxSideKind::Logical);
    assert_eq!(four.authored().len(), 4);
}

#[test]
fn typed_serialization_of_every_four_side_family_writes_the_shortest_form() {
    for (authored, expected) in [
        ("margin: 1px 1px 1px 1px", "margin: 1px;"),
        ("margin: 1px 2px 1px 2px", "margin: 1px 2px;"),
        ("margin: 1px 2px 3px 2px", "margin: 1px 2px 3px;"),
        ("margin: 1px 2px 1px 3px", "margin: 1px 2px 1px 3px;"),
        ("margin: 1px 1px 2px 1px", "margin: 1px 1px 2px;"),
        // Exact numeric equality, not spelling equality.
        ("margin: 1px 1.0px", "margin: 1px;"),
        (
            "margin: logical 1px 2px 1px 2px",
            "margin: logical 1px 2px;",
        ),
        ("padding: 1px 2px 1px", "padding: 1px 2px;"),
        ("inset: auto auto auto auto", "inset: auto;"),
        ("border-width: thin thin", "border-width: thin;"),
        (
            "border-style: solid none solid none",
            "border-style: solid none;",
        ),
        ("border-color: red red", "border-color: red;"),
        ("scroll-margin: 0 0 0 0", "scroll-margin: 0;"),
        (
            "scroll-padding: auto 1px auto 1px",
            "scroll-padding: auto 1px;",
        ),
    ] {
        assert_eq!(written(authored), expected, "{authored}");
    }
}

#[test]
fn shortening_never_merges_values_that_differ_only_in_rounded_text() {
    // Two exact lengths whose six-place text is equal stay separate.
    assert_eq!(
        written("margin: 0.12345641px 0.12345642px"),
        "margin: 0.123456px 0.123456px;"
    );
}

#[test]
fn physical_and_logical_four_sides_charge_the_same_aggregate_node() {
    // One aggregate node covers the `logical` keyword; physical forms pay it too, so a family's
    // physical and logical spellings of the same values cost the same.
    for (physical, logical) in [
        ("margin: 1px 2px", "margin: logical 1px 2px"),
        (
            "padding: 1px 2px 3px 4px",
            "padding: logical 1px 2px 3px 4px",
        ),
        ("inset: auto 2px 3px", "inset: logical auto 2px 3px"),
        ("scroll-margin: 1px 2px", "scroll-margin: logical 1px 2px"),
        (
            "scroll-padding: auto 2px",
            "scroll-padding: logical auto 2px",
        ),
        ("border-width: thin 2px", "border-width: logical thin 2px"),
        (
            "border-style: solid dotted",
            "border-style: logical solid dotted",
        ),
        ("border-color: red blue", "border-color: logical red blue"),
    ] {
        assert_eq!(
            input_nodes(physical),
            input_nodes(logical),
            "{physical} against {logical}"
        );
    }
}

#[test]
fn the_aggregate_node_is_charged_once_for_each_family_whatever_the_value_count() {
    // The aggregate adds one node to the sum of its authored values, whether or not a value is
    // written, so the cost grows by exactly the extra values.
    let one = input_nodes("margin: 1px");
    assert_eq!(input_nodes("margin: 1px 1px"), one + 1);
    assert_eq!(input_nodes("margin: 1px 1px 1px 1px"), one + 3);
    let border_one = input_nodes("border-width: thin");
    assert_eq!(
        input_nodes("border-width: thin thin thin thin"),
        border_one + 3
    );
    // `margin` now costs what `border-width` does for the same count of one-node values.
    assert_eq!(one, border_one);
}

#[test]
fn every_side_pair_family_writes_the_shortest_form() {
    // CSSOM 1 §6.7.2 step 2: a second value that the omitted-second rule reproduces is dropped.
    for (authored, expected) in [
        ("margin-block: 1px 1px", "margin-block: 1px;"),
        ("padding-inline: 1px 1px", "padding-inline: 1px;"),
        ("inset-block: auto auto", "inset-block: auto;"),
        ("size: 1px 1px", "size: 1px;"),
        ("max-size: 1px 1px", "max-size: 1px;"),
        ("border-block-width: thin thin", "border-block-width: thin;"),
        (
            "border-inline-style: solid solid",
            "border-inline-style: solid;",
        ),
        ("border-block-color: red red", "border-block-color: red;"),
        ("gap: 1px 1px", "gap: 1px;"),
        ("border-spacing: 2px", "border-spacing: 2px;"),
        ("border-spacing: 2px 2px", "border-spacing: 2px;"),
        ("scroll-margin-block: 1px 1px", "scroll-margin-block: 1px;"),
        (
            "scroll-padding-inline: auto auto",
            "scroll-padding-inline: auto;",
        ),
        ("cue: none none", "cue: none;"),
        ("pause: 1s 1s", "pause: 1s;"),
        ("rest: 1s 1s", "rest: 1s;"),
        ("interest-delay: 1s 1s", "interest-delay: 1s;"),
        // Scroll Snap 1 §5.2: "If only one value is specified, the second value defaults to the
        // same value."
        (
            "scroll-snap-align: start start",
            "scroll-snap-align: start;",
        ),
        ("scroll-snap-align: start", "scroll-snap-align: start;"),
        // Different values stay.
        ("margin-block: 1px 2px", "margin-block: 1px 2px;"),
        ("gap: 1px 2px", "gap: 1px 2px;"),
        ("border-spacing: 2px 3px", "border-spacing: 2px 3px;"),
        ("pause: 1s 2s", "pause: 1s 2s;"),
        ("interest-delay: 1s 2s", "interest-delay: 1s 2s;"),
        (
            "scroll-snap-align: start end",
            "scroll-snap-align: start end;",
        ),
    ] {
        assert_eq!(written(authored), expected, "{authored}");
    }
}

#[test]
fn every_side_pair_charges_the_aggregate_node_its_four_side_family_charges() {
    // A pair is the `{1,2}` form of the same family, so it costs what the four-side shorthand
    // of two values costs: one aggregate node plus each value.
    for (pair, four) in [
        ("margin-block: 1px 2px", "margin: 1px 2px"),
        ("padding-inline: 1px 2px", "padding: 1px 2px"),
        ("inset-block: auto 2px", "inset: auto 2px"),
        ("scroll-margin-block: 1px 2px", "scroll-margin: 1px 2px"),
        (
            "scroll-padding-inline: auto 2px",
            "scroll-padding: auto 2px",
        ),
        ("border-block-width: thin 2px", "border-width: thin 2px"),
        (
            "border-inline-style: solid dotted",
            "border-style: solid dotted",
        ),
        ("border-block-color: red blue", "border-color: red blue"),
    ] {
        assert_eq!(
            input_nodes(pair),
            input_nodes(four),
            "{pair} against {four}"
        );
    }
    // `gap` and `border-spacing` take the same lengths as a padding pair.
    assert_eq!(
        input_nodes("gap: 1px 2px"),
        input_nodes("padding-block: 1px 2px")
    );
    assert_eq!(
        input_nodes("border-spacing: 1px 2px"),
        input_nodes("padding-block: 1px 2px")
    );
    // `interest-delay` and `scroll-snap-align` are pairs of one-node values.
    assert_eq!(
        input_nodes("interest-delay: 1s 2s"),
        input_nodes("padding-block: 1px 2px")
    );
    assert_eq!(
        input_nodes("scroll-snap-align: start end"),
        input_nodes("padding-block: 1px 2px")
    );
}

#[test]
fn a_side_pair_retains_whether_its_second_value_was_authored() {
    let declaration = property_value("margin-block: 1px");
    let Some(CssKnownPropertyValueRef::MarginBlock(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("margin-block value");
    };
    let pair = value.value();
    assert!(pair.authored_second().is_none());
    assert_eq!(pair.second(), pair.first());

    let declaration = property_value("margin-block: 1px 2px");
    let Some(CssKnownPropertyValueRef::MarginBlock(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("margin-block value");
    };
    let pair = value.value();
    assert!(pair.authored_second().is_some());
    assert_ne!(pair.second(), pair.first());
}

#[test]
fn auto_or_distinguishes_the_keyword_from_the_checked_value() {
    let declaration = property_value("clip: rect(auto, 1px, auto, 2px)");
    let Some(CssKnownPropertyValueRef::Clip(value)) = declaration.known().unwrap().property_value()
    else {
        panic!("clip value");
    };
    let CssClip::Rect(rect) = value.value() else {
        panic!("a rect clip");
    };
    assert!(rect.top().is_auto());
    assert!(rect.top().value().is_none());
    assert!(!rect.right().is_auto());
    assert!(rect.right().value().is_some());
    assert!(matches!(rect.bottom(), CssAutoOr::Auto));
    assert!(matches!(rect.left(), CssAutoOr::Value(_)));
}

#[test]
fn the_background_size_component_and_scroll_padding_side_share_one_type() {
    let declaration = property_value("background-size: auto 10px");
    let Some(CssKnownPropertyValueRef::BackgroundSize(sizes)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("background-size value");
    };
    let [
        CssBackgroundSize::Explicit {
            width,
            height: Some(height),
        },
    ] = sizes.value().sizes()
    else {
        panic!("explicit width and height");
    };
    assert!(width.is_auto());
    let declaration = property_value("scroll-padding-top: 10px");
    let Some(CssKnownPropertyValueRef::ScrollPaddingTop(top)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("scroll-padding-top value");
    };
    let side: &CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>> = top.value();
    assert_eq!(side, height);
}

#[test]
fn outline_width_and_border_width_share_one_line_width() {
    let declaration = property_value("outline-width: 2px");
    let Some(CssKnownPropertyValueRef::OutlineWidth(outline)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("outline-width value");
    };
    let declaration = property_value("border-top-width: 2px");
    let Some(CssKnownPropertyValueRef::BorderTopWidth(border)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("border-top-width value");
    };
    let line_width: &CssLineWidth = outline.value();
    assert_eq!(line_width, border.value());
    assert!(matches!(line_width, CssLineWidth::Length(_)));
    // `<line-style>` is likewise the one border-style and column-rule-style type.
    let declaration = property_value("column-rule-style: dotted");
    let Some(CssKnownPropertyValueRef::ColumnRuleStyle(rule)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("column-rule-style value");
    };
    let declaration = property_value("border-top-style: dotted");
    let Some(CssKnownPropertyValueRef::BorderTopStyle(border)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("border-top-style value");
    };
    let line_style: &CssLineStyle = rule.value();
    assert_eq!(line_style, border.value());
}

#[test]
fn mask_boxes_and_motion_coordinate_boxes_are_one_type() {
    let declaration = property_value("mask-origin: fill-box, view-box");
    let Some(CssKnownPropertyValueRef::MaskOrigin(origins)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("mask-origin value");
    };
    assert_eq!(
        origins.value().boxes(),
        &[CssCoordBox::FillBox, CssCoordBox::ViewBox]
    );
    assert_eq!(
        CssCoordBox::from_keyword("fill-box"),
        Some(CssCoordBox::FillBox)
    );
    assert_eq!(CssCoordBox::FillBox.as_str(), "fill-box");
    // Box 4 `<coord-box>` excludes `margin-box`.
    assert_eq!(CssCoordBox::from_keyword("margin-box"), None);
}

#[test]
fn constructors_that_cannot_fail_do_not_return_an_option() {
    let level =
        CssNumeric::<CssInteger, CssPositive>::try_from_literal(CssNumericLiteral::from_i32(3))
            .unwrap();
    let bookmark = CssBookmarkLevel::new(level.clone());
    assert_eq!(bookmark.level(), Some(&level));
    assert!(CssBookmarkLevel::none().level().is_none());

    let content = CssGridTrackRepeatContent::try_new(vec![CssGridTrackRepeatComponent::TrackSize(
        CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto()),
    )])
    .unwrap();
    let repeat = CssGridIntegerTrackRepeat::new(level.clone(), content);
    assert_eq!(repeat.count(), &level);
}

/// The smallest `(input, projection)` limits at which the stylesheet's CSSOM text is written.
fn cssom_sheet_charge(css: &str) -> (usize, usize) {
    let report = CssParser::default().parse_sheet(css);
    assert!(report.is_clean(), "{css}: {report:?}");
    let sheet = report.syntax();
    let defaults = CssSerializationLimits::default();
    let smallest = |accepts: &dyn Fn(usize) -> bool| (0..).find(|value| accepts(*value)).unwrap();
    let input = smallest(&|nodes| {
        sheet
            .serialize_cssom(CssSerializationLimits::new(
                nodes,
                defaults.max_projection_nodes(),
                defaults.max_css_bytes(),
            ))
            .is_ok()
    });
    let projection = smallest(&|nodes| {
        sheet
            .serialize_cssom(CssSerializationLimits::new(
                defaults.max_input_nodes(),
                nodes,
                defaults.max_css_bytes(),
            ))
            .is_ok()
    });
    (input, projection)
}

#[test]
fn an_unequal_border_colour_pair_is_compared_once_when_a_block_is_reconstructed() {
    // Projection charge observed on the step 0 baseline (HEAD 32ef372a: 423; the shapes made it 424): the colour comparison that
    // decides the pair is already charged by the reconstruction, so writing the pair must not
    // compare, and charge, a second time.
    assert_eq!(
        cssom_sheet_charge(".a{border-block-color: red blue}"),
        (18, 423)
    );
}

#[test]
fn a_single_border_spacing_length_leaves_the_second_unauthored() {
    // CSS Tables 3 / CSS 2.1 `border-spacing`: `<length> <length>?`, the second copying the first.
    let declaration = property_value("border-spacing: 2px");
    let Some(CssKnownPropertyValueRef::BorderSpacing(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("border-spacing value");
    };
    let pair = value.value();
    assert!(pair.authored_second().is_none());
    assert_eq!(pair.second(), pair.first());
    let declaration = property_value("border-spacing: 2px 2px");
    let Some(CssKnownPropertyValueRef::BorderSpacing(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("border-spacing value");
    };
    assert!(value.value().authored_second().is_some());
}

#[test]
fn corner_radii_and_intrinsic_sizes_write_the_shortest_form() {
    // CSSOM 1 §6.7.2 step 2 with Backgrounds 3 (`border-*-radius` is `<length-percentage>{1,2}`
    // and an omitted vertical radius copies the horizontal one; `border-radius` assigns its
    // one-to-four values to the corners as `margin` does to the sides, then optionally a slash
    // and the vertical list) and Sizing 4 (`contain-intrinsic-size` is
    // `[ auto? [ none | <length> ] ]{1,2}`, one value applying to both axes).
    for (authored, expected) in [
        (
            "border-top-left-radius: 1px 1px",
            "border-top-left-radius: 1px;",
        ),
        (
            "border-top-left-radius: calc(1px + 2px) calc(1px + 2px)",
            "border-top-left-radius: calc(3px);",
        ),
        (
            "border-start-end-radius: 5% 5%",
            "border-start-end-radius: 5%;",
        ),
        ("border-radius: 1px 1px", "border-radius: 1px;"),
        ("border-radius: 1px 2px 1px 2px", "border-radius: 1px 2px;"),
        ("border-radius: 1px / 1px", "border-radius: 1px;"),
        (
            "border-radius: 1px 2px 1px 2px / 1px 2px 1px 2px",
            "border-radius: 1px 2px;",
        ),
        (
            "border-radius: 1px 2px 1px 2px / 3px 3px 3px 3px",
            "border-radius: 1px 2px / 3px;",
        ),
        (
            "contain-intrinsic-size: 10px 10px",
            "contain-intrinsic-size: 10px;",
        ),
        (
            "contain-intrinsic-size: auto 10px auto 10px",
            "contain-intrinsic-size: auto 10px;",
        ),
        (
            "contain-intrinsic-size: none none",
            "contain-intrinsic-size: none;",
        ),
        // Different values, and a different `auto`, stay.
        (
            "border-top-left-radius: 1px 2px",
            "border-top-left-radius: 1px 2px;",
        ),
        (
            "border-radius: 1px 2px / 3px",
            "border-radius: 1px 2px / 3px;",
        ),
        (
            "border-radius: 1px 2px 3px / 4px 5px",
            "border-radius: 1px 2px 3px / 4px 5px;",
        ),
        (
            "contain-intrinsic-size: 10px 20px",
            "contain-intrinsic-size: 10px 20px;",
        ),
        (
            "contain-intrinsic-size: auto 10px 10px",
            "contain-intrinsic-size: auto 10px 10px;",
        ),
    ] {
        assert_eq!(written(authored), expected, "{authored}");
    }
}

#[test]
fn corner_radii_keep_their_charges_and_intrinsic_sizes_gain_the_aggregate_node() {
    // Corner radii and `border-radius` already charged one aggregate node plus each authored
    // value (HEAD: `1px` is 4 input nodes, `1px 2px` is 5). `contain-intrinsic-size` charged no
    // aggregate (HEAD: `10px` is 3, `10px 20px` is 4) and now charges the one every
    // `<T>{1,2}` value charges.
    for (css, nodes) in [
        ("border-top-left-radius: 1px", 4),
        ("border-top-left-radius: 1px 2px", 5),
        ("border-top-left-radius: 1px 1px", 5),
        ("border-radius: 1px", 4),
        ("border-radius: 1px 1px", 5),
        ("border-radius: 1px 2px / 3px", 6),
        ("border-radius: 1px / 1px", 5),
        ("border-radius: 1px 2px 1px 2px / 1px 2px 1px 2px", 11),
        ("contain-intrinsic-size: 10px", 4),
        ("contain-intrinsic-size: 10px 20px", 5),
        ("contain-intrinsic-size: 10px 10px", 5),
        ("contain-intrinsic-size: auto 10px auto 10px", 7),
    ] {
        assert_eq!(input_nodes(css), nodes, "{css}");
    }
}

#[test]
fn a_calc_valued_scroll_pair_and_quad_keep_the_charges_they_had_plus_the_aggregate_node() {
    // Typed charges observed on HEAD 32ef372a for calc-valued scroll-margin and scroll-padding
    // (input, projection): the comparison that decides the shortest form already paid for the
    // calculation capture the value's own output reuses, so the shapes add exactly the one
    // aggregate node and nothing else.
    for (css, input, projection) in [
        (
            "scroll-margin-block: calc(1px + 2px) calc(1px + 2px)",
            10,
            8,
        ),
        (
            "scroll-margin-block: calc(1px + 2px) calc(1px + 3px)",
            10,
            8,
        ),
        ("scroll-margin-block: calc(1px + 2px) 5px", 7, 6),
        (
            "scroll-margin: calc(1px + 2px) calc(1px + 2px) calc(1px + 2px) calc(1px + 2px)",
            18,
            14,
        ),
        (
            "scroll-margin: calc(1px + 2px) calc(1px + 3px) calc(1px + 4px) calc(1px + 5px)",
            18,
            14,
        ),
        (
            "scroll-margin: calc(1px + 2px) 5px calc(1px + 2px) 5px",
            12,
            10,
        ),
        (
            "scroll-padding-inline: calc(1px + 2%) calc(1px + 2%)",
            10,
            12,
        ),
        (
            "scroll-padding: calc(1px + 2%) 5px calc(1px + 2%) 5px",
            12,
            14,
        ),
    ] {
        assert_eq!(input_nodes(css), input + 1, "{css}: input nodes");
        assert_eq!(
            projection_nodes(css),
            projection + 1,
            "{css}: projection nodes"
        );
    }
}

#[test]
fn shape_function_round_radii_stay_verbatim_in_specified_and_cssom_text() {
    // CSS Shapes 1 §3.3: shape functions serialize "as per their individual grammars", and
    // omitting components is limited to computed values; specified-value serialization is
    // "relatively trivial". The `border-radius` property shortens its own value (#1384), but the
    // `round` radii inside `inset()`, `xywh()` and `rect()` are function arguments and stay as
    // authored. The charges are the ones observed on HEAD 32ef372a (input nodes).
    for (css, nodes) in [
        ("clip-path: inset(1px round 2px 2px 2px 2px / 3px)", 12),
        ("clip-path: inset(1px round 2px 3px / 2px 3px)", 11),
        (
            "clip-path: xywh(1px 2px 3px 4px round 5px 5px / 5px 5px)",
            13,
        ),
        ("clip-path: rect(1px 2px 3px 4px round 4px / 4px)", 11),
        ("shape-outside: inset(1px round 2px / 2px)", 9),
    ] {
        let expected = format!("{css};");
        assert_eq!(written(css), expected, "specified: {css}");
        let report = CssParser::default().parse_block_contents(css);
        assert_eq!(
            report
                .syntax()
                .serialize_cssom(CssSerializationLimits::default())
                .unwrap(),
            expected,
            "cssom: {css}"
        );
        assert_eq!(input_nodes(css), nodes, "{css}");
        assert_eq!(projection_nodes(css), nodes, "{css}");
    }
}
