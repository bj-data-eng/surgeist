//! Font-independent UBA paragraph state at genuine structural frontiers.
//! UAX9 P1 gives each paragraph independent explicit-formatting state; X4 RLO
//! raises an LTR paragraph to odd level 1 and overrides subsequent body text.
//! Raw member markers alone are neither paragraph separators nor resets.

use crate::*;
use std::sync::Arc;

fn origin_event(origin: usize) -> SourceProcessingEvent {
    SourceProcessingEvent::Origin {
        origin,
        affinity: Affinity::After,
    }
}

fn request(child_block: bool) -> ContextualSourceInput {
    // Both controls have IDENTICAL authored UTF-16 and three genuine original
    // occurrences. No PDF, paragraph separator or invented boundary scalar is
    // inserted. Only the actual source/tree domain role of B differs.
    let mut input = super::input("\u{202e}ABA", WhiteSpace::Preserve);
    input.source = Source::identified("\u{202e}ABA", Id::from_u64(78473), 14);
    input.origins = [Range::new(0, 2), Range::new(2, 3), Range::new(3, 4)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    let used = SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    };
    input.paragraphs = vec![used; 3];
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        origin_event(0),
    ];
    if child_block {
        // A real ordinary Block cannot enter while its parent raw paragraph
        // is open. Close raw0, enter the child's own domain/raw1, then resume
        // the parent as a NEW raw2. The declared joint owns only parent0+2.
        input.scopes.push(SourceProcessingScope::Block {
            topology: SourceProcessingTopology::Unknown,
            trim: WhiteSpaceTrim::default(),
            base_style: super::style(WhiteSpace::Preserve, 10.0),
            used_direction: ParagraphBaseDirection::LeftToRight,
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
        });
        input.connections = vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 2,
            kind: ParagraphConnectionKind::Continuous,
        }];
        input.boundary_policies = vec![super::policy(ParagraphBoundaryDefinition::Connection(0))];
        input.events.extend([
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::EnterParagraph(1),
            origin_event(1),
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::EnterParagraph(2),
            // The edge is a real parent-domain neighbor relation at cursor3,
            // immediately before its actual after-origin, across the child hole.
            SourceProcessingEvent::Connection(0),
            origin_event(2),
        ]);
        input.joint_flows = vec![SourceProcessingJointFlow {
            scope: 0,
            paragraphs: vec![0, 2],
            facts_owner: 0,
        }];
    } else {
        input.connections = vec![
            SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: ParagraphConnectionKind::Continuous,
            },
            SourceProcessingConnection {
                before_origin: 1,
                after_origin: 2,
                kind: ParagraphConnectionKind::Continuous,
            },
        ];
        input.boundary_policies = (0..2)
            .map(|edge| super::policy(ParagraphBoundaryDefinition::Connection(edge)))
            .collect();
        input.events.extend([
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::EnterParagraph(1),
            origin_event(1),
            SourceProcessingEvent::Connection(1),
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::EnterParagraph(2),
            origin_event(2),
        ]);
        input.joint_flows = vec![SourceProcessingJointFlow {
            scope: 0,
            paragraphs: vec![0, 1, 2],
            facts_owner: 0,
        }];
    }
    input.events.extend([
        SourceProcessingEvent::ExitParagraph(2),
        SourceProcessingEvent::ExitScope(0),
    ]);
    input
}

fn captured(input: ContextualSourceInput) -> Arc<ProcessedSource> {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(
        processed.source().text().code_units(),
        [0x202e, 0x41, 0x42, 0x41]
    );
    assert_eq!(processed.processed_text(), &Text::from("\u{202e}ABA"));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        [0x202e, 0x41, 0x42, 0x41]
    );
    assert!(
        processed.source().boxes().is_empty(),
        "ordinary child Block, never an atomic substitute"
    );
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 3);
    for (i, (extent, text)) in [
        (Range::new(0, 2), "\u{202e}A"),
        (Range::new(2, 3), "B"),
        (Range::new(3, 4), "A"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            raw[i].captured_paragraph(),
            processed.captured_paragraph(i).unwrap()
        );
        assert_eq!(raw[i].source_pieces().collect::<Vec<_>>(), [extent]);
        assert_eq!(
            super::text(raw[i].plaintext_items(Affinity::After)),
            text.encode_utf16().collect::<Vec<_>>()
        );
        assert!(Arc::ptr_eq(raw[i].processed_source(), &processed));
    }
    processed
}

fn assert_atoms(
    analyzed: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    expected: &[(u32, Range)],
) {
    assert!(std::ptr::eq(
        analyzed.processed_source(),
        processed.as_ref()
    ));
    assert_eq!(analyzed.source(), processed.source());
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(analyzed.inner.analysis.levels.len(), expected.len());
    assert_eq!(analyzed.inner.analysis.base_level, 0);
    assert_eq!(
        analyzed.inner.paragraph.base_direction,
        Some(ParagraphBaseDirection::LeftToRight)
    );
    assert_eq!(analyzed.paragraph_strut(), ParagraphStrut::DefaultFont);
}

fn assert_originals(
    analyzed: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    origins: &[(usize, Range)],
) {
    assert_eq!(analyzed.inner.paragraph.origins.len(), origins.len());
    for (local, &(original, range)) in origins.iter().enumerate() {
        let row = &analyzed.inner.paragraph.origins[local];
        assert_eq!(row.aggregate, range);
        assert_eq!(row.local, range);
        assert_eq!(
            row.original(analyzed.processed_source()),
            processed.source()
        );
        let map = &analyzed.inner.paragraph.origin_map;
        assert_eq!(
            map.flow_origin(map.view(local).unwrap()).captured_event(),
            Some(processed.captured_origin(original).unwrap().index)
        );
    }
}

#[test]
fn an_actual_child_block_restarts_parent_bidi_after_an_unclosed_rlo() {
    let processed = captured(request(true));
    let root = processed.captured_scope(0).unwrap();
    let child_scope = processed.captured_scope(1).unwrap();
    assert_eq!(child_scope.parent(), Some(root.clone()));
    assert_eq!(
        child_scope.domain_definition(),
        Some(CapturedDomainDefinition::Block)
    );
    assert_eq!(processed.captured_paragraph(0).unwrap().domain(), root);
    assert_eq!(
        processed.captured_paragraph(1).unwrap().domain(),
        child_scope
    );
    assert_eq!(processed.captured_paragraph(2).unwrap().domain(), root);
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let parent = AnalyzedDomain::from_processed_paragraph(
        raw[2].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(Arc::ptr_eq(parent.processed_source(), &processed));
    assert_eq!(parent.domain(), root);
    assert_eq!(parent.outputs().len(), 1);
    let output = parent.outputs().next().unwrap();
    assert_eq!(output.inner.projection.text, "\u{202e}AA");
    assert_atoms(
        output,
        &processed,
        &[
            (0x202e, Range::new(0, 1)),
            (0x41, Range::new(1, 2)),
            (0x41, Range::new(3, 4)),
        ],
    );
    assert_originals(
        output,
        &processed,
        &[(0, Range::new(0, 2)), (2, Range::new(3, 4))],
    );
    let owner = output.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [
            processed.captured_paragraph(0).unwrap(),
            processed.captured_paragraph(2).unwrap()
        ]
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(owner.joint_flow(), processed.captured_joint_flow(0));
    // X4: the first body A is under RLO at odd level1. P1: the resumed
    // parent body starts fresh at the real Block frontier, so its A is level0.
    assert_eq!(output.inner.analysis.levels[1], 1);
    assert_eq!(output.inner.analysis.levels.last(), Some(&0));
    let child = AnalyzedDomain::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(child.domain(), child_scope);
    assert_eq!(child.outputs().len(), 1);
    let child_output = child.outputs().next().unwrap();
    assert_eq!(child_output.inner.projection.text, "B");
    assert_atoms(child_output, &processed, &[(0x42, Range::new(2, 3))]);
    assert_originals(child_output, &processed, &[(1, Range::new(2, 3))]);
    assert_eq!(child_output.inner.analysis.levels.last(), Some(&0));
    assert_eq!(
        child_output
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [processed.captured_paragraph(1).unwrap()]
    );
}

#[test]
fn raw_member_seams_do_not_restart_an_open_rlo_in_one_joint_output() {
    let processed = captured(request(false));
    let root = processed.captured_scope(0).unwrap();
    for ordinal in 0..3 {
        assert_eq!(
            processed.captured_paragraph(ordinal).unwrap().domain(),
            root
        );
    }
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().nth(2).unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.inner.projection.text, "\u{202e}ABA");
    assert_atoms(
        output,
        &processed,
        &[
            (0x202e, Range::new(0, 1)),
            (0x41, Range::new(1, 2)),
            (0x42, Range::new(2, 3)),
            (0x41, Range::new(3, 4)),
        ],
    );
    assert_originals(
        output,
        &processed,
        &[
            (0, Range::new(0, 2)),
            (1, Range::new(2, 3)),
            (2, Range::new(3, 4)),
        ],
    );
    let owner = output.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        (0..3)
            .map(|ordinal| processed.captured_paragraph(ordinal).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(owner.joint_flow(), processed.captured_joint_flow(0));
    // Three storage/raw records, but one uninterrupted source paragraph: the
    // unclosed override still owns the final A. Resetting at either raw seam
    // would incorrectly yield level0 instead of this independently derived1.
    assert_eq!(output.inner.analysis.levels[1], 1);
    assert_eq!(output.inner.analysis.levels.last(), Some(&1));
}
