//! General UTF16 stylesheet input, CSS Syntax preprocessing and provenance.
use crate::tokenization::bounded::{InputCursor, InputKind, Request};
use crate::{CssParseReport, CssParserContext, CssSheet, CssSourceSnapshot};
use std::sync::Arc;

/// Owner-issued attestation for a real complete stylesheet parse.
#[derive(Clone, Debug)]
pub struct CssStylesheetParseSource {
    source: CssSourceSnapshot,
    context: CssParserContext,
}
impl CssStylesheetParseSource {
    pub(crate) fn new(source: CssSourceSnapshot, context: CssParserContext) -> Self {
        Self { source, context }
    }
    #[must_use]
    pub const fn source(&self) -> &CssSourceSnapshot {
        &self.source
    }
    #[must_use]
    pub const fn context(&self) -> CssParserContext {
        self.context
    }
}

pub(crate) struct Utf16Source {
    pub(crate) original: Arc<Vec<u16>>,
    pub(crate) map: Vec<(usize, usize)>,
}

/// Independent finite preparation limits. Grammar execution retains its existing
/// resource limits; polling during grammar execution is not provided.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssStylesheetInputLimits {
    /// Logical original UTF16 extent; original spare backing is charged below.
    pub max_input_units: usize,
    /// Logical output bytes and actual normalized String capacity.
    pub max_output_bytes: usize,
    /// Actual normalized-boundary map backing slots.
    pub max_map_entries: usize,
    /// Actual source-position checkpoint backing slots.
    pub max_checkpoints: usize,
    /// Fixed decoder/checkpoint preparation registers; grammar scratch is separate.
    pub max_scratch_bytes: usize,
    /// Aggregate original-unit, text, map and checkpoint backing, excluding
    /// fixed headers, allocator overhead and the existing grammar's scratch.
    pub max_retained_bytes: usize,
    /// Cursor requests/acknowledgements, checkpoint characters and pre/post
    /// grammar observations. The initial interruption poll is not an action.
    pub max_work: usize,
}
impl Default for CssStylesheetInputLimits {
    fn default() -> Self {
        Self {
            max_input_units: 16 * 1024 * 1024,
            max_output_bytes: 16 * 1024 * 1024,
            max_map_entries: 16 * 1024 * 1024,
            max_checkpoints: 262_145,
            max_scratch_bytes: 1024,
            max_retained_bytes: 512 * 1024 * 1024,
            max_work: 128 * 1024 * 1024,
        }
    }
}

/// Atomic refusal: no partial source or parse report is returned.
#[derive(Debug, Eq, PartialEq)]
pub enum CssStylesheetInputError<E> {
    Limit(&'static str),
    Allocation,
    CapacityOverflow,
    Interrupted(E),
}
impl<E: std::fmt::Display> std::fmt::Display for CssStylesheetInputError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limit(resource) => write!(f, "stylesheet {resource} limit exceeded"),
            Self::Allocation => f.write_str("stylesheet input allocation failed"),
            Self::CapacityOverflow => f.write_str("stylesheet input capacity overflow"),
            Self::Interrupted(error) => write!(f, "stylesheet input interrupted: {error}"),
        }
    }
}
impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for CssStylesheetInputError<E> {}

pub(crate) fn check<E>(
    actual: usize,
    maximum: usize,
    resource: &'static str,
) -> Result<(), CssStylesheetInputError<E>> {
    if actual > maximum {
        Err(CssStylesheetInputError::Limit(resource))
    } else {
        Ok(())
    }
}

impl CssParserContext {
    /// Preprocesses shared original UTF16 with exact scalar-boundary mapping,
    /// then invokes the ordinary stylesheet grammar in this complete context.
    /// Polls before every preparation action and immediately before/after parsing.
    /// The grammar itself has no fine-grained interruption checkpoint.
    pub fn parse_sheet_utf16_with_limits<E>(
        self,
        original: Arc<Vec<u16>>,
        limits: CssStylesheetInputLimits,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<CssParseReport<CssSheet>, CssStylesheetInputError<E>> {
        use CssStylesheetInputError::{Allocation, CapacityOverflow, Interrupted};
        poll().map_err(Interrupted)?;
        check(
            original.len(),
            limits.max_input_units,
            "original UTF16 units",
        )?;
        let original_bytes = original.capacity().checked_mul(2).ok_or(CapacityOverflow)?;
        check(
            original_bytes,
            limits.max_retained_bytes,
            "retained source bytes",
        )?;
        check(
            std::mem::size_of::<InputCursor>() + std::mem::size_of::<crate::CssSourcePosition>(),
            limits.max_scratch_bytes,
            "preparation scratch bytes",
        )?;
        let mut work = 0usize;
        let mut step = || {
            poll().map_err(Interrupted)?;
            work = work.checked_add(1).ok_or(CapacityOverflow)?;
            check(work, limits.max_work, "preparation work")
        };
        let mut cursor = InputCursor::new(InputKind::Utf16, original.len());
        let mut text = String::new();
        let mut map = Vec::new();
        loop {
            step()?;
            match cursor.request() {
                Request::Read { index } => {
                    cursor.read(original[index]).expect("UTF16 cursor protocol")
                }
                Request::HeldUnit => cursor.process_held().expect("UTF16 held-unit protocol"),
                Request::Emit(scalar) => {
                    let end = text
                        .len()
                        .checked_add(scalar.value.len_utf8())
                        .ok_or(CapacityOverflow)?;
                    check(end, limits.max_output_bytes, "normalized UTF8 bytes")?;
                    check(
                        map.len().checked_add(1).ok_or(CapacityOverflow)?,
                        limits.max_map_entries,
                        "source map entries",
                    )?;
                    let text_capacity = if end > text.capacity() {
                        text.capacity()
                            .saturating_mul(2)
                            .max(end)
                            .min(limits.max_output_bytes)
                    } else {
                        text.capacity()
                    };
                    let map_capacity = if map.len() == map.capacity() {
                        map.capacity()
                            .saturating_mul(2)
                            .max(1)
                            .min(limits.max_map_entries)
                    } else {
                        map.capacity()
                    };
                    let backing = original_bytes
                        .checked_add(text_capacity)
                        .and_then(|v| {
                            map_capacity
                                .checked_mul(std::mem::size_of::<(usize, usize)>())
                                .and_then(|m| v.checked_add(m))
                        })
                        .ok_or(CapacityOverflow)?;
                    check(backing, limits.max_retained_bytes, "retained source bytes")?;
                    if text_capacity > text.capacity() {
                        text.try_reserve_exact(text_capacity - text.len())
                            .map_err(|_| Allocation)?;
                    }
                    if map_capacity > map.capacity() {
                        map.try_reserve_exact(map_capacity - map.len())
                            .map_err(|_| Allocation)?;
                    }
                    check(
                        text.capacity(),
                        limits.max_output_bytes,
                        "normalized UTF8 bytes",
                    )?;
                    check(map.capacity(), limits.max_map_entries, "source map entries")?;
                    let backing = original_bytes
                        .checked_add(text.capacity())
                        .and_then(|v| {
                            map.capacity()
                                .checked_mul(std::mem::size_of::<(usize, usize)>())
                                .and_then(|m| v.checked_add(m))
                        })
                        .ok_or(CapacityOverflow)?;
                    check(backing, limits.max_retained_bytes, "retained source bytes")?;
                    map.push((text.len(), scalar.start));
                    text.push(scalar.value);
                    cursor.emitted().expect("UTF16 emit protocol");
                }
                Request::End => {
                    if cursor.end().expect("UTF16 EOF protocol") {
                        break;
                    }
                }
            }
        }
        let other_bytes = original_bytes
            .checked_add(
                map.capacity()
                    .checked_mul(std::mem::size_of::<(usize, usize)>())
                    .ok_or(CapacityOverflow)?,
            )
            .ok_or(CapacityOverflow)?;
        let source =
            CssSourceSnapshot::try_from_preprocessed(text, limits, other_bytes, &mut step)?
                .attach_utf16(Utf16Source { original, map })
                .ok_or(CapacityOverflow)?;
        check(
            source.storage_usage().retained_bytes(),
            limits.max_retained_bytes,
            "retained source bytes",
        )?;
        step()?;
        let report = crate::parser::parse_sheet_snapshot(&source, self);
        step()?;
        Ok(report)
    }
}
