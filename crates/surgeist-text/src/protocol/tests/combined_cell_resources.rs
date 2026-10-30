//! Actual Cell allocation ownership, independent of glyph/geometry oracles.
//! This child reuses the genuine registered-font/captured-source fixture. The
//! expected storage comes from live capacities, row lengths and Rust type sizes;
//! no production visitor is used to calculate the expected retention totals.

use super::{native_metric_font, setup};
use crate::engine::layout::data::{CellPayload, ShapedData};
use crate::engine::scope_alignment::{accounting, census, visit_prepared_allocations};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use crate::*;
use std::mem::{size_of, size_of_val};
use std::sync::Arc;

#[derive(Clone, Copy, Default)]
struct Footprint {
    records: usize,
    bytes: usize,
}

impl Footprint {
    fn backing<T>(&mut self, values: &Vec<T>) {
        self.records += values.len();
        self.bytes += values.capacity() * size_of::<T>();
    }

    fn owned<T>(&mut self, values: &Vec<T>) {
        self.bytes += size_of::<Vec<T>>();
        self.backing(values);
    }
}

// The declared whole-arena adoption boundary, not per-Cell index subranges.
fn shared_arenas(shaped: &ShapedData) -> Footprint {
    let mut result = Footprint::default();
    result.owned(&shaped.runs);
    result.owned(&shaped.clusters);
    result.owned(&shaped.glyphs);
    result.owned(&shaped.provider_groups);
    result.owned(&shaped.items);
    for run in &shaped.runs {
        result.backing(&run.coords);
    }
    result
}

fn request(prepared: &PreparedText) -> LineSelectionRequest {
    let segment = prepared.source_segments().next().unwrap();
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

fn cap(prepared: &PreparedText) -> ScopeAlignmentAllowance {
    let root = prepared
        .analyzed_source()
        .processed_source()
        .captured_scope(0)
        .unwrap();
    let limits = root.scope_alignment_limits().unwrap();
    ScopeAlignmentAllowance::new(limits.records(), limits.payload_bytes())
}

fn limit(error: Error, expected: ScopeAlignmentLimitKind) {
    match error.detail() {
        Some(ErrorDetail::ScopeAlignment {
            reason:
                ScopeAlignmentError::Limit {
                    kind,
                    required,
                    limit,
                },
        }) => {
            assert_eq!(*kind, expected);
            assert!(*required > *limit);
        }
        other => panic!("expected actual resource refusal, got {other:?}"),
    }
}

fn prepare(text: &str, whitespace: WhiteSpace) -> PreparedText {
    let (mut system, _, analyzed, _) = setup(text, whitespace);
    system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap()
}

// Outside X rows and the genuine Inline A cell share selected arenas. Both
// registered families have the retained real wght axis, requested at max1;
// the actual normalized coordinate buffers therefore cannot be hidden by an
// all-empty/static-coordinate fixture.
fn mixed_variable() -> PreparedText {
    let variable = crate::test_fonts::variable_ahem();
    assert_eq!(
        &crate::test_fonts::table(&variable, *b"fvar")[16..20],
        b"wght"
    );
    let original = super::composition_font();
    let body = crate::test_fonts::replace_tables(
        &original,
        [*b"fvar", *b"gvar"]
            .into_iter()
            .map(|tag| (tag, crate::test_fonts::table(&variable, tag).to_vec()))
            .collect(),
    );
    // The vertical-table authoring helper qualifies static outline bounds.
    // Build those real tables before adding the genuine variable instance.
    let static_parent = native_metric_font(include_bytes!(
        "../../../tests/fixtures/fonts/ahem/Ahem.ttf"
    ));
    let parent = crate::test_fonts::replace_tables(
        &static_parent,
        [*b"fvar", *b"gvar"]
            .into_iter()
            .map(|tag| (tag, crate::test_fonts::table(&variable, tag).to_vec()))
            .collect(),
    );
    for tag in [*b"vhea", *b"vmtx", *b"BASE", *b"glyf", *b"loca", *b"maxp"] {
        assert_eq!(
            crate::test_fonts::table(&parent, tag),
            crate::test_fonts::table(&static_parent, tag)
        );
    }
    for font in [&parent, &body] {
        for tag in [*b"fvar", *b"gvar"] {
            assert_eq!(
                crate::test_fonts::table(font, tag),
                crate::test_fonts::table(&variable, tag)
            );
        }
    }
    let parent_style = Style {
        font: Font::new().family("Ahem").variation("\"wght\" 1"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        text_combine_upright: TextCombineUpright::None,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let child_style = Style {
        font: Font::new()
            .family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY)
            .variation("\"wght\" 1"),
        text_combine_upright: TextCombineUpright::All,
        ..parent_style.clone()
    };
    let mut source = Source::identified("XAX", Id::from_u64(784_905), 1);
    source.span(Range::new(1, 2), child_style.clone());
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: (0..3)
                .map(|index| ParagraphOrigin {
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                    text_node: None,
                })
                .collect(),
            connections: vec![
                SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: ParagraphConnectionKind::Continuous,
                },
                SourceProcessingConnection {
                    before_origin: 1,
                    after_origin: 2,
                    kind: ParagraphConnectionKind::Continuous,
                },
            ],
            controls: vec![],
            // Each actual local Continuous connection owns wrapping facts.
            // Both incident styles and their genuine common ancestor Block0
            // use Word; retain the real connection identities/event positions.
            boundary_policies: vec![
                ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                },
                ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(1),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                },
            ],
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
                    base_style: parent_style,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(super::font_reference()),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(child_style),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(super::font_reference()),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(0),
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::Connection(1),
                SourceProcessingEvent::Origin {
                    origin: 2,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let raw = processed.paragraphs().next().unwrap();
    let domain = AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::Mixed).unwrap();
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    assert_eq!(analyzed.combined_cells().len(), 1);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent).unwrap();
    system.register_fonts(body).unwrap();
    system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap()
}

#[test]
fn source_empty_cell_retains_its_actual_outer_item_allocation() {
    let prepared = prepare(" ", WhiteSpace::Collapse);
    let data = &prepared.inner;
    assert!(matches!(
        data.shaped.cells[0].payload,
        CellPayload::NoProviderInput
    ));
    assert!(data.shaped.glyphs.is_empty());
    assert_eq!(data.shaped.items.len(), 1);
    assert!(data.shaped.items.capacity() > 0);
    // The opaque identity is issued from the actual arena; expected bytes and
    // records come directly from its physical capacity/type and live row count.
    let item =
        census::AllocationWitness::buffer(&data.shaped.items, data.shaped.items.len()).unwrap();
    let mut seen = 0;
    visit_prepared_allocations(
        &data.scope_alignment,
        &data.analyzed,
        &data.shaped,
        &data.tables,
        &mut |allocation| {
            if allocation.identity() == item.identity() {
                seen += 1;
                assert_eq!(allocation.records(), data.shaped.items.len());
                assert_eq!(
                    allocation.payload_bytes(),
                    data.shaped.items.capacity()
                        * size_of::<crate::engine::layout::data::ShapedItem>()
                );
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(seen, 1);
}

#[test]
fn real_shared_arenas_are_adopted_with_exact_rows_and_capacity_before_growth() {
    let prepared = mixed_variable();
    let selected = prepared.select(request(&prepared)).unwrap();
    let shaped = &selected.inner.basis.shaped;
    assert!(shaped.runs.len() > 1);
    assert!(shaped.runs.iter().all(|run| run.coords == [16384]));
    let expected = shared_arenas(shaped);
    assert!(expected.records > 0 && expected.bytes > 0);
    let make = |records, bytes| {
        Some(
            ScopeAlignmentBudget::new(
                ScopeAlignmentLimits::new(records, 0, 1_000_000, bytes),
                0,
                0,
            )
            .unwrap(),
        )
    };
    let mut exact = make(expected.records, expected.bytes);
    accounting::adopt_cell_arenas(&mut exact, shaped).unwrap();
    let usage = exact.unwrap().usage();
    assert_eq!(usage.records(), expected.records);
    assert_eq!(usage.payload_bytes(), expected.bytes);
    assert_eq!(usage.peak_payload_bytes(), expected.bytes);
    let mut small_rows = make(expected.records - 1, expected.bytes);
    limit(
        accounting::adopt_cell_arenas(&mut small_rows, shaped).unwrap_err(),
        ScopeAlignmentLimitKind::Records,
    );
    let mut small_bytes = make(expected.records, expected.bytes - 1);
    limit(
        accounting::adopt_cell_arenas(&mut small_bytes, shaped).unwrap_err(),
        ScopeAlignmentLimitKind::PayloadBytes,
    );
}

#[test]
fn selected_retention_contains_whole_mixed_arenas_coordinates_and_unique_resources() {
    let prepared = mixed_variable();
    let selected = prepared.select(request(&prepared)).unwrap();
    let data = &selected.inner;
    let shaped = &data.basis.shaped;
    let scopes = &data.scopes;
    assert!(scopes.proof_frames.is_empty());
    assert!(shaped.empty_run.is_none());
    let mut own = shared_arenas(shaped);
    own.bytes += size_of_val(scopes);
    own.backing(&scopes.rows);
    own.backing(&scopes.contributions);
    own.backing(&scopes.cluster_owners);
    own.backing(&scopes.glyph_owners);
    own.backing(&scopes.unit_owners);
    own.backing(&scopes.strut_owners);
    own.backing(&scopes.shared);
    own.backing(&scopes.uncertain);
    own.backing(&scopes.proof_frames);
    own.owned(&shaped.scope_glyph_custody);
    own.owned(&shaped.scope_relations);
    own.owned(&shaped.cells);
    for cell in &shaped.cells {
        if let CellPayload::Shaped { positions, .. } = &cell.payload {
            own.backing(positions);
        }
    }
    let identity = |font: &crate::FontData| (font.bytes().as_ptr() as usize, font.bytes().len());
    let mut pins = prepared
        .inner
        .scope_alignment
        .metrics
        .iter()
        .map(|row| identity(&row.font))
        .collect::<Vec<_>>();
    for run in &shaped.runs {
        let key = identity(&run.font);
        if !pins.contains(&key) {
            own.bytes += run.font.bytes().len();
            pins.push(key);
        }
    }
    let input = prepared.scope_alignment_retention().unwrap();
    let retained = selected.scope_alignment_retention().unwrap();
    assert_eq!(retained.records(), input.records() + own.records);
    assert_eq!(retained.payload_bytes(), input.payload_bytes() + own.bytes);
    let maximum = cap(&prepared);
    limit(
        prepared
            .select_with_scope_allowance(
                request(&prepared),
                ScopeAlignmentAllowance::new(input.records() - 1, maximum.payload_bytes()),
            )
            .unwrap_err(),
        ScopeAlignmentLimitKind::Records,
    );
    let sufficient = prepared
        .select_with_scope_allowance(request(&prepared), maximum)
        .unwrap();
    assert_eq!(
        sufficient.scope_alignment_retention(),
        selected.scope_alignment_retention()
    );
}

#[test]
fn finalized_replay_census_counts_shared_ancestors_once_and_real_coexistence() {
    let prepared = prepare("A", WhiteSpace::Preserve);
    let selected = prepared.select(request(&prepared)).unwrap();
    let maximum = cap(&prepared);
    let line = selected
        .finalize_with_scope_allowance(allocations(), maximum)
        .unwrap();
    let old = line.scope_alignment_retention().unwrap();
    limit(
        line.with_spacing_with_scope_allowance(
            SpacingAllocations::None,
            ScopeAlignmentAllowance::new(old.records() - 1, maximum.payload_bytes()),
        )
        .unwrap_err(),
        ScopeAlignmentLimitKind::Records,
    );
    let replay = line
        .with_spacing_with_scope_allowance(SpacingAllocations::None, maximum)
        .unwrap();
    let input = ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&replay));
    let ancestors = [
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&selected),
    ];
    let shared =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&ancestors), maximum).unwrap();
    assert_eq!(shared.external_retention().records(), 0);
    assert_eq!(shared.external_retention().payload_bytes(), 0);
    let old_only = [ScopeAlignmentProduct::Finalized(&line)];
    let both =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&old_only), maximum).unwrap();
    assert!(both.external_retention().records() > 0);
    assert!(both.external_retention().payload_bytes() > 0);
    let clone = line.clone();
    let duplicates = [
        ScopeAlignmentProduct::Finalized(&line),
        ScopeAlignmentProduct::Finalized(&clone),
    ];
    let dedup =
        scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&duplicates), maximum).unwrap();
    assert_eq!(dedup.input_retention(), both.input_retention());
    assert_eq!(dedup.external_retention(), both.external_retention());
    let total = both.input_retention().payload_bytes() + both.external_retention().payload_bytes();
    assert_eq!(both.usage().payload_bytes(), total);
    limit(
        scope_alignment_census(
            input,
            ScopeAlignmentProducts::Mixed(&old_only),
            ScopeAlignmentAllowance::new(maximum.records(), total - 1),
        )
        .unwrap_err(),
        ScopeAlignmentLimitKind::PayloadBytes,
    );
}
