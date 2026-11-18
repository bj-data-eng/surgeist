//! Producer wakeups: a thread-safe staging queue that asks a host for future drain turns.
//!
//! An [`AppProxy`] is a thread-safe staging queue for inputs ([`ProxyInput`]) that requests a future host turn through a
//! [`WakeBridge`]; [`QueuePolicy`] bounds its queue. [`ProxyDrainReport`] is the result of one host-facing drain, and a
//! rejection is an [`AppProxyError`] (see [`AppProxyErrorCode`]) that preserves the rejected input. [`WakeError`] reports
//! a bridge failure.

pub use crate::internal::proxy::{
    AppProxy, AppProxyError, AppProxyErrorCode, ProxyDrainReport, ProxyInput, QueuePolicy,
    WakeBridge, WakeError,
};
