//! Semantic `Display` text and `source()` chains of the paint error types.
//!
//! Every expected string is written from the documented meaning of its variant
//! (the crate's explanation and the validation it names), not from the current
//! `Debug` output. Each table has an exhaustive `match` that maps a variant to a
//! row, so a new variant cannot be added without a row and a message here.

use std::error::Error;
use surgeist_layout::SourceIndex;
use surgeist_ledger::{ProfileMode, RefusalReason};
use surgeist_paint::{
    AccountingFailure, BorderImageGeometryError, BoxCategoryError, BoxFacts, BoxId, BoxPaintKind,
    BudgetRefusal, ChildOrder, ContentEntry, ContextEffect, FragmentId, FragmentOwner,
    GradientGeometryError, GradientStopsError, LineId, PaintChange, PaintError, PaintInputError,
    PaintLimits, PaintOptions, PaintPhase, Participation, Position, PrimitiveId,
    RadialGradientError, ResourceKind, SourceChild, TextId, WorkKind, ZIndex,
    ZeroHeightGradientError, plan_paint_order, plan_stacking,
};

fn options() -> PaintOptions {
    PaintOptions::new(
        PaintLimits::unlimited(),
        ProfileMode::Summary { timing: false },
    )
}

fn assert_table<E>(table: &[(E, &str)], row: fn(&E) -> usize, rows: usize)
where
    E: Error + std::fmt::Debug,
{
    let mut seen = vec![false; rows];
    for (value, expected) in table {
        assert_eq!(value.to_string(), *expected, "message of {value:?}");
        let rendered = format!("{value:?}");
        assert_ne!(
            value.to_string(),
            rendered,
            "Display must not be the Debug form"
        );
        assert!(
            !value.to_string().contains(['{', '}']),
            "message of {value:?} contains struct syntax"
        );
        seen[row(value)] = true;
    }
    assert!(
        seen.iter().all(|&covered| covered),
        "a variant has no message row: {seen:?}"
    );
}

fn box_id(n: usize) -> BoxId {
    BoxId::new(n)
}
fn fragment(n: usize) -> FragmentId {
    FragmentId::new(n)
}
fn line(n: usize) -> LineId {
    LineId::new(n)
}
fn primitive(n: usize) -> PrimitiveId {
    PrimitiveId::new(n)
}
fn text(n: usize) -> TextId {
    TextId::new(n)
}

fn input_row(error: &PaintInputError) -> usize {
    match error {
        PaintInputError::BoxPaintCount { .. } => 0,
        PaintInputError::UnsupportedBoxKind { .. } => 1,
        PaintInputError::InvalidBoxCategory { .. } => 2,
        PaintInputError::MissingTextParent { .. } => 3,
        PaintInputError::InvalidTextParent { .. } => 4,
        PaintInputError::DuplicateMixedSourceIndex { .. } => 5,
        PaintInputError::MissingFragmentOwner { .. } => 6,
        PaintInputError::MissingFragmentParent { .. } => 7,
        PaintInputError::InvalidFragmentParent { .. } => 8,
        PaintInputError::InvalidBlockInInlinePlacement { .. } => 9,
        PaintInputError::UnsupportedFragmentTopology { .. } => 10,
        PaintInputError::DuplicateFragmentOrder { .. } => 11,
        PaintInputError::InconsistentFragmentOrder { .. } => 12,
        PaintInputError::MissingLineContainer { .. } => 13,
        PaintInputError::InvalidLineContainer { .. } => 14,
        PaintInputError::DuplicateLineOrder { .. } => 15,
        PaintInputError::DuplicateContentOrder { .. } => 16,
        PaintInputError::MissingFragmentLine { .. } => 17,
        PaintInputError::MissingLineIncidence { .. } => 18,
        PaintInputError::UnexpectedLineIncidence { .. } => 19,
        PaintInputError::InvalidLineIncidence { .. } => 20,
        PaintInputError::MissingPrimitiveFragment { .. } => 21,
        PaintInputError::InvalidPrimitiveTarget { .. } => 22,
        PaintInputError::RootFragmentBackground { .. } => 23,
        PaintInputError::MissingDecorationApplicator { .. } => 24,
        PaintInputError::InvalidDecorationApplicator { .. } => 25,
        PaintInputError::DuplicatePrimitiveOrder { .. } => 26,
    }
}

#[test]
fn paint_input_errors_state_what_input_is_wrong_in_words() {
    use PaintInputError as E;
    let table = [
        (
            E::BoxPaintCount {
                boxes: 3,
                box_paint: 2,
            },
            "the box paint table has 2 records for 3 boxes",
        ),
        (
            E::UnsupportedBoxKind {
                box_id: box_id(5),
                kind: BoxPaintKind::Table,
            },
            "box 5 has the table category, which paint order does not support",
        ),
        (
            E::UnsupportedBoxKind {
                box_id: box_id(5),
                kind: BoxPaintKind::InlineBlock,
            },
            "box 5 has the inline-block category, which paint order does not support",
        ),
        (
            E::InvalidBoxCategory {
                box_id: box_id(6),
                reason: BoxCategoryError::FloatingRoot,
            },
            "box 6 has an invalid paint category",
        ),
        (
            E::MissingTextParent {
                text: text(1),
                parent: box_id(9),
            },
            "text 1 names parent box 9, which is not in the box table",
        ),
        (
            E::InvalidTextParent {
                text: text(2),
                parent: box_id(7),
            },
            "text 2 names parent box 7, which cannot directly contain text",
        ),
        (
            E::DuplicateMixedSourceIndex {
                parent: box_id(4),
                source_index: SourceIndex::new(8),
                first: SourceChild::Box(box_id(2)),
                other: SourceChild::Text(text(3)),
            },
            "box 4 has two children, box 2 and text 3, with source index 8",
        ),
        (
            E::MissingFragmentOwner {
                fragment: fragment(5),
                owner: FragmentOwner::Text(text(6)),
            },
            "fragment 5 names owner text 6, which is not in its table",
        ),
        (
            E::MissingFragmentParent {
                fragment: fragment(5),
                parent: fragment(11),
            },
            "fragment 5 names parent fragment 11, which is not in the fragment table",
        ),
        (
            E::InvalidFragmentParent {
                fragment: fragment(5),
                parent: Some(fragment(12)),
            },
            "fragment 5 has parent fragment 12, which does not match its owner's parent",
        ),
        (
            E::InvalidFragmentParent {
                fragment: fragment(5),
                parent: None,
            },
            "fragment 5 has no parent fragment although its owner has a parent",
        ),
        (
            E::InvalidBlockInInlinePlacement {
                fragment: fragment(5),
                container: fragment(13),
            },
            "fragment 5 is not a valid block-in-inline placement in container fragment 13",
        ),
        (
            E::UnsupportedFragmentTopology {
                fragment: fragment(5),
                container: fragment(13),
                through: box_id(2),
            },
            "fragment 5 reaches container fragment 13 through box 2, a wrapper that paint order cannot skip",
        ),
        (
            E::DuplicateFragmentOrder {
                owner: FragmentOwner::Box(box_id(1)),
                logical_order: 4,
                first: fragment(6),
                other: fragment(7),
            },
            "fragments 6 and 7 of box 1 share logical order 4",
        ),
        (
            E::InconsistentFragmentOrder {
                first: fragment(6),
                other: fragment(7),
            },
            "fragments 6 and 7 are ordered inconsistently with their parents and lines",
        ),
        (
            E::MissingLineContainer {
                line: line(2),
                container: fragment(14),
            },
            "line 2 names container fragment 14, which is not in the fragment table",
        ),
        (
            E::InvalidLineContainer {
                line: line(2),
                container: fragment(14),
            },
            "line 2 names container fragment 14, which cannot contain lines",
        ),
        (
            E::DuplicateLineOrder {
                container: fragment(14),
                logical_order: 3,
                first: line(1),
                other: line(2),
            },
            "lines 1 and 2 in container fragment 14 share logical order 3",
        ),
        (
            E::DuplicateContentOrder {
                container: fragment(14),
                logical_order: 3,
                first: ContentEntry::Line(line(1)),
                other: ContentEntry::PromotedBlock(fragment(9)),
            },
            "line 1 and promoted block fragment 9 in container fragment 14 share content order 3",
        ),
        (
            E::MissingFragmentLine {
                fragment: fragment(5),
                line: line(2),
            },
            "fragment 5 names line 2, which is not in the line table",
        ),
        (
            E::MissingLineIncidence {
                fragment: fragment(5),
            },
            "fragment 5 must be on a line but names none",
        ),
        (
            E::UnexpectedLineIncidence {
                fragment: fragment(5),
                line: line(2),
            },
            "fragment 5 names line 2 but does not belong on a line",
        ),
        (
            E::InvalidLineIncidence {
                fragment: fragment(5),
                line: line(2),
            },
            "fragment 5 names line 2, which does not contain it",
        ),
        (
            E::MissingPrimitiveFragment {
                primitive: primitive(8),
                fragment: fragment(5),
            },
            "primitive 8 names fragment 5, which is not in the fragment table",
        ),
        (
            E::InvalidPrimitiveTarget {
                primitive: primitive(8),
                fragment: fragment(5),
            },
            "primitive 8 targets fragment 5, whose owner cannot receive its paint kind",
        ),
        (
            E::RootFragmentBackground {
                primitive: primitive(8),
                fragment: fragment(5),
            },
            "primitive 8 paints the root background on fragment 5; root backgrounds use canvas primitives",
        ),
        (
            E::MissingDecorationApplicator {
                primitive: primitive(8),
                applying_box: box_id(3),
            },
            "decoration primitive 8 names applying box 3, which is not in the box table",
        ),
        (
            E::InvalidDecorationApplicator {
                primitive: primitive(8),
                text_fragment: fragment(5),
                applying_box: box_id(3),
            },
            "decoration primitive 8 names box 3, which does not decorate text fragment 5",
        ),
        (
            E::DuplicatePrimitiveOrder {
                first: primitive(8),
                other: primitive(9),
                record_order: 2,
            },
            "primitives 8 and 9 share record order 2",
        ),
    ];
    assert_table(&table, input_row, 27);
}

fn refusal() -> BudgetRefusal {
    BudgetRefusal {
        origin: PaintPhase::Validate,
        limiting_phase: PaintPhase::Operation,
        item: 2,
        change: PaintChange::Work {
            kind: WorkKind::BoxValidation,
            amount: 3,
        },
        current: 5,
        reason: RefusalReason::Limit { limit: 6 },
    }
}

fn paint_row(error: &PaintError) -> usize {
    match error {
        PaintError::Input(_) => 0,
        PaintError::Gradient(_) => 1,
        PaintError::GradientStops(_) => 2,
        PaintError::RadialGradient(_) => 3,
        PaintError::ZeroHeightGradient(_) => 4,
        PaintError::BorderImage(_) => 5,
        PaintError::MissingRoot => 6,
        PaintError::MultipleRoots { .. } => 7,
        PaintError::MissingParent { .. } => 8,
        PaintError::SelfParent { .. } => 9,
        PaintError::Cycle { .. } => 10,
        PaintError::DuplicateSourceIndex { .. } => 11,
        PaintError::IncompatibleParticipation { .. } => 12,
        PaintError::CapacityOverflow => 13,
        PaintError::Allocation { .. } => 14,
        PaintError::Budget(_) => 15,
        PaintError::Accounting(_) => 16,
    }
}

#[test]
fn paint_errors_state_what_failed_in_words() {
    let table = [
        (
            PaintError::Input(PaintInputError::BoxPaintCount {
                boxes: 3,
                box_paint: 2,
            }),
            "paint input is invalid",
        ),
        (
            PaintError::Gradient(GradientGeometryError::InvalidWidth),
            "linear gradient geometry is invalid",
        ),
        (
            PaintError::GradientStops(GradientStopsError::EmptyList),
            "gradient stop list is invalid",
        ),
        (
            PaintError::RadialGradient(RadialGradientError::InvalidWidth),
            "radial gradient geometry is invalid",
        ),
        (
            PaintError::ZeroHeightGradient(ZeroHeightGradientError::ExpectedZeroHeight),
            "zero-height gradient colors cannot be resolved",
        ),
        (
            PaintError::BorderImage(BorderImageGeometryError::InvalidBox),
            "border image geometry is invalid",
        ),
        (
            PaintError::MissingRoot,
            "the root box is not in the box table or is not the box without a parent",
        ),
        (
            PaintError::MultipleRoots {
                first: box_id(1),
                other: box_id(2),
            },
            "boxes 1 and 2 both have no parent; exactly one root is allowed",
        ),
        (
            PaintError::MissingParent {
                child: box_id(3),
                parent: box_id(9),
            },
            "box 3 names parent 9, which is not in the box table",
        ),
        (
            PaintError::SelfParent { box_id: box_id(4) },
            "box 4 names itself as its parent",
        ),
        (
            PaintError::Cycle { box_id: box_id(5) },
            "box 5 is part of a parent cycle",
        ),
        (
            PaintError::DuplicateSourceIndex {
                parent: box_id(2),
                source_index: SourceIndex::new(6),
            },
            "box 2 has two children with source index 6",
        ),
        (
            PaintError::IncompatibleParticipation { box_id: box_id(7) },
            "box 7 has a participation its parent's child ordering does not admit",
        ),
        (
            PaintError::CapacityOverflow,
            "a storage size calculation overflowed",
        ),
        (
            PaintError::Allocation {
                resource: ResourceKind::ScratchBytes,
            },
            "allocating scratch storage failed",
        ),
        (
            PaintError::Allocation {
                resource: ResourceKind::RetainedResultBytes,
            },
            "allocating retained result storage failed",
        ),
        (
            PaintError::Budget(refusal()),
            "paint budget refused the operation",
        ),
        (
            PaintError::Accounting(AccountingFailure::Invariant),
            "paint accounting failed",
        ),
    ];
    assert_table(&table, paint_row, 17);
}

#[test]
fn wrapping_paint_errors_expose_their_cause_and_the_others_have_none() {
    let wrappers = [
        PaintError::Input(PaintInputError::BoxPaintCount {
            boxes: 3,
            box_paint: 2,
        }),
        PaintError::Gradient(GradientGeometryError::InvalidWidth),
        PaintError::GradientStops(GradientStopsError::EmptyList),
        PaintError::RadialGradient(RadialGradientError::InvalidWidth),
        PaintError::ZeroHeightGradient(ZeroHeightGradientError::ExpectedZeroHeight),
        PaintError::BorderImage(BorderImageGeometryError::InvalidBox),
        PaintError::Budget(refusal()),
        PaintError::Accounting(AccountingFailure::Invariant),
    ];
    for error in &wrappers {
        assert!(error.source().is_some(), "{error:?} wraps a cause");
    }
    let leaves = [
        PaintError::MissingRoot,
        PaintError::MultipleRoots {
            first: box_id(1),
            other: box_id(2),
        },
        PaintError::MissingParent {
            child: box_id(3),
            parent: box_id(9),
        },
        PaintError::SelfParent { box_id: box_id(4) },
        PaintError::Cycle { box_id: box_id(5) },
        PaintError::DuplicateSourceIndex {
            parent: box_id(2),
            source_index: SourceIndex::new(6),
        },
        PaintError::IncompatibleParticipation { box_id: box_id(7) },
        PaintError::CapacityOverflow,
        PaintError::Allocation {
            resource: ResourceKind::ScratchBytes,
        },
    ];
    for error in &leaves {
        assert!(error.source().is_none(), "{error:?} has no cause");
    }
}

#[test]
fn an_invalid_box_category_exposes_its_reason_as_the_cause() {
    let error = PaintInputError::InvalidBoxCategory {
        box_id: box_id(6),
        reason: BoxCategoryError::FloatingRoot,
    };
    assert!(error.source().is_some());
    let other = PaintInputError::BoxPaintCount {
        boxes: 3,
        box_paint: 2,
    };
    assert!(other.source().is_none());
}

#[test]
fn a_box_paint_table_of_the_wrong_length_is_reported_in_words_end_to_end() {
    // Two boxes (a root and its child) but an empty box paint table.
    let boxes = [
        BoxFacts::new(
            None,
            SourceIndex::ZERO,
            ChildOrder::Source,
            Participation::Ordinary,
            Position::Static,
            ZIndex::Auto,
            ContextEffect::None,
        ),
        BoxFacts::new(
            Some(BoxId::new(0)),
            SourceIndex::ZERO,
            ChildOrder::Source,
            Participation::Ordinary,
            Position::Static,
            ZIndex::Auto,
            ContextEffect::None,
        ),
    ];
    let request = surgeist_paint::PaintRequest::new(&boxes, &[], &[], &[], &[], &[], BoxId::new(0));
    let error = plan_paint_order(request, options()).result.unwrap_err();
    assert_eq!(error.to_string(), "paint input is invalid");
    assert_eq!(
        error.source().unwrap().to_string(),
        "the box paint table has 0 records for 2 boxes"
    );
}

#[test]
fn a_stacking_request_without_a_root_is_reported_in_words_end_to_end() {
    // The only box names a parent that is not in the table, so no box is a root.
    let boxes = [BoxFacts::new(
        Some(BoxId::new(7)),
        SourceIndex::ZERO,
        ChildOrder::Source,
        Participation::Ordinary,
        Position::Static,
        ZIndex::Auto,
        ContextEffect::None,
    )];
    let error = plan_stacking(&boxes, BoxId::new(0), options())
        .result
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "box 0 names parent 7, which is not in the box table"
    );
}

#[test]
fn a_root_outside_the_box_table_is_reported_in_words_end_to_end() {
    let error = plan_stacking(&[], BoxId::new(0), options())
        .result
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "the root box is not in the box table or is not the box without a parent"
    );
}
