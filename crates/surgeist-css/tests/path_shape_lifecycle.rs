#![forbid(unsafe_code)]
//! Shapes 1 CRD 2025-06-12 path() imports SVG 1.1 Second Edition
//! (2011-08-16) paths §8.3.9 and Appendix F.5/F.6.2. These cases assert
//! authored syntax and full consumption, not drawing-prefix recovery or geometry.
//! Signed/zero arc radii follow the adopted F.6.2 interpretation; flags remain
//! single ASCII 0/1 characters. SVG spelling is retained, never float-normalized.

use surgeist_css::*;

fn clip(source: &CssDeclaration) -> &CssClipPath {
    let CssKnownPropertyValueRef::ClipPath(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed clip-path")
    };
    value.value()
}

fn declaration(value: &str) -> CssDeclaration {
    let css = format!("clip-path:{value}!important");
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(validate_style_attribute(&css), Ok(report.syntax().clone()));
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    if let Some(CssKnownPropertyValueRef::ClipPath(parsed)) =
        source.known().unwrap().property_value()
    {
        assert_eq!(parsed.as_css(), value, "authored CSS envelope");
    }
    source.clone()
}

fn accept(authored: &str, expected: &str) {
    let source = declaration(authored);
    assert_eq!(clip(&source).serialize_specified().unwrap(), expected);
    let components = parse_component_values(authored).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &components);
    assert_eq!(checked.importance(), CssImportance::Important);
    assert_eq!(clip(&checked).serialize_specified().unwrap(), expected);
    for declaration in [&source, &checked] {
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(declaration).unwrap()
        else {
            panic!("ordinary path terminal")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::ClipPath);
        assert!(item.source().same_occurrence(declaration));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
        let CssLonghandValueRef::ClipPath(value) = item.ordinary_value().unwrap().view() else {
            panic!("typed path terminal")
        };
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

fn accept_data(data: &str) {
    // These tables use ASCII SVG data without CSS quote/backslash characters.
    accept(&format!("path('{data}')"), &format!("path(\"{data}\")"));
}

#[test]
fn path_admits_each_complete_svg_command_and_repeated_group_in_both_cases() {
    for data in [
        "M0 0 1 2 3 4",
        "m0 0 1 2 3 4",
        "M0 0L1 2 3 4",
        "m0 0l1 2 3 4",
        "M0 0H1 2",
        "m0 0h1 2",
        "M0 0V1 2",
        "m0 0v1 2",
        "M0 0C1 2 3 4 5 6 7 8 9 10 11 12",
        "m0 0c1 2 3 4 5 6 7 8 9 10 11 12",
        "M0 0S1 2 3 4 5 6 7 8",
        "m0 0s1 2 3 4 5 6 7 8",
        "M0 0Q1 2 3 4 5 6 7 8",
        "m0 0q1 2 3 4 5 6 7 8",
        "M0 0T1 2 3 4",
        "m0 0t1 2 3 4",
        "M0 0A1 2 3 0 1 4 5 6 7 8 1 0 9 10",
        "m0 0a1 2 3 0 1 4 5 6 7 8 1 0 9 10",
        "M0 0L1 2Z",
        "m0 0l1 2z",
    ] {
        accept_data(data);
    }
}

#[test]
fn move_only_consecutive_subpaths_and_degenerate_segments_remain_authored() {
    for data in [
        "M0 0",
        "m0 0",
        "M0 0M1 2m3 4",
        "M0 0L0 0Z",
        "M0 0ZzM1 1",
        "M0 0C0 0 0 0 0 0",
        "M0 0A0 0 0 0 0 0 0",
    ] {
        accept_data(data);
    }
}

#[test]
fn maximal_svg_numbers_allow_decimal_sign_and_exponent_adjacency_without_conversion() {
    for data in [
        "M.5.6",
        "M1.2.3",
        "M1. 2.",
        "M1-2L+3+.4",
        "M1e2-3E-4",
        "M1e999999999999999999999 -1e-999999999999999999999",
        "M123456789012345678901234567890.123456789 0",
        "M0 0L1.,2.",
    ] {
        accept_data(data);
    }
}

#[test]
fn arcs_preserve_signed_and_zero_radii_and_single_character_adjacent_flags() {
    for data in [
        "M0 0A-1 +2 0 0 1 3 4",
        "M0 0a0 -1e-999 45 1 0 3 4",
        "M0 0A1 2 0 0110-20",
        "M0 0A1,2,0,0,1,3,4",
    ] {
        accept_data(data);
    }
}

#[test]
fn optional_fill_and_boxes_preserve_omission_and_svg_string_identity() {
    for fill in ["", "nonzero, ", "evenodd, "] {
        let shape = format!("path({fill}'M0,0 L1 2')");
        let expected = format!("path({fill}\"M0,0 L1 2\")");
        accept(&shape, &expected);
        for reference_box in [
            "content-box",
            "padding-box",
            "border-box",
            "margin-box",
            "fill-box",
            "stroke-box",
            "view-box",
        ] {
            let output = format!("{expected} {reference_box}");
            accept(&format!("{reference_box} {shape}"), &output);
            accept(&format!("{shape} {reference_box}"), &output);
        }
    }
}

#[test]
fn css_decoding_and_canonical_quoting_preserve_svg_bytes_and_whitespace() {
    accept(
        "PaTh(EvEnOdD, 'M\\30  0L1 2')",
        "path(evenodd, \"M0 0L1 2\")",
    );
    accept("p\\61 th(\"M0 0\")", "path(\"M0 0\")");
    accept(
        "path(' M0\\9 0\\d L1\\a 2 ')",
        "path(\" M0\\9 0\\d L1\\a 2 \")",
    );
    accept(
        "path(/**/nonzero/**/,/**/'M0 0'/**/)",
        "path(nonzero, \"M0 0\")",
    );
}

fn reject(value: &str) {
    let prefix = "/*😀*/ ";
    let unit = format!("clip-path:{value};");
    let css = format!("{prefix}{unit} color:blue");
    let report = parse_style_attribute(&css);
    let [sibling] = report.syntax().as_slice() else {
        panic!("only color survives {css}: {:?}", report.syntax())
    };
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    let [diagnostic] = report.diagnostics() else {
        panic!("one diagnostic for {css}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let end = prefix.len() + unit.len();
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
        css[..end].encode_utf16().count()
    );
    let position = diagnostic.error().position();
    let byte = position.byte_offset().value();
    assert!(byte >= prefix.len() + "clip-path:".len() && byte < end);
    assert_eq!(
        position.column().value() as usize,
        css[..byte].encode_utf16().count()
    );
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
            parse_component_values(value).unwrap(),
            CssImportance::Normal
        )
        .is_err()
    );
}

#[test]
fn malformed_svg_suffixes_and_incomplete_repeated_groups_reject_the_entire_path() {
    for data in [
        "",
        " ",
        "L0 0",
        "Z",
        "M",
        "M0",
        "M0 0 1",
        "M0 0L1",
        "M0 0L1 2 3",
        "M0 0H",
        "M0 0V",
        "M0 0C1 2 3 4 5",
        "M0 0S1 2 3",
        "M0 0Q1 2 3",
        "M0 0T1",
        "M0 0A1 2 0 0 1 3",
        "M0 0Z1 2",
        "M0 0L1 2X3 4",
        "M0 0r1 2",
        "M0 0L1 2C1 2 3 4 5 6 7",
        "M0 0S1 2 3 4 5",
        "M0 0Q1 2 3 4 5",
        "M0 0T1 2 3",
        "M0 0A1 2 0 0 1 3 4 5",
        "M0 0LNaN 1",
        "M0 0LInfinity 1",
        "M0 0L1px 2",
        "M0 0L1e 2",
        "M0 0L1e+ 2",
        "M0 0L. 2",
        "M0 0L--1 2",
        "M0 0L١ 2",
        "M0\u{a0}0",
        "M0 0L1\u{2003}2",
    ] {
        reject(&format!("path('{data}')"));
    }
    reject("path('\\9 \\a \\d ')");
    reject("path('M0\\c 0')"); // Form feed is not SVG wsp.
}

#[test]
fn malformed_svg_commas_arc_flags_and_required_rotation_separator_are_rejected() {
    for data in [
        "M,0 0",
        "M0,,0",
        "M0 0,",
        "M0 0,L1 2",
        "M0 0L1 2,",
        "M0 0Z,",
        "M0 0A1 2 0 2 1 3 4",
        "M0 0A1 2 0 -1 1 3 4",
        "M0 0A1 2 0 0.0 1 3 4",
        "M0 0A1 2 0 0 1e0 3 4",
        "M0 0A1 2 0+0 1 3 4",
        "M0 0A1 2 001 3 4",
        "M0 0A1 2 0 0,,1 3 4",
        "M0 0A1 2 0 0 1 3 4,",
    ] {
        reject(&format!("path('{data}')"));
    }
}

#[test]
fn malformed_css_path_envelopes_recover_once_without_widening_other_shapes() {
    for value in [
        "path()",
        "path(1)",
        "path(M0 0)",
        "path(, 'M0 0')",
        "path(nonzero 'M0 0')",
        "path(evenodd)",
        "path(nonzero,)",
        "path(winding, 'M0 0')",
        "path('M0 0',)",
        "path('M0 0', 'M1 1')",
        "path(evenodd, nonzero, 'M0 0')",
        "path('M0 0' trailing)",
        "path(border-box 'M0 0')",
        "path('M0 0' border-box)",
        "path('M0 0') border-box fill-box",
        "path('M0 0') circle()",
        "none path('M0 0')",
    ] {
        reject(value);
    }
}

fn strict_recovered(value: &str, expected: &str) {
    let css = format!("clip-path:{value}");
    let report = parse_style_attribute(&css);
    assert_eq!(
        report.syntax().len(),
        1,
        "ordinary recovery retains {css}: {:?}",
        report.diagnostics()
    );
    assert!(!report.is_clean());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    assert!(validate_style_attribute(&css).is_err());
    assert_eq!(
        clip(&report.syntax()[0]).serialize_specified().unwrap(),
        expected
    );
    let recovered = parse_component_values(value).unwrap();
    let Err(error) = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
        recovered.clone(),
        CssImportance::Normal,
    ) else {
        panic!("checked clip-path must reject recovered components: {value}")
    };
    let origin = match error.origin() {
        CssSerializedOrigin::End(Some(origin)) | CssSerializedOrigin::Token(origin) => origin,
        other => panic!("implicit termination origin, got {other:?}"),
    };
    let CssValueOrigin::ImplicitClosure { at, .. } = origin else {
        panic!("implicit EOF origin")
    };
    assert_eq!(at.source().as_str(), value);
    assert_eq!(at.span().start().byte_offset().value(), value.len());
    assert_eq!(at.span().end().byte_offset().value(), value.len());
    let source = declaration("var(--clip)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending clip")
    };
    assert!(matches!(
        pending.reenter(recovered).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
}

#[test]
fn recovered_path_function_and_string_remain_parse_only_admissions() {
    for value in ["path('M0 0'", "path('M0 0", "path('M0 0\\"] {
        strict_recovered(value, "path(\"M0 0\")");
    }
}

#[test]
fn recovered_circle_remains_parse_only_and_checked_reentry_rejects_it() {
    strict_recovered("circle(1px", "circle(1px)");
}

#[test]
fn recovered_rect_remains_parse_only_and_checked_reentry_rejects_it() {
    strict_recovered("rect(0 1px 2% 3px", "rect(0 1px 2% 3px)");
}

#[test]
fn path_function_string_and_closing_origins_retain_original_escaped_source() {
    let css = "/*😀*/ clip-path:PaTh(evenodd, 'M\\30  0L1 2')!important";
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let component = &report.syntax()[0].value_components().items()[0];
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("path function")
    };
    assert_origin(
        component.origin(),
        css,
        css.find("PaTh(").unwrap(),
        "PaTh(".len(),
    );
    assert_origin(function.closing_origin(), css, css.rfind(')').unwrap(), 1);
    let string = function
        .values()
        .items()
        .iter()
        .find(|item| {
            matches!(
                item.view(),
                CssComponentValueRef::Token(CssValueTokenRef::String(_))
            )
        })
        .unwrap();
    assert!(matches!(
        string.view(),
        CssComponentValueRef::Token(CssValueTokenRef::String("M0 0L1 2"))
    ));
    assert_origin(
        string.origin(),
        css,
        css.find("'M").unwrap(),
        "'M\\30  0L1 2'".len(),
    );
}

fn assert_origin(origin: &CssValueOrigin, source: &str, start: usize, length: usize) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("parsed source")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), start + length);
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
    assert_eq!(
        origin.span().end().column().value() as usize,
        source[..start + length].encode_utf16().count()
    );
}

#[test]
fn pending_path_reentry_is_strict_reusable_and_preserves_authored_replacement_origins() {
    let source = declaration("var(--path)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending path")
    };
    for invalid in [
        "path('M0 0L1')",
        "path('M0 0A1 2 0 2 1 3 4')",
        "path(nonzero 'M0 0')",
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
            .reenter(parse_component_values("path(var(--again))").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for (authored, expected) in [
        ("path('M0 0')", "path(\"M0 0\")"),
        (
            "border-box path(evenodd, 'M0,0 L1-2')",
            "path(evenodd, \"M0,0 L1-2\") border-box",
        ),
    ] {
        let replacement = parse_component_values(authored).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("longhand replacement")
            };
            let [item] = items.items() else {
                panic!("one replacement")
            };
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let CssLonghandValueRef::ClipPath(value) = item.ordinary_value().unwrap().view() else {
                panic!("typed clip replacement")
            };
            assert_eq!(value.serialize_specified().unwrap(), expected);
        }
    }
    assert!(pending.source().same_occurrence(&source));
}

#[test]
fn normalization_retains_path_family_fill_importance_and_source_order_after_recovery() {
    let css = ".a{clip-path:path('M0 0')!important;clip-path:path('M0 0L1');clip-path:border-box path(nonzero, 'M1-2Z');clip-path:var(--path)}";
    let report = parse_sheet(css);
    assert_eq!(report.diagnostics().len(), 1, "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let items: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(item) => Some(item),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 3);
    for (order, (item, authored)) in items
        .iter()
        .zip([
            "path('M0 0')",
            "border-box path(nonzero, 'M1-2Z')",
            "var(--path)",
        ])
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            css.find(authored).unwrap() - "clip-path:".len()
        );
        assert_eq!(
            item.source().importance(),
            if order == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        if order == 2 {
            let CssExpansion::Pending(pending) = item.expansion() else {
                panic!("pending terminal")
            };
            assert!(pending.source().same_occurrence(item.source()));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("ordinary terminal")
            };
            let [value] = values.items() else {
                panic!("one path terminal")
            };
            assert!(value.source().same_occurrence(item.source()));
            assert!(value.replacement_components().is_none());
            let CssLonghandValueRef::ClipPath(value) = value.ordinary_value().unwrap().view()
            else {
                panic!("path terminal")
            };
            assert_eq!(
                value.serialize_specified().unwrap(),
                if order == 0 {
                    "path(\"M0 0\")"
                } else {
                    "path(nonzero, \"M1-2Z\") border-box"
                }
            );
        }
    }
}

#[test]
fn existing_clip_serializer_charges_path_string_fill_and_box_cumulatively() {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    for (authored, expected, nodes) in [
        ("path('M0 0')", "path(\"M0 0\")", 3),
        ("path(nonzero, 'M0 0')", "path(nonzero, \"M0 0\")", 4),
        (
            "path(evenodd, 'M0 0') border-box",
            "path(evenodd, \"M0 0\") border-box",
            5,
        ),
    ] {
        let source = declaration(authored);
        let before = source.clone();
        let value = clip(&source);
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(nodes, nodes, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (L::new(nodes - 1, nodes, expected.len()), K::InputNodeLimit),
            (
                L::new(nodes, nodes - 1, expected.len()),
                K::ProjectionNodeLimit,
            ),
            (L::new(nodes, nodes, expected.len() - 1), K::ByteLimit),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(source, before);
    }
}

#[test]
fn pending_clip_reentry_rejects_recovered_circle_and_rect_before_canonical_closure() {
    let source = declaration("var(--clip)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending clip")
    };
    for value in ["circle(1px", "rect(0 1px 2% 3px"] {
        let Err(error) = pending.reenter(parse_component_values(value).unwrap()) else {
            panic!("pending reentry must reject recovered replacement: {value}")
        };
        assert!(matches!(
            error.kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
}
