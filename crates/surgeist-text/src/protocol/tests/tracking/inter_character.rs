//! Functional native InterCharacter qualification over genuine selected bases.
use super::fixtures::{abc_premises, allocations, assert_cells, ligature_font, select_method};
use super::*;
use crate::{
    SpacingBoundaryGeometry as BoundaryGeometry, SpacingSlotGeometry, SpacingSlotOwner,
    SpacingSlotTarget, SpacingStretchability,
};

const UNITS: [Range; 3] = [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)];

#[path = "exact_identity.rs"]
mod exact_identity;

fn abc_raw() -> [RawGlyph; 3] {
    [
        (UNITS[0], 35, 0.0, 0.0, 10.0),
        (UNITS[1], 36, 0.0, 0.0, 10.0),
        (UNITS[2], 37, 0.0, 0.0, 10.0),
    ]
}

fn endpoint(unit: &crate::SpacingUnit, prepared: &PreparedText, index: usize, tracking: f64) {
    assert_eq!(unit.letter_spacing_before(), tracking);
    assert_eq!(unit.letter_spacing_after(), tracking);
    let pieces = unit.pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(
        pieces[0].owner(),
        SpacingSlotOwner::Source {
            origin: prepared.origin_views().next().unwrap().handle(),
            local: UNITS[index],
            aggregate: UNITS[index],
        }
    );
    assert_eq!(
        (
            pieces[0].presentation_range().start(),
            pieces[0].presentation_range().end()
        ),
        (index, index + 1)
    );
    assert_eq!(
        unit.participants().cloned().collect::<Vec<_>>(),
        [PreparedParticipant::Text(
            prepared.unit_views().nth(index).unwrap().handle()
        )]
    );
}

fn boundary_custody(selected: &SelectedLine, index: usize, tracking: f64) {
    let slot = selected.spacing_slots().nth(index).unwrap();
    assert_eq!(slot.selected_line(), *selected);
    let SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slot.target()
    else {
        panic!("an inter-character target must retain both actual endpoint units")
    };
    endpoint(&visual_before, &selected.prepared_text(), index, tracking);
    endpoint(
        &visual_after,
        &selected.prepared_text(),
        index + 1,
        tracking,
    );
    let flattened = slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>();
    let endpoint_pieces = visual_before
        .pieces()
        .chain(visual_after.pieces())
        .map(|piece| piece.owner())
        .collect::<Vec<_>>();
    assert_eq!(flattened, endpoint_pieces);
    assert_eq!(
        slot.participants().cloned().collect::<Vec<_>>(),
        visual_before
            .participants()
            .chain(visual_after.participants())
            .cloned()
            .collect::<Vec<_>>()
    );
}

fn independent_rows(line: &FinalizedLine, authored: f64, allocated: f64) {
    let rows = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].traversal_order < rows[1].traversal_order);
    for row in rows {
        assert_eq!(
            row.geometry,
            SpacingSlotGeometry::Boundary(BoundaryGeometry::Independent)
        );
        assert_eq!(row.stretchability, SpacingStretchability::Direct);
        assert_eq!(row.authored_word_spacing, 0.0);
        assert_eq!(row.authored_letter_spacing, authored);
        assert_eq!(row.allocation, allocated);
        assert_eq!(row.advance, authored + allocated);
        assert!(row.can_expand());
    }
}

#[test]
fn inter_character_replaces_allocations_on_the_same_qualified_basis_and_shared_cells() {
    abc_premises();
    let source = Source::identified("ABC", Id::from_u64(1566010), 1);
    let control = whole(&prepare(AHEM, &source, own(0.0), &UNITS));
    assert_eq!(raw(&control, AHEM), abc_raw());
    assert_eq!(control.feedback().advance, 30.0);
    assert_eq!(
        paint_offsets(&control),
        [(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)]
    );
    let prepared = prepare(AHEM, &source, own(4.0), &UNITS);
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    assert_eq!(selected.spacing_slots().len(), 2);
    for index in 0..2 {
        boundary_custody(&selected, index, 4.0);
    }
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(raw(&natural, AHEM), abc_raw());
    assert_eq!(natural.feedback().advance, 38.0);
    assert_eq!(natural.measurements().alignment().advance(), 38.0);
    assert_eq!(
        paint_offsets(&natural),
        [(0.0, 0.0), (14.0, 0.0), (28.0, 0.0)]
    );
    independent_rows(&natural, 4.0, 0.0);
    let expanded = natural
        .with_spacing(allocations(&selected, &[2.0, 2.0]))
        .unwrap();
    assert_eq!(expanded.selected_line(), selected);
    assert_eq!(raw(&expanded, AHEM), abc_raw());
    assert_eq!(expanded.feedback().advance, 42.0);
    assert_eq!(expanded.measurements().alignment().advance(), 42.0);
    assert_eq!(
        paint_offsets(&expanded),
        [(0.0, 0.0), (16.0, 0.0), (32.0, 0.0)]
    );
    independent_rows(&expanded, 4.0, 2.0);
    assert_cells(
        &prepared,
        expanded.clone(),
        &source,
        &[0.0, 16.0, 32.0, 42.0],
    );
    let replaced = expanded
        .with_spacing(allocations(&selected, &[1.0, 1.0]))
        .unwrap();
    assert_eq!(replaced.feedback().advance, 40.0);
    assert_eq!(
        paint_offsets(&replaced),
        [(0.0, 0.0), (15.0, 0.0), (30.0, 0.0)]
    );
    let fresh = selected
        .finalize(crate::LineAllocations {
            spacing: allocations(&selected, &[1.0, 1.0]),
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(raw(&replaced, AHEM), raw(&fresh, AHEM));
    assert_eq!(paint_offsets(&replaced), paint_offsets(&fresh));
    let restored = replaced.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(restored.selected_line(), selected);
    assert_eq!(restored.feedback().advance, 38.0);
    assert_eq!(paint_offsets(&restored), paint_offsets(&natural));
    assert_eq!(raw(&restored, AHEM), abc_raw());
    assert_eq!(expanded.feedback().advance, 42.0);
    assert_eq!(
        paint_offsets(&expanded),
        [(0.0, 0.0), (16.0, 0.0), (32.0, 0.0)]
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn inter_character_complete_amounts_reject_foreign_missing_duplicate_and_invalid_values_atomically()
{
    abc_premises();
    let source = Source::identified("ABC", Id::from_u64(1566011), 1);
    let prepared = prepare(AHEM, &source, own(4.0), &UNITS);
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(raw(&natural, AHEM), abc_raw());
    assert_eq!(natural.feedback().advance, 38.0);
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    assert_eq!(
        reason(
            &natural
                .with_spacing(SpacingAllocations::Complete(vec![]))
                .unwrap_err()
        ),
        &ProtocolError::MissingSpacingSlot { slot: 0 }
    );
    assert_eq!(
        reason(
            &natural
                .with_spacing(SpacingAllocations::Complete(vec![
                    SpacingAllocation {
                        slot: slots[0].clone(),
                        amount: 0.0
                    },
                    SpacingAllocation {
                        slot: slots[0].clone(),
                        amount: 0.0
                    },
                ]))
                .unwrap_err()
        ),
        &ProtocolError::DuplicateSpacingSlot { slot: 0 }
    );
    let other = select_method(&prepared, SpacingPolicy::InterCharacter);
    let foreign = other.spacing_slots().next().unwrap();
    assert_eq!(
        reason(
            &natural
                .with_spacing(SpacingAllocations::Complete(vec![SpacingAllocation {
                    slot: foreign,
                    amount: 0.0
                }]))
                .unwrap_err()
        ),
        &ProtocolError::ForeignSelectedLine
    );
    for (value, category) in [
        (f64::NAN, crate::TextNumericError::NonFinite),
        (f64::INFINITY, crate::TextNumericError::NonFinite),
        (-1.0, crate::TextNumericError::OutOfRange),
    ] {
        let error = natural
            .with_spacing(allocations(&selected, &[value, 0.0]))
            .unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
        assert!(
            matches!(error.detail(), Some(crate::ErrorDetail::InvalidTextNumeric { phase: crate::TextNumericPhase::Finalization, category: actual, .. }) if *actual == category),
            "{error:?}"
        );
    }
    let retry = natural
        .with_spacing(allocations(&selected, &[2.0, 2.0]))
        .unwrap();
    assert_eq!(retry.feedback().advance, 42.0);
    assert_eq!(retry.selected_line(), selected);
    assert_eq!(raw(&retry, AHEM), abc_raw());
    assert_eq!(natural.feedback().advance, 38.0);
    assert_eq!(
        paint_offsets(&natural),
        [(0.0, 0.0), (14.0, 0.0), (28.0, 0.0)]
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn real_liga_and_clig_defaults_change_the_spacing_capable_natural_width_before_allocation() {
    for tag in [*b"liga", *b"clig"] {
        let bytes = ligature_font(tag);
        let source = Source::identified("fiA", Id::from_u64(1566012), 1);
        let prepared = prepare(&bytes, &source, own(0.0), &UNITS);
        let ordinary = select_method(&prepared, SpacingPolicy::None)
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let ordinary_raw = [
            (Range::new(0, 2), 58, 0.0, 0.0, 10.0),
            (Range::new(2, 3), 35, 0.0, 0.0, 10.0),
        ];
        assert_eq!(raw(&ordinary, &bytes), ordinary_raw);
        assert_eq!(ordinary.feedback().advance, 20.0);
        let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
        let natural = selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let unligated = [
            (UNITS[0], 35, 0.0, 0.0, 10.0),
            (UNITS[1], 36, 0.0, 0.0, 10.0),
            (UNITS[2], 35, 0.0, 0.0, 10.0),
        ];
        assert_eq!(raw(&natural, &bytes), unligated);
        assert_eq!(natural.feedback().advance, 30.0);
        assert_eq!(
            paint_offsets(&natural),
            [(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)]
        );
        independent_rows(&natural, 0.0, 0.0);
        let filled = natural
            .with_spacing(allocations(&selected, &[6.0, 6.0]))
            .unwrap();
        assert_eq!(filled.feedback().advance, 42.0);
        assert_eq!(
            paint_offsets(&filled),
            [(0.0, 0.0), (16.0, 0.0), (32.0, 0.0)]
        );
        assert_eq!(raw(&filled, &bytes), unligated);
        let restored = filled.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(restored.selected_line(), selected);
        assert_eq!(restored.feedback().advance, 30.0);
        assert_eq!(raw(&restored, &bytes), unligated);
        assert_eq!(ordinary.feedback().advance, 20.0);
        assert_eq!(prepared.analyzed_source().source(), &source);
    }
}

#[test]
fn authored_optional_ligature_keeps_the_internal_candidate_and_allows_only_real_outside_movement() {
    let bytes = ligature_font(*b"liga");
    let source = Source::identified("fiA", Id::from_u64(1566013), 1);
    let mut style = own(0.0);
    style.font = style.font.feature("\"liga\" 1");
    let prepared = prepare(&bytes, &source, style, &UNITS);
    let ordinary = whole(&prepared);
    let expected_raw = [
        (Range::new(0, 2), 58, 0.0, 0.0, 10.0),
        (Range::new(2, 3), 35, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&ordinary, &bytes), expected_raw);
    assert_eq!(ordinary.feedback().advance, 20.0);
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    assert_eq!(selected.spacing_slots().len(), 2);
    for index in 0..2 {
        boundary_custody(&selected, index, 0.0);
    }
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(raw(&natural, &bytes), expected_raw);
    assert_eq!(natural.feedback().advance, 20.0);
    let rows = natural.spacing_slots().collect::<Vec<_>>();
    assert_eq!(
        rows[0].geometry,
        SpacingSlotGeometry::Boundary(BoundaryGeometry::Constrained)
    );
    assert!(!rows[0].can_expand());
    assert_eq!(
        rows[1].geometry,
        SpacingSlotGeometry::Boundary(BoundaryGeometry::Independent)
    );
    assert!(rows[1].can_expand());
    let outside = natural
        .with_spacing(allocations(&selected, &[0.0, 22.0]))
        .unwrap();
    assert_eq!(outside.feedback().advance, 42.0);
    assert_eq!(paint_offsets(&outside), [(0.0, 0.0), (32.0, 0.0)]);
    assert_eq!(raw(&outside, &bytes), expected_raw);
    let error = natural
        .with_spacing(allocations(&selected, &[2.0, 0.0]))
        .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
    let Some(crate::ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        pieces,
        provider_atoms,
        reason,
        constraints,
        ..
    }) = error.detail()
    else {
        panic!("expected preserved-optional capability: {error:?}")
    };
    assert_eq!(*stage, crate::SpacingStage::SelectedShaping);
    assert_eq!(
        *operation,
        crate::SpacingOperation::InterCharacterAllocation
    );
    assert_eq!(
        *reason,
        crate::SpacingFailureReason::PreservedOptionalLigature
    );
    assert_eq!(
        pieces.as_ref(),
        [
            crate::SpacingPiece::Source {
                origin: 0,
                local: UNITS[0],
                aggregate: UNITS[0]
            },
            crate::SpacingPiece::Source {
                origin: 0,
                local: UNITS[1],
                aggregate: UNITS[1]
            },
        ]
    );
    assert_eq!(
        provider_atoms
            .iter()
            .flat_map(|range| range.clone())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0].run, 0);
    assert_eq!(
        constraints[0]
            .components
            .iter()
            .flat_map(|range| range.clone())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(constraints[0].provenance.iter().any(|feature| feature.tag() == surgeist_shaper::Tag::new(b"liga") && feature.role() == surgeist_shaper::FeatureRole::Optional && matches!(feature.source(), surgeist_shaper::FeatureSource::Request(request) if request.origin() == surgeist_shaper::FeatureOrigin::Explicit && request.value == 1)));
    assert_eq!(natural.feedback().advance, 20.0);
    assert_eq!(raw(&natural, &bytes), expected_raw);
    assert_eq!(
        outside
            .with_spacing(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        20.0
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn actual_rlig_protects_its_internal_pair_and_preserves_an_independent_outside_boundary() {
    let bytes = ligature_font(*b"rlig");
    let source = Source::identified("fiA", Id::from_u64(1566014), 1);
    let prepared = prepare(&bytes, &source, own(0.0), &UNITS);
    let ordinary = whole(&prepared);
    let expected_raw = [
        (Range::new(0, 2), 58, 0.0, 0.0, 10.0),
        (Range::new(2, 3), 35, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&ordinary, &bytes), expected_raw);
    assert_eq!(ordinary.feedback().advance, 20.0);
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    assert_eq!(selected.spacing_slots().len(), 2);
    for index in 0..2 {
        boundary_custody(&selected, index, 0.0);
    }
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(raw(&natural, &bytes), expected_raw);
    assert_eq!(natural.feedback().advance, 20.0);
    let rows = natural.spacing_slots().collect::<Vec<_>>();
    assert_eq!(
        rows[0].geometry,
        SpacingSlotGeometry::Boundary(BoundaryGeometry::ProtectedRequired)
    );
    assert!(!rows[0].can_expand());
    assert_eq!(
        rows[1].geometry,
        SpacingSlotGeometry::Boundary(BoundaryGeometry::Independent)
    );
    assert!(rows[1].can_expand());
    assert_eq!(
        reason(
            &natural
                .with_spacing(allocations(&selected, &[2.0, 0.0]))
                .unwrap_err()
        ),
        &ProtocolError::IneligibleSpacingSlot { slot: 0 }
    );
    let outside = natural
        .with_spacing(allocations(&selected, &[0.0, 22.0]))
        .unwrap();
    assert_eq!(outside.feedback().advance, 42.0);
    assert_eq!(paint_offsets(&outside), [(0.0, 0.0), (32.0, 0.0)]);
    assert_eq!(raw(&outside, &bytes), expected_raw);
    assert_eq!(
        outside
            .with_spacing(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        20.0
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}
