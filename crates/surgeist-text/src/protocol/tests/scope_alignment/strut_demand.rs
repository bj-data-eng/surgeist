//! Actual allocated numeric rows demand frames by ancestry, not by glyph presence.
//! Empty unclassified boxes retain source custody without acquiring font facts.
use super::capture::{reference, style};
use crate::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");

#[derive(Clone, Copy)]
enum Relation {
    Sibling,
    Child,
    EmptyParent,
}
struct Fixture {
    system: System,
    prepared: PreparedText,
    active: CapturedScope,
    empty: CapturedScope,
}

fn font_premises() {
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    let hhea = font.hhea().unwrap();
    assert_eq!(
        (hhea.ascender().to_i16(), hhea.descender().to_i16()),
        (800, -200)
    );
    assert_eq!(font.charmap().map('X'), Some(GlyphId::new(58)));
    let word = |at| u16::from_be_bytes(AHEM[at..at + 2].try_into().unwrap());
    let record = (0..usize::from(word(4)))
        .map(|i| 12 + 16 * i)
        .find(|&at| AHEM[at..at + 4] == *b"hmtx")
        .unwrap();
    let offset = usize::try_from(u32::from_be_bytes(
        AHEM[record + 8..record + 12].try_into().unwrap(),
    ))
    .unwrap();
    assert_eq!(word(offset + 58 * 4), 1000);
}

fn fixture(relation: Relation) -> Fixture {
    use SourceProcessingEvent as E;
    let mut input = super::prepared::two_leaves();
    // Inline.Style supplies a font context; the body is independently authored.
    input.source.span(Range::new(0, 2), style(10.0, 10.0));
    for origin in &mut input.origins {
        origin.source = input.source.clone();
    }
    input.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
    });
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Connection(0),
    ];
    match relation {
        Relation::Sibling => input.events.extend([
            E::EnterScope(1),
            E::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            E::ExitScope(1),
            E::EnterScope(2),
            E::ExitScope(2),
        ]),
        Relation::Child => input.events.extend([
            E::EnterScope(1),
            E::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            E::EnterScope(2),
            E::ExitScope(2),
            E::ExitScope(1),
        ]),
        Relation::EmptyParent => input.events.extend([
            E::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            E::EnterScope(1),
            E::EnterScope(2),
            E::ExitScope(2),
            E::ExitScope(1),
        ]),
    }
    input.events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let captured = Arc::new(process_contextual_source(input).unwrap());
    let active = captured.captured_scope(1).unwrap();
    let empty = captured.captured_scope(2).unwrap();
    assert_eq!(
        active.alignment_boundary(),
        Some(ScopeAlignmentBoundary::NonInitial)
    );
    assert_eq!(active.font_reference(), Some(reference()));
    assert_eq!(empty.alignment_boundary(), None);
    assert_eq!(empty.font_reference(), None);
    assert!(empty.style().is_none());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.control_views().count(), 0);
    assert_eq!(prepared.atomic_views().count(), 0);
    assert_eq!(prepared.participants().count(), 2);
    assert_eq!(
        prepared
            .origin_views()
            .map(|origin| (
                origin.source().id(),
                origin.source().revision(),
                origin.local_range(),
                origin.aggregate_range(),
                origin.kind(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                Some(Id::from_u64(1578020)),
                31,
                Range::new(0, 1),
                Range::new(0, 1),
                ParagraphOriginKind::Authored
            ),
            (
                Some(Id::from_u64(1578020)),
                31,
                Range::new(1, 2),
                Range::new(1, 2),
                ParagraphOriginKind::Authored
            ),
        ]
    );
    assert!(
        prepared
            .scopes()
            .all(|scope| scope.captured_scope() != empty)
    );
    assert_eq!(
        reason(prepared.scope_metrics(&empty).unwrap_err()),
        ScopeAlignmentError::MissingMetrics {
            scope: empty.capture_ordinal()
        }
    );
    Fixture {
        system,
        prepared,
        active,
        empty,
    }
}

fn reason(error: Error) -> ScopeAlignmentError {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    let Some(ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("expected exact scope refusal, got {error:?}")
    };
    *reason
}

fn request(prepared: &PreparedText, row: Option<LineBoxStrut>) -> LineSelectionRequest {
    let mut request = super::super::request(prepared);
    request.context.available_inline = None;
    request.context.spacing = SpacingPolicy::None;
    request.context.alignment = Alignment::Start;
    request.line_box_struts = row.into_iter().collect();
    request
}

fn shifted(selected: &SelectedLine, actual: &CapturedScope) -> FinalizedLine {
    assert_eq!(selected.scopes().len(), 1);
    let scope = selected.scopes().next().unwrap();
    assert_eq!(scope.captured_scope(), *actual);
    selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
                scope,
                displacement: BaselineShift::try_new(3.0).unwrap(),
            }]),
        })
        .unwrap()
}

fn place(prepared: &PreparedText, line: FinalizedLine) -> Layout {
    super::super::place(prepared.clone(), line, 0.0, 100.0).unwrap()
}

fn glyphs_and_source(layout: &Layout) {
    assert_eq!(layout.source().text().code_units(), [88, 88]);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 2)]
    );
    assert_eq!(
        layout
            .run_views()
            .map(|run| {
                assert_eq!(run.face().bytes(), AHEM);
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                (
                    run.range(),
                    run.origin(),
                    run.glyphs()
                        .map(|glyph| (glyph.id(), glyph.x(), glyph.y(), glyph.advance()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                Range::new(0, 1),
                Point::new(0.0, 100.0),
                vec![(58, 0.0, 0.0, 10.0)]
            ),
            (
                Range::new(1, 2),
                Point::new(10.0, 97.0),
                vec![(58, 0.0, 0.0, 10.0)]
            ),
        ]
    );
}

fn qualifies(relation: Relation, metrics: LineStrut, expected: (f64, f64), displacement: f64) {
    font_premises();
    let fixture = fixture(relation);
    let prepared = &fixture.prepared;
    let parent = prepared.scope_metrics(&fixture.active).unwrap();
    assert_eq!(parent.captured_scope(), fixture.active);
    assert_eq!(
        (parent.over(), parent.under(), parent.line_height()),
        (8.0, 2.0, 10.0)
    );
    assert_eq!(parent.effective_instance().unwrap().face().bytes(), AHEM);
    match relation {
        Relation::Sibling => assert_eq!(fixture.empty.parent(), Some(fixture.active.domain())),
        Relation::Child => assert_eq!(fixture.empty.parent(), Some(fixture.active.clone())),
        Relation::EmptyParent => unreachable!(),
    }
    let old_selected = prepared.select(request(prepared, None)).unwrap();
    let old = shifted(&old_selected, &fixture.active);
    let old_layout = place(prepared, old.clone());
    assert_eq!(
        (old.feedback().baseline, old.feedback().line_height),
        (11.0, 13.0)
    );
    assert_eq!(old_layout.bounds(), Rect::new(0.0, 89.0, 20.0, 13.0));
    glyphs_and_source(&old_layout);
    let row = LineBoxStrut {
        scope: Some(fixture.empty.clone()),
        metrics,
        alignment: LineBoxAlignment::Baseline,
    };
    let selected = prepared
        .select(request(prepared, Some(row.clone())))
        .unwrap();
    assert_eq!(
        selected.request().line_box_struts.as_slice(),
        std::slice::from_ref(&row)
    );
    let line = shifted(&selected, &fixture.active);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        expected
    );
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        line.line_box_strut_scope_displacement(0).unwrap().get(),
        displacement
    );
    assert_eq!(line.scope_allocations().len(), 1);
    assert_eq!(line.scope_allocations()[0].scope, fixture.active);
    assert_eq!(line.scope_allocations()[0].displacement.get(), 3.0);
    assert_eq!(
        line.scope_allocations()[0].cumulative_displacement.get(),
        3.0
    );
    assert_eq!(selected.scope_alignment_usage().unwrap().font_queries(), 0);
    assert_eq!(line.scope_alignment_usage().unwrap().font_queries(), 0);
    let stats = fixture.system.stats();
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(replay.scope_allocations(), line.scope_allocations());
    assert_eq!(replay.selected_line(), selected);
    assert_eq!(replay.selected_line().request().line_box_struts, [row]);
    assert_eq!(
        replay.line_box_strut_scope_displacement(0).unwrap().get(),
        displacement
    );
    assert_eq!(
        (replay.feedback().baseline, replay.feedback().line_height),
        expected
    );
    let layout = place(prepared, line.clone());
    let repeated = place(prepared, replay);
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(0.0, 100.0 - expected.0, 20.0, expected.1)
    );
    assert_eq!(repeated.lines()[0].bounds(), layout.lines()[0].bounds());
    glyphs_and_source(&layout);
    glyphs_and_source(&repeated);
    assert_eq!(
        reason(prepared.scope_metrics(&fixture.empty).unwrap_err()),
        ScopeAlignmentError::MissingMetrics {
            scope: fixture.empty.capture_ordinal()
        }
    );
    assert_eq!(fixture.empty.alignment_boundary(), None);
    assert_eq!(fixture.empty.font_reference(), None);
    assert_eq!(fixture.system.stats(), stats);
    // The already published old line and Layout stay live through the new
    // selection, allocation, placement and immutable replay.
    assert_eq!(
        (old.feedback().baseline, old.feedback().line_height),
        (11.0, 13.0)
    );
    assert!(old.selected_line().request().line_box_struts.is_empty());
    assert_eq!(old_layout.bounds(), Rect::new(0.0, 89.0, 20.0, 13.0));
    glyphs_and_source(&old_layout);
}

#[test]
fn empty_unclassified_sibling_strut_stays_common_beside_a_shifted_glyph_target() {
    // Common16/4 dominates the shifted glyph11/-1 and ordinary glyph8/2.
    qualifies(
        Relation::Sibling,
        LineStrut {
            baseline: 16.0,
            line_extent: 20.0,
        },
        (16.0, 20.0),
        0.0,
    );
}

#[test]
fn numeric_strut_in_an_empty_unclassified_child_inherits_its_prepared_parent_frame() {
    // The child's6/8 row becomes9/5 under parent+3. Together with shifted11/-1
    // and common8/2 glyphs, the actual union is11/5, hence height16.
    qualifies(
        Relation::Child,
        LineStrut {
            baseline: 6.0,
            line_extent: 14.0,
        },
        (11.0, 16.0),
        3.0,
    );
}

#[test]
fn numeric_child_strut_refuses_when_its_classified_parent_has_no_prepared_receipt() {
    font_premises();
    let fixture = fixture(Relation::EmptyParent);
    let prepared = &fixture.prepared;
    assert_eq!(fixture.empty.parent(), Some(fixture.active.clone()));
    assert_eq!(
        reason(prepared.scope_metrics(&fixture.active).unwrap_err()),
        ScopeAlignmentError::MissingMetrics {
            scope: fixture.active.capture_ordinal()
        }
    );
    let selected = prepared.select(request(prepared, None)).unwrap();
    assert_eq!(selected.scopes().len(), 0);
    let old = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    let old_layout = place(prepared, old.clone());
    assert_eq!(
        (old.feedback().baseline, old.feedback().line_height),
        (8.0, 10.0)
    );
    assert_eq!(old_layout.bounds(), Rect::new(0.0, 92.0, 20.0, 10.0));
    let usage = prepared.scope_alignment_usage();
    assert_eq!(usage.unwrap().font_queries(), 0);
    let stats = fixture.system.stats();
    let row = LineBoxStrut {
        scope: Some(fixture.empty.clone()),
        metrics: LineStrut {
            baseline: 6.0,
            line_extent: 14.0,
        },
        alignment: LineBoxAlignment::Baseline,
    };
    assert_eq!(
        reason(prepared.select(request(prepared, Some(row))).unwrap_err()),
        ScopeAlignmentError::MissingMetrics {
            scope: fixture.active.capture_ordinal()
        }
    );
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(prepared.scope_alignment_usage(), usage);
    assert_eq!(
        (old.feedback().baseline, old.feedback().line_height),
        (8.0, 10.0)
    );
    assert!(old.selected_line().request().line_box_struts.is_empty());
    assert!(old.scope_allocations().is_empty());
    assert_eq!(old_layout.source().text().code_units(), [88, 88]);
    assert_eq!(old_layout.bounds(), Rect::new(0.0, 92.0, 20.0, 10.0));
}
