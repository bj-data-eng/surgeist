//! Nonempty indications retain their genuine original attachment, not a guessed source owner.
use super::*;
use crate::protocol::prepared::presentation::Attachment;
use crate::protocol::prepared::{PresentationUnitKind, SelectedRelationEvent};

fn unit_at(p: &PreparedText, source: Range) -> crate::PreparedUnit {
    let mut found = p.unit_views().filter(|unit| unit.source_range() == source);
    let unit = found.next().unwrap().handle();
    assert!(found.next().is_none());
    unit
}

fn original_inside(p: &PreparedText, scope: &crate::CapturedScope, source: Range) -> bool {
    // These two authored originals are disjoint and cover the actual raw
    // paragraph. Find that issued original, then use its authentic EVENT --
    // neither an insertion offset nor a range hull establishes box ownership.
    let mut found = p.origin_views().filter(|origin| {
        let range = origin.aggregate_range();
        range.start <= source.start && source.end <= range.end
    });
    let origin = found.next().unwrap();
    assert!(found.next().is_none());
    let event = origin.capture_position().unwrap();
    scope.enter_position() < event && event < scope.exit_position()
}

fn receipt(
    p: &PreparedText,
    line: &crate::SelectedLine,
    before: bool,
    alternative: &crate::PreparedAlternative,
) -> (crate::PresentationRange, Range) {
    let presentation = &line.inner.presentation;
    assert_eq!(presentation.domains.len(), 1);
    let domain = &presentation.domains[0];
    assert_eq!((domain.outer, domain.base), (0, 0));
    let (participant, atom) = match (before, domain.attachment) {
        (true, Attachment::Before { participant, atom }) => (participant, atom),
        (false, Attachment::After { participant, atom }) => (participant, atom),
        other => panic!("actual Before/After authority must remain distinct: {other:?}"),
    };
    // Manual SOURCE Before attaches to the authored SHY itself. After attaches
    // to the actual first receiving X. It is NOT Before's preceding visible X.
    let host = if before {
        Range::new(2, 3)
    } else {
        Range::new(3, 4)
    };
    let host_unit = unit_at(p, host);
    assert_eq!(
        p.participants().nth(participant),
        Some(crate::PreparedParticipant::Text(host_unit.clone()))
    );
    let atom = atom.expect("these genuine hosts are actual source scalars");
    assert_eq!(p.inner.analyzed.projection.atoms[atom].source, host);
    let crate::protocol::prepared::ParticipantData::Text(index) =
        p.inner.tables.participants[participant]
    else {
        panic!("this fixture has an authentic text attachment")
    };
    assert!(p.inner.tables.units[index].atoms.contains(&atom));
    assert_eq!(domain.units.len(), 1, "the real indication is exactly X");
    let index = domain.units.start;
    let unit = &presentation.units[index];
    assert_eq!(
        unit.participant, None,
        "replacement must not counterfeit an original participant"
    );
    assert_eq!(
        unit.kind,
        PresentationUnitKind::Replacement {
            alternative: alternative.index,
            local: Range::new(0, 1)
        }
    );
    let expected = if before {
        Range::new(2, 3)
    } else {
        Range::new(0, 1)
    };
    assert_eq!(unit.presentation, expected);
    assert_eq!(
        presentation
            .selected_relation
            .events
            .iter()
            .filter_map(|event| match event {
                SelectedRelationEvent::Indication { unit, atom } if *unit == index => Some(*atom),
                _ => None,
            })
            .collect::<Vec<_>>(),
        unit.atoms.clone().collect::<Vec<_>>()
    );
    (
        crate::PresentationRange {
            start: unit.presentation.start,
            end: unit.presentation.end,
        },
        unit.source,
    )
}

fn selected_indications(
    line: &crate::SelectedLine,
    owner: &crate::InlineBoxSource,
) -> Vec<(crate::InlineBoxFragment, crate::PresentationRange, Range)> {
    line.inline_box_pieces()
        .unwrap()
        .filter(|piece| &piece.owner() == owner)
        .flat_map(|piece| {
            let fragment = piece.fragment();
            piece
                .members()
                .filter(|member| {
                    member.participant().is_none() && !member.presentation_range().is_empty()
                })
                .map(|member| {
                    (
                        fragment.clone(),
                        member.presentation_range(),
                        member.unit_source_range(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn finalized_indications(
    line: &FinalizedLine,
    owner: &crate::InlineBoxSource,
) -> Vec<(crate::InlineBoxFragment, crate::PresentationRange, Range)> {
    line.inline_box_pieces()
        .unwrap()
        .filter(|piece| &piece.owner() == owner)
        .flat_map(|piece| {
            let fragment = piece.fragment();
            piece
                .members()
                .filter(|member| {
                    member.participant().is_none() && !member.presentation_range().is_empty()
                })
                .map(|member| {
                    (
                        fragment.clone(),
                        member.presentation_range(),
                        member.unit_source_range(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn finish_selected(selected: crate::SelectedLine) -> FinalizedLine {
    selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn attached_indications(scope_start: usize) {
    let bytes = mixed_fonts::fixture(true);
    controls(&bytes);
    let mut style = own();
    style.hyphens = Hyphens::Manual;
    style.locale = Some("und-Latn".into());
    style.native_hyphenation_convention = Some(
        NativeHyphenationConvention::language_script(
            "und-Latn",
            "Latn",
            Text::from("X"),
            HyphenationPlacement::BothSides,
        )
        .unwrap(),
    );
    // Same maintained constructor as mixed_real_before_after_both_indications.
    // at1: source SHY and receiver are in C. at3: SHY is in the actual domain,
    // but receiver is in C, though both generated emissions have anchor3.
    let processed = captured("XX\u{ad}XXX", scope_start, style, false);
    let scope = processed.captured_scope(1).unwrap();
    assert_eq!(scope.domain_topology(), SourceProcessingTopology::Complete);
    assert_eq!(scope.parent().unwrap().role(), crate::ScopeRole::Domain);
    let owner = crate::InlineBoxSource::captured(scope.clone());
    let p = prepare(&mut system_with(&bytes), &processed);
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [88, 88, 173, 88, 88, 88]
    );
    assert!(p.unit_views().all(|unit| unit.level().get() == 0));
    assert_eq!(
        p.origin_views()
            .map(|origin| origin.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(0, scope_start), Range::new(scope_start, 6)]
    );
    assert_eq!(
        original_inside(&p, &scope, Range::new(2, 3)),
        scope_start == 1
    );
    assert!(original_inside(&p, &scope, Range::new(3, 4)));
    let offer = p
        .hyphenation_cut_views()
        .find(|cut| cut.boundary().view().source_before().get() == 3)
        .unwrap()
        .opportunity()
        .unwrap();
    let chosen = offer.view().alternatives().next().unwrap().handle();
    assert_eq!(chosen.view().source_range(), Some(Range::new(2, 3)));
    assert_eq!(chosen.view().insertion_anchor().get(), 3);
    assert_eq!(chosen.view().text().code_units(), [88]);
    let mut first_request = ending(
        &p,
        p.boundary_views().next().unwrap().handle(),
        offer.clone(),
    );
    first_request.alternative = Some(chosen.clone());
    let first_selected = p.select(first_request.clone()).unwrap();
    let before = receipt(&p, &first_selected, true, &chosen);
    let prior_first = finish_selected(first_selected);
    let mut second_request = ending(&p, offer.view().boundary(), terminal(&p));
    second_request.incoming = prior_first.hyphenation_continuation();
    assert!(second_request.incoming.is_some());
    let second_selected = p.select(second_request.clone()).unwrap();
    let after = receipt(&p, &second_selected, false, &chosen);
    let prior_second = finish_selected(second_selected);
    let expected_first = vec![
        (
            Range::new(0, 1),
            GlyphBasis::Clockwise,
            274,
            0.0,
            0.0,
            0.0,
            10.0,
        ),
        (
            Range::new(1, 2),
            GlyphBasis::Clockwise,
            274,
            10.0,
            0.0,
            0.0,
            10.0,
        ),
        (
            Range::new(3, 3),
            GlyphBasis::Clockwise,
            274,
            20.0,
            0.0,
            0.0,
            10.0,
        ),
    ];
    let expected_second = vec![
        (
            Range::new(3, 3),
            GlyphBasis::Clockwise,
            274,
            0.0,
            0.0,
            0.0,
            10.0,
        ),
        (
            Range::new(3, 4),
            GlyphBasis::Clockwise,
            274,
            10.0,
            0.0,
            0.0,
            10.0,
        ),
        (
            Range::new(4, 5),
            GlyphBasis::Clockwise,
            274,
            20.0,
            0.0,
            0.0,
            10.0,
        ),
        (
            Range::new(5, 6),
            GlyphBasis::Clockwise,
            274,
            30.0,
            0.0,
            0.0,
            10.0,
        ),
    ];
    assert_eq!(packets(&prior_first, &bytes), expected_first);
    assert_eq!(packets(&prior_second, &bytes), expected_second);
    assert_eq!(
        (
            prior_first.feedback().advance,
            prior_second.feedback().advance
        ),
        (30.0, 40.0)
    );
    assert_eq!(prior_first.control_slots().count(), 0);
    assert_eq!(prior_second.control_slots().count(), 0);
    assert!(prior_first.inline_box_pieces().is_none());
    assert!(prior_second.inline_box_pieces().is_none());
    let family = p.inline_box_family(Vec::new(), None).unwrap();
    first_request.inline_boxes = Some(family.clone());
    let first_selected = p.select(first_request).unwrap();
    assert_eq!(receipt(&p, &first_selected, true, &chosen), before);
    let projected_before = selected_indications(&first_selected, &owner);
    if scope_start == 1 {
        assert_eq!(projected_before.len(), 1);
        assert_eq!((projected_before[0].1, projected_before[0].2), before);
    } else {
        assert!(
            projected_before.is_empty(),
            "the domain-owned SHY cannot lend its outgoing indication to C at the same insertion cut"
        );
    }
    let first = finish_selected(first_selected);
    assert_eq!(finalized_indications(&first, &owner), projected_before);
    second_request.inline_boxes = Some(family.clone());
    second_request.incoming = first.hyphenation_continuation();
    assert_eq!(second_request.incoming, first.hyphenation_continuation());
    let second_selected = p.select(second_request).unwrap();
    assert_eq!(receipt(&p, &second_selected, false, &chosen), after);
    let projected_after = selected_indications(&second_selected, &owner);
    assert_eq!(projected_after.len(), 1);
    assert_eq!((projected_after[0].1, projected_after[0].2), after);
    if scope_start == 1 {
        assert_eq!(
            projected_before[0].0, projected_after[0].0,
            "one unwrapped C fragment projects onto both actual lines"
        );
    }
    let second = finish_selected(second_selected);
    assert_eq!(finalized_indications(&second, &owner), projected_after);
    for (line, expected, advance) in [
        (&first, &expected_first, 30.0),
        (&second, &expected_second, 40.0),
    ] {
        assert_eq!(line.inline_box_family(), Some(&family));
        assert_eq!(
            packets(line, &bytes),
            *expected,
            "one real indication glyph; metadata never duplicates emission"
        );
        assert_eq!(line.feedback().advance, advance);
        assert_eq!(line.control_slots().count(), 0);
        assert!(line.fragment_views().any(
            |fragment| fragment.owner() == FinalizedFragmentOwner::Alternative(chosen.clone())
        ));
        assert!(
            line.mapping_views()
                .any(|mapping| mapping.alternative() == Some(chosen.clone()))
        );
        let replay = line.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(replay.selected_line(), line.selected_line());
        assert_eq!(packets(&replay, &bytes), *expected);
        assert_eq!(
            finalized_indications(&replay, &owner),
            finalized_indications(line, &owner)
        );
    }
    assert_eq!(packets(&prior_first, &bytes), expected_first);
    assert_eq!(packets(&prior_second, &bytes), expected_second);
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [88, 88, 173, 88, 88, 88]
    );
}

#[test]
fn real_before_and_after_replacements_keep_none_participants_on_their_attached_inline_fragment() {
    attached_indications(1);
}

#[test]
fn after_uses_actual_receiver_scope_without_lending_the_outgoing_domain_indication_at_the_same_cut()
{
    attached_indications(3);
}
