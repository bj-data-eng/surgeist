//! Captured dl name/value grouping, HTML 877d6146487f §4.4.9.
//! This computes associations, including malformed groups with an empty side;
//! it does not certify content conformance or compare human-readable names.

use super::*;

/// One source group of ordinary HTML dt names and dd values.
/// Either side can be empty. Identities retain the capture qualification;
/// their contents are not flattened or normalized by grouping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDescriptionListGroup {
    names: Vec<HtmlCaptureSubjectId>,
    values: Vec<HtmlCaptureSubjectId>,
}
impl HtmlDescriptionListGroup {
    pub fn names(&self) -> &[HtmlCaptureSubjectId] {
        &self.names
    }
    pub fn values(&self) -> &[HtmlCaptureSubjectId] {
        &self.values
    }
}

/// Complete ordered groups from the pinned dl grouping algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDescriptionListGroups {
    groups: Vec<HtmlDescriptionListGroup>,
}
impl HtmlDescriptionListGroups {
    pub fn groups(&self) -> &[HtmlDescriptionListGroup] {
        &self.groups
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn description_list_groups(
        &mut self,
        node: HtmlCapturedNode<'a>,
        element: &HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlDescriptionListGroups>, HtmlSemanticFailure> {
        if self.known(element)? != Some(KnownHtmlElement::Dl) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(mut children) = node.children() else {
            return Ok(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OrderedChildren(node.id()),
            ));
        };
        self.charge(1)?;
        let mut groups = HtmlDescriptionListGroups { groups: Vec::new() };
        let mut current = HtmlDescriptionListGroup {
            names: Vec::new(),
            values: Vec::new(),
        };
        let mut seen_dd = false;
        loop {
            self.charge(1)?;
            let Some(id) = children.next() else { break };
            let child = self.node(&id)?;
            let kind = match child.kind() {
                HtmlCapturedNodeKind::Element(element) => self.known(element)?,
                _ => None,
            };
            if kind == Some(KnownHtmlElement::Div) {
                self.charge(1)?;
                let HtmlCapturedChildren::Complete(mut grandchildren) = child.children() else {
                    return Ok(HtmlSemanticValue::Missing(
                        HtmlSemanticMissing::OrderedChildren(id),
                    ));
                };
                loop {
                    self.charge(1)?;
                    let Some(id) = grandchildren.next() else {
                        break;
                    };
                    let grandchild = self.node(&id)?;
                    let kind = match grandchild.kind() {
                        HtmlCapturedNodeKind::Element(element) => self.known(element)?,
                        _ => None,
                    };
                    self.description_list_member(
                        id,
                        kind,
                        &mut groups,
                        &mut current,
                        &mut seen_dd,
                    )?;
                }
            } else {
                self.description_list_member(id, kind, &mut groups, &mut current, &mut seen_dd)?;
            }
        }
        self.charge(2)?;
        if !current.names.is_empty() || !current.values.is_empty() {
            self.reserve_nested_output(&mut groups.groups, 1)?;
            self.charge(1)?;
            groups.groups.push(current);
        }
        self.poll()?;
        Ok(HtmlSemanticValue::Known(groups))
    }

    // HTML #process-dt-or-dd. The one current/seenDd state deliberately spans
    // direct div boundaries; div is only a one-level traversal rule.
    fn description_list_member(
        &mut self,
        id: HtmlCaptureSubjectId,
        kind: Option<KnownHtmlElement>,
        groups: &mut HtmlDescriptionListGroups,
        current: &mut HtmlDescriptionListGroup,
        seen_dd: &mut bool,
    ) -> Result<(), HtmlSemanticFailure> {
        self.charge(1)?;
        match kind {
            Some(KnownHtmlElement::Dt) => {
                self.charge(1)?;
                if *seen_dd {
                    self.reserve_nested_output(&mut groups.groups, 1)?;
                    self.charge(1)?;
                    groups.groups.push(std::mem::replace(
                        current,
                        HtmlDescriptionListGroup {
                            names: Vec::new(),
                            values: Vec::new(),
                        },
                    ));
                    self.charge(1)?;
                    *seen_dd = false;
                }
                self.reserve_nested_output(&mut current.names, 1)?;
                self.charge(1)?;
                current.names.push(id);
            }
            Some(KnownHtmlElement::Dd) => {
                self.reserve_nested_output(&mut current.values, 1)?;
                self.charge(1)?;
                current.values.push(id);
                self.charge(1)?;
                *seen_dd = true;
            }
            _ => {}
        }
        Ok(())
    }
}
