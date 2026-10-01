#![forbid(unsafe_code)]
//! Authored shape() grammar: Shapes 1 CRD 2025-06-12 §3.1–3.3 and
//! Values 5 WD 2024-11-11 §4.2. The required first comma is the individually
//! adopted exception to SH1's printed outer production, corroborated by its
//! examples and WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d,
//! CSSPropertyParserConsumer+Shapes.cpp:809–837. Full positions follow V5;
//! To/By control affinity and omitted anchors retain the adopted SH1/WK meaning.
//! Arc options follow SH1's complete earlier && production, including signed
//! radii and a strict shared angle. These expectations concern authored values,
//! never geometry, computed defaults or unfinished exact math projection.
//! All tests use callable public APIs that predate shape() support.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

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
        assert_eq!(parsed.as_css(), value, "retain authored spelling and order");
    }
    let sheet = format!(".a{{{css}}}");
    let report = parse_sheet(&sheet);
    assert!(report.is_clean(), "{sheet}: {:?}", report.diagnostics());
    assert_eq!(validate_sheet(&sheet), Ok(report.syntax().clone()));
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
    .unwrap_or_else(|error| panic!("{authored}: {error:?}"));
    assert_eq!(checked.value_components(), &components);
    assert_eq!(checked.importance(), CssImportance::Important);
    assert_eq!(clip(&checked).serialize_specified().unwrap(), expected);
    for declaration in [&source, &checked] {
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(declaration).unwrap()
        else {
            panic!("terminal shape contribution")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::ClipPath);
        assert!(item.source().same_occurrence(declaration));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
        let CssLonghandValueRef::ClipPath(value) = item.ordinary_value().unwrap().view() else {
            panic!("typed shape terminal")
        };
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

fn accept_command(command: &str) {
    let text = format!("shape(from 0px 0px, {command})");
    accept(&text, &text);
}

#[test]
fn move_and_line_retain_full_absolute_endpoints_and_signed_relative_pairs() {
    for command in [
        "move to 1px 2%",
        "move by -1px -2%",
        "line to right 1px bottom 2%",
        "line by 0 -2em",
    ] {
        accept_command(command);
    }
}

#[test]
fn horizontal_and_vertical_lines_admit_only_their_axis_keywords_or_one_offset() {
    for keyword in ["left", "center", "right", "x-start", "x-end"] {
        accept_command(&format!("hline to {keyword}"));
    }
    for keyword in ["top", "center", "bottom", "y-start", "y-end"] {
        accept_command(&format!("vline to {keyword}"));
    }
    for command in [
        "hline to -1px",
        "hline by -2%",
        "vline to 0",
        "vline by -3em",
    ] {
        accept_command(command);
    }
}

#[test]
fn curves_retain_one_or_two_controls_and_independent_optional_anchors() {
    for affinity in ["to", "by"] {
        for first in [
            "1px 2%",
            "1px 2% from start",
            "1px 2% from end",
            "1px 2% from origin",
        ] {
            accept_command(&format!("curve {affinity} -3px 4% with {first}"));
            for second in [
                "-5px 6%",
                "-5px 6% from start",
                "-5px 6% from end",
                "-5px 6% from origin",
            ] {
                accept_command(&format!("curve {affinity} -3px 4% with {first} / {second}"));
            }
        }
    }
}

#[test]
fn smooth_commands_retain_omitted_or_single_control_for_both_affinities() {
    for affinity in ["to", "by"] {
        accept_command(&format!("smooth {affinity} -3px 4%"));
        for control in [
            "1px 2%",
            "1px 2% from start",
            "1px 2% from end",
            "1px 2% from origin",
        ] {
            accept_command(&format!("smooth {affinity} -3px 4% with {control}"));
        }
    }
}

#[test]
fn close_only_move_only_and_repeated_commands_preserve_authored_order() {
    for commands in [
        "close",
        "move to 1px 2%",
        "close, close",
        "line by 1px 2%, move by -3px 4%, hline to x-end, vline by -5px, close",
        "smooth by 0 0, curve to 1px 2px with 3px 4px, close, move to 5px 6px",
    ] {
        accept_command(commands);
    }
}

#[test]
fn full_position_families_apply_to_starts_endpoints_and_absolute_controls() {
    // Expected order and implied centers are specified by V5 §4.2.2.
    for (position, canonical) in [
        ("left", "left center"),
        ("top", "center top"),
        ("25%", "25% center"),
        ("center", "center center"),
        ("bottom right", "right bottom"),
        ("-1px 2%", "-1px 2%"),
        ("y-end x-start", "x-start y-end"),
        ("x-end -2%", "x-end -2%"),
        ("y-start 2% x-end -1px", "x-end -1px y-start 2%"),
        ("block-start", "block-start center"),
        ("inline-end", "center inline-end"),
        ("inline-end block-start", "block-start inline-end"),
        (
            "inline-end -2px block-start 10%",
            "block-start 10% inline-end -2px",
        ),
        ("start end", "start end"),
        ("end -2px start 10%", "end -2px start 10%"),
    ] {
        accept(
            &format!("shape(from {position}, close)"),
            &format!("shape(from {canonical}, close)"),
        );
        for verb in ["move", "line", "smooth", "arc"] {
            let options = if verb == "arc" { " of 1px" } else { "" };
            accept(
                &format!("shape(from 0px 0px, {verb} to {position}{options})"),
                &format!("shape(from 0px 0px, {verb} to {canonical}{options})"),
            );
        }
        accept(
            &format!("shape(from 0px 0px, curve to {position} with {position} / 1px 2% from end)"),
            &format!(
                "shape(from 0px 0px, curve to {canonical} with {canonical} / 1px 2% from end)"
            ),
        );
        accept(
            &format!("shape(from 0px 0px, smooth to 1px 2% with {position})"),
            &format!("shape(from 0px 0px, smooth to 1px 2% with {canonical})"),
        );
    }
}

#[test]
fn arcs_retain_signed_zero_and_one_or_two_radii_without_inserting_options() {
    for affinity in ["to", "by"] {
        for radii in ["-1px", "-2%", "0", "0px", "-1px -2%", "1px 1px", "0 0"] {
            accept_command(&format!("arc {affinity} -3px 4% of {radii}"));
        }
        for options in [
            "cw",
            "ccw",
            "large",
            "small",
            "rotate 0deg",
            "cw small",
            "ccw large rotate -45deg",
        ] {
            accept_command(&format!("arc {affinity} -3px 4% of -1px {options}"));
        }
    }
}

#[test]
fn every_arc_option_permutation_serializes_in_the_complete_grammar_order() {
    let groups = ["of -1px 2%", "cw", "large", "rotate -30deg"];
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    if a == b || a == c || a == d || b == c || b == d || c == d {
                        continue;
                    }
                    for affinity in ["to", "by"] {
                        accept(
                            &format!(
                                "shape(from 0px 0px, arc {affinity} 3px 4% {} {} {} {})",
                                groups[a], groups[b], groups[c], groups[d]
                            ),
                            &format!(
                                "shape(from 0px 0px, arc {affinity} 3px 4% of -1px 2% cw large rotate -30deg)"
                            ),
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn literal_coefficients_retain_raw_precision_and_units_while_output_rounds() {
    for unit in ["deg", "grad", "rad", "turn"] {
        accept(
            &format!(
                "shape(from 0px 0px, arc to 1px 2% of -3px rotate -0.12345678901234567890123456789{unit})"
            ),
            &format!("shape(from 0px 0px, arc to 1px 2% of -3px rotate -0.123457{unit})"),
        );
    }
    let huge = format!("1{}", "0".repeat(400));
    accept(
        "shape(from -1e-999px 2%, arc by -3px 4% of -1e400px rotate -1e-999deg)",
        &format!("shape(from 0px 2%, arc by -3px 4% of -{huge}px rotate 0deg)"),
    );
    accept(
        "shape(from +1px -0px, arc to 2px 3% of +4px rotate +0deg)",
        "shape(from 1px 0px, arc to 2px 3% of 4px rotate 0deg)",
    );
}

#[test]
fn symbolic_position_pair_radius_and_angle_graphs_remain_authored() {
    // No exact numerical projection oracle is claimed for unfinished math work.
    let authored = "shape(from calc(1px + 2%) 0px, curve by min(-1px, 2%) 3px with calc(4px + 5%) 6px from end, arc to 7px 8% of calc(-9px) min(10px, 11%) rotate calc(12deg + 13deg))";
    let source = declaration(authored);
    let components = parse_component_values(authored).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &components);
    assert_eq!(clip(&source), clip(&checked));
    let before = source.clone();
    assert!(matches!(
        expand_declaration(&source).unwrap(),
        CssExpansion::Contributions(CssContributions::Longhands(_))
    ));
    assert_eq!(source, before);
}

#[test]
fn fill_omission_and_both_reference_box_orders_retain_authored_identity() {
    let omitted = declaration("shape(from 0px 0px, close)");
    for fill in ["", "nonzero ", "evenodd "] {
        let shape = format!("shape({fill}from 0px 0px, close)");
        accept(&shape, &shape);
        if !fill.is_empty() {
            assert_ne!(clip(&declaration(&shape)), clip(&omitted));
        }
        for edge in [
            "content-box",
            "padding-box",
            "border-box",
            "margin-box",
            "fill-box",
            "stroke-box",
            "view-box",
        ] {
            let expected = format!("{shape} {edge}");
            accept(&expected, &expected);
            accept(&format!("{edge} {shape}"), &expected);
        }
    }
}

#[test]
fn comments_adjacent_percentages_case_and_escapes_serialize_canonically() {
    accept(
        "SHAPE(EVENODD FROM 0px 0px,CURVE BY 1%2% WITH 3%4% FROM END/5%6% FROM ORIGIN,CLOSE)",
        "shape(evenodd from 0px 0px, curve by 1% 2% with 3% 4% from end / 5% 6% from origin, close)",
    );
    accept(
        "sh\\61 pe(/**/from/**/0px/**/0px/**/,/**/line/**/by/**/1%2%/**/,/**/close/**/)",
        "shape(from 0px 0px, line by 1% 2%, close)",
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
    let sheet = format!(".a{{{css}}}");
    let report = parse_sheet(&sheet);
    assert_eq!(report.diagnostics().len(), 1, "{sheet}");
    assert_eq!(
        validate_sheet(&sheet).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn missing_repeated_and_trailing_commas_or_empty_command_lists_reject() {
    for value in [
        "shape()",
        "shape(from 0px 0px)",
        "shape(from 0px 0px,)",
        "shape(from 0px 0px close)",
        "shape(from 0px 0px,, close)",
        "shape(from 0px 0px, close,)",
        "shape(from 0px 0px, close,, close)",
        "shape(from 0px 0px, line to 1px 2px close)",
        "shape(evenodd, from 0px 0px, close)",
        "shape(from 0px 0px, unknown)",
        "shape(from 0px 0px, close to 1px 2px)",
    ] {
        reject(value);
    }
}

#[test]
fn whole_bounded_position_regions_and_relative_pair_arity_are_validated() {
    for region in [
        "",
        "left right",
        "left top 1px",
        "left 1px top",
        "1px 2px 3px",
        "left 1px top 2px 3px",
        "block-start left",
        "start",
        "start end 1px",
    ] {
        reject(&format!("shape(from {region}, close)"));
        reject(&format!("shape(from 0px 0px, line to {region})"));
        reject(&format!(
            "shape(from 0px 0px, curve to {region} with 1px 2px)"
        ));
        reject(&format!(
            "shape(from 0px 0px, curve to 1px 2px with {region})"
        ));
        reject(&format!("shape(from 0px 0px, arc to {region} of 1px)"));
    }
    for pair in ["", "1px", "1px 2px 3px", "left top", "start end", "1px 2"] {
        for verb in ["move", "line", "smooth", "arc"] {
            let options = if verb == "arc" { " of 1px" } else { "" };
            reject(&format!("shape(from 0px 0px, {verb} by {pair}{options})"));
        }
    }
    for command in [
        "hline to top",
        "hline by left",
        "hline to 1px 2px",
        "vline to right",
        "vline by bottom",
        "vline to 1px 2px",
    ] {
        reject(&format!("shape(from 0px 0px, {command})"));
    }
}

#[test]
fn controls_require_their_affinity_arity_separator_and_numeric_anchor_pair() {
    for command in [
        "curve to 1px 2px",
        "curve by 1px 2px",
        "curve by 1px 2px with",
        "curve by 1px 2px with left top",
        "curve to 1px 2px with left top from start",
        "curve to 1px 2px with 1px 2px from",
        "curve by 1px 2px with 1px 2px from bogus",
        "curve by 1px 2px with 1px 2px from start from end",
        "curve to 1px 2px with 1px 2px /",
        "curve to 1px 2px with / 1px 2px",
        "curve to 1px 2px with 1px 2px / / 3px 4px",
        "curve to 1px 2px with 1px 2px / 3px 4px / 5px 6px",
        "smooth by 1px 2px with",
        "smooth by 1px 2px with start end",
        "smooth to 1px 2px with 3px 4px / 5px 6px",
        "curve to 1px 2px with right 1px bottom 2px from origin",
        "curve to 1px 2px with block-start inline-end from end",
    ] {
        reject(&format!("shape(from 0px 0px, {command})"));
    }
}

#[test]
fn arcs_require_radii_once_and_reject_duplicate_options_or_non_angles() {
    for options in [
        "",
        "cw",
        "of",
        "of 1px 2px 3px",
        "of 1px of 2px",
        "of 1px cw cw",
        "of 1px cw ccw",
        "of 1px large small",
        "of 1px small small",
        "of 1px rotate",
        "of 1px rotate 1deg rotate 2deg",
        "of 1px unknown",
        "of 1px rotate 0",
        "of 1px rotate -0",
        "of 1px rotate 1px",
        "of 1px rotate 10%",
    ] {
        reject(&format!("shape(from 0px 0px, arc to 1px 2% {options})"));
    }
}

fn assert_origin(origin: &CssValueOrigin, css: &str, start: usize, length: usize) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("parsed source origin")
    };
    assert_eq!(origin.source().as_str(), css);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), start + length);
    assert_eq!(
        origin.span().start().column().value() as usize,
        css[..start].encode_utf16().count()
    );
    assert_eq!(
        origin.span().end().column().value() as usize,
        css[..start + length].encode_utf16().count()
    );
}

#[test]
fn original_function_command_keyword_scalar_and_punctuation_origins_survive() {
    let css = "/*😀*/ clip-path:ShApE(evenodd from -1e-999px 2%, curve by 3px 4% with 5px 6% from end / 7px 8% from origin, arc to x-end y-start of -9px rotate 10deg, close)!important";
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let component = &report.syntax()[0].value_components().items()[0];
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("shape function")
    };
    assert_origin(component.origin(), css, css.find("ShApE(").unwrap(), 6);
    assert_origin(function.closing_origin(), css, css.rfind(')').unwrap(), 1);
    for text in [
        "evenodd",
        "from",
        "-1e-999px",
        "2%",
        ",",
        "curve",
        "by",
        "3px",
        "with",
        "end",
        "/",
        "origin",
        "arc",
        "to",
        "x-end",
        "y-start",
        "of",
        "-9px",
        "rotate",
        "10deg",
        "close",
    ] {
        let function_start = css.find("ShApE(").unwrap();
        let start = function_start + css[function_start..].find(text).unwrap();
        let token = function
            .values()
            .items()
            .iter()
            .find(|item| match item.origin() {
                CssValueOrigin::Parsed(origin) => {
                    origin.span().start().byte_offset().value() == start
                }
                _ => false,
            })
            .unwrap_or_else(|| panic!("original {text}"));
        assert_origin(token.origin(), css, start, text.len());
    }
}

#[test]
fn aggregate_equality_ignores_raw_origins_but_retains_order_affinity_and_omissions() {
    let text = "shape(from 0px 0px, curve to 1px 2px with 3px 4px, arc by 5px 6px of 7px)";
    let a = declaration(text);
    let b = parse_style_attribute(&format!("/*different source*/ clip-path:{text}!important"));
    assert!(b.is_clean());
    assert_ne!(a.value_components(), b.syntax()[0].value_components());
    assert_eq!(clip(&a), clip(&b.syntax()[0]));
    for other in [
        "shape(nonzero from 0px 0px, curve to 1px 2px with 3px 4px, arc by 5px 6px of 7px)",
        "shape(from 0px 0px, curve by 1px 2px with 3px 4px, arc by 5px 6px of 7px)",
        "shape(from 0px 0px, curve to 1px 2px with 3px 4px from origin, arc by 5px 6px of 7px)",
        "shape(from 0px 0px, curve to 1px 2px with 3px 4px / 3px 4px, arc by 5px 6px of 7px)",
        "shape(from 0px 0px, curve to 1px 2px with 3px 4px, arc by 5px 6px of 7px 7px)",
        "shape(from 0px 0px, curve to 1px 2px with 3px 4px, arc by 5px 6px of 7px ccw small rotate 0deg)",
        "shape(from 0px 0px, arc by 5px 6px of 7px, curve to 1px 2px with 3px 4px)",
    ] {
        assert_ne!(clip(&a), clip(&declaration(other)), "{other}");
    }
    let a = declaration("shape(from 0px 0px, arc to 1px 2px of 3px cw large rotate 4deg)");
    let b = declaration("shape(from 0px 0px, arc to 1px 2px rotate 4deg large cw of 3px)");
    assert_ne!(a.value_components(), b.value_components());
    assert_eq!(clip(&a), clip(&b));
}

#[test]
fn diagnosed_implicit_closure_is_retained_but_original_components_remain_strict() {
    let value = "shape(from 0px 0px, close";
    let css = format!("clip-path:{value}");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 1, "{:?}", report.diagnostics());
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
        "shape(from 0px 0px, close)"
    );
    let components = parse_component_values(value).unwrap();
    let error = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
        components.clone(),
        CssImportance::Normal,
    )
    .unwrap_err();
    let origin = match error.origin() {
        CssSerializedOrigin::End(Some(origin)) | CssSerializedOrigin::Token(origin) => origin,
        other => panic!("implicit closing origin: {other:?}"),
    };
    let CssValueOrigin::ImplicitClosure { at, .. } = origin else {
        panic!("implicit EOF")
    };
    assert_eq!(at.source().as_str(), value);
    assert_eq!(at.span().start().byte_offset().value(), value.len());
    assert_eq!(at.span().end().byte_offset().value(), value.len());
    let source = declaration("var(--shape)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending clip")
    };
    assert!(matches!(
        pending.reenter(components).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
}

#[test]
fn checked_property_construction_accepts_programmatic_original_component_graphs() {
    let tokens = [
        "from", "0px", "0px", ",", "line", "by", "-1px", "2%", ",", "close",
    ]
    .into_iter()
    .map(|token| CssComponentValue::try_token(token).unwrap())
    .collect();
    let function =
        CssComponentValue::try_function("shape", CssComponentValues::try_new(tokens).unwrap())
            .unwrap();
    let components = CssComponentValues::try_new(vec![function]).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &components);
    assert_eq!(
        components.items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(
        clip(&declaration).serialize_specified().unwrap(),
        "shape(from 0px 0px, line by -1px 2%, close)"
    );
}

#[test]
fn pending_reentry_is_reusable_and_returns_only_validated_terminal_replacements() {
    let source = declaration("var(--shape)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending shape")
    };
    for invalid in [
        "shape(from 0px 0px close)",
        "shape(from 0px 0px, arc by 1px 2px)",
        "shape(from 0px 0px, close",
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
            .reenter(parse_component_values("shape(from var(--again), close)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for (authored, expected) in [
        ("shape(from 0px 0px, close)", "shape(from 0px 0px, close)"),
        (
            "border-box shape(evenodd from start end, line by -1px 2%)",
            "shape(evenodd from start end, line by -1px 2%) border-box",
        ),
    ] {
        let replacement = parse_component_values(authored).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed replacement")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let CssLonghandValueRef::ClipPath(value) = item.ordinary_value().unwrap().view() else {
                panic!("shape terminal")
            };
            assert_eq!(value.serialize_specified().unwrap(), expected);
        }
    }
    assert!(pending.source().same_occurrence(&source));
}

#[test]
fn normalization_preserves_surviving_occurrences_importance_and_pending_state() {
    let css = ".a{clip-path:shape(from 0px 0px, close)!important;clip-path:shape(from 0px 0px close);clip-path:border-box shape(nonzero from start end, line by -1px 2%);clip-path:inherit;clip-path:var(--shape)}";
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
    assert_eq!(items.len(), 4);
    for (order, (item, authored)) in items
        .iter()
        .zip([
            "shape(from 0px 0px, close)",
            "border-box shape(nonzero from start end, line by -1px 2%)",
            "inherit",
            "var(--shape)",
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
        if order == 3 {
            let CssExpansion::Pending(pending) = item.expansion() else {
                panic!("pending occurrence")
            };
            assert!(pending.source().same_occurrence(item.source()));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("completed occurrence")
            };
            let [value] = values.items() else {
                panic!("one terminal")
            };
            assert!(value.source().same_occurrence(item.source()));
            assert!(value.replacement_components().is_none());
            if order == 2 {
                assert_eq!(
                    value.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
            } else {
                let CssLonghandValueRef::ClipPath(value) = value.ordinary_value().unwrap().view()
                else {
                    panic!("typed occurrence")
                };
                assert_eq!(
                    value.serialize_specified().unwrap(),
                    if order == 0 {
                        "shape(from 0px 0px, close)"
                    } else {
                        "shape(nonzero from start end, line by -1px 2%) border-box"
                    }
                );
            }
        }
    }
}

#[test]
fn close_fixture_obeys_atomic_cumulative_limits_without_mutating_values_or_origins() {
    // Independent contract: function + from + position/2 axes/2 scalars + list
    // + close = 9 visits. ClipPathShape adds 1, an explicit reference box adds 1.
    let source = declaration("shape(from 0px 0px, close)");
    let before = source.clone();
    let CssClipPath::BasicShape(composition) = clip(&source) else {
        panic!("basic shape")
    };
    let basic = composition.shape();
    let expected = "shape(from 0px 0px, close)";
    assert_eq!(expected.len(), 26);
    assert_eq!(
        basic
            .serialize_specified_with_limits(L::new(9, 9, 26))
            .unwrap(),
        expected
    );
    assert_eq!(
        composition
            .serialize_specified_with_limits(L::new(10, 10, 26))
            .unwrap(),
        expected
    );
    assert_eq!(
        clip(&source)
            .serialize_specified_with_limits(L::new(10, 10, 26))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(8, 9, 26), K::InputNodeLimit),
        (L::new(9, 8, 26), K::ProjectionNodeLimit),
        (L::new(9, 9, 25), K::ByteLimit),
    ] {
        assert_eq!(
            basic
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    for (limits, kind) in [
        (L::new(9, 10, 26), K::InputNodeLimit),
        (L::new(10, 9, 26), K::ProjectionNodeLimit),
        (L::new(10, 10, 25), K::ByteLimit),
    ] {
        assert_eq!(
            clip(&source)
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let boxed = CssClipPathShape::new(basic.clone(), Some(CssBoxEdgeKeyword::BorderBox));
    let expected_boxed = "shape(from 0px 0px, close) border-box";
    assert_eq!(
        boxed
            .serialize_specified_with_limits(L::new(11, 11, expected_boxed.len()))
            .unwrap(),
        expected_boxed
    );
    assert_eq!(basic.serialize_specified().unwrap(), expected);
    assert_eq!(source, before);
}

// Functional evidence for APIs introduced with shape support. These constructor
// tests have no preimplementation RED; their expected states and counts follow
// the authored grammar and independently enumerated semantic visits.
mod construction {
    use super::*;
    use std::error::Error;

    fn lp(text: &str) -> CssSpecifiedLengthPercentage {
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap()
    }
    fn pair() -> CssShapeCoordinatePair {
        CssShapeCoordinatePair::new(lp("1px"), lp("2%"))
    }
    fn position() -> CssPosition {
        CssPosition::from_cartesian(
            CssCartesianPosition::try_new(
                CssHorizontalPosition::Offset(lp("0px")),
                CssVerticalPosition::Offset(lp("0px")),
            )
            .unwrap(),
        )
    }
    fn angle(text: &str) -> CssAngleValue {
        CssAngleValue::from_literal(
            CssAngleLiteral::try_from_component(CssComponentValue::try_token(text).unwrap())
                .unwrap(),
        )
    }
    fn shape(commands: Vec<CssShapeCommand>) -> CssShapeFunction {
        CssShapeFunction::new(
            None,
            position(),
            CssShapeCommandList::try_new(commands).unwrap(),
        )
    }
    fn absolute(anchor: Option<CssShapeControlAnchor>) -> CssShapeAbsoluteControlPoint {
        CssShapeAbsoluteControlPoint::from_coordinates(pair(), anchor)
    }
    fn relative(anchor: Option<CssShapeControlAnchor>) -> CssShapeRelativeControlPoint {
        CssShapeRelativeControlPoint::new(pair(), anchor)
    }

    #[test]
    fn empty_list_returns_only_the_typed_construction_error() {
        let error = CssShapeCommandList::try_new(vec![]).unwrap_err();
        assert_eq!(error, CssShapeConstructionError::EmptyCommands);
        assert!(error.source().is_none());
        assert_eq!(
            CssShapeCommandList::try_new(vec![CssShapeCommand::Close])
                .unwrap()
                .commands(),
            &[CssShapeCommand::Close]
        );
    }
    #[test]
    fn every_command_variant_composes_checked_children_without_defaults() {
        let commands = vec![
            CssShapeCommand::Move(CssShapeEndpoint::To(position())),
            CssShapeCommand::Line(CssShapeEndpoint::By(pair())),
            CssShapeCommand::HorizontalLine(CssShapeHorizontalLine::ToKeyword(
                CssHorizontalPositionKeyword::XStart,
            )),
            CssShapeCommand::VerticalLine(CssShapeVerticalLine::By(lp("-3px"))),
            CssShapeCommand::Curve(CssShapeCurve::to(position(), absolute(None), None)),
            CssShapeCommand::Smooth(CssShapeSmooth::by(pair(), None)),
            CssShapeCommand::Arc(CssShapeArc::new(
                CssShapeEndpoint::By(pair()),
                CssShapeArcRadii::One(lp("-4px")),
                None,
                None,
                None,
            )),
            CssShapeCommand::Close,
        ];
        let value = shape(commands.clone());
        assert_eq!(value.commands().commands(), commands);
        assert_eq!(value.fill_rule(), None);
        assert_eq!(value.start(), &position());
        assert_eq!(
            value.serialize_specified().unwrap(),
            "shape(from 0px 0px, move to 0px 0px, line by 1px 2%, hline to x-start, vline by -3px, curve to 0px 0px with 1px 2%, smooth by 1px 2%, arc by 1px 2% of -4px, close)"
        );
    }
    #[test]
    fn curve_views_keep_endpoint_and_control_affinities_coupled() {
        let first = absolute(Some(CssShapeControlAnchor::Start));
        let second = absolute(Some(CssShapeControlAnchor::End));
        let value = CssShapeCurve::to(position(), first.clone(), Some(second.clone()));
        let CssShapeCurveRef::To {
            end,
            first: a,
            second: b,
        } = value.view()
        else {
            panic!("absolute controls")
        };
        assert_eq!(end, &position());
        assert_eq!(a, &first);
        assert_eq!(b, Some(&second));
        let first = relative(None);
        let second = relative(Some(CssShapeControlAnchor::Origin));
        let value = CssShapeCurve::by(pair(), first.clone(), Some(second.clone()));
        let CssShapeCurveRef::By {
            end,
            first: a,
            second: b,
        } = value.view()
        else {
            panic!("relative controls")
        };
        assert_eq!(end, &pair());
        assert_eq!(a, &first);
        assert_eq!(b, Some(&second));
        assert_eq!(a.anchor(), None);
        assert_eq!(b.unwrap().anchor(), Some(CssShapeControlAnchor::Origin));
    }
    #[test]
    fn smooth_views_distinguish_both_affinities_and_control_omission() {
        let to = CssShapeSmooth::to(position(), None);
        assert!(matches!(
            to.view(),
            CssShapeSmoothRef::To { control: None, .. }
        ));
        let by = CssShapeSmooth::by(pair(), None);
        assert!(matches!(
            by.view(),
            CssShapeSmoothRef::By { control: None, .. }
        ));
        assert_ne!(to, CssShapeSmooth::to(position(), Some(absolute(None))));
        assert_ne!(by, CssShapeSmooth::by(pair(), Some(relative(None))));
        assert!(matches!(
            CssShapeSmooth::to(position(), Some(absolute(None))).view(),
            CssShapeSmoothRef::To {
                control: Some(_),
                ..
            }
        ));
        assert!(matches!(
            CssShapeSmooth::by(pair(), Some(relative(None))).view(),
            CssShapeSmoothRef::By {
                control: Some(_),
                ..
            }
        ));
    }
    #[test]
    fn absolute_numeric_pair_normalization_moves_original_scalar_origins() {
        let values = parse_component_values("-1e-999px 2%").unwrap();
        let scalars: Vec<_> = values
            .items()
            .iter()
            .filter(|v| {
                matches!(
                    v.view(),
                    CssComponentValueRef::Token(
                        CssValueTokenRef::Dimension { .. } | CssValueTokenRef::Percentage(_)
                    )
                )
            })
            .cloned()
            .collect();
        let x = CssSpecifiedLengthPercentage::try_from_component(scalars[0].clone()).unwrap();
        let y = CssSpecifiedLengthPercentage::try_from_component(scalars[1].clone()).unwrap();
        let value = CssShapeAbsoluteControlPoint::from_coordinates(
            CssShapeCoordinatePair::new(x.clone(), y.clone()),
            None,
        );
        let CssShapeAbsoluteControlPointRef::Position(p) = value.view() else {
            panic!("normalized position")
        };
        let CssPositionRef::Cartesian(p) = p.view() else {
            panic!("Cartesian pair")
        };
        let CssHorizontalPosition::Offset(a) = p.horizontal() else {
            panic!("x")
        };
        let CssVerticalPosition::Offset(b) = p.vertical() else {
            panic!("y")
        };
        assert_eq!(a, &x);
        assert_eq!(b, &y);
        assert_eq!(a.origin(), scalars[0].origin());
        assert_eq!(b.origin(), scalars[1].origin());
        for anchor in [
            CssShapeControlAnchor::Start,
            CssShapeControlAnchor::End,
            CssShapeControlAnchor::Origin,
        ] {
            let p = CssShapeAbsoluteControlPoint::from_coordinates(
                CssShapeCoordinatePair::new(x.clone(), y.clone()),
                Some(anchor),
            );
            let CssShapeAbsoluteControlPointRef::Coordinates { offset, anchor: a } = p.view()
            else {
                panic!("explicit pair")
            };
            assert_eq!(a, anchor);
            assert_eq!(offset.x(), &x);
            assert_eq!(offset.y(), &y);
        }
    }
    #[test]
    fn pair_equality_ignores_provenance_while_raw_scalars_retain_it() {
        let components = parse_component_values("1px").unwrap();
        let parsed =
            CssSpecifiedLengthPercentage::try_from_component(components.items()[0].clone())
                .unwrap();
        let programmatic = lp("1px");
        assert_ne!(parsed, programmatic);
        let a = CssShapeCoordinatePair::new(parsed, lp("2%"));
        let b = pair();
        assert_eq!(a, b);
        assert_ne!(a, CssShapeCoordinatePair::new(lp("2%"), lp("1px")));
        assert_ne!(
            CssShapeEndpoint::To(position()),
            CssShapeEndpoint::By(CssShapeCoordinatePair::new(lp("0px"), lp("0px")))
        );
    }
    #[test]
    fn control_anchor_omissions_and_explicit_defaults_remain_distinct() {
        assert_eq!(
            absolute(None),
            CssShapeAbsoluteControlPoint::from_position(CssPosition::from_cartesian(
                CssCartesianPosition::try_new(
                    CssHorizontalPosition::Offset(lp("1px")),
                    CssVerticalPosition::Offset(lp("2%"))
                )
                .unwrap()
            ))
        );
        assert_ne!(
            absolute(None),
            absolute(Some(CssShapeControlAnchor::Origin))
        );
        assert_ne!(relative(None), relative(Some(CssShapeControlAnchor::Start)));
        assert_eq!(relative(None).offset(), &pair());
    }
    #[test]
    fn arc_accessors_retain_radius_arity_and_each_optional_group() {
        let omitted = CssShapeArc::new(
            CssShapeEndpoint::By(pair()),
            CssShapeArcRadii::One(lp("-3px")),
            None,
            None,
            None,
        );
        assert_eq!(omitted.endpoint(), &CssShapeEndpoint::By(pair()));
        assert!(matches!(omitted.radii(),CssShapeArcRadii::One(v) if v==&lp("-3px")));
        assert_eq!(omitted.sweep(), None);
        assert_eq!(omitted.size(), None);
        assert_eq!(omitted.rotation(), None);
        let explicit = CssShapeArc::new(
            omitted.endpoint().clone(),
            omitted.radii().clone(),
            Some(CssShapeArcSweep::Ccw),
            Some(CssShapeArcSize::Small),
            Some(angle("0deg")),
        );
        assert_ne!(omitted, explicit);
        assert_eq!(explicit.sweep(), Some(CssShapeArcSweep::Ccw));
        assert_eq!(explicit.size(), Some(CssShapeArcSize::Small));
        assert_eq!(explicit.rotation(), Some(&angle("0deg")));
        let two = CssShapeArc::new(
            omitted.endpoint().clone(),
            CssShapeArcRadii::Two {
                horizontal: lp("-3px"),
                vertical: lp("-3px"),
            },
            None,
            None,
            None,
        );
        assert_ne!(omitted, two);
    }
    #[test]
    fn rotation_accepts_exact_dimensions_without_zero_number_or_float_bounds() {
        assert!(
            CssAngleLiteral::try_from_component(CssComponentValue::try_number("0").unwrap())
                .is_err()
        );
        for text in [
            "-1e-999deg",
            "1e400turn",
            "1.00000000000000000001rad",
            "-0grad",
        ] {
            let rotation = angle(text);
            let arc = CssShapeArc::new(
                CssShapeEndpoint::By(pair()),
                CssShapeArcRadii::One(lp("0")),
                None,
                None,
                Some(rotation.clone()),
            );
            assert_eq!(arc.rotation(), Some(&rotation));
            assert_eq!(
                arc.rotation().unwrap().literal().unwrap().component(),
                rotation.literal().unwrap().component()
            );
        }
    }
    fn budget(value: &CssShapeFunction, expected: &str, nodes: usize) {
        let before = value.clone();
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
            assert_eq!(value, &before);
        }
    }
    #[test]
    fn literal_commands_controls_radii_and_options_accumulate_independent_counts() {
        // Base visits: function/from/list + initial position/axes/scalars = 8.
        // A relative line adds verb/affinity/pair/two scalars = 5; close adds 1.
        budget(
            &shape(vec![
                CssShapeCommand::Line(CssShapeEndpoint::By(pair())),
                CssShapeCommand::Close,
            ]),
            "shape(from 0px 0px, line by 1px 2%, close)",
            14,
        );
        // Relative curve: verb/affinity/endpoint-pair(3)/with/control/pair(3)=10;
        // its second anchored control adds control/pair(3)/anchor=5.
        budget(
            &shape(vec![CssShapeCommand::Curve(CssShapeCurve::by(
                pair(),
                relative(None),
                Some(relative(Some(CssShapeControlAnchor::End))),
            ))]),
            "shape(from 0px 0px, curve by 1px 2% with 1px 2% / 1px 2% from end)",
            23,
        );
        // Absolute curve adds verb/affinity/position(5)/with/control/position(5)=14;
        // the explicit second numeric control adds control/pair(3)/anchor=5.
        budget(
            &shape(vec![CssShapeCommand::Curve(CssShapeCurve::to(
                position(),
                absolute(None),
                Some(absolute(Some(CssShapeControlAnchor::Origin))),
            ))]),
            "shape(from 0px 0px, curve to 0px 0px with 1px 2% / 1px 2% from origin)",
            27,
        );
        // Arc adds verb/affinity/pair(3)/of/two radii/sweep/size/rotate/angle=12.
        budget(
            &shape(vec![CssShapeCommand::Arc(CssShapeArc::new(
                CssShapeEndpoint::By(pair()),
                CssShapeArcRadii::Two {
                    horizontal: lp("-3px"),
                    vertical: lp("4%"),
                },
                Some(CssShapeArcSweep::Cw),
                Some(CssShapeArcSize::Large),
                Some(angle("5deg")),
            ))]),
            "shape(from 0px 0px, arc by 1px 2% of -3px 4% cw large rotate 5deg)",
            20,
        );
    }
    #[test]
    fn ordinary_close_fill_and_clip_wrappers_share_the_declared_budget() {
        let value = shape(vec![CssShapeCommand::Close]);
        budget(&value, "shape(from 0px 0px, close)", 9);
        let filled = CssShapeFunction::new(
            Some(CssFillRule::Nonzero),
            position(),
            value.commands().clone(),
        );
        budget(&filled, "shape(nonzero from 0px 0px, close)", 10);
        let basic = CssBasicShape::Shape(value);
        let clip = CssClipPath::BasicShape(CssClipPathShape::new(
            basic,
            Some(CssBoxEdgeKeyword::BorderBox),
        ));
        assert_eq!(
            clip.serialize_specified_with_limits(L::new(11, 11, 37))
                .unwrap(),
            "shape(from 0px 0px, close) border-box"
        );
        assert_eq!(
            clip.serialize_specified_with_limits(L::new(10, 11, 37))
                .unwrap_err()
                .kind(),
            K::InputNodeLimit
        );
    }

    #[test]
    fn axis_enums_keep_all_keywords_and_offset_affinities_distinct() {
        for (keyword, text) in [
            (CssHorizontalPositionKeyword::Left, "left"),
            (CssHorizontalPositionKeyword::Center, "center"),
            (CssHorizontalPositionKeyword::Right, "right"),
            (CssHorizontalPositionKeyword::XStart, "x-start"),
            (CssHorizontalPositionKeyword::XEnd, "x-end"),
        ] {
            let command = CssShapeHorizontalLine::ToKeyword(keyword);
            assert!(matches!(&command,CssShapeHorizontalLine::ToKeyword(v) if *v==keyword));
            assert_eq!(
                shape(vec![CssShapeCommand::HorizontalLine(command)])
                    .serialize_specified()
                    .unwrap(),
                format!("shape(from 0px 0px, hline to {text})")
            );
        }
        for (keyword, text) in [
            (CssVerticalPositionKeyword::Top, "top"),
            (CssVerticalPositionKeyword::Center, "center"),
            (CssVerticalPositionKeyword::Bottom, "bottom"),
            (CssVerticalPositionKeyword::YStart, "y-start"),
            (CssVerticalPositionKeyword::YEnd, "y-end"),
        ] {
            let command = CssShapeVerticalLine::ToKeyword(keyword);
            assert!(matches!(&command,CssShapeVerticalLine::ToKeyword(v) if *v==keyword));
            assert_eq!(
                shape(vec![CssShapeCommand::VerticalLine(command)])
                    .serialize_specified()
                    .unwrap(),
                format!("shape(from 0px 0px, vline to {text})")
            );
        }
        assert_ne!(
            CssShapeHorizontalLine::ToOffset(lp("-1px")),
            CssShapeHorizontalLine::By(lp("-1px"))
        );
        assert_ne!(
            CssShapeVerticalLine::ToOffset(lp("-1px")),
            CssShapeVerticalLine::By(lp("-1px"))
        );
        let value = shape(vec![
            CssShapeCommand::HorizontalLine(CssShapeHorizontalLine::ToOffset(lp("-1px"))),
            CssShapeCommand::HorizontalLine(CssShapeHorizontalLine::By(lp("-1px"))),
            CssShapeCommand::VerticalLine(CssShapeVerticalLine::ToOffset(lp("-1px"))),
            CssShapeCommand::VerticalLine(CssShapeVerticalLine::By(lp("-1px"))),
        ]);
        budget(
            &value,
            "shape(from 0px 0px, hline to -1px, hline by -1px, vline to -1px, vline by -1px)",
            20,
        );
    }
    #[test]
    fn sibling_command_math_arenas_share_budgets_and_keep_original_graphs() {
        let components = parse_component_values("calc(1px + 2%)").unwrap();
        let scalar = CssSpecifiedLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap(),
        )
        .unwrap();
        let offsets = CssShapeCoordinatePair::new(scalar.clone(), scalar.clone());
        let value = shape(vec![
            CssShapeCommand::Line(CssShapeEndpoint::By(offsets.clone())),
            CssShapeCommand::Smooth(CssShapeSmooth::by(offsets, None)),
        ]);
        let expected = "shape(from 0px 0px, line by calc(2% + 1px) calc(2% + 1px), smooth by calc(2% + 1px) calc(2% + 1px))";
        // Eight initial visits; two commands each add verb/affinity/pair (3).
        // Four independent mixed LP arenas add 4 input / 5 projection nodes each.
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(30, 34, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (L::new(29, 34, expected.len()), K::InputNodeLimit),
            (L::new(30, 33, expected.len()), K::ProjectionNodeLimit),
            (L::new(30, 34, expected.len() - 1), K::ByteLimit),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            let CssShapeCommand::Line(CssShapeEndpoint::By(pair)) = &value.commands().commands()[0]
            else {
                panic!("relative line")
            };
            assert_eq!(pair.x(), &scalar);
            assert_eq!(pair.y(), &scalar);
            assert_eq!(pair.x().calculation().unwrap().components(), &components);
            assert_eq!(pair.x().origin(), scalar.origin());
        }
    }
}
