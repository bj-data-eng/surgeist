//! Admission at optimizer-owned boundaries. Delegated producers describe their
//! own work; their adapters retain that attribution without charging it twice.

use crate::{JsResult, value::primitive::PrimitiveWork};
use surgeist_js_interner::InternerWork;

#[derive(Clone, Copy, Debug)]
pub(crate) enum OptimizerWork {
    Visit,
    Iteration,
    FoldAttempt,
    ReductionAttempt,
    Rewrite,
    DeclarationVisit,
    RetirementVisit,
    RetirementReservation,
    LiteralCopy { words: u64 },
    LiteralCapacity { elements: u64 },
    Primitive(PrimitiveWork),
    Interner(InternerWork),
}

pub(crate) type OptimizerAdmission<'a> = Option<&'a mut dyn FnMut(OptimizerWork) -> JsResult<()>>;

#[inline]
pub(crate) fn admit(admission: &mut OptimizerAdmission<'_>, work: OptimizerWork) -> JsResult<()> {
    if let Some(admission) = admission {
        admission(work)?;
    }
    Ok(())
}
