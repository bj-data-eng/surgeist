//! Qualification of supplied combination mode/context facts only. These
//! captures make no claim about combination analysis, shaping, or rendering.

use crate::*;
use std::sync::Arc;

fn mode_style(mode: TextCombineUpright) -> Style {
    Style {
        text_combine_upright: mode,
        ..Style::default()
    }
}

fn inline(
    style: Option<Style>,
    horizontal_inner_direction: Option<ParagraphBaseDirection>,
) -> SourceProcessingScope {
    SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style,
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
        horizontal_inner_direction,
    }
}

fn nested_input(axis: Option<SourceWritingAxis>) -> ContextualSourceInput {
    let mut input = super::input("A", WhiteSpace::Preserve);
    let SourceProcessingScope::Block {
        base_style,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("the real fixture's root is a Block")
    };
    base_style.text_combine_upright = TextCombineUpright::All;
    *writing_axis = axis;
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    input.scopes.extend([
        inline(
            Some(mode_style(TextCombineUpright::Digits(
                TextCombineUprightDigits::Three,
            ))),
            Some(ParagraphBaseDirection::RightToLeft),
        ),
        inline(None, None),
        inline(Some(mode_style(TextCombineUpright::None)), None),
    ]);
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::EnterScope(2),
        SourceProcessingEvent::EnterScope(3),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(3),
        SourceProcessingEvent::ExitScope(2),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    input
}

#[test]
fn closed_digit_limits_and_default_mode_survive_style_validation() {
    assert_eq!(TextCombineUpright::default(), TextCombineUpright::None);
    assert_eq!(
        Style::default().text_combine_upright,
        TextCombineUpright::None
    );
    for (limit, expected) in [
        (TextCombineUprightDigits::Two, 2),
        (TextCombineUprightDigits::Three, 3),
        (TextCombineUprightDigits::Four, 4),
    ] {
        assert_eq!(limit.max_units(), expected);
    }
    for mode in [
        TextCombineUpright::None,
        TextCombineUpright::All,
        TextCombineUpright::Digits(TextCombineUprightDigits::Two),
        TextCombineUpright::Digits(TextCombineUprightDigits::Three),
        TextCombineUpright::Digits(TextCombineUprightDigits::Four),
    ] {
        let authored = mode_style(mode);
        assert_eq!(authored.text_combine_upright, mode);
        assert_eq!(
            ValidatedStyle::try_from(authored)
                .unwrap()
                .text_combine_upright(),
            mode
        );
    }
}

#[test]
fn inline_axis_comes_from_its_domain_but_mode_and_inner_direction_are_own_facts() {
    for axis in [
        None,
        Some(SourceWritingAxis::Horizontal),
        Some(SourceWritingAxis::Vertical),
    ] {
        let input = nested_input(axis);
        let source = input.source.clone();
        let processed = Arc::new(process_contextual_source(input).unwrap());
        let root = processed.captured_scope(0).unwrap();
        let declared = processed.captured_scope(1).unwrap();
        let absent = processed.captured_scope(2).unwrap();
        let explicit_none = processed.captured_scope(3).unwrap();

        assert_eq!(processed.source(), &source);
        assert_eq!(processed.processed_text(), &Text::from("A"));
        assert_eq!(
            super::text(processed.plaintext_items(Affinity::After)),
            [0x41]
        );
        assert_eq!(root.text_combine_upright(), Some(TextCombineUpright::All));
        assert_eq!(
            declared.text_combine_upright(),
            Some(TextCombineUpright::Digits(TextCombineUprightDigits::Three))
        );
        assert_eq!(absent.text_combine_upright(), None);
        assert_eq!(
            explicit_none.text_combine_upright(),
            Some(TextCombineUpright::None)
        );
        assert!(absent.style().is_none());
        assert!(explicit_none.style().is_some());

        assert_eq!(
            root.horizontal_inner_direction(),
            Some(ParagraphBaseDirection::LeftToRight)
        );
        assert_eq!(
            declared.horizontal_inner_direction(),
            Some(ParagraphBaseDirection::RightToLeft)
        );
        assert_eq!(absent.horizontal_inner_direction(), None);
        assert_eq!(explicit_none.horizontal_inner_direction(), None);
        assert_eq!(declared.parent(), Some(root.clone()));
        assert_eq!(absent.parent(), Some(declared.clone()));
        assert_eq!(explicit_none.parent(), Some(absent.clone()));
        for scope in [&root, &declared, &absent, &explicit_none] {
            assert_eq!(scope.domain(), root);
            assert_eq!(scope.writing_axis(), axis);
        }
        // In particular, missing axis and missing inner direction do not gain
        // horizontal/LTR facts from the fixture's real outer LTR paragraph.
        assert_eq!(processed.paragraphs().count(), 1);
    }
}

#[test]
fn atomic_child_domains_do_not_borrow_parent_axis_or_inner_direction() {
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        for child_axis in [
            None,
            Some(SourceWritingAxis::Horizontal),
            Some(SourceWritingAxis::Vertical),
        ] {
            let mut input = super::hole(kind);
            let SourceProcessingScope::Block {
                writing_axis,
                horizontal_inner_direction,
                base_style,
                ..
            } = &mut input.scopes[0]
            else {
                unreachable!("actual parent Block")
            };
            *writing_axis = Some(SourceWritingAxis::Vertical);
            *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
            base_style.text_combine_upright = TextCombineUpright::All;
            match &mut input.scopes[1] {
                SourceProcessingScope::AtomicBlock {
                    writing_axis,
                    horizontal_inner_direction,
                    base_style,
                    ..
                }
                | SourceProcessingScope::OutOfFlowBlock {
                    writing_axis,
                    horizontal_inner_direction,
                    base_style,
                    ..
                } => {
                    *writing_axis = child_axis;
                    *horizontal_inner_direction = None;
                    base_style.text_combine_upright =
                        TextCombineUpright::Digits(TextCombineUprightDigits::Four);
                }
                _ => unreachable!("actual nominated content domain"),
            }
            // Declare a real parent inline scope before the child domain.
            // Child domain index 1 then has scope ordinal 2: the domain-root
            // lookup cannot accidentally use a domain index as a scope index.
            input.scopes.insert(1, inline(None, None));
            for event in &mut input.events {
                match event {
                    SourceProcessingEvent::EnterScope(scope)
                    | SourceProcessingEvent::ExitScope(scope)
                        if *scope == 1 =>
                    {
                        *scope = 2
                    }
                    _ => {}
                }
            }
            input.events.insert(2, SourceProcessingEvent::EnterScope(1));
            input
                .events
                .insert(input.events.len() - 2, SourceProcessingEvent::ExitScope(1));
            input
                .scopes
                .push(inline(None, Some(ParagraphBaseDirection::RightToLeft)));
            // Wrap the existing child Origin occurrence, keeping the genuine
            // atomic nomination, both original paragraphs and all connections.
            input.events.insert(7, SourceProcessingEvent::EnterScope(3));
            input.events.insert(9, SourceProcessingEvent::ExitScope(3));
            let source = input.source.clone();
            let processed = Arc::new(process_contextual_source(input).unwrap());
            let root = processed.captured_scope(0).unwrap();
            let parent_inline = processed.captured_scope(1).unwrap();
            let child = processed.captured_scope(2).unwrap();
            let descendant = processed.captured_scope(3).unwrap();

            assert_eq!(processed.source(), &source);
            assert_eq!(root.writing_axis(), Some(SourceWritingAxis::Vertical));
            assert_eq!(parent_inline.domain(), root);
            assert_eq!(
                parent_inline.writing_axis(),
                Some(SourceWritingAxis::Vertical)
            );
            assert_eq!(parent_inline.horizontal_inner_direction(), None);
            assert_eq!(child.domain(), child);
            assert_ne!(child.domain(), root);
            assert_eq!(child.parent(), Some(parent_inline));
            assert_eq!(descendant.parent(), Some(child.clone()));
            assert_eq!(descendant.domain(), child);
            assert_eq!(child.writing_axis(), child_axis);
            assert_eq!(descendant.writing_axis(), child_axis);
            assert_eq!(
                root.horizontal_inner_direction(),
                Some(ParagraphBaseDirection::LeftToRight)
            );
            assert_eq!(child.horizontal_inner_direction(), None);
            assert_eq!(
                descendant.horizontal_inner_direction(),
                Some(ParagraphBaseDirection::RightToLeft)
            );
            assert_eq!(
                child.text_combine_upright(),
                Some(TextCombineUpright::Digits(TextCombineUprightDigits::Four))
            );
            assert_eq!(descendant.text_combine_upright(), None);
            assert_eq!(descendant.domain_definition(), None);
            let nominated = processed.captured_atomic(0).unwrap();
            assert_eq!(
                child.domain_definition(),
                Some(match kind {
                    InlineBoxKind::InFlow =>
                        CapturedDomainDefinition::AtomicBlock { atomic: nominated },
                    InlineBoxKind::OutOfFlow =>
                        CapturedDomainDefinition::OutOfFlowBlock { atomic: nominated },
                })
            );
            let views = processed.paragraphs().collect::<Vec<_>>();
            assert_eq!(views.len(), 2);
            assert_eq!(
                views[0].source_pieces().collect::<Vec<_>>(),
                [Range::new(0, 3), Range::new(6, 9)]
            );
            assert_eq!(
                views[1].source_pieces().collect::<Vec<_>>(),
                [Range::new(3, 6)]
            );
            assert_eq!(
                super::text(views[0].plaintext_items(Affinity::After)),
                if kind == InlineBoxKind::InFlow {
                    "A  B"
                } else {
                    "A B"
                }
                .encode_utf16()
                .collect::<Vec<_>>()
            );
            assert_eq!(
                super::text(views[1].plaintext_items(Affinity::After)),
                " I ".encode_utf16().collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn equal_source_facts_do_not_merge_foreign_capture_identity() {
    let input = nested_input(Some(SourceWritingAxis::Vertical));
    let first = Arc::new(process_contextual_source(input.clone()).unwrap());
    let retained = Arc::clone(&first);
    let foreign = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(first.source(), foreign.source());
    assert_eq!(first.processed_text(), &Text::from("A"));
    assert_eq!(foreign.processed_text(), &Text::from("A"));
    for ordinal in 0..4 {
        let own = first.captured_scope(ordinal).unwrap();
        let alias = retained.captured_scope(ordinal).unwrap();
        let other = foreign.captured_scope(ordinal).unwrap();
        assert_eq!(own, own.clone());
        assert_eq!(own, alias);
        assert_eq!(own.capture_ordinal(), other.capture_ordinal());
        assert_eq!(own.writing_axis(), other.writing_axis());
        assert_eq!(
            own.horizontal_inner_direction(),
            other.horizontal_inner_direction()
        );
        assert_eq!(own.text_combine_upright(), other.text_combine_upright());
        assert_ne!(own, other);
        assert!(own.enter_position().same_capture(&alias.exit_position()));
        assert!(!own.enter_position().same_capture(&other.enter_position()));
        assert_eq!(
            own.enter_position().partial_cmp(&other.enter_position()),
            None
        );
        assert_ne!(own.domain(), other.domain());
    }
}
