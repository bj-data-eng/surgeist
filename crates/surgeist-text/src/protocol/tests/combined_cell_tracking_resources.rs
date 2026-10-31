//! Named retained-subset qualification of genuine mixed ordinary/Cell tracking.
//! Private Graph script/region buffers are outside these storage assertions;
//! their complete inventory remains an owning-source review obligation.
#[path = "combined_cell_allocated_tracking.rs"]
mod allocated;
#[path = "combined_cell_allocated_word.rs"]
mod allocated_word;

use super::*;
use crate::engine::{
    layout::data::ShapedData,
    scope_alignment::{accounting, census},
    spacing::character,
};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use std::mem::{size_of, size_of_val};

struct Mixed {
    prepared: PreparedText,
    processed: Arc<ProcessedSource>,
    parent: FontData,
    body: FontData,
}

fn mixed() -> Mixed {
    mixed_with_cell("BBAAB", Range::new(2, 4), 0.0, 784_972)
}

fn mixed_with_cell(text: &str, inner: Range, inner_word_spacing: f64, source_id: u64) -> Mixed {
    assert!(text.is_ascii());
    let count = text.encode_utf16().count();
    assert!(inner.start < inner.end && inner.end <= count);
    assert!(text[inner.start..inner.end].starts_with('A'));
    assert!(text[inner.start..inner.end].ends_with('A'));
    let parent_bytes = parent_font();
    let body_bytes = composition_font();
    // The same glyph ID36 has different genuine horizontal advances in these
    // distinct retained families: outside B1000 versus hwid's real B500.
    assert_eq!(
        sfnt::u16_at(sfnt::table(&parent_bytes, *b"hmtx"), 36 * 4),
        1000
    );
    assert_eq!(
        sfnt::u16_at(sfnt::table(&body_bytes, *b"hmtx"), 36 * 4),
        500
    );
    glyph_frame_font_premises(&body_bytes);
    let outside = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        letter_spacing: 4.0,
        word_spacing: 0.0,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let inside = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: TextCombineUpright::All,
        letter_spacing: 2.0,
        word_spacing: inner_word_spacing,
        ..outside.clone()
    };
    let mut source = Source::identified(text, Id::from_u64(source_id), 1);
    source.span(inner, inside.clone());
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for index in 0..count {
        if index > 0 {
            events.push(SourceProcessingEvent::Connection(index - 1));
        }
        if index == inner.start {
            events.push(SourceProcessingEvent::EnterScope(1));
        }
        events.push(SourceProcessingEvent::Origin {
            origin: index,
            affinity: Affinity::After,
        });
        if index + 1 == inner.end {
            events.push(SourceProcessingEvent::ExitScope(1));
        }
    }
    events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: (0..count)
                .map(|index| ParagraphOrigin {
                    source: source.clone(),
                    text_node: None,
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..count - 1)
                .map(|index| SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..count - 1)
                .map(|index| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: outside,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(font_reference()),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(inside),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(font_reference()),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events,
        })
        .unwrap(),
    );
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::Mixed,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    let mut cells = analyzed.combined_cells();
    let cell = cells.next().unwrap();
    assert!(cells.next().is_none());
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        source.text().code_units()[inner.start..inner.end]
            .iter()
            .map(|&unit| u32::from(unit))
            .collect::<Vec<_>>()
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), inner.end - inner.start);
    for (piece, index) in pieces.iter().zip(inner.start..inner.end) {
        assert_eq!(piece.source(), &source);
        assert_eq!(
            (piece.local(), piece.aggregate()),
            (Range::new(index, index + 1), Range::new(index, index + 1))
        );
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert!(piece.is_used());
    }
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent_bytes.clone()).unwrap();
    system.register_fonts(body_bytes.clone()).unwrap();
    let parent = system
        .registered_font_faces()
        .find(|face| face.bytes() == parent_bytes.as_slice())
        .unwrap()
        .to_owned();
    let body = system
        .registered_font_faces()
        .find(|face| face.bytes() == body_bytes.as_slice())
        .unwrap()
        .to_owned();
    assert!(!parent.face().is_same_face(&body.face()));
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    Mixed {
        prepared,
        processed,
        parent,
        body,
    }
}

fn request(prepared: &PreparedText) -> LineSelectionRequest {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    LineSelectionRequest {
        segment: segment.clone(),
        start: segment.start(),
        end: segment.end(),
        participants: prepared.participants().collect(),
        termination: PreparedTermination::Ordinary(segment.end_opportunity()),
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![],
        context: LineContext {
            alignment_insets: LineAlignmentInsets::ZERO,
            formatted_line: FormattedLineContext::default(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: LineBoxDirection::LeftToRight,
            orientation: LineOrientation::Vertical,
            start: LineStart::Paragraph,
            end: LineEnd::Paragraph,
            available_inline: None,
            indent: 0.0,
            alignment: Alignment::Start,
            tabs: TabContext::NoTabs,
            spacing: SpacingPolicy::None,
        },
    }
}
fn allocations() -> LineAllocations {
    LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: ScopeAllocations::None,
    }
}
fn maximum(mixed: &Mixed) -> ScopeAlignmentAllowance {
    let limits = mixed
        .processed
        .captured_scope(0)
        .unwrap()
        .scope_alignment_limits()
        .unwrap();
    ScopeAlignmentAllowance::new(limits.records(), limits.payload_bytes())
}
fn geometry(mixed: &Mixed, line: &FinalizedLine) {
    // Source outer units B,B,Cell,B carry10+10+10+10; only boundary means
    //4,3,3 add10. Inner AA tracking is suppressed: hwid raw5+5 stays E10.
    assert_eq!(line.feedback().advance, 50.0);
    let mut cells = line.cell_views();
    let cell = cells.next().unwrap();
    assert!(cells.next().is_none());
    assert_eq!(
        cell.source_cell().scope(),
        mixed.processed.captured_scope(1).unwrap()
    );
    assert_eq!(cell.square().origin.x, 27.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    let mut outside = Vec::new();
    let mut inside = Vec::new();
    for run in line.run_views() {
        if run.cell().is_some() {
            assert!(run.face().is_same_face(&mixed.body.face()));
            inside.extend(
                run.glyphs()
                    .map(|g| (g.range(), g.id(), g.pen(), g.advance())),
            );
        } else {
            assert!(run.face().is_same_face(&mixed.parent.face()));
            outside.extend(run.glyphs().map(|g| {
                assert_eq!((g.id(), g.advance()), (36, 10.0));
                (g.range(), run.origin().x + g.pen())
            }));
        }
    }
    assert_eq!(
        inside,
        [
            (Range::new(2, 3), 36, 0.0, 5.0),
            (Range::new(3, 4), 36, 5.0, 5.0)
        ]
    );
    assert_eq!(
        outside,
        [
            (Range::new(0, 1), 0.0),
            (Range::new(1, 2), 14.0),
            (Range::new(4, 5), 40.0)
        ]
    );
}

struct Expected {
    name: &'static str,
    identity: census::AllocationIdentity,
    records: usize,
    bytes: usize,
}
fn header<T>(name: &'static str, value: &T) -> Expected {
    Expected {
        name,
        identity: census::AllocationWitness::object(value, 0).identity(),
        records: 0,
        bytes: size_of_val(value),
    }
}
fn backing<T>(name: &'static str, value: &Vec<T>) -> Expected {
    Expected {
        name,
        identity: census::AllocationWitness::buffer(value, 0)
            .unwrap()
            .identity(),
        records: value.len(),
        bytes: value.capacity() * size_of::<T>(),
    }
}
fn covered(expected: &[Expected], actual: &[census::AllocationWitness]) {
    for expected in expected {
        let matches = actual
            .iter()
            .filter(|row| row.identity() == expected.identity)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "{}", expected.name);
        assert_eq!(matches[0].records(), expected.records, "{}", expected.name);
        assert_eq!(
            matches[0].payload_bytes(),
            expected.bytes,
            "{}",
            expected.name
        );
    }
}
fn shaped_allocations(shaped: &ShapedData) -> Vec<census::AllocationWitness> {
    let mut rows = Vec::new();
    character::visit_shaped_allocations(shaped, &mut |row| {
        rows.push(row);
        Ok(())
    })
    .unwrap();
    rows
}
fn payload_limit(error: Error) {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    match error.detail() {
        Some(ErrorDetail::ScopeAlignment {
            reason:
                ScopeAlignmentError::Limit {
                    kind: ScopeAlignmentLimitKind::PayloadBytes,
                    required,
                    limit,
                },
        }) => assert!(required > limit),
        detail => panic!("expected actual payload admission refusal, got {detail:?}"),
    }
}

#[test]
fn mixed_tracking_retains_whole_gap_arena_and_named_graph_plan_subset() {
    let mixed = mixed();
    let selected = mixed.prepared.select(request(&mixed.prepared)).unwrap();
    // Preparation executes original outer tracking and retains its real gaps.
    // The selected basis is an untracked finalization seed, whose Graph is
    // applied only after finalization clones it. Its retention closes over this
    // actual preparation owner rather than copying those already-executed gaps.
    let shaped = &mixed.prepared.inner.shaped;
    let gaps = &shaped.character_gaps;
    assert_eq!(gaps.len(), 3);
    assert_eq!(
        gaps.iter().map(|row| row.amount).collect::<Vec<_>>(),
        [4.0, 3.0, 3.0]
    );
    assert_eq!(
        gaps.iter()
            .filter(|row| matches!(row.owner, character::Owner::Cluster(_)))
            .count(),
        2
    );
    assert_eq!(
        gaps.iter()
            .filter(|row| matches!(row.owner, character::Owner::Cell(_)))
            .count(),
        1
    );
    let gap_expected = [
        header("whole gap header", gaps),
        backing("whole mixed gap backing", gaps),
    ];
    covered(&gap_expected, &shaped_allocations(shaped));
    let graph = &selected.inner.basis.characters;
    let selected_clone = selected.clone();
    assert!(std::ptr::eq(graph, &selected_clone.inner.basis.characters));
    assert!(!graph.units.is_empty());
    assert!(!graph.boundaries.is_empty());
    let mut graph_actual = Vec::new();
    graph
        .visit_allocations(&mut |row| {
            graph_actual.push(row);
            Ok(())
        })
        .unwrap();
    let graph_expected = [
        header("retained graph", graph),
        backing("graph unit backing", &graph.units),
        backing("graph boundary backing", &graph.boundaries),
    ];
    covered(&graph_expected, &graph_actual);
    let requirements = selected
        .inner
        .basis
        .shaped
        .character_requirements
        .as_ref()
        .unwrap();
    assert!(!requirements.coverage.is_empty());
    let plan_expected = [
        header("retained plan", requirements.as_ref()),
        backing("plan coverage backing", &requirements.coverage),
    ];
    let mut plan_actual = Vec::new();
    requirements
        .visit_allocations(&mut |row| {
            plan_actual.push(row);
            Ok(())
        })
        .unwrap();
    covered(&plan_expected, &plan_actual);
    // This is a guaranteed retained MINIMUM, not the visitor's complete total.
    // Other arenas/resources/private graph buffers remain additional custody.
    let minimum: usize = gap_expected
        .iter()
        .chain(&graph_expected)
        .chain(&plan_expected)
        .map(|row| row.bytes)
        .sum();
    assert!(
        selected
            .scope_alignment_retention()
            .unwrap()
            .payload_bytes()
            >= minimum
    );
    let line = selected.finalize(allocations()).unwrap();
    geometry(&mixed, &line);
}

#[test]
fn actual_deep_clone_restore_retains_gap_backing_and_shared_plan_owner() {
    let mixed = mixed();
    // Clone the real executed preparation arena, not the untracked selected
    // seed. Its actual retained owner supplies the operation's budget seed.
    let source = &mixed.prepared.inner.shaped;
    let input = mixed.prepared.scope_alignment_retention().unwrap();
    let limits = mixed
        .processed
        .captured_scope(0)
        .unwrap()
        .scope_alignment_limits()
        .unwrap();
    let mut ledger =
        Some(ScopeAlignmentBudget::new(limits, input.records(), input.payload_bytes()).unwrap());
    let mut first = accounting::clone_shaped(&mut ledger, source).unwrap();
    let mut second = accounting::clone_shaped(&mut ledger, source).unwrap();
    assert!(Arc::ptr_eq(
        first.character_requirements.as_ref().unwrap(),
        source.character_requirements.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        second.character_requirements.as_ref().unwrap(),
        source.character_requirements.as_ref().unwrap()
    ));
    let source_backing = backing("source gaps", &source.character_gaps);
    let first_backing = backing("first deep copy", &first.character_gaps);
    let second_backing = backing("second deep copy", &second.character_gaps);
    assert!(first_backing.bytes > 0 && second_backing.bytes > 0);
    assert_ne!(source_backing.identity, first_backing.identity);
    assert_ne!(source_backing.identity, second_backing.identity);
    assert_ne!(first_backing.identity, second_backing.identity);
    covered(
        &[backing("first copied backing", &first.character_gaps)],
        &shaped_allocations(&first),
    );
    covered(
        &[backing("second copied backing", &second.character_gaps)],
        &shaped_allocations(&second),
    );
    character::restore(&mut first);
    character::restore(&mut second);
    assert!(first.character_gaps.is_empty() && second.character_gaps.is_empty());
    assert_eq!(
        (first.cells[0].extent, first.cells[0].advance),
        (10.0, 10.0)
    );
    assert_eq!(
        (second.cells[0].extent, second.cells[0].advance),
        (10.0, 10.0)
    );
    assert_eq!(
        first.character_gaps.capacity() * size_of::<character::Gap>(),
        first_backing.bytes
    );
    assert_eq!(
        second.character_gaps.capacity() * size_of::<character::Gap>(),
        second_backing.bytes
    );
    let first_empty = backing("first retained empty backing", &first.character_gaps);
    let second_empty = backing("second retained empty backing", &second.character_gaps);
    assert_eq!(first_empty.identity, first_backing.identity);
    assert_eq!(second_empty.identity, second_backing.identity);
    assert_eq!((first_empty.records, second_empty.records), (0, 0));
    covered(&[first_empty], &shaped_allocations(&first));
    covered(&[second_empty], &shaped_allocations(&second));
    // Neither a remaining Cell gap nor the width-arena flag is consulted to
    // prove this retained capacity. No fake product is built around scratch.
    accounting::drop_shaped(&mut ledger, first).unwrap();
    accounting::drop_shaped(&mut ledger, second).unwrap();
    assert_eq!(
        ledger.unwrap().usage().payload_bytes(),
        input.payload_bytes()
    );
    let selected = mixed.prepared.select(request(&mixed.prepared)).unwrap();
    geometry(&mixed, &selected.finalize(allocations()).unwrap());
}

#[test]
fn real_tracking_replay_dedups_ancestors_clones_and_refuses_tight_allowances() {
    let mixed = mixed();
    let selected = mixed.prepared.select(request(&mixed.prepared)).unwrap();
    let maximum = maximum(&mixed);
    let line = selected
        .finalize_with_scope_allowance(allocations(), maximum)
        .unwrap();
    geometry(&mixed, &line);
    let retained = line.scope_alignment_retention().unwrap();
    payload_limit(
        line.with_spacing_with_scope_allowance(
            SpacingAllocations::None,
            ScopeAlignmentAllowance::new(maximum.records(), retained.payload_bytes() - 1),
        )
        .unwrap_err(),
    );
    geometry(&mixed, &line);
    let replay = line
        .with_spacing_with_scope_allowance(SpacingAllocations::None, maximum)
        .unwrap();
    geometry(&mixed, &replay);
    let input = ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&replay));
    let ancestors = [
        ScopeAlignmentProduct::Prepared(&mixed.prepared),
        ScopeAlignmentProduct::Selected(&selected),
    ];
    let shared =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&ancestors), maximum).unwrap();
    assert_eq!(shared.external_retention().records(), 0);
    assert_eq!(shared.external_retention().payload_bytes(), 0);
    let old = [ScopeAlignmentProduct::Finalized(&line)];
    let coexist =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&old), maximum).unwrap();
    assert!(
        coexist.external_retention().records() > 0
            && coexist.external_retention().payload_bytes() > 0
    );
    let clone = line.clone();
    let duplicated = [
        ScopeAlignmentProduct::Finalized(&line),
        ScopeAlignmentProduct::Finalized(&clone),
    ];
    let dedup =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&duplicated), maximum).unwrap();
    assert_eq!(dedup.input_retention(), coexist.input_retention());
    assert_eq!(dedup.external_retention(), coexist.external_retention());
    let union =
        coexist.input_retention().payload_bytes() + coexist.external_retention().payload_bytes();
    payload_limit(
        scope_alignment_census(
            input,
            ScopeAlignmentProducts::Mixed(&old),
            ScopeAlignmentAllowance::new(maximum.records(), union - 1),
        )
        .unwrap_err(),
    );
    let recovered =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&duplicated), maximum).unwrap();
    assert_eq!(recovered.external_retention(), coexist.external_retention());
    geometry(&mixed, &line);
    geometry(&mixed, &replay);
}
