use std::ops::Range as StdRange;

use super::{Error, ErrorCode, ErrorDetail, Result, Text};

/// UTF-16 code-unit range in source text.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[must_use]
    pub const fn contains(self, index: usize) -> bool {
        self.start <= index && index < self.end
    }
}

impl From<Range> for StdRange<usize> {
    fn from(range: Range) -> Self {
        range.start..range.end
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourcePosition(usize);

impl SourcePosition {
    #[must_use]
    pub(crate) const fn from_unchecked(index: usize) -> Self {
        Self(index)
    }

    pub fn try_new(text: &Text, index: usize) -> Result<Self> {
        validate_index_raw(text, index, "source position")?;
        Ok(Self(index))
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SourceRange {
    start: SourcePosition,
    end: SourcePosition,
}

impl SourceRange {
    #[must_use]
    pub(crate) const fn from_unchecked(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end }
    }

    pub fn try_new(text: &Text, start: usize, end: usize) -> Result<Self> {
        let range = Range::new(start, end);
        validate_raw(text, range)?;
        Ok(Self {
            start: SourcePosition(start),
            end: SourcePosition(end),
        })
    }

    #[must_use]
    pub const fn start(self) -> SourcePosition {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> SourcePosition {
        self.end
    }

    #[must_use]
    pub const fn range(self) -> Range {
        Range::new(self.start.0, self.end.0)
    }
}

pub(crate) fn validate(text: &Text, range: Range) -> Result<()> {
    SourceRange::try_new(text, range.start, range.end)?;
    Ok(())
}

pub(crate) fn validate_index(text: &Text, index: usize, name: &'static str) -> Result<()> {
    SourcePosition::try_new_with_name(text, index, name)?;
    Ok(())
}

impl SourcePosition {
    fn try_new_with_name(text: &Text, index: usize, name: &'static str) -> Result<Self> {
        validate_index_raw(text, index, name)?;
        Ok(Self(index))
    }
}

fn validate_raw(text: &Text, range: Range) -> Result<()> {
    if range.start > range.end || range.end > text.len() {
        return Err(Error::new(
            ErrorCode::InvalidRange,
            format!(
                "invalid range {}..{} for text length {}",
                range.start,
                range.end,
                text.len()
            ),
        )
        .with_detail(ErrorDetail::InvalidSourceRange {
            start: range.start,
            end: range.end,
            text_len: text.len(),
        }));
    }
    Ok(())
}

fn validate_index_raw(text: &Text, index: usize, name: &'static str) -> Result<()> {
    if index > text.len() {
        return Err(Error::new(
            ErrorCode::InvalidRange,
            format!("{name} {index} is not an in-bounds UTF-16 endpoint"),
        )
        .with_detail(ErrorDetail::InvalidSourceIndex {
            name,
            index,
            text_len: text.len(),
        }));
    }
    Ok(())
}
