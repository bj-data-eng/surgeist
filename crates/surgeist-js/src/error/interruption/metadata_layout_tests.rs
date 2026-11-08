// Intended as a cfg(test)-only child of the interruption error owner. Reexport
// this helper through the error test harness for the lifecycle owner fixture.
use super::*;

pub(crate) fn expected_cause_payloads() -> [usize; 3] {
    [
        std::mem::size_of::<CauseData>(),
        std::mem::size_of::<CauseAnchor>(),
        std::mem::size_of::<RetainedInterruption>(),
    ]
}
