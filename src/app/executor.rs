use std::{error::Error, fmt};

use super::{AppScope, StartTaskEffect, TaskAttemptId, TaskHandle, TaskId, TaskKey};

pub trait RuntimeExecutor<Input> {
    fn spawn_task(
        &mut self,
        request: SpawnRequest<Input>,
    ) -> Result<ExecutorTaskHandle, ExecutorError>;

    fn spawn_blocking_task(
        &mut self,
        request: SpawnRequest<Input>,
    ) -> Result<ExecutorTaskHandle, ExecutorError>;

    fn cancel(&mut self, handle: TaskHandle) -> Result<(), ExecutorError>;

    fn name(&self) -> &'static str;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlockingPolicy {
    #[default]
    Abortable,
    Blocking,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnRequest<Input> {
    task_id: TaskId,
    attempt_id: TaskAttemptId,
    key: TaskKey,
    scope: AppScope,
    blocking: BlockingPolicy,
    input: Option<Input>,
}

impl<Input> SpawnRequest<Input> {
    #[must_use]
    pub fn new(task_id: TaskId, attempt_id: TaskAttemptId, key: TaskKey, scope: AppScope) -> Self {
        Self {
            task_id,
            attempt_id,
            key,
            scope,
            blocking: BlockingPolicy::Abortable,
            input: None,
        }
    }

    #[must_use]
    pub fn from_start_effect(
        task_id: TaskId,
        attempt_id: TaskAttemptId,
        effect: &StartTaskEffect,
    ) -> Self {
        Self::new(
            task_id,
            attempt_id,
            effect.key().clone(),
            effect.scope().clone(),
        )
    }

    #[must_use]
    pub fn with_blocking_policy(mut self, blocking: BlockingPolicy) -> Self {
        self.blocking = blocking;
        self
    }

    #[must_use]
    pub fn with_input(mut self, input: Input) -> Self {
        self.input = Some(input);
        self
    }

    #[must_use]
    pub const fn task_id(&self) -> TaskId {
        self.task_id
    }

    #[must_use]
    pub const fn attempt_id(&self) -> TaskAttemptId {
        self.attempt_id
    }

    #[must_use]
    pub fn key(&self) -> &TaskKey {
        &self.key
    }

    #[must_use]
    pub const fn scope(&self) -> &AppScope {
        &self.scope
    }

    #[must_use]
    pub const fn blocking_policy(&self) -> BlockingPolicy {
        self.blocking
    }

    #[must_use]
    pub const fn input(&self) -> Option<&Input> {
        self.input.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutorTaskHandle {
    task_id: TaskId,
    attempt_id: TaskAttemptId,
}

impl ExecutorTaskHandle {
    #[must_use]
    pub const fn new(task_id: TaskId, attempt_id: TaskAttemptId) -> Self {
        Self {
            task_id,
            attempt_id,
        }
    }

    #[must_use]
    pub const fn task_id(&self) -> TaskId {
        self.task_id
    }

    #[must_use]
    pub const fn attempt_id(&self) -> TaskAttemptId {
        self.attempt_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutorError {
    message: String,
}

impl ExecutorError {
    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ExecutorError {}
