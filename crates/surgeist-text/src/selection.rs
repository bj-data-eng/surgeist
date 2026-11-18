//! Line selection requests and the selected line.
//!
//! A [`LineSelectionRequest`] is checked by `PreparedText::select` and yields a [`SelectedLine`]. The request
//! holds a [`LineContext`], which in turn holds a [`FormattedLineContext`], a [`LineBoxDirection`], a
//! [`LineOrientation`], a [`LineStart`] and a [`LineEnd`], and a [`PreparedTermination`]. A termination is an
//! ordinary opportunity, an interval-bound conditional capability, or a measurement that is never placeable: a
//! lexical word portion trial (`Measurement`) or a whole remaining phrase trial (`PhraseMeasurement`).

pub use crate::protocol::{
    FormattedLineContext, LineBoxDirection, LineContext, LineEnd, LineOrientation,
    LineSelectionRequest, LineStart, PreparedTermination, SelectedLine,
};
