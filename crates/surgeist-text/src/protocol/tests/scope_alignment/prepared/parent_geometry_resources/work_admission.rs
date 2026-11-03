//! Borrowed derivation helpers admit source inspection before doing the work.
use super::*;
use crate::engine::{analysis, scope_alignment::accounting::Ledger};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;

fn no_work() -> Ledger {
    // This fresh ledger funds only the helper invocation below. The real public
    // preparation already succeeded using the fixture's separate actual budget.
    Some(ScopeAlignmentBudget::new(ScopeAlignmentLimits::new(0, 0, 0, 0), 0, 0).unwrap())
}

fn refused_before_inspection(error: &Error) {
    assert_eq!(
        super::super::reason(error),
        ScopeAlignmentError::Limit {
            kind: ScopeAlignmentLimitKind::StructuralWork,
            required: 1,
            limit: 0,
        }
    );
}

#[test]
fn section_lookup_refuses_before_unfunded_source_inspection() {
    let f = funded();
    let a = &f.prepared.inner.analyzed;
    assert_eq!(a.sections.len(), 1);
    for owner in [
        analysis::BidiOwner::Atom(0),
        analysis::BidiOwner::Control(0),
    ] {
        assert_eq!(
            analysis::structural::owner_section(
                owner,
                &a.paragraph,
                &a.sections,
                a.combination.as_deref(),
                None,
            )
            .unwrap(),
            0,
        );
        let mut ledger = no_work();
        let error = analysis::structural::owner_section(
            owner,
            &a.paragraph,
            &a.sections,
            a.combination.as_deref(),
            ledger.as_mut(),
        )
        .unwrap_err();
        refused_before_inspection(&error);
        assert_eq!(ledger.unwrap().usage().structural_work(), 0);
    }
    let order = f
        .capture
        .captured_control(1)
        .unwrap()
        .event_position()
        .index;
    assert_eq!(
        analysis::structural::section_at_order(order, &a.sections, None).unwrap(),
        Some(0),
    );
    let mut ledger = no_work();
    let error =
        analysis::structural::section_at_order(order, &a.sections, ledger.as_mut()).unwrap_err();
    refused_before_inspection(&error);
}

#[test]
fn bidi_owner_walk_refuses_before_unfunded_anchor_inspection() {
    let f = funded();
    let a = &f.prepared.inner.analyzed;
    let p = &a.paragraph;
    assert_eq!(p.anchor_order.len(), 3);
    let positions = (0..p.anchor_order.len())
        .map(|anchor| analysis::anchor_atom(anchor, p, a.processed.source().boxes(), &a.projection))
        .collect::<Vec<_>>();
    let input = analysis::bidi_input::Input {
        typography: a.typography,
        projection: &a.projection,
        infos: &a.analysis.infos,
        boxes: a.processed.source().boxes(),
        paragraph: p,
        combination: a.combination.as_deref(),
        events: Some(&a.logical.events),
        anchor_positions: &positions,
    };
    let mut owners = Vec::new();
    input
        .visit(&mut None, |row, _| {
            owners.push(row.owner);
            Ok(())
        })
        .unwrap();
    // All three real endpoint anchors are nonemitting; only the authored X is
    // a bidi row. Emitted-row accounting cannot fund the preceding anchor walk.
    assert_eq!(owners, [analysis::BidiOwner::Atom(0)]);
    let mut ledger = no_work();
    let mut emitted = false;
    let error = input
        .visit(&mut ledger, |_, _| {
            emitted = true;
            Ok(())
        })
        .unwrap_err();
    refused_before_inspection(&error);
    assert!(!emitted);
    assert_eq!(ledger.unwrap().usage().structural_work(), 0);
}
