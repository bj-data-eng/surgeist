#![forbid(unsafe_code)]
//! Motion WD2024-11-05 imports Values5 WD2024-09-17 §4.2 directly at
//! offset-position, offset-anchor and ray's at-position. These tests exercise
//! existing public callers, with independent one/two/four-component oracles.
//! Symbolic output follows the accepted Position owner: Cartesian x/y order,
//! named and relative flow block/inline order, without writing-mode resolution.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

const LONGHANDS: [&str; 5] = [
    "offset-position",
    "offset-path",
    "offset-distance",
    "offset-rotate",
    "offset-anchor",
];

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name).unwrap()
}

fn declaration(name: &str, value: &str, front: usize) -> CssDeclaration {
    let p = property(name);
    let source = match front {
        0 => {
            let css = format!("/*😀*/{name}:{value}!important");
            let report = parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            assert_eq!(validate_style_attribute(&css), Ok(report.syntax().clone()));
            assert_eq!(report.syntax().len(), 1);
            let source = report.syntax()[0].clone();
            assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
            source
        }
        1 | 2 => {
            let components = parse_component_values(value).unwrap();
            let snapshot = components.clone();
            let result = if front == 1 {
                parse_property_value(
                    CssPropertyNameRef::Known(p),
                    components.clone(),
                    CssImportance::Important,
                )
            } else {
                parse_property_value_for_grammar(
                    p.grammar(),
                    components.clone(),
                    CssImportance::Important,
                )
            };
            let source = result.unwrap_or_else(|error| panic!("{name}:{value}: {error:?}"));
            assert_eq!(components, snapshot);
            assert_eq!(source.value_components(), &components);
            source
        }
        3 | 4 => {
            let report = if front == 3 {
                parse_property_value_text(
                    value,
                    CssPropertyNameRef::Known(p),
                    CssImportance::Important,
                )
            } else {
                parse_property_value_text_for_grammar(value, p.grammar(), CssImportance::Important)
            };
            assert!(
                report.is_clean(),
                "{name}:{value}: {:?}",
                report.diagnostics()
            );
            let source = report.syntax().as_ref().unwrap().clone();
            assert_eq!(source.parsed_value().unwrap().source().as_str(), value);
            source
        }
        _ => unreachable!(),
    };
    assert_eq!(source.known().unwrap().property(), p);
    assert_eq!(source.importance(), CssImportance::Important);
    source
}

fn full_position(position: impl Into<CssPosition>) -> CssPosition {
    position.into()
}

fn same_position(left: &CssOffsetPosition, right: &CssOffsetPosition) {
    match (left, right) {
        (CssOffsetPosition::Position(a), CssOffsetPosition::Position(b)) => {
            let a: CssPosition = full_position(a.clone());
            let b: CssPosition = full_position(b.clone());
            assert_eq!(a, b);
        }
        _ => assert_eq!(left, right),
    }
}

fn same_anchor(left: &CssOffsetAnchor, right: &CssOffsetAnchor) {
    match (left, right) {
        (CssOffsetAnchor::Position(a), CssOffsetAnchor::Position(b)) => {
            let a: CssPosition = full_position(a.clone());
            let b: CssPosition = full_position(b.clone());
            assert_eq!(a, b);
        }
        _ => assert_eq!(left, right),
    }
}

fn same_path(left: &CssOffsetPath, right: &CssOffsetPath) {
    match (left.view(), right.view()) {
        (CssOffsetPathRef::Path(a), CssOffsetPathRef::Path(b)) => {
            assert_eq!(a.coord_box(), b.coord_box());
            match (a.path(), b.path()) {
                (CssOffsetPathKind::Ray(a), CssOffsetPathKind::Ray(b)) => {
                    assert_eq!(a.contain(), b.contain());
                    assert_eq!(
                        a.size().unwrap_or(CssRaySize::ClosestSide),
                        b.size().unwrap_or(CssRaySize::ClosestSide)
                    );
                    assert_eq!(
                        a.position().cloned().map(full_position),
                        b.position().cloned().map(full_position)
                    );
                    if let (Some(a), Some(b)) = (a.angle().literal(), b.angle().literal()) {
                        assert_eq!(a.unit(), b.unit());
                        // Literal authored spelling is intentionally retained by Eq;
                        // the independent decimal values compare reparse meaning.
                        assert_eq!(
                            a.numeric().representation().parse::<f64>().unwrap(),
                            b.numeric().representation().parse::<f64>().unwrap()
                        );
                    } else {
                        assert_eq!(
                            CssRay::new(a.angle().clone(), None, false, None)
                                .serialize_specified()
                                .unwrap(),
                            CssRay::new(b.angle().clone(), None, false, None)
                                .serialize_specified()
                                .unwrap()
                        );
                    }
                }
                _ => assert_eq!(
                    a.path().serialize_specified().unwrap(),
                    b.path().serialize_specified().unwrap()
                ),
            }
        }
        _ => assert_eq!(left, right),
    }
}

fn same_motion_value(left: &CssDeclaration, right: &CssDeclaration) {
    let left = left.known().unwrap().property_value().unwrap();
    let right = right.known().unwrap().property_value().unwrap();
    match (left, right) {
        (
            CssKnownPropertyValueRef::OffsetPosition(a),
            CssKnownPropertyValueRef::OffsetPosition(b),
        ) => same_position(a.value(), b.value()),
        (CssKnownPropertyValueRef::OffsetAnchor(a), CssKnownPropertyValueRef::OffsetAnchor(b)) => {
            same_anchor(a.value(), b.value())
        }
        (CssKnownPropertyValueRef::OffsetPath(a), CssKnownPropertyValueRef::OffsetPath(b)) => {
            same_path(a.value(), b.value())
        }
        (CssKnownPropertyValueRef::Offset(a), CssKnownPropertyValueRef::Offset(b)) => {
            let (a, b) = (a.value(), b.value());
            match (a.position(), b.position()) {
                (Some(a), Some(b)) => same_position(a, b),
                (None, None) => {}
                _ => panic!("position presence"),
            }
            match (a.path(), b.path()) {
                (Some(a), Some(b)) => same_path(a, b),
                (None, None) => {}
                _ => panic!("path presence"),
            }
            match (a.anchor(), b.anchor()) {
                (Some(a), Some(b)) => same_anchor(a, b),
                (None, None) => {}
                _ => panic!("anchor presence"),
            }
            assert_eq!(
                a.distance()
                    .map(|value| value.serialize_specified().unwrap()),
                b.distance()
                    .map(|value| value.serialize_specified().unwrap())
            );
            assert_eq!(
                a.rotate().map(|value| value.serialize_specified().unwrap()),
                b.rotate().map(|value| value.serialize_specified().unwrap())
            );
        }
        _ => panic!("same motion property"),
    }
}

fn accept(name: &str, cases: &[(&str, &str)]) {
    for &(input, expected) in cases {
        for front in 0..5 {
            let source = declaration(name, input, front);
            let snapshot = source.clone();
            let output = format!("{name}: {expected} !important;");
            assert_eq!(source.to_specified_css().unwrap(), output, "front {front}");
            let reparsed = declaration(name, expected, front);
            same_motion_value(&source, &reparsed);
            assert_eq!(reparsed.to_specified_css().unwrap(), output);
            assert_eq!(source, snapshot);
        }
    }
}

macro_rules! positions {
    ($module:ident, $cases:expr) => {
        mod $module {
            use super::*;
            #[test]
            fn offset_position_retains_the_imported_family() {
                accept("offset-position", $cases);
            }
            #[test]
            fn offset_anchor_retains_the_imported_family() {
                accept("offset-anchor", $cases);
            }
        }
    };
}
positions!(
    cartesian_one,
    &[
        ("x-start", "x-start center"),
        ("x-end", "x-end center"),
        ("y-start", "center y-start"),
        ("y-end", "center y-end")
    ]
);
positions!(
    cartesian_two,
    &[
        ("y-end x-start", "x-start y-end"),
        ("x-end top", "x-end top"),
        ("right y-start", "right y-start"),
        ("center x-start", "x-start center"),
        ("x-start 10px", "x-start 10px"),
        ("-2% y-end", "-2% y-end")
    ]
);
positions!(
    cartesian_four,
    &[
        ("y-end -2% x-start 10px", "x-start 10px y-end -2%"),
        ("left -2px y-start 10%", "left -2px y-start 10%"),
        ("x-end 1em bottom -3px", "x-end 1em bottom -3px"),
        (
            "left -2px y-end calc(10% - 1px)",
            "left -2px y-end calc(10% - 1px)"
        )
    ]
);
positions!(
    named_one,
    &[
        ("block-start", "block-start center"),
        ("block-end", "block-end center"),
        ("inline-start", "center inline-start"),
        ("inline-end", "center inline-end")
    ]
);
positions!(
    named_two,
    &[
        ("inline-end block-start", "block-start inline-end"),
        ("center block-end", "block-end center"),
        ("inline-start center", "center inline-start")
    ]
);
positions!(
    named_four,
    &[
        (
            "inline-end -2px block-start 10%",
            "block-start 10% inline-end -2px"
        ),
        (
            "block-end -2px inline-start 2em",
            "block-end -2px inline-start 2em"
        ),
        (
            "block-end calc(10% - 2px) inline-start -3px",
            "block-end calc(10% - 2px) inline-start -3px"
        )
    ]
);
positions!(
    relative_two,
    &[
        ("start end", "start end"),
        ("end start", "end start"),
        ("start center", "start center"),
        ("center end", "center end")
    ]
);
positions!(
    relative_four,
    &[
        ("start 10px end -2%", "start 10px end -2%"),
        ("end -2px start 10%", "end -2px start 10%"),
        ("end calc(10% - 2px) end 4px", "end calc(10% - 2px) end 4px")
    ]
);

#[test]
fn physical_positions_remain_clean_in_every_motion_consumer() {
    for name in ["offset-position", "offset-anchor"] {
        accept(
            name,
            &[
                ("left", "left center"),
                ("bottom right", "right bottom"),
                ("left 10px", "left 10px"),
                ("top -2% right 3px", "right 3px top -2%"),
            ],
        );
    }
    accept(
        "offset-path",
        &[(
            "ray(at left top .25turn contain sides)",
            "ray(0.25turn sides contain at left top)",
        )],
    );
    accept(
        "offset",
        &[(
            "top left none reverse 100grad -2% / bottom right",
            "left top none -2% reverse 100grad / right bottom",
        )],
    );
}

#[test]
fn existing_full_position_owner_supplies_three_symbolic_families() {
    for (input, expected) in [
        ("y-end x-start", "x-start y-end"),
        (
            "inline-end 2px block-start -3%",
            "block-start -3% inline-end 2px",
        ),
        ("start end", "start end"),
    ] {
        let report = parse_style_attribute(&format!("clip-path:circle(at {input})"));
        assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("clip")
        };
        let CssClipPath::BasicShape(value) = value.value() else {
            panic!("shape")
        };
        let CssBasicShape::Circle(value) = value.shape() else {
            panic!("circle")
        };
        assert_eq!(
            value.position().unwrap().serialize_specified().unwrap(),
            expected
        );
    }
    let checked =
        CssPosition::try_from_named_axes(CssBlockPosition::Start, CssInlinePosition::End).unwrap();
    assert_eq!(
        checked.serialize_specified().unwrap(),
        "block-start inline-end"
    );
    assert_eq!(
        CssPosition::try_from_relative_axes(
            CssRelativeAxisPosition::Start,
            CssRelativeAxisPosition::End
        )
        .unwrap()
        .serialize_specified()
        .unwrap(),
        "start end"
    );
}

macro_rules! rays {
    ($test:ident, $input:literal, $expected:literal) => {
        #[test]
        fn $test() {
            accept(
                "offset-path",
                &[
                    (
                        concat!("ray(at ", $input, " .25turn contain sides)"),
                        concat!("ray(0.25turn sides contain at ", $expected, ")"),
                    ),
                    (
                        concat!(
                            "ray(100grad at ",
                            $input,
                            " contain closest-side) border-box"
                        ),
                        concat!("ray(100grad contain at ", $expected, ") border-box"),
                    ),
                    (
                        concat!("ray(at ", $input, " calc(90deg) farthest-corner)"),
                        concat!("ray(calc(90deg) farthest-corner at ", $expected, ")"),
                    ),
                ],
            );
        }
    };
}
rays!(
    ray_cartesian_position_leaves_literal_and_math_angles_to_the_caller,
    "y-end -2% x-start 10px",
    "x-start 10px y-end -2%"
);
rays!(
    ray_named_position_leaves_size_and_contain_to_the_caller,
    "inline-start block-end",
    "block-end inline-start"
);
rays!(
    ray_relative_position_keeps_block_then_inline_grouping,
    "start 10px end -2%",
    "start 10px end -2%"
);

#[test]
fn shorthand_cartesian_position_keeps_path_angle_and_slash_boundaries() {
    accept(
        "offset",
        &[
            (
                "y-end x-start none reverse .25turn -2% / x-end",
                "x-start y-end none -2% reverse 0.25turn / x-end center",
            ),
            (
                "x-start 10px y-end -2% ray(at y-start 100grad) 2px / top x-end",
                "x-start 10px y-end -2% ray(100grad at center y-start) 2px / x-end top",
            ),
            ("x-start 10px", "x-start 10px"),
        ],
    );
}

#[test]
fn shorthand_named_position_keeps_quoted_rotation_contiguous() {
    for rotation in [
        "reverse .25turn 1px",
        ".25turn reverse 1px",
        "1px reverse .25turn",
        "1px .25turn reverse",
    ] {
        accept(
            "offset",
            &[(
                &format!("inline-end block-start none {rotation} / block-end"),
                "block-start inline-end none 1px reverse 0.25turn / block-end center",
            )],
        );
    }
    accept(
        "offset",
        &[(
            "inline-end 1px block-start 2% / inline-start",
            "block-start 2% inline-end 1px / center inline-start",
        )],
    );
}

#[test]
fn shorthand_relative_position_preserves_order_and_authored_omissions() {
    accept(
        "offset",
        &[
            ("end start / start end", "end start / start end"),
            (
                "start 10px end -2% none / end 3px start 4%",
                "start 10px end -2% none / end 3px start 4%",
            ),
        ],
    );
    let source = declaration("offset", "start end", 0);
    let CssKnownPropertyValueRef::Offset(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("offset")
    };
    assert!(value.value().position().is_some());
    assert!(value.value().path().is_none());
    assert!(value.value().distance().is_none());
    assert!(value.value().rotate().is_none());
    assert!(value.value().anchor().is_none());
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("longhands")
    };
    assert_eq!(items.items().len(), 5);
    for (item, name) in items.items().iter().zip(LONGHANDS) {
        assert_eq!(item.property(), property(name));
        assert!(item.source().same_occurrence(&source));
    }
    let CssLonghandValueRef::OffsetPath(path) = items.items()[1].ordinary_value().unwrap().view()
    else {
        panic!("initial path")
    };
    assert!(matches!(path.view(), CssOffsetPathRef::None));
    let CssLonghandValueRef::OffsetPosition(position) =
        items.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("position")
    };
    assert_eq!(position.serialize_specified().unwrap(), "start end");
}

#[test]
fn logical_positions_compose_with_the_full_quoted_path_and_six_boxes() {
    for coord in [
        "content-box",
        "padding-box",
        "border-box",
        "fill-box",
        "stroke-box",
        "view-box",
    ] {
        accept(
            "offset",
            &[(
                &format!("block-start {coord} / inline-end"),
                &format!("block-start center {coord} / center inline-end"),
            )],
        );
    }
    accept(
        "offset",
        &[
            (
                "x-start none / start end",
                "x-start center none / start end",
            ),
            (
                "block-start view-box url('#p') / y-end",
                "block-start center url(\"#p\") view-box / center y-end",
            ),
            (
                "start end src('#p') fill-box / inline-start",
                "start end src(\"#p\") fill-box / center inline-start",
            ),
        ],
    );
}

fn reject(name: &str, input: &str) {
    let middle = format!("{name}:{input}!important;");
    let css = format!("/*😀*/color:red;{middle}color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one diagnostic for {css}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let start = css.find(&middle).unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + middle.len()
    );
    assert_eq!(
        diagnostic.span().start().column().value() as usize,
        css[..start].encode_utf16().count()
    );
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    for grammar in [false, true] {
        let components = parse_component_values(input).unwrap();
        let snapshot = components.clone();
        let result = if grammar {
            parse_property_value_for_grammar(
                property(name).grammar(),
                components.clone(),
                CssImportance::Important,
            )
        } else {
            parse_property_value(
                CssPropertyNameRef::Known(property(name)),
                components.clone(),
                CssImportance::Important,
            )
        };
        assert!(result.is_err(), "checked {name}:{input}");
        assert_eq!(components, snapshot);
        let report = if grammar {
            parse_property_value_text_for_grammar(
                input,
                property(name).grammar(),
                CssImportance::Important,
            )
        } else {
            parse_property_value_text(
                input,
                CssPropertyNameRef::Known(property(name)),
                CssImportance::Important,
            )
        };
        assert!(!report.is_clean(), "text {name}:{input}");
    }
}

#[test]
fn mixed_families_invalid_arity_and_background_three_values_drop_atomically() {
    for input in [
        "start",
        "end",
        "x-start x-end",
        "block-start block-end",
        "block-start y-end",
        "start inline-end",
        "left inline-start",
        "x-start 1px y-end",
        "block-start 1px inline-end",
        "start 1px end",
        "right 2px bottom",
        "center 1px y-end 2px",
        "x-start y-end 1px 2px",
        "start 1px end 2px 3px",
    ] {
        for name in ["offset-position", "offset-anchor"] {
            reject(name, input);
        }
        reject("offset-path", &format!("ray(90deg at {input})"));
        reject("offset", &format!("{input} none"));
        reject("offset", &format!("none / {input}"));
    }
    for value in [
        "left top none reverse 1px 90deg",
        "left top none 90deg 1px reverse",
        "left top margin-box",
        "left top /",
        "/ left top",
    ] {
        reject("offset", value);
    }
}

fn terminal_css(item: &CssLonghandContribution) -> String {
    match item.ordinary_value().unwrap().view() {
        CssLonghandValueRef::OffsetPosition(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::OffsetPath(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::OffsetAnchor(value) => value.serialize_specified().unwrap(),
        other => panic!("position consumer: {other:?}"),
    }
}

fn pending(name: &str, valid: &str, expected: &[&str], invalid: &str) {
    for authored in ["var(--motion)", "env(motion)"] {
        let source = declaration(name, authored, 0);
        let snapshot = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for _ in 0..2 {
            let bad = parse_component_values(invalid).unwrap();
            assert!(matches!(
                handle.reenter(bad).unwrap_err().kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement = parse_component_values(&format!("/*😀*/{valid}")).unwrap();
            let before = replacement.clone();
            let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("terminals")
            };
            let names = if name == "offset" {
                LONGHANDS.to_vec()
            } else {
                vec![name]
            };
            assert_eq!(items.items().len(), names.len());
            for (item, name) in items.items().iter().zip(names) {
                assert_eq!(item.property(), property(name));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
            if name == "offset" {
                assert_eq!(terminal_css(&items.items()[0]), expected[0]);
                assert_eq!(terminal_css(&items.items()[1]), expected[1]);
                assert_eq!(terminal_css(&items.items()[4]), expected[2]);
            } else {
                assert_eq!(terminal_css(&items.items()[0]), expected[0]);
            }
            assert_eq!(replacement, before);
            for (text, keyword) in [
                ("inherit", CssGlobalKeyword::Inherit),
                ("revert-layer", CssGlobalKeyword::RevertLayer),
            ] {
                let CssContributions::Longhands(items) = handle
                    .reenter(parse_component_values(text).unwrap())
                    .unwrap()
                else {
                    panic!("global")
                };
                for item in items.items() {
                    assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                }
            }
        }
        assert!(handle.source().same_occurrence(&source));
        assert_eq!(source, snapshot);
    }
}
#[test]
fn pending_position_reenters_logical_cartesian_after_failed_replacement() {
    pending(
        "offset-position",
        "y-end x-start",
        &["x-start y-end"],
        "x-start 1px y-end",
    );
}
#[test]
fn pending_anchor_reenters_named_flow_after_failed_replacement() {
    pending(
        "offset-anchor",
        "inline-end block-start",
        &["block-start inline-end"],
        "block-start y-end",
    );
}
#[test]
fn pending_ray_reenters_relative_flow_and_preserves_the_angle() {
    pending(
        "offset-path",
        "ray(at start end .25turn)",
        &["ray(0.25turn at start end)"],
        "ray(90deg at start)",
    );
}
#[test]
fn pending_shorthand_reenters_all_three_families_with_original_source() {
    pending(
        "offset",
        "x-start ray(at inline-end block-start 100grad) / start end",
        &[
            "x-start center",
            "ray(100grad at block-start inline-end)",
            "start end",
        ],
        "x-start none / block-start y-end",
    );
}

#[test]
fn css_wide_values_remain_symbolic_across_motion_consumers() {
    for name in ["offset-position", "offset-anchor", "offset-path", "offset"] {
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text, 0);
            let CssExpansion::Contributions(CssContributions::Longhands(items)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("globals")
            };
            assert_eq!(items.items().len(), if name == "offset" { 5 } else { 1 });
            for item in items.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            }
        }
    }
}

fn offset(source: &CssDeclaration) -> &CssOffset {
    let CssKnownPropertyValueRef::Offset(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("offset")
    };
    value.value()
}

#[test]
fn immutable_checked_motion_construction_reuses_parsed_symbolic_children() {
    let source = declaration(
        "offset",
        "x-start ray(at inline-end block-start .25turn) -2% reverse 100grad / start end",
        0,
    );
    let value = offset(&source);
    let snapshot = value.clone();
    let Some(CssOffsetPosition::Position(position)) = value.position() else {
        panic!("position")
    };
    let position: CssPosition = full_position(position.clone());
    assert_eq!(
        position,
        CssPosition::from_cartesian(
            CssCartesianPosition::try_new(
                CssHorizontalPosition::XStart,
                CssVerticalPosition::Center
            )
            .unwrap()
        )
    );
    let Some(CssOffsetAnchor::Position(anchor)) = value.anchor() else {
        panic!("anchor")
    };
    let anchor: CssPosition = full_position(anchor.clone());
    assert_eq!(
        anchor,
        CssPosition::try_from_relative_axes(
            CssRelativeAxisPosition::Start,
            CssRelativeAxisPosition::End
        )
        .unwrap()
    );
    let checked = CssOffset::try_new(
        value.position().cloned(),
        value.path().cloned(),
        value.distance().cloned(),
        value.rotate().cloned(),
        value.anchor().cloned(),
    )
    .unwrap();
    assert_eq!(checked, snapshot);
    assert_eq!(
        checked.serialize_specified().unwrap(),
        "x-start center ray(0.25turn at block-start inline-end) -2% reverse 100grad / start end"
    );
    let CssOffsetPathRef::Path(path) = checked.path().unwrap().view() else {
        panic!("path")
    };
    let CssOffsetPathKind::Ray(ray) = path.path() else {
        panic!("ray")
    };
    let rebuilt = CssRay::new(
        ray.angle().clone(),
        ray.size(),
        ray.contain(),
        ray.position().cloned(),
    );
    assert_eq!(&rebuilt, ray);
    let ray_position: CssPosition = full_position(ray.position().unwrap().clone());
    assert_eq!(
        ray_position,
        CssPosition::try_from_named_axes(CssBlockPosition::Start, CssInlinePosition::End).unwrap()
    );
    let CssValueOrigin::Parsed(origin) = checked.distance().unwrap().origin() else {
        panic!("distance provenance")
    };
    assert_eq!(
        origin.source().as_str(),
        "/*😀*/offset:x-start ray(at inline-end block-start .25turn) -2% reverse 100grad / start end!important"
    );
    assert_eq!(ray.angle().literal().unwrap().unit(), CssAngleUnit::Turns);
    assert_eq!(
        checked
            .rotate()
            .unwrap()
            .angle()
            .unwrap()
            .literal()
            .unwrap()
            .unit(),
        CssAngleUnit::Gradians
    );
    assert_eq!(value, &snapshot);
}

#[test]
fn normalization_preserves_logical_source_order_importance_and_atomic_limits() {
    let report = parse_sheet(
        ".a{offset-position:x-start!important;offset: start end none / inline-end;offset-anchor:var(--anchor)!important}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style")
    };
    let before = report.clone();
    // One position, five shorthand terminals and one pending anchor member.
    let exact = CssNormalizationLimits::try_new(0, 1, 3, 7).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| {
            if let CssNormalizedItem::Declaration(value) = item {
                Some(value)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (order, (item, source)) in declarations
        .iter()
        .zip(style.declarations().iter())
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert!(item.source().same_occurrence(source));
        assert_eq!(
            item.source().importance(),
            if order == 1 {
                CssImportance::Normal
            } else {
                CssImportance::Important
            }
        );
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), if order == 1 { 5 } else { 1 })
            }
            CssExpansion::Pending(handle) if order == 2 => {
                assert!(handle.source().same_occurrence(source))
            }
            other => panic!("{other:?}"),
        }
    }
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(0, 1, 2, 7).unwrap(),
            CssNormalizationResource::Declarations,
            2,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 3, 6).unwrap(),
            CssNormalizationResource::Contributions,
            6,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
        assert_eq!(report, before);
    }
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

fn named_offset_origin(position: CssPosition, expected_source: &str) {
    let CssPositionRef::NamedFlow(position) = position.view() else {
        panic!("named flow")
    };
    let CssBlockPosition::EndOffset(block) = position.block() else {
        panic!("block offset")
    };
    let CssInlinePosition::StartOffset(inline) = position.inline() else {
        panic!("inline offset")
    };
    for (scalar, text) in [(block, "-2px"), (inline, "10%")] {
        let CssValueOrigin::Parsed(origin) = scalar.origin() else {
            panic!("scalar origin")
        };
        assert_eq!(origin.source().as_str(), expected_source);
        let span = origin.span();
        assert_eq!(
            &expected_source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            text
        );
    }
}

#[test]
fn logical_axis_offsets_retain_checked_parsed_and_replacement_scalar_origins() {
    let input = "inline-start 10% block-end -2px";
    for front in 0..5 {
        let source = declaration("offset-position", input, front);
        let CssKnownPropertyValueRef::OffsetPosition(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("position")
        };
        let CssOffsetPosition::Position(position) = value.value() else {
            panic!("coordinates")
        };
        let expected_source = if front == 0 {
            "/*😀*/offset-position:inline-start 10% block-end -2px!important"
        } else {
            input
        };
        named_offset_origin(full_position(position.clone()), expected_source);
    }
    let source = declaration("offset-anchor", "env(anchor)", 0);
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    let replacement = parse_component_values(input).unwrap();
    let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("anchor")
    };
    let item = &items.items()[0];
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.replacement_components(), Some(&replacement));
    let CssLonghandValueRef::OffsetAnchor(CssOffsetAnchor::Position(position)) =
        item.ordinary_value().unwrap().view()
    else {
        panic!("anchor coordinates")
    };
    named_offset_origin(full_position(position.clone()), input);
}

#[test]
fn composed_symbolic_positions_share_input_projection_and_final_byte_budgets() {
    let source = declaration(
        "offset",
        "x-start ray(.25turn closest-side contain at inline-end block-start) / start end",
        0,
    );
    let value = offset(&source);
    let expected = "x-start center ray(0.25turn contain at block-start inline-end) / start end";
    composed_budget(value, expected);
}

#[test]
fn physical_composition_confirms_the_same_cumulative_budget_control() {
    let source = declaration(
        "offset",
        "left ray(.25turn closest-side contain at right top) / bottom left",
        0,
    );
    composed_budget(
        offset(&source),
        "left center ray(0.25turn contain at right top) / left bottom",
    );
}

fn composed_budget(value: &CssOffset, expected: &str) {
    let snapshot = value.clone();
    // Offset 1 + three positions (aggregate and two axes each) 9 + path pair 1
    // + ray 1 + literal angle 1 + authored default size 1 + contain 1 = 15.
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(15, 15, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(14, 15, expected.len()), K::InputNodeLimit),
        (L::new(15, 14, expected.len()), K::ProjectionNodeLimit),
        (L::new(15, 15, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, &snapshot);
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(value, &snapshot);
}
