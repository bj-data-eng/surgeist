//! Source-only AX is a real consumer of the retained analysis, without fonts.
use super::*;
use crate::{Accessibility, Cursor, Selection};
use accesskit::{NodeId, Role};

#[test]
fn logical_ax_keeps_original_transformed_text_and_exact_byte_lengths_at_all_three_fronts() {
    let original = source("אa\u{301}😀");
    let transformed = Style {
        text_transform: TextTransform::Set(
            TextTransformSet::none().with_case(CaseTransform::Uppercase),
        ),
        ..style()
    };
    let raw = AnalyzedSource::new(original.clone(), transformed.clone(), upright(RTL)).unwrap();
    let mut input = native(original.clone());
    input.base_style = transformed.clone();
    let captured = AnalyzedSource::from_paragraph(input, upright(RTL)).unwrap();
    let mut input = contextual(original.clone());
    let SourceProcessingScope::Block { base_style, .. } = &mut input.scopes[0] else {
        unreachable!()
    };
    *base_style = transformed;
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let issued = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        upright(RTL),
    )
    .unwrap();
    drop(processed);
    for a in [raw, captured, issued] {
        assert_eq!(a.inner.projection.text, "אA\u{301}😀");
        assert_eq!(
            a.graphemes().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 3), Range::new(3, 5)]
        );
        let mut state = Accessibility::new(500);
        let parent = NodeId(100);
        let update = a.accessibility_update(&mut state, parent, parent).unwrap();
        let runs: Vec<_> = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::TextRun)
            .collect();
        assert_eq!(
            runs.iter()
                .map(|(_, n)| n.value().unwrap())
                .collect::<Vec<_>>(),
            ["א", "a\u{301}", "😀"]
        );
        assert_eq!(
            runs.iter()
                .map(|(_, n)| n.character_lengths().to_vec())
                .collect::<Vec<_>>(),
            [vec![2], vec![3], vec![4]]
        );
        assert!(update.nodes.iter().all(|(_, n)| n.bounds().is_none()
            && n.font_family().is_none()
            && n.font_size().is_none()
            && n.text_direction().is_none()
            && n.character_positions().is_none()
            && n.character_widths().is_none()));
        let caret = a
            .access_position(&state, Cursor::new(3, Affinity::After))
            .unwrap()
            .unwrap();
        assert_eq!(caret.node, runs[2].0);
        assert_eq!(caret.character_index, 0);
        let selected = a
            .access_selection(
                &state,
                Selection::new(
                    Cursor::new(0, Affinity::After),
                    Cursor::new(5, Affinity::Before),
                ),
            )
            .unwrap()
            .unwrap();
        assert_eq!(selected.anchor.node, runs[0].0);
        assert_eq!(selected.anchor.character_index, 0);
        assert_eq!(selected.focus.node, runs[2].0);
        assert_eq!(selected.focus.character_index, 1);
        assert_eq!(a.source(), &original);
        assert_eq!(a.typography(), upright(RTL));
        assert_eq!(a.direction(), Direction::LeftToRight);
    }
}

#[test]
fn logical_ax_binds_actual_analysis_and_preserves_prior_state_after_ill_formed_refusal() {
    let a = AnalyzedSource::new(source("אA"), style(), upright(RTL)).unwrap();
    let clone = a.clone();
    let equal = AnalyzedSource::new(a.source().clone(), style(), upright(RTL)).unwrap();
    let horizontal = AnalyzedSource::new(a.source().clone(), style(), H).unwrap();
    let mut state = Accessibility::new(500);
    let parent = NodeId(100);
    let initial = a.accessibility_update(&mut state, parent, parent).unwrap();
    let cursor = Cursor::new(1, Affinity::After);
    let point = a.access_position(&state, cursor).unwrap();
    assert_eq!(clone.access_position(&state, cursor).unwrap(), point);
    for foreign in [&equal, &horizontal] {
        assert_eq!(
            foreign
                .access_position(&state, cursor)
                .unwrap_err()
                .detail(),
            Some(&ErrorDetail::AccessibilityBindingMismatch)
        );
    }
    let bad = AnalyzedSource::new(
        source(Text::from_code_units(vec![0xd800, 0x5d0])),
        style(),
        upright(RTL),
    )
    .unwrap();
    assert_eq!(
        bad.accessibility_update(&mut state, parent, parent)
            .unwrap_err()
            .detail(),
        Some(&ErrorDetail::AccessibilityIllFormedSource {
            index: 0,
            unit: 0xd800
        })
    );
    assert_eq!(a.access_position(&state, cursor).unwrap(), point);
    let repeat = a.accessibility_update(&mut state, parent, parent).unwrap();
    assert_eq!(
        repeat.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        initial.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>()
    );
    drop(a);
    drop(equal);
    drop(horizontal);
    let update = clone
        .accessibility_update(&mut state, parent, parent)
        .unwrap();
    assert_eq!(
        update
            .nodes
            .iter()
            .filter_map(|(_, n)| n.value())
            .collect::<Vec<_>>(),
        ["א", "A"]
    );
    assert_eq!(clone.typography(), upright(RTL));
}

#[test]
fn wrapping_derived_context_has_new_ax_binding_without_losing_source_or_typography() {
    let a = AnalyzedSource::from_paragraph(native(source("אA")), upright(RTL)).unwrap();
    let policies = a
        .wrapping_relations()
        .map(|r| WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: Wrap::None,
        })
        .collect();
    let derived = a.with_wrapping_policies(policies).unwrap();
    let parent = NodeId(100);
    let mut state = Accessibility::new(500);
    a.accessibility_update(&mut state, parent, parent).unwrap();
    let cursor = Cursor::new(1, Affinity::After);
    assert_eq!(
        derived
            .access_position(&state, cursor)
            .unwrap_err()
            .detail(),
        Some(&ErrorDetail::AccessibilityBindingMismatch)
    );
    let new = derived
        .accessibility_update(&mut state, parent, parent)
        .unwrap();
    assert_eq!(
        new.nodes
            .iter()
            .filter_map(|(_, n)| n.value())
            .collect::<Vec<_>>(),
        ["א", "A"]
    );
    assert!(derived.access_position(&state, cursor).unwrap().is_some());
    assert_eq!(
        a.access_position(&state, cursor).unwrap_err().detail(),
        Some(&ErrorDetail::AccessibilityBindingMismatch)
    );
    assert_eq!(derived.typography(), upright(RTL));
    assert_eq!(derived.direction(), Direction::LeftToRight);
    assert_eq!(derived.source(), a.source());
}
