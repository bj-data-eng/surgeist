use super::*;
use crate::ResourceKind;
use crate::{
    BoxFacts, BoxId, ChildOrder, ContextEffect, PaintChange, PaintLimits, PaintPhase,
    Participation, Position, ProfileMode, WorkKind, ZIndex,
};
use std::{cell::Cell, time::Duration};
use surgeist_layout::SourceIndex;

struct Fixture {
    boxes: Vec<BoxFacts>,
    paint: Vec<BoxPaint>,
    texts: Vec<TextSource>,
    lines: Vec<PaintLine>,
    fragments: Vec<PaintFragment>,
    primitives: Vec<PaintPrimitive>,
}
impl Fixture {
    fn request(&self) -> PaintRequest<'_> {
        PaintRequest::new(
            &self.boxes,
            &self.paint,
            &self.texts,
            &self.lines,
            &self.fragments,
            &self.primitives,
            BoxId::new(0),
        )
    }
}
fn fixture() -> Fixture {
    let make = |parent, source, position, level| {
        BoxFacts::new(
            parent,
            SourceIndex::new(source),
            ChildOrder::Source,
            Participation::Ordinary,
            position,
            level,
            ContextEffect::None,
        )
    };
    let boxes = vec![
        make(None, 0, Position::Static, ZIndex::Auto),
        make(
            Some(BoxId::new(0)),
            0,
            Position::Relative,
            ZIndex::Integer(-2),
        ),
        make(Some(BoxId::new(0)), 1, Position::Static, ZIndex::Auto),
        make(Some(BoxId::new(0)), 2, Position::Static, ZIndex::Auto),
        make(Some(BoxId::new(0)), 3, Position::Static, ZIndex::Auto),
        make(Some(BoxId::new(0)), 4, Position::Relative, ZIndex::Auto),
        make(
            Some(BoxId::new(0)),
            5,
            Position::Relative,
            ZIndex::Integer(0),
        ),
        make(
            Some(BoxId::new(0)),
            6,
            Position::Relative,
            ZIndex::Integer(3),
        ),
    ];
    let mut paint = vec![BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None); 8];
    paint[3] = BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating);
    paint[4] = BoxPaint::new(BoxPaintKind::Inline, FloatParticipation::None);
    let mut fragments = vec![PaintFragment::box_fragment(BoxId::new(0), None, None, 0)];
    for owner in 1..8 {
        fragments.push(PaintFragment::box_fragment(
            BoxId::new(owner),
            Some(FragmentId::new(0)),
            (owner == 4).then(|| LineId::new(0)),
            0,
        ));
    }
    fragments.push(PaintFragment::text_fragment(
        TextId::new(0),
        FragmentId::new(4),
        LineId::new(0),
        0,
    ));
    let mut primitives = vec![PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0)];
    for id in 1..8 {
        primitives.push(PaintPrimitive::fragment(
            FragmentId::new(id),
            FragmentPaintKind::BackgroundColor,
            0,
        ));
    }
    primitives.push(PaintPrimitive::fragment(
        FragmentId::new(8),
        FragmentPaintKind::Text,
        0,
    ));
    primitives.push(PaintPrimitive::fragment(
        FragmentId::new(0),
        FragmentPaintKind::Outline,
        0,
    ));
    primitives.push(PaintPrimitive::decoration(
        FragmentId::new(8),
        BoxId::new(0),
        DecorationKind::Underline,
        0,
    ));
    primitives.push(PaintPrimitive::decoration(
        FragmentId::new(8),
        BoxId::new(4),
        DecorationKind::Underline,
        0,
    ));
    Fixture {
        boxes,
        paint,
        fragments,
        primitives,
        texts: vec![TextSource::new(BoxId::new(4), SourceIndex::new(0))],
        lines: vec![PaintLine::new(FragmentId::new(0), 0)],
    }
}
fn options(limits: PaintLimits) -> PaintOptions {
    PaintOptions::new(limits, ProfileMode::Summary { timing: false })
}
fn events(plan: &PaintOrder<'_>) -> Vec<(char, usize)> {
    plan.events()
        .map(|event| match event {
            PaintOrderEvent::BeginContext(view) => ('[', view.id().get()),
            PaintOrderEvent::Draw(view) => ('d', view.id().get()),
            PaintOrderEvent::EndContext(view) => (']', view.id().get()),
        })
        .collect()
}

#[test]
fn literal_counts_and_all_exact_one_under_limits_preserve_atomic_output_and_retry() {
    let fixture = fixture();
    let baseline = plan_paint_order(fixture.request(), options(PaintLimits::unlimited()));
    let expected = events(&baseline.result.unwrap());
    let report = baseline.report.unwrap();
    assert_eq!(report.work(WorkKind::BoxValidation), 8);
    assert_eq!(report.work(WorkKind::ParentEdge), 7);
    assert_eq!(report.work(WorkKind::MembershipVisit), 8);
    // Root's seven ordered siblings: merge comparisons 3+4+4, then six
    // distinct-source checks. Three child contexts take 1+2 comparisons.
    assert_eq!(report.work(WorkKind::SiblingComparison), 17);
    assert_eq!(report.work(WorkKind::ContextComparison), 3);
    // Eight initialized membership slots, eight assigned memberships, four
    // contexts, three child references and four context ranges =27. Twelve
    // draws and eight context boundaries add20 actual sequence records.
    assert_eq!(report.work(WorkKind::OutputRecord), 47);
    // This ordering fixture exercises every ordering dimension. Gradient work
    // belongs to its separate operation and must stay zero even with zero caps.
    let ordering_work = [
        WorkKind::BoxValidation,
        WorkKind::ParentEdge,
        WorkKind::TopologyVisit,
        WorkKind::SiblingComparison,
        WorkKind::MembershipVisit,
        WorkKind::ContextComparison,
        WorkKind::OrderMove,
        WorkKind::OutputRecord,
        WorkKind::PaintValidation,
        WorkKind::FragmentVisit,
        WorkKind::LineVisit,
        WorkKind::InlineMemberVisit,
        WorkKind::PrimitiveVisit,
        WorkKind::PaintScopeVisit,
        WorkKind::PaintOrderComparison,
        WorkKind::DecorationComparison,
    ];
    let mut exact = PaintLimits::unlimited();
    for kind in ordering_work {
        assert!(report.work(kind) > 0, "fixture must exercise {kind:?}");
        exact = exact.with_work_limit(kind, report.work(kind));
    }
    for kind in [
        WorkKind::GradientValidation,
        WorkKind::GradientDirection,
        WorkKind::GradientCoordinate,
        WorkKind::GradientStopVisit,
    ] {
        assert_eq!(report.work(kind), 0);
        exact = exact.with_work_limit(kind, 0);
    }
    for &kind in ResourceKind::ALL {
        exact = exact.with_resource_limit(kind, report.peak(kind));
    }
    assert_eq!(
        events(
            &plan_paint_order(fixture.request(), options(exact))
                .result
                .unwrap()
        ),
        expected
    );
    for kind in ordering_work {
        let limit = report.work(kind) - 1;
        let outcome = plan_paint_order(
            fixture.request(),
            options(exact.with_work_limit(kind, limit)),
        );
        assert!(
            matches!(outcome.result, Err(PaintError::Budget(_))),
            "{kind:?}"
        );
        let refused = outcome.report.unwrap();
        assert!(refused.work(kind) <= limit);
        for &resource in ResourceKind::ALL {
            assert_eq!(refused.live(resource), 0, "{kind:?}");
        }
    }
    for &kind in ResourceKind::ALL {
        let outcome = plan_paint_order(
            fixture.request(),
            options(exact.with_resource_limit(kind, report.peak(kind) - 1)),
        );
        assert!(
            matches!(outcome.result, Err(PaintError::Budget(_))),
            "{kind:?}"
        );
        let refused = outcome.report.unwrap();
        for &resource in ResourceKind::ALL {
            assert_eq!(refused.live(resource), 0);
        }
    }
    assert_eq!(
        events(
            &plan_paint_order(fixture.request(), options(exact))
                .result
                .unwrap()
        ),
        expected
    );
}

#[test]
fn final_owner_refusal_publishes_no_order_after_the_complete_staged_sequence() {
    let fixture = fixture();
    let baseline = plan_paint_order(fixture.request(), options(PaintLimits::unlimited()));
    drop(baseline.result.unwrap());
    let report = baseline.report.unwrap();
    let fixed = std::mem::size_of::<PaintOrder<'_>>() as u64;
    let outcome = plan_paint_order(
        fixture.request(),
        options(PaintLimits::unlimited().with_resource_limit(
            ResourceKind::RetainedResultBytes,
            report.live(ResourceKind::RetainedResultBytes) - 1,
        )),
    );
    let Err(PaintError::Budget(refusal)) = outcome.result else {
        panic!("final returned owner must be admitted");
    };
    assert_eq!(
        refusal.change,
        PaintChange::Acquire {
            kind: ResourceKind::RetainedResultBytes,
            amount: fixed
        }
    );
    assert_eq!(
        refusal.current,
        report.live(ResourceKind::RetainedResultBytes) - fixed
    );
    let refused = outcome.report.unwrap();
    assert_eq!(refused.work(WorkKind::OutputRecord), 47);
    for &kind in ResourceKind::ALL {
        assert_eq!(refused.live(kind), 0);
    }
    assert_eq!(
        refused
            .phases()
            .find(|phase| phase.phase() == PaintPhase::Publication)
            .unwrap()
            .invocations(),
        1
    );
}

thread_local! {static READS:Cell<u64>=const{Cell::new(0)};}
fn clock() -> Duration {
    READS.with(|reads| {
        let now = reads.get();
        reads.set(now + 1);
        Duration::from_nanos(now)
    })
}
fn forbidden_clock() -> Duration {
    panic!("Off and untimed operations must not read a clock")
}

#[test]
fn fixed_phase_timing_detail_loss_and_off_untimed_preserve_work_and_outcomes() {
    let fixture = fixture();
    READS.with(|reads| reads.set(0));
    let timed = plan_paint_order(
        fixture.request(),
        PaintOptions::new(
            PaintLimits::unlimited(),
            ProfileMode::Summary { timing: true },
        )
        .with_clock(clock),
    );
    let expected = events(&timed.result.unwrap());
    let baseline = timed.report.unwrap();
    assert_eq!(baseline.phases().len(), 10);
    for phase in baseline.phases().skip(1) {
        assert_eq!(phase.invocations(), 1);
        assert_eq!(phase.elapsed(), Some(Duration::from_nanos(1)));
    }
    assert_eq!(READS.with(Cell::get), 20);
    let mut exact_limits = PaintLimits::unlimited();
    for &kind in WorkKind::ALL {
        exact_limits = exact_limits.with_work_limit(kind, baseline.work(kind));
    }
    for &kind in ResourceKind::ALL {
        exact_limits = exact_limits.with_resource_limit(kind, baseline.peak(kind));
    }
    for profile in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: crate::EventFilter::All,
        },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 1,
            filter: crate::EventFilter::ScopeName("paint_sequence"),
        },
    ] {
        let outcome = plan_paint_order(
            fixture.request(),
            PaintOptions::new(exact_limits, profile).with_clock(forbidden_clock),
        );
        assert_eq!(events(&outcome.result.unwrap()), expected);
        if let Some(report) = outcome.report {
            assert_eq!(report.elapsed(), None);
            for &kind in WorkKind::ALL {
                assert_eq!(report.work(kind), baseline.work(kind));
            }
            for &kind in ResourceKind::ALL {
                assert_eq!(report.live(kind), baseline.live(kind));
                assert_eq!(report.peak(kind), baseline.peak(kind));
            }
        }
    }
}
