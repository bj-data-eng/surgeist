//! One location for every property or descriptor value that holds no authored
//! token (crate reference, "Source coordinates and diagnostics"): a value of
//! only whitespace and comments has no responsible token, and its rejection
//! sits right after that trivia, where the `;`, `!important`, `}` or end of
//! input that follows the value begins. Syntax 3 section 5.5.6 (consume a
//! declaration) discards the whitespace after the colon, consumes the value
//! from the next token and removes the `!important` that ends it, so an empty
//! value begins and ends there; trivia is never responsible, and the next
//! authored token or the end of input locates it. The rejection names no
//! token, for every property and descriptor, in every front.
//!
//! The expected offset is built from the literal source, never read from the
//! parser.

use surgeist_css::CssParser;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::property::{
    CssKnownProperty, CssPropertyNameRef, CssPropertyTextTarget,
    CssSvgGlyphOrientationVerticalDeclaration,
};

/// Values with no authored token: empty, whitespace of every kind, comments,
/// and both mixed.
const BLANKS: &[&str] = &[
    "",
    " ",
    "   ",
    "\t\n\r\u{c} ",
    "/*c*/",
    "  /*c*/ ",
    "/*c*/  ",
    "/*a*//*b*/",
    " /*a*/ /*b*/ ",
    "/**/",
];

/// What ends the declaration after the blank value.
const ENDINGS: &[&str] = &["", ";", ";width:1px", " ;"];

/// The properties the issue observed at different positions.
const OBSERVED: &[&str] = &[
    "margin",
    "flex",
    "display",
    "voice-rate",
    "voice-pitch",
    "voice-range",
    "outline-offset",
    "aspect-ratio",
    "pause",
    "cue",
    "will-change",
    "color",
];

fn every_property_spelling() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = OBSERVED.to_vec();
    for property in CssKnownProperty::all() {
        names.push(property.canonical_name());
        names.extend(property.aliases().iter().copied());
    }
    names.extend([
        "page-break-before",
        "page-break-after",
        "page-break-inside",
        "glyph-orientation-vertical",
    ]);
    names.sort_unstable();
    names.dedup();
    names
}

/// The first diagnostic of `report`: its code, its offset and its token.
#[track_caller]
fn first_error<T>(
    source: &str,
    report: &surgeist_css::CssParseReport<T>,
) -> (CssErrorCode, usize, Option<String>) {
    let diagnostic = report
        .diagnostics()
        .first()
        .unwrap_or_else(|| panic!("a diagnostic for {source:?}"));
    let error = diagnostic.error();
    (
        error.code(),
        error.position().byte_offset().value(),
        error.encountered().map(|token| token.authored().to_owned()),
    )
}

/// The whitespace that opens `ending`, which is part of the blank value.
fn trivia_of(ending: &str) -> usize {
    ending.len() - ending.trim_start().len()
}

#[track_caller]
fn assert_after_trivia(
    source: &str,
    expected: usize,
    found: (CssErrorCode, usize, Option<String>),
) {
    assert_eq!(
        found,
        (CssErrorCode::InvalidPropertyValue, expected, None),
        "{source:?}: a blank value is rejected after its trivia, naming no token"
    );
}

#[test]
fn a_blank_value_is_rejected_after_its_trivia_for_every_observed_property() {
    for name in OBSERVED {
        for blank in BLANKS {
            for ending in ENDINGS {
                let source = format!("{name}:{blank}{ending}");
                let report = CssParser::default().parse_block_contents(&source);
                assert_after_trivia(
                    &source,
                    name.len() + 1 + blank.len() + trivia_of(ending),
                    first_error(&source, &report),
                );
                assert!(
                    report.syntax().as_slice().iter().all(|declaration| {
                        declaration
                            .known()
                            .map(|known| known.property().canonical_name())
                            != Some(CssKnownProperty::from_name(name).unwrap().canonical_name())
                    }),
                    "{source:?}: the blank declaration is dropped"
                );
            }
        }
    }
}

#[test]
fn a_blank_value_is_rejected_after_its_trivia_for_every_known_property_and_alias() {
    let names = every_property_spelling();
    assert!(names.len() > 400, "the sweep covers the property registry");
    for name in names {
        for blank in BLANKS {
            for ending in ENDINGS {
                let source = format!("{name}:{blank}{ending}");
                let report = CssParser::default().parse_block_contents(&source);
                assert_after_trivia(
                    &source,
                    name.len() + 1 + blank.len() + trivia_of(ending),
                    first_error(&source, &report),
                );
            }
        }
    }
}

#[test]
fn a_blank_value_after_other_declarations_is_rejected_after_its_own_trivia() {
    let prefix = "color:red; /*c*/ ";
    for name in every_property_spelling() {
        for blank in ["", "  ", " /*c*/ "] {
            for ending in ["", ";", ";width:1px"] {
                let source = format!("{prefix}{name}:{blank}{ending}");
                let report = CssParser::default().parse_block_contents(&source);
                assert_after_trivia(
                    &source,
                    prefix.len() + name.len() + 1 + blank.len() + trivia_of(ending),
                    first_error(&source, &report),
                );
            }
        }
    }
}

#[test]
fn a_blank_value_in_a_style_rule_is_rejected_after_its_trivia() {
    for name in OBSERVED {
        for blank in ["", "  ", "/*c*/", " /*c*/ "] {
            for ending in [" }", "; }", ";}"] {
                let head = ".a {";
                let source = format!("{head}{name}:{blank}{ending}");
                let report = CssParser::default().parse_sheet(&source);
                assert_after_trivia(
                    &source,
                    head.len() + name.len() + 1 + blank.len() + trivia_of(ending),
                    first_error(&source, &report),
                );
            }
        }
    }
}

#[test]
fn a_blank_value_text_is_rejected_after_its_trivia() {
    for property in CssKnownProperty::all() {
        for blank in BLANKS {
            let report = CssParser::default().parse_property_value(
                blank,
                CssPropertyTextTarget::Property(CssPropertyNameRef::Known(*property)),
                CssImportance::Normal,
            );
            assert_after_trivia(
                &format!("{} <- {blank:?}", property.canonical_name()),
                blank.len(),
                first_error(blank, &report),
            );
        }
    }
}

#[test]
fn a_blank_value_in_the_utf16_front_is_rejected_where_the_text_front_rejects_it() {
    use std::sync::Arc;
    use surgeist_css::source::CssUtf16InputLimits;
    for name in every_property_spelling() {
        for blank in ["", "  ", " /*c*/ "] {
            for ending in ["", ";", ";width:1px"] {
                let source = format!("{name}:{blank}{ending}");
                let units = Arc::new(source.encode_utf16().collect::<Vec<u16>>());
                let report = CssParser::default()
                    .parse_block_contents_utf16(units, CssUtf16InputLimits::default(), || {
                        Ok::<_, ()>(())
                    })
                    .unwrap();
                assert_after_trivia(
                    &source,
                    name.len() + 1 + blank.len() + trivia_of(ending),
                    first_error(&source, report.report()),
                );
            }
        }
    }
}

#[test]
fn a_blank_value_in_a_keyframe_block_is_rejected_after_its_trivia() {
    for name in OBSERVED {
        for blank in ["", "  ", " /*c*/ "] {
            let head = "@keyframes k { from {";
            let source = format!("{head}{name}:{blank}; }} }}");
            let report = CssParser::default().parse_sheet(&source);
            assert_after_trivia(
                &source,
                head.len() + name.len() + 1 + blank.len(),
                first_error(&source, &report),
            );
        }
    }
}

#[test]
fn a_blank_value_in_a_page_rule_is_rejected_after_its_trivia() {
    for name in ["margin", "width", "color", "quotes", "overflow"] {
        for blank in ["", "  ", " /*c*/ "] {
            let head = "@page {";
            let source = format!("{head}{name}:{blank}; }}");
            let report = CssParser::default().parse_sheet(&source);
            assert_after_trivia(
                &source,
                head.len() + name.len() + 1 + blank.len(),
                first_error(&source, &report),
            );
            let contents = format!("{name}:{blank}");
            let report = CssParser::default().parse_page_block_contents(&contents);
            assert_after_trivia(
                &contents,
                name.len() + 1 + blank.len(),
                first_error(&contents, &report),
            );
        }
    }
}

#[test]
fn a_blank_value_before_important_is_rejected_at_the_bang_naming_no_token_for_every_property() {
    for name in every_property_spelling() {
        for blank in ["", "  ", " /*c*/ "] {
            for importance in ["!important", "!IMPORTANT", " !important ;width:1px"] {
                let source = format!("{name}:{blank}{importance}");
                let report = CssParser::default().parse_block_contents(&source);
                assert_after_trivia(
                    &source,
                    name.len() + 1 + blank.len() + trivia_of(importance),
                    first_error(&source, &report),
                );
            }
        }
    }
}

#[test]
fn a_blank_value_in_the_single_declaration_front_is_rejected_after_its_trivia() {
    for name in OBSERVED {
        for (blank, ending) in [
            ("", ""),
            ("  ", ""),
            (" /*c*/ ", ""),
            ("", "!important"),
            (" /*c*/ ", " !important"),
        ] {
            let source = format!(" /*a*/ {name} /*b*/ :{blank}{ending}");
            let report = CssDeclaration::parse(&CssParser::default(), &source);
            let colon = source.find(':').unwrap();
            assert_after_trivia(
                &source,
                colon + 1 + blank.len() + trivia_of(ending),
                first_error(&source, &report),
            );
        }
    }
}

#[test]
fn a_blank_svg_glyph_attribute_value_is_rejected_at_the_end_of_its_text() {
    for blank in ["", " ", "  /*c*/ ", "/*c*/"] {
        let report = CssSvgGlyphOrientationVerticalDeclaration::parse_attribute(
            &CssParser::default(),
            blank,
        );
        let (_, offset, token) = first_error(blank, &report);
        assert_eq!((offset, token), (blank.len(), None), "{blank:?}");
    }
}

#[test]
fn a_blank_value_in_a_nested_rule_is_rejected_after_its_trivia() {
    for name in OBSERVED {
        for blank in ["", "  ", " /*c*/ "] {
            for ending in ["}", ";}", "!important}"] {
                let head = ".a{margin:1px; .b{";
                let source = format!("{head}{name}:{blank}{ending}}}");
                let report = CssParser::default().parse_sheet(&source);
                assert_after_trivia(
                    &source,
                    head.len() + name.len() + 1 + blank.len(),
                    first_error(&source, &report),
                );
            }
        }
    }
}

#[test]
fn a_blank_value_in_a_page_margin_box_is_rejected_after_its_trivia() {
    for name in ["width", "quotes", "overflow"] {
        for blank in ["", "  ", " /*c*/ "] {
            for ending in ["}", ";}", "!important}"] {
                let head = "@page{@top-left{";
                let source = format!("{head}{name}:{blank}{ending}}}");
                let report = CssParser::default().parse_sheet(&source);
                assert_after_trivia(
                    &source,
                    head.len() + name.len() + 1 + blank.len(),
                    first_error(&source, &report),
                );
            }
        }
    }
}

/// What a blank list text of a front that reads no property or descriptor value
/// reports, as its code, offset and token: the rule of this file governs values
/// only, so these keep the position their own grammars give.
fn first_of<T>(
    report: &surgeist_css::CssParseReport<T>,
) -> Option<(CssErrorCode, usize, Option<String>)> {
    report.diagnostics().first().map(|diagnostic| {
        let error = diagnostic.error();
        (
            error.code(),
            error.position().byte_offset().value(),
            error.encountered().map(|token| token.authored().to_owned()),
        )
    })
}

#[test]
fn a_blank_selector_or_query_list_text_keeps_the_position_of_its_own_grammar() {
    use surgeist_css::query::media::{CssMediaQuery, CssMediaQueryList};
    use surgeist_css::rule::keyframes::CssParsedKeyframeSelectors;
    use surgeist_css::rule::page::CssPageSelectorList;

    let parser = CssParser::default();
    for blank in ["", " ", "  ", "/*c*/", " /*c*/ ", "\n\n"] {
        assert_eq!(
            first_of(&CssParsedKeyframeSelectors::parse(&parser, blank)),
            Some((CssErrorCode::UnexpectedEnd, 0, None)),
            "keyframe selectors {blank:?}"
        );
        // A blank page selector list and a blank media query list are valid
        // and empty.
        assert_eq!(
            first_of(&CssPageSelectorList::parse(&parser, blank)),
            None,
            "page selectors {blank:?}"
        );
        assert_eq!(
            first_of(&CssMediaQueryList::parse(&parser, blank)),
            None,
            "media query list {blank:?}"
        );
        // A single media query is rejected by its own grammar after the trivia.
        assert_eq!(
            first_of(&CssMediaQuery::parse(&parser, blank)),
            Some((CssErrorCode::InvalidMediaQuery, blank.len(), None)),
            "media query {blank:?}"
        );
    }
}

#[test]
fn the_position_counts_utf16_columns_and_lines_through_multibyte_trivia() {
    // The `;` follows a comment holding a two-byte and a four-byte character
    // (three UTF-16 units) on the same line.
    let source = "margin: /*é😀*/ ;";
    let report = CssParser::default().parse_block_contents(source);
    let position = report.diagnostics()[0].error().position();
    assert_eq!(
        (
            position.byte_offset().value(),
            position.line().value(),
            position.column().value()
        ),
        (19, 0, 16),
        "{source:?}"
    );

    // The comment spans a line break and the `;` stands on the fourth line.
    let source = "color: red;\nmargin: /*é\n😀*/ \n  ;";
    let report = CssParser::default().parse_block_contents(source);
    let position = report.diagnostics()[0].error().position();
    assert_eq!(
        (
            position.byte_offset().value(),
            position.line().value(),
            position.column().value()
        ),
        (35, 3, 2),
        "{source:?}"
    );
}

/// Every descriptor-bearing at-rule: its header, what its body opens with so
/// the rule stays valid, and the descriptors whose values are read.
const DESCRIPTOR_RULES: &[(&str, &str, &[&str])] = &[
    (
        "@font-face",
        "",
        &[
            "font-family",
            "src",
            "font-weight",
            "font-style",
            "font-width",
            "font-stretch",
            "font-display",
            "unicode-range",
            "font-feature-settings",
            "font-variation-settings",
            "font-named-instance",
            "font-language-override",
            "ascent-override",
            "descent-override",
            "line-gap-override",
        ],
    ),
    ("@page", "", &["size", "page-orientation", "marks", "bleed"]),
    (
        "@counter-style x",
        "",
        &[
            "system",
            "negative",
            "prefix",
            "suffix",
            "range",
            "pad",
            "fallback",
            "symbols",
            "additive-symbols",
            "speak-as",
        ],
    ),
    (
        "@font-palette-values --x",
        "font-family: Demo; ",
        &["base-palette", "override-colors"],
    ),
    (
        "@color-profile --x",
        "",
        &["src", "rendering-intent", "components"],
    ),
    (
        "@font-feature-values Foo",
        "",
        &[
            "@styleset{a",
            "@stylistic{a",
            "@character-variant{a",
            "@swash{a",
            "@ornaments{a",
            "@annotation{a",
        ],
    ),
];

#[track_caller]
fn assert_descriptor_after_trivia(
    source: &str,
    expected: usize,
    found: (CssErrorCode, usize, Option<String>),
) {
    assert_eq!(
        found,
        (CssErrorCode::InvalidDescriptorValue, expected, None),
        "{source:?}: a blank descriptor value is rejected after its trivia, naming no token"
    );
}

#[test]
fn a_blank_descriptor_value_is_rejected_after_its_trivia_in_every_descriptor_rule() {
    for (header, opening, descriptors) in DESCRIPTOR_RULES {
        for descriptor in *descriptors {
            for blank in BLANKS {
                for ending in ["", ";", " ;", "!important", " !important;"] {
                    let close = if descriptor.contains('{') { "}}" } else { "}" };
                    let head = format!("{header}{{{opening}");
                    let source = format!("{head}{descriptor}:{blank}{ending}{close}");
                    let report = CssParser::default().parse_sheet(&source);
                    assert_descriptor_after_trivia(
                        &source,
                        head.len() + descriptor.len() + 1 + blank.len() + trivia_of(ending),
                        first_error(&source, &report),
                    );
                }
            }
        }
    }
}

#[test]
fn a_blank_descriptor_value_is_rejected_after_its_trivia_in_the_raw_contents_fronts() {
    for (header, _, descriptors) in DESCRIPTOR_RULES {
        if !["@font-face", "@page"].contains(header) {
            continue;
        }
        for descriptor in *descriptors {
            for blank in BLANKS {
                for ending in ["", ";", " ;", "!important", " !important;"] {
                    let source = format!("{descriptor}:{blank}{ending}");
                    let found = if *header == "@page" {
                        first_error(
                            &source,
                            &CssParser::default().parse_page_block_contents(&source),
                        )
                    } else {
                        first_error(
                            &source,
                            &CssParser::default().parse_font_face_block_contents(&source),
                        )
                    };
                    assert_descriptor_after_trivia(
                        &source,
                        descriptor.len() + 1 + blank.len() + trivia_of(ending),
                        found,
                    );
                }
            }
        }
    }
}

#[test]
fn a_descriptor_importance_annotation_is_rejected_at_the_bang_naming_it_after_a_valid_value() {
    // No descriptor of these rules admits `!important`: it is judged once the
    // value has been read, at the `!`, which it names.
    for (source, bang) in [
        ("@font-face{font-family: Demo !important}", 29),
        ("@counter-style x{symbols: a !important}", 28),
        (
            "@font-palette-values --x{font-family: Demo; base-palette: 1 !important}",
            60,
        ),
        ("@font-feature-values Foo{@styleset{a: 1 !important}}", 40),
    ] {
        let report = CssParser::default().parse_sheet(source);
        assert_eq!(
            first_error(source, &report),
            (
                CssErrorCode::InvalidDeclarationAnnotation,
                bang,
                Some("!".to_owned())
            ),
            "{source:?}"
        );
    }
}

#[test]
fn a_context_rejection_is_about_the_property_and_is_located_independently_of_its_value() {
    // A property that does not apply in `@page` or a margin box is rejected
    // where its value begins to hold a token, which it names; a blank value
    // has none, so it is rejected after the trivia like any blank value.
    for (source, offset, token) in [
        ("@page{outline-offset: 1px}", 22, Some("1px")),
        ("@page{outline-offset:  }", 23, None),
        ("@page{outline-offset: /*c*/ !important}", 28, None),
        ("@page{@top-left{outline-offset: 1px}}", 32, Some("1px")),
        ("@page{@top-left{outline-offset:  }}", 33, None),
    ] {
        let report = CssParser::default().parse_sheet(source);
        assert_eq!(
            first_error(source, &report),
            (
                CssErrorCode::InvalidPropertyValue,
                offset,
                token.map(str::to_owned)
            ),
            "{source:?}"
        );
    }
    for (source, offset, token) in [
        ("outline-offset: 1px", 16, Some("1px")),
        ("outline-offset:  ", 17, None),
        ("outline-offset:", 15, None),
    ] {
        let report = CssParser::default().parse_page_block_contents(source);
        assert_eq!(
            first_error(source, &report),
            (
                CssErrorCode::InvalidPropertyValue,
                offset,
                token.map(str::to_owned)
            ),
            "page contents {source:?}"
        );
    }
    // A property that cannot be animated is rejected at its own name in a
    // keyframe block, whatever its value, and names it.
    for source in [
        "@keyframes k{from{animation-name: x}}",
        "@keyframes k{from{animation-name:  }}",
        "@keyframes k{from{animation-name:}}",
    ] {
        let report = CssParser::default().parse_sheet(source);
        assert_eq!(
            first_error(source, &report),
            (
                CssErrorCode::InvalidPropertyValue,
                "@keyframes k{from{".len(),
                Some("animation-name".to_owned())
            ),
            "{source:?}"
        );
    }
}
