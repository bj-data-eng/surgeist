//! Immutable option/select string getter readbacks, HTML 877d6146487f.
//! These operations do not perform setters, selection resets or rendering.

use super::*;
use ascii_whitespace::AsciiWhitespaceText;
use scan::name_part_is_with;
use surgeist_html::{
    values::{DocumentString, HtmlNameInput},
    vocabulary::KnownHtmlElement,
};

/// Streaming HTML-aware Option text with image alt text excluded.
///
/// Retain one instance across all reached Text nodes. The caller owns the
/// ordinary descendant walk and admits each input unit before feeding it.
#[derive(Debug, Default)]
pub struct HtmlOptionText {
    text: AsciiWhitespaceText,
}

impl HtmlOptionText {
    const HTML_EXCLUSIONS: [KnownHtmlElement; 2] =
        [KnownHtmlElement::Script, KnownHtmlElement::Img];

    /// Admit the reached name comparisons and decide whether to skip this
    /// descendant Element and its subtree. Prefixes are immaterial.
    pub fn prunes_descendants_with<E>(
        name: &(impl HtmlNameInput + ?Sized),
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let mut known = None;
        if name_part_is_with(
            || name.namespace_uri(),
            scan::HTML_NAMESPACE.as_bytes(),
            &mut step,
        )? {
            for tag in Self::HTML_EXCLUSIONS {
                if name_part_is_with(
                    || Some(name.local_name()),
                    tag.local_name().as_bytes(),
                    &mut step,
                )? {
                    known = Some(tag);
                    break;
                }
            }
        }
        Self::prunes_after_recognition(name, known, step)
    }

    fn prunes_after_recognition<E>(
        name: &(impl HtmlNameInput + ?Sized),
        known: Option<KnownHtmlElement>,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        if known.is_some_and(|tag| Self::HTML_EXCLUSIONS.contains(&tag)) {
            return Ok(true);
        }
        step()?;
        let Some(uri) = name.namespace_uri() else {
            return Ok(false);
        };
        Ok(
            name_part_is_with(|| Some(uri), b"http://www.w3.org/2000/svg", &mut step)?
                && name_part_is_with(|| Some(name.local_name()), b"script", step)?,
        )
    }

    /// Feed an admitted input unit. Emit one chunk of one or two exact UTF-16
    /// units when there is output. The caller admits and stores the chunk.
    /// A refused output ends the walk.
    pub fn push<E>(
        &mut self,
        unit: u16,
        emit: impl FnOnce(&[u16]) -> Result<(), E>,
    ) -> Result<(), E> {
        let (units, len) = self.text.push(unit);
        if len == 0 {
            return Ok(());
        }
        emit(&units[..len])
    }
}

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
            self.option_string_copy(attribute.value())?;
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
        self.release_scratch(&frames);
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
        let mut normalized = HtmlOptionText::default();
        if children.len() != 0 {
            self.reserve_scratch(frames, 1)?;
            self.charge(1)?;
            frames.push(children);
        }
        while let Some(frame) = frames.last_mut() {
            self.charge(1)?;
            let Some(id) = frame.next() else {
                self.charge(1)?;
                frames.pop();
                continue;
            };
            let descendant = self.node(&id)?;
            match descendant.kind() {
                HtmlCapturedNodeKind::Text(text) => {
                    for &unit in text.code_units() {
                        self.charge(1)?;
                        normalized.push(unit, |output| {
                            self.reserve_nested_output(&mut units, output.len())?;
                            for &unit in output {
                                self.charge(1)?;
                                units.push(unit);
                            }
                            Ok(())
                        })?;
                    }
                    continue;
                }
                HtmlCapturedNodeKind::Element(element) => {
                    let known = self.known(element)?;
                    if HtmlOptionText::prunes_after_recognition(element.name(), known, || {
                        self.charge(1)
                    })? {
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
            self.option_string_copy(self.context.empty_language())?;
            self.context.empty_language().clone()
        } else {
            // The admitted Vec is moved; only the fixed immutable Arc wrapper
            // is created here. No second variable UTF-16 buffer is allocated.
            self.option_string_header()?;
            DocumentString::from_utf16(units)
        }))
    }
}
