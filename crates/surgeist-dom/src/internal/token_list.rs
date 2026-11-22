//! DOM ordered sets over an Element's canonical null-namespace attribute.

use std::ops::ControlFlow;

use surgeist_html::{
    bounded::lexical::{HtmlSpaceTokenCursor, HtmlSpaceTokenStep},
    values::DocumentString,
};

use crate::{
    DomError, DomSnapshot,
    internal::{identity::NodeId, names::same_code_units},
};

pub(crate) const CLASS: &[u16] = &[0x63, 0x6c, 0x61, 0x73, 0x73];

/// A live association between an exact Element and a null-namespace attribute.
///
/// The association keeps its identity when the attribute is absent, replaced or
/// removed. Read it against the current qualified snapshot for each observation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomTokenList {
    element: NodeId,
    attribute: Attribute,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Attribute {
    Class,
}

impl DomTokenList {
    /// The Element's exact null-namespace `class` association.
    pub const fn class(element: NodeId) -> Self {
        Self {
            element,
            attribute: Attribute::Class,
        }
    }

    pub const fn element(self) -> NodeId {
        self.element
    }

    pub(crate) const fn attribute_name_str(self) -> &'static str {
        match self.attribute {
            Attribute::Class => "class",
        }
    }

    /// The exact associated local name in the null namespace.
    pub const fn attribute_name(self) -> &'static [u16] {
        match self.attribute {
            Attribute::Class => CLASS,
        }
    }

    /// Observe the exact raw attribute value, including whitespace and duplicates.
    /// An absent attribute has no raw value.
    pub fn raw_with<E: From<DomError>>(
        self,
        snapshot: &DomSnapshot,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<&DocumentString>, E> {
        let attributes = snapshot
            .node_with(self.element, &mut step)?
            .attributes()
            .ok_or(DomError::WrongNodeKind)?;
        Ok(attributes
            .get_null_namespace_attribute_with(self.attribute_name(), step)?
            .map(|attribute| attribute.value()))
    }

    /// Observe membership without materializing the ordered set.
    pub fn contains_with<E: From<DomError>>(
        self,
        snapshot: &DomSnapshot,
        token: &[u16],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let Some(raw) = self.raw_with(snapshot, &mut step)? else {
            return Ok(false);
        };
        Ok(
            scan_space_tokens(raw.code_units(), step, |candidate, step| {
                Ok(if same_code_units(candidate, token, step)? {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                })
            })?
            .is_some(),
        )
    }

    /// Capture the first occurrence of each token in order, borrowing its exact
    /// UTF-16 units from this snapshot. `reserve` admits actual backing before
    /// allocating a token bank, including a replacement while its predecessor
    /// still exists. Its receipt stays with the bank until release.
    pub fn capture_with<'a, E: From<DomError>, B>(
        self,
        snapshot: &'a DomSnapshot,
        mut step: impl FnMut() -> Result<(), E>,
        mut reserve: impl FnMut(usize) -> Result<B, E>,
    ) -> Result<DomTokenListCapture<'a, B>, E> {
        let mut captured = DomTokenListCapture {
            tokens: Vec::new(),
            backing: None,
        };
        if let Some(raw) = self.raw_with(snapshot, &mut step)? {
            scan_space_tokens(raw.code_units(), step, |token, step| {
                if token_position(&captured.tokens, token, &mut *step)?.is_none() {
                    captured.push(token, step, &mut reserve)?;
                }
                Ok(ControlFlow::<()>::Continue(()))
            })?;
        }
        Ok(captured)
    }
}

/// One ordered-set observation and its caller-admitted temporary backing.
pub struct DomTokenListCapture<'a, B> {
    tokens: Vec<&'a [u16]>,
    backing: Option<B>,
}

impl<B> std::fmt::Debug for DomTokenListCapture<'_, B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DomTokenListCapture")
            .field("len", &self.tokens.len())
            .finish_non_exhaustive()
    }
}

impl<'a, B> DomTokenListCapture<'a, B> {
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn item(&self, index: usize) -> Option<&'a [u16]> {
        self.tokens.get(index).copied()
    }

    fn push<E: From<DomError>>(
        &mut self,
        token: &'a [u16],
        step: &mut impl FnMut() -> Result<(), E>,
        reserve: &mut impl FnMut(usize) -> Result<B, E>,
    ) -> Result<(), E> {
        step()?;
        if self.tokens.len() == self.tokens.capacity() {
            let capacity = self.tokens.capacity();
            let maximum = isize::MAX as usize / size_of::<&[u16]>();
            let required = capacity
                .checked_add(1)
                .filter(|required| *required <= maximum)
                .ok_or(DomError::RepresentationExhausted)?;
            let capacity = capacity
                .checked_mul(2)
                .filter(|capacity| *capacity <= maximum)
                .unwrap_or(required)
                .max(required);
            let backing = reserve(capacity * size_of::<&[u16]>())?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(capacity)
                .map_err(|_| DomError::AllocationFailed)?;
            if replacement.capacity() != capacity {
                return Err(DomError::AllocationFailed.into());
            }
            for &old in &self.tokens {
                step()?;
                replacement.push(old);
            }
            self.tokens = replacement;
            self.backing = Some(backing);
        }
        self.tokens.push(token);
        Ok(())
    }
}

/// One token-list operation after argument conversion to DOMString.
#[derive(Clone, Debug)]
pub enum DomTokenListEdit {
    Add(Vec<DocumentString>),
    Remove(Vec<DocumentString>),
    Toggle {
        token: DocumentString,
        force: Option<bool>,
    },
    Replace {
        token: DocumentString,
        replacement: DocumentString,
    },
}

pub(crate) fn scan_space_tokens<'a, T, E, Step: FnMut() -> Result<(), E>>(
    units: &'a [u16],
    mut step: Step,
    mut visit: impl FnMut(&'a [u16], &mut Step) -> Result<ControlFlow<T>, E>,
) -> Result<Option<T>, E> {
    for _ in 0..HtmlSpaceTokenCursor::BEGIN_WORK {
        step()?;
    }
    let mut cursor = HtmlSpaceTokenCursor::new(units);
    loop {
        for _ in 0..HtmlSpaceTokenCursor::STEP_WORK {
            step()?;
        }
        match cursor.step() {
            HtmlSpaceTokenStep::Continue => (),
            HtmlSpaceTokenStep::Token { start, end } => {
                if let ControlFlow::Break(value) = visit(&units[start..end], &mut step)? {
                    return Ok(Some(value));
                }
            }
            HtmlSpaceTokenStep::Done => return Ok(None),
        }
    }
}

pub(crate) fn token_position<E>(
    tokens: &[&[u16]],
    token: &[u16],
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<Option<usize>, E> {
    for (index, candidate) in tokens.iter().enumerate() {
        step()?;
        if same_code_units(candidate, token, &mut step)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}
