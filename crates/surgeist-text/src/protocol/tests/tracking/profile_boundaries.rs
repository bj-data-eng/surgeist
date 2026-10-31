//! Complete native profile admission, actual atomic-barrier custody and neutral
//! operations over actual registered shaping. No subset is a complete basis.
use super::fixtures::allocations;
use super::*;
use crate::{AtomicPlacement, Cursor, InlineBox, InlineBoxKind, Point, Rect, Size};

const BOX: Id = Id::from_u64(1566090);

fn system_with(bytes: &[u8]) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system
}

fn analyze(source: &Source, style: Style) -> AnalyzedSource {
    let extent = Range::new(0, source.text().len());
    let input = ParagraphInput {
        source: source.clone(),
        base_style: style,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: extent,
            aggregate: extent,
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        replacements: vec![],
        anchor_order: source
            .boxes()
            .iter()
            .enumerate()
            .map(|(occurrence, _)| crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::Atomic(occurrence),
                affinity: crate::Affinity::After,
            })
            .collect(),
        boundary_policies: source
            .boxes()
            .iter()
            .enumerate()
            .flat_map(|(occurrence, _)| {
                [
                    crate::ParagraphBoundaryDefinition::AtomicBefore(occurrence),
                    crate::ParagraphBoundaryDefinition::AtomicAfter(occurrence),
                ]
            })
            .map(|definition| crate::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
    };
    let analyzed =
        AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
    crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word)
}

fn prepared(system: &mut System, source: &Source, style: Style) -> crate::Result<PreparedText> {
    let analyzed = analyze(source, style);
    assert_eq!(analyzed.source(), source);
    system.prepare(&analyzed, PreparationOptions::default())
}

fn prepared_raw(system: &mut System, source: &Source, style: Style) -> crate::Result<PreparedText> {
    // This existing public route retains original host grapheme readback.
    // Processing and selected spacing still honor actual atomic barriers.
    let analyzed = AnalyzedSource::new(
        source.clone(),
        style,
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.source(), source);
    system.prepare(&analyzed, PreparationOptions::default())
}

fn flow() -> AtomicLineAllocation {
    AtomicLineAllocation::InFlow {
        fit: crate::AtomicFitAllocation::NativeOccupied,
        flow: AtomicFlowAllocation {
            advance: 10.0,
            before_baseline: 8.0,
            after_baseline: 2.0,
            occupied_start: 0.0,
            occupied_end: 10.0,
        },
        box_geometry: AtomicBoxAllocation {
            border: Rect::new(0.0, -8.0, 10.0, 10.0),
            content: Rect::new(0.0, -8.0, 10.0, 10.0),
            baseline: 0.0,
        },
        alignment: LineBoxAlignment::Baseline,
    }
}

fn selection(
    prepared: &PreparedText,
    method: SpacingPolicy,
    width: Option<f64>,
) -> crate::Result<SelectedLine> {
    let mut request = request(prepared);
    request.context.spacing = method;
    request.context.available_inline = width;
    request.atomics = prepared
        .atomic_views()
        .map(|view| AtomicAllocation {
            occurrence: view.handle(),
            allocation: if view.original_box().kind() == InlineBoxKind::InFlow {
                flow()
            } else {
                AtomicLineAllocation::OutOfFlow
            },
        })
        .collect();
    prepared.select(request)
}

fn ordinary(prepared: &PreparedText) -> FinalizedLine {
    selection(prepared, SpacingPolicy::None, None)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn placed(prepared: &PreparedText, line: &FinalizedLine) -> crate::Layout {
    // Host placement uses the actual finalized H slots and the same supplied
    // box geometry. It supplies no provider glyph or character-unit geometry.
    let map = |origin: Point, rect: Rect| {
        Rect::new(
            20.0 + origin.x + rect.origin.x,
            70.0 + origin.y - line.feedback().baseline + rect.origin.y,
            rect.size.width,
            rect.size.height,
        )
    };
    let atomics = line
        .atomic_slots()
        .map(|row| match row.allocation {
            AtomicLineAllocation::InFlow { box_geometry, .. } => AtomicPlacement {
                occurrence: row.occurrence,
                border: map(row.origin, box_geometry.border),
                content: map(row.origin, box_geometry.content),
                baseline: 70.0 + row.origin.y - line.feedback().baseline + box_geometry.baseline,
            },
            AtomicLineAllocation::OutOfFlow => AtomicPlacement {
                occurrence: row.occurrence,
                border: Rect::new(120.0, 60.0, 10.0, 10.0),
                content: Rect::new(120.0, 60.0, 10.0, 10.0),
                baseline: 70.0,
            },
        })
        .collect();
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 70.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics,
        })],
    })
    .unwrap()
}

fn carets(layout: &crate::Layout) -> Vec<(Cursor, Rect)> {
    layout
        .positions()
        .map(|position| {
            (
                position.source_cursor(),
                layout.position_geometry(&position).unwrap().rect(),
            )
        })
        .collect()
}

fn piece(range: Range) -> crate::SpacingPiece {
    crate::SpacingPiece::Source {
        origin: 0,
        local: range,
        aggregate: range,
    }
}

fn profile_error(
    error: &crate::Error,
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
    script: [u8; 4],
    locale: Option<&str>,
    tracking: f64,
) {
    assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
    let Some(crate::ErrorDetail::UnsupportedSpacing {
        stage: actual_stage,
        operation: actual_operation,
        pieces,
        provider_atoms,
        reason,
        contexts,
        constraints,
    }) = error.detail()
    else {
        panic!("expected an exact profile admission failure: {error:?}")
    };
    assert_eq!((*actual_stage, *actual_operation), (stage, operation));
    assert_eq!(
        *reason,
        crate::SpacingFailureReason::UnqualifiedWritingSystem
    );
    assert_eq!(pieces.as_ref(), [piece(Range::new(0, 1))]);
    assert_eq!(provider_atoms.as_ref(), std::slice::from_ref(&(0..1)));
    assert!(
        constraints.is_empty(),
        "profile admission precedes provider constraints"
    );
    assert_eq!(contexts.len(), 1);
    let context = &contexts[0];
    assert_eq!(context.pieces.as_ref(), pieces.as_ref());
    assert_eq!(context.locale.as_deref(), locale);
    assert_eq!(
        context
            .candidates
            .iter()
            .map(|script| script.iso15924())
            .collect::<Vec<_>>(),
        [script]
    );
    assert_eq!(
        context
            .resolved_scripts
            .iter()
            .map(|script| script.iso15924())
            .collect::<Vec<_>>(),
        [script]
    );
    assert!(context.writing_systems.is_empty());
    assert_eq!(
        (context.letter_spacing_before, context.letter_spacing_after),
        (tracking, tracking)
    );
}

fn mapped_font() -> Vec<u8> {
    // Real cmap data changes resource coverage, never the source's Unicode
    // properties. Nominal A/B geometry does not advertise complex shaping.
    let mappings = [
        (0x41_u32, 35_u32),
        (0x42, 36),
        (0x5d0, 36),
        (0x627, 36),
        (0x915, 35),
        (0x916, 36),
        (0xe01, 35),
        (0xe02, 36),
        (0xe81, 35),
        (0xe82, 36),
    ];
    let mut cmap = vec![];
    for word in [0u16, 1, 3, 10] {
        cmap.extend(word.to_be_bytes());
    }
    cmap.extend(12u32.to_be_bytes());
    for word in [12u16, 0] {
        cmap.extend(word.to_be_bytes());
    }
    cmap.extend((16 + mappings.len() as u32 * 12).to_be_bytes());
    cmap.extend(0u32.to_be_bytes());
    cmap.extend((mappings.len() as u32).to_be_bytes());
    for (scalar, glyph) in mappings {
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let bytes = sfnt::replace_tables(AHEM, vec![(*b"cmap", cmap)]);
    font_premises(&bytes, false);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    for (scalar, glyph) in mappings {
        assert_eq!(
            font.charmap().map(char::from_u32(scalar).unwrap()),
            Some(GlyphId::new(glyph))
        );
    }
    assert!(font.table_data(skrifa::Tag::new(b"GSUB")).is_none());
    bytes
}

#[test]
fn unqualified_complex_profiles_allow_ordinary_zero_but_never_issue_a_complete_character_subset() {
    let bytes = mapped_font();
    let mut system = system_with(&bytes);
    for (source, script) in [
        ("\u{e01}\u{e02}", *b"Thai"),
        ("\u{e81}\u{e82}", *b"Laoo"),
        ("\u{915}\u{916}", *b"Deva"),
    ] {
        let source = Source::new(source);
        let prepared = prepared(&mut system, &source, own(0.0)).unwrap();
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 2)]
        );
        let natural = ordinary(&prepared);
        assert_eq!(
            raw(&natural, &bytes),
            [
                (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
                (Range::new(1, 2), 36, 0.0, 0.0, 10.0)
            ]
        );
        assert_eq!(natural.feedback().advance, 20.0);
        let old = placed(&prepared, &natural);
        // This is exact natural capacity, so no numeric extra could demand
        // spacing. The explicit API nevertheless promises a complete basis.
        let error = selection(&prepared, SpacingPolicy::InterCharacter, Some(20.0)).unwrap_err();
        profile_error(
            &error,
            crate::SpacingStage::Selection,
            crate::SpacingOperation::InterCharacterBasis,
            script,
            None,
            0.0,
        );
        assert_eq!(old.source().text(), source.text());
        assert_eq!(natural.feedback().advance, 20.0);
        assert_eq!(ordinary(&prepared).feedback().advance, 20.0);
        let error = self::prepared(&mut system, &source, own(4.0)).unwrap_err();
        profile_error(
            &error,
            crate::SpacingStage::Preparation,
            crate::SpacingOperation::AuthoredTracking,
            script,
            None,
            4.0,
        );
    }
    // One unsupported unit has no between-unit candidate to partially issue.
    let source = Source::new("\u{e01}");
    let prepared = prepared(&mut system, &source, own(0.0)).unwrap();
    assert_eq!(ordinary(&prepared).feedback().advance, 10.0);
    let error = selection(&prepared, SpacingPolicy::InterCharacter, Some(10.0)).unwrap_err();
    profile_error(
        &error,
        crate::SpacingStage::Selection,
        crate::SpacingOperation::InterCharacterBasis,
        *b"Thai",
        None,
        0.0,
    );
}

#[test]
fn an_unqualified_bare_language_remains_declared_and_refuses_material_or_complete_basis_demand() {
    let mut system = system_with(AHEM);
    let source = Source::new("AB");
    let mut style = own(0.0);
    style.locale = Some("qaa".into());
    let prepared = prepared(&mut system, &source, style.clone()).unwrap();
    assert_eq!(
        prepared.analyzed_source().style().locale.as_deref(),
        Some("qaa")
    );
    let natural = ordinary(&prepared);
    assert_eq!(natural.feedback().advance, 20.0);
    assert_eq!(
        raw(&natural, AHEM),
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 36, 0.0, 0.0, 10.0)
        ]
    );
    let error = selection(&prepared, SpacingPolicy::InterCharacter, Some(20.0)).unwrap_err();
    profile_error(
        &error,
        crate::SpacingStage::Selection,
        crate::SpacingOperation::InterCharacterBasis,
        *b"Latn",
        Some("qaa"),
        0.0,
    );
    style.letter_spacing = 4.0;
    let error = self::prepared(&mut system, &source, style).unwrap_err();
    profile_error(
        &error,
        crate::SpacingStage::Preparation,
        crate::SpacingOperation::AuthoredTracking,
        *b"Latn",
        Some("qaa"),
        4.0,
    );
    assert_eq!(natural.feedback().advance, 20.0);
}

fn marked_source(at: usize, kind: InlineBoxKind, exterior: f64) -> Source {
    let mut source = Source::new("A\u{301}B");
    source.inline_box(
        InlineBox::new(BOX, kind, at, Size::new(10.0, 10.0))
            .try_with_letter_spacing(exterior)
            .unwrap(),
    );
    source
}

fn mark_scripts(resolved: bool) -> Vec<[u8; 4]> {
    use icu_properties::{PropertyNamesShort, props::Script, script::ScriptWithExtensions};
    let names = PropertyNamesShort::<Script>::new();
    ScriptWithExtensions::new()
        .get_script_extensions_val32(0x301)
        .iter()
        .filter(|script| {
            !resolved
                || !matches!(
                    *script,
                    Script::Common | Script::Inherited | Script::Unknown
                )
        })
        .map(|script| names.get(script).unwrap().as_bytes().try_into().unwrap())
        .collect()
}

fn mark_context(context: &crate::SpacingContext, pieces: &[crate::SpacingPiece], tracking: f64) {
    use icu_properties::{CodePointMapData, props::GeneralCategory};
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get('\u{301}'),
        GeneralCategory::NonspacingMark
    );
    assert_eq!(context.pieces.as_ref(), pieces);
    assert_eq!(context.locale, None);
    assert_eq!(
        context
            .candidates
            .iter()
            .map(|script| script.iso15924())
            .collect::<Vec<_>>(),
        mark_scripts(false)
    );
    assert_eq!(
        context
            .resolved_scripts
            .iter()
            .map(|script| script.iso15924())
            .collect::<Vec<_>>(),
        mark_scripts(true)
    );
    assert!(context.writing_systems.is_empty());
    assert_eq!(
        (context.letter_spacing_before, context.letter_spacing_after),
        (tracking, tracking)
    );
}

fn mark_profile_error(
    error: &crate::Error,
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
    mark_pieces: &[crate::SpacingPiece],
    atoms: &[std::ops::Range<usize>],
    tracking: f64,
    preceding_atomic: Option<f64>,
) {
    assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
    let Some(crate::ErrorDetail::UnsupportedSpacing {
        stage: actual_stage,
        operation: actual_operation,
        pieces,
        provider_atoms,
        reason,
        contexts,
        constraints,
    }) = error.detail()
    else {
        panic!("expected actual standalone-mark profile custody: {error:?}")
    };
    assert_eq!((*actual_stage, *actual_operation), (stage, operation));
    assert_eq!(
        *reason,
        crate::SpacingFailureReason::UnqualifiedWritingSystem
    );
    assert_eq!(provider_atoms.as_ref(), atoms);
    assert!(constraints.is_empty());
    if let Some(value) = preceding_atomic {
        let mut expected = vec![crate::SpacingPiece::Atomic { occurrence: 0 }];
        expected.extend_from_slice(mark_pieces);
        assert_eq!(pieces.as_ref(), expected);
        assert_eq!(contexts.len(), 2);
        let atomic = &contexts[0];
        assert_eq!(
            atomic.pieces.as_ref(),
            [crate::SpacingPiece::Atomic { occurrence: 0 }]
        );
        assert_eq!(
            (atomic.letter_spacing_before, atomic.letter_spacing_after),
            (value, value)
        );
        assert_eq!(atomic.locale, None);
        assert!(atomic.candidates.is_empty());
        assert!(atomic.resolved_scripts.is_empty());
        assert!(atomic.writing_systems.is_empty());
        mark_context(&contexts[1], mark_pieces, tracking);
    } else {
        assert_eq!(pieces.as_ref(), mark_pieces);
        assert_eq!(contexts.len(), 1);
        mark_context(&contexts[0], mark_pieces, tracking);
    }
}

#[test]
fn actual_atomic_barrier_preserves_zero_but_standalone_mark_demands_its_own_profile() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    font_premises(&bytes, true);
    let mut system = system_with(&bytes);
    let source = marked_source(1, InlineBoxKind::InFlow, 0.0);
    for native in [true, false] {
        let route = |system: &mut System, source: &Source, style| {
            if native {
                prepared(system, source, style)
            } else {
                prepared_raw(system, source, style)
            }
        };
        let prepared = route(&mut system, &source, own(0.0)).unwrap();
        // Host readback is route-specific; neither host view overrides the
        // actual processed/selected split at the in-flow atomic occurrence.
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            if native {
                vec![Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
            } else {
                vec![Range::new(0, 2), Range::new(2, 3)]
            }
        );
        assert_eq!(
            prepared
                .unit_views()
                .map(|unit| unit.source_range())
                .collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
        );
        assert_eq!(
            prepared
                .atomic_views()
                .next()
                .unwrap()
                .original_box()
                .index(),
            1
        );
        let baseline = ordinary(&prepared);
        let baseline_raw = raw(&baseline, &bytes);
        let baseline_layout = placed(&prepared, &baseline);
        let baseline_feedback = baseline.feedback();
        let baseline_carets = carets(&baseline_layout);
        assert_eq!(baseline_layout.source().text(), source.text());
        assert_eq!(baseline_layout.inline_boxes().len(), 1);

        // An unused empty span is not material. The actual standalone mark's
        // own nonzero style, unlike an attached mark, is governing here.
        let mut unused = source.clone();
        unused.span(Range::new(3, 3), own(99.0));
        let unused_prepared = route(&mut system, &unused, own(0.0)).unwrap();
        let unchanged = ordinary(&unused_prepared);
        let unchanged_layout = placed(&unused_prepared, &unchanged);
        assert_eq!(raw(&unchanged, &bytes), baseline_raw);
        assert_eq!(unchanged.feedback(), baseline_feedback);
        assert_eq!(
            unchanged_layout.inline_boxes(),
            baseline_layout.inline_boxes()
        );
        assert_eq!(carets(&unchanged_layout), baseline_carets);
        let mut marked = source.clone();
        marked.span(Range::new(1, 2), own(99.0));
        let error = route(&mut system, &marked, own(0.0)).unwrap_err();
        mark_profile_error(
            &error,
            crate::SpacingStage::Preparation,
            crate::SpacingOperation::AuthoredTracking,
            &[piece(Range::new(1, 2))],
            std::slice::from_ref(&(1..2)),
            99.0,
            None,
        );

        let error = selection(
            &prepared,
            SpacingPolicy::InterCharacter,
            Some(baseline.feedback().advance),
        )
        .unwrap_err();
        mark_profile_error(
            &error,
            crate::SpacingStage::Selection,
            crate::SpacingOperation::InterCharacterBasis,
            &[piece(Range::new(1, 2))],
            std::slice::from_ref(&(1..2)),
            0.0,
            None,
        );
        let error = selection(
            &unused_prepared,
            SpacingPolicy::InterCharacter,
            Some(unchanged.feedback().advance),
        )
        .unwrap_err();
        mark_profile_error(
            &error,
            crate::SpacingStage::Selection,
            crate::SpacingOperation::InterCharacterBasis,
            &[piece(Range::new(1, 2))],
            std::slice::from_ref(&(1..2)),
            0.0,
            None,
        );
        for (text, atomic) in [(4.0, 0.0), (0.0, 4.0), (4.0, -4.0)] {
            let material = marked_source(1, InlineBoxKind::InFlow, atomic);
            let error = route(&mut system, &material, own(text)).unwrap_err();
            // A material unqualified unit is admitted before boundary effects.
            // With mark0/atomic4, the first failing effect is Atomic -> mark.
            mark_profile_error(
                &error,
                crate::SpacingStage::Preparation,
                crate::SpacingOperation::AuthoredTracking,
                &[piece(Range::new(1, 2))],
                std::slice::from_ref(&(1..2)),
                text,
                (text == 0.0).then_some(atomic),
            );
        }
        assert_eq!(raw(&baseline, &bytes), baseline_raw);
        assert_eq!(baseline.feedback(), baseline_feedback);
        assert_eq!(carets(&baseline_layout), baseline_carets);
        assert_eq!(raw(&ordinary(&prepared), &bytes), baseline_raw);
    }
}

#[test]
fn an_actual_atomic_at_the_egc_endpoint_uses_its_own_exterior_value_and_remains_supported() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    font_premises(&bytes, true);
    let mut system = system_with(&bytes);
    let control_source = marked_source(2, InlineBoxKind::InFlow, 0.0);
    let control = prepared(&mut system, &control_source, own(0.0)).unwrap();
    let control = ordinary(&control);
    let expected = [
        (Range::new(0, 2), 35, 0.0, 0.0, 10.0),
        (Range::new(0, 2), 100, -7.0, -4.0, 0.0),
        (Range::new(2, 3), 36, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&control, &bytes), expected);
    assert_eq!(control.feedback().advance, 30.0);
    // Here the atomic is at the full A+mark endpoint: mark tracking is
    // genuinely nongoverning, unlike the standalone mark at atomic1 above.
    let mut nongoverning = control_source.clone();
    nongoverning.span(Range::new(1, 2), own(99.0));
    let attached = prepared(&mut system, &nongoverning, own(0.0)).unwrap();
    let attached_line = ordinary(&attached);
    assert_eq!(raw(&attached_line, &bytes), expected);
    assert_eq!(attached_line.feedback().advance, 30.0);
    let attached_selection =
        selection(&attached, SpacingPolicy::InterCharacter, Some(30.0)).unwrap();
    assert_eq!(attached_selection.spacing_slots().len(), 2);
    assert_eq!(
        attached_selection
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None
            })
            .unwrap()
            .feedback()
            .advance,
        30.0
    );
    let source = marked_source(2, InlineBoxKind::InFlow, 8.0);
    let prepared = prepared(&mut system, &source, own(4.0)).unwrap();
    let selected = selection(&prepared, SpacingPolicy::InterCharacter, Some(42.0)).unwrap();
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(raw(&natural, &bytes), expected);
    assert_eq!(natural.feedback().advance, 42.0);
    let rows = natural.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_eq!(
            row.geometry,
            crate::SpacingSlotGeometry::Boundary(crate::SpacingBoundaryGeometry::Independent)
        );
        assert_eq!(
            (row.authored_letter_spacing, row.allocation, row.advance),
            (6.0, 0.0, 6.0)
        );
    }
    let atomic = natural.atomic_slots().next().unwrap();
    assert_eq!(
        atomic.occurrence,
        prepared.atomic_views().next().unwrap().handle()
    );
    assert_eq!(atomic.origin.x, 16.0);
    assert_eq!(atomic.allocation, flow());
    assert_eq!(
        paint_offsets(&natural),
        [(0.0, 0.0), (3.0, -4.0), (32.0, 0.0)]
    );
    let allocated = natural
        .with_spacing(allocations(&selected, &[1.0, 1.0]))
        .unwrap();
    assert_eq!(allocated.feedback().advance, 44.0);
    assert_eq!(allocated.atomic_slots().next().unwrap().origin.x, 17.0);
    assert_eq!(
        paint_offsets(&allocated),
        [(0.0, 0.0), (3.0, -4.0), (34.0, 0.0)]
    );
    assert_eq!(raw(&allocated, &bytes), expected);
    assert_eq!(natural.feedback().advance, 42.0);
}

#[test]
fn positioned_atomic_and_ignored_format_keep_the_two_supported_endpoint_units() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    font_premises(&bytes, true);
    let mut system = system_with(&bytes);
    for source in [
        marked_source(1, InlineBoxKind::OutOfFlow, 99.0),
        Source::new("A\u{200d}\u{301}B"),
    ] {
        let control = prepared(&mut system, &source, own(0.0)).unwrap();
        let control = ordinary(&control);
        let provider = raw(&control, &bytes);
        assert_eq!(control.feedback().advance, 20.0);
        let prepared = prepared(&mut system, &source, own(4.0)).unwrap();
        let selected = selection(&prepared, SpacingPolicy::InterCharacter, Some(26.0)).unwrap();
        let natural = selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(natural.feedback().advance, 24.0);
        assert_eq!(raw(&natural, &bytes), provider);
        assert_eq!(selected.spacing_slots().len(), 1);
        assert!(
            !selected
                .spacing_slots()
                .next()
                .unwrap()
                .pieces()
                .any(|piece| matches!(piece.owner(), crate::SpacingSlotOwner::Atomic(_)))
        );
        let actual = natural
            .with_spacing(allocations(&selected, &[2.0]))
            .unwrap();
        assert_eq!(actual.measurements().alignment().advance(), 26.0);
        assert_eq!(raw(&actual, &bytes), provider);
        let b = actual
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .filter(|glyph| glyph.id() == 36)
                    .map(|glyph| run.origin().x + glyph.pen())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(b, [16.0]);
        assert_eq!(prepared.analyzed_source().source(), &source);
    }
}

#[test]
fn declared_en_does_not_misclassify_unambiguous_hebrew_or_arabic_as_latin_membership_failure() {
    let bytes = mapped_font();
    let mut system = system_with(&bytes);
    for (source, scalar, script) in [
        ("A\u{5d0}", '\u{5d0}', icu_properties::props::Script::Hebrew),
        ("A\u{627}", '\u{627}', icu_properties::props::Script::Arabic),
    ] {
        assert_eq!(
            icu_properties::CodePointMapData::<icu_properties::props::Script>::new().get(scalar),
            script
        );
        let source = Source::new(source);
        let mut style = own(0.0);
        style.locale = Some("en".into());
        let control = prepared(&mut system, &source, style.clone()).unwrap();
        let control = ordinary(&control);
        let expected = [
            (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 36, 0.0, 0.0, 10.0),
        ];
        assert_eq!(raw(&control, &bytes), expected);
        assert_eq!(control.feedback().advance, 20.0);
        style.letter_spacing = 4.0;
        let prepared = prepared(&mut system, &source, style).unwrap();
        assert_eq!(
            prepared.analyzed_source().style().locale.as_deref(),
            Some("en")
        );
        assert_eq!(prepared.analyzed_source().source().text(), source.text());
        let natural = ordinary(&prepared);
        assert_eq!(natural.feedback().advance, 24.0);
        assert_eq!(raw(&natural, &bytes), expected);
        let selected = selection(&prepared, SpacingPolicy::InterCharacter, Some(26.0)).unwrap();
        let actual = selected
            .finalize(crate::LineAllocations {
                spacing: allocations(&selected, &[2.0]),
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(actual.feedback().advance, 26.0);
        assert_eq!(actual.measurements().alignment().advance(), 26.0);
        assert_eq!(raw(&actual, &bytes), expected);
        assert_eq!(paint_offsets(&actual), [(0.0, 0.0), (16.0, 0.0)]);
        let row = actual.spacing_slots().next().unwrap();
        assert_eq!(
            (row.authored_letter_spacing, row.allocation, row.advance),
            (4.0, 2.0, 6.0)
        );
    }
}

#[test]
fn profile_and_atomic_barrier_refusals_preserve_a_real_completed_layout_and_its_cache_publication()
{
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let mut system = system_with(&bytes);
    let source = Source::identified("AB", Id::from_u64(1566091), 1);
    let options = crate::Options {
        quantize: false,
        ..crate::Options::default()
    };
    let published = system.layout(source.clone(), own(0.0), options).unwrap();
    let before = system.stats();
    let before_carets = carets(&published);
    let before_glyphs = published
        .glyph_runs()
        .into_iter()
        .flat_map(|run| run.glyphs().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(published.source(), &source);
    let seam = marked_source(1, InlineBoxKind::InFlow, 0.0);
    let zero = prepared_raw(&mut system, &seam, own(0.0)).unwrap();
    assert_eq!(
        zero.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    assert_eq!(
        zero.unit_views()
            .map(|unit| unit.source_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    let zero_line = ordinary(&zero);
    let zero_packet = raw(&zero_line, &bytes);
    let zero_layout = placed(&zero, &zero_line);
    assert_eq!(zero_layout.source().text(), seam.text());
    let zero_carets = carets(&zero_layout);
    let error = selection(
        &zero,
        SpacingPolicy::InterCharacter,
        Some(zero_line.feedback().advance),
    )
    .unwrap_err();
    mark_profile_error(
        &error,
        crate::SpacingStage::Selection,
        crate::SpacingOperation::InterCharacterBasis,
        &[piece(Range::new(1, 2))],
        std::slice::from_ref(&(1..2)),
        0.0,
        None,
    );
    let error = prepared_raw(&mut system, &seam, own(4.0)).unwrap_err();
    mark_profile_error(
        &error,
        crate::SpacingStage::Preparation,
        crate::SpacingOperation::AuthoredTracking,
        &[piece(Range::new(1, 2))],
        std::slice::from_ref(&(1..2)),
        4.0,
        None,
    );
    let mut unqualified = own(0.0);
    unqualified.locale = Some("qaa".into());
    let unknown = prepared(&mut system, &source, unqualified).unwrap();
    let error = selection(&unknown, SpacingPolicy::InterCharacter, Some(20.0)).unwrap_err();
    profile_error(
        &error,
        crate::SpacingStage::Selection,
        crate::SpacingOperation::InterCharacterBasis,
        *b"Latn",
        Some("qaa"),
        0.0,
    );
    assert_eq!(carets(&published), before_carets);
    assert_eq!(
        published
            .glyph_runs()
            .into_iter()
            .flat_map(|run| run.glyphs().to_vec())
            .collect::<Vec<_>>(),
        before_glyphs
    );
    assert_eq!(raw(&ordinary(&zero), &bytes), zero_packet);
    assert_eq!(carets(&zero_layout), zero_carets);
    let cached = system.layout(source.clone(), own(0.0), options).unwrap();
    assert_eq!(system.stats().layout_hits(), before.layout_hits() + 1);
    assert_eq!(system.stats().layout_misses(), before.layout_misses());
    assert_eq!(cached.source(), &source);
    assert_eq!(carets(&cached), before_carets);
}

#[path = "format_barriers.rs"]
mod format_barriers;

#[path = "visual_unit_interleave.rs"]
mod visual_unit_interleave;
