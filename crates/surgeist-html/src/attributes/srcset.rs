//! HTML srcset candidate constraints and literal recovery, with symbolic URLs.

pub use crate::internal::attributes::srcset::{
    HtmlParsedImageCandidate, HtmlParsedSrcsetDescriptor, HtmlSrcsetAuthorDiagnostic,
    HtmlSrcsetAuthorState, HtmlSrcsetAuthorViolation, HtmlSrcsetCandidate, HtmlSrcsetContext,
    HtmlSrcsetContextAuthorState, HtmlSrcsetContextViolation, HtmlSrcsetDensity,
    HtmlSrcsetDensityError, HtmlSrcsetDensityProjectionError, HtmlSrcsetDescriptor,
    HtmlSrcsetExternalConformance, HtmlSrcsetFollowingImageObservation,
    HtmlSrcsetInterpretationError, HtmlSrcsetParentObservation, HtmlSrcsetParseDiagnostic,
    HtmlSrcsetParseError, HtmlSrcsetProcessingState, HtmlSrcsetRawRange, HtmlSrcsetReport,
    HtmlSrcsetUrlToken, HtmlSrcsetUrlTokenError, HtmlSrcsetValue, HtmlSrcsetWidth,
    HtmlSrcsetWidthError,
};
