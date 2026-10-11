//! Immutable option/select string getter readbacks, HTML 877d6146487f.
//! These operations do not perform setters, selection resets or rendering.

use super::*;

/// The source selected by an option string getter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOptionReadbackOrigin {
    LabelAttribute,
    ValueAttribute,
    DescendantText,
}

/// Exact option string with its selecting source.
///
/// A present label/value attribute, including empty, is returned unchanged.
/// Descendant text uses HTML-aware text content with alt text excluded and
/// only ASCII whitespace stripped/collapsed. This is an immutable getter
/// observation, not a live IDL object or a general rendering label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlOptionReadback {
    value: DocumentString,
    origin: HtmlOptionReadbackOrigin,
}
impl HtmlOptionReadback {
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    pub fn origin(&self) -> HtmlOptionReadbackOrigin {
        self.origin
    }
}

/// Value of the first currently selected source-list option, or known empty.
/// `selected_option=None` proves no selected option; `Some` with empty value
/// distinguishes a selected option whose value is empty.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSelectValue {
    value: DocumentString,
    selected_option: Option<HtmlCaptureSubjectId>,
}
impl HtmlSelectValue {
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    pub fn selected_option(&self) -> Option<&HtmlCaptureSubjectId> {
        self.selected_option.as_ref()
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn option_readback(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        requested: HtmlOptionReadbackOrigin,
    ) -> Result<HtmlSemanticValue<HtmlOptionReadback>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Option) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let attribute = match requested {
            HtmlOptionReadbackOrigin::LabelAttribute => self.attribute(element, "label")?,
            HtmlOptionReadbackOrigin::ValueAttribute => self.attribute(element, "value")?,
            HtmlOptionReadbackOrigin::DescendantText => None,
        };
        if let Some(attribute) = attribute {
            self.interpreter
                .admit_attribute_raw(Some(attribute.value()))
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
            self.charge(1)?;
            return Ok(HtmlSemanticValue::Known(HtmlOptionReadback {
                value: attribute.value().clone(),
                origin: requested,
            }));
        }
        Ok(self
            .html_aware_option_text(subject)?
            .map(|value| HtmlOptionReadback {
                value,
                origin: HtmlOptionReadbackOrigin::DescendantText,
            }))
    }

    pub(super) fn select_value(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlSelectValue>, HtmlSemanticFailure> {
        match self.first_selected_option(subject)? {
            HtmlSemanticValue::Known(Some(option)) => Ok(self
                .option_readback(&option, HtmlOptionReadbackOrigin::ValueAttribute)?
                .map(|readback| HtmlSelectValue {
                    value: readback.value,
                    selected_option: Some(option),
                })),
            HtmlSemanticValue::Known(None) => {
                self.charge(1)?;
                Ok(HtmlSemanticValue::Known(HtmlSelectValue {
                    value: self.context.empty_language().clone(),
                    selected_option: None,
                }))
            }
            HtmlSemanticValue::Missing(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            HtmlSemanticValue::Inapplicable => Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(reason) => Ok(HtmlSemanticValue::Unsupported(reason)),
        }
    }

    fn html_aware_option_text(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<DocumentString>, HtmlSemanticFailure> {
        let mut frames = Vec::new();
        let result = self.html_aware_option_text_inner(subject, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }

    fn html_aware_option_text_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<DocumentString>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(children) = node.children() else {
            return Ok(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OrderedChildren(subject.clone()),
            ));
        };
        let mut units = Vec::new();
        let mut pending_space = false;
        if children.len() != 0 {
            self.reserve_scratch(frames, 1)?;
            self.charge(1)?;
            frames.push(children);
        }
        while !frames.is_empty() {
            self.charge(1)?;
            let Some(id) = frames.last_mut().expect("nonempty frame stack").next() else {
                self.charge(1)?;
                frames.pop();
                continue;
            };
            let descendant = self.node(&id)?;
            match descendant.kind() {
                HtmlCapturedNodeKind::Text(text) => {
                    for unit in text.code_units() {
                        self.charge(1)?;
                        if matches!(*unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20) {
                            pending_space = !units.is_empty();
                        } else {
                            self.reserve_nested_output(
                                &mut units,
                                if pending_space { 2 } else { 1 },
                            )?;
                            if pending_space {
                                self.charge(1)?;
                                units.push(0x20);
                                pending_space = false;
                            }
                            self.charge(1)?;
                            units.push(*unit);
                        }
                    }
                    continue;
                }
                HtmlCapturedNodeKind::Element(element) => {
                    if matches!(
                        self.known(element)?,
                        Some(KnownHtmlElement::Script | KnownHtmlElement::Img)
                    ) || self.option_text_is_svg_script(element)?
                    {
                        continue;
                    }
                }
                HtmlCapturedNodeKind::Document(_) | HtmlCapturedNodeKind::Fragment(_) => {}
                _ => continue,
            }
            self.charge(1)?;
            let HtmlCapturedChildren::Complete(children) = descendant.children() else {
                return Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::OrderedChildren(id),
                ));
            };
            if children.len() != 0 {
                self.reserve_scratch(frames, 1)?;
                self.charge(1)?;
                frames.push(children);
            }
        }
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(if units.is_empty() {
            self.context.empty_language().clone()
        } else {
            // The admitted Vec is moved; only the fixed immutable Arc wrapper
            // is created here. No second variable UTF-16 buffer is allocated.
            DocumentString::from_utf16(units)
        }))
    }

    fn option_text_is_svg_script(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        let Some(uri) = element.name().namespace().uri() else {
            return Ok(false);
        };
        Ok(
            self.option_text_exact_ascii(uri, "http://www.w3.org/2000/svg")?
                && self.option_text_exact_ascii(element.name().local_name(), "script")?,
        )
    }

    fn option_text_exact_ascii(
        &mut self,
        value: &DocumentString,
        expected: &str,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        if value.len() != expected.len() {
            return Ok(false);
        }
        for (unit, byte) in value.code_units().iter().zip(expected.bytes()) {
            self.charge(1)?;
            if *unit != u16::from(byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
