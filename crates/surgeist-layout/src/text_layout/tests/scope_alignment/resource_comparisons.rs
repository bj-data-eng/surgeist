//! Conservative frame-comparison admission over a real completed native line.
//!
//! Fresh production resource owners isolate the helper's work law. The initial
//! supported default-cap path first completes capture, preparation, association
//! proof and Layout. The separate small-cap path retains its real compute
//! refusal. Neither case claims a history trace or complete caller coverage.
use super::*;
use crate::text_layout::scope_alignment::{ScopeAlignmentResources, admit_frame_comparisons};

const SMALL_WORK: usize = 4096;
const WORK: usize = 1_000_000;
// One candidate descriptor plus one actual equal-length frame.
const COMPARISON_WORK: usize = 2;

fn admitted(result: text::Result<()>) {
    if let Err(error) = result {
        panic!("unexpected comparison admission refusal: {error:?}");
    }
}

fn exact_refusal(error: text::Error, required: usize, limit: usize) {
    assert_eq!(error.code, text::ErrorCode::ScopeAlignmentRefused);
    let Some(text::ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("expected exact scope resource detail, got {error:?}")
    };
    assert_eq!(
        *reason,
        text::ScopeAlignmentError::Limit {
            kind: text::ScopeAlignmentLimitKind::StructuralWork,
            required,
            limit,
        }
    );
}

fn native<S: LayoutScalar>(
    case: &Case,
    batch: &CompletedLayoutBatchOf<u32, S>,
    line: &text::FinalizedLine,
    limits: text::ScopeAlignmentLimits,
) {
    assert_eq!(line.prepared_text(), case.binding.prepared);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (20.0, 11.0, 13.0)
    );
    assert_eq!(line.scope_allocations().len(), 1);
    assert_eq!(line.scope_allocations()[0].scope, case.scopes[1]);
    assert_eq!(own(line, &case.scopes[1]), (3.0, 3.0));
    assert_eq!(case.scopes[1].scope_alignment_limits(), Some(limits));
    assert_eq!(
        case.scopes[1].alignment_boundary(),
        Some(text::ScopeAlignmentBoundary::NonInitial)
    );
    let selected = line.selected_line();
    assert_eq!(selected.scopes().len(), 1);
    assert_eq!(
        selected.scopes().next().unwrap().captured_scope(),
        case.scopes[1]
    );
    assert!(selected.request().line_box_struts.is_empty());
    assert_eq!(selected.request().controls.len(), 2);
    for binding in &case.binding.controls {
        let captured = binding.occurrence.captured_control().unwrap();
        assert_eq!(captured.scope(), case.scopes[1]);
        assert_eq!(
            Some(captured),
            if binding.node == 20 {
                case.scopes[1].entry_control()
            } else {
                assert_eq!(binding.node, 21);
                case.scopes[1].exit_control()
            }
        );
        let allocation = selected
            .request()
            .controls
            .iter()
            .find(|allocation| allocation.occurrence == binding.occurrence)
            .unwrap();
        assert_eq!(
            allocation.boundary,
            Some(text::InlineBoundaryAllocation::CONTRIBUTION_ZERO)
        );
        assert_eq!(
            allocation.strut,
            Some(text::ControlStrut {
                baseline: 0.0,
                line_extent: 0.0,
                alignment: text::LineBoxAlignment::Baseline,
            })
        );
    }
    assert_eq!(
        case.binding
            .prepared
            .origin_views()
            .map(|origin| (
                origin.source().id(),
                origin.source().revision(),
                origin.local_range(),
                origin.aggregate_range(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                Some(text::Id::from_u64(1578101)),
                7,
                text::Range::new(0, 1),
                text::Range::new(0, 1)
            ),
            (
                Some(text::Id::from_u64(1578102)),
                7,
                text::Range::new(0, 1),
                text::Range::new(1, 2)
            ),
        ]
    );
    assert_eq!(placed(batch).source().text().code_units(), [88, 88]);
    assert_eq!(placed(batch).lines()[0].range(), text::Range::new(0, 2));
    assert_eq!(
        glyph_references(placed(batch)),
        [
            (text::Range::new(0, 1), 11.0),
            (text::Range::new(1, 2), 8.0)
        ]
    );
    assert_eq!(
        output(batch, 0).size,
        Size::new(S::from_f64(100.0), S::from_f64(13.0))
    );
    raw(placed(batch));
    source_custody(placed(batch));
}

fn qualify<S: LayoutScalar>(work: usize) {
    // Only the structural-work cap differs from the existing Case defaults.
    // In particular, no post-capture or PreparedText metadata is rewritten.
    let limits = text::ScopeAlignmentLimits::new(10000, 100, work, 16 << 20);
    let case = Case::new_with_limits(
        Shape::Single,
        text::ScopeAlignmentBoundary::NonInitial,
        10.0,
        "",
        1.0,
        limits,
    );
    let capture_usage = case.scopes[1].scope_alignment_usage();
    let prepared_usage = case.binding.prepared.scope_alignment_usage();
    let tree = case.tree::<S>(100.0, VerticalAlignOf::Baseline);
    let row = case.row(1, shifted::<S>(3.0));
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        case.bindings(),
        WrappingInput::default(),
        vec![row.clone()],
    )
    .unwrap();
    assert_eq!(flow.scope_alignments.rows(), [row]);
    // The original tested default cap is the successful pipeline prerequisite;
    // the sufficient-bound control uses a fresh capture with WORK+2.
    let batch = compute_layout_with_flow(&flow, 0, request(100.0)).unwrap();
    let finalized = lines(&batch);
    assert_eq!(finalized.len(), 1);
    let line = &finalized[0];
    native(&case, &batch, line, limits);
    let prior_entries = batch.final_entries().to_vec();
    let prior_feedback = line.feedback();
    let prior_frames = line.scope_allocations().to_vec();
    let prior_request = line.selected_line().request().clone();
    // This production ctor retains the genuine fully validated snapshot, while
    // creating its own operation-local resource state whose work starts at0.
    let facts = ScopeAlignmentResources::new(Arc::clone(&flow.scope_alignments));
    if work == WORK {
        // The conservative law is1 descriptor plus1 equal-length actual frame
        // per comparison. Repeated aliases are real immutable FinalizedLines;
        // the test invents neither frame records nor numeric owner identities.
        admitted(admit_frame_comparisons(
            line,
            std::iter::repeat_n(line, WORK / COMPARISON_WORK)
                .map(|other| other.scope_allocations().len()),
            &facts,
        ));
        for _ in 0..2 {
            let error = admit_frame_comparisons(
                line,
                std::iter::once(line).map(|other| other.scope_allocations().len()),
                &facts,
            )
            .unwrap_err();
            exact_refusal(error, WORK + COMPARISON_WORK, WORK);
        }
        // The identical retry diagnostic also checks the rejected debit did
        // not advance the accepted WORK in this fresh resource owner.
    } else {
        assert_eq!(work, WORK + COMPARISON_WORK);
        admitted(admit_frame_comparisons(
            line,
            std::iter::repeat_n(line, WORK / COMPARISON_WORK + 1)
                .map(|other| other.scope_allocations().len()),
            &facts,
        ));
    }
    assert_eq!(case.scopes[1].scope_alignment_usage(), capture_usage);
    assert_eq!(
        case.binding.prepared.scope_alignment_usage(),
        prepared_usage
    );
    assert_eq!(batch.final_entries(), prior_entries);
    assert_eq!(line.feedback(), prior_feedback);
    assert_eq!(line.scope_allocations(), prior_frames);
    assert_eq!(line.selected_line().request(), &prior_request);
    native(&case, &batch, line, limits);
}

fn small_capture_refuses_during_compute<S: LayoutScalar>() {
    let limits = text::ScopeAlignmentLimits::new(10000, 100, SMALL_WORK, 16 << 20);
    let case = Case::new_with_limits(
        Shape::Single,
        text::ScopeAlignmentBoundary::NonInitial,
        10.0,
        "",
        1.0,
        limits,
    );
    let capture_usage = case.scopes[1].scope_alignment_usage();
    let prepared_usage = case.binding.prepared.scope_alignment_usage();
    let origins = case
        .binding
        .prepared
        .origin_views()
        .map(|origin| origin.handle())
        .collect::<Vec<_>>();
    let controls = case
        .binding
        .controls
        .iter()
        .map(|control| control.occurrence.clone())
        .collect::<Vec<_>>();
    assert_eq!(case.scopes[1].scope_alignment_limits(), Some(limits));
    assert_eq!(
        case.scopes[1].alignment_boundary(),
        Some(text::ScopeAlignmentBoundary::NonInitial)
    );
    let tree = case.tree::<S>(100.0, VerticalAlignOf::Baseline);
    let row = case.row(1, shifted::<S>(3.0));
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        case.bindings(),
        WrappingInput::default(),
        vec![row.clone()],
    )
    .unwrap();
    assert_eq!(flow.scope_alignments.rows(), std::slice::from_ref(&row));
    // Actual execution established that capture, preparation and flow admission
    // succeed at4096, while complete computation needs more. Keep this genuine
    // refusal without pinning the internal operation or its exact visit count.
    let error = match compute_layout_with_flow(&flow, 0, request(100.0)) {
        Ok(_) => panic!("the independently retained small-cap witness published a complete batch"),
        Err(error) => error,
    };
    let LayoutErrorKindOf::Text(error) = error.kind() else {
        panic!("the valid small-cap fixture must refuse as the exact Text resource error")
    };
    assert_eq!(error.code, text::ErrorCode::ScopeAlignmentRefused);
    let Some(text::ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("expected scope work refusal, got {error:?}")
    };
    let text::ScopeAlignmentError::Limit {
        kind,
        required,
        limit,
    } = *reason
    else {
        panic!("expected the actual immutable work limit, got {reason:?}")
    };
    assert_eq!(kind, text::ScopeAlignmentLimitKind::StructuralWork);
    assert_eq!(limit, SMALL_WORK);
    assert!(required > limit);
    assert_eq!(case.scopes[1].scope_alignment_usage(), capture_usage);
    assert_eq!(
        case.binding.prepared.scope_alignment_usage(),
        prepared_usage
    );
    assert_eq!(case.scopes[1].scope_alignment_limits(), Some(limits));
    assert_eq!(flow.scope_alignments.rows(), [row]);
    assert_eq!(
        case.binding
            .prepared
            .origin_views()
            .map(|origin| origin.handle())
            .collect::<Vec<_>>(),
        origins
    );
    assert_eq!(
        case.binding
            .controls
            .iter()
            .map(|control| control.occurrence.clone())
            .collect::<Vec<_>>(),
        controls
    );
    for control in &case.binding.controls {
        assert_eq!(
            control.occurrence.captured_control().unwrap().scope(),
            case.scopes[1]
        );
    }
    assert_eq!(
        case.binding
            .prepared
            .origin_views()
            .map(|origin| (
                origin.source().id(),
                origin.source().revision(),
                origin.local_range(),
                origin.aggregate_range()
            ))
            .collect::<Vec<_>>(),
        [
            (
                Some(text::Id::from_u64(1578101)),
                7,
                text::Range::new(0, 1),
                text::Range::new(0, 1)
            ),
            (
                Some(text::Id::from_u64(1578102)),
                7,
                text::Range::new(0, 1),
                text::Range::new(1, 2)
            ),
        ]
    );
}

#[test]
fn genuine_small_work_cap_retains_capture_and_source_but_refuses_complete_layout() {
    font_premises();
    small_capture_refuses_during_compute::<f32>();
    small_capture_refuses_during_compute::<f64>();
}

#[test]
fn genuine_frame_comparison_admission_honors_immutable_work_caps_and_prior_publication() {
    font_premises();
    qualify::<f32>(WORK);
    qualify::<f32>(WORK + COMPARISON_WORK);
    qualify::<f64>(WORK);
    qualify::<f64>(WORK + COMPARISON_WORK);
}
