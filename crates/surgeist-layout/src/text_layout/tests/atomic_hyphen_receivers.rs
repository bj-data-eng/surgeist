//! A real ordinary atomic-edge cut remains available beside a manual indication.
use super::*;
use surgeist_text as text;

fn paragraph(placement: text::HyphenationPlacement) -> PreparedParagraph {
    let style = text::Style {
        hyphens: text::Hyphens::Manual,
        white_space: text::WhiteSpace::Collapse,
        locale: Some("und-Latn".into()),
        hyphenate_character: text::HyphenateCharacter::Auto,
        native_hyphenation_convention: Some(
            text::NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                "A".into(),
                placement,
            )
            .unwrap(),
        ),
        // The actual atomic terminates this lexical portion at AB + SHY.
        // Its after-count is zero, rather than borrowing CDE through the box.
        hyphenate_limit_chars: text::HyphenateLimitChars {
            total: text::HyphenationMinimum::Count(0),
            before: text::HyphenationMinimum::Count(0),
            after: text::HyphenationMinimum::Count(0),
        },
        ..ahem_style(10.0, 10.0)
    };
    PreparedParagraph::new(
        vec![
            ParagraphPart::text(
                1,
                Source::identified("AB\u{ad}", text::Id::from_u64(150301), 7),
                style.clone(),
            ),
            ParagraphPart::Atomic {
                node: 2,
                size: text::Size::new(10.0, 10.0),
            },
            ParagraphPart::text(
                3,
                Source::identified("CDE", text::Id::from_u64(150303), 9),
                style.clone(),
            ),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    )
}

fn check<S: LayoutScalar>(placement: text::HyphenationPlacement, wrap_style: TextWrapStyle) {
    let paragraph = paragraph(placement);
    let prepared = &paragraph.prepared;
    let original = [65, 66, 173, 67, 68, 69];
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        original
    );
    let origins = prepared.origin_views().collect::<Vec<_>>();
    assert_eq!(origins.len(), 2);
    for (origin, id, revision, units) in [
        (&origins[0], 150301, 7, &[65, 66, 173][..]),
        (&origins[1], 150303, 9, &[67, 68, 69][..]),
    ] {
        assert_eq!(origin.source().id(), Some(text::Id::from_u64(id)));
        assert_eq!(origin.source().revision(), revision);
        assert_eq!(origin.source().text().code_units(), units);
    }
    let cuts = prepared.hyphenation_cut_views().collect::<Vec<_>>();
    assert_eq!(cuts.len(), 1);
    assert_eq!(cuts[0].provenance(), text::HyphenationProvenance::Source);
    assert_eq!(
        cuts[0].counts(),
        Some(text::HyphenationCounts {
            total: 2,
            before: 2,
            after: 0
        })
    );
    let offer = cuts[0].opportunity().unwrap();
    // Allowed retains its independent None, unlike a Discretionary-only offer.
    // Do not suppress this real permission to force an incoming indication.
    let offer_view = offer.view();
    assert_eq!(offer_view.kind(), text::PreparedBreakKind::Allowed);
    let alternatives = offer_view.alternatives().collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 1);
    assert_eq!(prepared.alternative_views().count(), 1);
    assert_eq!(alternatives[0].text().code_units(), [65]);
    assert_eq!(alternatives[0].source_range(), Some(text::Range::new(2, 3)));
    assert_eq!(alternatives[0].insertion_anchor().get(), 3);
    assert_eq!(alternatives[0].opportunity(), Some(offer.clone()));
    assert!(matches!(alternatives[0].definition(),
        text::PreparedAlternativeDefinition::Source { origin, source }
            if origin == paragraph.origin(1) && source == text::Range::new(2, 3)));

    let atomic_views = prepared.atomic_views().collect::<Vec<_>>();
    assert_eq!(atomic_views.len(), 1);
    assert_eq!(atomic_views[0].source_position().get(), 3);
    assert_eq!(
        atomic_views[0].original_box().kind(),
        text::InlineBoxKind::InFlow
    );
    let occurrence = atomic_views[0].handle();
    let binding = paragraph.binding(0);
    assert_eq!(binding.atomics.len(), 1);
    assert_eq!(binding.atomics[0].node, 2);
    assert_eq!(binding.atomics[0].occurrence, occurrence);

    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::AUTO,
                ),
                text_wrap_style: wrap_style,
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .style(
            2,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(3, paragraph.text_input(3));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(40.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text().code_units(), original);
    assert_eq!(layout.lines().len(), 2);
    let mut lines = vec![None; 2];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let finalized = fragment.finalized_fragment().finalized_line();
        let line = &mut lines[fragment.line_index()];
        if let Some(previous) = line.as_ref() {
            assert_eq!(previous, &finalized);
        }
        *line = Some(finalized);
    }
    let lines = lines.into_iter().map(Option::unwrap).collect::<Vec<_>>();
    let mut allocation_count = 0;
    let mut slot_count = 0;
    for (index, line) in lines.iter().enumerate() {
        let selected = line.selected_line();
        let selected = selected.request();
        assert!(selected.alternative.is_none());
        assert!(selected.incoming.is_none());
        assert!(line.hyphenation_continuation().is_none());
        assert!(line.fragment_views().all(|fragment| !matches!(
            fragment.owner(),
            text::FinalizedFragmentOwner::Alternative(_)
        )));
        assert_eq!(line.feedback().advance, 30.0);
        assert_eq!(
            (line.feedback().occupied_start, line.feedback().occupied_end),
            (0.0, 30.0)
        );
        assert_eq!(
            line.feedback().baseline,
            if index == 0 { 10.0 } else { 8.0 }
        );
        assert_eq!(
            line.feedback().line_height,
            if index == 0 { 12.0 } else { 10.0 }
        );
        assert_eq!(
            layout.lines()[index].range(),
            text::Range::new(index * 3, index * 3 + 3)
        );
        assert_eq!(layout.lines()[index].baseline(), 10.0 + 10.0 * index as f64);
        let mut glyphs = Vec::new();
        for run in line.run_views() {
            assert_eq!(
                run.face().catalogue_identity(),
                prepared.font_catalogue_identity()
            );
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            for glyph in run.glyphs() {
                if glyph.advance() == 0.0 {
                    assert_eq!(glyph.range(), text::Range::new(2, 3));
                    assert_eq!(original[2], 173);
                    continue;
                }
                glyphs.push((glyph.range(), glyph.id(), glyph.advance()));
            }
        }
        assert_eq!(
            glyphs,
            if index == 0 {
                vec![
                    (text::Range::new(0, 1), 35, 10.0),
                    (text::Range::new(1, 2), 36, 10.0),
                ]
            } else {
                vec![
                    (text::Range::new(3, 4), 37, 10.0),
                    (text::Range::new(4, 5), 38, 10.0),
                    (text::Range::new(5, 6), 39, 10.0),
                ]
            }
        );
        allocation_count += selected.atomics.len();
        let slots = line.atomic_slots().collect::<Vec<_>>();
        slot_count += slots.len();
        assert_eq!(slots.len(), usize::from(index == 0));
        if let Some(slot) = slots.first() {
            assert_eq!(slot.occurrence, occurrence);
            assert_eq!(selected.atomics[0].occurrence, occurrence);
            assert_eq!(selected.atomics[0].allocation, slot.allocation);
            let text::AtomicLineAllocation::InFlow {
                flow, box_geometry, ..
            } = slot.allocation
            else {
                panic!("the real atomic retains its in-flow allocation")
            };
            assert_eq!(
                (flow.advance, flow.before_baseline, flow.after_baseline),
                (10.0, 10.0, 0.0)
            );
            assert_eq!(slot.origin.x + box_geometry.border.origin.x, 20.0);
            assert_eq!(slot.origin.y + box_geometry.border.origin.y, 0.0);
        }
    }
    assert_eq!((allocation_count, slot_count), (1, 1));
    assert_eq!(layout.inline_boxes().len(), 1);
    assert_eq!(
        layout.inline_boxes()[0].rect(),
        text::Rect::new(20.0, 0.0, 10.0, 10.0)
    );
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size, Size::new(S::from_f64(40.0), S::from_f64(22.0)));
        let atomics = entries
            .iter()
            .filter(|entry| entry.node() == 2)
            .collect::<Vec<_>>();
        assert_eq!(atomics.len(), 1);
        let atomic = atomics[0].output();
        assert_eq!(atomic.location, Point::new(S::from_f64(20.0), S::ZERO));
        assert_eq!(atomic.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
    }
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        original
    );
}

#[test]
fn real_atomic_none_cut_preserves_source_and_geometry_beside_after_and_both_indications() {
    // At width40 the later ordinary cut keeps AB + atomic30 / CDE30.
    // Pretty has no Tail gain for equal30/30; the alternative two-line20/40
    // program worsens body residual. Neither mode may force an indication.
    for placement in [
        text::HyphenationPlacement::LogicalAfter,
        text::HyphenationPlacement::BothSides,
    ] {
        for wrap_style in [TextWrapStyle::Auto, TextWrapStyle::Pretty] {
            check::<f32>(placement, wrap_style);
            check::<f64>(placement, wrap_style);
        }
    }
}
