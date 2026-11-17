//! Private implementation of the task contracts.

pub(crate) mod cancel;
pub(crate) mod coordination;
pub(crate) mod event;
pub(crate) mod executor;
pub(crate) mod id;
pub(crate) mod lifecycle;
pub(crate) mod policy;
pub(crate) mod provenance;
pub(crate) mod queue;
pub(crate) mod runtime_tokio;
pub(crate) mod scope;
#[cfg(test)]
pub(crate) mod testing;
