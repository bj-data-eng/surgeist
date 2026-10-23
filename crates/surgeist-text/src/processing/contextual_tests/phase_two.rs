//! Terminal authority follows event/domain custody, not equal source offsets.
use super::*;

#[test]
fn repeated_paragraphs_and_neutral_scope_closures_preserve_real_block_end() {
    let mut capture = input("X Y ", WhiteSpace::Preserve);
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 2)),
        origin(&capture.source, Range::new(2, 4)),
    ];
    capture
        .paragraphs
        .push(facts(ParagraphBaseDirection::LeftToRight));
    capture.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
    });
    use SourceProcessingEvent::*;
    capture.events = vec![
        EnterScope(0),
        EnterParagraph(0),
        EnterScope(1),
        Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        ExitParagraph(0),
        EnterParagraph(1),
        Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        ExitParagraph(1),
        ExitScope(1),
        ExitScope(0),
    ];
    let processed = Arc::new(process_contextual_source(capture).unwrap());
    let views: Vec<_> = processed.paragraphs().collect();
    let mut system = system();
    let first = prepare(&mut system, views[0].clone());
    let last = prepare(&mut system, views[1].clone());
    assert_eq!(
        (first.terminal_edge(), last.terminal_edge()),
        (
            ParagraphTerminalEdge::NonBlockBoundary,
            ParagraphTerminalEdge::BlockEnd
        )
    );
    assert_eq!(
        last.analyzed_source().source().text().code_units(),
        &[88, 32, 89, 32]
    );
    assert_eq!(line(&last).feedback().advance, 20.0);
}

#[test]
fn later_empty_control_and_out_of_flow_paragraph_are_not_erased_by_equal_offsets() {
    for later in 0..3 {
        let mut capture = input("X ", WhiteSpace::Preserve);
        capture
            .paragraphs
            .push(facts(ParagraphBaseDirection::LeftToRight));
        use SourceProcessingEvent::*;
        capture.events.pop();
        if later == 0 {
            capture
                .origins
                .push(origin(&capture.source, Range::new(2, 2)));
            capture.events.extend([
                EnterParagraph(1),
                Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                ExitParagraph(1),
            ]);
        } else if later == 1 {
            capture.controls.push(ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            });
            capture.events.extend([
                EnterParagraph(1),
                Control {
                    control: 0,
                    affinity: Affinity::After,
                },
                ExitParagraph(1),
            ]);
        } else {
            capture.source.inline_box(InlineBox::new(
                Id::from_u64(51),
                InlineBoxKind::OutOfFlow,
                2,
                Size::new(5.0, 10.0),
            ));
            capture.scopes.push(SourceProcessingScope::OutOfFlowBlock {
                atomic: 0,
                trim: WhiteSpaceTrim::default(),
                base_style: style(WhiteSpace::Preserve, 10.0),
                used_direction: ParagraphBaseDirection::LeftToRight,
            });
            capture
                .paragraphs
                .push(facts(ParagraphBaseDirection::LeftToRight));
            capture.events.extend([
                EnterParagraph(1),
                Atomic {
                    atomic: 0,
                    affinity: Affinity::After,
                },
                EnterScope(1),
                EnterParagraph(2),
                ExitParagraph(2),
                ExitScope(1),
                ExitParagraph(1),
            ]);
        }
        capture.events.push(ExitScope(0));
        let processed = Arc::new(process_contextual_source(capture).unwrap());
        let mut system = system();
        let first = prepare(&mut system, processed.paragraphs().next().unwrap());
        assert_eq!(
            first.terminal_edge(),
            ParagraphTerminalEdge::NonBlockBoundary
        );
        assert_eq!(
            first.analyzed_source().source().text().code_units(),
            &[88, 32]
        );
    }
}

#[test]
fn empty_control_and_child_domain_inside_final_paragraph_keep_its_terminal_custody() {
    let mut capture = input("X I", WhiteSpace::Preserve);
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(52),
        InlineBoxKind::OutOfFlow,
        2,
        Size::new(5.0, 10.0),
    ));
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 2)),
        origin(&capture.source, Range::new(3, 3)),
        origin(&capture.source, Range::new(2, 3)),
    ];
    capture.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    capture
        .boundary_policies
        .push(policy(ParagraphBoundaryDefinition::Connection(0)));
    capture.controls.push(ParagraphControl {
        at: 3,
        kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
    });
    capture.scopes.push(SourceProcessingScope::OutOfFlowBlock {
        atomic: 0,
        trim: WhiteSpaceTrim::default(),
        base_style: style(WhiteSpace::Preserve, 10.0),
        used_direction: ParagraphBaseDirection::LeftToRight,
    });
    capture
        .paragraphs
        .push(facts(ParagraphBaseDirection::LeftToRight));
    use SourceProcessingEvent::*;
    capture.events = vec![
        EnterScope(0),
        EnterParagraph(0),
        Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        Atomic {
            atomic: 0,
            affinity: Affinity::After,
        },
        EnterScope(1),
        EnterParagraph(1),
        Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        ExitParagraph(1),
        ExitScope(1),
        Connection(0),
        Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        Control {
            control: 0,
            affinity: Affinity::After,
        },
        ExitParagraph(0),
        ExitScope(0),
    ];
    let processed = Arc::new(process_contextual_source(capture).unwrap());
    let view = processed.paragraphs().next().unwrap();
    assert_eq!(
        view.source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(3, 3)]
    );
    let mut system = system();
    let p = prepare(&mut system, view);
    assert_eq!(p.terminal_edge(), ParagraphTerminalEdge::BlockEnd);
    let actual = line(&p);
    assert_eq!(
        (
            actual.feedback().advance,
            actual.measurements().max_content_contribution().advance()
        ),
        (20.0, 20.0)
    );
    assert_eq!(
        actual
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.range()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
}
