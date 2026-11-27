//! The shared HTML877d setRangeText calculation, in UTF-16 units.

use super::input::{HtmlInputSelection, HtmlInputSelectionDirection, HtmlTextSelectionEdit};
use std::ops::Range;

/// Which extent to select after replacing the requested text.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HtmlRangeTextMode {
    /// Select all inserted code units.
    Select,
    /// Collapse the selection at the start of the replacement.
    Start,
    /// Collapse the selection after the replacement.
    End,
    /// Adjust both existing endpoints for the removed and inserted extents.
    #[default]
    Preserve,
}

/// The selected overload after argument conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRangeTextTarget {
    /// Use the current selection and preserve its adjusted endpoints.
    CurrentSelection,
    /// Use both explicitly supplied offsets.
    Offsets {
        /// Requested start in UTF-16 units before clamping.
        start: usize,
        /// Requested end in UTF-16 units before clamping.
        end: usize,
        /// Omission in this overload selects `Preserve`.
        mode: HtmlRangeTextMode,
    },
}

impl HtmlRangeTextTarget {
    /// Resolve the overload and qualify the original bounds. The adapter
    /// marks the actual control dirty before this step. The relevant-value
    /// length and replacement extent are used only after it succeeds.
    pub fn resolve(
        self,
        selection: HtmlInputSelection,
    ) -> Result<HtmlRangeTextBounds, HtmlRangeTextError> {
        let (selection_start, selection_end) = selection.offsets();
        let (start, end, mode) = match self {
            Self::CurrentSelection => (selection_start, selection_end, HtmlRangeTextMode::Preserve),
            Self::Offsets { start, end, mode } => (start, end, mode),
        };
        if start > end {
            return Err(HtmlRangeTextError::Reversed);
        }
        Ok(HtmlRangeTextBounds {
            start,
            end,
            mode,
            selection,
        })
    }
}

/// Why the source extent could not be calculated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRangeTextError {
    /// Explicit start exceeds explicit end before either offset is clamped.
    Reversed,
    /// The result extent exceeds the native representation.
    ExtentOverflow,
}

impl std::fmt::Display for HtmlRangeTextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Reversed => "replacement start exceeds its end",
            Self::ExtentOverflow => "replacement exceeds the native text extent",
        })
    }
}

impl std::error::Error for HtmlRangeTextError {}

/// The literal splice and the requested selection after writing it. The
/// adapter owns the actual string, dirty flag, value-change clamp and event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRangeTextPlan {
    replaced: Range<usize>,
    output_len: usize,
    selection: HtmlTextSelectionEdit,
}

/// Resolved original offsets after the required pre-clamp order check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlRangeTextBounds {
    start: usize,
    end: usize,
    mode: HtmlRangeTextMode,
    selection: HtmlInputSelection,
}

impl HtmlRangeTextBounds {
    /// Calculate the splice against the actual previous relevant value length.
    pub fn plan(
        self,
        value_len: usize,
        replacement_len: usize,
    ) -> Result<HtmlRangeTextPlan, HtmlRangeTextError> {
        use HtmlRangeTextMode::{End, Preserve, Select, Start};

        let (selection_start, selection_end) = self.selection.offsets();
        let start = self.start.min(value_len);
        let end = self.end.min(value_len);
        let new_end = start
            .checked_add(replacement_len)
            .ok_or(HtmlRangeTextError::ExtentOverflow)?;
        let output_len = new_end
            .checked_add(value_len - end)
            .ok_or(HtmlRangeTextError::ExtentOverflow)?;
        let (next_start, next_end) = match self.mode {
            Select => (start, new_end),
            Start => (start, start),
            End => (new_end, new_end),
            Preserve => (
                preserve_offset(selection_start.min(value_len), start..end, start, new_end),
                preserve_offset(selection_end.min(value_len), start..end, new_end, new_end),
            ),
        };
        Ok(HtmlRangeTextPlan {
            replaced: start..end,
            output_len,
            selection: HtmlTextSelectionEdit::Range {
                start: next_start,
                end: next_end,
                direction: HtmlInputSelectionDirection::None,
            },
        })
    }
}

impl HtmlRangeTextPlan {
    /// Code units to replace in the original relevant value.
    pub fn replaced(&self) -> Range<usize> {
        self.replaced.clone()
    }

    /// Code units in the spliced string before any later value normalization.
    pub const fn output_len(&self) -> usize {
        self.output_len
    }

    /// The final selection request, applied after the actual value changes.
    pub const fn selection(&self) -> HtmlTextSelectionEdit {
        self.selection
    }
}

fn preserve_offset(offset: usize, replaced: Range<usize>, inside: usize, new_end: usize) -> usize {
    if offset > replaced.end {
        // This lies within the retained suffix of the checked output extent.
        new_end + (offset - replaced.end)
    } else if offset > replaced.start {
        inside
    } else {
        offset
    }
}
