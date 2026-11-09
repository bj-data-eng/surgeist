//! One captured request advances through preparation, incidence and sequencing.
//! Only this owner assembles the final result; all failed paths are cleanup-only.

#[path = "sequencing.rs"]
mod sequencing;
#[path = "validation.rs"]
mod validation;

#[path = "model.rs"]
pub(crate) mod model;
use crate::{
    BoxFacts, BoxId, PaintError, ResourceKind, StackingPlan,
    accounting::{Meter, PhaseMeter, apply, release, setup_error},
    algorithm::{self, CoreCleanup, PreparationFailure, PreparedCore, StackingPublication},
};
use model::*;
use std::{
    mem::{size_of, take},
    ops::ControlFlow::{self, Break, Continue},
};
use surgeist_ledger::{Change, Scope};

struct PreparedPaint<'a> {
    request: PaintRequest<'a>,
    core: PreparedCore<'a>,
}

impl<'a> PreparedPaint<'a> {
    fn new(
        request: PaintRequest<'a>,
        meter: &mut Meter,
        root: Scope,
        scopes: &[Scope; 7],
    ) -> ControlFlow<PaintCleanup<'a>, Self> {
        match algorithm::prepare(meter, root, scopes, request.boxes(), request.root()) {
            Continue(core) => Continue(Self { request, core }),
            Break(PreparationFailure { error, cleanup }) => Break(PaintCleanup {
                core: cleanup,
                index: None,
                output: None,
                publication: Publication::Failed(error),
            }),
        }
    }
}

struct IndexedPaint<'a> {
    input: PreparedPaint<'a>,
    index: validation::PaintIndex,
}

struct SequencedPaint<'a> {
    input: IndexedPaint<'a>,
    output: sequencing::SequenceState,
}

impl<'a> SequencedPaint<'a> {
    fn publish(self, meter: &mut Meter, phase: Scope) -> PaintCleanup<'a> {
        let IndexedPaint {
            input: PreparedPaint { request, core },
            index,
        } = self.input;
        let mut publication = PaintPublication {
            request,
            core: core.into_publication(),
            index,
            sequence: self.output,
        };
        let root = publication.core.cleanup.root;
        // The callback borrows publication-only custody. Failed phase entry
        // leaves every backing here; fixed-owner refusal returns the actual
        // constructed plan together with its original admission error.
        let construction = meter.with_phase(phase, |p| {
            let plan = PaintOrder {
                request: publication.request,
                stacking: StackingPlan {
                    facts: publication.core.facts,
                    membership: take(&mut publication.core.cleanup.state.membership),
                    contexts: take(&mut publication.core.cleanup.state.contexts),
                    children: take(&mut publication.core.cleanup.state.children),
                },
                events: take(&mut publication.sequence.events),
            };
            let admission = (|| {
                let amount = u64::try_from(size_of::<PaintOrder<'_>>())
                    .map_err(|_| PaintError::CapacityOverflow)?;
                apply(
                    p,
                    root,
                    Change::Acquire(ResourceKind::RetainedResultBytes, amount),
                )
            })();
            (plan, admission)
        });
        PaintCleanup {
            core: publication.core.cleanup,
            index: Some(publication.index),
            output: Some(publication.sequence),
            publication: match construction {
                Ok((plan, Ok(()))) => Publication::Admitted(plan),
                Ok((plan, Err(error))) => Publication::Rejected { error, plan },
                Err(error) => Publication::Failed(setup_error(error)),
            },
        }
    }
}

/// A sequenced request has consumed its ready owner. Only final construction
/// and terminal cleanup remain available, with the original core source intact.
struct PaintPublication<'a> {
    request: PaintRequest<'a>,
    core: StackingPublication<'a>,
    index: validation::PaintIndex,
    sequence: sequencing::SequenceState,
}

enum Publication<'a> {
    Admitted(PaintOrder<'a>),
    Rejected {
        error: PaintError,
        plan: PaintOrder<'a>,
    },
    Failed(PaintError),
}

/// This terminal owner can release custody, but cannot prepare, validate,
/// sequence or issue observations of unfinished state.
struct PaintCleanup<'a> {
    core: CoreCleanup,
    index: Option<validation::PaintIndex>,
    output: Option<sequencing::SequenceState>,
    publication: Publication<'a>,
}

impl<'a> PaintCleanup<'a> {
    fn validation_failure(input: PreparedPaint<'a>, error: PaintError) -> Self {
        Self {
            core: input.core.into_cleanup(),
            index: None,
            output: None,
            publication: Publication::Failed(error),
        }
    }

    fn sequencing_failure(
        input: IndexedPaint<'a>,
        output: sequencing::SequenceState,
        error: PaintError,
    ) -> Self {
        Self {
            core: input.input.core.into_cleanup(),
            index: Some(input.index),
            output: Some(output),
            publication: Publication::Failed(error),
        }
    }

    fn cleanup(mut self, p: &mut PhaseMeter<'_, '_>) -> Result<PaintOrder<'a>, PaintError> {
        let root = self.core.root;
        let scratch = (|| {
            self.core.scratch_cleanup(p)?;
            if let Some(index) = self.index {
                index.cleanup(p, root)?;
            }
            if let Some(output) = self.output.as_mut() {
                output.scratch_cleanup(p, root)?;
            }
            Ok(())
        })();
        match self.publication {
            Publication::Admitted(plan) => scratch.map(|()| plan),
            Publication::Rejected { error, plan } => {
                let released = scratch.and_then(|()| {
                    self.core.result_cleanup(p)?;
                    let PaintOrder {
                        stacking, events, ..
                    } = plan;
                    release(
                        p,
                        root,
                        stacking.membership,
                        ResourceKind::RetainedResultBytes,
                    )?;
                    release(
                        p,
                        root,
                        stacking.contexts,
                        ResourceKind::RetainedResultBytes,
                    )?;
                    release(
                        p,
                        root,
                        stacking.children,
                        ResourceKind::RetainedResultBytes,
                    )?;
                    release(p, root, events, ResourceKind::RetainedResultBytes)?;
                    if let Some(output) = self.output {
                        output.result_cleanup(p, root)?;
                    }
                    Ok(())
                });
                Err(error).and_then(|value| released.map(|()| value))
            }
            Publication::Failed(error) => {
                let released = scratch.and_then(|()| {
                    self.core.result_cleanup(p)?;
                    if let Some(output) = self.output {
                        output.result_cleanup(p, root)?;
                    }
                    Ok(())
                });
                Err(error).and_then(|value| released.map(|()| value))
            }
        }
    }
}

pub(crate) fn plan<'a>(
    request: PaintRequest<'a>,
    meter: &mut Meter,
    root: Scope,
    scopes: &[Scope; 7],
    validate: Scope,
    sequence: Scope,
) -> Result<PaintOrder<'a>, PaintError> {
    let completion = 'phases: {
        let input = match PreparedPaint::new(request, meter, root, scopes) {
            Continue(input) => input,
            Break(failure) => break 'phases failure,
        };
        let input = match validation::prepare(input, meter, root, validate) {
            Continue(input) => input,
            Break(failure) => break 'phases failure,
        };
        let sequenced = match sequencing::run(input, meter, root, sequence) {
            Continue(sequenced) => sequenced,
            Break(failure) => break 'phases failure,
        };
        sequenced.publish(meter, scopes[5])
    };
    meter
        .with_phase(scopes[6], |p| completion.cleanup(p))
        .map_err(setup_error)
        .and_then(|done| done)
}

/// Complete validated ordering borrowing its exact request. Actual contexts are
/// nested atomic intervals; these events are not a coordinate/clip tree or a
/// backend command stream. The result deliberately does not implement Clone.
#[derive(Debug)]
pub struct PaintOrder<'a> {
    request: PaintRequest<'a>,
    stacking: StackingPlan<'a>,
    events: Vec<OrderEvent>,
}
impl PaintOrder<'_> {
    pub const fn request(&self) -> PaintRequest<'_> {
        self.request
    }
    pub const fn stacking(&self) -> &StackingPlan<'_> {
        &self.stacking
    }
    pub fn events(&self) -> impl ExactSizeIterator<Item = PaintOrderEvent<'_>> {
        self.events.iter().map(|event| match *event {
            OrderEvent::BeginContext(id) => {
                PaintOrderEvent::BeginContext(BoxView { plan: self, id })
            }
            OrderEvent::Draw(id) => PaintOrderEvent::Draw(PrimitiveView { plan: self, id }),
            OrderEvent::EndContext(id) => PaintOrderEvent::EndContext(BoxView { plan: self, id }),
        })
    }
    pub fn box_view(&self, id: BoxId) -> Option<BoxView<'_>> {
        self.request
            .boxes()
            .get(id.get())
            .map(|_| BoxView { plan: self, id })
    }
    pub fn fragment_view(&self, id: FragmentId) -> Option<FragmentView<'_>> {
        self.request
            .fragments()
            .get(id.get())
            .map(|_| FragmentView { plan: self, id })
    }
    pub fn line_view(&self, id: LineId) -> Option<LineView<'_>> {
        self.request
            .lines()
            .get(id.get())
            .map(|_| LineView { plan: self, id })
    }
    pub fn primitive_view(&self, id: PrimitiveId) -> Option<PrimitiveView<'_>> {
        self.request
            .primitives()
            .get(id.get())
            .map(|_| PrimitiveView { plan: self, id })
    }
}

/// Borrowed observations, bound to the validated plan that issued their views.
#[derive(Clone, Copy, Debug)]
pub enum PaintOrderEvent<'plan> {
    BeginContext(BoxView<'plan>),
    Draw(PrimitiveView<'plan>),
    EndContext(BoxView<'plan>),
}

#[derive(Clone, Copy, Debug)]
pub struct BoxView<'plan> {
    plan: &'plan PaintOrder<'plan>,
    id: BoxId,
}
impl<'plan> BoxView<'plan> {
    pub const fn id(self) -> BoxId {
        self.id
    }
    pub fn facts(self) -> &'plan BoxFacts {
        &self.plan.request.boxes()[self.id.get()]
    }
    pub fn paint(self) -> &'plan BoxPaint {
        &self.plan.request.box_paint()[self.id.get()]
    }
    pub fn parent(self) -> Option<Self> {
        self.facts().parent().map(|id| Self {
            plan: self.plan,
            id,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FragmentView<'plan> {
    plan: &'plan PaintOrder<'plan>,
    id: FragmentId,
}
impl<'plan> FragmentView<'plan> {
    pub const fn id(self) -> FragmentId {
        self.id
    }
    pub fn record(self) -> &'plan PaintFragment {
        &self.plan.request.fragments()[self.id.get()]
    }
    pub fn owner(self) -> BoxView<'plan> {
        let id = match self.record().owner() {
            FragmentOwner::Box(id) => id,
            FragmentOwner::Text(id) => self.plan.request.texts()[id.get()].parent(),
        };
        BoxView {
            plan: self.plan,
            id,
        }
    }
    pub fn text_source(self) -> Option<&'plan TextSource> {
        match self.record().owner() {
            FragmentOwner::Box(_) => None,
            FragmentOwner::Text(id) => Some(&self.plan.request.texts()[id.get()]),
        }
    }
    pub fn parent(self) -> Option<Self> {
        self.record().parent().map(|id| Self {
            plan: self.plan,
            id,
        })
    }
    pub fn line(self) -> Option<LineView<'plan>> {
        self.record().line().map(|id| LineView {
            plan: self.plan,
            id,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LineView<'plan> {
    plan: &'plan PaintOrder<'plan>,
    id: LineId,
}
impl<'plan> LineView<'plan> {
    pub const fn id(self) -> LineId {
        self.id
    }
    pub fn record(self) -> &'plan PaintLine {
        &self.plan.request.lines()[self.id.get()]
    }
    pub fn container(self) -> FragmentView<'plan> {
        FragmentView {
            plan: self.plan,
            id: self.record().container(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrimitiveView<'plan> {
    plan: &'plan PaintOrder<'plan>,
    id: PrimitiveId,
}
impl<'plan> PrimitiveView<'plan> {
    pub const fn id(self) -> PrimitiveId {
        self.id
    }
    pub fn record(self) -> &'plan PaintPrimitive {
        &self.plan.request.primitives()[self.id.get()]
    }
    pub fn fragment(self) -> Option<FragmentView<'plan>> {
        self.record().fragment_id().map(|id| FragmentView {
            plan: self.plan,
            id,
        })
    }
    pub fn owner(self) -> BoxView<'plan> {
        self.fragment().map_or(
            BoxView {
                plan: self.plan,
                id: self.plan.request.root(),
            },
            FragmentView::owner,
        )
    }
    pub fn applying_box(self) -> Option<BoxView<'plan>> {
        self.record().applying_box().map(|id| BoxView {
            plan: self.plan,
            id,
        })
    }
}
