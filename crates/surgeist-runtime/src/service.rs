//! Services managed by a root adapter.
//!
//! A [`ServiceRegistration`] records a service's identity, scope, [`MailboxPolicy`] and its startup, shutdown and restart
//! policies. A [`ServiceMailbox`] is a FIFO queue created for one service identity and bounded by a [`MailboxPolicy`];
//! [`MailboxOverflow`] selects what a full mailbox does with an incoming message and [`MailboxPushOutcome`] reports the
//! result of a push. [`ServiceStartup`],
//! [`ServiceRestart`] and [`ServiceShutdown`] select when a service starts, whether it restarts and how it stops;
//! [`ServiceStatus`] is its observed lifecycle state. [`ServiceCommandName`] and [`ServiceCommandPayload`] are opaque
//! values passed unchanged to the root adapter.

pub use crate::internal::service::{
    MailboxOverflow, MailboxPolicy, MailboxPushOutcome, ServiceCommandName, ServiceCommandPayload,
    ServiceMailbox, ServiceRegistration, ServiceRestart, ServiceShutdown, ServiceStartup,
    ServiceStatus,
};
