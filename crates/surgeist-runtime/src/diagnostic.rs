//! Structured runtime diagnostics.
//!
//! A [`Diagnostic`] carries a [`DiagnosticSeverity`] and a [`DiagnosticCode`] with optional ownership and effect context.
//! [`DiagnosticLog`] is a bounded retained history of diagnostics with per-code occurrence counts, and
//! [`QueueDiagnostic`] is queue context attached to a diagnostic.

pub use crate::internal::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticLog, DiagnosticSeverity, QueueDiagnostic,
};
