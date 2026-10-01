#![forbid(unsafe_code)]
//! Shapes 1 CRD 2025-06-12 §3.1 imports Values 5 WD 2024-11-11
//! §4.2/4.2.1: https://www.w3.org/TR/2024/WD-css-values-5-20241111/#position
//! The independent grammar oracle admits Cartesian and named-flow one/two/four
//! components, and relative-flow two/four components. No writing-mode mapping
//! or coordinate evaluation is implied. New model/serialization APIs are tested
//! separately once available; this target uses existing callable boundaries.

use surgeist_css::*;

fn clip(source: &CssDeclaration) -> &CssClipPath {
    let CssKnownPropertyValueRef::ClipPath(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed clip-path")
    };
    value.value()
}

fn assert_position(value: &CssClipPath) {
    let CssClipPath::BasicShape(shape) = value else {
        panic!("basic shape")
    };
    match shape.shape() {
        CssBasicShape::Circle(circle) => assert!(circle.position().is_some()),
        CssBasicShape::Ellipse(ellipse) => assert!(ellipse.position().is_some()),
        _ => panic!("circle or ellipse"),
    }
}

fn accept(positions: &[&str]) {
    for position in positions {
        for function in ["circle", "ellipse"] {
            let value = format!("{function}(at {position})");
            let source = format!("clip-path:{value}!important");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(
                validate_style_attribute(&source),
                Ok(report.syntax().clone())
            );
            let [declaration] = report.syntax().as_slice() else {
                panic!("one shape declaration")
            };
            assert_position(clip(declaration));
            let CssExpansion::Contributions(CssContributions::Longhands(items)) =
                expand_declaration(declaration).unwrap()
            else {
                panic!("ordinary shape terminal")
            };
            let [item] = items.items() else {
                panic!("one clip-path terminal")
            };
            assert!(item.source().same_occurrence(declaration));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
            let CssLonghandValueRef::ClipPath(expanded) = item.ordinary_value().unwrap().view()
            else {
                panic!("typed shape contribution")
            };
            assert_eq!(expanded, clip(declaration));
            let CssKnownPropertyValueRef::ClipPath(parsed) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("clip-path")
            };
            assert_eq!(parsed.as_css(), value, "authored family and axis order");
            let components = parse_component_values(&value).unwrap();
            let checked = parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
                components.clone(),
                CssImportance::Important,
            )
            .unwrap_or_else(|error| panic!("{value}: {error:?}"));
            assert_eq!(checked.value_components(), &components);
            assert_eq!(clip(&checked), clip(declaration));
            assert_eq!(checked.importance(), CssImportance::Important);
        }
    }
}

#[test]
fn physical_one_two_and_paired_four_component_controls_stay_clean() {
    accept(&[
        "center",
        "left",
        "top",
        "10%",
        "-1e-999px",
        "left top",
        "bottom right",
        "left 10px",
        "10px bottom",
        "-2px 25%",
        "right -2px bottom 10%",
        "top calc(10% - 2px) left -1e-999px",
    ]);
}

#[test]
fn cartesian_axis_relative_one_component_positions_are_clean() {
    accept(&["x-start", "x-end", "y-start", "y-end"]);
}

#[test]
fn cartesian_two_components_allow_axis_reordering_and_physical_mixtures() {
    accept(&[
        "x-start y-end",
        "y-start x-end",
        "x-start bottom",
        "right y-start",
        "center y-end",
        "y-start center",
        "x-end center",
        "center x-start",
        "x-start 10px",
        "-2% y-end",
    ]);
}

#[test]
fn cartesian_four_components_pair_signed_literal_or_math_offsets() {
    accept(&[
        "x-start 10px y-end -2%",
        "y-start -1e-999px x-end 20%",
        "left -2px y-end calc(10% - 1px)",
        "x-end min(1px, 2%) top -3px",
    ]);
}

#[test]
fn named_flow_one_component_positions_are_clean() {
    accept(&["block-start", "block-end", "inline-start", "inline-end"]);
}

#[test]
fn named_flow_two_components_reorder_axes_and_allow_center() {
    accept(&[
        "block-start inline-end",
        "inline-end block-start",
        "block-end inline-start",
        "center block-start",
        "block-end center",
        "center inline-start",
        "inline-end center",
    ]);
}

#[test]
fn named_flow_four_components_pair_symbolic_signed_offsets() {
    accept(&[
        "block-start 10% inline-end -2px",
        "inline-end -2px block-start 10%",
        "block-end -1e-999px inline-start calc(10% - 2px)",
        "inline-start min(1px, 2%) block-end -3px",
    ]);
}

#[test]
fn relative_flow_two_components_preserve_block_then_inline_keywords() {
    accept(&[
        "start start",
        "start center",
        "start end",
        "center start",
        "center end",
        "end start",
        "end center",
        "end end",
        "center center",
    ]);
}

#[test]
fn relative_flow_four_components_pair_offsets_in_block_then_inline_order() {
    accept(&[
        "start 10px end -2%",
        "end -2px start 10%",
        "start -1e-999px start 0",
        "end calc(10% - 2px) end min(1px, 2%)",
    ]);
}

fn reject(position: &str) {
    for function in ["circle", "ellipse"] {
        let prefix = "/*😀*/ ";
        let declaration = format!("clip-path:{function}(at {position});");
        let source = format!("{prefix}{declaration} color:blue");
        let report = parse_style_attribute(&source);
        let [sibling] = report.syntax().as_slice() else {
            panic!("only sibling survives {source}: {:?}", report.syntax())
        };
        assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
        let [diagnostic] = report.diagnostics() else {
            panic!("one diagnostic: {source}")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let end = prefix.len() + declaration.len();
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            prefix.len()
        );
        assert_eq!(diagnostic.span().end().byte_offset().value(), end);
        assert_eq!(
            diagnostic.span().start().column().value() as usize,
            prefix.encode_utf16().count()
        );
        assert_eq!(
            diagnostic.span().end().column().value() as usize,
            source[..end].encode_utf16().count()
        );
        let byte = diagnostic.error().position().byte_offset().value();
        assert!(byte >= prefix.len() + "clip-path:".len() && byte < end);
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            source[..byte].encode_utf16().count()
        );
        assert_eq!(
            validate_style_attribute(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
                parse_component_values(&format!("{function}(at {position})")).unwrap(),
                CssImportance::Normal,
            )
            .is_err()
        );
    }
}

#[test]
fn invalid_arities_duplicate_axes_and_mixed_families_recover_once() {
    for position in [
        "",
        "start",
        "end",
        "left top 1px",
        "left 1px top",
        "center start end",
        "x-start x-end",
        "x-start 1px x-end 2px",
        "y-start 1px bottom 2px",
        "block-start 1px block-end 2px",
        "inline-start 1px inline-end 2px",
        "left 1px block-end 2px",
        "x-start 1px inline-end 2px",
        "start 1px y-end 2px",
        "y-start bottom",
        "block-start block-end",
        "inline-start inline-end",
        "left block-end",
        "x-start inline-end",
        "block-start y-end",
        "start inline-end",
        "end top",
        "block-start 10px",
        "10px inline-end",
        "block-start 10px inline-end",
        "center 1px top 2px",
        "x-start 1px center 2px",
        "block-start 1px center 2px",
        "start 1px center 2px",
        "start 1px end",
        "start 1px 2px end",
        "block-start 1px inline-start 2px block-end",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ] {
        reject(position);
    }
}

#[test]
fn parsed_function_and_offset_origins_address_exact_non_bmp_source() {
    let source =
        "/*😀*/ clip-path:ellipse(at inline-end -1e-999px block-start calc(10% - 2px))!important";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = &report.syntax()[0];
    let component = &declaration.value_components().items()[0];
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("shape function")
    };
    let CssValueOrigin::Parsed(origin) = component.origin() else {
        panic!("parsed function")
    };
    let start = source.find("ellipse").unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
    let offsets: Vec<_> = function
        .values()
        .items()
        .iter()
        .filter(|component| {
            matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. })
                    | CssComponentValueRef::Function(_)
            )
        })
        .collect();
    assert_eq!(offsets.len(), 2);
    for (offset, text) in offsets.into_iter().zip(["-1e-999px", "calc(10% - 2px)"]) {
        let CssValueOrigin::Parsed(origin) = offset.origin() else {
            panic!("parsed offset")
        };
        let start = source.find(text).unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start + text.len()
        );
        assert_eq!(
            origin.span().start().column().value() as usize,
            source[..start].encode_utf16().count()
        );
        assert_eq!(origin.source().as_str(), source);
    }
}

#[test]
fn pending_shape_reentry_is_strict_reusable_and_retains_replacement_origins() {
    let report = parse_style_attribute("/*😀*/ clip-path:var(--shape)!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssExpansion::Pending(pending) = expand_declaration(source).unwrap() else {
        panic!("pending clip-path")
    };
    for invalid in [
        "circle(at start)",
        "ellipse(at left block-end)",
        "circle(at center 1px top 2px)",
    ] {
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("circle(at var(--again))").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for text in [
        "circle(at x-start bottom)",
        "ellipse(at inline-end -2px block-start 10%)",
        "circle(at end start)",
    ] {
        let replacement = parse_component_values(text).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("longhand replacement")
            };
            let [item] = items.items() else {
                panic!("one clip terminal")
            };
            assert_eq!(item.property(), CssKnownProperty::ClipPath);
            assert!(item.source().same_occurrence(source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert_eq!(
                item.replacement_components().unwrap().items()[0].origin(),
                replacement.items()[0].origin()
            );
            let CssLonghandValueRef::ClipPath(value) = item.ordinary_value().unwrap().view() else {
                panic!("typed replacement")
            };
            assert_position(value);
        }
    }
    assert!(pending.source().same_occurrence(source));
}

#[test]
fn normalization_retains_logical_shapes_pending_and_surviving_source_order() {
    let source = ".a{clip-path:circle(at x-start y-end)!important;clip-path:ellipse(at start);clip-path:ellipse(at inline-end block-start);clip-path:var(--shape)}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1, "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (order, (item, text)) in declarations
        .iter()
        .zip([
            "circle(at x-start y-end)",
            "ellipse(at inline-end block-start)",
            "var(--shape)",
        ])
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert_eq!(
            item.source().importance(),
            if order == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            source.find(text).unwrap() - "clip-path:".len()
        );
        if order < 2 {
            let CssExpansion::Contributions(CssContributions::Longhands(items)) = item.expansion()
            else {
                panic!("ordinary terminal")
            };
            let [value] = items.items() else {
                panic!("one terminal")
            };
            assert!(value.source().same_occurrence(item.source()));
            assert!(value.replacement_components().is_none());
            let CssLonghandValueRef::ClipPath(value) = value.ordinary_value().unwrap().view()
            else {
                panic!("clip-path")
            };
            assert_position(value);
        } else {
            let CssExpansion::Pending(pending) = item.expansion() else {
                panic!("pending terminal")
            };
            assert!(pending.source().same_occurrence(item.source()));
        }
    }
}
