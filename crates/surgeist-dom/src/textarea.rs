//! Native Textarea raw state and bounded API/default-value reads.
use crate::{DocumentString, DomError, DomNode, DomNodeKind, DomResource, NodeId};
use std::sync::atomic::AtomicBool;
use surgeist_html::{HtmlInputSelection, HtmlNewlineNormalizer, HtmlTextareaFlags};

/// Intrinsic current state; defaultValue remains actual immediate child Text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomTextareaState {
    pub(crate) control: crate::control::NativeControlState,
    pub(crate) flags: HtmlTextareaFlags,
    pub(crate) raw: DocumentString,
    pub(crate) selection: HtmlInputSelection,
}
impl DomTextareaState {
    pub(crate) fn fresh(platform: crate::DomNativeInputPlatform) -> Self {
        Self {
            control: crate::control::NativeControlState::fresh(),
            flags: HtmlTextareaFlags::default(),
            raw: DocumentString::default(),
            selection: HtmlInputSelection::initial(platform.none_direction()),
        }
    }
    pub fn flags(&self) -> HtmlTextareaFlags {
        self.flags
    }
    /// Exact raw UTF-16, before API newline normalization.
    pub fn raw_value(&self) -> &DocumentString {
        &self.raw
    }
    pub fn selection(&self) -> HtmlInputSelection {
        self.selection
    }
    pub fn parser_inserted(&self) -> bool {
        self.control.parser_inserted
    }
    pub fn custom_validity_message(&self) -> &DocumentString {
        &self.control.message
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.raw.len().checked_add(self.control.message.len())
    }
}

/// Independent output and work admission for immutable Textarea string reads.
/// Work charges each raw normalization read, materialized output unit, child
/// record visit and output reservation. This does not spend a mutation budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomTextareaReadLimits {
    units: usize,
    work: u64,
}
impl DomTextareaReadLimits {
    pub const fn new(code_units: usize, work: u64) -> Self {
        Self {
            units: code_units,
            work,
        }
    }
    pub const fn code_units(self) -> usize {
        self.units
    }
    pub const fn work(self) -> u64 {
        self.work
    }
}
struct Read<'a> {
    remaining: u64,
    limit: u64,
    cancel: &'a AtomicBool,
}
impl Read<'_> {
    fn step(&mut self) -> Result<(), DomError> {
        crate::store::check_cancel(self.cancel)?;
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(DomError::TextareaReadWorkLimit { limit: self.limit })?;
        Ok(())
    }
}

/// Checked Textarea view at one immutable canonical revision.
#[derive(Clone, Copy, Debug)]
pub struct DomTextarea<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomTextarea<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> &'a DomTextareaState {
        self.node
            .as_element()
            .expect("checked Textarea")
            .textarea_state()
            .expect("checked Textarea")
    }
    pub fn form_owner(self) -> Result<Option<NodeId>, crate::DomError> {
        self.node.snapshot.require_complete_source()?;
        Ok(self.node.node.control_owner)
    }
    /// API value, with Infra newlines normalized; no submitted hard wrapping.
    pub fn value(
        self,
        limits: DomTextareaReadLimits,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        crate::store::check_cancel(cancel)?;
        let mut read = Read {
            remaining: limits.work,
            limit: limits.work,
            cancel,
        };
        normalized_value(
            self.state().raw.code_units(),
            limits.units,
            &mut || read.step(),
            None,
            cancel,
        )
    }
    pub fn text_length(
        self,
        limits: DomTextareaReadLimits,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        crate::store::check_cancel(cancel)?;
        let mut read = Read {
            remaining: limits.work,
            limit: limits.work,
            cancel,
        };
        let length = normalized_length(self.state().raw.code_units(), &mut || read.step())?;
        crate::store::admit(length, limits.units, DomResource::StringCodeUnits)?;
        Ok(length)
    }
    /// Exact immediate child Text content; nested text does not contribute.
    pub fn default_value(
        self,
        limits: DomTextareaReadLimits,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        crate::store::check_cancel(cancel)?;
        let mut read = Read {
            remaining: limits.work,
            limit: limits.work,
            cancel,
        };
        let mut length = 0usize;
        for &child in self.node.children() {
            read.step()?;
            if let DomNodeKind::Text(text) = self.node.snapshot.node(child)?.kind() {
                length = length
                    .checked_add(text.len())
                    .ok_or(DomError::RepresentationExhausted)?;
                crate::store::admit(length, limits.units, DomResource::StringCodeUnits)?;
            }
        }
        read.step()?;
        let mut output = output(length, limits.units, None, cancel)?;
        for &child in self.node.children() {
            read.step()?;
            if let DomNodeKind::Text(text) = self.node.snapshot.node(child)?.kind() {
                for &unit in text.code_units() {
                    read.step()?;
                    output.push(unit);
                }
            }
        }
        Ok(DocumentString::from_utf16(output))
    }
}

pub(crate) fn output(
    length: usize,
    limit: usize,
    budget: Option<&crate::store::style::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<Vec<u16>, DomError> {
    crate::store::style::formation::utf16_buffer(length, limit, budget, cancel)
}
pub(crate) fn normalized_length(
    raw: &[u16],
    step: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<usize, DomError> {
    let mut normalizer = HtmlNewlineNormalizer::new();
    let mut length = 0usize;
    for unit in raw {
        step()?;
        if normalizer.push(*unit).is_some() {
            length = length
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
        }
    }
    Ok(length)
}
pub(crate) fn normalized_value(
    raw: &[u16],
    limit: usize,
    step: &mut impl FnMut() -> Result<(), DomError>,
    budget: Option<&crate::store::style::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    let length = normalized_length(raw, step)?;
    step()?;
    let mut output = output(length, limit, budget, cancel)?;
    let mut normalizer = HtmlNewlineNormalizer::new();
    for unit in raw {
        step()?;
        if let Some(unit) = normalizer.push(*unit) {
            step()?;
            output.push(unit);
        }
    }
    Ok(DocumentString::from_utf16(output))
}
