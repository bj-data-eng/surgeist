//! Layout-allocated atomic, control, boundary, strut and fit geometry.
//!
//! An atomic box on a line is described by [`AtomicAllocation`], [`AtomicBoxAllocation`], [`AtomicFlowAllocation`]
//! and [`AtomicLineAllocation`]; [`AtomicFitAllocation`] names the operation that determines its line-fit
//! contribution and [`CssMarginBoxFit`] holds used CSS inline margins and border size. A selected control is
//! described by [`ControlAllocation`] and [`ControlStrut`]. [`LineStrut`], [`LineBoxStrut`] and
//! [`LineBoxAlignment`] describe struts and their alignment, [`InlineBoundaryAllocation`] and
//! [`InlineBoundaryContextAllocation`] describe inline-box boundaries, and [`LineAlignmentInsets`] holds
//! alignment-only padding inside the unchanged outer available inline span.

pub use crate::protocol::{
    AtomicAllocation, AtomicBoxAllocation, AtomicFitAllocation, AtomicFlowAllocation,
    AtomicLineAllocation, ControlAllocation, ControlStrut, CssMarginBoxFit,
    InlineBoundaryAllocation, InlineBoundaryContextAllocation, LineAlignmentInsets,
    LineBoxAlignment, LineBoxStrut, LineStrut,
};
