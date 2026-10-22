use super::{
    Error, ErrorCode, ErrorDetail, Range, Result, Source, SourceRange, SourceRevision, Text, range,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEdit {
    range: Range,
    inserted_text: Text,
}

impl TextEdit {
    pub fn insert(source: &Source, index: usize, text: impl Into<Text>) -> Result<Self> {
        Self::replace(source, Range::new(index, index), text)
    }

    pub fn replace(source: &Source, range: Range, text: impl Into<Text>) -> Result<Self> {
        range::validate(source.text(), range)?;
        Ok(Self {
            range,
            inserted_text: text.into(),
        })
    }

    pub fn delete(source: &Source, range: Range) -> Result<Self> {
        Self::replace(source, range, "")
    }

    #[must_use]
    pub const fn range(&self) -> Range {
        self.range
    }

    #[must_use]
    pub fn inserted_text(&self) -> &Text {
        &self.inserted_text
    }

    /// Applies the edit at the next source revision, refusing revision exhaustion.
    pub fn apply_to(&self, mut source: Source) -> Result<Source> {
        let source_range = SourceRange::try_new(source.text(), self.range.start, self.range.end)?;
        let revision = SourceRevision::new(source.revision)
            .checked_next()
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::SourceRevisionExhausted,
                    "source revision is exhausted",
                )
                .with_detail(ErrorDetail::SourceRevisionExhausted {
                    revision: source.revision,
                })
            })?;
        source
            .text
            .len()
            .checked_sub(self.range.len())
            .and_then(|len| len.checked_add(self.inserted_text.len()))
            .ok_or_else(super::projection::length_overflow)?;
        for span in &source.spans {
            range::validate(source.text(), span.range)?;
        }
        for box_ in &source.boxes {
            range::validate_index(source.text(), box_.index, "inline box index")?;
        }
        let mut units = source.text.code_units().to_vec();
        units.splice(
            self.range.start..self.range.end,
            self.inserted_text.code_units().iter().copied(),
        );
        project_edit_ranges(&mut source, source_range, self.inserted_text.len());
        source.revision = revision.get();
        source.text = Text::from_code_units(units);
        Ok(source)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Edit {
    Insert { index: usize, text: Text },
    Replace { range: Range, text: Text },
    Delete { range: Range },
}

impl Edit {
    pub fn normalize(self, source: &Source) -> Result<TextEdit> {
        match self {
            Self::Insert { index, text } => TextEdit::insert(source, index, text),
            Self::Replace { range, text } => TextEdit::replace(source, range, text),
            Self::Delete { range } => TextEdit::delete(source, range),
        }
    }
}

fn project_edit_ranges(source: &mut Source, source_range: SourceRange, inserted_len: usize) {
    let range = source_range.range();
    let removed_len = range.len();
    for span in &mut source.spans {
        span.range = Range::new(
            project_edit_start(span.range.start, range, inserted_len, removed_len),
            project_edit_end(span.range.end, range, inserted_len, removed_len),
        );
    }
    for box_ in &mut source.boxes {
        box_.index = project_edit_anchor(box_.index, range, inserted_len, removed_len);
    }
}

fn project_edit_start(
    index: usize,
    range: Range,
    inserted_len: usize,
    removed_len: usize,
) -> usize {
    if index <= range.start {
        index
    } else if index < range.end {
        range.start
    } else {
        index - removed_len + inserted_len
    }
}

fn project_edit_end(index: usize, range: Range, inserted_len: usize, removed_len: usize) -> usize {
    if index < range.start {
        index
    } else if index <= range.end {
        range.start + inserted_len
    } else {
        index - removed_len + inserted_len
    }
}

fn project_edit_anchor(
    index: usize,
    range: Range,
    inserted_len: usize,
    removed_len: usize,
) -> usize {
    if index <= range.start {
        index
    } else if index <= range.end {
        range.start + inserted_len
    } else {
        index - removed_len + inserted_len
    }
}
