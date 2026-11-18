//! Driving a canonical Source operation and servicing its events.
//!
//! [`DomSourceRuntime`] is a weak permission to route one operation through the same session's owners; the services a
//! reached request needs are borrowed as [`DomSourceServices`]; a [`DomSourceUrlService`] observes URLs and a
//! [`DomSourceInlineStyleEdit`] is one requested edit with its inline qualifications. A drive returns [`DomSourceDriveProgress`] or fails with a [`DomSourceDriveError`];
//! [`DomSourceRuntimeError`] is the refusal of the Source owners and [`DomSourceUnavailable`] names a missing service at the
//! actual next request. [`DomSelectCompletionProgress`] is the progress of an authorized Select task's completion and
//! [`DomSourceUrlObservationResult`] the result of a URL observation. The Source event servicing owner has its
//! own limits ([`DomSourceEventRuntimeLimits`], [`DomSourceEventServiceLimits`]), resources
//! ([`DomSourceEventResource`]), progress ([`DomSourceEventServiceProgress`]) and errors ([`DomSourceEventRuntimeError`]).

pub use crate::internal::dom_dialog::{
    DomSourceEventResource, DomSourceEventRuntimeError, DomSourceEventRuntimeLimits,
    DomSourceEventServiceLimits, DomSourceEventServiceProgress,
};
pub use crate::internal::dom_source::{
    DomSelectCompletionProgress, DomSourceDriveError, DomSourceDriveProgress,
    DomSourceInlineStyleEdit, DomSourceRuntime, DomSourceRuntimeError, DomSourceServices,
    DomSourceUnavailable, DomSourceUrlObservationResult, DomSourceUrlService,
};
