//! Immutable readback of the source select option list, HTML 877d6146487f.
//! This is not a live collection or a selection-reset operation. Ordinary
//! children and producer-established current selectedness are the only inputs.

use super::*;

/// Complete source-ordered option identities from one qualified query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSelectOptions {
    options: Vec<HtmlCaptureSubjectId>,
}
impl HtmlSelectOptions {
    pub fn options(&self) -> &[HtmlCaptureSubjectId] {
        &self.options
    }
}

#[derive(Clone, Copy)]
enum SelectWalkMode<'s> {
    All,
    Selected,
    FirstSelected,
    Target(&'s HtmlCaptureSubjectId),
}
struct SelectWalkResult {
    options: HtmlSelectOptions,
    index: Option<usize>,
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
    ) -> Result<HtmlSemanticValue<HtmlSelectOptions>, HtmlSemanticFailure> {
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
    ) -> Result<HtmlSemanticValue<HtmlSelectOptions>, HtmlSemanticFailure> {
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
            options: HtmlSelectOptions {
                options: Vec::new(),
            },
            index: None,
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
                    SelectWalkMode::Target(target) => {
                        self.charge(1)?;
                        if &id == target {
                            result.index = Some(index);
                            return Ok(HtmlSemanticValue::Known(result));
                        }
                        false
                    }
                    SelectWalkMode::Selected | SelectWalkMode::FirstSelected => {
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
