//! Real paragraphs share capture facts while retaining local executed cuts.

use super::*;
use crate::{CapturedScopeIncidence as R, SourceProcessingEvent as E};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn inline() -> crate::SourceProcessingScope {
    crate::SourceProcessingScope::Inline {
        trim: Default::default(),
        style: None,
        boundaries: None,
    }
}

// Each paragraph has identical body/support: L owns "A ", R owns "B".
// An outer inline begins before EnterParagraph and ends after ExitParagraph.
// Padding consists of genuine empty scopes in the LAST paragraph only.
fn capture(paragraph_count: usize, padding_scopes: usize) -> Arc<crate::ProcessedSource> {
    assert!(paragraph_count >= 2);
    let source = Source::new("A B".repeat(paragraph_count));
    let mut scopes = vec![crate::SourceProcessingScope::Block {
        topology: crate::SourceProcessingTopology::Complete,
        trim: Default::default(),
        base_style: Style {
            white_space: WhiteSpace::Preserve,
            wrap: crate::Wrap::Word,
            ..style()
        },
        used_direction: ParagraphBaseDirection::LeftToRight,
    }];
    for _ in 0..3 * paragraph_count + padding_scopes {
        scopes.push(inline());
    }
    let mut origins = Vec::new();
    let mut connections = Vec::new();
    let mut events = vec![E::EnterScope(0)];
    for p in 0..paragraph_count {
        let first = 2 * p;
        let outer = 1 + 3 * p;
        for (a, b) in [(3 * p, 3 * p + 2), (3 * p + 2, 3 * p + 3)] {
            origins.push(crate::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: crate::Range::new(a, b),
                aggregate: crate::Range::new(a, b),
                kind: crate::ParagraphOriginKind::Authored,
            });
        }
        connections.push(crate::SourceProcessingConnection {
            before_origin: first,
            after_origin: first + 1,
            kind: crate::ParagraphConnectionKind::Continuous,
        });
        events.extend([E::EnterScope(outer), E::EnterParagraph(p)]);
        if p + 1 == paragraph_count {
            for scope in 1 + 3 * paragraph_count..scopes.len() {
                events.extend([E::EnterScope(scope), E::ExitScope(scope)]);
            }
        }
        events.extend([
            E::EnterScope(outer + 1),
            E::Origin {
                origin: first,
                affinity: crate::Affinity::After,
            },
            E::ExitScope(outer + 1),
            E::Connection(p),
            E::EnterScope(outer + 2),
            E::Origin {
                origin: first + 1,
                affinity: crate::Affinity::After,
            },
            E::ExitScope(outer + 2),
            E::ExitParagraph(p),
            E::ExitScope(outer),
        ]);
    }
    events.push(E::ExitScope(0));
    Arc::new(
        crate::process_contextual_source(crate::ContextualSourceInput {
            source,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            envelope: crate::SourceProcessingEnvelope::Complete,
            origins,
            connections,
            controls: vec![],
            boundary_policies: (0..paragraph_count)
                .map(|p| crate::ParagraphBoundaryPolicy {
                    definition: crate::ParagraphBoundaryDefinition::Connection(p),
                    preceding_wrap: crate::Wrap::Word,
                    following_wrap: crate::Wrap::Word,
                    ancestor_wrap: crate::Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes,
            paragraphs: (0..paragraph_count)
                .map(|_| crate::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: ParagraphStrut::DefaultFont,
                })
                .collect(),
            events,
        })
        .unwrap(),
    )
}

fn prepare(
    fonts: &mut System,
    capture: &Arc<crate::ProcessedSource>,
    count: usize,
) -> Vec<(AnalyzedSource, PreparedText)> {
    capture
        .paragraphs()
        .take(count)
        .map(|p| {
            assert!(Arc::ptr_eq(p.processed_source(), capture));
            let a = AnalyzedSource::from_processed_paragraph(p).unwrap();
            let policies = a
                .wrapping_relations()
                .filter(|r| r.ancestor_wrap().is_none())
                .map(|r| crate::WrappingPolicy {
                    relation: r.handle(),
                    ancestor_wrap: crate::Wrap::Word,
                })
                .collect();
            let a = a.with_wrapping_policies(policies).unwrap();
            let p = fonts.prepare(&a, PreparationOptions::default()).unwrap();
            (a, p)
        })
        .collect()
}

fn soft(p: &PreparedText, paragraph: usize) -> PreparedBoundary {
    let inventory: Vec<_> = p
        .opportunity_views()
        .map(|o| (o.kind(), o.boundary().view().source_before().get()))
        .collect();
    assert_eq!(
        inventory,
        [
            (PreparedBreakKind::Allowed, 3 * paragraph + 2),
            (PreparedBreakKind::ParagraphEnd, 3 * paragraph + 3),
        ]
    );
    p.opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Allowed)
        .unwrap()
        .boundary()
}

fn query(capture: &Arc<crate::ProcessedSource>, p: &PreparedText, paragraph: usize) -> Duration {
    let b = soft(p, paragraph);
    let started = Instant::now();
    let incidences = p.boundary_incidences(&b).unwrap();
    let query_time = started.elapsed();
    assert_eq!(incidences.len(), 1);
    assert_eq!(incidences[0].boundary(), b);
    for (scope, expected) in [
        (1 + 3 * paragraph, R::Inside),
        (2 + 3 * paragraph, R::AtAfter),
        (3 + 3 * paragraph, R::AtBefore),
    ] {
        assert_eq!(
            p.incidence_scope_relation(&incidences[0], &capture.captured_scope(scope).unwrap())
                .unwrap(),
            expected
        );
    }
    // Query distant same-domain scopes as well as both immediate neighbors.
    // Sharing or local sparse storage must not turn these into WrongDomain.
    let paragraphs = capture.paragraphs().count();
    for other in [
        0,
        paragraph.saturating_sub(1),
        paragraph + 1,
        paragraphs - 1,
    ] {
        if other >= paragraphs || other == paragraph {
            continue;
        }
        let expected = if other < paragraph {
            R::After
        } else {
            R::Before
        };
        for scope in 1 + 3 * other..=3 + 3 * other {
            assert_eq!(
                p.incidence_scope_relation(&incidences[0], &capture.captured_scope(scope).unwrap())
                    .unwrap(),
                expected
            );
        }
    }
    assert_eq!(p.boundary_incidences(&b).unwrap(), incidences);
    query_time
}

fn retained(paragraphs: &[(AnalyzedSource, PreparedText)]) -> usize {
    let mut census = crate::engine::analysis::scope::IndexPayloadCensus::default();
    for (analyzed, _) in paragraphs {
        census.analyzed(&analyzed.inner.core);
    }
    census.bytes()
}

#[derive(Debug)]
struct Sample {
    paragraphs: usize,
    padding_scopes: usize,
    one: usize,
    eight: usize,
    thirty_two: Option<usize>,
}

fn sample(paragraph_count: usize, padding_scopes: usize) -> Sample {
    let started = Instant::now();
    let capture = capture(paragraph_count, padding_scopes);
    let capture_time = started.elapsed();
    let started = Instant::now();
    let mut fonts = system();
    let font_setup_time = started.elapsed();
    let started = Instant::now();
    let paragraphs = prepare(&mut fonts, &capture, paragraph_count.min(32));
    let preparation_time = started.elapsed();
    // All capture/analysis/font preparation precedes the cold query census.
    let before = retained(&paragraphs);
    let first_query_time = query(&capture, &paragraphs[0].1, 0);
    let one = retained(&paragraphs) - before;
    let mut following_seven_time = Duration::ZERO;
    for (i, (_, p)) in paragraphs.iter().enumerate().take(8).skip(1) {
        following_seven_time += query(&capture, p, i);
    }
    let eight = retained(&paragraphs) - before;
    let mut following_twenty_four_time = Duration::ZERO;
    let thirty_two = if paragraphs.len() == 32 {
        for (i, (_, p)) in paragraphs.iter().enumerate().skip(8) {
            following_twenty_four_time += query(&capture, p, i);
        }
        Some(retained(&paragraphs) - before)
    } else {
        None
    };
    let warm_before = retained(&paragraphs);
    let queried = if thirty_two.is_some() { 32 } else { 8 };
    let mut warm_time = Duration::ZERO;
    for (i, (_, p)) in paragraphs.iter().enumerate().take(queried) {
        // Clones retain the same preparation; returned handles are dropped
        // before counting retained source-owned payload.
        warm_time += query(&capture, &p.clone(), i);
    }
    assert_eq!(retained(&paragraphs), warm_before);
    eprintln!(
        "P={paragraph_count},K={padding_scopes},Q={queried}: capture={capture_time:?},font_setup={font_setup_time:?},analysis_and_preparation={preparation_time:?},first_query={first_query_time:?},following_seven_queries={following_seven_time:?},following_twenty_four_queries={following_twenty_four_time:?},warm_queries={warm_time:?},R1={one},R8={eight},R32={thirty_two:?}"
    );
    Sample {
        paragraphs: paragraph_count,
        padding_scopes,
        one,
        eight,
        thirty_two,
    }
}

#[test]
fn additional_paragraph_queries_do_not_retain_remote_capture_payload_again() {
    let small = sample(16, 0);
    let medium = sample(128, 0);
    let large = sample(1024, 0);
    let padding = sample(16, 1024);
    eprintln!("retained incidence payload: {small:?} {medium:?} {large:?} {padding:?}");
    // Queries 2..8 have identical executed events, supports and local scopes.
    // Subtracting query 1 permits a one-time capture-wide shared allocation.
    let local_seven = small.eight - small.one;
    for actual in [&medium, &large, &padding] {
        assert!(
            actual.eight - actual.one <= local_seven,
            "remote capture growth increased seven identical local indexes: {actual:?}; reference={small:?}"
        );
    }
    assert!(
        large.thirty_two.unwrap() - large.eight <= medium.thirty_two.unwrap() - medium.eight,
        "remote capture growth increased twenty-four identical local indexes: {large:?}; reference={medium:?}"
    );
    assert_eq!(small.paragraphs, 16);
    assert_eq!(padding.padding_scopes, 1024);
}

#[test]
fn shared_capture_keeps_local_global_cut_and_foreign_preparation_custody() {
    let capture = capture(16, 0);
    let mut fonts = system();
    let paragraphs = prepare(&mut fonts, &capture, 16);
    for (i, (_, p)) in paragraphs.iter().enumerate() {
        let _ = query(&capture, p, i);
        let scopes: Vec<_> = (1 + 3 * i..=3 + 3 * i)
            .map(|scope| capture.captured_scope(scope).unwrap())
            .collect();
        // EnterParagraph separates OuterBefore from LeftBefore, and
        // ExitParagraph separates RightAfter from OuterAfter. No original
        // text/control pin chooses either side of those genuine event edges.
        for (first, expected) in [
            (
                true,
                [
                    vec![R::AtBefore, R::Before, R::Before],
                    vec![R::Inside, R::AtBefore, R::Before],
                ],
            ),
            (
                false,
                [
                    vec![R::Inside, R::After, R::AtAfter],
                    vec![R::AtAfter, R::After, R::After],
                ],
            ),
        ] {
            let b = if first {
                p.boundary_views().next().unwrap().handle()
            } else {
                p.boundary_views().last().unwrap().handle()
            };
            let incidences = p.boundary_incidences(&b).unwrap();
            let actual: Vec<Vec<_>> = incidences
                .iter()
                .map(|incidence| {
                    assert_eq!(incidence.boundary(), b);
                    scopes
                        .iter()
                        .map(|scope| p.incidence_scope_relation(incidence, scope).unwrap())
                        .collect()
                })
                .collect();
            assert_eq!(
                actual, expected,
                "complete ordered [outer,left,right] sentinel inventory; paragraph={i}, first={first}"
            );
        }
    }
    let original = &paragraphs[0].1;
    let other = &prepare(&mut fonts, &capture, 1)[0].1;
    let b = soft(original, 0);
    let incidence = original.boundary_incidences(&b).unwrap().remove(0);
    let scope = capture.captured_scope(1).unwrap();
    assert!(matches!(
        reason(&other.boundary_incidences(&b).unwrap_err()),
        ProtocolError::ForeignPreparation {
            kind: ProtocolHandleKind::Boundary,
            ..
        }
    ));
    assert!(matches!(
        reason(
            &other
                .incidence_scope_relation(&incidence, &scope)
                .unwrap_err()
        ),
        ProtocolError::ForeignPreparation {
            kind: ProtocolHandleKind::BoundaryIncidence,
            ..
        }
    ));
    assert_eq!(
        original
            .incidence_scope_relation(&incidence, &scope)
            .unwrap(),
        R::Inside
    );
}
