//! Genuine styled wbr bindings, including text units without any source origin.

use super::*;

fn paragraph(shared_mark: bool, expanded: bool) -> PreparedParagraph {
    let mut base = ahem_style(10.0, 10.0);
    base.wrap = Wrap::None;
    let wbr = |replacement| ParagraphControlKind::Wbr {
        style: Box::new(surgeist_text::Style {
            word_space_transform: replacement,
            ..base.clone()
        }),
    };
    let first = if expanded {
        surgeist_text::WordSpaceTransform::Space
    } else {
        surgeist_text::WordSpaceTransform::None
    };
    let second = if expanded {
        surgeist_text::WordSpaceTransform::IdeographicSpace
    } else {
        surgeist_text::WordSpaceTransform::None
    };
    let parts = if shared_mark {
        vec![
            ParagraphPart::text(1, Source::new("A"), base.clone()),
            ParagraphPart::Control {
                node: 2,
                kind: wbr(first),
            },
            ParagraphPart::Control {
                node: 3,
                kind: wbr(second),
            },
            ParagraphPart::text(4, Source::new("\u{3099}"), base.clone()),
        ]
    } else {
        vec![
            ParagraphPart::Control {
                node: 1,
                kind: wbr(first),
            },
            ParagraphPart::Control {
                node: 2,
                kind: wbr(second),
            },
        ]
    };
    PreparedParagraph::new(parts, base, ParagraphBaseDirection::LeftToRight)
}

fn tree<S: LayoutScalar>(p: &PreparedParagraph, order: &[u32]) -> PublicLayoutTreeOf<S> {
    let binding = p.binding(0);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, order.iter().copied())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    for &node in order {
        tree = tree.children(node, []);
        tree = if binding.origins.iter().any(|row| row.node == node) {
            tree.input(node, p.text_input(node))
        } else {
            let control = &binding
                .controls
                .iter()
                .find(|row| row.node == node)
                .unwrap()
                .occurrence;
            tree.input(
                node,
                LayoutInputOf::text_control(TextControlParticipation::new(control.clone())),
            )
        };
    }
    tree
}

fn rejects<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    binding: TextParagraphBinding<u32>,
    expected: TextBindingError,
) {
    let Err(error) = LayoutFlowInput::try_new(
        tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("invalid actual wbr binding must refuse");
    };
    assert!(
        matches!(error.kind(), LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(actual)) if *actual == expected)
    );
}

fn generated_inventory<S: LayoutScalar>() {
    for expanded in [false, true] {
        let p = paragraph(false, expanded);
        let foreign = paragraph(false, expanded);
        let tree = tree::<S>(&p, &[1, 2]);
        let binding = p.binding(0);
        LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding.clone()],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        assert!(binding.origins.is_empty());
        if expanded {
            let participants: Vec<_> = p.prepared.participants().collect();
            assert_eq!(participants.len(), 2);
            assert!(participants.iter().all(|participant| matches!(
                participant,
                surgeist_text::PreparedParticipant::Text(_)
            )));
            for (participant, row) in participants.iter().zip(&binding.controls) {
                let surgeist_text::PreparedParticipant::Text(unit) = participant else {
                    unreachable!()
                };
                assert_eq!(unit.view().origins().count(), 0);
                assert_eq!(
                    unit.view().controls().collect::<Vec<_>>().as_slice(),
                    std::slice::from_ref(&row.occurrence)
                );
            }
        }
        let mut missing = binding.clone();
        missing.controls.remove(0);
        rejects(&tree, missing, TextBindingError::ControlCoverage);
        let mut duplicate = binding.clone();
        duplicate.controls.push(duplicate.controls[0].clone());
        rejects(&tree, duplicate, TextBindingError::ControlCoverage);
        let mut wrong = binding.clone();
        wrong.controls[0].occurrence = foreign.binding(0).controls[0].occurrence.clone();
        rejects(&tree, wrong, TextBindingError::ControlCoverage);
        let wrong_tree = tree.clone().input(
            1,
            LayoutInputOf::text_control(TextControlParticipation::new(
                foreign.binding(0).controls[0].occurrence.clone(),
            )),
        );
        rejects(
            &wrong_tree,
            binding.clone(),
            TextBindingError::ParticipantRole,
        );
        let forced_tree = tree
            .clone()
            .input(1, LayoutInputOf::line_break(LineBreakInputOf::new()));
        rejects(
            &forced_tree,
            binding.clone(),
            TextBindingError::ParticipantRole,
        );
        let reversed_tree = self::tree::<S>(&p, &[2, 1]);
        let mut reversed = binding;
        reversed.nodes = vec![2, 1];
        rejects(&reversed_tree, reversed, TextBindingError::ParticipantOrder);
    }
}

#[test]
fn wbr_generated_only_and_unexpanded_bindings_require_exact_inventory_role_and_order() {
    generated_inventory::<f32>();
    generated_inventory::<f64>();
}

fn shared_order<S: LayoutScalar>() {
    let p = paragraph(true, true);
    let binding = p.binding(0);
    let normal = tree::<S>(&p, &[1, 2, 3, 4]);
    LayoutFlowInput::try_new(
        &normal,
        0,
        vec![binding.clone()],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let reversed = tree::<S>(&p, &[1, 3, 2, 4]);
    let mut wrong = binding;
    wrong.nodes = vec![1, 3, 2, 4];
    // A and the mark remain in genuine source order. Generated scalar Text
    // units have no origins; their actual control positions still govern order.
    rejects(&reversed, wrong, TextBindingError::ParticipantOrder);
}

#[test]
fn equal_anchor_wbr_bindings_cannot_reverse_beside_real_mark_origins() {
    shared_order::<f32>();
    shared_order::<f64>();
}
