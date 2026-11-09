//! Executors that inject publication failures while preserving real queue custody.

use std::{cell::RefCell, rc::Rc};
use surgeist_js::{
    ContextAccess, JsResult,
    error::EngineResourceError,
    job::{
        Job, JobAcceptance, JobExecutor, JobObserver, JobPublicationCause, JobPublicationError,
        SimpleJobExecutor,
    },
};

pub(crate) struct PublicationExecutor {
    pub(crate) accepted: bool,
    pub(crate) inner: Rc<SimpleJobExecutor>,
    pub(crate) observations: RefCell<Vec<JobObserver>>,
}

impl PublicationExecutor {
    pub(crate) fn new(accepted: bool) -> Self {
        Self {
            accepted,
            inner: Rc::new(SimpleJobExecutor::new()),
            observations: RefCell::default(),
        }
    }
}

impl JobExecutor for PublicationExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut context: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Job::PromiseJob(job) = &job {
            self.observations.borrow_mut().push(job.observer());
        }
        if self.accepted {
            match self.inner.clone().enqueue_job(job, context.reborrow())? {
                JobAcceptance::Accepted => {}
                other => return Ok(other),
            }
            let cause = JobPublicationCause::try_new(
                EngineResourceError::Allocation.into(),
                context.reborrow(),
            )
            .unwrap();
            Ok(JobAcceptance::AcceptedWithError(cause))
        } else {
            let cause = JobPublicationCause::try_new(
                EngineResourceError::Allocation.into(),
                context.reborrow(),
            )
            .unwrap();
            Err(JobPublicationError::new(job, cause))
        }
    }

    fn run_jobs(self: Rc<Self>, context: ContextAccess<'_>) -> JsResult<()> {
        self.inner.clone().run_jobs(context)
    }
}
