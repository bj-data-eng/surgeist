//! Computed native minima: actual word counts and actual governing cut styles.
use super::*;
use crate::{
    HyphenateCharacter, HyphenateLimitChars, HyphenationCounts, HyphenationError,
    HyphenationMinimum::{Auto, Count},
    HyphenationPlacement, HyphenationProvenance, Hyphens, NativeHyphenationConvention,
    ParagraphAnchor, ParagraphAnchorItem, ParagraphControl, ParagraphControlKind, ParagraphOrigin,
    ParagraphOriginKind, ParagraphTerminalEdge, Range, Text,
};

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");
fn limits(total: usize, before: usize, after: usize) -> HyphenateLimitChars {
    HyphenateLimitChars {
        total: Count(total),
        before: Count(before),
        after: Count(after),
    }
}
fn own(limits: HyphenateLimitChars) -> Style {
    Style {
        hyphenate_limit_chars: limits,
        hyphenate_character: HyphenateCharacter::String(Text::from("A")),
        ..style()
    }
}
fn input(source: Source, base_style: Style) -> ParagraphInput {
    let end = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, end),
            aggregate: Range::new(0, end),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn capture(system: &mut System, input: ParagraphInput) -> crate::Result<PreparedText> {
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)?,
        crate::Wrap::Word,
    );
    system.prepare(&analyzed, PreparationOptions::default())
}
fn prepare(source: &str, limits: HyphenateLimitChars) -> PreparedText {
    capture(&mut system(), input(Source::new(source), own(limits))).unwrap()
}
fn cuts(p: &PreparedText) -> Vec<usize> {
    p.hyphenation_cut_views()
        .filter(|cut| cut.opportunity().is_some())
        .map(|cut| cut.boundary().view().source_before().get())
        .collect()
}
fn whole(p: &PreparedText) -> FinalizedLine {
    let mut value = request(p);
    value.controls = p
        .control_views()
        .map(|row| {
            let occurrence = row.handle();
            ControlAllocation {
                boundary: matches!(
                    occurrence.view().kind(),
                    crate::ParagraphControlKind::InlineBoundaryStart
                        | crate::ParagraphControlKind::InlineBoundaryEnd
                )
                .then_some(crate::InlineBoundaryAllocation::ZERO),
                occurrence,
                strut: None,
            }
        })
        .collect();
    p.select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn selected(p: &PreparedText, offer: PreparedOpportunity) -> FinalizedLine {
    let mut value = request(p);
    value.end = offer.view().boundary();
    value.termination = PreparedTermination::Ordinary(offer.clone());
    value.alternative = Some(offer.view().alternatives().next().unwrap().handle());
    value.context.end = LineEnd::Soft;
    value.participants = p
        .participants()
        .take(value.end.view().participant_offset())
        .collect();
    value.controls = p
        .control_views()
        .filter(|row| {
            value
                .participants
                .contains(&PreparedParticipant::Control(row.handle()))
        })
        .map(|row| {
            let occurrence = row.handle();
            ControlAllocation {
                boundary: matches!(
                    occurrence.view().kind(),
                    crate::ParagraphControlKind::InlineBoundaryStart
                        | crate::ParagraphControlKind::InlineBoundaryEnd
                )
                .then_some(crate::InlineBoundaryAllocation::ZERO),
                occurrence,
                strut: None,
            }
        })
        .collect();
    p.select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn error_reason(error: &crate::Error) -> HyphenationError {
    match error.detail() {
        Some(crate::ErrorDetail::Hyphenation { reason }) => *reason,
        other => panic!("unexpected {other:?}"),
    }
}
fn control(value: &mut ParagraphInput, at: usize, style: Style) {
    value.controls.push(ParagraphControl {
        at,
        kind: ParagraphControlKind::DiscretionaryBreak {
            style: Box::new(style),
        },
    });
    value.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::After,
    });
}
#[test]
fn explicit_total_four_admits_the_real_short_shy_while_default_keeps_unbroken_source() {
    for (minimum, admitted) in [
        (HyphenateLimitChars::default(), false),
        (limits(4, 2, 2), true),
    ] {
        let p = prepare("ab\u{ad}cd", minimum);
        let line = whole(&p);
        assert_eq!(line.feedback().advance, 80.0);
        let mut glyphs = Vec::new();
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.effective_instance().unwrap().font_size(), 20.0);
            glyphs.extend(
                run.glyphs()
                    .filter(|glyph| glyph.advance() != 0.0)
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
            );
        }
        assert_eq!(
            glyphs,
            [
                (Range::new(0, 1), 67, 20.0),
                (Range::new(1, 2), 68, 20.0),
                (Range::new(3, 4), 69, 20.0),
                (Range::new(4, 5), 70, 20.0)
            ]
        );
        let cut = p.hyphenation_cut_views().next().unwrap();
        assert_eq!(
            cut.counts(),
            Some(HyphenationCounts {
                total: 4,
                before: 2,
                after: 2
            })
        );
        assert_eq!(cut.style().hyphenate_limit_chars, minimum);
        assert_eq!(cut.provenance(), HyphenationProvenance::Source);
        assert_eq!(cut.opportunity().is_some(), admitted);
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [97, 98, 173, 99, 100]
        );
        if let Some(offer) = cut.opportunity() {
            let first = selected(&p, offer);
            assert_eq!(first.feedback().advance, 60.0);
            assert!(first.run_views().any(|run| {
                run.glyphs().any(|glyph| {
                    glyph.range() == Range::new(3, 3) && glyph.id() == 35 && glyph.advance() == 20.0
                })
            }));
        }
    }
}
#[test]
fn each_minimum_is_independent_and_every_auto_component_resolves_its_own_fallback() {
    for (minimum, admitted) in [
        (limits(7, 3, 4), true),
        (limits(8, 0, 0), false),
        (limits(0, 4, 0), false),
        (limits(0, 0, 5), false),
        (
            HyphenateLimitChars {
                total: Auto,
                before: Count(3),
                after: Auto,
            },
            true,
        ),
    ] {
        let p = prepare("abc\u{ad}defg", minimum);
        assert_eq!(cuts(&p).is_empty(), !admitted, "{minimum:?}");
        assert_eq!(
            p.hyphenation_cut_views().next().unwrap().counts(),
            Some(HyphenationCounts {
                total: 7,
                before: 3,
                after: 4
            })
        );
    }
    for (source, minimum, admitted) in [
        (
            "ab\u{ad}cd",
            HyphenateLimitChars {
                total: Count(4),
                before: Auto,
                after: Auto,
            },
            true,
        ),
        (
            "ab\u{ad}cd",
            HyphenateLimitChars {
                total: Auto,
                before: Count(0),
                after: Count(0),
            },
            false,
        ),
        (
            "a\u{ad}bcde",
            HyphenateLimitChars {
                total: Count(0),
                before: Auto,
                after: Count(0),
            },
            false,
        ),
        (
            "abcd\u{ad}e",
            HyphenateLimitChars {
                total: Count(0),
                before: Count(0),
                after: Auto,
            },
            false,
        ),
        ("a\u{ad}b", limits(0, 0, 0), true),
    ] {
        assert_eq!(
            cuts(&prepare(source, minimum)).is_empty(),
            !admitted,
            "{source}/{minimum:?}"
        );
    }
}
#[test]
fn zero_and_maximum_counts_are_valid_values_without_creating_words_or_allocating_by_threshold() {
    for minimum in [
        limits(usize::MAX, 0, 0),
        limits(0, usize::MAX, 0),
        limits(0, 0, usize::MAX),
        limits(1, usize::MAX, usize::MAX),
    ] {
        let authored = own(minimum);
        let validated = crate::ValidatedStyle::try_from(authored.clone()).unwrap();
        assert_eq!(validated.authored().hyphenate_limit_chars, minimum);
        let p = prepare("ab\u{ad}cde", minimum);
        assert!(cuts(&p).is_empty());
        assert_eq!(whole(&p).feedback().advance, 100.0);
    }
    let mut value = input(Source::new("$"), own(limits(0, 0, 0)));
    control(&mut value, 0, own(limits(0, 0, 0)));
    let p = capture(&mut system(), value).unwrap();
    let row = p.hyphenation_cut_views().next().unwrap();
    assert_eq!(row.provenance(), HyphenationProvenance::Control);
    assert_eq!(row.word(), None);
    assert_eq!(row.counts(), None);
    assert!(row.opportunity().is_none());
    assert!(p.alternative_views().next().is_none());
}
#[test]
fn governing_shy_and_control_styles_override_surrounding_minima_without_a_preparation_wide_override()
 {
    let rejected = own(limits(usize::MAX, usize::MAX, usize::MAX));
    let allowed = own(limits(4, 2, 2));
    for (base, defining, admitted) in [
        (rejected.clone(), allowed.clone(), true),
        (allowed.clone(), rejected.clone(), false),
    ] {
        let mut source = Source::new("ab\u{ad}cd");
        source.span(Range::new(2, 3), defining.clone());
        let p = capture(&mut system(), input(source, base)).unwrap();
        let cut = p.hyphenation_cut_views().next().unwrap();
        assert_eq!(
            cut.style().hyphenate_limit_chars,
            defining.hyphenate_limit_chars
        );
        assert_eq!(cut.opportunity().is_some(), admitted);
    }
    for (base, defining, admitted) in [
        (rejected.clone(), allowed.clone(), true),
        (allowed, rejected, false),
    ] {
        let mut defining = defining;
        defining.size = 10.0;
        let mut value = input(Source::new("abcd"), base);
        control(&mut value, 2, defining.clone());
        let p = capture(&mut system(), value).unwrap();
        let cut = p.hyphenation_cut_views().next().unwrap();
        assert_eq!(cut.provenance(), HyphenationProvenance::Control);
        assert_eq!(
            cut.counts(),
            Some(HyphenationCounts {
                total: 4,
                before: 2,
                after: 2
            })
        );
        assert_eq!(
            cut.style().hyphenate_limit_chars,
            defining.hyphenate_limit_chars
        );
        assert_eq!(cut.opportunity().is_some(), admitted);
        if let Some(offer) = cut.opportunity() {
            assert_eq!(selected(&p, offer).feedback().advance, 50.0);
        }
    }
}
#[test]
fn count_rejection_precedes_unsupported_separator_and_missing_primary_resolution() {
    let mut system = system();
    let convention = NativeHyphenationConvention::language_script(
        "en-Latn",
        "Latn",
        Text::from("A\u{2028}"),
        HyphenationPlacement::BothSides,
    )
    .unwrap();
    for (minimum, admitted) in [
        (HyphenateLimitChars::default(), false),
        (limits(4, 2, 2), true),
    ] {
        let captured = Style {
            locale: Some("en-Latn".into()),
            native_hyphenation_convention: Some(convention.clone()),
            hyphenate_character: HyphenateCharacter::Auto,
            ..own(minimum)
        };
        let result = capture(&mut system, input(Source::new("ab\u{ad}cd"), captured));
        if admitted {
            assert_eq!(
                error_reason(&result.unwrap_err()),
                HyphenationError::UnsupportedIndicationSeparator
            );
        } else {
            assert!(cuts(&result.unwrap()).is_empty());
        }
        let mut value = input(Source::new("abcd"), own_default());
        control(
            &mut value,
            2,
            Style {
                font: Font::new().family("Missing owning primary"),
                hyphenate_character: HyphenateCharacter::Auto,
                ..own(minimum)
            },
        );
        let result = capture(&mut system, value);
        if admitted {
            assert_eq!(
                error_reason(&result.unwrap_err()),
                HyphenationError::MissingPrimaryFont
            );
        } else {
            assert!(cuts(&result.unwrap()).is_empty());
        }
    }
}
fn own_default() -> Style {
    own(HyphenateLimitChars::default())
}
#[test]
fn empty_indication_does_not_bypass_native_count_admission() {
    for (minimum, admitted) in [
        (HyphenateLimitChars::default(), false),
        (limits(4, 2, 2), true),
    ] {
        let value = input(
            Source::new("ab\u{ad}cd"),
            Style {
                hyphenate_character: HyphenateCharacter::String(Text::from("")),
                ..own(minimum)
            },
        );
        let p = capture(&mut system(), value).unwrap();
        let cut = p.hyphenation_cut_views().next().unwrap();
        assert_eq!(cut.opportunity().is_some(), admitted);
        if let Some(offer) = cut.opportunity() {
            assert!(
                offer
                    .view()
                    .alternatives()
                    .next()
                    .unwrap()
                    .text()
                    .is_empty()
            );
            assert_eq!(selected(&p, offer).feedback().advance, 40.0);
        }
    }
}
#[test]
fn automatic_points_use_their_own_preceding_styles_and_keep_rejected_shy_identity() {
    let auto = Style {
        hyphens: Hyphens::Auto,
        locale: Some("en-US".into()),
        size: 10.0,
        ..own_default()
    };
    for (minimum, expected) in [
        (HyphenateLimitChars::default(), vec![2, 4]),
        (limits(0, 4, 0), vec![4]),
    ] {
        let p = capture(
            &mut system(),
            input(
                Source::new("associate"),
                Style {
                    hyphenate_limit_chars: minimum,
                    ..auto.clone()
                },
            ),
        )
        .unwrap();
        assert_eq!(cuts(&p), expected);
        assert!(
            p.hyphenation_cut_views()
                .all(|cut| cut.style().hyphenate_limit_chars == minimum)
        );
    }
    let mut source = Source::new("associate");
    source.span(
        Range::new(1, 2),
        Style {
            hyphenate_limit_chars: limits(0, 4, 0),
            ..auto.clone()
        },
    );
    let p = capture(&mut system(), input(source, auto.clone())).unwrap();
    assert_eq!(cuts(&p), [4]);
    for mixed in [false, true] {
        let restricted = Style {
            hyphenate_limit_chars: limits(0, 4, 0),
            ..auto.clone()
        };
        let mut source = Source::new("as\u{ad}sociate");
        let base = if mixed {
            source.span(Range::new(2, 3), restricted);
            auto.clone()
        } else {
            restricted
        };
        let raw_source = source.clone();
        let p = capture(&mut system(), input(source, base.clone())).unwrap();
        assert_eq!(cuts(&p), [5]);
        let manual = p
            .hyphenation_cut_views()
            .find(|cut| cut.provenance() == HyphenationProvenance::Source)
            .unwrap();
        assert!(manual.opportunity().is_none());
        assert!(
            p.hyphenation_cut_views()
                .all(|cut| cut.provenance() == HyphenationProvenance::Source
                    || cut.boundary().view().source_before().get() != 3)
        );
        let portions = p.word_views().next().unwrap().portions().unwrap();
        assert_eq!(portions.len(), 1);
        assert!(!portions[0].has_manual_partition());
        let part = &portions[0];
        let mut value = request(&p);
        value.start = part.start();
        value.end = part.end();
        value.termination = PreparedTermination::Measurement(part.clone());
        value.context.end = LineEnd::Soft;
        let measured = p
            .select(value)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(measured.feedback().advance, 90.0);
        let mut system = system();
        let raw = system
            .layout(
                raw_source,
                base,
                crate::Options {
                    width: Some(80.0),
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(raw.lines()[0].range(), Range::new(0, 3));
        let advance: f64 = raw
            .inline_metric_facts()
            .runs()
            .iter()
            .filter(|row| row.line() == 0)
            .map(|row| row.advance())
            .sum();
        assert_eq!(advance, 30.0);
    }
}
#[test]
fn native_direct_analysis_consumes_custom_limits_while_raw_remains_unconstrained() {
    let mut system = system();
    let source = Source::new("ab\u{ad}cd");
    for (minimum, admitted) in [
        (HyphenateLimitChars::default(), false),
        (limits(4, 2, 2), true),
        (limits(usize::MAX, usize::MAX, usize::MAX), false),
    ] {
        let style = own(minimum);
        let analyzed = AnalyzedSource::new(
            source.clone(),
            style.clone(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let native = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        assert_eq!(cuts(&native).is_empty(), !admitted);
        let raw = system
            .layout_analyzed(
                &analyzed,
                crate::Options {
                    width: Some(60.0),
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            raw.lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 3), Range::new(3, 5)]
        );
        let mut advance = vec![0.0; 2];
        for row in raw.inline_metric_facts().runs() {
            advance[row.line()] += row.advance();
        }
        assert_eq!(advance, [60.0, 40.0]);
    }
}
#[test]
fn symbolic_minima_participate_in_base_and_span_keys_and_foreign_preparations_do_not_alias() {
    let mut system = system();
    let source = Source::identified("ab\u{ad}cd", crate::Id::from_u64(6604), 1);
    let options = crate::ValidatedOptions::try_from(crate::Options::default()).unwrap();
    let validated = crate::ValidatedSource::try_from(source.clone()).unwrap();
    let mut keys = std::collections::HashSet::new();
    for minimum in [
        HyphenateLimitChars::default(),
        limits(5, 2, 2),
        limits(6, 2, 2),
        limits(5, 3, 2),
        limits(5, 2, 3),
    ] {
        let style = crate::ValidatedStyle::try_from(own(minimum)).unwrap();
        assert_eq!(style.authored().hyphenate_limit_chars, minimum);
        keys.insert(
            crate::Key::from_validated(
                &validated,
                &style,
                options,
                system.font_catalogue_identity(),
            )
            .styles(),
        );
    }
    assert_eq!(keys.len(), 5);
    let default = crate::ValidatedStyle::try_from(own_default()).unwrap();
    let plain = crate::Key::from_validated(
        &validated,
        &default,
        options,
        system.font_catalogue_identity(),
    );
    let mut changed = source.clone();
    changed.span(Range::new(2, 3), own(limits(5, 2, 2)));
    let changed = crate::ValidatedSource::try_from(changed).unwrap();
    assert_ne!(
        plain.styles(),
        crate::Key::from_validated(
            &changed,
            &default,
            options,
            system.font_catalogue_identity()
        )
        .styles()
    );
    let old = capture(&mut system, input(source.clone(), own(limits(4, 2, 2)))).unwrap();
    let offer = old
        .hyphenation_cut_views()
        .next()
        .unwrap()
        .opportunity()
        .unwrap();
    let retained = selected(&old, offer.clone());
    let new = capture(&mut system, input(source, own(limits(usize::MAX, 0, 0)))).unwrap();
    assert!(cuts(&new).is_empty());
    assert_eq!(retained.feedback().advance, 60.0);
    let mut foreign = request(&new);
    foreign.end = offer.view().boundary();
    foreign.termination = PreparedTermination::Ordinary(offer);
    foreign.context.end = LineEnd::Soft;
    assert!(matches!(
        new.select(foreign).unwrap_err().detail(),
        Some(crate::ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::ForeignPreparation { .. }
        })
    ));
}
