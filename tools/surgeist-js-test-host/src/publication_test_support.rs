//! Publication failures with separately observable accepted and rejected custody.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use surgeist_js::{
    ContextAccess, JsResult,
    error::EngineResourceError,
    job::{
        Job, JobAcceptance, JobExecutor, JobObserver, JobPublicationCause, JobPublicationError,
        SimpleJobExecutor,
    },
};

pub(crate) struct PublicationExecutor {
    accepted_attempt: Option<usize>,
    pub(crate) attempts: Cell<usize>,
    pub(crate) observations: RefCell<Vec<JobObserver>>,
    inner: Rc<SimpleJobExecutor>,
}

impl PublicationExecutor {
    pub(crate) fn new(accepted_attempt: Option<usize>) -> Self {
        Self {
            accepted_attempt,
            attempts: Cell::new(0),
            observations: RefCell::default(),
            inner: Rc::new(SimpleJobExecutor::new()),
        }
    }
}

impl JobExecutor for PublicationExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut context: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        let attempt = self.attempts.get();
        self.attempts.set(attempt + 1);
        let observer = match &job {
            Job::GenericJob(job) => Some(job.observer()),
            Job::AsyncJob(job) => Some(job.observer()),
            _ => None,
        };
        if let Some(observer) = observer {
            self.observations.borrow_mut().push(observer);
        }
        let error = if attempt == 0 {
            EngineResourceError::Allocation
        } else {
            EngineResourceError::SizeOverflow
        };
        if self.accepted_attempt == Some(attempt) {
            match self.inner.clone().enqueue_job(job, context.reborrow())? {
                JobAcceptance::Accepted => {}
                other => return Ok(other),
            }
            Ok(JobAcceptance::AcceptedWithError(
                JobPublicationCause::try_new(error.into(), context.reborrow()).unwrap(),
            ))
        } else {
            let cause = JobPublicationCause::try_new(error.into(), context.reborrow()).unwrap();
            Err(JobPublicationError::new(job, cause))
        }
    }

    fn run_jobs(self: Rc<Self>, context: ContextAccess<'_>) -> JsResult<()> {
        self.inner.clone().run_jobs(context)
    }
}
