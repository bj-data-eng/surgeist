//! Semantic `Display` text and `source()` chains of the paint error types.
//!
//! Every expected string is written from the documented meaning of its variant
//! (the crate's explanation and the validation it names), not from the current
//! `Debug` output. Each table has an exhaustive `match` that maps a variant to a
//! row, so a new variant cannot be added without a row and a message here.

use std::error::Error;
use surgeist_layout::{Edges, ItemOrder, PhysicalAxis, Point, Size, SourceIndex};
use surgeist_ledger::{Defect, ProfileMode, RefusalReason};
use surgeist_paint::PaintError;
use surgeist_paint::accounting::{
    AccountingFailure, BudgetRefusal, PaintChange, PaintLimits, PaintOptions, PaintPhase,
    ResourceKind, WorkKind,
};
use surgeist_paint::border_image::{
    BorderImageGeometryError, BorderImageOutset, BorderImageRequest, BorderImageSlice,
    BorderImageSource, BorderImageWidth, resolve_border_image,
};
use surgeist_paint::gradient_stops::{
    GradientPositionComponent, GradientStopItem, GradientStopsError, resolve_gradient_stops,
};
use surgeist_paint::linear_gradient::{
    GradientGeometryError, LinearGradientDirection, resolve_linear_gradient,
};
use surgeist_paint::paint_order::{
    BoxCategoryError, BoxPaint, BoxPaintKind, CanvasPaintKind, ContentEntry, FloatParticipation,
    FragmentId, FragmentOwner, LineId, PaintInputError, PaintPrimitive, PaintRequest, PrimitiveId,
    SourceChild, TextId, plan_paint_order,
};
use surgeist_paint::radial_gradient::{
    RadialGradientError, RadialGradientSize, resolve_radial_gradient,
};
use surgeist_paint::stacking::{
    BoxFacts, BoxId, ChildOrder, ContextEffect, Participation, Position, ZIndex, plan_stacking,
};
use surgeist_paint::zero_height_gradient::ZeroHeightGradientError;

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
            "primitives 8 and 9 have the same record order 2 for the same target and paint role (and applying box, for a decoration)",
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
            "box 7 has a participation that does not fit its place in the tree: a root must be ordinary; a child that is out of flow or under a source-order parent must be ordinary; an in-flow child of a flex or grid parent must be that parent's flex or grid item",
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
    let request = PaintRequest::new(&boxes, &[], &[], &[], &[], &[], BoxId::new(0));
    let error = plan_paint_order(request, options()).result.unwrap_err();
    assert_eq!(error.to_string(), "paint input is invalid");
    assert_eq!(
        error.source().unwrap().to_string(),
        "the box paint table has 0 records for 2 boxes"
    );
}

#[test]
fn a_child_naming_a_missing_parent_is_reported_in_words_end_to_end() {
    // The only box names a parent that is not in the table.
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

fn item_box(parent: Option<BoxId>, participation: Participation) -> BoxFacts {
    BoxFacts::new(
        parent,
        SourceIndex::ZERO,
        ChildOrder::Source,
        participation,
        Position::Static,
        ZIndex::Auto,
        ContextEffect::None,
    )
}

fn incompatible(box_id: usize) -> String {
    format!(
        "box {box_id} has a participation that does not fit its place in the tree: a root must be ordinary; a child that is out of flow or under a source-order parent must be ordinary; an in-flow child of a flex or grid parent must be that parent's flex or grid item"
    )
}

#[test]
fn a_root_with_an_item_participation_is_reported_without_naming_a_parent() {
    // The only box has no parent, so it is the root, but it claims to be a flex item.
    let boxes = [item_box(None, Participation::FlexItem(ItemOrder::new(0)))];
    let error = plan_stacking(&boxes, BoxId::new(0), options())
        .result
        .unwrap_err();
    assert!(matches!(
        error,
        PaintError::IncompatibleParticipation { box_id } if box_id == BoxId::new(0)
    ));
    assert_eq!(error.to_string(), incompatible(0));
}

#[test]
fn a_flex_item_under_a_source_order_parent_is_reported_in_words() {
    let boxes = [
        item_box(None, Participation::Ordinary),
        item_box(
            Some(BoxId::new(0)),
            Participation::FlexItem(ItemOrder::new(0)),
        ),
    ];
    let error = plan_stacking(&boxes, BoxId::new(0), options())
        .result
        .unwrap_err();
    assert!(matches!(
        error,
        PaintError::IncompatibleParticipation { box_id } if box_id == BoxId::new(1)
    ));
    assert_eq!(error.to_string(), incompatible(1));
}

#[test]
fn two_primitives_with_one_target_and_role_and_order_are_reported_in_words() {
    // One root box and two canvas background colors with the same record order.
    let boxes = [item_box(None, Participation::Ordinary)];
    let appearance = [BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None)];
    let primitives = [
        PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0),
        PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0),
    ];
    let request = PaintRequest::new(
        &boxes,
        &appearance,
        &[],
        &[],
        &[],
        &primitives,
        BoxId::new(0),
    );
    let error = plan_paint_order(request, options()).result.unwrap_err();
    assert_eq!(error.to_string(), "paint input is invalid");
    assert_eq!(
        error.source().unwrap().to_string(),
        "primitives 0 and 1 have the same record order 0 for the same target and paint role (and applying box, for a decoration)"
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

fn assert_error<E: Error + Send + Sync + 'static>() {}

#[test]
fn every_paint_error_type_is_a_sendable_standard_error() {
    assert_error::<PaintError>();
    assert_error::<PaintInputError>();
    assert_error::<BoxCategoryError>();
    assert_error::<BorderImageGeometryError>();
    assert_error::<GradientGeometryError>();
    assert_error::<GradientStopsError>();
    assert_error::<RadialGradientError>();
    assert_error::<ZeroHeightGradientError>();
    assert_error::<AccountingFailure>();
    assert_error::<BudgetRefusal>();
    let boxed: Box<dyn Error + Send + Sync> = Box::new(PaintError::CapacityOverflow);
    assert_eq!(boxed.to_string(), "a storage size calculation overflowed");
}

fn border_row(error: &BorderImageGeometryError) -> usize {
    match error {
        BorderImageGeometryError::InvalidBox => 0,
        BorderImageGeometryError::InvalidBorderWidth => 1,
        BorderImageGeometryError::InvalidSource => 2,
        BorderImageGeometryError::InvalidSlice => 3,
        BorderImageGeometryError::InvalidWidth => 4,
        BorderImageGeometryError::InvalidOutset => 5,
        BorderImageGeometryError::UnrepresentableGeometry => 6,
    }
}

#[test]
fn border_image_geometry_errors_name_the_invalid_input() {
    use BorderImageGeometryError as E;
    let table = [
        (
            E::InvalidBox,
            "border image box origin must be finite and its size finite and nonnegative",
        ),
        (
            E::InvalidBorderWidth,
            "border image border widths must be finite and nonnegative",
        ),
        (
            E::InvalidSource,
            "border image source coordinate size must be finite and positive, and its natural size finite and nonnegative",
        ),
        (
            E::InvalidSlice,
            "border image slice values must be finite and nonnegative",
        ),
        (
            E::InvalidWidth,
            "border image width multiples must be finite and nonnegative",
        ),
        (
            E::InvalidOutset,
            "border image outset values must be finite and nonnegative",
        ),
        (
            E::UnrepresentableGeometry,
            "border image geometry cannot be represented in f64",
        ),
    ];
    assert_table(&table, border_row, 7);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

fn gradient_row(error: &GradientGeometryError) -> usize {
    match error {
        GradientGeometryError::InvalidWidth => 0,
        GradientGeometryError::InvalidHeight => 1,
        GradientGeometryError::NonFiniteAngle => 2,
        GradientGeometryError::UnrepresentableLine => 3,
    }
}

#[test]
fn linear_gradient_geometry_errors_name_the_invalid_input() {
    use GradientGeometryError as E;
    let table = [
        (
            E::InvalidWidth,
            "gradient box width must be finite and nonnegative",
        ),
        (
            E::InvalidHeight,
            "gradient box height must be finite and nonnegative",
        ),
        (E::NonFiniteAngle, "gradient angle must be finite"),
        (
            E::UnrepresentableLine,
            "gradient line cannot keep finite, positive-length geometry in f64",
        ),
    ];
    assert_table(&table, gradient_row, 4);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

fn stops_row(error: &GradientStopsError) -> usize {
    match error {
        GradientStopsError::InvalidLineLength => 0,
        GradientStopsError::EmptyList => 1,
        GradientStopsError::EndpointHint { .. } => 2,
        GradientStopsError::AdjacentHints { .. } => 3,
        GradientStopsError::NonFiniteComponent { .. } => 4,
        GradientStopsError::UnrepresentablePosition { .. } => 5,
        GradientStopsError::UnrepresentableDistribution { .. } => 6,
    }
}

#[test]
fn gradient_stop_errors_name_the_item_and_position_at_fault() {
    use GradientStopsError as E;
    let table = [
        (
            E::InvalidLineLength,
            "gradient line length must be finite and nonnegative",
        ),
        (E::EmptyList, "gradient stop list is empty"),
        (
            E::EndpointHint { source_index: 4 },
            "gradient item 4 is a hint at the start or end of the list; the first and last items must be stops",
        ),
        (
            E::AdjacentHints {
                previous_source_index: 2,
                source_index: 3,
            },
            "gradient items 2 and 3 are adjacent hints",
        ),
        (
            E::NonFiniteComponent {
                source_index: 5,
                position_index: 1,
                component: GradientPositionComponent::Length,
            },
            "the length of position 1 of gradient item 5 is not finite",
        ),
        (
            E::NonFiniteComponent {
                source_index: 5,
                position_index: 0,
                component: GradientPositionComponent::Fraction,
            },
            "the fraction of position 0 of gradient item 5 is not finite",
        ),
        (
            E::UnrepresentablePosition {
                source_index: 6,
                position_index: 1,
            },
            "position 1 of gradient item 6 cannot be represented as a finite used position",
        ),
        (
            E::UnrepresentableDistribution {
                left_source_index: 1,
                right_source_index: 7,
            },
            "the used positions from gradient item 1 to gradient item 7 cannot be kept finite and non-decreasing in f64",
        ),
    ];
    assert_table(&table, stops_row, 7);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

fn radial_row(error: &RadialGradientError) -> usize {
    match error {
        RadialGradientError::InvalidWidth => 0,
        RadialGradientError::InvalidHeight => 1,
        RadialGradientError::NonFiniteCenter { .. } => 2,
        RadialGradientError::InvalidCircleRadius => 3,
        RadialGradientError::NonFiniteRadiusComponent { .. } => 4,
        RadialGradientError::NegativeRadius { .. } => 5,
        RadialGradientError::UnrepresentableGeometry => 6,
    }
}

#[test]
fn radial_gradient_errors_name_the_axis_and_component_at_fault() {
    use RadialGradientError as E;
    let table = [
        (
            E::InvalidWidth,
            "radial gradient box width must be finite and nonnegative",
        ),
        (
            E::InvalidHeight,
            "radial gradient box height must be finite and nonnegative",
        ),
        (
            E::NonFiniteCenter {
                axis: PhysicalAxis::Horizontal,
            },
            "radial gradient center is not finite on the horizontal axis",
        ),
        (
            E::NonFiniteCenter {
                axis: PhysicalAxis::Vertical,
            },
            "radial gradient center is not finite on the vertical axis",
        ),
        (
            E::InvalidCircleRadius,
            "radial gradient circle radius must be finite and nonnegative",
        ),
        (
            E::NonFiniteRadiusComponent {
                axis: PhysicalAxis::Vertical,
                component: GradientPositionComponent::Length,
            },
            "the length of the vertical radius is not finite",
        ),
        (
            E::NonFiniteRadiusComponent {
                axis: PhysicalAxis::Horizontal,
                component: GradientPositionComponent::Fraction,
            },
            "the fraction of the horizontal radius is not finite",
        ),
        (
            E::NegativeRadius {
                axis: PhysicalAxis::Horizontal,
            },
            "radial gradient horizontal radius is negative",
        ),
        (
            E::UnrepresentableGeometry,
            "radial gradient geometry cannot be represented in f64",
        ),
    ];
    assert_table(&table, radial_row, 7);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

fn zero_height_row(error: &ZeroHeightGradientError) -> usize {
    match error {
        ZeroHeightGradientError::ExpectedZeroHeight => 0,
        ZeroHeightGradientError::UnrepresentableWeight { .. } => 1,
    }
}

#[test]
fn zero_height_gradient_errors_name_the_case_and_positions_at_fault() {
    use ZeroHeightGradientError as E;
    let table = [
        (
            E::ExpectedZeroHeight,
            "gradient geometry is not the zero-height case",
        ),
        (
            E::UnrepresentableWeight {
                left_source_index: 2,
                left_position_index: 1,
                right_source_index: 4,
                right_position_index: 0,
            },
            "the color weight between position 1 of gradient item 2 and position 0 of gradient item 4 cannot be represented",
        ),
    ];
    assert_table(&table, zero_height_row, 2);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

fn category_row(error: &BoxCategoryError) -> usize {
    match error {
        BoxCategoryError::RootMustBeBlock => 0,
        BoxCategoryError::FloatingRoot => 1,
        BoxCategoryError::FloatingInline => 2,
        BoxCategoryError::OutOfFlowInline => 3,
        BoxCategoryError::InlineItem => 4,
        BoxCategoryError::FloatingItem => 5,
        BoxCategoryError::ReplacedHasChildren => 6,
    }
}

#[test]
fn box_category_errors_state_the_violated_category_rule() {
    use BoxCategoryError as E;
    let table = [
        (
            E::RootMustBeBlock,
            "the root box must have a block category",
        ),
        (E::FloatingRoot, "the root box must not float"),
        (
            E::FloatingInline,
            "a floating box must have a block category, not an inline one",
        ),
        (
            E::OutOfFlowInline,
            "an out-of-flow box must have a block category, not an inline one",
        ),
        (
            E::InlineItem,
            "a flex or grid item must have a block category, not an inline one",
        ),
        (E::FloatingItem, "a flex or grid item must not float"),
        (
            E::ReplacedHasChildren,
            "a replaced box must not have children",
        ),
    ];
    assert_table(&table, category_row, 7);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

fn accounting_row(error: &AccountingFailure) -> usize {
    match error {
        AccountingFailure::Allocation => 0,
        AccountingFailure::IdentityExhausted => 1,
        AccountingFailure::ForeignScope => 2,
        AccountingFailure::ScopeLimit { .. } => 3,
        AccountingFailure::TimingUnavailable => 4,
        AccountingFailure::LedgerDefect(_) => 5,
        AccountingFailure::Invariant => 6,
    }
}

#[test]
fn accounting_failures_describe_the_setup_or_invariant_failure() {
    let table = [
        (
            AccountingFailure::Allocation,
            "paint accounting could not reserve its storage",
        ),
        (
            AccountingFailure::IdentityExhausted,
            "paint accounting scope identities are exhausted",
        ),
        (
            AccountingFailure::ForeignScope,
            "an accounting scope belongs to a different operation",
        ),
        (
            AccountingFailure::ScopeLimit { limit: 10 },
            "paint accounting scope limit of 10 exceeded",
        ),
        (
            AccountingFailure::TimingUnavailable,
            "profile timing was requested but is unavailable",
        ),
        (
            AccountingFailure::LedgerDefect(Defect::ScopeEntry),
            "ledger invariant violated: scope identity names no scope entry",
        ),
        (
            AccountingFailure::Invariant,
            "paint accounting found an inconsistent internal state",
        ),
    ];
    assert_table(&table, accounting_row, 7);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn budget_refusals_state_the_change_phases_item_and_reason() {
    let work = |reason| BudgetRefusal {
        origin: PaintPhase::SiblingOrder,
        limiting_phase: PaintPhase::Operation,
        item: 4,
        change: PaintChange::Work {
            kind: WorkKind::SiblingComparison,
            amount: 3,
        },
        current: 5,
        reason,
    };
    let acquire = BudgetRefusal {
        origin: PaintPhase::Validate,
        limiting_phase: PaintPhase::Validate,
        item: 0,
        change: PaintChange::Acquire {
            kind: ResourceKind::ScratchBytes,
            amount: 64,
        },
        current: 8,
        reason: RefusalReason::Overflow,
    };
    let release = BudgetRefusal {
        origin: PaintPhase::Cleanup,
        limiting_phase: PaintPhase::Cleanup,
        item: 1,
        change: PaintChange::Release {
            kind: ResourceKind::RetainedResultBytes,
            amount: 16,
        },
        current: 0,
        reason: RefusalReason::Underflow,
    };
    let cases = [
        (
            work(RefusalReason::Limit { limit: 6 }),
            "3 sibling comparison work refused in the sibling_order phase by the operation phase at item 4 (usage 5): the limit is 6",
        ),
        (
            acquire,
            "acquiring 64 bytes of scratch storage refused in the validate phase by the validate phase at item 0 (usage 8): the total would overflow",
        ),
        (
            release,
            "releasing 16 bytes of retained result storage refused in the cleanup phase by the cleanup phase at item 1 (usage 0): the release exceeds the held amount",
        ),
    ];
    for (refusal, expected) in &cases {
        assert_eq!(refusal.to_string(), *expected);
        assert!(refusal.source().is_none());
    }
}

#[test]
fn work_and_storage_labels_read_as_words() {
    let label = |kind| {
        BudgetRefusal {
            origin: PaintPhase::Operation,
            limiting_phase: PaintPhase::Operation,
            item: 0,
            change: PaintChange::Work { kind, amount: 1 },
            current: 0,
            reason: RefusalReason::Overflow,
        }
        .to_string()
    };
    let words = [
        (WorkKind::BoxValidation, "box validation"),
        (WorkKind::ParentEdge, "parent edge"),
        (WorkKind::TopologyVisit, "topology visit"),
        (WorkKind::SiblingComparison, "sibling comparison"),
        (WorkKind::MembershipVisit, "membership visit"),
        (WorkKind::ContextComparison, "context comparison"),
        (WorkKind::OrderMove, "order move"),
        (WorkKind::OutputRecord, "output record"),
        (WorkKind::PaintValidation, "paint validation"),
        (WorkKind::FragmentVisit, "fragment visit"),
        (WorkKind::LineVisit, "line visit"),
        (WorkKind::InlineMemberVisit, "inline member visit"),
        (WorkKind::PrimitiveVisit, "primitive visit"),
        (WorkKind::PaintScopeVisit, "paint scope visit"),
        (WorkKind::PaintOrderComparison, "paint order comparison"),
        (WorkKind::DecorationComparison, "decoration comparison"),
        (WorkKind::GradientValidation, "gradient validation"),
        (WorkKind::GradientDirection, "gradient direction"),
        (WorkKind::GradientCoordinate, "gradient coordinate"),
        (WorkKind::GradientStopVisit, "gradient stop visit"),
        (WorkKind::BorderImageValidation, "border image validation"),
        (WorkKind::BorderImageCoordinate, "border image coordinate"),
    ];
    assert_eq!(words.len(), WorkKind::ALL.len());
    for (kind, word) in words {
        assert!(
            label(kind).starts_with(&format!("1 {word} work refused")),
            "{kind:?}"
        );
    }
}

#[test]
fn wrapping_errors_expose_the_wrapped_error_as_their_cause() {
    let input = PaintInputError::BoxPaintCount {
        boxes: 3,
        box_paint: 2,
    };
    let error = PaintError::from(input.clone());
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&input));
    assert!(
        !error.to_string().contains("2 records"),
        "cause text is not repeated"
    );

    let stops = GradientStopsError::AdjacentHints {
        previous_source_index: 1,
        source_index: 2,
    };
    let error = PaintError::from(stops);
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&stops));

    let radial = RadialGradientError::NegativeRadius {
        axis: PhysicalAxis::Vertical,
    };
    let error = PaintError::from(radial);
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&radial));

    let zero = ZeroHeightGradientError::ExpectedZeroHeight;
    let error = PaintError::from(zero);
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&zero));

    let gradient = GradientGeometryError::NonFiniteAngle;
    let error = PaintError::Gradient(gradient);
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&gradient));

    let border = BorderImageGeometryError::InvalidSlice;
    let error = PaintError::BorderImage(border);
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&border));

    let budget = refusal();
    let error = PaintError::Budget(budget);
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&budget));

    let accounting = AccountingFailure::ScopeLimit { limit: 3 };
    let error = PaintError::Accounting(accounting);
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&accounting));

    let category = BoxCategoryError::ReplacedHasChildren;
    let error = PaintInputError::InvalidBoxCategory {
        box_id: box_id(6),
        reason: category,
    };
    assert_eq!(error.source().unwrap().downcast_ref(), Some(&category));
}

#[test]
fn failed_operations_report_their_cause_chain_in_words() {
    let negative = plan_nan_width();
    assert_eq!(negative.to_string(), "linear gradient geometry is invalid");
    assert_eq!(
        negative.source().unwrap().to_string(),
        "gradient box width must be finite and nonnegative"
    );

    let empty: [GradientStopItem<()>; 0] = [];
    let error = resolve_gradient_stops(100.0, &empty, options())
        .result
        .unwrap_err();
    assert_eq!(error.to_string(), "gradient stop list is invalid");
    assert_eq!(
        error.source().unwrap().to_string(),
        "gradient stop list is empty"
    );

    let error = resolve_radial_gradient(
        Size::new(10.0, 10.0),
        None,
        RadialGradientSize::CircleRadius(f64::NAN),
        options(),
    )
    .result
    .unwrap_err();
    assert_eq!(error.to_string(), "radial gradient geometry is invalid");
    assert_eq!(
        error.source().unwrap().to_string(),
        "radial gradient circle radius must be finite and nonnegative"
    );

    let request = BorderImageRequest {
        border_origin: Point::new(0.0, 0.0),
        border_size: Size::new(10.0, 10.0),
        border_widths: Edges::all(1.0),
        source: BorderImageSource {
            coordinate_size: Size::new(10.0, 10.0),
            natural_size: Size::new(None, None),
        },
        slices: Edges::all(BorderImageSlice::SourceUnits(-1.0)),
        widths: Edges::all(BorderImageWidth::Auto),
        outsets: Edges::all(BorderImageOutset::Length(0.0)),
        fill_center: false,
    };
    let error = resolve_border_image(request, options()).result.unwrap_err();
    assert_eq!(error.to_string(), "border image geometry is invalid");
    assert_eq!(
        error.source().unwrap().to_string(),
        "border image slice values must be finite and nonnegative"
    );
}

fn plan_nan_width() -> PaintError {
    resolve_linear_gradient(
        Size::new(f64::NAN, 10.0),
        LinearGradientDirection::Bottom,
        options(),
    )
    .result
    .unwrap_err()
}

#[test]
fn a_refused_budget_reports_the_refusal_as_the_cause() {
    let limits = PaintLimits::unlimited().with_work_limit(WorkKind::GradientValidation, 0);
    let error = resolve_linear_gradient(
        Size::new(10.0, 10.0),
        LinearGradientDirection::Bottom,
        PaintOptions::new(limits, ProfileMode::Off),
    )
    .result
    .unwrap_err();
    assert_eq!(error.to_string(), "paint budget refused the operation");
    let cause = error.source().unwrap();
    assert!(
        cause
            .to_string()
            .starts_with("1 gradient validation work refused in the "),
        "{cause}"
    );
    assert!(cause.to_string().ends_with("the limit is 0"), "{cause}");
}
