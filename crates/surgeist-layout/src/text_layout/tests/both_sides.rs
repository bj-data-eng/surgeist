//! Explicit native convention facts supply one exact A, once on each side.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

#[derive(Clone, Copy)]
enum Definition {
    Source,
    Control,
}

fn style(placement: text::HyphenationPlacement) -> text::Style {
    text::Style {
        hyphens: text::Hyphens::Manual,
        white_space: text::WhiteSpace::Collapse,
        locale: Some("und-Latn".into()),
        hyphenate_character: text::HyphenateCharacter::Auto,
        native_hyphenation_convention: Some(
            text::NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                "A".into(),
                placement,
            )
            .unwrap(),
        ),
        ..ahem_style(10.0, 10.0)
    }
}

fn original(definition: Definition, consecutive: bool) -> &'static str {
    match (definition, consecutive) {
        (Definition::Source, false) => "AB\u{ad}CDE",
        (Definition::Source, true) => "AB\u{ad}CD\u{ad}EF",
        (Definition::Control, false) => "ABCDE",
        (Definition::Control, true) => "ABCDEF",
    }
}

fn prepare(
    definition: Definition,
    consecutive: bool,
    placement: text::HyphenationPlacement,
) -> text::Result<text::PreparedText> {
    let style = style(placement);
    let parts: &[&str] = match (definition, consecutive) {
        (Definition::Source, false) => &["AB\u{ad}", "CDE"],
        (Definition::Source, true) => &["AB\u{ad}", "CD\u{ad}", "EF"],
        (Definition::Control, false) => &["AB", "CDE"],
        (Definition::Control, true) => &["AB", "CD", "EF"],
    };
    let mut origins = Vec::new();
    let mut controls = Vec::new();
    let mut connections = Vec::new();
    let mut boundary_policies = Vec::new();
    let mut anchors = Vec::new();
    let mut offset = 0;
    for (index, part) in parts.iter().enumerate() {
        if index != 0 {
            connections.push(text::ParagraphConnection {
                at: offset,
                kind: text::ParagraphConnectionKind::Continuous,
            });
            boundary_policies.push(text::ParagraphBoundaryPolicy {
                definition: text::ParagraphBoundaryDefinition::Connection(index - 1),
                preceding_wrap: text::Wrap::Word,
                following_wrap: text::Wrap::Word,
                ancestor_wrap: text::Wrap::Word,
            });
            if matches!(definition, Definition::Control) {
                anchors.push(text::ParagraphAnchor {
                    item: text::ParagraphAnchorItem::Control(controls.len()),
                    affinity: text::Affinity::Before,
                });
                controls.push(text::ParagraphControl {
                    at: offset,
                    kind: text::ParagraphControlKind::DiscretionaryBreak {
                        style: Box::new(style.clone()),
                    },
                });
            }
        }
        let source = Source::identified(*part, text::Id::from_u64(66100 + index as u64), 7);
        let len = source.text().len();
        origins.push(text::ParagraphOrigin {
            text_node: None,
            source,
            local: text::Range::new(0, len),
            aggregate: text::Range::new(offset, offset + len),
            kind: text::ParagraphOriginKind::Authored,
        });
        offset += len;
    }
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        source: Source::identified(
            original(definition, consecutive),
            text::Id::from_u64(66199),
            7,
        ),
        base_style: style,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins,
        connections,
        controls,
        boundary_policies,
        replacements: Vec::new(),
        anchor_order: anchors,
    })?;
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })?;
    system.register_fonts(AHEM.to_vec())?;
    system.prepare(&analyzed, text::PreparationOptions::default())
}

fn qualify(p: &text::PreparedText, definition: Definition, consecutive: bool) {
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        original(definition, consecutive)
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    let expected = if consecutive {
        vec![
            text::HyphenationCounts {
                total: 6,
                before: 2,
                after: 4,
            },
            text::HyphenationCounts {
                total: 6,
                before: 4,
                after: 2,
            },
        ]
    } else {
        vec![text::HyphenationCounts {
            total: 5,
            before: 2,
            after: 3,
        }]
    };
    assert_eq!(
        p.hyphenation_cut_views()
            .map(|cut| {
                assert_eq!(
                    cut.provenance(),
                    match definition {
                        Definition::Source => text::HyphenationProvenance::Source,
                        Definition::Control => text::HyphenationProvenance::Control,
                    }
                );
                assert_eq!(
                    cut.opportunity().unwrap().view().kind(),
                    text::PreparedBreakKind::Discretionary
                );
                cut.counts().unwrap()
            })
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        p.alternative_views().count(),
        if consecutive { 2 } else { 1 }
    );
    let step = if matches!(definition, Definition::Source) {
        3
    } else {
        2
    };
    for (index, alternative) in p.alternative_views().enumerate() {
        assert_eq!(alternative.text().code_units(), [65]);
        assert_eq!(alternative.insertion_anchor().get(), (index + 1) * step);
        match (definition, alternative.definition()) {
            (
                Definition::Source,
                text::PreparedAlternativeDefinition::Source { origin, source },
            ) => {
                assert_eq!(source.end - source.start, 1);
                assert_eq!(
                    p.analyzed_source().source().text().code_units()[source.start],
                    173
                );
                assert_eq!(origin.view().source().revision(), 7);
            }
            (Definition::Control, text::PreparedAlternativeDefinition::Control { control, .. }) => {
                assert!(matches!(
                    control.view().kind(),
                    text::ParagraphControlKind::DiscretionaryBreak { .. }
                ));
                assert!(alternative.source_range().is_none());
            }
            _ => panic!("the alternative retains its actual defining occurrence"),
        }
    }
}

fn compute<S: LayoutScalar>(
    p: &text::PreparedText,
    available: AvailableOf<S>,
    width: Option<f64>,
) -> CompletedLayoutBatchOf<u32, S> {
    compute_with_style::<S>(p, available, width, crate::TextWrapStyle::Auto)
}

fn compute_with_style<S: LayoutScalar>(
    p: &text::PreparedText,
    available: AvailableOf<S>,
    width: Option<f64>,
    wrap_style: crate::TextWrapStyle,
) -> CompletedLayoutBatchOf<u32, S> {
    let origins = p
        .origin_views()
        .enumerate()
        .map(|(index, row)| TextOriginBinding {
            origin: row.handle(),
            node: 10 + index as u32,
        })
        .collect::<Vec<_>>();
    let controls = p
        .control_views()
        .enumerate()
        .map(|(index, row)| TextControlBinding {
            occurrence: row.handle(),
            node: 20 + index as u32,
        })
        .collect::<Vec<_>>();
    let mut nodes = Vec::new();
    for (index, origin) in origins.iter().enumerate() {
        if index != 0
            && let Some(control) = controls.get(index - 1)
        {
            nodes.push(control.node);
        }
        nodes.push(origin.node);
    }
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, nodes.clone())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                text_wrap_style: wrap_style,
                size: Size::new(
                    width.map_or(PreferredSizeOf::AUTO, |w| {
                        PreferredSizeOf::px(S::from_f64(w))
                    }),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        );
    for row in &origins {
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
        );
    }
    for row in &controls {
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::text_control(TextControlParticipation::new(row.occurrence.clone())),
        );
    }
    let binding = TextParagraphBinding {
        prepared: p.clone(),
        container: 0,
        nodes,
        origins,
        atomics: Vec::new(),
        controls,
        line_strut: None,
    };
    let captured = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let input =
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap();
    compute_layout_with_flow(&captured, 0, input).unwrap()
}

fn lines<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut rows = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        if let Some(previous) = &rows[fragment.line_index()] {
            assert_eq!(previous, &line);
        }
        rows[fragment.line_index()] = Some(line);
    }
    rows.into_iter().map(Option::unwrap).collect()
}

fn assert_lines<S: LayoutScalar>(
    p: &text::PreparedText,
    definition: Definition,
    consecutive: bool,
    placement: text::HyphenationPlacement,
) {
    assert_lines_with_style::<S>(
        p,
        definition,
        consecutive,
        placement,
        crate::TextWrapStyle::Auto,
    );
}

fn assert_lines_with_style<S: LayoutScalar>(
    p: &text::PreparedText,
    definition: Definition,
    consecutive: bool,
    placement: text::HyphenationPlacement,
    wrap_style: crate::TextWrapStyle,
) {
    let source = original(definition, consecutive)
        .encode_utf16()
        .collect::<Vec<_>>();
    let batch = compute_with_style::<S>(
        p,
        AvailableOf::definite(S::from_f64(40.0)),
        Some(40.0),
        wrap_style,
    );
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        source
    );
    let cut = if matches!(definition, Definition::Source) {
        3
    } else {
        2
    };
    let before = placement == text::HyphenationPlacement::LogicalBefore;
    let after = placement == text::HyphenationPlacement::LogicalAfter;
    let ends = if consecutive && after {
        vec![2 * cut, source.len()]
    } else if consecutive && !before {
        vec![cut, 2 * cut, source.len()]
    } else {
        vec![cut, source.len()]
    };
    let advances: &[f64] = match (consecutive, placement) {
        (false, text::HyphenationPlacement::LogicalBefore) => &[30.0, 30.0],
        (false, text::HyphenationPlacement::LogicalAfter) => &[20.0, 40.0],
        (false, text::HyphenationPlacement::BothSides) => &[30.0, 40.0],
        (true, text::HyphenationPlacement::LogicalBefore) => &[30.0, 40.0],
        (true, text::HyphenationPlacement::LogicalAfter) => &[40.0, 30.0],
        (true, text::HyphenationPlacement::BothSides) => &[30.0, 40.0, 30.0],
    };
    let actual = lines(&batch);
    assert_eq!(actual.len(), advances.len());
    let mut start = 0;
    let mut incoming = None;
    for (ordinal, (line, &end)) in actual.iter().zip(&ends).enumerate() {
        let selected = line.selected_line();
        let request = selected.request();
        let outgoing = ordinal + 1 != actual.len();
        assert_eq!(request.incoming, incoming);
        assert_eq!(request.alternative.is_some(), outgoing);
        if outgoing {
            assert_eq!(
                request.alternative,
                p.alternative_views()
                    .nth(end / cut - 1)
                    .map(|row| row.handle())
            );
        }
        assert_eq!(
            batch.placed_text()[0].layout().lines()[ordinal].range(),
            text::Range::new(start, end)
        );
        assert_eq!(
            batch.placed_text()[0].layout().lines()[ordinal].baseline(),
            8.0 + ordinal as f64 * 10.0
        );
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().occupied_start,
                line.feedback().occupied_end,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (advances[ordinal], 0.0, advances[ordinal], 8.0, 10.0)
        );

        let mut expected = Vec::new();
        if ordinal != 0 && !before {
            expected.push((text::Range::new(start, start), 35));
        }
        for (offset, &cp) in source.iter().enumerate().take(end).skip(start) {
            if cp != 173 {
                assert!((65..=70).contains(&cp));
                expected.push((
                    text::Range::new(offset, offset + 1),
                    35 + u32::from(cp - 65),
                ));
            }
        }
        if outgoing && !after {
            expected.push((text::Range::new(end, end), 35));
        }
        let expected = expected
            .into_iter()
            .enumerate()
            .map(|(index, (range, glyph))| (range, glyph, index as f64 * 10.0))
            .collect::<Vec<_>>();
        let mut glyphs = Vec::new();
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().catalogue_identity(), p.font_catalogue_identity());
            let instance = run.effective_instance().unwrap();
            assert!(instance.face().is_same_face(&run.face()));
            assert_eq!(instance.font_size(), 10.0);
            for glyph in run.glyphs() {
                if glyph.advance() == 0.0 {
                    let range = glyph.range();
                    match definition {
                        Definition::Source => assert_eq!(&source[range.start..range.end], &[173]),
                        Definition::Control => assert!(
                            range.is_empty()
                                && p.control_views()
                                    .any(|c| c.source_position().get() == range.start)
                        ),
                    }
                    continue;
                }
                assert_eq!(glyph.advance(), 10.0);
                assert_eq!(glyph.y(), 0.0);
                glyphs.push((
                    glyph.range(),
                    glyph.id(),
                    run.origin().x + glyph.pen() + f64::from(glyph.x()),
                ));
            }
        }
        glyphs.sort_by(|a, b| a.2.total_cmp(&b.2));
        assert_eq!(glyphs, expected);
        let mut owners = Vec::new();
        if let Some(previous) = &incoming {
            owners.push(previous.alternative());
        }
        if !after && let Some(next) = &request.alternative {
            owners.push(next.clone());
        }
        let actual_owners = line
            .fragment_views()
            .filter_map(|row| match row.owner() {
                text::FinalizedFragmentOwner::Alternative(owner) => Some(owner),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_owners.len(), owners.len());
        assert!(owners.iter().all(|owner| actual_owners.contains(owner)));
        incoming = line.hyphenation_continuation();
        assert_eq!(incoming.is_some(), outgoing && !before);
        if let Some(pending) = &incoming {
            assert_eq!(Some(pending.alternative()), request.alternative);
        }
        start = end;
    }
    assert!(incoming.is_none());
}

fn assert_intrinsics<S: LayoutScalar>(p: &text::PreparedText, minimum: f64, maximum: f64) {
    for (available, expected) in [
        (AvailableOf::MIN_CONTENT, minimum),
        (AvailableOf::MAX_CONTENT, maximum),
    ] {
        let batch = compute::<S>(p, available, None);
        let root = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size.width, S::from_f64(expected));
    }
}

fn assert_outgoing_counter_once(p: &text::PreparedText) {
    assert_outgoing_counter_once_with_style(p, crate::TextWrapStyle::Auto);
}

fn assert_outgoing_counter_once_with_style(
    p: &text::PreparedText,
    wrap_style: crate::TextWrapStyle,
) {
    // Exercise the actual private fitter's committed state, not a duplicated
    // counter fold over public output. No floats/atomics/struts are authored.
    use crate::text_layout::fit;
    let controls = p
        .control_views()
        .map(|row| text::ControlAllocation {
            occurrence: row.handle(),
            strut: None,
            boundary: None,
        })
        .collect::<Vec<_>>();
    let fitted = fit::fit(
        fit::FitInput {
            groups: crate::group_alignment::Snapshot::default(),
            wrap_style,
            wrapping: None,
            formatted: &super::super::formatted::FormattedTailSnapshot::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                Direction::Ltr,
            ),
            prepared: p,
            available: fit::Available::Definite(40.0),
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            block_direction: text::ParagraphBaseDirection::LeftToRight,
            alignment: TextAlign::Start,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::new(
                HyphenateLimitLines::Count(3),
            ),
            hyphenate_limit_zone: LengthPercentageOf::ZERO,
            tail: fit::TailContext::Unrestricted,
            orientation: text::LineOrientation::Horizontal,
            atomics: &[],
            controls: &controls,
            line_strut: None,
            reusable: &[],
        },
        |_, _| {
            Ok(fit::Band {
                start: 0.0,
                end: 40.0,
                intrinsic_end_occupation: None,
                next_transition: None,
            })
        },
        |position, _| {
            Ok(position
                .minimum
                .map_or(position.proposed, |minimum| position.proposed.max(minimum)))
        },
        |line, _, _, box_struts| {
            Ok(fit::LineAllocations {
                line_box_struts: box_struts.to_vec(),
                atomics: Vec::new(),
                controls: line.selected_line().request().controls.clone(),
                line_strut: None,
            })
        },
        |_, _| Ok(None),
        fit::EncounterCallbacks {
            checkpoint_group: || {},
            replay: |_: &text::FinalizedLine, _: fit::EncounterReplay| Ok(Vec::new()),
            accept: || Ok(()),
            reset: |_| {},
        },
    )
    .unwrap_or_else(|_| panic!("the actual two-cut BothSides fitter must complete"));
    use crate::hyphenation::LineHyphenationState::Finite;
    assert_eq!(
        fitted
            .iter()
            .map(|line| line.hyphenation_end)
            .collect::<Vec<_>>(),
        [
            Finite {
                limit: 3,
                remaining: 2
            },
            Finite {
                limit: 3,
                remaining: 1
            },
            Finite {
                limit: 3,
                remaining: 3
            }
        ]
    );
}

fn check<S: LayoutScalar>(definition: Definition, consecutive: bool) {
    for placement in [
        text::HyphenationPlacement::LogicalBefore,
        text::HyphenationPlacement::LogicalAfter,
    ] {
        let p = prepare(definition, consecutive, placement).unwrap();
        qualify(&p, definition, consecutive);
        assert_lines::<S>(&p, definition, consecutive, placement);
        assert_intrinsics::<S>(
            &p,
            if consecutive || placement == text::HyphenationPlacement::LogicalBefore {
                30.0
            } else {
                40.0
            },
            if consecutive { 60.0 } else { 50.0 },
        );
    }
    // Existing callable front door: current nonempty BothSides refusal is the
    // intended semantic RED, after the independently derived controls above.
    let p = prepare(
        definition,
        consecutive,
        text::HyphenationPlacement::BothSides,
    )
    .expect("native exact-string BothSides must admit one indication on each actual side");
    qualify(&p, definition, consecutive);
    assert_lines::<S>(
        &p,
        definition,
        consecutive,
        text::HyphenationPlacement::BothSides,
    );
    assert_intrinsics::<S>(&p, 40.0, if consecutive { 60.0 } else { 50.0 });
    if consecutive {
        assert_outgoing_counter_once(&p);
    }
}

#[test]
fn source_both_sides_has_two_independent_indications_f32() {
    check::<f32>(Definition::Source, false);
}
#[test]
fn source_both_sides_has_two_independent_indications_f64() {
    check::<f64>(Definition::Source, false);
}
#[test]
fn control_both_sides_has_two_independent_indications_f32() {
    check::<f32>(Definition::Control, false);
}
#[test]
fn control_both_sides_has_two_independent_indications_f64() {
    check::<f64>(Definition::Control, false);
}
#[test]
fn consecutive_source_both_sides_keeps_each_outgoing_count_once_f32() {
    check::<f32>(Definition::Source, true);
}
#[test]
fn consecutive_source_both_sides_keeps_each_outgoing_count_once_f64() {
    check::<f64>(Definition::Source, true);
}
#[test]
fn consecutive_control_both_sides_keeps_each_outgoing_count_once_f32() {
    check::<f32>(Definition::Control, true);
}
#[test]
fn consecutive_control_both_sides_keeps_each_outgoing_count_once_f64() {
    check::<f64>(Definition::Control, true);
}

fn check_pretty<S: LayoutScalar>() {
    // At width40 the single actual BothSides cut is the only fitting split:
    // AB + Before A =30; receiving After A + CDE =40. With two cuts, skipping
    // either real cut requires50, so the unique legal sequence is30/40/30.
    // Quality goals cannot create another opportunity or discard a receiver.
    for definition in [Definition::Source, Definition::Control] {
        for consecutive in [false, true] {
            let p = prepare(
                definition,
                consecutive,
                text::HyphenationPlacement::BothSides,
            )
            .expect("real native BothSides fixture must prepare");
            qualify(&p, definition, consecutive);
            assert_lines_with_style::<S>(
                &p,
                definition,
                consecutive,
                text::HyphenationPlacement::BothSides,
                crate::TextWrapStyle::Pretty,
            );
            if consecutive {
                assert_outgoing_counter_once_with_style(&p, crate::TextWrapStyle::Pretty);
            }
        }
    }
}

#[test]
fn pretty_both_sides_retains_real_source_control_receivers_and_outgoing_counts_f32() {
    check_pretty::<f32>();
}

#[test]
fn pretty_both_sides_retains_real_source_control_receivers_and_outgoing_counts_f64() {
    check_pretty::<f64>();
}
