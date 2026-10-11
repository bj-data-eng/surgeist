//! Immutable datalist descendant membership, HTML 877d6146487f.
//! Association lookup lives with the shared control first-ID owner. This kernel
//! does not rank/render suggestions, expose a live collection or set values.

use super::*;

#[derive(Clone, Copy)]
enum DatalistFilter {
    All,
    Suggestions,
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn datalist_options(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        suggestions: bool,
    ) -> Result<HtmlSemanticValue<HtmlOptionCollection>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Datalist) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let filter = if suggestions {
            DatalistFilter::Suggestions
        } else {
            DatalistFilter::All
        };
        let mut frames = Vec::new();
        let result = self.datalist_options_inner(subject, filter, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }

    fn datalist_options_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        filter: DatalistFilter,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<HtmlOptionCollection>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(children) = node.children() else {
            return Ok(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OrderedChildren(subject.clone()),
            ));
        };
        let mut result = HtmlOptionCollection {
            options: Vec::new(),
        };
        if children.len() == 0 {
            return Ok(HtmlSemanticValue::Known(result));
        }
        self.reserve_scratch(frames, 1)?;
        self.charge(1)?;
        frames.push(children);
        while !frames.is_empty() {
            self.charge(1)?;
            let Some(id) = frames.last_mut().expect("nonempty frame stack").next() else {
                self.charge(1)?;
                frames.pop();
                continue;
            };
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Option)
            {
                let keep = match filter {
                    DatalistFilter::All => true,
                    DatalistFilter::Suggestions => match self.datalist_suggestion(&id)? {
                        HtmlSemanticValue::Known(keep) => keep,
                        HtmlSemanticValue::Missing(missing) => {
                            return Ok(HtmlSemanticValue::Missing(missing));
                        }
                        HtmlSemanticValue::Inapplicable => {
                            return Ok(HtmlSemanticValue::Inapplicable);
                        }
                        HtmlSemanticValue::Unsupported(reason) => {
                            return Ok(HtmlSemanticValue::Unsupported(reason));
                        }
                    },
                };
                if keep {
                    self.reserve_nested_output(&mut result.options, 1)?;
                    self.charge(1)?;
                    result.options.push(id.clone());
                }
            }
            // The collection is all ordinary descendants. No select-list or
            // text-content pruning, even for an ineligible outer option.
            if matches!(
                node.kind(),
                HtmlCapturedNodeKind::Element(_)
                    | HtmlCapturedNodeKind::Document(_)
                    | HtmlCapturedNodeKind::Fragment(_)
            ) {
                self.charge(1)?;
                let HtmlCapturedChildren::Complete(children) = node.children() else {
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
        }
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(result))
    }

    fn datalist_suggestion(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        match self.option_disabled(subject)? {
            HtmlSemanticValue::Known(true) => Ok(HtmlSemanticValue::Known(false)),
            HtmlSemanticValue::Known(false) => {
                let value =
                    self.option_readback(subject, HtmlOptionReadbackOrigin::ValueAttribute)?;
                self.charge(1)?;
                Ok(value.map(|value| !value.value().is_empty()))
            }
            HtmlSemanticValue::Missing(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            HtmlSemanticValue::Inapplicable => Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(reason) => Ok(HtmlSemanticValue::Unsupported(reason)),
        }
    }
}
