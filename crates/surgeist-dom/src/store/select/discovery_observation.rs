//! Scoped test observations of actual discovery, separate from participant work.
use super::super::scale_metrics;
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Counts {
    pub options: usize,
    pub option_candidates: usize,
    pub option_members: usize,
    pub targets: usize,
    pub target_candidates: usize,
    pub target_members: usize,
    pub selection_reads: usize,
    pub captures: usize,
    pub capture_nodes: usize,
    pub capture_edges: usize,
    pub map_comparisons: usize,
}
pub(super) enum Event {
    OptionsEntered,
    OptionCandidate,
    OptionMember,
    TargetsEntered,
    TargetCandidate,
    TargetMember,
    SelectionRead,
    CaptureEntered,
    CaptureNode,
    CaptureEdge,
    MapCompared,
}
thread_local! { static ACTIVE: Cell<Option<Counts>> = const { Cell::new(None) }; }
pub(super) fn record(event: Event) {
    if !scale_metrics::ENABLED {
        return;
    }
    ACTIVE.with(|active| {
        if let Some(mut counts) = active.get() {
            let count = match event {
                Event::OptionsEntered => &mut counts.options,
                Event::OptionCandidate => &mut counts.option_candidates,
                Event::OptionMember => &mut counts.option_members,
                Event::TargetsEntered => &mut counts.targets,
                Event::TargetCandidate => &mut counts.target_candidates,
                Event::TargetMember => &mut counts.target_members,
                Event::SelectionRead => &mut counts.selection_reads,
                Event::CaptureEntered => &mut counts.captures,
                Event::CaptureNode => &mut counts.capture_nodes,
                Event::CaptureEdge => &mut counts.capture_edges,
                Event::MapCompared => &mut counts.map_comparisons,
            };
            *count += 1;
            active.set(Some(counts));
        }
    });
}
pub(super) struct Probe(bool);
impl Probe {
    pub fn start() -> Self {
        scale_metrics::require_enabled().expect("select discovery requires enabled counters");
        ACTIVE.with(|active| {
            assert!(active.get().is_none(), "select discovery already active");
            active.set(Some(Counts::default()));
        });
        Self(true)
    }
    pub fn finish(mut self) -> Counts {
        self.0 = false;
        ACTIVE.with(|active| active.take().expect("active select discovery"))
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        if scale_metrics::ENABLED && self.0 {
            ACTIVE.with(|active| active.set(None));
        }
    }
}
