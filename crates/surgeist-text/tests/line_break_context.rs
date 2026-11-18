#![forbid(unsafe_code)]

//! Nonlocal Unicode context survives continuous source and computed-style seams.
//!
//! Independent oracles: UAX14 revision 55 LB30a pairs a logical RI run from its
//! beginning, so four supplementary RIs permit only UTF-16 cut4. LB14 keeps an
//! opening parenthesis with following text even across spaces. UAX29 revision47
//! GB12/GB13 independently gives the two whole flag graphemes [0,4) and [4,8).
//! https://www.unicode.org/reports/tr14/tr14-55.html#LB30a
//! https://www.unicode.org/reports/tr14/tr14-55.html#LB14
//! https://www.unicode.org/reports/tr29/tr29-47.html#GB12
//!
//! Preparation uses registered Ahem, but these are opportunity contracts: Ahem
//! does not supply RI glyphs, and no glyph coverage or line-fit oracle is assumed.

use surgeist_text as t;

const FLAGS: &str = "\u{1f1fa}\u{1f1f8}\u{1f1e8}\u{1f1e6}";
const OPENING: &str = "(  A";
const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const DIRECTIONS: [t::paragraph::ParagraphBaseDirection; 2] = [
    t::paragraph::ParagraphBaseDirection::LeftToRight,
    t::paragraph::ParagraphBaseDirection::RightToLeft,
];
const MODES: [t::style::WordBreak; 3] = [
    t::style::WordBreak::Normal,
    t::style::WordBreak::KeepAll,
    t::style::WordBreak::BreakAll,
];

#[derive(Clone, Copy, Debug)]
enum Seam {
    Uniform,
    Style(usize),
    Source(usize),
}

fn style(mode: t::style::WordBreak) -> t::style::Style {
    t::style::Style {
        font: t::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::style::LineHeight::Absolute(10.0),
        white_space: t::style::WhiteSpace::Preserve,
        wrap: t::style::Wrap::Word,
        word_break: mode,
        hyphens: t::hyphenation::Hyphens::None,
        ..t::style::Style::default()
    }
}

fn prepare(
    value: &str,
    preceding: t::style::WordBreak,
    following: t::style::WordBreak,
    seam: Seam,
    direction: t::paragraph::ParagraphBaseDirection,
) -> t::prepared::PreparedText {
    let base = style(preceding);
    let mut source = t::source::Source::identified(
        t::source::Text::from(value),
        t::source::Id::from_u64(65101),
        1,
    );
    let end = source.text().len();
    let mut suffix = style(following);
    if matches!(seam, Seam::Style(_)) {
        // A paint-only style change can leave word-break unchanged.
        suffix.brush = t::style::Brush::color(1.0, 0.0, 0.0, 1.0);
    }
    if let Seam::Style(at) | Seam::Source(at) = seam {
        source.span(t::source::Range::new(at, end), suffix.clone());
    }
    let mut input = t::paragraph::ParagraphInput {
        source: source.clone(),
        base_style: base.clone(),
        segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
        terminal_edge: t::paragraph::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![t::paragraph::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: t::source::Range::new(0, end),
            aggregate: t::source::Range::new(0, end),
            kind: t::paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    if let Seam::Source(at) = seam {
        input.origins.clear();
        let units = value.encode_utf16().collect::<Vec<_>>();
        for (index, (start, stop, computed)) in
            [(0, at, base), (at, end, suffix)].into_iter().enumerate()
        {
            let local = t::source::Range::new(0, stop - start);
            let mut original = t::source::Source::identified(
                t::source::Text::from(String::from_utf16(&units[start..stop]).unwrap()),
                t::source::Id::from_u64(65102 + u64::try_from(index).unwrap()),
                1,
            );
            original.span(local, computed);
            input.origins.push(t::paragraph::ParagraphOrigin {
                text_node: Some(
                    t::paragraph::CompleteTextNodeContribution::new(original.text().clone(), local)
                        .unwrap(),
                ),
                source: original,
                local,
                aggregate: t::source::Range::new(start, stop),
                kind: t::paragraph::ParagraphOriginKind::Authored,
            });
        }
        input.connections.push(t::paragraph::ParagraphConnection {
            at,
            kind: t::paragraph::ParagraphConnectionKind::Continuous,
        });
        // Both inline children and their actual enclosing block permit wrapping.
        input
            .boundary_policies
            .push(t::paragraph::ParagraphBoundaryPolicy {
                definition: t::paragraph::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: t::style::Wrap::Word,
                following_wrap: t::style::Wrap::Word,
                ancestor_wrap: t::style::Wrap::Word,
            });
    }
    let analyzed = t::analyzed::AnalyzedSource::from_paragraph(
        input,
        t::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: t::style::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::default();
    system
        .register_fonts(AHEM.to_vec(), crate::common::FONT_LIMITS)
        .unwrap();
    let prepared = system
        .prepare(&analyzed, t::prepared::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    prepared
}

fn ordinary_cuts(prepared: &t::prepared::PreparedText) -> Vec<usize> {
    prepared
        .opportunity_views()
        .filter(|offer| offer.kind() == t::prepared::PreparedBreakKind::Allowed)
        .map(|offer| {
            assert_eq!(offer.alternatives().count(), 0);
            let boundary = offer.boundary();
            assert_eq!(
                boundary.view().source_before(),
                boundary.view().source_after()
            );
            boundary.view().source_after().get()
        })
        .collect()
}

fn flag_cuts(prepared: &t::prepared::PreparedText) -> Vec<usize> {
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [t::source::Range::new(0, 4), t::source::Range::new(4, 8)],
        "GB12/GB13 pair RIs from the logical sequence start"
    );
    let cuts = ordinary_cuts(prepared);
    assert!(
        cuts.iter().all(|cut| *cut != 2 && *cut != 6),
        "whole flag typographic units forbid their interior RI cuts: {cuts:?}"
    );
    cuts
}

#[test]
fn uniform_word_break_modes_preserve_the_break_between_whole_flags() {
    for direction in DIRECTIONS {
        for mode in MODES {
            let prepared = prepare(FLAGS, mode, mode, Seam::Uniform, direction);
            assert_eq!(flag_cuts(&prepared), [4], "{direction:?} {mode:?}");
        }
    }
}

#[test]
fn paint_only_style_seams_preserve_ri_parity_and_whole_flags() {
    for direction in DIRECTIONS {
        for mode in MODES {
            for at in [2, 4, 6] {
                let prepared = prepare(FLAGS, mode, mode, Seam::Style(at), direction);
                assert_eq!(flag_cuts(&prepared), [4], "{direction:?} {mode:?} at{at}");
            }
        }
    }
}

#[test]
fn continuous_source_seams_preserve_ri_parity_and_whole_flags() {
    for direction in DIRECTIONS {
        for mode in MODES {
            for at in [2, 4, 6] {
                let prepared = prepare(FLAGS, mode, mode, Seam::Source(at), direction);
                assert_eq!(flag_cuts(&prepared), [4], "{direction:?} {mode:?} at{at}");
            }
        }
    }
}

fn check_mixed_flags(source_seam: bool) {
    let mut mismatches = Vec::new();
    for direction in DIRECTIONS {
        for (preceding, following) in [
            (t::style::WordBreak::Normal, t::style::WordBreak::KeepAll),
            (t::style::WordBreak::KeepAll, t::style::WordBreak::Normal),
        ] {
            for at in [2, 4, 6] {
                let seam = if source_seam {
                    Seam::Source(at)
                } else {
                    Seam::Style(at)
                };
                let prepared = prepare(FLAGS, preceding, following, seam, direction);
                let cuts = flag_cuts(&prepared);
                if cuts != [4] {
                    mismatches.push((direction, preceding, following, seam, cuts));
                }
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "LB30a requires exactly cut4 in the full logical RI run: {mismatches:?}"
    );
}

#[test]
fn word_break_style_changes_preserve_full_logical_ri_parity() {
    check_mixed_flags(false);
}

#[test]
fn word_break_changes_at_continuous_source_seams_preserve_full_logical_ri_parity() {
    check_mixed_flags(true);
}

#[test]
fn uniform_word_break_modes_keep_opening_punctuation_with_text_after_spaces() {
    for direction in DIRECTIONS {
        for mode in MODES {
            let prepared = prepare(OPENING, mode, mode, Seam::Uniform, direction);
            assert_eq!(ordinary_cuts(&prepared), [], "{direction:?} {mode:?}");
        }
    }
}

#[test]
fn unchanged_mode_seams_keep_opening_punctuation_context() {
    for direction in DIRECTIONS {
        for mode in MODES {
            for seam in [Seam::Style(2), Seam::Source(2)] {
                let prepared = prepare(OPENING, mode, mode, seam, direction);
                assert_eq!(
                    ordinary_cuts(&prepared),
                    [],
                    "{direction:?} {mode:?} {seam:?}"
                );
            }
        }
    }
}

#[test]
fn word_break_changes_preserve_opening_punctuation_context_across_spaces() {
    let mut mismatches = Vec::new();
    for direction in DIRECTIONS {
        for seam in [Seam::Style(2), Seam::Source(2)] {
            let prepared = prepare(
                OPENING,
                t::style::WordBreak::Normal,
                t::style::WordBreak::KeepAll,
                seam,
                direction,
            );
            let cuts = ordinary_cuts(&prepared);
            if !cuts.is_empty() {
                mismatches.push((direction, seam, cuts));
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "LB14 OP SP* protection forbids cut3 before A in '(  A': {mismatches:?}"
    );
}

mod common;
