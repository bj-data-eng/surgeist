//! Task scopes, segments and their validation errors.
//!
//! A [`TaskScope`] is UI-agnostic path data: an ordered list of
//! [`TaskScopeSegment`] values, each a namespace and a value. The empty scope is
//! the application scope. Namespace and value are trimmed and must be non-empty
//! without ASCII control characters, and the namespaces `ui`, `window`, `render`
//! and `retained` are reserved; violations are reported as [`TaskScopeError`].

pub use crate::internal::scope::{TaskScope, TaskScopeError, TaskScopeErrorKind, TaskScopeSegment};
