#![forbid(unsafe_code)]

//! Native public LineBreak contracts. Independent expectations come from the
//! retained CSS Text4 section6.2 mandatory distinctions, AppendixF script rules,
//! section6.5 TCU/wrapping rules, and the adopted actual-owner determination:
//! references/text-anywhere-owner-precedence--determination-20261025.md.
//! CJK tests assert semantic opportunities, never unsupported Ahem CJK metrics.
//! These exercise a new API; they do not claim executable preimplementation RED.

use surgeist_text as t;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const DIRECTIONS: [t::ParagraphBaseDirection; 2] = [
    t::ParagraphBaseDirection::LeftToRight,
    t::ParagraphBaseDirection::RightToLeft,
];
const MODES: [t::LineBreak; 5] = [
    t::LineBreak::Auto,
    t::LineBreak::Loose,
    t::LineBreak::Normal,
    t::LineBreak::Strict,
    t::LineBreak::Anywhere,
];

fn style(mode: t::LineBreak, locale: Option<&str>) -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        line_break: mode,
        locale: locale.map(str::to_owned),
        hyphens: t::Hyphens::None,
        hyphenate_character: t::HyphenateCharacter::String(t::Text::from("X")),
        hyphenate_limit_chars: t::HyphenateLimitChars {
            total: t::HyphenationMinimum::Count(0),
            before: t::HyphenationMinimum::Count(0),
            after: t::HyphenationMinimum::Count(0),
        },
        ..t::Style::default()
    }
}

fn input(value: &str, base: t::Style, direction: t::ParagraphBaseDirection) -> t::ParagraphInput {
    let source = t::Source::identified(t::Text::from(value), t::Id::from_u64(65201), 1);
    let end = source.text().len();
    t::ParagraphInput {
        source: source.clone(),
        base_style: base,
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: t::ParagraphStrut::DefaultFont,
        terminal_edge: t::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![t::ParagraphOrigin {
            text_node: None,
            source,
            local: t::Range::new(0, end),
            aggregate: t::Range::new(0, end),
            kind: t::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn prepare(input: t::ParagraphInput) -> t::PreparedText {
    // Every fixture has an actual enclosing block with this authored Wrap.
    let wrap = input.base_style.wrap;
    let source = input.source.clone();
    let analyzed =
        t::AnalyzedSource::from_paragraph(input, t::AnalysisTypography::HorizontalComposition)
            .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    prepared
}

fn cuts(p: &t::PreparedText, kind: t::PreparedBreakKind) -> Vec<usize> {
    p.opportunity_views()
        .filter(|offer| offer.kind() == kind)
        .map(|offer| offer.boundary().view().source_after().get())
        .collect()
}

fn ordinary(p: &t::PreparedText) -> Vec<usize> {
    cuts(p, t::PreparedBreakKind::Allowed)
}

fn two_origins(input: &mut t::ParagraphInput, at: usize, connected: bool) {
    let source = input.source.clone();
    let end = source.text().len();
    input.origins = [(0, at), (at, end)]
        .into_iter()
        .map(|(start, stop)| t::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: t::Range::new(start, stop),
            aggregate: t::Range::new(start, stop),
            kind: t::ParagraphOriginKind::Authored,
        })
        .collect();
    input.connections.push(t::ParagraphConnection {
        at,
        kind: if connected {
            t::ParagraphConnectionKind::Continuous
        } else {
            t::ParagraphConnectionKind::Disconnected
        },
    });
    input.boundary_policies.push(t::ParagraphBoundaryPolicy {
        definition: t::ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: input.base_style.wrap,
        following_wrap: input.base_style.wrap,
        ancestor_wrap: input.base_style.wrap,
    });
}

#[test]
fn mandatory_cjk_distinctions_apply_to_every_named_character() {
    assert_eq!(t::Style::default().line_break, t::LineBreak::Auto);
    // Columns are Auto, Loose, Normal, Strict, Anywhere. Auto's deterministic
    // Strict resolution is an adopted native policy, rather than a CSS mandate.
    let loose = [false, true, false, false, true];
    let cjk_hyphen = [false, true, true, false, true];
    let mut cases = Vec::new();
    for mark in ['ぁ', 'ァ', 'ㇰ', 'ー'] {
        cases.push((format!("漢{mark}"), loose));
    }
    for mark in ['\u{2010}', '\u{2013}'] {
        cases.push((format!("漢{mark}"), loose));
    }
    for mark in ['\u{301c}', '\u{30a0}'] {
        cases.push((format!("漢{mark}"), cjk_hyphen));
    }
    for mark in ['々', '〻', 'ゝ', 'ゞ', 'ヽ', 'ヾ'] {
        cases.push((format!("漢{mark}"), loose));
    }
    for left in ['‥', '…'] {
        for right in ['‥', '…'] {
            cases.push((format!("{left}{right}"), loose));
        }
    }
    for mark in ['・', '：', '；', '･', '‼', '⁇', '⁈', '⁉', '！', '？'] {
        cases.push((format!("漢{mark}"), loose));
    }
    // PO/PR EAW A,F,W examples; these are not a blanket percent/currency rule.
    use icu_properties::{
        CodePointMapData,
        props::{EastAsianWidth as Width, LineBreak as Class},
    };
    for (mark, width) in [
        ('‰', Width::Ambiguous),
        ('％', Width::Fullwidth),
        ('﹪', Width::Wide),
    ] {
        assert_eq!(
            CodePointMapData::<Class>::new().get32(u32::from(mark)),
            Class::PostfixNumeric
        );
        assert_eq!(
            CodePointMapData::<Width>::new().get32(u32::from(mark)),
            width
        );
        cases.push((format!("漢{mark}"), loose));
    }
    for (mark, width) in [
        ('€', Width::Ambiguous),
        ('＄', Width::Fullwidth),
        ('﹩', Width::Wide),
    ] {
        assert_eq!(
            CodePointMapData::<Class>::new().get32(u32::from(mark)),
            Class::PrefixNumeric
        );
        assert_eq!(
            CodePointMapData::<Width>::new().get32(u32::from(mark)),
            width
        );
        cases.push((format!("{mark}漢"), loose));
    }
    cases.push(("漢%".into(), [false, false, false, false, true]));
    cases.push(("$漢".into(), [false, false, false, false, true]));
    let mut failures = Vec::new();
    for direction in DIRECTIONS {
        for (value, permissions) in &cases {
            for (mode, admitted) in MODES.into_iter().zip(permissions) {
                let p = prepare(input(value, style(mode, Some("ja")), direction));
                let expected = if *admitted { vec![1] } else { vec![] };
                if ordinary(&p) != expected {
                    failures.push((value.clone(), mode, direction, ordinary(&p), expected));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "mandatory CSS Text4 pairs: {failures:?}"
    );
}

#[test]
fn hyphen_permission_uses_effective_preceding_id_class() {
    for direction in DIRECTIONS {
        for mark in ['\u{2010}', '\u{2013}'] {
            for word_break in [t::WordBreak::Normal, t::WordBreak::BreakAll] {
                for mode in MODES {
                    let mut base = style(mode, Some("en"));
                    base.word_break = word_break;
                    let p = prepare(input(&format!("a{mark}"), base, direction));
                    let admitted = mode == t::LineBreak::Anywhere
                        || (mode == t::LineBreak::Loose && word_break == t::WordBreak::BreakAll);
                    assert_eq!(
                        ordinary(&p),
                        if admitted { vec![1] } else { vec![] },
                        "{mode:?} {word_break:?} {mark}"
                    );
                }
            }
        }
    }
}

#[test]
fn full_locale_script_overrides_language_and_preserves_authored_extensions() {
    let locales = [
        (None, false),
        (Some("ja"), true),
        (Some("zh"), true),
        (Some("ja-Latn"), false),
        (Some("zh-Latn"), false),
        (Some("ko-Hani"), true),
        (Some("en-Hira"), true),
        (Some("und-Hani"), true),
        (Some("und"), false),
        (Some("ja-Zzzz"), false),
        (Some("und-Zzzz"), false),
        (Some("ja-Latn-u-nu-latn-x-example"), false),
        (Some("en-Hira-u-nu-latn-x-example"), true),
        (Some("en"), false),
        (Some("ko"), false),
    ];
    for direction in DIRECTIONS {
        for (locale, cjk) in locales {
            for (value, mode) in [
                ("漢〜", t::LineBreak::Normal),
                ("漢゠", t::LineBreak::Normal),
                ("漢・", t::LineBreak::Loose),
                ("漢％", t::LineBreak::Loose),
                ("＄漢", t::LineBreak::Loose),
            ] {
                let p = prepare(input(value, style(mode, locale), direction));
                assert_eq!(p.analyzed_source().style().locale.as_deref(), locale);
                assert_eq!(
                    ordinary(&p),
                    if cjk { vec![1] } else { vec![] },
                    "{locale:?} {value} {mode:?} {direction:?}"
                );
            }
        }
    }
}

#[test]
fn following_logical_text_owns_ordinary_strictness_and_script_across_seams() {
    for direction in DIRECTIONS {
        for connected_sources in [false, true] {
            for (value, before, after, admitted) in [
                (
                    "漢ぁ",
                    style(t::LineBreak::Loose, Some("ja")),
                    style(t::LineBreak::Strict, Some("ja")),
                    false,
                ),
                (
                    "漢ぁ",
                    style(t::LineBreak::Strict, Some("ja")),
                    style(t::LineBreak::Loose, Some("ja")),
                    true,
                ),
                (
                    "漢〜",
                    style(t::LineBreak::Normal, Some("ja")),
                    style(t::LineBreak::Normal, Some("ja-Latn")),
                    false,
                ),
                (
                    "漢〜",
                    style(t::LineBreak::Normal, Some("ja-Latn")),
                    style(t::LineBreak::Normal, Some("ko-Hani")),
                    true,
                ),
            ] {
                let mut request = input(value, before.clone(), direction);
                // An enclosing span and nested following span exercise actual winners.
                request.source.span(t::Range::new(0, 2), before);
                request.source.span(t::Range::new(1, 2), after);
                if connected_sources {
                    two_origins(&mut request, 1, true);
                }
                let p = prepare(request);
                assert_eq!(
                    ordinary(&p),
                    if admitted { vec![1] } else { vec![] },
                    "{direction:?} source_seam={connected_sources} {value}"
                );
            }
        }
    }
}

#[test]
fn anywhere_on_either_adjacent_text_overrides_keep_all_and_text_glue() {
    for direction in DIRECTIONS {
        for (value, seam) in [
            ("ab", 1),
            ("a\u{a0}b", 1),
            ("a\u{2060}b", 1),
            ("a\u{200d}b", 2),
            ("(a", 1),
            ("漢漢", 1),
        ] {
            for anywhere_before in [false, true] {
                let normal = style(t::LineBreak::Strict, None);
                let mut request = input(value, normal.clone(), direction);
                let mut anywhere = style(t::LineBreak::Anywhere, None);
                anywhere.word_break = t::WordBreak::KeepAll;
                request.source.span(
                    if anywhere_before {
                        t::Range::new(0, seam)
                    } else {
                        t::Range::new(seam, value.encode_utf16().count())
                    },
                    anywhere,
                );
                let p = prepare(request);
                assert!(
                    ordinary(&p).contains(&seam),
                    "either text donor: {value:?} before={anywhere_before} {direction:?}"
                );
            }
        }
        let mut normal = style(t::LineBreak::Strict, None);
        normal.word_break = t::WordBreak::KeepAll;
        for value in ["ab", "a\u{a0}b", "a\u{2060}b", "(a", "漢漢"] {
            let p = prepare(input(value, normal.clone(), direction));
            assert!(!ordinary(&p).contains(&1), "no Anywhere donor: {value:?}");
        }
        let p = prepare(input("a\u{200d}b", normal, direction));
        assert!(!ordinary(&p).contains(&2), "ZWJ's ordinary prohibition");
    }
}

#[test]
fn anywhere_preserves_typographic_units_across_inline_style_seams() {
    for direction in DIRECTIONS {
        for (value, expected) in [
            ("a\u{301}b", vec![2]),
            ("\u{1f600}b", vec![2]),
            ("\u{1f469}\u{200d}\u{1f4bb}b", vec![5]),
            ("\u{1f1fa}\u{1f1f8}\u{1f1e8}\u{1f1e6}", vec![4]),
        ] {
            let end = value.encode_utf16().count();
            let mut request = input(value, style(t::LineBreak::Anywhere, None), direction);
            let mut nested = style(t::LineBreak::Anywhere, None);
            nested.brush = t::Brush::color(1.0, 0.0, 0.0, 1.0);
            let scalar_seam = value.chars().next().unwrap().len_utf16();
            request.source.span(t::Range::new(scalar_seam, end), nested);
            let p = prepare(request);
            assert_eq!(ordinary(&p), expected, "{value:?} {direction:?}");
        }
    }
}

#[test]
fn anywhere_respects_nowrap_disconnected_domains_and_mandatory_breaks() {
    for direction in DIRECTIONS {
        let mut nowrap = style(t::LineBreak::Anywhere, None);
        nowrap.wrap = t::Wrap::None;
        assert_eq!(
            ordinary(&prepare(input("ab", nowrap.clone(), direction))),
            []
        );
        let mut nca = input("ab", nowrap, direction);
        nca.source
            .span(t::Range::new(0, 2), style(t::LineBreak::Anywhere, None));
        assert_eq!(
            ordinary(&prepare(nca)),
            [],
            "the actual enclosing block is nowrap"
        );
        let mut disconnected = input("ab", style(t::LineBreak::Anywhere, None), direction);
        two_origins(&mut disconnected, 1, false);
        assert_eq!(ordinary(&prepare(disconnected)), []);
        for (value, forced, expected_soft) in [("a\nb", 2, vec![]), ("a\r\nb", 3, vec![])] {
            let mut request = input(value, style(t::LineBreak::Anywhere, None), direction);
            if value == "a\r\nb" {
                // Capture the authentic two-unit document delimiter rather
                // than treating CR as whitespace under the LF-only default.
                request.segment_breaks =
                    t::DocumentSegmentBreaks::Defined(vec![t::Range::new(1, 3)]);
            }
            let p = prepare(request);
            assert_eq!(cuts(&p, t::PreparedBreakKind::Forced), [forced]);
            assert_eq!(
                ordinary(&p),
                expected_soft,
                "no soft cut before hard separator or inside CRLF"
            );
        }
    }
}

#[test]
fn independent_preserved_space_cuts_survive_strict_cj_and_anywhere_adds_before_first_space() {
    for direction in DIRECTIONS {
        for mode in [
            t::LineBreak::Auto,
            t::LineBreak::Normal,
            t::LineBreak::Strict,
        ] {
            let p = prepare(input("漢 ぁ", style(mode, Some("ja")), direction));
            assert_eq!(
                ordinary(&p),
                [2],
                "SPACE permission is independent of before-CJ prohibition"
            );
        }
        let mut collapsing = style(t::LineBreak::Strict, Some("ja"));
        collapsing.white_space = t::WhiteSpace::Collapse;
        let mut request = input("漢 ぁ", collapsing.clone(), direction);
        let mut space_owner = collapsing;
        space_owner.wrap = t::Wrap::None;
        request.source.span(t::Range::new(1, 2), space_owner);
        assert_eq!(
            ordinary(&prepare(request)),
            [],
            "the disappearing SPACE's actual owner forbids its own wrapping"
        );
        for (mode, expected) in [
            (t::LineBreak::Strict, vec![2, 3]),
            (t::LineBreak::Anywhere, vec![1, 2, 3]),
        ] {
            let mut base = style(mode, None);
            base.white_space = t::WhiteSpace::BreakSpaces;
            assert_eq!(ordinary(&prepare(input("a  b", base, direction))), expected);
        }
    }
}

fn finish(
    p: &t::PreparedText,
    start: t::PreparedBoundary,
    end: t::PreparedOpportunity,
    alternative: Option<t::PreparedAlternative>,
) -> t::FinalizedLine {
    let boundary = end.view().boundary();
    let from = start.view().participant_offset();
    let to = boundary.view().participant_offset();
    let participants = p
        .participants()
        .skip(from)
        .take(to - from)
        .collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            t::PreparedParticipant::Control(occurrence) => Some(t::ControlAllocation {
                occurrence: occurrence.clone(),
                strut: None,
                boundary: None,
            }),
            _ => None,
        })
        .collect();
    p.select(t::LineSelectionRequest {
        segment: p.source_segments().next().unwrap(),
        start,
        end: boundary,
        participants,
        termination: t::PreparedTermination::Ordinary(end.clone()),
        alternative,
        incoming: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_box_struts: Vec::new(),
        line_strut: None,
        context: t::LineContext {
            alignment_insets: surgeist_text::LineAlignmentInsets::ZERO,
            formatted_line: surgeist_text::FormattedLineContext::default(),
            base_direction: if p.analyzed_source().direction() == t::Direction::RightToLeft {
                DIRECTIONS[1]
            } else {
                DIRECTIONS[0]
            },
            line_box_direction: match if p.analyzed_source().direction()
                == t::Direction::RightToLeft
            {
                DIRECTIONS[1]
            } else {
                DIRECTIONS[0]
            } {
                surgeist_text::ParagraphBaseDirection::LeftToRight => {
                    surgeist_text::LineBoxDirection::LeftToRight
                }
                surgeist_text::ParagraphBaseDirection::RightToLeft => {
                    surgeist_text::LineBoxDirection::RightToLeft
                }
            },
            orientation: t::LineOrientation::Horizontal,
            start: if from == 0 {
                t::LineStart::Paragraph
            } else {
                t::LineStart::Continuation
            },
            end: if end.view().kind() == t::PreparedBreakKind::ParagraphEnd {
                t::LineEnd::Paragraph
            } else {
                t::LineEnd::Soft
            },
            available_inline: None,
            indent: 0.0,
            alignment: t::Alignment::Start,
            tabs: t::TabContext::NoTabs,
            spacing: t::SpacingPolicy::None,
        },
    })
    .unwrap()
    .finalize(surgeist_text::LineAllocations {
        spacing: t::SpacingAllocations::None,
        scopes: surgeist_text::ScopeAllocations::None,
    })
    .unwrap()
}

fn start(p: &t::PreparedText) -> t::PreparedBoundary {
    p.boundary_views().next().unwrap().handle()
}

fn terminal(p: &t::PreparedText) -> t::PreparedOpportunity {
    p.opportunity_views()
        .find(|offer| offer.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}

fn glyphs(line: &t::FinalizedLine) -> Vec<(t::Range, u32, f32)> {
    let mut result = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        for glyph in run.glyphs() {
            result.push((glyph.range(), glyph.id(), glyph.advance()));
        }
    }
    result.sort_by_key(|row| (row.0.start, row.0.end));
    result
}

#[test]
fn anywhere_offers_have_equal_ordinary_priority_and_real_ahem_piece_measurements() {
    for direction in DIRECTIONS {
        let p = prepare(input("a,b", style(t::LineBreak::Anywhere, None), direction));
        assert_eq!(ordinary(&p), [1, 2]);
        assert_eq!(p.alternative_views().count(), 0);
        let offers = p
            .opportunity_views()
            .filter(|offer| offer.kind() == t::PreparedBreakKind::Allowed)
            .map(|offer| offer.handle())
            .collect::<Vec<_>>();
        let first = finish(&p, start(&p), offers[0].clone(), None);
        let second = finish(&p, start(&p), offers[1].clone(), None);
        assert_eq!(first.feedback().advance, 10.0);
        assert_eq!(second.feedback().advance, 20.0);
        assert_eq!(
            glyphs(&second),
            [
                (t::Range::new(0, 1), 67, 10.0),
                (t::Range::new(1, 2), 14, 10.0)
            ]
        );
        let whole = finish(&p, start(&p), terminal(&p), None);
        assert_eq!(
            whole.measurements().max_content_contribution().advance(),
            30.0
        );
        let mut from = start(&p);
        let mut minimum: f64 = 0.0;
        for end in offers.into_iter().chain(std::iter::once(terminal(&p))) {
            let line = finish(&p, from, end.clone(), None);
            minimum = minimum.max(line.measurements().min_content_contribution().advance());
            from = end.view().boundary();
        }
        // A consumer's ordinary partition has one real Ahem TCU per piece.
        // This is line-local public measurement, not a claimed paragraph fitter.
        assert_eq!(minimum, 10.0);
    }
}

#[test]
fn source_shy_hyphenation_uses_actual_owner_even_with_empty_indication() {
    for direction in DIRECTIONS {
        for hyphens in [t::Hyphens::None, t::Hyphens::Manual, t::Hyphens::Auto] {
            for indication in ["", "X"] {
                for owner in [t::LineBreak::Normal, t::LineBreak::Anywhere] {
                    for following in [t::LineBreak::Normal, t::LineBreak::Anywhere] {
                        let mut base = style(owner, Some("en-US"));
                        base.hyphens = hyphens;
                        base.hyphenate_character =
                            t::HyphenateCharacter::String(t::Text::from(indication));
                        let mut request = input("a\u{ad}b", base.clone(), direction);
                        request.source.span(t::Range::new(1, 2), base.clone());
                        let mut next = base.clone();
                        next.line_break = following;
                        request.source.span(t::Range::new(2, 3), next);
                        let p = prepare(request);
                        let cut = p
                            .hyphenation_cut_views()
                            .find(|cut| cut.provenance() == t::HyphenationProvenance::Source)
                            .unwrap();
                        assert_eq!(cut.style().line_break, owner);
                        assert_eq!(cut.boundary().view().source_after().get(), 2);
                        let alternatives = p.alternative_views().collect::<Vec<_>>();
                        let admitted =
                            hyphens != t::Hyphens::None && owner != t::LineBreak::Anywhere;
                        assert_eq!(
                            alternatives.len(),
                            usize::from(admitted),
                            "{hyphens:?} {owner:?}->{following:?} indication={indication:?} {direction:?}"
                        );
                        for alternative in alternatives {
                            assert_eq!(alternative.source_range(), Some(t::Range::new(1, 2)));
                            assert_eq!(alternative.text(), &t::Text::from(indication));
                            let line = finish(
                                &p,
                                start(&p),
                                alternative.opportunity().unwrap(),
                                Some(alternative.handle()),
                            );
                            assert_eq!(
                                line.feedback().advance,
                                if indication.is_empty() { 10.0 } else { 20.0 }
                            );
                        }
                        if owner == t::LineBreak::Anywhere || following == t::LineBreak::Anywhere {
                            let offer = p
                                .opportunity_views()
                                .find(|offer| {
                                    offer.boundary() == cut.boundary()
                                        && offer.kind() == t::PreparedBreakKind::Allowed
                                })
                                .unwrap();
                            let line = finish(&p, start(&p), offer.handle(), None);
                            assert_eq!(glyphs(&line), [(t::Range::new(0, 1), 67, 10.0)]);
                            assert!(line.selected_line().request().alternative.is_none());
                            assert!(line.selected_line().request().incoming.is_none());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn explicit_discretionary_control_cannot_donate_anywhere_or_evade_owner_exclusion() {
    for direction in DIRECTIONS {
        for hyphens in [t::Hyphens::None, t::Hyphens::Manual, t::Hyphens::Auto] {
            for text_mode in [t::LineBreak::Normal, t::LineBreak::Anywhere] {
                for owner in [t::LineBreak::Normal, t::LineBreak::Anywhere] {
                    for indication in ["", "X"] {
                        let mut control_style = style(owner, Some("en-US"));
                        control_style.hyphens = hyphens;
                        control_style.hyphenate_character =
                            t::HyphenateCharacter::String(t::Text::from(indication));
                        let mut request = input("abc", style(text_mode, Some("en-US")), direction);
                        request.controls.push(t::ParagraphControl {
                            at: 1,
                            kind: t::ParagraphControlKind::DiscretionaryBreak {
                                style: Box::new(control_style),
                            },
                        });
                        request.anchor_order.push(t::ParagraphAnchor {
                            item: t::ParagraphAnchorItem::Control(0),
                            affinity: t::Affinity::After,
                        });
                        let p = prepare(request);
                        let cut = p
                            .hyphenation_cut_views()
                            .find(|cut| cut.provenance() == t::HyphenationProvenance::Control)
                            .unwrap();
                        assert_eq!(cut.style().line_break, owner);
                        assert_eq!(
                            p.alternative_views().count(),
                            usize::from(
                                owner == t::LineBreak::Normal && hyphens != t::Hyphens::None
                            )
                        );
                        assert_eq!(cut.boundary().view().source_after().get(), 1);
                        assert_eq!(
                            ordinary(&p),
                            if text_mode == t::LineBreak::Anywhere {
                                vec![2]
                            } else {
                                vec![]
                            },
                            "the influencing control is not a text TCU or an Anywhere donor"
                        );
                        let participants = p.participants().collect::<Vec<_>>();
                        for offer in p
                            .opportunity_views()
                            .filter(|offer| offer.kind() == t::PreparedBreakKind::Allowed)
                        {
                            assert_ne!(
                                offer.boundary(),
                                cut.boundary(),
                                "numeric adjacency cannot transfer ordinary authority to the control cut"
                            );
                            let offset = offer.boundary().view().participant_offset();
                            assert!(matches!(
                                participants[offset - 1],
                                t::PreparedParticipant::Text(_)
                            ));
                            assert!(matches!(
                                participants[offset],
                                t::PreparedParticipant::Text(_)
                            ));
                        }
                        for alternative in p.alternative_views() {
                            assert_eq!(alternative.source_range(), None);
                            assert_eq!(alternative.text(), &t::Text::from(indication));
                            assert_eq!(
                                alternative.opportunity().unwrap().view().boundary(),
                                cut.boundary()
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn automatic_cut_eligibility_follows_preceding_actual_owner_and_hyphens() {
    // The independently retained en-US exception is as-so-ciate, cuts2/4.
    for direction in DIRECTIONS {
        for indication in ["", "X"] {
            for hyphens in [t::Hyphens::None, t::Hyphens::Manual, t::Hyphens::Auto] {
                for owner in [t::LineBreak::Normal, t::LineBreak::Anywhere] {
                    for following in [t::LineBreak::Normal, t::LineBreak::Anywhere] {
                        let mut base = style(t::LineBreak::Normal, Some("en-US"));
                        base.hyphens = hyphens;
                        base.hyphenate_character =
                            t::HyphenateCharacter::String(t::Text::from(indication));
                        let mut request = input("associate", base.clone(), direction);
                        let mut previous = base.clone();
                        previous.line_break = owner;
                        let mut next = base;
                        next.line_break = following;
                        request.source.span(t::Range::new(1, 2), previous);
                        request.source.span(t::Range::new(2, 3), next);
                        let p = prepare(request);
                        let actual = p
                            .alternative_views()
                            .filter(|alternative| {
                                matches!(
                                    alternative.definition(),
                                    t::PreparedAlternativeDefinition::Automatic { .. }
                                )
                            })
                            .map(|alternative| alternative.insertion_anchor().get())
                            .collect::<Vec<_>>();
                        let expected = if hyphens != t::Hyphens::Auto {
                            vec![]
                        } else if owner == t::LineBreak::Anywhere {
                            vec![4]
                        } else {
                            vec![2, 4]
                        };
                        assert_eq!(
                            actual, expected,
                            "{hyphens:?} {owner:?}->{following:?} {direction:?}"
                        );
                        for alternative in p.alternative_views() {
                            assert_eq!(alternative.text(), &t::Text::from(indication));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn standalone_line_break_changes_miss_cache_and_preserve_old_immutable_layouts() {
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let source = t::Source::identified("abc", t::Id::from_u64(65202), 7);
    let options = t::Options {
        width: Some(10.0),
        quantize: false,
        ..t::Options::default()
    };
    let normal_style = style(t::LineBreak::Normal, None);
    let mut anywhere_style = normal_style.clone();
    anywhere_style.line_break = t::LineBreak::Anywhere;
    let before = system.stats();
    let normal = system
        .layout(source.clone(), normal_style.clone(), options)
        .unwrap();
    assert_eq!(
        normal.lines().len(),
        1,
        "unbroken Latin word overflows at its sole ordinary end"
    );
    assert_eq!(normal.metrics().full_width(), 30.0);
    let normal_key = normal.key();
    let anywhere = system
        .layout(source.clone(), anywhere_style.clone(), options)
        .unwrap();
    assert_eq!(
        anywhere.lines().len(),
        3,
        "three Ahem TCUs each fit exactly one line"
    );
    assert_eq!(anywhere.metrics().full_width(), 10.0);
    assert_ne!(normal_key, anywhere.key());
    assert_eq!(system.stats().layout_misses(), before.layout_misses() + 2);
    assert_eq!(system.stats().layout_hits(), before.layout_hits());
    for (requested, key, lines) in [
        (normal_style, normal_key.clone(), 1),
        (anywhere_style, anywhere.key(), 3),
    ] {
        let cached = system.layout(source.clone(), requested, options).unwrap();
        assert_eq!(cached.key(), key);
        assert_eq!(cached.lines().len(), lines);
    }
    assert_eq!(system.stats().layout_hits(), before.layout_hits() + 2);
    assert_eq!(system.stats().layout_misses(), before.layout_misses() + 2);
    assert_eq!(normal.key(), normal_key);
    assert_eq!(normal.lines().len(), 1);
    assert_eq!(normal.metrics().full_width(), 30.0);
    for layout in [&normal, &anywhere] {
        let mut actual = Vec::new();
        for run in layout.glyph_runs() {
            assert_eq!(run.font().bytes(), AHEM);
            for glyph in run.glyphs() {
                actual.push((glyph.range(), glyph.id(), glyph.advance()));
            }
        }
        actual.sort_by_key(|row| (row.0.start, row.0.end));
        assert_eq!(
            actual,
            [
                (t::Range::new(0, 1), 67, 10.0),
                (t::Range::new(1, 2), 68, 10.0),
                (t::Range::new(2, 3), 69, 10.0)
            ]
        );
    }
    let punctuation = system
        .layout(
            t::Source::identified("a,b", t::Id::from_u64(65204), 1),
            style(t::LineBreak::Anywhere, None),
            t::Options {
                width: Some(20.0),
                quantize: false,
                ..t::Options::default()
            },
        )
        .unwrap();
    assert_eq!(
        punctuation
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [t::Range::new(0, 2), t::Range::new(2, 3)],
        "the fuller a-comma line wins over an earlier Anywhere word-interior cut without priority preference"
    );
}

#[test]
fn full_size_kana_transform_reclassifies_executed_cj_before_line_analysis() {
    for direction in DIRECTIONS {
        let base = style(t::LineBreak::Strict, Some("ja"));
        let original = prepare(input("漢ぁ", base.clone(), direction));
        assert_eq!(ordinary(&original), []);
        let mut transformed = base;
        transformed.text_transform =
            t::TextTransform::Set(t::TextTransformSet::none().with_full_size_kana(true));
        let actual = prepare(input("漢ぁ", transformed, direction));
        assert_eq!(
            actual.analyzed_source().source().text(),
            &t::Text::from("漢ぁ")
        );
        // Phase-I whitespace correspondence retains pre-transform text. The
        // changed opportunity below observes the executed typography phase.
        assert_eq!(
            actual.analyzed_source().processed_source().processed_text(),
            &t::Text::from("漢ぁ")
        );
        assert_eq!(
            ordinary(&actual),
            [1],
            "full-size kana is ID, not original-source CJ"
        );
    }
}

#[test]
fn anywhere_text_preserves_atomic_non_nbsp_glue_and_actual_ancestor_wrapping() {
    // The more specific atomic restriction follows the adopted Anywhere
    // determination; an atomic is not assigned a manufactured text style.
    for direction in DIRECTIONS {
        for (value, nbsp) in [
            ("\u{a0}", true),
            ("\u{2007}", false),
            ("\u{202f}", false),
            ("\u{2060}", false),
            ("\u{200d}", false),
        ] {
            for ancestor in [t::Wrap::Word, t::Wrap::None] {
                for atomic_before_text in [false, true] {
                    let text_style = style(t::LineBreak::Anywhere, None);
                    let mut block = text_style.clone();
                    block.wrap = ancestor;
                    let mut request = input(value, block, direction);
                    request.source.span(t::Range::new(0, 1), text_style);
                    request.source.inline_box(t::InlineBox::new(
                        t::Id::from_u64(65203),
                        t::InlineBoxKind::InFlow,
                        if atomic_before_text { 0 } else { 1 },
                        t::Size::new(10.0, 10.0),
                    ));
                    request.anchor_order.push(t::ParagraphAnchor {
                        item: t::ParagraphAnchorItem::Atomic(0),
                        affinity: t::Affinity::Before,
                    });
                    for definition in [
                        t::ParagraphBoundaryDefinition::AtomicBefore(0),
                        t::ParagraphBoundaryDefinition::AtomicAfter(0),
                    ] {
                        request.boundary_policies.push(t::ParagraphBoundaryPolicy {
                            definition,
                            preceding_wrap: t::Wrap::Word,
                            following_wrap: t::Wrap::Word,
                            ancestor_wrap: ancestor,
                        });
                    }
                    let p = prepare(request);
                    assert_eq!(p.atomic_views().count(), 1);
                    let atomic = p.atomic_views().next().unwrap().handle();
                    let participant = p.participants().position(|part| {
                        matches!(part, t::PreparedParticipant::Atomic(ref actual) if *actual == atomic)
                    }).unwrap();
                    let target_offset = participant + usize::from(atomic_before_text);
                    let target = p
                        .boundary_views()
                        .find(|boundary| boundary.participant_offset() == target_offset)
                        .unwrap()
                        .handle();
                    let admitted = p.opportunity_views().any(|offer| {
                        offer.kind() == t::PreparedBreakKind::Allowed && offer.boundary() == target
                    });
                    assert_eq!(
                        admitted,
                        nbsp && ancestor == t::Wrap::Word,
                        "actual atomic relation: {value:?}, before={atomic_before_text}, ancestor={ancestor:?}, {direction:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn loose_iteration_tailoring_keeps_contextual_quote_protection_without_blanket_veto() {
    // UAX14 revision55 LB15a protects initial Pi-QU SP* at paragraph start;
    // LB19/19a permit the Pf-QU edge surrounded by East Asian characters.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB15a
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB19a
    for direction in DIRECTIONS {
        let protected = prepare(input(
            "\u{201c}  々",
            style(t::LineBreak::Loose, Some("ja")),
            direction,
        ));
        assert!(
            !ordinary(&protected).contains(&3),
            "Pi QU at SOT retains SP* attachment before iteration mark"
        );
        let east_asian = prepare(input(
            "漢\u{201d}々",
            style(t::LineBreak::Loose, Some("ja")),
            direction,
        ));
        assert!(
            ordinary(&east_asian).contains(&2),
            "Pf QU between East Asian text does not categorically veto Loose iteration permission"
        );
    }
}

#[test]
fn keep_all_retains_line_break_required_cjk_tailoring_without_general_word_breaks() {
    // Pinned Text4 section6.1 explicitly preserves opportunities required by
    // line-break even when word-break would otherwise suppress the text pair.
    for direction in DIRECTIONS {
        for (value, mode, expected) in [
            ("漢ぁ", t::LineBreak::Loose, vec![1]),
            ("漢々", t::LineBreak::Loose, vec![1]),
            ("……", t::LineBreak::Loose, vec![1]),
            ("漢〜", t::LineBreak::Normal, vec![1]),
            ("漢漢", t::LineBreak::Loose, vec![]),
            ("漢漢", t::LineBreak::Normal, vec![]),
        ] {
            let mut base = style(mode, Some("ja"));
            base.word_break = t::WordBreak::KeepAll;
            let p = prepare(input(value, base, direction));
            assert_eq!(
                ordinary(&p),
                expected,
                "KeepAll {mode:?} {value} {direction:?}"
            );
        }
    }
}

#[test]
fn processed_small_kana_word_break_precedence_preserves_named_punctuation_rules() {
    // Independent matrix: the committed small-kana precedence determination
    // applies BreakAll's effective ID before the class-based CJ restriction.
    // Named iteration/hyphen rules retain their codepoint-specific precedence.
    // references/text-small-kana-precedence--determination-20261025.md
    let strictnesses = [
        t::LineBreak::Auto,
        t::LineBreak::Loose,
        t::LineBreak::Normal,
        t::LineBreak::Strict,
    ];
    let words = [
        t::WordBreak::Normal,
        t::WordBreak::BreakAll,
        t::WordBreak::KeepAll,
    ];
    for direction in DIRECTIONS {
        for mode in strictnesses {
            for word in words {
                for small in ['ぁ', '\u{1b132}'] {
                    for combining in [false, true] {
                        let value = format!("漢{small}{}", if combining { "\u{301}" } else { "" });
                        for transform in [false, true] {
                            let mut base = style(mode, Some("ja"));
                            base.word_break = word;
                            base.text_transform = t::TextTransform::Set(
                                t::TextTransformSet::none().with_full_size_kana(transform),
                            );
                            let p = prepare(input(&value, base, direction));
                            let admitted = if transform {
                                word != t::WordBreak::KeepAll
                            } else {
                                word == t::WordBreak::BreakAll || mode == t::LineBreak::Loose
                            };
                            assert_eq!(
                                ordinary(&p),
                                if admitted { vec![1] } else { vec![] },
                                "processed CJ/ID precedence: {value:?} {word:?} {mode:?} transform={transform} {direction:?}"
                            );
                            assert_eq!(
                                p.analyzed_source().source().text(),
                                &t::Text::from(value.as_str())
                            );
                            assert_eq!(
                                p.analyzed_source().processed_source().processed_text(),
                                &t::Text::from(value.as_str())
                            );
                            let original_small = t::Range::new(1, 1 + small.len_utf16());
                            assert!(
                                p.unit_views()
                                    .any(|unit| unit.source_range() == original_small),
                                "transformed kana retains its original UTF-16 range {original_small:?}"
                            );
                            assert_eq!(
                                p.analyzed_source().graphemes().collect::<Vec<_>>(),
                                [
                                    t::Range::new(0, 1),
                                    t::Range::new(1, value.encode_utf16().count())
                                ],
                                "the combining sequence stays whole, including supplementary small kana"
                            );
                        }
                    }
                }
                for mark in ['々', '〻', 'ゝ', 'ゞ', 'ヽ', 'ヾ'] {
                    let mut base = style(mode, Some("ja"));
                    base.word_break = word;
                    let p = prepare(input(&format!("漢{mark}"), base, direction));
                    assert_eq!(
                        ordinary(&p),
                        if mode == t::LineBreak::Loose {
                            vec![1]
                        } else {
                            vec![]
                        },
                        "named iteration rule: {mark} {word:?} {mode:?} {direction:?}"
                    );
                }
                for mark in ['〜', '゠'] {
                    for (locale, cjk) in [("ja", true), ("ja-Latn", false)] {
                        let mut base = style(mode, Some(locale));
                        base.word_break = word;
                        let p = prepare(input(&format!("漢{mark}"), base, direction));
                        let admitted =
                            cjk && matches!(mode, t::LineBreak::Loose | t::LineBreak::Normal);
                        assert_eq!(
                            ordinary(&p),
                            if admitted { vec![1] } else { vec![] },
                            "named CJK hyphen: {mark} {word:?} {mode:?} {locale} {direction:?}"
                        );
                    }
                }
                for mark in ['\u{2010}', '\u{2013}'] {
                    for prefix in ['漢', 'A'] {
                        let mut base = style(mode, Some("ja"));
                        base.word_break = word;
                        let p = prepare(input(&format!("{prefix}{mark}"), base, direction));
                        let admitted = mode == t::LineBreak::Loose
                            && (prefix == '漢' || word == t::WordBreak::BreakAll);
                        assert_eq!(
                            ordinary(&p),
                            if admitted { vec![1] } else { vec![] },
                            "effective ID before named hyphen: {prefix}{mark} {word:?} {mode:?} {direction:?}"
                        );
                    }
                }
            }
        }
    }
}
