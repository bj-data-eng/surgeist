#![forbid(unsafe_code)]
//! Authored UI 4 grammar, lifecycle and canonical provider contracts.
//! UI4 WD20260120 tables 389/529/985/1490/1778, cursor production 598,
//! limited URL-set 614 and required alias 1760 independently define this scope.
//! Dated Images4 WD20250930 section2.4/8/AppendixA define string candidates,
//! descriptors and canonical image-set alias. Values4 WD20240312 owns exact
//! Number/Resolution math, deferred calculation ranges and projection policy.
//! Selected Color4 CRD20260908 sections5/9/16 and Color5 WD20260908
//! sections2/3/4/5/6/7/8/11 own the already adopted imported color forms.
//! Existing public provider contract owns aggregate/child prices; name and
//! declaration add2, punctuation/importance charge final UTF8 bytes only.
//! New Resize variants, new borrowed longhand payloads and private cumulative
//! writer suppression tests must accompany functional implementation.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const FAMILY: [&str; 5] = [
    "caret-color",
    "cursor",
    "pointer-events",
    "user-select",
    "resize",
];
const CURSOR: [&str; 36] = [
    "auto",
    "default",
    "none",
    "context-menu",
    "help",
    "pointer",
    "progress",
    "wait",
    "cell",
    "crosshair",
    "text",
    "vertical-text",
    "alias",
    "copy",
    "move",
    "no-drop",
    "not-allowed",
    "grab",
    "grabbing",
    "all-scroll",
    "col-resize",
    "row-resize",
    "n-resize",
    "e-resize",
    "s-resize",
    "w-resize",
    "ne-resize",
    "nw-resize",
    "se-resize",
    "sw-resize",
    "ew-resize",
    "ns-resize",
    "nesw-resize",
    "nwse-resize",
    "zoom-in",
    "zoom-out",
];
const CARET: [&str; 4] = ["auto", "currentcolor", "transparent", "rebeccapurple"];
const POINTER: [&str; 2] = ["auto", "none"];
const SELECTION: [&str; 5] = ["auto", "text", "none", "contain", "all"];
const RESIZE: [&str; 6] = ["none", "both", "horizontal", "vertical", "block", "inline"];
fn keywords(name: &str) -> &'static [&'static str] {
    match name {
        "caret-color" => &CARET,
        "cursor" => &CURSOR,
        "pointer-events" => &POINTER,
        "user-select" => &SELECTION,
        "resize" => &RESIZE,
        _ => panic!("selected UI domain"),
    }
}
fn initial(name: &str) -> &'static str {
    if name == "resize" { "none" } else { "auto" }
}
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
// Specialized authored/canonical witnesses; these are not a property inventory.
const CURSOR_SPECIFIED_WITNESSES: &[(&str, &str)] = &[
    ("url(first.cur), auto", "url(\"first.cur\"), auto"),
    ("url(first.cur) 0 -0, auto", "url(\"first.cur\") 0 0, auto"),
    (
        "url(first.cur) -3.500 +4.250, pointer",
        "url(\"first.cur\") -3.5 4.25, pointer",
    ),
    (
        "url(first.cur) -0.0000005 0.0000005, auto",
        "url(\"first.cur\") -0.000001 0.000001, auto",
    ),
    (
        "url(first.cur) calc(1 + 2) calc(-0.25), pointer",
        "url(\"first.cur\") calc(3) calc(-0.25), pointer",
    ),
    (
        "url(first.cur) calc(pi) calc(e), auto",
        "url(\"first.cur\") calc(3.141593) calc(2.718282), auto",
    ),
    (
        "url(first.cur) calc(1em / 1px) calc(1px / 1px), auto",
        "url(\"first.cur\") calc(1em / 1px) calc(1), auto",
    ),
    (
        "url(first.cur) min(30,40) max(-2.5,-0.25), move",
        "url(\"first.cur\") calc(30) calc(-0.25), move",
    ),
    (
        "url(first.cur) 0 0,url(second.cur) -3 4,default",
        "url(\"first.cur\") 0 0, url(\"second.cur\") -3 4, default",
    ),
    (
        "image-set(\"first.png\"), pointer",
        "image-set(\"first.png\"), pointer",
    ),
    (
        r#"image-set("first\2e png"),auto"#,
        "image-set(\"first.png\"), auto",
    ),
    (
        "image-set(url(first.png),url(second.png) 1x), auto",
        "image-set(url(\"first.png\"), url(\"second.png\") 1dppx), auto",
    ),
    (
        "image-set(\"first.png\" type(\"image/png\") 96dpi,url(second.png) 2x),pointer",
        "image-set(\"first.png\" 1dppx type(\"image/png\"), url(\"second.png\") 2dppx), pointer",
    ),
    (
        "image-set(\"first.png\" 2x type(\"image/png\")),pointer",
        "image-set(\"first.png\" 2dppx type(\"image/png\")), pointer",
    ),
    (
        "image-set(url(first.png) calc(1x + 1x)),auto",
        "image-set(url(\"first.png\") calc(2dppx)), auto",
    ),
    (
        "image-set(url(first.png) type(\"not a MIME type\")),auto",
        "image-set(url(\"first.png\") type(\"not a MIME type\")), auto",
    ),
    (
        "image-set(url(first.png) 1x,url(second.png) 96dpi),auto",
        "image-set(url(\"first.png\") 1dppx, url(\"second.png\") 1dppx), auto",
    ),
    (
        "image-set(url(first.png) -0x,url(second.png) calc(-1x)),auto",
        "image-set(url(\"first.png\") 0dppx, url(\"second.png\") calc(-1dppx)), auto",
    ),
    (
        "-WEBKIT-IMAGE-SET(\"first.png\" type(\"\") 2x),POINTER",
        "image-set(\"first.png\" 2dppx type(\"\")), pointer",
    ),
];

// Specialized authored/canonical witnesses; these are not a property inventory.
const CARET_SPECIFIED_WITNESSES: &[(&str, &str)] = &[
    ("red", "red"),
    ("currentcolor", "currentcolor"),
    ("transparent", "transparent"),
    ("Canvas", "canvas"),
    ("ActiveBorder", "activeborder"),
    ("#abc", "rgb(170, 187, 204)"),
    ("#abcdef", "rgb(171, 205, 239)"),
    ("#00000080", "rgba(0, 0, 0, 0.5)"),
    ("rgb(1 2 3 / .5)", "rgba(1, 2, 3, 0.5)"),
    ("rgba(1,2,3,.5)", "rgba(1, 2, 3, 0.5)"),
    ("hsl(0 100% 50%)", "rgb(255, 0, 0)"),
    ("hwb(0 0% 0%)", "rgb(255, 0, 0)"),
    ("lab(50% 20 -30 / .5)", "lab(50 20 -30 / 0.5)"),
    ("lch(50% 20 30deg)", "lch(50 20 30)"),
    ("oklab(.5 .1 -.1)", "oklab(0.5 0.1 -0.1)"),
    ("oklch(.5 .1 30)", "oklch(0.5 0.1 30)"),
    ("rgb(none 0 255)", "color(srgb none 0 1)"),
    ("lab(none 0 0)", "lab(none 0 0)"),
    ("color(srgb 100% 50% 0%)", "color(srgb 1 0.5 0)"),
    ("color(display-p3 1 0 0)", "color(display-p3 1 0 0)"),
    ("color(--P 0% 70% 20% 0%)", "color(--P 0 0.7 0.2 0)"),
    ("rgb(from red r g b / alpha)", "rgb(from red r g b / alpha)"),
    ("lab(from red l a b)", "lab(from red l a b)"),
    (
        "color(from red --P Cyan / alpha)",
        "color(from red --P Cyan / alpha)",
    ),
    ("color-mix(in oklab, red, blue)", "color-mix(red, blue)"),
    (
        "color-mix(in oklch longer hue, red 25%, blue 75%)",
        "color-mix(in oklch longer hue, red 25%, blue 75%)",
    ),
    ("light-dark(red,blue)", "light-dark(red, blue)"),
    (
        "contrast-color(hsl(0 100% 50%))",
        "contrast-color(rgb(255, 0, 0))",
    ),
    ("alpha(from red / 50%)", "alpha(from red / 0.5)"),
    (
        "device-cmyk(0% 81% 81% 30%)",
        "device-cmyk(0 0.81 0.81 0.3)",
    ),
];

fn property(name: &str) -> P {
    P::from_name(name).unwrap_or_else(|| panic!("authored property unavailable: {name}"))
}
fn parsed(name: &str, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{value}!important", name.to_ascii_uppercase());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(source.known().unwrap().property().canonical_name(), name);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    let position = source.position().unwrap();
    assert_eq!(position.byte_offset().value(), "/*😀*/".len());
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value() as usize,
        "/*😀*/".encode_utf16().count()
    );
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
    source.clone()
}
fn checked_components(
    name: &str,
    value: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let p = property(name);
    if grammar {
        parse_property_value_for_grammar(p.grammar(), value, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            value,
            CssImportance::Important,
        )
    }
}
fn checked(name: &str, value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    let source = checked_components(name, components.clone(), grammar).unwrap();
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(source.known().unwrap().grammar(), property(name).grammar());
    source
}
fn text_front(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    let p = property(name);
    let report = if grammar {
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Important)
    } else {
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Important)
    };
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    let source = report.syntax().as_ref().unwrap().clone();
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert_eq!(source.parsed_value().unwrap().source().as_str(), text);
    source
}
fn fronts(name: &str, text: &str) -> [CssDeclaration; 5] {
    [
        parsed(name, text),
        checked(name, text, false),
        checked(name, text, true),
        text_front(name, text, false),
        text_front(name, text, true),
    ]
}
fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("complete longhand contribution")
    };
    assert_eq!(values.items().len(), 1);
    let item = &values.items()[0];
    assert_eq!(item.property(), source.known().unwrap().property());
    assert_source(item, source);
    values
}
fn accepted(name: &str, input: &str, canonical: &str) {
    for source in fronts(name, input) {
        let before = source.clone();
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{name}: {canonical} !important;")
        );
        assert!(source.known().unwrap().property_value().is_some());
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
        let values = completed(&source);
        let item = &values.items()[0];
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            property(name)
        );
        assert!(item.replacement_components().is_none());
        assert_eq!(source, before);
    }
    assert_eq!(
        checked(name, canonical, true).to_specified_css().unwrap(),
        format!("{name}: {canonical} !important;")
    );
}
fn invalid(name: &str, value: &str) {
    let p = property(name);
    let css = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|v| v.known().unwrap().property() == P::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic grammar failure")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    match diagnostic.error().kind() {
        ErrorKind::InvalidPropertyValue(detail) => assert_eq!(detail.property(), p),
        // Caret imports the shared Color grammar and its structured diagnostics.
        ErrorKind::InvalidColorSyntax(_) if p == P::CaretColor => {
            assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
        }
        other => panic!("owned grammar diagnostic for {css}: {other:?}"),
    }
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    for grammar in [false, true] {
        assert!(matches!(
            checked_components(name, components.clone(), grammar)
                .unwrap_err()
                .kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
    for report in [
        parse_property_value_text(value, CssPropertyNameRef::Known(p), CssImportance::Normal),
        parse_property_value_text_for_grammar(value, p.grammar(), CssImportance::Normal),
    ] {
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
    }
    assert_eq!(components, before);
}

#[test]
fn programmatic_keyword_components_preserve_origins_identity_and_importance() {
    for name in FAMILY {
        for keyword in keywords(name) {
            let components =
                CssComponentValues::try_new(vec![CssComponentValue::try_ident(*keyword).unwrap()])
                    .unwrap();
            let before = components.clone();
            for grammar in [false, true] {
                let source = checked_components(name, components.clone(), grammar).unwrap();
                assert_eq!(source.value_components(), &components);
                assert_eq!(
                    source.value_components().items()[0].origin(),
                    &CssValueOrigin::Programmatic
                );
                assert_eq!(source.importance(), CssImportance::Important);
                assert!(source.position().is_none());
                assert!(source.parsed_name().is_none());
                assert!(source.parsed_value().is_none());
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {keyword} !important;")
                );
                completed(&source);
            }
            assert_eq!(components, before);
        }
    }
}

#[test]
fn all_css_wide_values_are_symbolic_single_terminals_preserving_occurrence_and_importance() {
    for name in FAMILY {
        for (text, keyword) in GLOBALS {
            for source in fronts(name, text) {
                assert_eq!(source.known().unwrap().global(), Some(keyword));
                assert!(source.known().unwrap().property_value().is_none());
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {text} !important;")
                );
                let values = completed(&source);
                let item = &values.items()[0];
                assert!(matches!(item.value(),CssContributionValueRef::Global(v) if v == keyword));
                assert!(item.replacement_components().is_none());
            }
        }
    }
}

#[test]
fn var_env_attr_reentry_admits_every_keyword_and_keeps_original_and_replacement_snapshots() {
    for name in FAMILY {
        for pending in ["var(--ui)", "env(ui)", "attr(data-ui *)"] {
            for source in fronts(name, pending) {
                let before = source.clone();
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("pending handle")
                };
                assert!(handle.source().same_occurrence(&source));
                assert_eq!(handle.source().importance(), CssImportance::Important);
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {pending} !important;")
                );
                for text in keywords(name) {
                    let replacement = parse_component_values(&text.to_ascii_uppercase()).unwrap();
                    let snapshot = replacement.clone();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("ordinary replacement")
                    };
                    assert_eq!(values.items().len(), 1);
                    let item = &values.items()[0];
                    assert_eq!(item.property(), property(name));
                    assert_source(item, &source);
                    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                    assert_eq!(item.replacement_components(), Some(&replacement));
                    for (actual, supplied) in item
                        .replacement_components()
                        .unwrap()
                        .items()
                        .iter()
                        .zip(replacement.items())
                    {
                        assert_eq!(actual.origin(), supplied.origin());
                        let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                            (actual.origin(), supplied.origin())
                        else {
                            panic!("replacement origin")
                        };
                        assert!(actual.source().same_snapshot(supplied.source()));
                        assert_eq!(actual.source().as_str(), text.to_ascii_uppercase());
                    }
                    let ordinary = checked_components(name, replacement.clone(), true).unwrap();
                    assert_eq!(
                        ordinary.to_specified_css().unwrap(),
                        format!("{name}: {text} !important;")
                    );
                    let ordinary_values = completed(&ordinary);
                    assert_eq!(
                        item.ordinary_value(),
                        ordinary_values.items()[0].ordinary_value()
                    );
                    assert_eq!(replacement, snapshot);
                }
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("global replacement")
                    };
                    assert_eq!(values.items().len(), 1);
                    let item = &values.items()[0];
                    assert_source(item, &source);
                    assert!(
                        matches!(item.value(),CssContributionValueRef::Global(v) if v == keyword)
                    );
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
                assert_eq!(source, before);
            }
        }
    }
}

#[test]
fn invalid_or_residual_replacements_fail_atomically_and_the_same_handle_can_retry() {
    for name in FAMILY {
        for pending in ["var(--ui)", "env(ui)", "attr(data-ui *)"] {
            let source = checked(name, pending, true);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            for text in [
                "var(--again)",
                "env(again)",
                "attr(data-again *)",
                "auto var(--again)",
                r"\76 ar(--again)",
            ] {
                for _ in 0..2 {
                    assert!(matches!(
                        handle
                            .reenter(parse_component_values(text).unwrap())
                            .unwrap_err()
                            .kind(),
                        CssExpansionErrorKind::ResidualSubstitution
                    ));
                }
            }
            for text in ["auto auto", "bogus", "auto!important", "auto;color:red"] {
                for _ in 0..2 {
                    let error = handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("strict replacement")
                    };
                    assert!(matches!(
                        error.kind(),
                        CssPropertyValueErrorKind::Grammar(_)
                    ));
                }
            }
            let replacement = parse_component_values(initial(name)).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("successful reusable retry")
            };
            assert_eq!(values.items().len(), 1);
            assert_source(&values.items()[0], &source);
            assert_eq!(
                values.items()[0].replacement_components(),
                Some(&replacement)
            );
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(source, before);
        }
    }
}
fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("public original implicit closing origin")
}
fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin, text: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("implicit EOF")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}

#[test]
fn both_checked_fronts_reject_original_comment_and_pending_function_closures() {
    for name in FAMILY {
        let mut texts: Vec<String> = keywords(name)
            .iter()
            .map(|text| format!("{text}/*"))
            .collect();
        texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
        texts.extend(
            [
                "var(--ui)/*",
                "env(ui)/*",
                "attr(data-ui *)/*",
                "var(--ui",
                "env(ui",
                "attr(data-ui *",
            ]
            .map(str::to_owned),
        );
        for text in texts {
            let components = parse_component_values(&text).unwrap();
            let before = components.clone();
            let origin = implicit_origin(&components);
            for grammar in [false, true] {
                assert_closure(
                    &checked_components(name, components.clone(), grammar).unwrap_err(),
                    &origin,
                    &text,
                );
            }
            assert_eq!(components, before);
        }
    }
}

#[test]
fn replacement_closure_is_strict_and_residual_priority_and_complete_comment_controls_hold() {
    for name in FAMILY {
        for pending in ["var(--ui)", "env(ui)", "attr(data-ui *)"] {
            let source = checked(name, pending, false);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            let mut texts: Vec<String> = keywords(name)
                .iter()
                .map(|text| format!("{text}/*"))
                .collect();
            texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
            for text in texts {
                let components = parse_component_values(&text).unwrap();
                let origin = implicit_origin(&components);
                for _ in 0..2 {
                    let error = handle.reenter(components.clone()).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("original replacement closure")
                    };
                    assert_closure(error, &origin, &text);
                }
            }
            for text in [
                "var(--again",
                "env(again",
                "attr(data-again *",
                "var(--again)/*",
            ] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
            for grammar in [false, true] {
                checked(name, &format!("{} /**/", initial(name)), grammar);
            }
            for text in [
                "initial/**/",
                "var(--ui)/**/",
                "env(ui)/**/",
                "attr(data-ui *)/**/",
            ] {
                checked(name, text, false);
                checked(name, text, true);
            }
            assert!(
                handle
                    .reenter(parse_component_values(&format!("{} /**/", initial(name))).unwrap())
                    .is_ok()
            );
        }
    }
}

#[test]
fn checked_value_annotations_map_to_original_tokens_and_authored_importance_is_valid() {
    for name in FAMILY {
        let text = format!("/*😀*/{}!important", initial(name));
        let components = parse_component_values(&text).unwrap();
        let serialized = components.serialize().unwrap();
        let origin = serialized
            .origin_at(serialized.as_css().find('!').unwrap())
            .unwrap();
        for grammar in [false, true] {
            let error = checked_components(name, components.clone(), grammar).unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
            assert_eq!(error.origin(), origin);
        }
        for report in [
            parse_property_value_text(
                &text,
                CssPropertyNameRef::Known(property(name)),
                CssImportance::Important,
            ),
            parse_property_value_text_for_grammar(
                &text,
                property(name).grammar(),
                CssImportance::Important,
            ),
        ] {
            assert!(report.syntax().is_none());
            assert!(!report.is_clean());
        }
        accepted(name, initial(name), initial(name));
    }
}

#[test]
fn browser_eof_recovery_retains_diagnostics_and_normalized_original_occurrences() {
    for name in FAMILY {
        let mut texts = vec![
            format!("{}/*", initial(name)),
            "var(--ui".to_owned(),
            "env(ui".to_owned(),
            "attr(data-ui *".to_owned(),
        ];
        texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
        for text in texts {
            let css = format!(".a{{{name}:{text}");
            let report = parse_sheet(&css);
            assert!(!report.is_clean());
            assert!(validate_sheet(&css).is_err());
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|v| v.action() == CssRecoveryAction::RetainWithImplicitClosure)
            );
            let before = report.clone();
            let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
            let normalized = normalize_report_with_limits(&report, exact).unwrap();
            assert_eq!(normalized.diagnostics(), report.diagnostics());
            let declarations: Vec<_> = normalized
                .syntax()
                .items()
                .iter()
                .filter_map(|v| match v {
                    CssNormalizedItem::Declaration(v) => Some(v),
                    _ => None,
                })
                .collect();
            let [item] = declarations.as_slice() else {
                panic!("retained recovered declaration")
            };
            assert_eq!(item.order(), 0);
            assert_eq!(item.source().known().unwrap().property(), property(name));
            assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(values.items().len(), 1);
                    assert_source(&values.items()[0], item.source());
                }
                _ => panic!("retained intrinsic lifecycle"),
            }
            let error = normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, 0).unwrap(),
            )
            .unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded {
                    resource: CssNormalizationResource::Contributions,
                    limit: 0
                }
            );
            assert_eq!(error.declaration_order(), Some(0));
            assert!(error.declaration().unwrap().same_occurrence(item.source()));
            assert_eq!(report, before);
            assert!(normalize_report_with_limits(&report, exact).is_ok());
        }
    }
}

#[test]
fn canonical_keyword_declarations_have_exact_node_byte_budgets_and_atomic_reusable_failures() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for name in FAMILY {
        let values: Vec<_> = keywords(name)
            .iter()
            .copied()
            .chain(GLOBALS.map(|(text, _)| text))
            .collect();
        for text in values {
            for source in [parsed(name, text), checked(name, text, true)] {
                let before = source.clone();
                let expected = format!("{name}: {text} !important;");
                let exact = Limits::new(3, 3, expected.len());
                assert_eq!(
                    source.to_specified_css_with_limits(exact).unwrap(),
                    expected
                );
                for (limits, kind) in [
                    (Limits::new(2, 3, expected.len()), Kind::InputNodeLimit),
                    (Limits::new(3, 2, expected.len()), Kind::ProjectionNodeLimit),
                    (Limits::new(3, 3, expected.len() - 1), Kind::ByteLimit),
                ] {
                    for _ in 0..2 {
                        assert_eq!(
                            source
                                .to_specified_css_with_limits(limits)
                                .unwrap_err()
                                .kind(),
                            kind
                        );
                        assert_eq!(source, before);
                    }
                }
                assert_eq!(
                    source.to_specified_css_with_limits(exact).unwrap(),
                    expected
                );
            }
        }
    }
}

#[test]
fn existing_flow_flex_grid_item_and_color_controls_execute_independently_of_ui_metadata() {
    for (name, input, expected, count) in [
        (
            "flow-tolerance",
            "calc(-2px - 3%)",
            "flow-tolerance: calc(-3% - 2px);",
            1,
        ),
        (
            "flex-flow",
            "wrap row-reverse",
            "flex-flow: row-reverse wrap;",
            2,
        ),
        ("grid-template", "none", "grid-template: none;", 3),
        ("item-flow", "row", "item-flow: row auto normal normal;", 4),
        ("color", "red", "color: red;", 1),
    ] {
        let report = parse_style_attribute(&format!("{name}:{input}"));
        assert!(report.is_clean());
        for source in report.syntax().iter() {
            assert_eq!(source.to_specified_css().unwrap(), expected);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(source).unwrap()
            else {
                panic!("supported independent control")
            };
            assert_eq!(values.items().len(), count);
            for item in values.items() {
                assert_source(item, source);
            }
        }
    }
}

#[test]
fn all_selected_ui_keywords_keep_canonical_grammar_through_five_front_doors() {
    for name in FAMILY {
        for keyword in keywords(name) {
            accepted(name, keyword, keyword);
        }
    }
}
#[test]
fn decoded_keyword_case_comments_and_escapes_keep_authored_components() {
    for (name, input, canonical) in [
        ("caret-color", r"\72 ebeccapurple", "rebeccapurple"),
        ("cursor", r"\70 ointer", "pointer"),
        ("pointer-events", r"\6e one", "none"),
        ("user-select", r"\63 ontain", "contain"),
        ("resize", r"\62 lock", "block"),
    ] {
        accepted(name, input, canonical);
    }
    for name in FAMILY {
        for keyword in keywords(name) {
            accepted(
                name,
                &format!("/**/{} /**/", keyword.to_ascii_uppercase()),
                keyword,
            );
        }
    }
}
#[test]
fn canonical_and_standard_alias_share_one_identity_and_original_name_provenance() {
    let canonical = property("user-select");
    for alias in ["-webkit-user-select", "-WEBKIT-USER-SELECT"] {
        assert_eq!(P::from_name(alias), Some(canonical));
        assert_eq!(
            CssPropertyGrammar::from_name(alias),
            Some(canonical.grammar())
        );
        for text in SELECTION {
            let css = format!("/*😀*/{alias}:{text}!important");
            let report = parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            assert_eq!(
                validate_style_attribute(&css).unwrap(),
                report.syntax().clone()
            );
            let [source] = report.syntax().as_slice() else {
                panic!("one alias occurrence")
            };
            assert_eq!(source.known().unwrap().property(), canonical);
            assert_eq!(source.known().unwrap().grammar(), canonical.grammar());
            assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
            let origin = source.parsed_name().unwrap();
            let start = "/*😀*/".len();
            assert_eq!(origin.span().start().byte_offset().value(), start);
            assert_eq!(
                origin.span().end().byte_offset().value(),
                start + alias.len()
            );
            assert_eq!(source.position(), Some(origin.span().start()));
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("user-select: {text} !important;")
            );
            completed(source);
        }
    }
    let css = r"\2d webkit-user-select:ALL!important";
    let report = parse_style_attribute(css);
    assert!(report.is_clean());
    assert_eq!(report.syntax()[0].known().unwrap().property(), canonical);
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "user-select: all !important;"
    );
    for alias in ["-fake-user-select", "webkit-user-select"] {
        assert!(P::from_name(alias).is_none());
    }
}
#[test]
fn standard_alias_globals_and_pending_keep_the_canonical_lifecycle_and_alias_occurrence() {
    for text in
        GLOBALS
            .map(|(text, _)| text)
            .into_iter()
            .chain(["var(--ui)", "env(ui)", "attr(data-ui *)"])
    {
        let css = format!("/*😀*/-WEBKIT-USER-SELECT:{text}!important");
        let report = parse_style_attribute(&css);
        assert!(report.is_clean());
        let [source] = report.syntax().as_slice() else {
            panic!("one alias occurrence")
        };
        assert_eq!(source.known().unwrap().property(), property("user-select"));
        assert_eq!(
            source.known().unwrap().grammar(),
            property("user-select").grammar()
        );
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("user-select: {text} !important;")
        );
        if let Some((_, keyword)) = GLOBALS.iter().find(|(name, _)| *name == text) {
            let values = completed(source);
            assert!(
                matches!(values.items()[0].value(),CssContributionValueRef::Global(v) if v==*keyword)
            );
        } else {
            let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
                panic!("one pending alias")
            };
            assert!(handle.source().same_occurrence(source));
            let replacement = parse_component_values("ALL").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("one replacement")
            };
            assert_eq!(values.items().len(), 1);
            assert_eq!(values.items()[0].property(), property("user-select"));
            assert_source(&values.items()[0], source);
            assert_eq!(
                values.items()[0].replacement_components(),
                Some(&replacement)
            );
        }
    }
}
#[test]
fn standard_alias_rejects_invalid_grammar_and_original_checked_closures() {
    for alias in ["-webkit-user-select", "-WEBKIT-USER-SELECT"] {
        for text in [
            "element",
            "toggle",
            "auto text",
            "initial auto",
            "text all",
            "25%",
            "calc(1 + 2)",
        ] {
            invalid(alias, text);
        }
        for text in ["ALL/*", "initial/*", "var(--ui", "env(ui", "attr(data-ui *"] {
            let components = parse_component_values(text).unwrap();
            let before = components.clone();
            let origin = implicit_origin(&components);
            for grammar in [false, true] {
                assert_closure(
                    &checked_components(alias, components.clone(), grammar).unwrap_err(),
                    &origin,
                    text,
                );
            }
            assert_eq!(components, before);
        }
        for grammar in [false, true] {
            let source = checked(alias, "ALL/**/", grammar);
            assert_eq!(source.known().unwrap().property(), property("user-select"));
            assert_eq!(
                source.to_specified_css().unwrap(),
                "user-select: all !important;"
            );
        }
    }
}
#[test]
fn finite_domains_and_malformed_whole_values_reject_atomically_preserving_color_neighbors() {
    for name in FAMILY {
        for value in [
            "",
            "bogus",
            "auto auto",
            "1",
            "-1px",
            "25%",
            "calc(1 + 2)",
            "auto,auto",
            "[auto]",
            "\"auto\"",
            "initial auto",
            "auto inherit",
        ] {
            invalid(name, value);
        }
    }
    for (name, values) in [
        (
            "pointer-events",
            &[
                "visiblePainted",
                "visible",
                "all",
                "bounding-box",
                "fill",
                "stroke",
                "painted",
            ][..],
        ),
        (
            "user-select",
            &["element", "toggle", "auto text", "text all", "all all"][..],
        ),
        (
            "resize",
            &[
                "auto",
                "resize",
                "x",
                "y",
                "block inline",
                "both horizontal",
                "vertical vertical",
            ][..],
        ),
        (
            "caret-color",
            &[
                "none",
                "red blue",
                "url(a)",
                "rgb(1 2)",
                "rgb(1px 2 3)",
                "color(not-a-space 1 0 0)",
            ][..],
        ),
    ] {
        for value in values {
            invalid(name, value);
        }
    }
}
#[test]
fn cursor_image_grammar_retains_order_omission_signed_math_and_standard_function_alias() {
    for &(input, canonical) in CURSOR_SPECIFIED_WITNESSES {
        accepted("cursor", input, canonical);
    }
}
#[test]
fn incomplete_hotspots_missing_fallbacks_and_invalid_url_sets_drop_only_the_cursor() {
    for text in [
        "url(a)",
        "url(a),",
        "url(a) 1, auto",
        "url(a) 1 2 3,auto",
        "url(a) 1px 2,auto",
        "url(a) 1 2%,auto",
        "url(a) calc(1px) 2,auto",
        "url(a) 1 calc(2%),auto",
        "url(a) 1 2",
        "url(a) 1 2,url(b) 3,auto",
        "image-set(url(a)) 1,auto",
        "image-set(),auto",
        "image-set(url(a) 1x,),auto",
        "image-set(linear-gradient(red,blue) 1x),auto",
        "image-set(image-set(url(a)) 1x),auto",
        "image-set(url(a) -1x),auto",
        "image-set(\"a\" -1e999dpi),auto",
        "image-set(url(a) 1x 2x),auto",
        "image-set(url(a) type(\"a\") type(\"b\")),auto",
        "image-set(url(a) type()),auto",
        "image-set(url(a) type(image/png)),auto",
        "image-set(url(a) type(\"a\" \"b\")),auto",
        "-webkit-image-set(linear-gradient(red,blue) 1x),auto",
        "pointer url(a)",
        "pointer,pointer",
        "url(a),1",
        "url(a),inherit",
    ] {
        invalid("cursor", text);
    }
}
#[test]
fn signed_extreme_number_hotspots_retain_exact_literals_without_clamping_or_underflow() {
    for (text, spelling) in [
        ("url(a) -1e999 1e-999,auto", ["-1e999", "1e-999"]),
        ("url(a) -1e-10000 1e10000,auto", ["-1e-10000", "1e10000"]),
    ] {
        for source in fronts("cursor", text) {
            let before = source.clone();
            let CssKnownPropertyValueRef::Cursor(wrapper) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("cursor")
            };
            let CssCursor::Images(images) = wrapper.value() else {
                panic!("authored images")
            };
            let pair = images.images()[0].hotspot().unwrap();
            for (number, expected) in pair.iter().zip(spelling) {
                assert!(number.calculation().is_none());
                let CssComponentValueRef::Token(CssValueTokenRef::Number(actual)) =
                    number.literal_component().unwrap().view()
                else {
                    panic!("exact Number root")
                };
                assert_eq!(actual.representation(), expected);
                let CssValueOrigin::Parsed(origin) = number.origin() else {
                    panic!("original literal origin")
                };
                assert!(origin.source().as_str().contains(expected));
            }
            completed(&source);
            assert_eq!(source, before);
        }
    }
}

#[test]
fn caret_imports_complete_color_families_and_delegates_canonical_child_output() {
    // UI4 imports the selected Color4/5 authored domain. Explicit expectations
    // follow Color4 sections16.2-16.5 and Color5 section11, independently of the
    // child provider. Supplied components also permit exact origin-bearing graph
    // comparison, which is a composition assertion rather than a text oracle.
    for &(text, expected) in CARET_SPECIFIED_WITNESSES {
        let supplied = parse_component_values(text).unwrap();
        let color = checked_components("color", supplied.clone(), true).unwrap();
        let CssKnownPropertyValueRef::Color(child) =
            color.known().unwrap().property_value().unwrap()
        else {
            panic!("shared Color")
        };
        assert_eq!(child.value().to_specified_css().unwrap(), expected);
        for source in fronts("caret-color", text) {
            let CssKnownPropertyValueRef::CaretColor(caret) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("CaretColor")
            };
            let CssCaretColor::Color(child) = caret.caret() else {
                panic!("transparent Color carrier")
            };
            assert_eq!(child.to_specified_css().unwrap(), expected);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("caret-color: {expected} !important;")
            );
            completed(&source);
        }
        for grammar in [false, true] {
            let caret = checked_components("caret-color", supplied.clone(), grammar).unwrap();
            let CssKnownPropertyValueRef::CaretColor(wrapper) =
                caret.known().unwrap().property_value().unwrap()
            else {
                panic!("caret wrapper")
            };
            let CssCaretColor::Color(actual) = wrapper.caret() else {
                panic!("color branch")
            };
            assert_eq!(actual.as_ref(), child.value());
            assert_eq!(caret.value_components(), &supplied);
        }
    }
}
#[test]
fn original_function_closure_is_strict_for_imported_colors_cursor_sets_and_math() {
    for (name, text) in [
        ("caret-color", "rgb(1 2 3"),
        ("caret-color", "color-mix(in oklab,red,blue"),
        ("caret-color", "contrast-color(red"),
        ("caret-color", "alpha(from red / .5"),
        ("caret-color", "device-cmyk(0 1 1 0"),
        ("cursor", "image-set(url(a) 1x"),
        ("cursor", "url(a) calc(1 + 2"),
    ] {
        let components = parse_component_values(text).unwrap();
        let before = components.clone();
        let origin = implicit_origin(&components);
        for grammar in [false, true] {
            assert_closure(
                &checked_components(name, components.clone(), grammar).unwrap_err(),
                &origin,
                text,
            );
        }
        let source = checked(name, "var(--ui)", true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for _ in 0..2 {
            let error = handle.reenter(components.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("closure precedes grammar")
            };
            assert_closure(error, &origin, text);
        }
        assert!(
            handle
                .reenter(parse_component_values(initial(name)).unwrap())
                .is_ok()
        );
        assert_eq!(components, before);
    }
}
#[test]
fn imported_replacements_retain_supplied_graphs_origins_and_original_occurrences() {
    for (name, text) in [
        ("caret-color", "rgb(1 2 3 / .5)"),
        ("caret-color", "color-mix(in oklab,red,blue)"),
        ("caret-color", "contrast-color(hsl(0 100% 50%))"),
        ("caret-color", "alpha(from red / 50%)"),
        ("caret-color", "device-cmyk(0% 81% 81% 30%)"),
        (
            "cursor",
            "image-set(\"a\" type(\"image/png\") 96.000dpi) -3.500 +4.250, pointer",
        ),
        ("cursor", "url(a) calc(1 + 2) calc(-.25),auto"),
    ] {
        for pending in ["var(--ui)", "env(ui)", "attr(data-ui *)"] {
            let source = parsed(name, pending);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            let replacement = parse_component_values(text).unwrap();
            let snapshot = replacement.clone();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("one ordinary")
            };
            assert_eq!(values.items().len(), 1);
            let item = &values.items()[0];
            assert_source(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let checked = checked_components(name, replacement.clone(), true).unwrap();
            let ordinary = completed(&checked);
            assert_eq!(item.ordinary_value(), ordinary.items()[0].ordinary_value());
            for (actual, supplied) in item
                .replacement_components()
                .unwrap()
                .items()
                .iter()
                .zip(replacement.items())
            {
                assert_eq!(actual.origin(), supplied.origin());
                let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                    (actual.origin(), supplied.origin())
                else {
                    panic!("replacement source")
                };
                assert!(actual.source().same_snapshot(supplied.source()));
                assert_eq!(actual.source().as_str(), text);
            }
            for residual in [
                "rgb(var(--again) 0 0)",
                "image-set(url(a) var(--again)),auto",
                "bogus env(again)",
                "url(a) calc(attr(data-again type(<number>))),auto",
            ] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
            assert_eq!(replacement, snapshot);
            assert_eq!(source, before);
            assert!(handle.reenter(replacement).is_ok());
        }
    }
}
#[test]
fn cursor_checked_payload_retains_order_omissions_descriptors_and_exact_scalar_origins() {
    let text = "image-set(\"first.png\" type(\"image/png\") 96.000dpi,url(second.png)) -3.500 +4.250,url(last.cur),pointer";
    let supplied = parse_component_values(text).unwrap();
    for grammar in [false, true] {
        let source = checked_components("cursor", supplied.clone(), grammar).unwrap();
        let CssKnownPropertyValueRef::Cursor(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("cursor wrapper")
        };
        let CssCursor::Images(images) = wrapper.value() else {
            panic!("images")
        };
        assert_eq!(images.fallback(), CssCursorKeyword::Pointer);
        assert_eq!(images.images().len(), 2);
        assert!(images.images()[1].hotspot().is_none());
        let CssCursorImageSource::UrlSet(set) = images.images()[0].source() else {
            panic!("set")
        };
        assert_eq!(set.options().len(), 2);
        assert!(
            matches!(set.options()[0].reference(),CssCursorUrlSetReference::String(v) if v.as_str()=="first.png")
        );
        assert!(
            matches!(set.options()[1].reference(),CssCursorUrlSetReference::Url(v) if v.as_str()=="second.png")
        );
        assert!(matches!(
            set.options()[0].descriptors(),
            [
                CssCursorUrlSetDescriptor::Type(_),
                CssCursorUrlSetDescriptor::Resolution(_)
            ]
        ));
        assert!(set.options()[1].resolution().is_none());
        assert_eq!(set.options()[0].image_type().unwrap().as_str(), "image/png");
        let resolution = set.options()[0].resolution().unwrap().literal().unwrap();
        assert_eq!(resolution.numeric().representation(), "96.000");
        for (origin, spelling) in [
            (resolution.origin(), "96.000dpi"),
            (images.images()[0].hotspot().unwrap()[0].origin(), "-3.500"),
            (images.images()[0].hotspot().unwrap()[1].origin(), "+4.250"),
        ] {
            let CssValueOrigin::Parsed(origin) = origin else {
                panic!("original scalar")
            };
            assert_eq!(origin.source().as_str(), text);
            assert_eq!(
                origin.span().start().byte_offset().value(),
                text.find(spelling).unwrap()
            );
            assert_eq!(
                origin.span().end().byte_offset().value(),
                text.find(spelling).unwrap() + spelling.len()
            );
        }
        assert_eq!(source.value_components(), &supplied);
        assert_eq!(
            source.to_specified_css().unwrap(),
            "cursor: image-set(\"first.png\" 1dppx type(\"image/png\"), url(\"second.png\")) -3.5 4.25, url(\"last.cur\"), pointer !important;"
        );
    }
}
#[test]
fn normalization_keeps_ui_order_contexts_alias_pending_importance_and_exact_counts() {
    let report = parse_sheet(
        "@media screen{.a{caret-color:auto!important;cursor:pointer;pointer-events:none;user-select:all;resize:block;-webkit-user-select:var(--ui);cursor:unset!important;resize:inline;color:red}}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 9, 9).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    let names = [
        "caret-color",
        "cursor",
        "pointer-events",
        "user-select",
        "resize",
        "user-select",
        "cursor",
        "resize",
        "color",
    ];
    assert_eq!(declarations.len(), names.len());
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            names[index]
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 || index == 6 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        if index > 0 {
            assert!(
                !item
                    .source()
                    .same_occurrence(declarations[index - 1].source())
            );
        }
        match item.expansion() {
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 5);
                assert!(handle.source().same_occurrence(item.source()));
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1);
                assert_source(&values.items()[0], item.source());
                assert_eq!(
                    values.items()[0].property(),
                    item.source().known().unwrap().property()
                );
                if index == 6 {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                    ));
                } else {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Ordinary(_)
                    ));
                }
            }
            _ => panic!("one terminal or handle"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 2, 9, 8).unwrap(),
            CssNormalizationResource::Contributions,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 8, 9).unwrap(),
            CssNormalizationResource::Declarations,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit: 8 }
            );
            assert_eq!(error.declaration_order(), Some(8));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[8].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
fn primitive_css(
    source: &CssDeclaration,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::CaretColor(v) => {
            v.caret().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Cursor(v) => v.value().serialize_specified_with_limits(limits),
        CssKnownPropertyValueRef::PointerEvents(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::UserSelect(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Resize(v) => v.resize().serialize_specified_with_limits(limits),
        _ => panic!("owning UI provider"),
    }
}
fn existing_provider_witnesses(name: &str, witnesses: &[(&str, &str)]) {
    for &(input, expected) in witnesses {
        for source in fronts(name, input) {
            let before = source.clone();
            let supplied = source.value_components().clone();
            assert_eq!(
                primitive_css(&source, CssSpecifiedValueSerializationLimits::default()).unwrap(),
                expected,
                "{name}:{input}"
            );
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {expected} !important;"),
                "{name}:{input}"
            );
            assert_eq!(source.value_components(), &supplied);
            assert_eq!(source, before);
        }
    }
}
#[test]
fn existing_cursor_providers_emit_every_explicit_golden_without_intrinsic_metadata() {
    existing_provider_witnesses("cursor", CURSOR_SPECIFIED_WITNESSES);
}
#[test]
fn existing_caret_providers_emit_every_explicit_golden_without_intrinsic_metadata() {
    existing_provider_witnesses("caret-color", CARET_SPECIFIED_WITNESSES);
}

#[test]
fn existing_primitive_providers_charge_exact_keyword_and_cursor_child_work_atomically() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let mut cases: Vec<(&str, &str, &str, usize)> = Vec::new();
    for name in FAMILY {
        for text in keywords(name) {
            cases.push((name, text, text, 1));
        }
    }
    // URL owns2; Number/Resolution owns1. List/entry/set/option each1,
    // fallback1; string reference1; type function plus string2.
    cases.extend([
        (
            "cursor",
            "url(a) 3.000e1 -0.25,pointer",
            "url(\"a\") 30 -0.25, pointer",
            7,
        ),
        (
            "cursor",
            "image-set(url(a) type(\"image/png\") 1x),auto",
            "image-set(url(\"a\") 1dppx type(\"image/png\")), auto",
            10,
        ),
        (
            "cursor",
            "image-set(\"a\" type(\"image/png\") 1x),auto",
            "image-set(\"a\" 1dppx type(\"image/png\")), auto",
            9,
        ),
    ]);
    for (name, input, expected, nodes) in cases {
        let source = checked(name, input, true);
        let before = source.clone();
        let exact = Limits::new(nodes, nodes, expected.len());
        assert_eq!(primitive_css(&source, exact).unwrap(), expected);
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes, nodes, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(primitive_css(&source, limits).unwrap_err().kind(), kind);
                assert_eq!(source, before);
            }
        }
        assert_eq!(primitive_css(&source, exact).unwrap(), expected);
        let declaration = format!("{name}: {expected} !important;");
        let exact = Limits::new(nodes + 2, nodes + 2, declaration.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            declaration
        );
        for (limits, kind) in [
            (
                Limits::new(nodes + 1, nodes + 2, declaration.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes + 2, nodes + 1, declaration.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes + 2, nodes + 2, declaration.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(source, before);
    }
}
#[test]
fn huge_rgb_scalar_suppression_uses_only_final_output_bytes_and_keeps_source_reusable() {
    use CssSpecifiedValueSerializationLimits as Limits;
    let source = checked("caret-color", "rgb(1e2000000 0 0)", true);
    let before = source.clone();
    let expected = "caret-color: rgb(255, 0, 0) !important;";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for _ in 0..2 {
        assert_eq!(
            source
                .to_specified_css_with_limits(Limits::new(
                    usize::MAX,
                    usize::MAX,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(source, before);
    }
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
}
#[test]
fn sheet_siblings_share_final_bytes_and_nodes_and_remain_reusable_after_failure() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let report = parse_sheet(
        ".a{caret-color:auto!important;cursor:url(a) 30 -.25,pointer}.b{pointer-events:none;-webkit-user-select:all}.c{resize:block}",
    );
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { caret-color: auto !important; cursor: url(\"a\") 30 -0.25, pointer; }\n.b { pointer-events: none; user-select: all; }\n.c { resize: block; }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|rule| rule.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(Kind::ByteLimit)
        );
        assert_eq!(error.rule_index(), Some(2));
        assert_eq!(sheet, &before);
    }
    // Each keyword declaration is3 and the cursor declaration is9. Four keyword
    // siblings alone are12; the whole sheet must exceed9 even before rule work.
    // Exact rule/selector graph totals stay with their existing owning contracts.
    for limits in [
        Limits::new(9, usize::MAX, expected.len()),
        Limits::new(usize::MAX, 9, expected.len()),
    ] {
        let error = sheet.to_specified_css_with_limits(limits).unwrap_err();
        assert!(matches!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                Kind::InputNodeLimit | Kind::ProjectionNodeLimit
            )
        ));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
