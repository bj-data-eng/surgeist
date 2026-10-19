//! HTML877d template-for author relation and find-markers selection.
//! This immutable captured operation does not execute content patching.
use super::*;
use crate::HtmlCapturedProcessingInstruction;
use crate::bounded::{
    HtmlPiPseudoAttributeAction as Action, HtmlPiPseudoAttributeCursor as Cursor,
    HtmlPiPseudoAttributeField as Field, HtmlPiPseudoAttributeName as Name,
    HtmlPiPseudoAttributeReply as Reply, HtmlTemplateMarkerFirst as First,
    HtmlTemplateMarkerSiblings as Siblings, HtmlTemplateMarkerTarget as Target,
    template_marker_first,
};
use std::ops::Range;

/// A sufficient same-tree name witness; Match does not promise first selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTemplateForNameRelation {
    NotSpecified,
    Match(HtmlCaptureSubjectId),
    NoMatch,
    Missing(HtmlSemanticMissing),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    fn fixture(point: CancellationPoint) -> (HtmlCapture, HtmlCaptureSubjectId) {
        let mut b = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let d = b
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let t = b
            .add_element(
                &d,
                QualifiedName::new(Namespace::html(), None, "template"),
                OrderedAttributes::new(vec![Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "for"),
                    "X",
                )])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let p = b
            .add_processing_instruction(
                &d,
                (if point == CancellationPoint::SiblingStep {
                    "start"
                } else {
                    "marker"
                })
                .into(),
                (if point == CancellationPoint::DuplicateRead {
                    "name='X' x='1' x='2'"
                } else {
                    "name='X'"
                })
                .into(),
            )
            .unwrap();
        let mut ids = vec![p.clone()];
        if point == CancellationPoint::UnknownSkipped {
            let unknown = b
                .add_element(
                    &d,
                    QualifiedName::new(Namespace::html(), None, "div"),
                    OrderedAttributes::new(vec![]).unwrap(),
                    HtmlCapturedIsValue::Unavailable,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            ids.insert(0, unknown);
        }
        if point == CancellationPoint::SiblingStep {
            ids.push(
                b.add_processing_instruction(&d, "start".into(), "name='other'".into())
                    .unwrap(),
            );
            ids.push(
                b.add_processing_instruction(&d, "end".into(), "".into())
                    .unwrap(),
            );
            ids.push(
                b.add_processing_instruction(&d, "end".into(), "".into())
                    .unwrap(),
            );
        }
        b.establish_children(&t, &[]).unwrap();
        ids.push(t.clone());
        b.establish_children(&d, &ids).unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&t).unwrap();
        (c, id)
    }
    #[test]
    fn cancellation_at_real_grammar_duplicate_sibling_and_author_continuation_actions() {
        for point in [
            CancellationPoint::PairCommitted,
            CancellationPoint::DuplicateRead,
            CancellationPoint::SiblingStep,
            CancellationPoint::UnknownSkipped,
        ] {
            let (c, id) = fixture(point);
            let cx = HtmlSemanticContext::new(
                &c,
                c.host_facts_revision().clone(),
                HtmlSemanticContextInput::new(vec![]),
                0,
            )
            .unwrap();
            let cancel = HtmlSemanticCancellation::new();
            let mut s = HtmlSemanticSession::new(
                &c,
                &cx,
                HtmlValueInterpreter::new(10000),
                HtmlSemanticLimits::new(1000000, 10, 100, 0, 100000, 100),
                cancel.clone(),
            )
            .unwrap();
            s.template_cancellation = Some((point, cancel.clone()));
            let q = [HtmlSemanticRequest::new(
                id.clone(),
                HtmlSemanticQuery::TemplateForReference,
            )];
            let first = s.query(&q).unwrap_err();
            assert_eq!(first.kind(), &HtmlSemanticFailureKind::Cancelled);
            assert!(cancel.is_cancelled());
            assert!(s.template_cancellation.is_none());
            assert!(first.usage().peak_scratch_records() > 0);
            assert_eq!(s.usage().scratch_records(), 0);
            let usage = s.usage();
            assert_eq!(s.query(&q).unwrap_err(), first);
            assert_eq!(s.usage(), usage);
            assert!(matches!(
                c.node(&id).unwrap().kind(),
                HtmlCapturedNodeKind::Element(_)
            ));
        }
    }
    #[test]
    fn known_start_with_unavailable_siblings_keeps_the_real_dependency() {
        let mut b = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(10, 100, 10, 1000, 10, 0, 0, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let d = b
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let p = b
            .add_processing_instruction(&d, "start".into(), "name='X'".into())
            .unwrap();
        b.establish_parent(&p, &d).unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&p).unwrap();
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]),
            0,
        )
        .unwrap();
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(100),
            HtmlSemanticLimits::new(10000, 10, 10, 0, 1000, 10),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        assert_eq!(
            s.template_end(&id).unwrap(),
            Err(HtmlSemanticMissing::OrderedChildren(
                c.subject_id(&d).unwrap()
            ))
        );
    }
}
/// Preparation eligibility and exact first marker/range; no nodes are removed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTemplateForSelection {
    NotSpecified,
    EmptyName,
    NoMatch,
    Marker(HtmlCaptureSubjectId),
    Start {
        start: HtmlCaptureSubjectId,
        end: Option<HtmlCaptureSubjectId>,
    },
    Missing(HtmlSemanticMissing),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTemplateForReport {
    raw: Option<Attribute>,
    name_relation: HtmlTemplateForNameRelation,
    selection: HtmlTemplateForSelection,
}
impl HtmlTemplateForReport {
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.raw.as_ref()
    }
    pub fn name_relation(&self) -> &HtmlTemplateForNameRelation {
        &self.name_relation
    }
    pub fn selection(&self) -> &HtmlTemplateForSelection {
        &self.selection
    }
}
#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum CancellationPoint {
    PairCommitted,
    DuplicateRead,
    SiblingStep,
    UnknownSkipped,
}
impl<'a> HtmlSemanticSession<'a> {
    #[cfg(test)]
    fn template_cancel(&mut self, point: CancellationPoint) {
        if self
            .template_cancellation
            .as_ref()
            .is_some_and(|(expected, _)| *expected == point)
        {
            let (_, cancel) = self
                .template_cancellation
                .take()
                .expect("observed fixed fixture");
            cancel.cancel();
        }
    }
    fn template_raw(&mut self, value: &DocumentString) -> Result<(), HtmlSemanticFailure> {
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(Some(value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))
    }
    fn template_target(
        &mut self,
        pi: &HtmlCapturedProcessingInstruction,
    ) -> Result<Target, HtmlSemanticFailure> {
        self.template_raw(pi.target())?;
        for (word, target) in [
            ("marker", Target::Marker),
            ("start", Target::Start),
            ("end", Target::End),
        ] {
            self.charge(1)?;
            if pi.target().len() != word.len() {
                continue;
            }
            let mut equal = true;
            for (index, byte) in word.bytes().enumerate() {
                self.charge(1)?;
                if pi.target().code_units()[index] != u16::from(byte) {
                    equal = false;
                    break;
                }
            }
            if equal {
                return Ok(target);
            }
        }
        Ok(Target::Other)
    }
    fn template_pi_name_matches(
        &mut self,
        pi: &HtmlCapturedProcessingInstruction,
        expected: &DocumentString,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.template_raw(pi.target())?;
        self.template_raw(pi.data())?;
        let mut cursor = Cursor::new(pi.data().len());
        let mut names: Vec<Range<usize>> = Vec::new();
        let mut current = 0..0;
        let mut name_count = 0usize;
        let mut is_name = true;
        let mut value_count = 0usize;
        let mut value_equal = true;
        let mut matched = false;
        let result = (|| {
            while let Some(action) = cursor.action() {
                self.charge(std::mem::size_of::<Cursor>())?;
                let reply = match action {
                    Action::Read(offset) => {
                        self.charge(1)?;
                        let unit = pi.data().code_units().get(offset).copied();
                        Reply::Unit(unit)
                    }
                    Action::Step => Reply::Ack,
                    Action::Begin(Field::Name) => {
                        current = 0..0;
                        name_count = 0;
                        is_name = true;
                        Reply::Ack
                    }
                    Action::Begin(Field::Value) => {
                        value_count = 0;
                        value_equal = true;
                        Reply::Ack
                    }
                    Action::Emit {
                        field,
                        units,
                        len,
                        start,
                        end,
                        ..
                    } => {
                        if field == Field::Name {
                            if name_count == 0 {
                                current.start = start;
                            }
                            current.end = end;
                            for unit in &units[..len] {
                                self.charge(1)?;
                                if b"name".get(name_count).map(|b| u16::from(*b)) != Some(*unit) {
                                    is_name = false;
                                }
                                name_count = name_count.checked_add(1).ok_or_else(|| {
                                    self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
                                })?;
                            }
                        } else if is_name && name_count == 4 {
                            for unit in &units[..len] {
                                self.charge(1)?;
                                if expected.code_units().get(value_count) != Some(unit) {
                                    value_equal = false;
                                }
                                value_count = value_count.checked_add(1).ok_or_else(|| {
                                    self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
                                })?;
                            }
                        }
                        Reply::Ack
                    }
                    Action::NameLength(index) => {
                        self.charge(1)?;
                        let span = names.get(index).ok_or_else(|| {
                            self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        })?;
                        Reply::Length(span.len())
                    }
                    Action::NameUnit(which, offset) => {
                        self.charge(1)?;
                        let span = match which {
                            Name::Current => &current,
                            Name::Pair(index) => names.get(index).ok_or_else(|| {
                                self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                            })?,
                        };
                        if offset >= span.len() {
                            return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
                        }
                        let index = span.start.checked_add(offset).ok_or_else(|| {
                            self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
                        })?;
                        self.charge(1)?;
                        let unit = pi.data().code_units().get(index).copied();
                        #[cfg(test)]
                        if matches!(which, Name::Pair(_)) {
                            self.template_cancel(CancellationPoint::DuplicateRead);
                        }
                        Reply::Unit(unit)
                    }
                    Action::Commit => {
                        self.reserve_scratch(&mut names, 1)?;
                        self.charge(1)?;
                        names.push(current.clone());
                        #[cfg(test)]
                        self.template_cancel(CancellationPoint::PairCommitted);
                        if is_name && name_count == 4 {
                            matched = value_equal && value_count == expected.len();
                        }
                        Reply::Ack
                    }
                    Action::Discard => Reply::Ack,
                    Action::Pop => {
                        self.charge(1)?;
                        names.pop().ok_or_else(|| {
                            self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        })?;
                        Reply::Ack
                    }
                    Action::Ready(valid) => {
                        if !valid {
                            matched = false;
                        }
                        Reply::Ack
                    }
                };
                cursor.reply(reply).map_err(|error| {
                    self.stop(match error {
                        crate::bounded::HtmlPiPseudoAttributeError::Integrity => {
                            HtmlSemanticFailureKind::IntrinsicUnavailable
                        }
                        crate::bounded::HtmlPiPseudoAttributeError::Representation => {
                            HtmlSemanticFailureKind::RepresentationExhausted
                        }
                    })
                })?;
            }
            Ok(matched)
        })();
        self.release_scratch(names.capacity());
        result
    }
    fn template_end(
        &mut self,
        start: &HtmlCaptureSubjectId,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        let node = self.node(start)?;
        let parent = match self.parent(node)? {
            Ok(Some(parent)) => parent,
            Ok(None) => return Ok(Ok(None)),
            Err(missing) => return Ok(Err(missing)),
        };
        let node = self.node(&parent)?;
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(mut children) = node.children() else {
            return Ok(Err(HtmlSemanticMissing::OrderedChildren(parent)));
        };
        let mut found = false;
        let mut siblings = Siblings::default();
        loop {
            self.charge(1)?;
            let Some(child) = children.next() else {
                return if found {
                    Ok(Ok(None))
                } else {
                    Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
                };
            };
            if !found {
                if &child == start {
                    found = true;
                }
                continue;
            }
            let node = self.node(&child)?;
            let target = match node.kind() {
                HtmlCapturedNodeKind::ProcessingInstruction(pi) => self.template_target(pi)?,
                _ => Target::Other,
            };
            self.charge(std::mem::size_of::<Siblings>().max(1))?;
            let closes = siblings
                .inspect(target)
                .map_err(|_| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
            #[cfg(test)]
            self.template_cancel(CancellationPoint::SiblingStep);
            if closes {
                return Ok(Ok(Some(child)));
            }
        }
    }
    pub(super) fn template_for_reference(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlTemplateForReport>, HtmlSemanticFailure> {
        let raw = self.attribute(element, "for")?;
        if let Some(raw) = raw {
            self.template_raw(raw.value())?;
        }
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Template) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let Some(raw) = raw else {
            return Ok(HtmlSemanticValue::Known(HtmlTemplateForReport {
                raw: None,
                name_relation: HtmlTemplateForNameRelation::NotSpecified,
                selection: HtmlTemplateForSelection::NotSpecified,
            }));
        };
        let mut scope = subject.clone();
        let mut first_missing = None;
        loop {
            let node = self.node(&scope)?;
            match self.parent(node)? {
                Ok(Some(parent)) => scope = parent,
                Ok(None) => break,
                Err(missing) => {
                    first_missing = Some(missing);
                    break;
                }
            }
        }
        let mut selection = if raw.value().is_empty() {
            Some(HtmlTemplateForSelection::EmptyName)
        } else {
            first_missing.clone().map(HtmlTemplateForSelection::Missing)
        };
        let mut frames = Vec::new();
        let outcome = (|| {
            let root = self.node(&scope)?;
            let mut next = match self.ordinary_tree_next(root, &mut frames)? {
                Ok(next) => next,
                Err(missing) => {
                    first_missing.get_or_insert(missing.clone());
                    if selection.is_none() {
                        selection = Some(HtmlTemplateForSelection::Missing(missing));
                    }
                    {
                        #[cfg(test)]
                        self.template_cancel(CancellationPoint::UnknownSkipped);
                        self.ordinary_tree_resume(&mut frames)?
                    }
                }
            };
            while let Some(id) = next {
                let node = self.node(&id)?;
                if let HtmlCapturedNodeKind::ProcessingInstruction(pi) = node.kind() {
                    let matches = self.template_pi_name_matches(pi, raw.value())?;
                    let target = self.template_target(pi)?;
                    let first = template_marker_first(target, matches);
                    if first != First::Continue {
                        if selection.is_none() {
                            selection = Some(if first == First::Marker {
                                HtmlTemplateForSelection::Marker(id.clone())
                            } else {
                                match self.template_end(&id)? {
                                    Ok(end) => HtmlTemplateForSelection::Start {
                                        start: id.clone(),
                                        end,
                                    },
                                    Err(missing) => HtmlTemplateForSelection::Missing(missing),
                                }
                            });
                        }
                        return Ok(HtmlTemplateForReport {
                            raw: Some(raw.clone()),
                            name_relation: HtmlTemplateForNameRelation::Match(id),
                            selection: selection.take().ok_or_else(|| {
                                self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                            })?,
                        });
                    }
                }
                next = match self.ordinary_tree_next(node, &mut frames)? {
                    Ok(next) => next,
                    Err(missing) => {
                        first_missing.get_or_insert(missing.clone());
                        if selection.is_none() {
                            selection = Some(HtmlTemplateForSelection::Missing(missing));
                        }
                        {
                            #[cfg(test)]
                            self.template_cancel(CancellationPoint::UnknownSkipped);
                            self.ordinary_tree_resume(&mut frames)?
                        }
                    }
                };
            }
            Ok(HtmlTemplateForReport {
                raw: Some(raw.clone()),
                name_relation: first_missing.clone().map_or(
                    HtmlTemplateForNameRelation::NoMatch,
                    HtmlTemplateForNameRelation::Missing,
                ),
                selection: selection
                    .take()
                    .unwrap_or(HtmlTemplateForSelection::NoMatch),
            })
        })();
        self.release_scratch(frames.capacity());
        outcome.map(HtmlSemanticValue::Known)
    }
}
