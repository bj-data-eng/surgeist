//! Captured advisory information, HTML 877d6146487f #advisory-information.
//! Own title is normalized; parent-element recursion excludes host/slot edges.
//! The specific link/style noninheritance clauses stop generic recursion when
//! either HTML element has no own title. This selected reconciliation does not
//! compute title-of-link, stylesheet-set names or tooltip/accessibility policy.

use super::*;

/// Normalized advisory information and its actual selecting attribute holder.
///
/// A selected empty title retains `Some(source)` and suppresses inheritance.
/// `None` establishes that no title was selected before an ordinary-parent or
/// title-less HTML link/style boundary. Exact raw text remains in the capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAdvisoryInformation {
    value: DocumentString,
    source: Option<HtmlCaptureSubjectId>,
}
impl HtmlAdvisoryInformation {
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    pub fn source(&self) -> Option<&HtmlCaptureSubjectId> {
        self.source.as_ref()
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn advisory_information(
        &mut self,
        mut node: HtmlCapturedNode<'a>,
        mut element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlAdvisoryInformation>, HtmlSemanticFailure> {
        // The query front door qualified the initial subject's HTML namespace.
        let mut html = true;
        loop {
            if html {
                let raw = self.attribute(element, "title")?;
                self.charge(1)?;
                if let Some(raw) = raw {
                    self.charge(1)?;
                    self.interpreter
                        .admit_attribute_raw(Some(raw.value()))
                        .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
                    let value = self.advisory_normalize(raw.value())?;
                    self.charge(1)?;
                    return Ok(HtmlSemanticValue::Known(HtmlAdvisoryInformation {
                        value,
                        source: Some(node.id()),
                    }));
                }
                if self.advisory_title_boundary(element)? {
                    return self.advisory_empty();
                }
            }
            let parent = match self.parent(node)? {
                Ok(Some(parent)) => parent,
                Ok(None) => return self.advisory_empty(),
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            node = self.node(&parent)?;
            self.charge(1)?;
            let HtmlCapturedNodeKind::Element(parent_element) = node.kind() else {
                return self.advisory_empty();
            };
            element = parent_element;
            html = self.html_namespace(element)?;
        }
    }

    fn advisory_title_boundary(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        let units = element.name().local_name().code_units();
        let expected: &[u8] = match units.len() {
            4 => b"link",
            5 => b"style",
            _ => return Ok(false),
        };
        for (unit, byte) in units.iter().zip(expected) {
            self.charge(1)?;
            if *unit != u16::from(*byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn advisory_empty(
        &mut self,
    ) -> Result<HtmlSemanticValue<HtmlAdvisoryInformation>, HtmlSemanticFailure> {
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlAdvisoryInformation {
            value: self.context.empty_language().clone(),
            source: None,
        }))
    }

    fn advisory_normalize(
        &mut self,
        raw: &DocumentString,
    ) -> Result<DocumentString, HtmlSemanticFailure> {
        let mut units = Vec::new();
        let mut after_cr = false;
        for unit in raw.code_units() {
            self.charge(1)?;
            let unit = *unit;
            if after_cr && unit == 10 {
                after_cr = false;
                continue;
            }
            after_cr = unit == 13;
            self.reserve_nested_output(&mut units, 1)?;
            self.charge(1)?;
            units.push(if after_cr { 10 } else { unit });
        }
        self.charge(1)?;
        Ok(if units.is_empty() {
            self.context.empty_language().clone()
        } else {
            // Transfer the admitted Vec without variable copying. As with the
            // existing semantic text getter, std Arc's fixed allocation is not
            // a recoverable allocator/header guarantee.
            DocumentString::from_utf16(units)
        })
    }
}
