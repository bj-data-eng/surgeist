//! Native custody and presentation contracts over the published BudouX example.
use crate::*;
use std::sync::Arc;

// BudouX 0.9.3's published README marks 今日は▁天気です。.
const ORIGINAL: &str = "今日は天気です。";

fn style(operation: WordSpaceTransform) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        locale: Some("ja".into()),
        wrap: Wrap::Word,
        word_space_transform: operation,
        ..Style::default()
    }
}

fn origin(source: &Source, range: Range) -> ParagraphOrigin {
    ParagraphOrigin {
        source: source.clone(),
        local: range,
        aggregate: range,
        kind: ParagraphOriginKind::Authored,
        text_node: None,
    }
}

fn input(text: &str, operation: WordSpaceTransform, cuts: &[usize]) -> ContextualSourceInput {
    let source = Source::new(text);
    let mut ends = vec![0];
    ends.extend_from_slice(cuts);
    ends.push(source.text().len());
    let origins: Vec<_> = ends
        .windows(2)
        .map(|w| origin(&source, Range::new(w[0], w[1])))
        .collect();
    let connections: Vec<_> = (1..origins.len())
        .map(|i| SourceProcessingConnection {
            before_origin: i - 1,
            after_origin: i,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    let boundary_policies = (0..connections.len())
        .map(|i| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(i),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for i in 0..origins.len() {
        if i > 0 {
            events.push(SourceProcessingEvent::Connection(i - 1));
        }
        events.push(SourceProcessingEvent::Origin {
            origin: i,
            affinity: Affinity::After,
        });
    }
    events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    ContextualSourceInput {
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins,
        connections,
        controls: vec![],
        boundary_policies,
        replacements: vec![],
        scopes: vec![SourceProcessingScope::Block {
            font_reference: None,
            topology: SourceProcessingTopology::Complete,
            trim: WhiteSpaceTrim::default(),
            base_style: style(operation),
            used_direction: ParagraphBaseDirection::LeftToRight,
        }],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events,
    }
}

fn base(input: &mut ContextualSourceInput) -> &mut Style {
    let SourceProcessingScope::Block { base_style, .. } = &mut input.scopes[0] else {
        unreachable!()
    };
    base_style
}

fn analyze(input: ContextualSourceInput) -> Result<AnalyzedSource> {
    let owner = Arc::new(process_contextual_source(input)?);
    AnalyzedSource::from_processed_paragraph(
        owner.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
}

fn prepare(analyzed: &AnalyzedSource) -> PreparedText {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .unwrap();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(analyzed, Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}

fn add_seam_control(input: &mut ContextualSourceInput, kind: ParagraphControlKind) {
    assert_eq!(input.origins.len(), 2);
    input.controls.push(ParagraphControl {
        at: input.origins[0].aggregate.end,
        kind,
    });
    let right = input
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::Origin { origin: 1, .. }))
        .unwrap();
    input.events.insert(
        right,
        SourceProcessingEvent::Control {
            control: input.controls.len() - 1,
            affinity: Affinity::After,
        },
    );
}

fn wrap_inline(input: &mut ContextualSourceInput, governing: Option<Style>) {
    let end = input.source.text().len();
    let start_control = input.controls.len();
    input.controls.extend([
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: end,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ]);
    let scope = input.scopes.len();
    input.scopes.push(SourceProcessingScope::Inline {
        alignment_boundary: None,
        font_reference: None,
        trim: WhiteSpaceTrim::default(),
        style: governing,
        boundaries: Some(SourceProcessingScopeBoundaries {
            start: start_control,
            end: start_control + 1,
        }),
    });
    input.events.splice(
        2..2,
        [
            SourceProcessingEvent::EnterScope(scope),
            SourceProcessingEvent::Control {
                control: start_control,
                affinity: Affinity::After,
            },
        ],
    );
    let exit = input.events.len() - 2;
    input.events.splice(
        exit..exit,
        [
            SourceProcessingEvent::Control {
                control: start_control + 1,
                affinity: Affinity::Before,
            },
            SourceProcessingEvent::ExitScope(scope),
        ],
    );
}

fn wrap_origin(input: &mut ContextualSourceInput, origin: usize) {
    let range = input.origins[origin].aggregate;
    let control = input.controls.len();
    input.controls.extend([
        ParagraphControl {
            at: range.start,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: range.end,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ]);
    let scope = input.scopes.len();
    input.scopes.push(SourceProcessingScope::Inline {
        alignment_boundary: None,
        font_reference: None,
        trim: WhiteSpaceTrim::default(),
        style: Some(style(WordSpaceTransform::SpaceAutoPhrase)),
        boundaries: Some(SourceProcessingScopeBoundaries {
            start: control,
            end: control + 1,
        }),
    });
    let at = input.events.iter().position(|event| matches!(event, SourceProcessingEvent::Origin { origin: actual, .. } if *actual == origin)).unwrap();
    input.events.splice(
        at + 1..at + 1,
        [
            SourceProcessingEvent::Control {
                control: control + 1,
                affinity: Affinity::Before,
            },
            SourceProcessingEvent::ExitScope(scope),
        ],
    );
    input.events.splice(
        at..at,
        [
            SourceProcessingEvent::EnterScope(scope),
            SourceProcessingEvent::Control {
                control,
                affinity: Affinity::After,
            },
        ],
    );
}

fn whole(prepared: &PreparedText) -> FinalizedLine {
    let termination = prepared
        .opportunity_views()
        .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    prepared
        .select(LineSelectionRequest {
            boundary_context: Vec::new(),
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            termination: PreparedTermination::Ordinary(termination),
            alternative: None,
            participants: prepared.participants().collect(),
            atomics: vec![],
            controls: prepared
                .control_views()
                .map(|view| {
                    let occurrence = view.handle();
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
                .collect(),
            line_box_struts: Vec::new(),
            line_strut: None,
            context: LineContext {
                alignment_insets: crate::LineAlignmentInsets::ZERO,
                formatted_line: crate::FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: crate::LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::Spaces,
            },
        })
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn place(prepared: PreparedText, line: FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared,
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 20.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap()
}

#[test]
fn published_phrase_cut_survives_origin_font_and_request_splits() {
    for (cuts, split_request) in [
        (vec![], false),
        (vec![1, 2, 3, 5], false),
        (vec![2, 3], true),
    ] {
        let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &cuts);
        if split_request {
            let mut actual = style(WordSpaceTransform::None);
            actual.size = 17.0;
            capture.source.span(Range::new(0, 3), actual);
            for row in &mut capture.origins {
                row.source = capture.source.clone();
            }
        }
        let analyzed = analyze(capture).unwrap();
        assert_eq!(analyzed.inner.projection.text, "今日は 天気です。");
        assert_eq!(analyzed.source().text(), &Text::from(ORIGINAL));
        assert_eq!(
            analyzed.processed_source().processed_text(),
            &Text::from(ORIGINAL)
        );
        let prepared = prepare(&analyzed);
        let virtuals: Vec<_> = prepared.virtual_separators().collect();
        assert_eq!(virtuals.len(), 1);
        assert_eq!(virtuals[0].view().source_position().get(), 3);
        assert_eq!(virtuals[0].view().processed_position().get(), 3);
        assert_eq!(virtuals[0].view().scope().view().capture_ordinal(), 0);
        assert_eq!(virtuals[0].view().scope().view().role(), ScopeRole::Domain);
    }
}

#[test]
fn generated_separator_survives_each_whitespace_mode_and_full_width_composes_afterward() {
    for mode in [
        WhiteSpace::Collapse,
        WhiteSpace::Preserve,
        WhiteSpace::PreserveSpaces,
        WhiteSpace::Discard,
    ] {
        for operation in [
            WordSpaceTransform::SpaceAutoPhrase,
            WordSpaceTransform::IdeographicSpaceAutoPhrase,
        ] {
            let mut capture = input(ORIGINAL, operation, &[]);
            base(&mut capture).white_space = mode;
            let expected = if operation == WordSpaceTransform::SpaceAutoPhrase {
                "今日は 天気です。"
            } else {
                "今日は　天気です。"
            };
            assert_eq!(
                analyze(capture).unwrap().inner.projection.text,
                expected,
                "{mode:?} {operation:?}"
            );
        }
    }
    let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[]);
    base(&mut capture).text_transform =
        TextTransform::Set(TextTransformSet::none().with_full_width(true));
    assert_eq!(
        analyze(capture).unwrap().inner.projection.text,
        "今日は　天気です。"
    );
}

#[test]
fn unsupported_or_unknown_language_never_borrows_japanese_from_region_or_script() {
    for locale in [
        None,
        Some("und-Jpan"),
        Some("en-JP"),
        Some("ja-Latn"),
        Some("zh"),
        Some("ko"),
    ] {
        let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[]);
        base(&mut capture).locale = locale.map(str::to_owned);
        let SourceProcessingScope::Block { topology, .. } = &mut capture.scopes[0] else {
            unreachable!()
        };
        *topology = SourceProcessingTopology::Unknown;
        assert_eq!(
            analyze(capture).unwrap().inner.projection.text,
            ORIGINAL,
            "{locale:?}"
        );
    }
    for locale in ["ja", "ja-JP", "ja-Jpan", "ja-Jpan-JP"] {
        let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[]);
        base(&mut capture).locale = Some(locale.into());
        assert_eq!(
            analyze(capture).unwrap().inner.projection.text,
            "今日は 天気です。",
            "{locale}"
        );
    }
}

#[test]
fn topology_and_actual_owner_style_are_demanded_only_by_a_surviving_candidate() {
    let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[]);
    let SourceProcessingScope::Block { topology, .. } = &mut capture.scopes[0] else {
        unreachable!()
    };
    *topology = SourceProcessingTopology::Unknown;
    assert_eq!(
        analyze(capture).unwrap_err().detail(),
        Some(&ErrorDetail::TextTransform {
            reason: TextTransformError::MissingPhraseTopology { at: 3 }
        })
    );
    let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[]);
    wrap_inline(&mut capture, None);
    assert_eq!(
        analyze(capture).unwrap_err().detail(),
        Some(&ErrorDetail::TextTransform {
            reason: TextTransformError::MissingPhraseScopeStyle { scope: 1, at: 3 }
        })
    );
    for text in ["", "今", "今日は"] {
        let mut capture = input(text, WordSpaceTransform::SpaceAutoPhrase, &[]);
        wrap_inline(&mut capture, None);
        assert_eq!(analyze(capture).unwrap().inner.projection.text, text);
    }
}

#[test]
fn detected_cut_uses_the_actual_inline_owner_without_redetecting_its_locale() {
    for operation in [
        WordSpaceTransform::None,
        WordSpaceTransform::Space,
        WordSpaceTransform::IdeographicSpace,
    ] {
        let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[3]);
        let mut owner_style = style(operation);
        owner_style.locale = Some("en".into());
        owner_style.size = 23.0;
        wrap_inline(&mut capture, Some(owner_style.clone()));
        let analyzed = analyze(capture).unwrap();
        let expected = match operation {
            WordSpaceTransform::None => ORIGINAL,
            WordSpaceTransform::Space => "今日は 天気です。",
            _ => "今日は　天気です。",
        };
        assert_eq!(analyzed.inner.projection.text, expected);
        let prepared = prepare(&analyzed);
        let virtuals: Vec<_> = prepared.virtual_separators().collect();
        if operation == WordSpaceTransform::None {
            assert!(virtuals.is_empty());
        } else {
            assert_eq!(virtuals.len(), 1);
            let owner = virtuals[0].view().scope();
            assert_eq!(owner.view().capture_ordinal(), 1);
            assert_eq!(owner.view().role(), ScopeRole::Inline);
            assert_eq!(owner.view().style(), &owner_style);
            assert_eq!(
                owner.view().entry_control().unwrap().view().kind(),
                &ParagraphControlKind::InlineBoundaryStart
            );
            assert_eq!(
                owner.view().exit_control().unwrap().view().kind(),
                &ParagraphControlKind::InlineBoundaryEnd
            );
        }
    }
}

#[test]
fn genuine_wbr_and_virtual_cut_keep_independent_ordered_multiplicity() {
    for operation in [
        WordSpaceTransform::None,
        WordSpaceTransform::IdeographicSpace,
    ] {
        let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[3]);
        add_seam_control(
            &mut capture,
            ParagraphControlKind::Wbr {
                style: Box::new(style(operation)),
            },
        );
        let analyzed = analyze(capture).unwrap();
        let expected = if operation == WordSpaceTransform::None {
            "今日は 天気です。"
        } else {
            "今日は 　天気です。"
        };
        assert_eq!(analyzed.inner.projection.text, expected);
        let prepared = prepare(&analyzed);
        assert_eq!(prepared.virtual_separators().count(), 1);
        assert_eq!(prepared.control_views().count(), 1);
        assert_eq!(prepared.origin_views().count(), 2);
    }
    let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[3]);
    add_seam_control(
        &mut capture,
        ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
    );
    assert_eq!(
        analyze(capture).unwrap().inner.projection.text,
        "今日は 天気です。"
    );
    for parent in [WordSpaceTransform::None, WordSpaceTransform::Space] {
        let mut capture = input(ORIGINAL, parent, &[3]);
        capture.source.span(
            Range::new(0, ORIGINAL.encode_utf16().count()),
            style(WordSpaceTransform::SpaceAutoPhrase),
        );
        for origin in &mut capture.origins {
            origin.source = capture.source.clone();
        }
        add_seam_control(
            &mut capture,
            ParagraphControlKind::Wbr {
                style: Box::new(style(WordSpaceTransform::IdeographicSpace)),
            },
        );
        let analyzed = analyze(capture).unwrap();
        assert_eq!(
            analyzed.inner.projection.text,
            if parent == WordSpaceTransform::None {
                "今日は　天気です。"
            } else {
                "今日は 　天気です。"
            }
        );
        assert_eq!(
            prepare(&analyzed).virtual_separators().count(),
            usize::from(parent == WordSpaceTransform::Space)
        );
    }
}

#[test]
fn genuine_forced_seam_suppresses_expansion_before_missing_owner_style_is_consumed() {
    let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[3]);
    add_seam_control(&mut capture, ParagraphControlKind::ForcedBreak);
    wrap_inline(&mut capture, None);
    let analyzed = analyze(capture).unwrap();
    assert_eq!(analyzed.inner.projection.text, ORIGINAL);
    assert_eq!(analyzed.source().text(), &Text::from(ORIGINAL));
    assert!(
        prepare(&analyzed)
            .control_views()
            .any(|control| control.kind() == &ParagraphControlKind::ForcedBreak)
    );
}

#[test]
fn disconnected_origins_do_not_create_an_exterior_phrase_separator() {
    let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[3]);
    capture.connections[0].kind = ParagraphConnectionKind::Disconnected;
    assert_eq!(analyze(capture).unwrap().inner.projection.text, ORIGINAL);
}

#[test]
fn invalid_scope_boundary_facts_refuse_even_when_detection_is_inactive() {
    let mut capture = input("A", WordSpaceTransform::None, &[]);
    wrap_inline(&mut capture, Some(style(WordSpaceTransform::None)));
    let SourceProcessingScope::Inline { boundaries, .. } = &mut capture.scopes[1] else {
        unreachable!()
    };
    boundaries.as_mut().unwrap().end = 0;
    assert_eq!(
        process_contextual_source(capture).unwrap_err().detail(),
        Some(&ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::ScopeBoundaries
        })
    );
    let mut capture = input("A", WordSpaceTransform::None, &[]);
    let mut invalid = style(WordSpaceTransform::None);
    invalid.size = f32::NAN;
    wrap_inline(&mut capture, Some(invalid));
    assert_eq!(
        process_contextual_source(capture).unwrap_err().code,
        ErrorCode::InvalidStyle
    );
}

#[test]
fn a_continuous_origin_split_inside_a_supplementary_scalar_preserves_the_domain_decision() {
    let original = "今日は😀天気です。";
    let unsplit = analyze(input(original, WordSpaceTransform::SpaceAutoPhrase, &[])).unwrap();
    let split = analyze(input(original, WordSpaceTransform::SpaceAutoPhrase, &[4])).unwrap();
    // The split is between the genuine high and low surrogate. Source-domain
    // equivalence is the oracle; no fresh model output is linguistic truth.
    assert_eq!(split.inner.projection.text, unsplit.inner.projection.text);
    assert_eq!(split.source().text(), &Text::from(original));
    assert_eq!(
        prepare(&split).virtual_separators().count(),
        prepare(&unsplit).virtual_separators().count()
    );
}

#[test]
fn selecting_a_real_shy_indication_keeps_virtual_owner_font_and_mapping() {
    let original = "今日は天気です。 AX\u{ad}XXX";
    let mut capture = input(original, WordSpaceTransform::SpaceAutoPhrase, &[]);
    let mut governing = style(WordSpaceTransform::Space);
    governing.size = 23.0;
    wrap_inline(&mut capture, Some(governing));
    let analyzed = analyze(capture).unwrap();
    let prepared = prepare(&analyzed);
    assert_eq!(prepared.virtual_separators().count(), 1);
    let offer = prepared
        .opportunity_views()
        .find(|view| view.kind() == PreparedBreakKind::Discretionary)
        .expect("the real AX-SHY-XXX word admits manual indication");
    let alternative = offer.alternatives().next().unwrap().handle();
    let mut request = whole(&prepared).selected_line().request().clone();
    request.end = offer.boundary();
    request.termination = PreparedTermination::Ordinary(offer.handle());
    request.alternative = Some(alternative.clone());
    request.participants = prepared
        .participants()
        .take(request.end.view().participant_offset())
        .collect();
    request.controls.retain(|allocation| {
        request
            .participants
            .contains(&PreparedParticipant::Control(allocation.occurrence.clone()))
    });
    request.context.end = LineEnd::Soft;
    let selected = crate::test_wrapping::select_neutral(&prepared, request)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert!(
        selected
            .mapping_views()
            .any(|mapping| mapping.alternative() == Some(alternative.clone()))
    );
    let virtual_ = selected
        .mapping_views()
        .find(|mapping| mapping.kind() == PresentationMappingKind::GeneratedSeparator)
        .unwrap()
        .separator_virtual()
        .unwrap();
    assert_eq!(virtual_.view().scope().view().capture_ordinal(), 1);
    assert_eq!(virtual_.view().scope().view().style().size, 23.0);
    let generated_runs: Vec<_> = selected
        .run_views()
        .filter(|run| run.range() == Range::new(3, 3))
        .collect();
    assert!(!generated_runs.is_empty());
    assert!(generated_runs.iter().all(|run| run.font_size() == 23.0));
    assert_eq!(analyzed.source().text(), &Text::from(original));
}

#[test]
fn multiple_virtual_occurrences_share_one_actual_scope_and_keep_source_out_of_copy_and_range() {
    // Independent original upstream marked row: 今日は▁とても▁良い▁天気です。.
    let original = "今日はとても良い天気です。";
    let analyzed = analyze(input(original, WordSpaceTransform::SpaceAutoPhrase, &[])).unwrap();
    assert_eq!(
        analyzed.inner.projection.text,
        "今日は とても 良い 天気です。"
    );
    let prepared = prepare(&analyzed);
    assert_eq!(prepared.scopes().count(), 1);
    let virtuals: Vec<_> = prepared.virtual_separators().collect();
    assert_eq!(
        virtuals
            .iter()
            .map(|handle| handle.view().source_position().get())
            .collect::<Vec<_>>(),
        [3, 6, 8]
    );
    assert_eq!(
        virtuals
            .iter()
            .map(|handle| handle.view().occurrence_ordinal())
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let source_origin = prepared.origin_views().next().unwrap().handle();
    for handle in &virtuals {
        let units: Vec<_> = handle.view().units().collect();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].view().virtual_separators().count(), 1);
        assert_eq!(units[0].view().origins().count(), 0);
    }
    let line = whole(&prepared);
    let mappings: Vec<_> = line
        .mapping_views()
        .filter(|view| view.kind() == PresentationMappingKind::GeneratedSeparator)
        .collect();
    assert_eq!(mappings.len(), 3);
    for (mapping, anchor) in mappings.iter().zip([3, 6, 8]) {
        assert_eq!(mapping.source_range(), Range::new(anchor, anchor));
        assert!(mapping.separator_control().is_none());
        assert_eq!(
            mapping
                .separator_virtual()
                .unwrap()
                .view()
                .source_position()
                .get(),
            anchor
        );
    }
    assert_eq!(
        line.fragment_views()
            .filter(|view| matches!(view.owner(), FinalizedFragmentOwner::VirtualSeparator(_)))
            .count(),
        3
    );
    let layout = place(prepared, line);
    let positions: Vec<_> = layout.positions().collect();
    let selection = layout
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(selection.source_text(), &Text::from(original));
    assert_eq!(selection.plaintext(), &Text::from(original));
    let ranges = layout
        .source_range_fragments(
            &source_origin,
            Range::new(0, original.encode_utf16().count()),
        )
        .unwrap();
    assert!(
        ranges
            .units()
            .all(|unit| unit.virtual_separators().count() == 0)
    );
    #[cfg(feature = "text-accessibility")]
    {
        let mut state = Accessibility::new(200);
        let id = accesskit::NodeId(100);
        let update = layout
            .accessibility_update(&mut state, id, id, Point::new(0.0, 0.0))
            .unwrap();
        assert_eq!(
            update
                .nodes
                .iter()
                .filter_map(|(_, node)| node.value())
                .collect::<String>(),
            original
        );
    }
}

#[test]
fn real_model_proposals_incident_to_existing_separators_and_glue_are_excluded() {
    // Raw proposals establish that the eligibility filter is exercised; they
    // are diagnostic stimulus, never an oracle for linguistic correctness.
    for scalar in [
        '\u{0020}',
        '\u{1361}',
        '\u{10100}',
        '\u{10101}',
        '\u{1039f}',
        '\u{1091f}',
        '\u{103d0}',
        '\u{12470}',
        '\u{2002}',
        '\u{200b}',
        '\u{00a0}',
        '\u{202f}',
        '\u{2060}',
        '\u{200d}',
    ] {
        for (text, raw_cut) in [
            (format!("今日は{scalar}天気です。"), 3),
            (format!("これ以{scalar}上利用する場合は教えてください。"), 4),
        ] {
            let raw = crate::phrase::PhraseModel::BudouXJapanese093
                .boundaries(&text.chars().map(u32::from).collect::<Vec<_>>())
                .unwrap();
            assert!(
                raw.contains(&raw_cut),
                "real model must propose the incident cut {raw_cut} for U+{:04X}",
                u32::from(scalar)
            );
            let analyzed = analyze(input(&text, WordSpaceTransform::SpaceAutoPhrase, &[])).unwrap();
            let prepared = prepare(&analyzed);
            let after = 3 + scalar.len_utf16();
            assert!(
                prepared.virtual_separators().all(
                    |separator| ![3, after].contains(&separator.view().source_position().get())
                ),
                "incident virtual separator survived beside U+{:04X}",
                u32::from(scalar)
            );
            assert_eq!(analyzed.source().text(), &Text::from(text));
        }
    }
}

#[test]
fn typographic_unit_interior_is_excluded_before_uncaptured_style_is_demanded() {
    let text = "今日は\u{3099}天気です。";
    let raw = crate::phrase::PhraseModel::BudouXJapanese093
        .boundaries(&text.chars().map(u32::from).collect::<Vec<_>>())
        .unwrap();
    assert_eq!(
        raw,
        [3],
        "real model proposes inside the authored kana mark unit"
    );
    let mut capture = input(text, WordSpaceTransform::SpaceAutoPhrase, &[]);
    wrap_inline(&mut capture, None);
    assert_eq!(analyze(capture).unwrap().inner.projection.text, text);
}

#[test]
fn punctuation_attachment_filters_genuine_proposals_without_relocating_them() {
    for (text, raw_cut, expected) in [
        ("今日は「天気です。", 3, "今日は 「天気です。"),
        ("今日は」今日は天気です。", 4, "今日は」 今日は 天気です。"),
        ("今日は?今日は天気です。", 4, "今日は? 今日は 天気です。"),
        ("今日は/天気です。", 3, "今日は/天気です。"),
    ] {
        let raw = crate::phrase::PhraseModel::BudouXJapanese093
            .boundaries(&text.chars().map(u32::from).collect::<Vec<_>>())
            .unwrap();
        assert!(raw.contains(&raw_cut), "actual model trigger for {text}");
        assert_eq!(
            analyze(input(text, WordSpaceTransform::SpaceAutoPhrase, &[]))
                .unwrap()
                .inner
                .projection
                .text,
            expected
        );
    }
}

#[test]
fn text_empty_parent_and_empty_seam_scope_keep_the_outer_actual_font_and_style() {
    let mut capture = input(ORIGINAL, WordSpaceTransform::SpaceAutoPhrase, &[3]);
    let mut governing = style(WordSpaceTransform::Space);
    governing.locale = Some("en".into());
    governing.size = 23.0;
    governing.line_height = LineHeight::Absolute(29.0);
    wrap_inline(&mut capture, Some(governing.clone()));
    wrap_origin(&mut capture, 0);
    wrap_origin(&mut capture, 1);
    let control = capture.controls.len();
    capture.controls.extend([
        ParagraphControl {
            at: 3,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 3,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ]);
    let scope = capture.scopes.len();
    capture.scopes.push(SourceProcessingScope::Inline {
        alignment_boundary: None,
        font_reference: None,
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: Some(SourceProcessingScopeBoundaries {
            start: control,
            end: control + 1,
        }),
    });
    let connection = capture
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::Connection(0)))
        .unwrap();
    capture.events.splice(
        connection + 1..connection + 1,
        [
            SourceProcessingEvent::EnterScope(scope),
            SourceProcessingEvent::Control {
                control,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::Control {
                control: control + 1,
                affinity: Affinity::Before,
            },
            SourceProcessingEvent::ExitScope(scope),
        ],
    );
    let analyzed = analyze(capture).unwrap();
    assert_eq!(analyzed.inner.projection.text, "今日は 天気です。");
    let prepared = prepare(&analyzed);
    let virtual_ = prepared.virtual_separators().next().unwrap();
    assert_eq!(virtual_.view().scope().view().capture_ordinal(), 1);
    assert_eq!(virtual_.view().scope().view().style(), &governing);
    let units: Vec<_> = virtual_.view().units().collect();
    assert_eq!(units.len(), 1);
    let sizes: Vec<_> = units[0]
        .view()
        .font_instances()
        .map(|instance| instance.unwrap().font_size())
        .collect();
    assert_eq!(sizes, [23.0]);
    assert_eq!(prepared.participants().filter(|participant| matches!(participant, PreparedParticipant::Text(unit) if unit.view().virtual_separators().count() == 1)).count(), 1);
    let line = whole(&prepared);
    let generated = line
        .fragment_views()
        .find(|view| matches!(view.owner(), FinalizedFragmentOwner::VirtualSeparator(_)))
        .unwrap();
    assert!(generated.origin().is_none());
}

#[test]
fn out_of_flow_child_interiors_do_not_split_parent_analysis_but_in_flow_atomics_do() {
    for kind in [InlineBoxKind::OutOfFlow, InlineBoxKind::InFlow] {
        let mut capture = input(
            "今日はX天気です。",
            WordSpaceTransform::SpaceAutoPhrase,
            &[3, 4],
        );
        capture.source.inline_box(InlineBox::new(
            Id::from_u64(63),
            kind,
            3,
            Size::new(5.0, 10.0),
        ));
        capture.connections = vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 2,
            kind: ParagraphConnectionKind::Continuous,
        }];
        capture.boundary_policies.truncate(1);
        if kind == InlineBoxKind::InFlow {
            capture.boundary_policies.extend([
                ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::AtomicBefore(0),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                },
                ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::AtomicAfter(0),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                },
            ]);
        }
        let child_style = style(WordSpaceTransform::None);
        capture.scopes.push(match kind {
            InlineBoxKind::OutOfFlow => SourceProcessingScope::OutOfFlowBlock {
                font_reference: None,
                topology: SourceProcessingTopology::Complete,
                trim: WhiteSpaceTrim::default(),
                base_style: child_style,
                used_direction: ParagraphBaseDirection::LeftToRight,
                atomic: 0,
            },
            InlineBoxKind::InFlow => SourceProcessingScope::AtomicBlock {
                font_reference: None,
                topology: SourceProcessingTopology::Complete,
                trim: WhiteSpaceTrim::default(),
                base_style: child_style,
                used_direction: ParagraphBaseDirection::LeftToRight,
                atomic: 0,
            },
        });
        capture.paragraphs.push(SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        });
        capture.events = vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::Atomic {
                atomic: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::EnterParagraph(1),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ];
        let processed = Arc::new(process_contextual_source(capture).unwrap());
        let views: Vec<_> = processed.paragraphs().collect();
        let parent = AnalyzedSource::from_processed_paragraph(
            views[0].clone(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            parent.inner.projection.text,
            if kind == InlineBoxKind::OutOfFlow {
                "今日は 天気です。"
            } else {
                ORIGINAL
            }
        );
        assert_eq!(
            AnalyzedSource::from_processed_paragraph(
                views[1].clone(),
                crate::AnalysisTypography::HorizontalComposition
            )
            .unwrap()
            .inner
            .projection
            .text,
            "X"
        );
        assert_eq!(parent.source().text(), &Text::from("今日はX天気です。"));
    }
}
