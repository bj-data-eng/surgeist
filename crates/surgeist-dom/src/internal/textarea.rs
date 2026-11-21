//! Native Textarea raw state and bounded API/default-value reads.
use crate::internal::{
    form::DomForm,
    identity::NodeId,
    snapshot::DomNode,
    store::{DomError, DomResource},
};
use std::sync::atomic::AtomicBool;
use surgeist_html::values::DocumentString;
use surgeist_html::{
    elements::{form::HtmlTextareaFlags, input::HtmlInputSelection},
    values::HtmlNewlineNormalizer,
};

/// Intrinsic current state; defaultValue remains actual immediate child Text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomTextareaState {
    pub(crate) control: crate::internal::control::NativeControlState,
    pub(crate) flags: HtmlTextareaFlags,
    pub(crate) raw: DocumentString,
    pub(crate) selection: HtmlInputSelection,
}
impl DomTextareaState {
    pub(crate) fn fresh(platform: crate::internal::document::DomNativeInputPlatform) -> Self {
        Self {
            control: crate::internal::control::NativeControlState::fresh(),
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
        crate::internal::store::check_cancel(self.cancel)?;
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
    pub fn form_owner(self) -> Result<Option<NodeId>, DomError> {
        self.node.snapshot.require_complete_source()?;
        Ok(self.node.node.control_owner())
    }
    /// Read the stored Form owner in this published snapshot segment.
    ///
    /// A live caller qualifies its access to the current segment. `step` admits
    /// the association read and each reached owner lookup.
    ///
    /// # Errors
    ///
    /// Returns the first admission or lookup failure, or
    /// [`DomError::WrongNodeKind`] if the stored owner is not a native Form.
    pub fn form_owner_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DomForm<'a>>, E> {
        self.node.stored_form_owner_with(step)
    }

    /// Visit the current API value's normalized UTF-16 units in order.
    ///
    /// `step` admits every raw unit, including any LF suppressed after CR.
    /// The visitor admits its own output work and backing. The first failed
    /// step or visit is returned unchanged. A live caller qualifies access
    /// to the current published segment.
    pub fn visit_value_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
        mut visit: impl FnMut(u16) -> Result<(), E>,
    ) -> Result<(), E> {
        visit_normalized_units(self.raw_units()?, step, |unit, _| visit(unit))
    }

    /// Count normalized UTF-16 units without materializing the API value.
    ///
    /// `step` admits each reached raw unit and its first refusal is returned
    /// unchanged. A live caller qualifies access to the published segment.
    pub fn text_length_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<usize, E> {
        normalized_length(self.raw_units()?, step)
    }

    fn raw_units(self) -> Result<&'a [u16], DomError> {
        self.node
            .as_element()
            .and_then(|element| element.textarea_state())
            .map(|state| state.raw_value().code_units())
            .ok_or(DomError::WrongNodeKind)
    }
    /// API value, with Infra newlines normalized; no submitted hard wrapping.
    pub fn value(
        self,
        limits: DomTextareaReadLimits,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        crate::internal::store::check_cancel(cancel)?;
        let mut read = Read {
            remaining: limits.work,
            limit: limits.work,
            cancel,
        };
        normalized_value(
            self.raw_units()?,
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
        crate::internal::store::check_cancel(cancel)?;
        let mut read = Read {
            remaining: limits.work,
            limit: limits.work,
            cancel,
        };
        let length = self.text_length_with(|| read.step())?;
        crate::internal::store::admit(length, limits.units, DomResource::StringCodeUnits)?;
        Ok(length)
    }
    /// Exact immediate child Text content; nested text does not contribute.
    pub fn default_value(
        self,
        limits: DomTextareaReadLimits,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        crate::internal::store::check_cancel(cancel)?;
        let mut read = Read {
            remaining: limits.work,
            limit: limits.work,
            cancel,
        };
        let mut length = 0usize;
        for &child in self.node.children() {
            read.step()?;
            if let Some(text) = self.node.snapshot.node(child)?.kind().text_data() {
                length = length
                    .checked_add(text.len())
                    .ok_or(DomError::RepresentationExhausted)?;
                crate::internal::store::admit(length, limits.units, DomResource::StringCodeUnits)?;
            }
        }
        read.step()?;
        let mut output = output(length, limits.units, None, cancel)?;
        for &child in self.node.children() {
            read.step()?;
            if let Some(text) = self.node.snapshot.node(child)?.kind().text_data() {
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
    budget: Option<&crate::internal::store::source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<Vec<u16>, DomError> {
    crate::internal::store::source::formation::utf16_buffer(length, limit, budget, cancel)
}
fn visit_normalized_units<E, Step: FnMut() -> Result<(), E>>(
    raw: &[u16],
    mut step: Step,
    mut visit: impl FnMut(u16, &mut Step) -> Result<(), E>,
) -> Result<(), E> {
    let mut normalizer = HtmlNewlineNormalizer::new();
    for unit in raw {
        step()?;
        if let Some(unit) = normalizer.push(*unit) {
            visit(unit, &mut step)?;
        }
    }
    Ok(())
}

pub(crate) fn normalized_length<E: From<DomError>>(
    raw: &[u16],
    step: impl FnMut() -> Result<(), E>,
) -> Result<usize, E> {
    let mut length = 0usize;
    visit_normalized_units(raw, step, |_, _| {
        length = length
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(())
    })?;
    Ok(length)
}
pub(crate) fn normalized_value(
    raw: &[u16],
    limit: usize,
    step: &mut impl FnMut() -> Result<(), DomError>,
    budget: Option<&crate::internal::store::source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    let length = normalized_length(raw, &mut *step)?;
    step()?;
    #[cfg(test)]
    let mut output_writes = crate::internal::store::CopyObservation::default();
    let mut output = output(length, limit, budget, cancel)?;
    visit_normalized_units(raw, step, |unit, step| {
        step()?;
        output.push(unit);
        #[cfg(test)]
        output_writes.pushed::<u16>();
        Ok(())
    })?;
    Ok(DocumentString::from_utf16(output))
}
