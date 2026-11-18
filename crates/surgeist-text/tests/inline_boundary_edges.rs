#![forbid(unsafe_code)]

//! Endpoint custody and independently authored native edge geometry.
//! Contextual scope/control capture and registered Ahem exercise the
//! public prepare/select/finalize path. The original custody fixtures supply
//! affirmative zero edges; geometry fixtures supply their declared used widths.

use std::sync::Arc;
use surgeist_text as t;

#[path = "inline_boundary_edges/font.rs"]
mod font;

#[path = "inline_boundary_edges/geometry.rs"]
mod geometry;

#[path = "inline_boundary_edges/scope_incidence.rs"]
mod scope_incidence;

use t::{capture::SourceProcessingEvent as E, paragraph::ParagraphControlKind as K};

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

fn style(mode: t::style::WhiteSpace) -> t::style::Style {
    t::style::Style {
        font: t::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::style::LineHeight::Absolute(10.0),
        white_space: mode,
        wrap: t::style::Wrap::Word,
        ..t::style::Style::default()
    }
}

fn control(control: usize) -> E {
    E::Control {
        control,
        affinity: t::layout::Affinity::After,
    }
}

fn origin(origin: usize) -> E {
    E::Origin {
        origin,
        affinity: t::layout::Affinity::After,
    }
}

fn prepare_capture(
    source: t::source::Source,
    block: t::capture::SourceProcessingScope,
    ranges: &[(usize, usize)],
    controls: &[(K, usize)],
    pairs: &[(usize, usize)],
    body: Vec<E>,
    fixture: &[u8],
) -> t::prepared::PreparedText {
    let (base, direction) = match &block {
        t::capture::SourceProcessingScope::Block {
            base_style,
            used_direction,
            ..
        } => (base_style.clone(), *used_direction),
        _ => panic!("fixture supplies a real containing block"),
    };
    let mut scopes = vec![block];
    scopes.extend(
        pairs
            .iter()
            .map(|&(start, end)| t::capture::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: t::capture::WhiteSpaceTrim::default(),
                style: Some(base.clone()),
                boundaries: Some(t::capture::SourceProcessingScopeBoundaries { start, end }),
            }),
    );
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    events.extend(body);
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let processed = Arc::new(
        t::capture::process_contextual_source(t::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::capture::SourceProcessingEnvelope::Complete,
            origins: ranges
                .iter()
                .map(|&(start, end)| t::paragraph::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: t::source::Range::new(start, end),
                    aggregate: t::source::Range::new(start, end),
                    kind: t::paragraph::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (1..ranges.len())
                .map(|after_origin| t::capture::SourceProcessingConnection {
                    before_origin: after_origin - 1,
                    after_origin,
                    kind: t::paragraph::ParagraphConnectionKind::Continuous,
                })
                .collect(),
            controls: controls
                .iter()
                .map(|(kind, at)| t::paragraph::ParagraphControl {
                    kind: kind.clone(),
                    at: *at,
                })
                .collect(),
            boundary_policies: (0..ranges.len().saturating_sub(1))
                .map(|connection| t::paragraph::ParagraphBoundaryPolicy {
                    definition: t::paragraph::ParagraphBoundaryDefinition::Connection(connection),
                    preceding_wrap: t::style::Wrap::Word,
                    following_wrap: t::style::Wrap::Word,
                    ancestor_wrap: t::style::Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            scopes,
            paragraphs: vec![t::capture::SourceProcessingParagraph {
                base_direction: direction,
                paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    assert_eq!(processed.source().text(), source.text());
    let paragraph = processed.paragraphs().next().unwrap();
    let analyzed = t::analyzed::AnalyzedSource::from_processed_paragraph(
        paragraph,
        t::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    // Every relation here has the same captured enclosing block's
    // computed Word wrapping policy; no permission is inferred from width.
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
        .register_fonts(fixture.to_vec(), crate::common::FONT_LIMITS)
        .unwrap();
    let prepared = system
        .prepare(&analyzed, t::prepared::PreparationOptions::default())
        .unwrap();
    let handles = prepared
        .control_views()
        .map(|view| {
            assert!(view.capture_position().is_some());
            let handle = view.handle();
            assert!(handle.captured_control().is_some());
            handle
        })
        .collect::<Vec<_>>();
    assert_eq!(handles.len(), controls.len());
    prepared
}

fn capture(
    value: &str,
    mode: t::style::WhiteSpace,
    ranges: &[(usize, usize)],
    controls: &[(K, usize)],
    pairs: &[(usize, usize)],
    body: Vec<E>,
) -> t::prepared::PreparedText {
    let prepared = prepare_capture(
        t::source::Source::identified(
            t::source::Text::from(value),
            t::source::Id::from_u64(64072),
            1,
        ),
        t::capture::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: t::capture::SourceProcessingTopology::Complete,
            trim: t::capture::WhiteSpaceTrim::default(),
            base_style: style(mode),
            used_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
        },
        ranges,
        controls,
        pairs,
        body,
        AHEM,
    );
    // Qualify the real registered resource through complete final shaping
    // before testing an attachment cut. Both X and Y have Ahem ink.
    let mut ink_ranges = Vec::new();
    let mut complete_controls = Vec::new();
    let mut start = prepared.boundary_views().next().unwrap().handle();
    for end in prepared
        .opportunity_views()
        .filter(|view| {
            matches!(
                view.kind(),
                t::prepared::PreparedBreakKind::Forced
                    | t::prepared::PreparedBreakKind::ParagraphEnd
            )
        })
        .map(|view| view.handle())
    {
        let next = end.view().boundary();
        let full = finish(&prepared, start, end);
        complete_controls.extend(control_indices(
            &prepared,
            full.control_slots().map(|slot| slot.control()),
        ));
        for run in full.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            let instance = run.effective_instance().unwrap();
            for glyph in run.glyphs() {
                let outline = instance
                    .glyph_outline(glyph.id(), run.face().catalogue_identity())
                    .unwrap();
                if outline
                    .bounds()
                    .is_some_and(|bounds| bounds.size.width > 0.0 && bounds.size.height > 0.0)
                {
                    assert_eq!(glyph.advance(), 10.0);
                    ink_ranges.push(glyph.range());
                }
            }
        }
        start = next;
    }
    assert!(ink_ranges.contains(&t::source::Range::new(0, 1)));
    assert!(ink_ranges.contains(&t::source::Range::new(
        value.encode_utf16().count() - 1,
        value.encode_utf16().count()
    )));
    assert_eq!(complete_controls, (0..controls.len()).collect::<Vec<_>>());
    prepared
}

fn paragraph_end(prepared: &t::prepared::PreparedText) -> t::prepared::PreparedOpportunity {
    prepared
        .opportunity_views()
        .find(|view| view.kind() == t::prepared::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}

fn finish(
    prepared: &t::prepared::PreparedText,
    start: t::prepared::PreparedBoundary,
    end: t::prepared::PreparedOpportunity,
) -> t::finalized::FinalizedLine {
    let boundary = end.view().boundary();
    let from = start.view().participant_offset();
    let to = boundary.view().participant_offset();
    let participants = prepared
        .participants()
        .skip(from)
        .take(to - from)
        .collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            t::prepared::PreparedParticipant::Control(occurrence) => {
                Some(t::allocation::ControlAllocation {
                    occurrence: occurrence.clone(),
                    strut: None,
                    // These original custody fixtures author neutral endpoints;
                    // their geometry is affirmatively zero, not unavailable.
                    boundary: matches!(
                        occurrence.view().kind(),
                        K::InlineBoundaryStart | K::InlineBoundaryEnd
                    )
                    .then_some(t::allocation::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                })
            }
            _ => None,
        })
        .collect();
    let mut request = t::selection::LineSelectionRequest {
        inline_boxes: None,
        segment: prepared.source_segments().next().unwrap(),
        start,
        end: boundary,
        termination: t::selection::PreparedTermination::Ordinary(end.clone()),
        participants,
        incoming: None,
        alternative: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_box_struts: Vec::new(),
        line_strut: None,
        context: t::selection::LineContext {
            alignment_insets: surgeist_text::allocation::LineAlignmentInsets::ZERO,
            formatted_line: surgeist_text::selection::FormattedLineContext::default(),
            base_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
            line_box_direction: surgeist_text::selection::LineBoxDirection::LeftToRight,
            orientation: t::selection::LineOrientation::Horizontal,
            start: if from == 0 {
                t::selection::LineStart::Paragraph
            } else {
                t::selection::LineStart::Continuation
            },
            end: match end.view().kind() {
                t::prepared::PreparedBreakKind::Forced => t::selection::LineEnd::Forced,
                t::prepared::PreparedBreakKind::ParagraphEnd => t::selection::LineEnd::Paragraph,
                _ => t::selection::LineEnd::Soft,
            },
            available_inline: Some(100.0),
            indent: 0.0,
            alignment: t::options::Alignment::Start,
            tabs: t::tabs::TabContext::NoTabs,
            spacing: t::spacing::SpacingPolicy::Spaces,
        },
    };
    while let Some(occurrence) = prepared.boundary_context_demand(&request).unwrap() {
        request
            .boundary_context
            .push(t::allocation::InlineBoundaryContextAllocation {
                occurrence,
                boundary: t::allocation::InlineBoundaryAllocation::CONTRIBUTION_ZERO,
            });
    }
    prepared
        .select(request)
        .unwrap()
        .finalize(surgeist_text::scope_alignment::LineAllocations {
            spacing: t::spacing::SpacingAllocations::None,
            scopes: surgeist_text::scope_alignment::ScopeAllocations::None,
        })
        .unwrap()
}

fn control_indices(
    prepared: &t::prepared::PreparedText,
    controls: impl Iterator<Item = t::prepared::PreparedControl>,
) -> Vec<usize> {
    let handles = prepared
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    controls
        .map(|control| {
            handles
                .iter()
                .position(|handle| *handle == control)
                .unwrap()
        })
        .collect()
}

fn fragment_controls(
    prepared: &t::prepared::PreparedText,
    line: &t::finalized::FinalizedLine,
) -> Vec<usize> {
    control_indices(
        prepared,
        line.fragment_views().filter_map(|fragment| {
            if let t::finalized::FinalizedFragmentOwner::Control(control) = fragment.owner() {
                assert!(fragment.origin().is_none());
                assert!(fragment.aggregate_range().is_none());
                Some(control)
            } else {
                None
            }
        }),
    )
}

#[derive(Debug, PartialEq)]
struct Cut {
    participant: usize,
    outgoing_controls: Vec<usize>,
    outgoing_fragments: Vec<usize>,
    receiving_controls: Vec<usize>,
    receiving_fragments: Vec<usize>,
}

fn observe(prepared: &t::prepared::PreparedText, offer: t::prepared::PreparedOpportunity) -> Cut {
    let boundary = offer.view().boundary();
    let outgoing = finish(
        prepared,
        prepared.boundary_views().next().unwrap().handle(),
        offer,
    );
    let receiving = finish(prepared, boundary.clone(), paragraph_end(prepared));
    Cut {
        participant: boundary.view().participant_offset(),
        outgoing_controls: control_indices(
            prepared,
            outgoing.control_slots().map(|slot| slot.control()),
        ),
        outgoing_fragments: fragment_controls(prepared, &outgoing),
        receiving_controls: control_indices(
            prepared,
            receiving.control_slots().map(|slot| slot.control()),
        ),
        receiving_fragments: fragment_controls(prepared, &receiving),
    }
}

fn soft_at(
    prepared: &t::prepared::PreparedText,
    source: usize,
) -> Vec<t::prepared::PreparedOpportunity> {
    let offers = prepared
        .opportunity_views()
        .filter(|view| {
            view.kind() == t::prepared::PreparedBreakKind::Allowed
                && view.boundary().view().source_after().get() == source
        })
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert!(
        !offers.is_empty(),
        "qualify an actual source-owned allowed offer at {source}"
    );
    offers
}

fn closing(value: &str) -> t::prepared::PreparedText {
    capture(
        value,
        t::style::WhiteSpace::Preserve,
        &[(0, 2), (2, 3)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 2)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
            E::Connection(0),
            origin(1),
        ],
    )
}

fn assert_closing(value: &str) {
    let prepared = closing(value);
    let actual = soft_at(&prepared, 2)
        .into_iter()
        .map(|offer| observe(&prepared, offer))
        .collect::<Vec<_>>();
    // The permitted box-edge offer owns the closing event, not just
    // an equal numeric source offset. No competing pre-End offer may evade it.
    assert_eq!(
        actual,
        [Cut {
            participant: 4,
            outgoing_controls: vec![0, 1],
            outgoing_fragments: vec![0, 1],
            receiving_controls: vec![],
            receiving_fragments: vec![],
        }]
    );
}

#[test]
fn soft_space_after_last_box_character_keeps_the_closing_endpoint_outgoing() {
    assert_closing("X Y");
}

#[test]
fn real_zwsp_after_last_box_character_keeps_the_closing_endpoint_outgoing() {
    assert_closing("X\u{200b}Y");
}

#[test]
fn soft_cut_keeps_the_opening_endpoint_with_its_receiving_content() {
    let prepared = capture(
        "X Y",
        t::style::WhiteSpace::Preserve,
        &[(0, 2), (2, 3)],
        &[(K::InlineBoundaryStart, 2), (K::InlineBoundaryEnd, 3)],
        &[(0, 1)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            origin(1),
            control(1),
            E::ExitScope(1),
        ],
    );
    let actual = soft_at(&prepared, 2)
        .into_iter()
        .map(|offer| observe(&prepared, offer))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [Cut {
            participant: 2,
            outgoing_controls: vec![],
            outgoing_fragments: vec![],
            receiving_controls: vec![0, 1],
            receiving_fragments: vec![0, 1],
        }]
    );
}

#[test]
fn nested_closing_sequence_precedes_the_cut_and_opening_sequence_follows_it() {
    let prepared = capture(
        "X Y",
        t::style::WhiteSpace::Preserve,
        &[(0, 2), (2, 3)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::InlineBoundaryStart, 0),
            (K::InlineBoundaryEnd, 2),
            (K::InlineBoundaryEnd, 2),
            (K::InlineBoundaryStart, 2),
            (K::InlineBoundaryStart, 2),
            (K::InlineBoundaryEnd, 3),
            (K::InlineBoundaryEnd, 3),
        ],
        &[(0, 3), (1, 2), (4, 7), (5, 6)],
        vec![
            E::EnterScope(1),
            control(0),
            E::EnterScope(2),
            control(1),
            origin(0),
            control(2),
            E::ExitScope(2),
            control(3),
            E::ExitScope(1),
            E::Connection(0),
            E::EnterScope(3),
            control(4),
            E::EnterScope(4),
            control(5),
            origin(1),
            control(6),
            E::ExitScope(4),
            control(7),
            E::ExitScope(3),
        ],
    );
    let actual = soft_at(&prepared, 2)
        .into_iter()
        .map(|offer| observe(&prepared, offer))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [Cut {
            participant: 6,
            outgoing_controls: vec![0, 1, 2, 3],
            outgoing_fragments: vec![0, 1, 2, 3],
            receiving_controls: vec![4, 5, 6, 7],
            receiving_fragments: vec![4, 5, 6, 7],
        }]
    );
}

#[test]
fn deleted_positive_source_space_blocks_attachment_from_an_earlier_owner() {
    let prepared = capture(
        "X  Y",
        t::style::WhiteSpace::Collapse,
        &[(0, 2), (2, 3), (3, 4)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 3)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            E::Connection(0),
            origin(1),
            control(1),
            E::ExitScope(1),
            E::Connection(1),
            origin(2),
        ],
    );
    let deleted = prepared
        .unit_views()
        .find(|unit| unit.source_range() == t::source::Range::new(2, 3))
        .unwrap();
    assert!(deleted.processed_range().is_empty());
    let first = soft_at(&prepared, 2);
    let second = soft_at(&prepared, 3);
    let processed_cut = first[0].view().boundary().view().processed_position();
    for offer in first {
        assert_eq!(offer.view().boundary().view().participant_offset(), 3);
        assert_eq!(offer.view().boundary().view().source_before().get(), 2);
        let actual = observe(&prepared, offer);
        assert_eq!(actual.outgoing_controls, [0]);
        assert_eq!(actual.outgoing_fragments, [0]);
        assert_eq!(actual.receiving_controls, [1]);
        assert_eq!(actual.receiving_fragments, [1]);
    }
    // Equal processed cuts still retain the two distinct original SPACE owners.
    for offer in second {
        let boundary = offer.view().boundary();
        assert!(boundary.view().participant_offset() >= 4);
        assert_eq!(boundary.view().processed_position(), processed_cut);
        assert_eq!(boundary.view().source_before().get(), 3);
        assert_eq!(boundary.view().source_after().get(), 3);
        let participants = prepared
            .participants()
            .take(boundary.view().participant_offset())
            .collect::<Vec<_>>();
        assert!(participants.contains(&t::prepared::PreparedParticipant::Text(deleted.handle())));
    }
}

#[test]
fn forced_break_chronology_does_not_relocate_a_later_closing_endpoint() {
    let prepared = capture(
        "XY",
        t::style::WhiteSpace::Preserve,
        &[(0, 1), (1, 2)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::ForcedBreak, 1),
            (K::InlineBoundaryEnd, 1),
        ],
        &[(0, 2)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            control(2),
            E::ExitScope(1),
            E::Connection(0),
            origin(1),
        ],
    );
    let offers = prepared
        .opportunity_views()
        .filter(|view| view.kind() == t::prepared::PreparedBreakKind::Forced)
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(offers.len(), 1);
    let actual = observe(&prepared, offers[0].clone());
    assert_eq!(
        actual,
        Cut {
            participant: 3,
            outgoing_controls: vec![0, 1],
            outgoing_fragments: vec![0, 1],
            receiving_controls: vec![2],
            receiving_fragments: vec![2],
        }
    );
}

#[test]
fn real_font_tables_qualify_joining_and_ligature_oracles() {
    for (bytes, value, expected) in [
        (
            font::arabic(),
            "\u{628}\u{628}\u{628}",
            vec![
                (t::source::Range::new(0, 1), 100),
                (t::source::Range::new(1, 2), 277),
                (t::source::Range::new(2, 3), 82),
            ],
        ),
        (
            font::ligature(),
            "AA",
            vec![(t::source::Range::new(0, 2), 58)],
        ),
        (
            font::ligature(),
            "A\u{301}",
            vec![(t::source::Range::new(0, 2), 58)],
        ),
    ] {
        let head = font::table(&bytes, *b"head");
        assert_eq!(u16::from_be_bytes(head[18..20].try_into().unwrap()), 1000);
        let metrics = font::table(&bytes, *b"hmtx");
        for glyph in [35_usize, 36, 58, 82, 100, 277] {
            assert_eq!(
                u16::from_be_bytes(metrics[glyph * 4..glyph * 4 + 2].try_into().unwrap()),
                1000
            );
        }
        let mut system = t::System::default();
        system
            .register_fonts(bytes.clone(), crate::common::FONT_LIMITS)
            .unwrap();
        let layout = system
            .layout(
                t::source::Source::new(value),
                style(t::style::WhiteSpace::Preserve),
                t::options::Options::default(),
            )
            .unwrap();
        for run in layout.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
            let instance = run.effective_instance().unwrap();
            for glyph in run.glyphs() {
                assert_eq!(glyph.advance(), 10.0);
                let bounds = instance
                    .glyph_outline(glyph.id(), run.face().catalogue_identity())
                    .unwrap()
                    .bounds()
                    .unwrap();
                assert!(bounds.size.width > 0.0 && bounds.size.height > 0.0);
            }
        }
        let mut observed = layout
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs().iter().map(|glyph| (glyph.range(), glyph.id())))
            .collect::<Vec<_>>();
        observed.sort_by_key(|entry| (entry.0.start, entry.0.end));
        assert_eq!(observed, expected, "literal font oracle for {value:?}");
        assert_eq!(layout.source().text(), &t::source::Text::from(value));
    }
}

#[test]
fn source_free_bidi_before_end_cannot_strand_the_captured_closing_endpoint() {
    let p = capture(
        "X Y",
        t::style::WhiteSpace::Preserve,
        &[(0, 2), (2, 3)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::RightToLeftEmbedding, 2),
            (K::InlineBoundaryEnd, 2),
            (K::PopDirectionalFormat, 2),
        ],
        &[(0, 2)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            control(2),
            E::ExitScope(1),
            control(3),
            E::Connection(0),
            origin(1),
        ],
    );
    let actual = soft_at(&p, 2)
        .into_iter()
        .map(|o| observe(&p, o))
        .collect::<Vec<_>>();
    assert!(
        actual
            .iter()
            .all(|cut| cut.outgoing_controls.contains(&2) && cut.outgoing_fragments.contains(&2)),
        "closing End must be outgoing at every owner2 cut; {actual:?}"
    );
}
#[test]
fn source_free_bidi_after_start_cannot_select_the_captured_opening_endpoint_outgoing() {
    let p = capture(
        "X Y",
        t::style::WhiteSpace::Preserve,
        &[(0, 2), (2, 3)],
        &[
            (K::InlineBoundaryStart, 2),
            (K::RightToLeftEmbedding, 2),
            (K::PopDirectionalFormat, 3),
            (K::InlineBoundaryEnd, 3),
        ],
        &[(0, 3)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            control(1),
            origin(1),
            control(2),
            control(3),
            E::ExitScope(1),
        ],
    );
    let actual = soft_at(&p, 2)
        .into_iter()
        .map(|o| observe(&p, o))
        .collect::<Vec<_>>();
    assert!(
        actual
            .iter()
            .all(|cut| !cut.outgoing_controls.contains(&0) && !cut.outgoing_fragments.contains(&0)),
        "opening Start must be receiving at every owner2 cut; {actual:?}"
    );
}

#[path = "inline_boundary_edges/boundary_incidences.rs"]
mod boundary_incidences;

mod common;
