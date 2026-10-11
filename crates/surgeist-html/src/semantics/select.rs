//! Immutable readback of select option lists and option form ownership, HTML 877d6146487f.
//! This is not a live collection or a selection-reset operation. Ordinary
//! relations, current selectedness and qualified form associations are consumed
//! only by the algorithms that require them.

use super::*;

/// An option form getter observation with the selecting evidence retained.
/// A null nearest select differs from a select with an established null form
/// owner. The latter retains the actual/reset association origin. No current
/// association is inferred from markup or ownerDocument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlOptionFormOwner {
    selection: OptionFormSelection,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum OptionFormSelection {
    NoSelect,
    Select {
        subject: HtmlCaptureSubjectId,
        owner: HtmlFormOwner,
    },
}
impl HtmlOptionFormOwner {
    pub fn select(&self) -> Option<&HtmlCaptureSubjectId> {
        match &self.selection {
            OptionFormSelection::NoSelect => None,
            OptionFormSelection::Select { subject, .. } => Some(subject),
        }
    }
    /// Returns the select's complete association, including a known null owner.
    /// None means that no nearest select exists, not unavailable association.
    pub fn form_owner(&self) -> Option<&HtmlFormOwner> {
        match &self.selection {
            OptionFormSelection::NoSelect => None,
            OptionFormSelection::Select { owner, .. } => Some(owner),
        }
    }
}

#[derive(Clone, Copy)]
enum SelectWalkMode<'s> {
    All,
    Selected,
    FirstSelected,
    FirstSelectedIdentity,
    FirstOptionIdentity,
    Target(&'s HtmlCaptureSubjectId),
}
struct SelectWalkResult {
    options: HtmlOptionCollection,
    index: Option<usize>,
    first_selected: Option<HtmlCaptureSubjectId>,
    first_option: Option<SelectFirstOption>,
}
struct SelectFirstOption {
    subject: HtmlCaptureSubjectId,
    inside_optgroup: bool,
}
struct SelectFrame<'a> {
    children: HtmlCapturedChildIds<'a>,
    inside_optgroup: bool,
}

impl<'a> HtmlSemanticSession<'a> {
    fn select_applicable<T>(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        kind: KnownHtmlElement,
    ) -> Result<Option<HtmlSemanticValue<T>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(Some(HtmlSemanticValue::Inapplicable));
        };
        if !self.html_namespace(element)? {
            return Ok(Some(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            )));
        }
        Ok(if self.known(element)? == Some(kind) {
            None
        } else {
            Some(HtmlSemanticValue::Inapplicable)
        })
    }

    pub(super) fn select_options(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlOptionCollection>, HtmlSemanticFailure> {
        if let Some(value) = self.select_applicable(subject, KnownHtmlElement::Select)? {
            return Ok(value);
        }
        Ok(self
            .walk_select(subject, SelectWalkMode::All)?
            .map(|result| result.options))
    }
    pub(super) fn selected_options(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlOptionCollection>, HtmlSemanticFailure> {
        if let Some(value) = self.select_applicable(subject, KnownHtmlElement::Select)? {
            return Ok(value);
        }
        Ok(self
            .walk_select(subject, SelectWalkMode::Selected)?
            .map(|result| result.options))
    }
    pub(super) fn select_selected_index(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<usize>>, HtmlSemanticFailure> {
        if let Some(value) = self.select_applicable(subject, KnownHtmlElement::Select)? {
            return Ok(value);
        }
        Ok(self
            .walk_select(subject, SelectWalkMode::FirstSelected)?
            .map(|result| result.index))
    }
    pub(super) fn option_index(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<usize>, HtmlSemanticFailure> {
        if let Some(value) = self.select_applicable(subject, KnownHtmlElement::Option)? {
            return Ok(value);
        }
        match self.nearest_select(subject)? {
            Err(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            Ok(None) => Ok(HtmlSemanticValue::Known(0)),
            Ok(Some(select)) => Ok(self
                .walk_select(&select, SelectWalkMode::Target(subject))?
                .map(|result| result.index.unwrap_or(0))),
        }
    }

    pub(super) fn option_form_owner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlOptionFormOwner>, HtmlSemanticFailure> {
        if let Some(value) = self.select_applicable(subject, KnownHtmlElement::Option)? {
            return Ok(value);
        }
        match self.nearest_select(subject)? {
            Err(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            Ok(None) => {
                self.charge(1)?;
                Ok(HtmlSemanticValue::Known(HtmlOptionFormOwner {
                    selection: OptionFormSelection::NoSelect,
                }))
            }
            Ok(Some(select)) => {
                let owner = self.form_owner(&select)?;
                self.charge(1)?;
                Ok(owner.map(|owner| HtmlOptionFormOwner {
                    selection: OptionFormSelection::Select {
                        subject: select,
                        owner,
                    },
                }))
            }
        }
    }

    // The value getter consumes the same list/pruning/selectedness owner but
    // needs the identity rather than materializing a complete option list.
    pub(super) fn first_selected_option(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        if let Some(value) = self.select_applicable(subject, KnownHtmlElement::Select)? {
            return Ok(value);
        }
        Ok(self
            .walk_select(subject, SelectWalkMode::FirstSelectedIdentity)?
            .map(|result| result.first_selected))
    }

    pub(super) fn select_placeholder_label_option(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        if let Some(value) = self.select_applicable(subject, KnownHtmlElement::Select)? {
            return Ok(value);
        }
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            unreachable!("select_applicable established an element");
        };
        // The source consumes Boolean presence, not its authored spelling.
        if self.attribute(element, "required")?.is_none() {
            return Ok(HtmlSemanticValue::Known(None));
        }
        let size = self.attribute(element, "size")?;
        let projected_size = if let Some(size) = size {
            self.interpreter
                .admit_attribute_raw(Some(size.value()))
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
            // Same allocation-free nonnegative recovery owner used for spans.
            // Its two scans are one input-bounded atomic semantic operation.
            let work = size
                .value()
                .len()
                .checked_mul(2)
                .and_then(|n| n.checked_add(8))
                .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
            self.charge(work)?;
            let value = crate::numeric::parse_nonnegative_integer_capped(size.value(), 2);
            self.poll()?;
            value.ok()
        } else {
            None
        };
        let size_is_one = match projected_size {
            Some(value) => value == 1,
            None => self.attribute(element, "multiple")?.is_none(),
        };
        self.charge(1)?;
        if !size_is_one {
            return Ok(HtmlSemanticValue::Known(None));
        }
        match self.walk_select(subject, SelectWalkMode::FirstOptionIdentity)? {
            HtmlSemanticValue::Known(result) => {
                let Some(first) = result.first_option else {
                    return Ok(HtmlSemanticValue::Known(None));
                };
                self.charge(1)?;
                if first.inside_optgroup {
                    return Ok(HtmlSemanticValue::Known(None));
                }
                let value =
                    self.option_readback(&first.subject, HtmlOptionReadbackOrigin::ValueAttribute)?;
                self.charge(1)?;
                Ok(value.map(|readback| {
                    // Dropping this intermediate does not undo its cumulative
                    // output-capacity or observed-byte admission.
                    readback.value().is_empty().then_some(first.subject)
                }))
            }
            HtmlSemanticValue::Missing(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            HtmlSemanticValue::Inapplicable => Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(reason) => Ok(HtmlSemanticValue::Unsupported(reason)),
        }
    }

    fn walk_select(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        mode: SelectWalkMode<'_>,
    ) -> Result<HtmlSemanticValue<SelectWalkResult>, HtmlSemanticFailure> {
        let mut frames = Vec::new();
        let result = self.walk_select_inner(subject, mode, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }

    fn walk_select_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        mode: SelectWalkMode<'_>,
        frames: &mut Vec<SelectFrame<'a>>,
    ) -> Result<HtmlSemanticValue<SelectWalkResult>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(children) = node.children() else {
            return Ok(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OrderedChildren(subject.clone()),
            ));
        };
        let mut result = SelectWalkResult {
            options: HtmlOptionCollection {
                options: Vec::new(),
            },
            index: None,
            first_selected: None,
            first_option: None,
        };
        // Established empty needs neither a frame allocation nor option state.
        if children.len() == 0 {
            return Ok(HtmlSemanticValue::Known(result));
        }
        self.reserve_scratch(frames, 1)?;
        self.charge(1)?;
        frames.push(SelectFrame {
            children,
            inside_optgroup: false,
        });
        let mut index = 0usize;
        while !frames.is_empty() {
            self.charge(1)?;
            let frame = frames.last_mut().expect("nonempty frame stack");
            let inside_optgroup = frame.inside_optgroup;
            let Some(id) = frame.children.next() else {
                self.charge(1)?;
                frames.pop();
                continue;
            };
            let node = self.node(&id)?;
            let kind = match node.kind() {
                HtmlCapturedNodeKind::Element(element) => self.known(element)?,
                _ => None,
            };
            if kind == Some(KnownHtmlElement::Option) {
                let keep = match mode {
                    SelectWalkMode::All => true,
                    SelectWalkMode::FirstOptionIdentity => {
                        self.charge(1)?;
                        result.first_option = Some(SelectFirstOption {
                            subject: id,
                            inside_optgroup,
                        });
                        return Ok(HtmlSemanticValue::Known(result));
                    }
                    SelectWalkMode::Target(target) => {
                        self.charge(1)?;
                        if &id == target {
                            result.index = Some(index);
                            return Ok(HtmlSemanticValue::Known(result));
                        }
                        false
                    }
                    SelectWalkMode::Selected
                    | SelectWalkMode::FirstSelected
                    | SelectWalkMode::FirstSelectedIdentity => {
                        match self.control_flag(&id, HtmlControlFlag::Selectedness)? {
                            HtmlSemanticValue::Known(selected) => selected,
                            HtmlSemanticValue::Missing(missing) => {
                                return Ok(HtmlSemanticValue::Missing(missing));
                            }
                            HtmlSemanticValue::Inapplicable => {
                                return Ok(HtmlSemanticValue::Inapplicable);
                            }
                            HtmlSemanticValue::Unsupported(reason) => {
                                return Ok(HtmlSemanticValue::Unsupported(reason));
                            }
                        }
                    }
                };
                if keep {
                    if matches!(mode, SelectWalkMode::FirstSelected) {
                        self.charge(1)?;
                        result.index = Some(index);
                        return Ok(HtmlSemanticValue::Known(result));
                    }
                    if matches!(mode, SelectWalkMode::FirstSelectedIdentity) {
                        self.charge(1)?;
                        result.first_selected = Some(id);
                        return Ok(HtmlSemanticValue::Known(result));
                    }
                    self.reserve_nested_output(&mut result.options.options, 1)?;
                    self.charge(1)?;
                    result.options.options.push(id);
                }
                self.charge(1)?;
                index = index
                    .checked_add(1)
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
                continue; // Option descendants never belong to this list.
            }
            if matches!(
                kind,
                Some(KnownHtmlElement::Select | KnownHtmlElement::Hr | KnownHtmlElement::Datalist)
            ) || (kind == Some(KnownHtmlElement::Optgroup) && inside_optgroup)
            {
                continue;
            }
            if !matches!(
                node.kind(),
                HtmlCapturedNodeKind::Element(_)
                    | HtmlCapturedNodeKind::Document(_)
                    | HtmlCapturedNodeKind::Fragment(_)
            ) {
                continue;
            }
            self.charge(1)?;
            let HtmlCapturedChildren::Complete(children) = node.children() else {
                return Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::OrderedChildren(id),
                ));
            };
            if children.len() != 0 {
                self.reserve_scratch(frames, 1)?;
                self.charge(1)?;
                frames.push(SelectFrame {
                    children,
                    inside_optgroup: inside_optgroup || kind == Some(KnownHtmlElement::Optgroup),
                });
            }
        }
        Ok(HtmlSemanticValue::Known(result))
    }
}
