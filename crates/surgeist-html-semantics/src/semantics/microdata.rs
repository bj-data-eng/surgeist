//! Generic HTML §5.2 microdata over one qualified immutable ordinary tree.
//! External URL/vocabulary observations are supplied facts, never URL guesses.
use super::*;
use crate::{HtmlCaptureDocumentId, HtmlMicrodataTokenKind, HtmlMicrodataTokenReport};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
        HtmlDocumentMode, HtmlScriptingStatus, OrderedAttributes,
    };

    #[derive(Clone, Copy, Eq, PartialEq)]
    pub(super) enum CancellationStage {
        Token,
        Edge,
    }
    std::thread_local! {
        static CANCELLATION: std::cell::RefCell<Option<(CancellationStage, HtmlSemanticCancellation)>> = const { std::cell::RefCell::new(None) };
        static COMMITTED: std::cell::Cell<Option<HtmlSemanticUsage>> = const { std::cell::Cell::new(None) };
    }
    // Test scheduling only: after a real admitted write, cancel the original
    // handle. The next unchanged charge/poll supplies the terminal result.
    pub(super) fn cancel_after_commit(stage: CancellationStage, usage: HtmlSemanticUsage) {
        CANCELLATION.with_borrow_mut(|slot| {
            if slot
                .as_ref()
                .is_some_and(|(selected, _)| *selected == stage)
            {
                let (_, cancellation) = slot.take().unwrap();
                COMMITTED.set(Some(usage));
                cancellation.cancel();
            }
        });
    }

    #[test]
    fn original_cancellation_after_token_and_item_edge_commit_is_atomic_and_reclaims_scratch() {
        for stage in [CancellationStage::Token, CancellationStage::Edge] {
            let mut b = HtmlCaptureBuilder::new(
                HtmlCaptureLimits::new(10, 100, 10, 1000, 10, 0, 0),
                HtmlHostFactsRevision::fresh(),
            );
            let d = b
                .add_document(
                    HtmlDocumentMode::NoQuirks,
                    HtmlScriptingStatus::Unavailable,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            let mut make = |attributes| {
                b.add_element(
                    &d,
                    KnownHtmlElement::Div.qualified_name(),
                    OrderedAttributes::new(attributes).unwrap(),
                    HtmlCapturedIsValue::Unavailable,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap()
            };
            let outer = make(vec![Attribute::new(
                KnownHtmlAttribute::Itemscope.qualified_name(),
                "",
            )]);
            let inner = make(vec![
                Attribute::new(KnownHtmlAttribute::Itemscope.qualified_name(), ""),
                Attribute::new(KnownHtmlAttribute::Itemprop.qualified_name(), "p"),
            ]);
            b.establish_children(&d, std::slice::from_ref(&outer))
                .unwrap();
            b.establish_children(&outer, std::slice::from_ref(&inner))
                .unwrap();
            b.establish_children(&inner, &[]).unwrap();
            let c = b.finish().unwrap();
            let doc = c.subject_id(&d).unwrap();
            let inner = c.subject_id(&inner).unwrap();
            let cx = HtmlSemanticContext::new(
                &c,
                c.host_facts_revision().clone(),
                HtmlSemanticContextInput::new(vec![]),
                0,
            )
            .unwrap();
            let cancellation = HtmlSemanticCancellation::new();
            let mut s = HtmlSemanticSession::new(
                &c,
                &cx,
                HtmlValueInterpreter::new(1),
                HtmlSemanticLimits::new(1_000_000, 10, 100, 0, 100_000, 100),
                cancellation.clone(),
            )
            .unwrap();
            COMMITTED.set(None);
            CANCELLATION.with_borrow_mut(|slot| *slot = Some((stage, cancellation.clone())));
            let requested = match stage {
                CancellationStage::Token => {
                    HtmlSemanticRequest::new(inner, HtmlSemanticQuery::MicrodataProperty)
                }
                CancellationStage::Edge => HtmlSemanticRequest::new(
                    doc.clone(),
                    HtmlSemanticQuery::MicrodataDocumentAuthor,
                ),
            };
            // The first real result is ready before the second operation stops;
            // the batch still returns Err rather than publishing that prefix.
            let err = s
                .query(&[
                    HtmlSemanticRequest::new(doc, HtmlSemanticQuery::MicrodataProperty),
                    requested,
                ])
                .unwrap_err();
            let committed = COMMITTED.take().expect("actual selected write occurred");
            assert!(cancellation.is_cancelled());
            assert_eq!(err.kind(), &HtmlSemanticFailureKind::Cancelled);
            assert_eq!(err.usage(), committed);
            assert_eq!(s.usage().work(), committed.work());
            assert!(committed.allocated_bytes() > 0);
            match stage {
                CancellationStage::Token => assert!(committed.nested_output_records() > 0),
                CancellationStage::Edge => assert!(committed.scratch_records() > 0),
            }
            assert_eq!(s.usage().scratch_records(), 0);
            let stopped = s.usage();
            assert_eq!(s.query(&[]).unwrap_err(), err);
            assert_eq!(s.usage(), stopped);
            CANCELLATION.with_borrow(|slot| assert!(slot.is_none()));
        }
    }

    #[test]
    fn canonical_token_step_exact_work_and_one_short_preserve_first_stop() {
        let mut b = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(10, 100, 10, 1000, 10, 0, 0),
            HtmlHostFactsRevision::fresh(),
        );
        let d = b
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let n = b
            .add_element(
                &d,
                KnownHtmlElement::Span.qualified_name(),
                OrderedAttributes::new([Attribute::new(
                    KnownHtmlAttribute::Itemprop.qualified_name(),
                    "p",
                )])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&n).unwrap();
        let HtmlCapturedNodeKind::Element(e) = c.node(&id).unwrap().kind() else {
            unreachable!()
        };
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]),
            0,
        )
        .unwrap();
        // Exact declared operations: 1 attribute visit + 8 local-name units +
        // 1 raw admission + 1 cursor creation + (1 header+4 unit step) +
        // (1 header+2 EOF/token step) + 1 punctuation read + 1 capacity
        // admission + 1 token write + (1 header+2 Done step) + 1 raw share.
        const EXACT: usize = 1 + 8 + 1 + 1 + (1 + 4) + (1 + 2) + 1 + 1 + 1 + (1 + 2) + 1;
        for work in [EXACT, EXACT - 1] {
            let mut s = HtmlSemanticSession::new(
                &c,
                &cx,
                HtmlValueInterpreter::new(1),
                HtmlSemanticLimits::new(work, 0, 0, 0, 1000, 1),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
            let result = s.md_tokens(e, HtmlMicrodataTokenKind::ItemProp);
            if work == EXACT {
                assert_eq!(result.unwrap().token_units(0), Some([0x70].as_slice()));
                assert_eq!(s.usage().work(), EXACT);
            } else {
                let err = result.unwrap_err();
                assert_eq!(
                    err.kind(),
                    &HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::Work,
                        limit: EXACT - 1,
                        required: EXACT
                    }
                );
                assert_eq!(s.usage().work(), EXACT - 1);
                assert_eq!(s.query(&[]).unwrap_err(), err);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct HtmlMicrodataProviderId(Arc<()>);
impl HtmlMicrodataProviderId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlMicrodataProviderId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlMicrodataProviderId {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataProviderRevision {
    producer: HtmlMicrodataProviderId,
    configuration: u64,
    revision: u64,
}
impl HtmlMicrodataProviderRevision {
    pub fn new(producer: HtmlMicrodataProviderId, configuration: u64, revision: u64) -> Self {
        Self {
            producer,
            configuration,
            revision,
        }
    }
}

// A proved violation dominates unavailable unrelated alternatives in an AND;
// a proved satisfying alternative dominates unknown alternatives in an OR.
fn md_and<T>(
    left: HtmlMicrodataCondition<T>,
    right: HtmlMicrodataCondition<T>,
) -> HtmlMicrodataCondition<T> {
    use HtmlMicrodataCondition::*;
    match (left, right) {
        (Violated(v), _) | (_, Violated(v)) => Violated(v),
        (Missing(m), _) | (_, Missing(m)) => Missing(m),
        (Unchecked, _) | (_, Unchecked) => Unchecked,
        _ => Satisfied,
    }
}
fn md_or<T>(
    left: HtmlMicrodataCondition<T>,
    right: HtmlMicrodataCondition<T>,
) -> HtmlMicrodataCondition<T> {
    use HtmlMicrodataCondition::*;
    match (left, right) {
        (Satisfied, _) | (_, Satisfied) => Satisfied,
        (Missing(m), _) | (_, Missing(m)) => Missing(m),
        (Unchecked, _) | (_, Unchecked) => Unchecked,
        (Violated(v), Violated(_)) => Violated(v),
    }
}

impl<'a> HtmlSemanticSession<'a> {
    fn md_boolean<T>(
        &self,
        value: Option<bool>,
        witness: T,
        id: &HtmlCaptureSubjectId,
    ) -> HtmlMicrodataCondition<T> {
        match value {
            Some(true) => HtmlMicrodataCondition::Satisfied,
            Some(false) => HtmlMicrodataCondition::Violated(witness),
            None => HtmlMicrodataCondition::Missing(self.md_missing(id)),
        }
    }
    fn md_absolute(
        &mut self,
        id: &HtmlCaptureSubjectId,
        source: HtmlMicrodataRawSource,
        index: usize,
    ) -> Result<HtmlMicrodataCondition<usize>, HtmlSemanticFailure> {
        let value = match self.md_url(id, source, UrlUse::Absolute)? {
            Some(HtmlMicrodataUrlResult::AuthorSyntax(v)) => Some(*v),
            _ => None,
        };
        Ok(self.md_boolean(value, index, id))
    }
    fn md_receiver(
        &mut self,
        id: &HtmlCaptureSubjectId,
        names: &HtmlMicrodataTokenReport,
        receiver: &HtmlCaptureSubjectId,
        origin: Option<&HtmlCaptureSubjectId>,
        token: usize,
        path: &[HtmlCaptureSubjectId],
    ) -> Result<HtmlMicrodataReceiverCheck, HtmlSemanticFailure> {
        self.charge(1)?;
        let punctuation = names.tokens()[token].contains_dot_or_colon();
        let permission = self.md_property_fact(id, token, receiver, origin)?;
        let defined = if origin.is_none() || punctuation {
            HtmlMicrodataCondition::Violated(token)
        } else {
            self.md_boolean(permission.and_then(|p| p.allowed), token, id)
        };
        let absolute = self.md_absolute(id, HtmlMicrodataRawSource::ItemPropToken(token), token)?;
        let url_definition = self.md_boolean(permission.and_then(|p| p.defined_url), token, id);
        let proprietary = match permission.and_then(|p| p.proprietary) {
            Some(v) => self.md_boolean(Some(v), token, id),
            None => HtmlMicrodataCondition::Unchecked,
        };
        let url_name = md_and(absolute, md_or(url_definition, proprietary.clone()));
        let untyped = if origin.is_none() && !punctuation {
            proprietary
        } else {
            HtmlMicrodataCondition::Violated(token)
        };
        let name = md_or(defined, md_or(url_name, untyped));
        let value_kind = match permission.and_then(|p| p.takes_url) {
            Some(true) => {
                let e = self.md_element(id)?.expect("property");
                let role = self.known(e)?;
                self.md_boolean(Some(property_url_source(role).is_some()), token, id)
            }
            Some(false) => HtmlMicrodataCondition::Satisfied,
            None => HtmlMicrodataCondition::Missing(self.md_missing(id)),
        };
        let mut type_path = Vec::new();
        for item in path {
            self.reserve_nested_output(&mut type_path, 1)?;
            self.charge(1)?;
            type_path.push(item.clone());
        }
        self.charge(1)?;
        Ok(HtmlMicrodataReceiverCheck {
            receiver: receiver.clone(),
            origin: origin.cloned(),
            type_path,
            token,
            name,
            value_kind,
        })
    }
    fn md_cycles(
        &mut self,
        graph: &Graph,
    ) -> Result<HtmlMicrodataCondition<HtmlCaptureSubjectId>, HtmlSemanticFailure> {
        let mut visited = Vec::new();
        let mut path = Vec::new();
        let mut frames = Vec::new();
        let result = (|| {
            for root in &graph.items {
                self.charge(1)?;
                if self.md_contains(&visited, root)? {
                    continue;
                }
                self.md_push(&mut path, root.clone())?;
                self.md_push(&mut frames, (root.clone(), 0usize))?;
                while !frames.is_empty() {
                    self.charge(1)?;
                    let index = frames.len() - 1;
                    let (current, next) = frames[index].clone();
                    let mut cursor = next;
                    let mut child = None;
                    while cursor < graph.edges.len() {
                        self.charge(1)?;
                        let (from, to) = &graph.edges[cursor];
                        cursor += 1;
                        if from == &current {
                            child = Some(to.clone());
                            break;
                        }
                    }
                    self.charge(1)?;
                    frames[index].1 = cursor;
                    if let Some(child) = child {
                        if self.md_contains(&path, &child)? {
                            return Ok(HtmlMicrodataCondition::Violated(child));
                        }
                        if !self.md_contains(&visited, &child)? {
                            self.md_push(&mut path, child.clone())?;
                            self.md_push(&mut frames, (child, 0))?;
                        }
                    } else {
                        self.md_push(&mut visited, current)?;
                        self.charge(2)?;
                        frames.pop();
                        path.pop();
                    }
                }
            }
            Ok(graph.missing.clone().map_or(
                HtmlMicrodataCondition::Satisfied,
                HtmlMicrodataCondition::Missing,
            ))
        })();
        self.release_scratch(visited.capacity());
        self.release_scratch(path.capacity());
        self.release_scratch(frames.capacity());
        result
    }
    pub(super) fn microdata_document_author(
        &mut self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMicrodataDocumentAuthor>, HtmlSemanticFailure> {
        let node = self.node(id)?;
        if !matches!(node.kind(), HtmlCapturedNodeKind::Document(_)) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let graph = self.md_graph(id)?;
        let mut frames = Vec::new();
        let result = (|| {
            let cycles = self.md_cycles(&graph)?;
            let revisits = if let Some(id) = &graph.revisit {
                HtmlMicrodataCondition::Violated(id.clone())
            } else {
                graph.missing.clone().map_or(
                    HtmlMicrodataCondition::Satisfied,
                    HtmlMicrodataCondition::Missing,
                )
            };
            let mut orphans = HtmlMicrodataCondition::Satisfied;
            let mut current = Some(id.clone());
            while let Some(subject) = current {
                let node = self.node(&subject)?;
                if let Some(e) = self.md_element(&subject)?
                    && self.md_attribute(e, "itemprop")?.is_some()
                {
                    let names = self.md_tokens(e, HtmlMicrodataTokenKind::ItemProp)?;
                    let mut contributed = false;
                    for (_, property) in &graph.properties {
                        self.charge(1)?;
                        if property == &subject {
                            contributed = true;
                            break;
                        }
                    }
                    if !contributed {
                        orphans = md_and(
                            orphans,
                            if names.tokens().is_empty() {
                                HtmlMicrodataCondition::Violated(subject.clone())
                            } else if let Some(m) = &graph.missing {
                                HtmlMicrodataCondition::Missing(m.clone())
                            } else {
                                HtmlMicrodataCondition::Violated(subject.clone())
                            },
                        );
                    }
                }
                current = match self.ordinary_tree_next(node, &mut frames)? {
                    Ok(next) => next,
                    Err(m) => {
                        orphans = md_and(orphans, HtmlMicrodataCondition::Missing(m));
                        self.ordinary_tree_resume(&mut frames)?
                    }
                };
            }
            // Missing item enumeration can affect orphan absence even when no
            // currently observed property supplied a violation.
            if let Some(m) = &graph.missing {
                orphans = md_and(orphans, HtmlMicrodataCondition::Missing(m.clone()));
            }
            self.charge(1)?;
            Ok(HtmlSemanticValue::Known(HtmlMicrodataDocumentAuthor {
                revisits,
                cycles,
                orphans,
            }))
        })();
        self.release_scratch(frames.capacity());
        self.md_release_graph(&graph);
        result
    }
    pub(super) fn microdata_author(
        &mut self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMicrodataAuthor>, HtmlSemanticFailure> {
        let Some(e) = self.md_element(id)? else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        let scope_raw = self.md_attribute(e, "itemscope")?;
        // Boolean author spelling remains independent of true presence. Reuse
        // the actual intrinsic producer and its existing admitted envelope.
        self.admit_intrinsic(e, scope_raw, 1)?;
        let scope_report = self
            .interpreter
            .boolean_attribute(crate::HtmlBooleanAttribute::Itemscope, e.name(), scope_raw)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        self.poll()?;
        let itemscope_author = scope_report.author();
        self.charge(1)?;
        let itemscope = scope_raw.cloned();
        let itemid = self.md_attribute(e, "itemid")?.cloned();
        let types = self.md_tokens(e, HtmlMicrodataTokenKind::ItemType)?;
        let names = self.md_tokens(e, HtmlMicrodataTokenKind::ItemProp)?;
        let refs = self.md_tokens(e, HtmlMicrodataTokenKind::ItemRef)?;
        let eligibility = |present: bool, allowed: bool| {
            if !present || allowed {
                HtmlMicrodataCondition::Satisfied
            } else {
                HtmlMicrodataCondition::Violated(())
            }
        };
        let type_eligibility = eligibility(types.raw().is_some(), itemscope.is_some());
        let id_eligibility = eligibility(
            itemid.is_some(),
            itemscope.is_some() && !types.tokens().is_empty(),
        );
        let ref_eligibility = eligibility(refs.raw().is_some(), itemscope.is_some());
        let (mut type_urls, mut type_definitions, mut same_vocabulary) = (
            HtmlMicrodataCondition::Satisfied,
            HtmlMicrodataCondition::Satisfied,
            HtmlMicrodataCondition::Satisfied,
        );
        let mut vocabulary = None;
        let mut identifier_permission = HtmlMicrodataCondition::Satisfied;
        for i in 0..types.tokens().len() {
            self.charge(1)?;
            type_urls = md_and(
                type_urls,
                self.md_absolute(id, HtmlMicrodataRawSource::ItemTypeToken(i), i)?,
            );
            let fact = self.md_type_fact(id, i)?;
            type_definitions = md_and(
                type_definitions,
                self.md_boolean(fact.and_then(|f| f.defined), i, id),
            );
            match fact.and_then(|f| f.vocabulary.as_ref()) {
                Some(v) => {
                    if let Some(previous) = vocabulary {
                        same_vocabulary =
                            md_and(same_vocabulary, self.md_boolean(Some(previous == v), i, id));
                    } else {
                        vocabulary = Some(v);
                    }
                }
                None => {
                    same_vocabulary = md_and(
                        same_vocabulary,
                        HtmlMicrodataCondition::Missing(self.md_missing(id)),
                    )
                }
            }
            if itemid.is_some() {
                identifier_permission = md_and(
                    identifier_permission,
                    self.md_boolean(fact.and_then(|f| f.global_ids), i, id),
                );
            }
        }
        let identifier_syntax = if itemid.is_some() {
            match self.md_url(id, HtmlMicrodataRawSource::ItemId, UrlUse::IdAuthor)? {
                Some(HtmlMicrodataUrlResult::AuthorSyntax(v)) => self.md_boolean(Some(*v), (), id),
                _ => HtmlMicrodataCondition::Missing(self.md_missing(id)),
            }
        } else {
            HtmlMicrodataCondition::Satisfied
        };
        let mut references = HtmlMicrodataCondition::Satisfied;
        if !refs.tokens().is_empty() {
            match self.ordinary_tree_root(id)? {
                Ok(root) => {
                    for (i, t) in refs.tokens().iter().enumerate() {
                        self.charge(1)?;
                        let raw = refs.raw().expect("token raw");
                        references = md_and(
                            references,
                            match self.md_find_id(&root, &raw.code_units()[t.start()..t.end()])? {
                                HtmlSemanticValue::Known(Some(_)) => {
                                    HtmlMicrodataCondition::Satisfied
                                }
                                HtmlSemanticValue::Known(None) => {
                                    HtmlMicrodataCondition::Violated(i)
                                }
                                HtmlSemanticValue::Missing(m) => HtmlMicrodataCondition::Missing(m),
                                _ => unreachable!("ID domain"),
                            },
                        );
                    }
                }
                Err(m) => references = HtmlMicrodataCondition::Missing(m),
            }
        }
        let (mut contribution, mut receivers) = (
            HtmlMicrodataCondition::Satisfied,
            HtmlSemanticValue::Known(Vec::new()),
        );
        if names.raw().is_some() && names.tokens().is_empty() {
            contribution = HtmlMicrodataCondition::Violated(());
        } else if names.raw().is_some() {
            match self.ordinary_tree_root(id)? {
                Ok(root) => {
                    let graph = self.md_graph(&root)?;
                    let result = (|| {
                        let mut rows = Vec::new();
                        let mut received = false;
                        let mut missing = graph.missing.clone();
                        for (receiver, property) in &graph.properties {
                            self.charge(1)?;
                            if property != id {
                                continue;
                            }
                            received = true;
                            let re = self.md_element(receiver)?.expect("receiver item");
                            let own = self.md_tokens(re, HtmlMicrodataTokenKind::ItemType)?;
                            let inherited = own.tokens().is_empty();
                            let context = self.md_type_context(
                                receiver,
                                own,
                                if inherited { Some(&graph) } else { None },
                            )?;
                            match context.paths {
                                HtmlSemanticValue::Known(paths) => {
                                    if paths.is_empty() {
                                        if matches!(context.typing, HtmlMicrodataTyping::Cyclic) {
                                            missing.get_or_insert(
                                                HtmlSemanticMissing::MicrodataTypeCycle(
                                                    receiver.clone(),
                                                ),
                                            );
                                        } else {
                                            for (i, t) in names.tokens().iter().enumerate() {
                                                self.charge(1)?;
                                                if t.duplicate_of().is_some() {
                                                    continue;
                                                }
                                                let row = self.md_receiver(
                                                    id,
                                                    &names,
                                                    receiver,
                                                    None,
                                                    i,
                                                    &[],
                                                )?;
                                                self.reserve_nested_output(&mut rows, 1)?;
                                                self.charge(1)?;
                                                rows.push(row);
                                            }
                                        }
                                    } else {
                                        for path in paths {
                                            let origin = path.items.last().expect("type origin");
                                            for (i, t) in names.tokens().iter().enumerate() {
                                                self.charge(1)?;
                                                if t.duplicate_of().is_some() {
                                                    continue;
                                                }
                                                let row = self.md_receiver(
                                                    id,
                                                    &names,
                                                    receiver,
                                                    Some(origin),
                                                    i,
                                                    &path.items,
                                                )?;
                                                self.reserve_nested_output(&mut rows, 1)?;
                                                self.charge(1)?;
                                                rows.push(row);
                                            }
                                        }
                                    }
                                }
                                HtmlSemanticValue::Missing(m) => {
                                    missing.get_or_insert(m);
                                }
                                _ => unreachable!("context paths"),
                            }
                        }
                        let contribution = if received {
                            HtmlMicrodataCondition::Satisfied
                        } else if let Some(m) = &graph.missing {
                            HtmlMicrodataCondition::Missing(m.clone())
                        } else {
                            HtmlMicrodataCondition::Violated(())
                        };
                        let receivers = if let Some(m) = missing {
                            HtmlSemanticValue::Missing(m)
                        } else {
                            HtmlSemanticValue::Known(rows)
                        };
                        Ok((contribution, receivers))
                    })();
                    self.md_release_graph(&graph);
                    (contribution, receivers) = result?;
                }
                Err(m) => {
                    contribution = HtmlMicrodataCondition::Missing(m.clone());
                    receivers = HtmlSemanticValue::Missing(m);
                }
            }
        }
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlMicrodataAuthor {
            itemscope,
            itemscope_author,
            itemid,
            types,
            names,
            refs,
            type_eligibility,
            id_eligibility,
            ref_eligibility,
            type_urls,
            type_definitions,
            same_vocabulary,
            identifier_syntax,
            identifier_permission,
            references,
            contribution,
            receivers,
        }))
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataRevision {
    url: Option<HtmlMicrodataProviderRevision>,
    vocabulary: Option<HtmlMicrodataProviderRevision>,
}
impl HtmlMicrodataRevision {
    pub fn new(
        url: Option<HtmlMicrodataProviderRevision>,
        vocabulary: Option<HtmlMicrodataProviderRevision>,
    ) -> Self {
        Self { url, vocabulary }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataRawSource {
    ItemTypeToken(usize),
    ItemPropToken(usize),
    ItemId,
    Src,
    Href,
    Data,
}
impl HtmlMicrodataRawSource {
    fn local(self) -> &'static str {
        match self {
            Self::ItemTypeToken(_) => "itemtype",
            Self::ItemPropToken(_) => "itemprop",
            Self::ItemId => "itemid",
            Self::Src => "src",
            Self::Href => "href",
            Self::Data => "data",
        }
    }
}
/// Exact captured current slot. Historical operation identity is separate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataRawBinding {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    subject: HtmlCaptureSubjectId,
    document: HtmlCaptureDocumentId,
    source: HtmlMicrodataRawSource,
    raw: DocumentString,
    start: usize,
    end: usize,
}
impl HtmlMicrodataRawBinding {
    /// Checks the actual no-namespace slot and canonical token boundary. Ordinary
    /// bounded construction; this is not a semantic-session work budget.
    pub fn new(
        capture: &HtmlCapture,
        subject: HtmlCaptureSubjectId,
        source: HtmlMicrodataRawSource,
        max_raw_units: usize,
    ) -> Result<Self, HtmlMicrodataFactsError> {
        let node = capture
            .node(&subject)
            .map_err(HtmlMicrodataFactsError::Capture)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Err(HtmlMicrodataFactsError::WrongRole);
        };
        if element.name().namespace() != &crate::Namespace::html() {
            return Err(HtmlMicrodataFactsError::WrongRole);
        }
        let raw = element
            .attributes()
            .iter()
            .find(|a| {
                a.name().namespace().uri().is_none()
                    && a.name()
                        .local_name()
                        .code_units()
                        .iter()
                        .copied()
                        .eq(source.local().encode_utf16())
            })
            .ok_or(HtmlMicrodataFactsError::MissingRaw)?
            .value();
        let interpreter = HtmlValueInterpreter::new(max_raw_units);
        interpreter
            .admit_attribute_raw(Some(raw))
            .map_err(HtmlMicrodataFactsError::Attribute)?;
        let (start, end) = match source {
            HtmlMicrodataRawSource::ItemTypeToken(index)
            | HtmlMicrodataRawSource::ItemPropToken(index) => {
                let kind = if matches!(source, HtmlMicrodataRawSource::ItemTypeToken(_)) {
                    HtmlMicrodataTokenKind::ItemType
                } else {
                    HtmlMicrodataTokenKind::ItemProp
                };
                let report = interpreter
                    .microdata_tokens(kind, Some(raw))
                    .map_err(HtmlMicrodataFactsError::Attribute)?;
                let t = report
                    .tokens()
                    .get(index)
                    .ok_or(HtmlMicrodataFactsError::TokenIndex)?;
                (t.start(), t.end())
            }
            _ => (0, raw.len()),
        };
        if matches!(
            source,
            HtmlMicrodataRawSource::Src
                | HtmlMicrodataRawSource::Href
                | HtmlMicrodataRawSource::Data
        ) {
            let role = KnownHtmlElement::from_name(element.name());
            let expected = property_url_source(role);
            if expected != Some(source) {
                return Err(HtmlMicrodataFactsError::WrongRole);
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host: capture.host_facts_revision().clone(),
            document: node.node_document(),
            subject,
            source,
            raw: raw.clone(),
            start,
            end,
        })
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn source(&self) -> HtmlMicrodataRawSource {
        self.source
    }
    pub fn units(&self) -> &[u16] {
        &self.raw.code_units()[self.start..self.end]
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataUrlOperation {
    AbsoluteAuthorToken(HtmlMicrodataProviderRevision),
    ItemIdAuthor(HtmlMicrodataProviderRevision),
    CurrentItemIdentifier {
        revision: HtmlMicrodataProviderRevision,
        document: HtmlCaptureDocumentId,
    },
    /// Actual historical set-time document/base/encoding and event, supplied by
    /// the producer. It need not equal the current captured node-document.
    PropertyAttributeSet {
        revision: HtmlMicrodataProviderRevision,
        event: u64,
        document: HtmlCaptureDocumentId,
        base_revision: u64,
        encoding_revision: u64,
    },
}
impl HtmlMicrodataUrlOperation {
    fn revision(&self) -> &HtmlMicrodataProviderRevision {
        match self {
            Self::AbsoluteAuthorToken(r) | Self::ItemIdAuthor(r) => r,
            Self::CurrentItemIdentifier { revision, .. }
            | Self::PropertyAttributeSet { revision, .. } => revision,
        }
    }
    fn accepts(&self, source: HtmlMicrodataRawSource) -> bool {
        match self {
            Self::AbsoluteAuthorToken(_) => matches!(
                source,
                HtmlMicrodataRawSource::ItemTypeToken(_) | HtmlMicrodataRawSource::ItemPropToken(_)
            ),
            Self::ItemIdAuthor(_) | Self::CurrentItemIdentifier { .. } => {
                source == HtmlMicrodataRawSource::ItemId
            }
            Self::PropertyAttributeSet { .. } => matches!(
                source,
                HtmlMicrodataRawSource::Src
                    | HtmlMicrodataRawSource::Href
                    | HtmlMicrodataRawSource::Data
            ),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataUrlResult {
    AuthorSyntax(bool),
    Parsed(Option<DocumentString>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataUrlFact {
    binding: HtmlMicrodataRawBinding,
    expected: HtmlMicrodataUrlOperation,
    actual: Option<HtmlMicrodataUrlOperation>,
    result: Option<HtmlMicrodataUrlResult>,
}
impl HtmlMicrodataUrlFact {
    pub fn new(
        binding: HtmlMicrodataRawBinding,
        expected: HtmlMicrodataUrlOperation,
        actual: Option<HtmlMicrodataUrlOperation>,
        result: Option<HtmlMicrodataUrlResult>,
    ) -> Result<Self, HtmlMicrodataFactsError> {
        if !expected.accepts(binding.source)
            || actual.as_ref().is_some_and(|a| a != &expected)
            || actual.is_some() != result.is_some()
        {
            return Err(HtmlMicrodataFactsError::OperationMismatch);
        }
        if let HtmlMicrodataUrlOperation::CurrentItemIdentifier { document, .. } = &expected
            && document != &binding.document
        {
            return Err(HtmlMicrodataFactsError::OperationMismatch);
        }
        if let Some(result) = &result {
            let author = matches!(
                expected,
                HtmlMicrodataUrlOperation::AbsoluteAuthorToken(_)
                    | HtmlMicrodataUrlOperation::ItemIdAuthor(_)
            );
            if author != matches!(result, HtmlMicrodataUrlResult::AuthorSyntax(_)) {
                return Err(HtmlMicrodataFactsError::OperationMismatch);
            }
        }
        Ok(Self {
            binding,
            expected,
            actual,
            result,
        })
    }
}
/// Opaque provider-declared vocabulary identity; no URL deconstruction.
#[derive(Clone, Debug)]
pub struct HtmlMicrodataVocabularyId(Arc<()>);
impl HtmlMicrodataVocabularyId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlMicrodataVocabularyId {
    fn eq(&self, o: &Self) -> bool {
        Arc::ptr_eq(&self.0, &o.0)
    }
}
impl Eq for HtmlMicrodataVocabularyId {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTypeFact {
    binding: HtmlMicrodataRawBinding,
    revision: HtmlMicrodataProviderRevision,
    defined: Option<bool>,
    vocabulary: Option<HtmlMicrodataVocabularyId>,
    global_ids: Option<bool>,
}
impl HtmlMicrodataTypeFact {
    pub fn new(
        binding: HtmlMicrodataRawBinding,
        revision: HtmlMicrodataProviderRevision,
        defined: Option<bool>,
        vocabulary: Option<HtmlMicrodataVocabularyId>,
        global_ids: Option<bool>,
    ) -> Result<Self, HtmlMicrodataFactsError> {
        if !matches!(binding.source, HtmlMicrodataRawSource::ItemTypeToken(_)) {
            return Err(HtmlMicrodataFactsError::WrongRole);
        }
        Ok(Self {
            binding,
            revision,
            defined,
            vocabulary,
            global_ids,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataPropertyFact {
    binding: HtmlMicrodataRawBinding,
    revision: HtmlMicrodataProviderRevision,
    receiver: HtmlCaptureSubjectId,
    origin: Option<HtmlCaptureSubjectId>,
    origin_types: Option<DocumentString>,
    allowed: Option<bool>,
    defined_url: Option<bool>,
    takes_url: Option<bool>,
    proprietary: Option<bool>,
}
/// Independent provider observations, not a certificate of vocabulary truth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataPropertyPermission {
    defined_name: Option<bool>,
    defined_url: Option<bool>,
    takes_url: Option<bool>,
    proprietary: Option<bool>,
}
impl HtmlMicrodataPropertyPermission {
    pub fn new(
        defined_name: Option<bool>,
        defined_url: Option<bool>,
        takes_url: Option<bool>,
        proprietary: Option<bool>,
    ) -> Self {
        Self {
            defined_name,
            defined_url,
            takes_url,
            proprietary,
        }
    }
}
impl HtmlMicrodataPropertyFact {
    /// Actual receiving item and optional own-type origin. Graph membership is
    /// checked by the controlled consumer, not asserted by this constructor.
    pub fn new(
        capture: &HtmlCapture,
        binding: HtmlMicrodataRawBinding,
        revision: HtmlMicrodataProviderRevision,
        receiver: HtmlCaptureSubjectId,
        origin: Option<HtmlCaptureSubjectId>,
        permission: HtmlMicrodataPropertyPermission,
        max_raw_units: usize,
    ) -> Result<Self, HtmlMicrodataFactsError> {
        if binding.capture != *capture.identity()
            || !matches!(binding.source, HtmlMicrodataRawSource::ItemPropToken(_))
        {
            return Err(HtmlMicrodataFactsError::WrongRole);
        }
        capture
            .element_id(&receiver)
            .map_err(HtmlMicrodataFactsError::Capture)?;
        let origin_types = if let Some(origin) = &origin {
            let node = capture
                .node(origin)
                .map_err(HtmlMicrodataFactsError::Capture)?;
            let HtmlCapturedNodeKind::Element(e) = node.kind() else {
                return Err(HtmlMicrodataFactsError::WrongRole);
            };
            if e.name().namespace() != &crate::Namespace::html()
                || !e.attributes().iter().any(|a| {
                    a.name().namespace().uri().is_none()
                        && a.name()
                            .local_name()
                            .code_units()
                            .iter()
                            .copied()
                            .eq("itemscope".encode_utf16())
                })
            {
                return Err(HtmlMicrodataFactsError::WrongRole);
            }
            let raw = e
                .attributes()
                .iter()
                .find(|a| {
                    a.name().namespace().uri().is_none()
                        && a.name()
                            .local_name()
                            .code_units()
                            .iter()
                            .copied()
                            .eq("itemtype".encode_utf16())
                })
                .ok_or(HtmlMicrodataFactsError::MissingRaw)?
                .value();
            let report = HtmlValueInterpreter::new(max_raw_units)
                .microdata_tokens(HtmlMicrodataTokenKind::ItemType, Some(raw))
                .map_err(HtmlMicrodataFactsError::Attribute)?;
            if report.tokens().is_empty() {
                return Err(HtmlMicrodataFactsError::WrongRole);
            }
            Some(raw.clone())
        } else {
            None
        };
        if permission.defined_name == Some(true)
            && binding.units().iter().any(|u| matches!(*u, 0x2e | 0x3a))
        {
            return Err(HtmlMicrodataFactsError::WrongRole);
        }
        Ok(Self {
            binding,
            revision,
            receiver,
            origin,
            origin_types,
            allowed: permission.defined_name,
            defined_url: permission.defined_url,
            takes_url: permission.takes_url,
            proprietary: permission.proprietary,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataFactsError {
    Capture(HtmlCaptureError),
    Attribute(HtmlValueAdmissionError),
    WrongRole,
    MissingRaw,
    TokenIndex,
    OperationMismatch,
    Duplicate,
    Limit,
}
impl fmt::Display for HtmlMicrodataFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlMicrodataFactsError {}
/// Moved, finite ordinary producer observations. No tree projection or resolver.
#[derive(Debug)]
pub struct HtmlMicrodataFacts {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    revision: HtmlMicrodataRevision,
    urls: Vec<HtmlMicrodataUrlFact>,
    types: Vec<HtmlMicrodataTypeFact>,
    properties: Vec<HtmlMicrodataPropertyFact>,
}
impl HtmlMicrodataFacts {
    pub(super) fn qualifies(
        &self,
        capture: &HtmlCapture,
        expected: Option<&HtmlMicrodataRevision>,
    ) -> bool {
        self.capture == *capture.identity()
            && self.host == *capture.host_facts_revision()
            && expected == Some(&self.revision)
    }
    pub fn new(
        capture: &HtmlCapture,
        revision: HtmlMicrodataRevision,
        urls: Vec<HtmlMicrodataUrlFact>,
        types: Vec<HtmlMicrodataTypeFact>,
        properties: Vec<HtmlMicrodataPropertyFact>,
        max_records: usize,
        max_units: usize,
    ) -> Result<Self, HtmlMicrodataFactsError> {
        if urls
            .capacity()
            .checked_add(types.capacity())
            .and_then(|n| n.checked_add(properties.capacity()))
            .is_none_or(|n| n > max_records)
        {
            return Err(HtmlMicrodataFactsError::Limit);
        }
        let qualify = |b: &HtmlMicrodataRawBinding| -> Result<(), HtmlMicrodataFactsError> {
            if b.capture != *capture.identity() || b.host != *capture.host_facts_revision() {
                return Err(HtmlMicrodataFactsError::OperationMismatch);
            }
            if b.raw.len() > max_units {
                return Err(HtmlMicrodataFactsError::Limit);
            }
            Ok(())
        };
        for (i, f) in urls.iter().enumerate() {
            qualify(&f.binding)?;
            if revision.url.as_ref() != Some(f.expected.revision()) {
                return Err(HtmlMicrodataFactsError::OperationMismatch);
            }
            if let Some(HtmlMicrodataUrlResult::Parsed(Some(s))) = &f.result
                && s.len() > max_units
            {
                return Err(HtmlMicrodataFactsError::Limit);
            }
            if urls[..i].iter().any(|o| {
                o.binding == f.binding
                    && std::mem::discriminant(&o.expected) == std::mem::discriminant(&f.expected)
            }) {
                return Err(HtmlMicrodataFactsError::Duplicate);
            }
        }
        for (i, f) in types.iter().enumerate() {
            qualify(&f.binding)?;
            if revision.vocabulary.as_ref() != Some(&f.revision) {
                return Err(HtmlMicrodataFactsError::OperationMismatch);
            }
            if types[..i].iter().any(|o| o.binding == f.binding) {
                return Err(HtmlMicrodataFactsError::Duplicate);
            }
        }
        for (i, f) in properties.iter().enumerate() {
            qualify(&f.binding)?;
            if revision.vocabulary.as_ref() != Some(&f.revision) {
                return Err(HtmlMicrodataFactsError::OperationMismatch);
            }
            if f.origin_types.as_ref().is_some_and(|s| s.len() > max_units) {
                return Err(HtmlMicrodataFactsError::Limit);
            }
            if properties[..i]
                .iter()
                .any(|o| o.binding == f.binding && o.receiver == f.receiver && o.origin == f.origin)
            {
                return Err(HtmlMicrodataFactsError::Duplicate);
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host: capture.host_facts_revision().clone(),
            revision,
            urls,
            types,
            properties,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataCondition<T> {
    Satisfied,
    Violated(T),
    Missing(HtmlSemanticMissing),
    Unchecked,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataValue {
    String(DocumentString),
    Item(HtmlCaptureSubjectId),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataProperty {
    subject: HtmlCaptureSubjectId,
    names: HtmlMicrodataTokenReport,
    value: HtmlSemanticValue<HtmlMicrodataValue>,
}
impl HtmlMicrodataProperty {
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn names(&self) -> &HtmlMicrodataTokenReport {
        &self.names
    }
    pub fn value(&self) -> &HtmlSemanticValue<HtmlMicrodataValue> {
        &self.value
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTypePath {
    items: Vec<HtmlCaptureSubjectId>,
    types: HtmlMicrodataTokenReport,
}
impl HtmlMicrodataTypePath {
    pub fn items(&self) -> &[HtmlCaptureSubjectId] {
        &self.items
    }
    pub fn types(&self) -> &HtmlMicrodataTokenReport {
        &self.types
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTyping {
    Typed,
    Untyped,
    Cyclic,
    Missing(HtmlSemanticMissing),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTypeContext {
    typing: HtmlMicrodataTyping,
    paths: HtmlSemanticValue<Vec<HtmlMicrodataTypePath>>,
    cycle: Option<HtmlCaptureSubjectId>,
}
impl HtmlMicrodataTypeContext {
    pub fn typing(&self) -> &HtmlMicrodataTyping {
        &self.typing
    }
    pub fn paths(&self) -> &HtmlSemanticValue<Vec<HtmlMicrodataTypePath>> {
        &self.paths
    }
    pub fn cycle(&self) -> Option<&HtmlCaptureSubjectId> {
        self.cycle.as_ref()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataItem {
    subject: HtmlCaptureSubjectId,
    own_types: HtmlMicrodataTokenReport,
    top_level: bool,
    types: HtmlMicrodataTypeContext,
    identifier: HtmlSemanticValue<Option<DocumentString>>,
    properties: HtmlSemanticValue<Vec<HtmlMicrodataProperty>>,
    revisits: HtmlMicrodataCondition<HtmlCaptureSubjectId>,
}
impl HtmlMicrodataItem {
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn own_types(&self) -> &HtmlMicrodataTokenReport {
        &self.own_types
    }
    pub fn top_level(&self) -> bool {
        self.top_level
    }
    pub fn type_context(&self) -> &HtmlMicrodataTypeContext {
        &self.types
    }
    pub fn identifier(&self) -> &HtmlSemanticValue<Option<DocumentString>> {
        &self.identifier
    }
    pub fn properties(&self) -> &HtmlSemanticValue<Vec<HtmlMicrodataProperty>> {
        &self.properties
    }
    pub fn revisits(&self) -> &HtmlMicrodataCondition<HtmlCaptureSubjectId> {
        &self.revisits
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataReceiverCheck {
    receiver: HtmlCaptureSubjectId,
    origin: Option<HtmlCaptureSubjectId>,
    type_path: Vec<HtmlCaptureSubjectId>,
    token: usize,
    name: HtmlMicrodataCondition<usize>,
    value_kind: HtmlMicrodataCondition<usize>,
}
impl HtmlMicrodataReceiverCheck {
    pub fn receiver(&self) -> &HtmlCaptureSubjectId {
        &self.receiver
    }
    pub fn origin(&self) -> Option<&HtmlCaptureSubjectId> {
        self.origin.as_ref()
    }
    pub fn type_path(&self) -> &[HtmlCaptureSubjectId] {
        &self.type_path
    }
    pub fn token(&self) -> usize {
        self.token
    }
    pub fn name(&self) -> &HtmlMicrodataCondition<usize> {
        &self.name
    }
    pub fn value_kind(&self) -> &HtmlMicrodataCondition<usize> {
        &self.value_kind
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataAuthor {
    itemscope: Option<Attribute>,
    itemscope_author: crate::HtmlAttributeAuthorState,
    itemid: Option<Attribute>,
    types: HtmlMicrodataTokenReport,
    names: HtmlMicrodataTokenReport,
    refs: HtmlMicrodataTokenReport,
    type_eligibility: HtmlMicrodataCondition<()>,
    id_eligibility: HtmlMicrodataCondition<()>,
    ref_eligibility: HtmlMicrodataCondition<()>,
    type_urls: HtmlMicrodataCondition<usize>,
    type_definitions: HtmlMicrodataCondition<usize>,
    same_vocabulary: HtmlMicrodataCondition<usize>,
    identifier_syntax: HtmlMicrodataCondition<()>,
    identifier_permission: HtmlMicrodataCondition<usize>,
    references: HtmlMicrodataCondition<usize>,
    contribution: HtmlMicrodataCondition<()>,
    receivers: HtmlSemanticValue<Vec<HtmlMicrodataReceiverCheck>>,
}
impl HtmlMicrodataAuthor {
    pub fn itemscope(&self) -> Option<&Attribute> {
        self.itemscope.as_ref()
    }
    pub fn itemscope_author(&self) -> crate::HtmlAttributeAuthorState {
        self.itemscope_author
    }
    pub fn itemid(&self) -> Option<&Attribute> {
        self.itemid.as_ref()
    }
    pub fn types(&self) -> &HtmlMicrodataTokenReport {
        &self.types
    }
    pub fn names(&self) -> &HtmlMicrodataTokenReport {
        &self.names
    }
    pub fn references_local(&self) -> &HtmlMicrodataTokenReport {
        &self.refs
    }
    pub fn type_eligibility(&self) -> &HtmlMicrodataCondition<()> {
        &self.type_eligibility
    }
    pub fn id_eligibility(&self) -> &HtmlMicrodataCondition<()> {
        &self.id_eligibility
    }
    pub fn ref_eligibility(&self) -> &HtmlMicrodataCondition<()> {
        &self.ref_eligibility
    }
    pub fn type_urls(&self) -> &HtmlMicrodataCondition<usize> {
        &self.type_urls
    }
    pub fn type_definitions(&self) -> &HtmlMicrodataCondition<usize> {
        &self.type_definitions
    }
    pub fn same_vocabulary(&self) -> &HtmlMicrodataCondition<usize> {
        &self.same_vocabulary
    }
    pub fn identifier_syntax(&self) -> &HtmlMicrodataCondition<()> {
        &self.identifier_syntax
    }
    pub fn identifier_permission(&self) -> &HtmlMicrodataCondition<usize> {
        &self.identifier_permission
    }
    pub fn references(&self) -> &HtmlMicrodataCondition<usize> {
        &self.references
    }
    pub fn contribution(&self) -> &HtmlMicrodataCondition<()> {
        &self.contribution
    }
    pub fn receivers(&self) -> &HtmlSemanticValue<Vec<HtmlMicrodataReceiverCheck>> {
        &self.receivers
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataDocumentAuthor {
    revisits: HtmlMicrodataCondition<HtmlCaptureSubjectId>,
    cycles: HtmlMicrodataCondition<HtmlCaptureSubjectId>,
    orphans: HtmlMicrodataCondition<HtmlCaptureSubjectId>,
}
impl HtmlMicrodataDocumentAuthor {
    pub fn revisits(&self) -> &HtmlMicrodataCondition<HtmlCaptureSubjectId> {
        &self.revisits
    }
    pub fn cycles(&self) -> &HtmlMicrodataCondition<HtmlCaptureSubjectId> {
        &self.cycles
    }
    pub fn orphans(&self) -> &HtmlMicrodataCondition<HtmlCaptureSubjectId> {
        &self.orphans
    }
}

fn property_url_source(role: Option<KnownHtmlElement>) -> Option<HtmlMicrodataRawSource> {
    use KnownHtmlElement::*;
    match role {
        Some(Audio | Embed | Iframe | Img | Source | Track | Video) => {
            Some(HtmlMicrodataRawSource::Src)
        }
        Some(A | Area | Link) => Some(HtmlMicrodataRawSource::Href),
        Some(Object) => Some(HtmlMicrodataRawSource::Data),
        _ => None,
    }
}
struct Crawl {
    // Every admitted element is a proved membership witness even when another
    // branch prevents a complete ordered public collection.
    elements: Vec<HtmlCaptureSubjectId>,
    missing: Option<HtmlSemanticMissing>,
    revisit: HtmlMicrodataCondition<HtmlCaptureSubjectId>,
}
struct Graph {
    items: Vec<HtmlCaptureSubjectId>,
    edges: Vec<(HtmlCaptureSubjectId, HtmlCaptureSubjectId)>,
    properties: Vec<(HtmlCaptureSubjectId, HtmlCaptureSubjectId)>,
    missing: Option<HtmlSemanticMissing>,
    revisit: Option<HtmlCaptureSubjectId>,
}
#[derive(Clone, Copy)]
enum UrlUse {
    Absolute,
    IdAuthor,
    Identifier,
    Property,
}
impl UrlUse {
    fn matches(self, o: &HtmlMicrodataUrlOperation) -> bool {
        matches!(
            (self, o),
            (
                Self::Absolute,
                HtmlMicrodataUrlOperation::AbsoluteAuthorToken(_)
            ) | (Self::IdAuthor, HtmlMicrodataUrlOperation::ItemIdAuthor(_))
                | (
                    Self::Identifier,
                    HtmlMicrodataUrlOperation::CurrentItemIdentifier { .. }
                )
                | (
                    Self::Property,
                    HtmlMicrodataUrlOperation::PropertyAttributeSet { .. }
                )
        )
    }
}
impl<'a> HtmlSemanticSession<'a> {
    fn md_missing(&self, subject: &HtmlCaptureSubjectId) -> HtmlSemanticMissing {
        HtmlSemanticMissing::MicrodataFacts(subject.clone())
    }
    fn md_element(
        &mut self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<Option<&'a HtmlCapturedElement>, HtmlSemanticFailure> {
        let n = self.node(id)?;
        let HtmlCapturedNodeKind::Element(e) = n.kind() else {
            return Ok(None);
        };
        if self.html_namespace(e)? {
            Ok(Some(e))
        } else {
            Ok(None)
        }
    }
    fn md_attribute(
        &mut self,
        e: &'a HtmlCapturedElement,
        local: &str,
    ) -> Result<Option<&'a Attribute>, HtmlSemanticFailure> {
        let raw = self.attribute(e, local)?;
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        Ok(raw)
    }
    fn md_scope(&mut self, id: &HtmlCaptureSubjectId) -> Result<bool, HtmlSemanticFailure> {
        let Some(e) = self.md_element(id)? else {
            return Ok(false);
        };
        Ok(self.md_attribute(e, "itemscope")?.is_some())
    }
    fn md_tokens(
        &mut self,
        e: &'a HtmlCapturedElement,
        kind: HtmlMicrodataTokenKind,
    ) -> Result<HtmlMicrodataTokenReport, HtmlSemanticFailure> {
        let raw = self
            .md_attribute(
                e,
                match kind {
                    HtmlMicrodataTokenKind::ItemType => "itemtype",
                    HtmlMicrodataTokenKind::ItemProp => "itemprop",
                    HtmlMicrodataTokenKind::ItemRef => "itemref",
                },
            )?
            .map(Attribute::value);
        use crate::bounded::{HtmlMicrodataTokenActionError, HtmlMicrodataTokenInterpretation};
        let mut owner = Some(HtmlMicrodataTokenInterpretation::new(kind, raw));
        while let Some(action) = owner
            .as_mut()
            .expect("retained owner")
            .next_action()
            .map_err(|_| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?
        {
            let step = action.advance(self).map_err(|error| match error {
                HtmlMicrodataTokenActionError::Admission(error) => error,
                HtmlMicrodataTokenActionError::Protocol(_) => {
                    self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
                }
            })?;
            #[cfg(test)]
            if step == crate::bounded::HtmlMicrodataTokenStep::TokenCommitted {
                tests::cancel_after_commit(tests::CancellationStage::Token, self.usage());
            }
            #[cfg(not(test))]
            let _ = step;
        }
        HtmlMicrodataTokenInterpretation::try_finish(&mut owner)
            .map_err(|_| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))
    }
    fn md_contains(
        &mut self,
        ids: &[HtmlCaptureSubjectId],
        id: &HtmlCaptureSubjectId,
    ) -> Result<bool, HtmlSemanticFailure> {
        for other in ids {
            self.charge(1)?;
            if other == id {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn md_push<T>(&mut self, values: &mut Vec<T>, value: T) -> Result<(), HtmlSemanticFailure> {
        self.reserve_scratch(values, 1)?;
        self.charge(1)?;
        values.push(value);
        Ok(())
    }
    fn md_children(
        &mut self,
        id: &HtmlCaptureSubjectId,
        pending: &mut Vec<HtmlCaptureSubjectId>,
    ) -> Result<Option<HtmlSemanticMissing>, HtmlSemanticFailure> {
        let node = self.node(id)?;
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(mut children) = node.children() else {
            return Ok(Some(HtmlSemanticMissing::OrderedChildren(id.clone())));
        };
        loop {
            self.charge(1)?;
            let Some(child) = children.next() else {
                break;
            };
            if matches!(self.node(&child)?.kind(), HtmlCapturedNodeKind::Element(_)) {
                self.md_push(pending, child)?;
            }
        }
        Ok(None)
    }
    fn md_find_id(
        &mut self,
        root: &HtmlCaptureSubjectId,
        units: &[u16],
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        let mut frames = Vec::new();
        let result = (|| {
            let mut current = Some(root.clone());
            let mut missing = None;
            while let Some(id) = current {
                let node = self.node(&id)?;
                if let HtmlCapturedNodeKind::Element(e) = node.kind()
                    && let Some(raw) = self.md_attribute(e, "id")?
                    && !raw.value().is_empty()
                    && self.control_equal_units(raw.value().code_units(), units)?
                {
                    return Ok(if let Some(m) = missing {
                        HtmlSemanticValue::Missing(m)
                    } else {
                        HtmlSemanticValue::Known(Some(id))
                    });
                }
                current = match self.ordinary_tree_next(node, &mut frames)? {
                    Ok(next) => next,
                    Err(m) => {
                        missing.get_or_insert(m);
                        self.ordinary_tree_resume(&mut frames)?
                    }
                };
            }
            Ok(if let Some(m) = missing {
                HtmlSemanticValue::Missing(m)
            } else {
                HtmlSemanticValue::Known(None)
            })
        })();
        self.release_scratch(frames.capacity());
        result
    }
    fn md_path(
        &mut self,
        id: &HtmlCaptureSubjectId,
        bound: &HtmlCaptureSubjectId,
        out: &mut Vec<HtmlCaptureSubjectId>,
    ) -> Result<Option<HtmlSemanticMissing>, HtmlSemanticFailure> {
        let mut at = id.clone();
        loop {
            self.md_push(out, at.clone())?;
            if &at == bound {
                return Ok(None);
            }
            let node = self.node(&at)?;
            match self.parent(node)? {
                Ok(Some(p)) => at = p,
                Ok(None) => return Ok(Some(self.md_missing(id))),
                Err(m) => return Ok(Some(m)),
            }
        }
    }
    fn md_before(
        &mut self,
        left: &HtmlCaptureSubjectId,
        right: &HtmlCaptureSubjectId,
        bound: &HtmlCaptureSubjectId,
    ) -> Result<Result<bool, HtmlSemanticMissing>, HtmlSemanticFailure> {
        let mut a = Vec::new();
        let mut b = Vec::new();
        let result = (|| {
            if let Some(m) = self.md_path(left, bound, &mut a)? {
                return Ok(Err(m));
            }
            if let Some(m) = self.md_path(right, bound, &mut b)? {
                return Ok(Err(m));
            }
            let (mut ai, mut bi) = (a.len(), b.len());
            while ai > 0 && bi > 0 {
                self.charge(1)?;
                if a[ai - 1] != b[bi - 1] {
                    break;
                }
                ai -= 1;
                bi -= 1;
            }
            if ai == 0 {
                return Ok(Ok(true));
            }
            if bi == 0 {
                return Ok(Ok(false));
            }
            let common = &a[ai];
            let node = self.node(common)?;
            self.charge(1)?;
            let HtmlCapturedChildren::Complete(mut children) = node.children() else {
                return Ok(Err(HtmlSemanticMissing::OrderedChildren(common.clone())));
            };
            loop {
                self.charge(1)?;
                let Some(child) = children.next() else {
                    return Ok(Err(self.md_missing(common)));
                };
                if child == a[ai - 1] {
                    return Ok(Ok(true));
                }
                if child == b[bi - 1] {
                    return Ok(Ok(false));
                }
            }
        })();
        self.release_scratch(a.capacity());
        self.release_scratch(b.capacity());
        result
    }
    fn md_crawl(&mut self, root: &HtmlCaptureSubjectId) -> Result<Crawl, HtmlSemanticFailure> {
        let mut pending = Vec::new();
        let mut memory = Vec::new();
        let mut results = Vec::new();
        let result = (|| {
            self.md_push(&mut memory, root.clone())?;
            let mut missing = self.md_children(root, &mut pending)?;
            let mut bound = root.clone();
            let mut revisit = None;
            let e = self.md_element(root)?.expect("qualified item");
            let refs = self.md_tokens(e, HtmlMicrodataTokenKind::ItemRef)?;
            if !refs.tokens().is_empty() {
                match self.ordinary_tree_root(root)? {
                    Ok(tree) => {
                        bound = tree;
                        for t in refs.tokens() {
                            let units =
                                &refs.raw().expect("token raw").code_units()[t.start()..t.end()];
                            match self.md_find_id(&bound, units)? {
                                HtmlSemanticValue::Known(Some(id)) => {
                                    self.md_push(&mut pending, id)?
                                }
                                HtmlSemanticValue::Missing(m) => {
                                    missing.get_or_insert(m);
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(m) => {
                        missing.get_or_insert(m);
                    }
                }
            }
            while !pending.is_empty() {
                self.charge(1)?;
                let current = pending.pop().expect("pending");
                if self.md_contains(&memory, &current)? {
                    revisit.get_or_insert(current);
                    continue;
                }
                self.md_push(&mut memory, current.clone())?;
                let e = self.md_element(&current)?;
                let scope = if let Some(e) = e {
                    self.md_attribute(e, "itemscope")?.is_some()
                } else {
                    false
                };
                if !scope && let Some(m) = self.md_children(&current, &mut pending)? {
                    missing.get_or_insert(m);
                }
                if let Some(e) = e {
                    let names = self.md_tokens(e, HtmlMicrodataTokenKind::ItemProp)?;
                    if !names.tokens().is_empty() {
                        self.reserve_nested_output(&mut results, 1)?;
                        self.charge(1)?;
                        results.push(current);
                    }
                }
            }
            if missing.is_none() {
                for i in 1..results.len() {
                    let mut j = i;
                    while j > 0 {
                        self.charge(1)?;
                        match self.md_before(&results[j], &results[j - 1], &bound)? {
                            Ok(true) => {
                                self.charge(2)?;
                                results.swap(j, j - 1);
                                j -= 1;
                            }
                            Ok(false) => break,
                            Err(m) => {
                                missing = Some(m);
                                break;
                            }
                        }
                    }
                    if missing.is_some() {
                        break;
                    }
                }
            }
            let error = if let Some(id) = revisit {
                HtmlMicrodataCondition::Violated(id)
            } else if let Some(m) = &missing {
                HtmlMicrodataCondition::Missing(m.clone())
            } else {
                HtmlMicrodataCondition::Satisfied
            };
            Ok(Crawl {
                elements: results,
                missing,
                revisit: error,
            })
        })();
        self.release_scratch(pending.capacity());
        self.release_scratch(memory.capacity());
        result
    }
    fn md_url(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        source: HtmlMicrodataRawSource,
        use_kind: UrlUse,
    ) -> Result<Option<&'a HtmlMicrodataUrlResult>, HtmlSemanticFailure> {
        if let Some(facts) = &self.context.microdata {
            for fact in &facts.urls {
                self.charge(1)?;
                if &fact.binding.subject == subject
                    && fact.binding.source == source
                    && use_kind.matches(&fact.expected)
                {
                    self.charge(1)?;
                    return Ok(fact.result.as_ref());
                }
            }
        }
        Ok(None)
    }
    fn md_type_fact(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        index: usize,
    ) -> Result<Option<&'a HtmlMicrodataTypeFact>, HtmlSemanticFailure> {
        if let Some(facts) = &self.context.microdata {
            for f in &facts.types {
                self.charge(1)?;
                if &f.binding.subject == subject
                    && f.binding.source == HtmlMicrodataRawSource::ItemTypeToken(index)
                {
                    return Ok(Some(f));
                }
            }
        }
        Ok(None)
    }
    fn md_property_fact(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        index: usize,
        item: &HtmlCaptureSubjectId,
        origin: Option<&HtmlCaptureSubjectId>,
    ) -> Result<Option<&'a HtmlMicrodataPropertyFact>, HtmlSemanticFailure> {
        if let Some(facts) = &self.context.microdata {
            for f in &facts.properties {
                self.charge(1)?;
                if &f.binding.subject == subject
                    && f.binding.source == HtmlMicrodataRawSource::ItemPropToken(index)
                    && &f.receiver == item
                    && f.origin.as_ref() == origin
                {
                    return Ok(Some(f));
                }
            }
        }
        Ok(None)
    }
    fn md_property_value(
        &mut self,
        id: &HtmlCaptureSubjectId,
        e: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlMicrodataValue>, HtmlSemanticFailure> {
        if self.md_attribute(e, "itemscope")?.is_some() {
            self.charge(1)?;
            return Ok(HtmlSemanticValue::Known(HtmlMicrodataValue::Item(
                id.clone(),
            )));
        }
        let role = self.known(e)?;
        if let Some(source) = property_url_source(role) {
            let raw = self.md_attribute(e, source.local())?;
            if raw.is_none() {
                self.charge(1)?;
                return Ok(HtmlSemanticValue::Known(HtmlMicrodataValue::String(
                    self.context.empty_language.clone(),
                )));
            }
            let Some(HtmlMicrodataUrlResult::Parsed(parsed)) =
                self.md_url(id, source, UrlUse::Property)?
            else {
                return Ok(HtmlSemanticValue::Missing(self.md_missing(id)));
            };
            self.charge(1)?;
            return Ok(HtmlSemanticValue::Known(HtmlMicrodataValue::String(
                parsed
                    .as_ref()
                    .unwrap_or(&self.context.empty_language)
                    .clone(),
            )));
        }
        let attr = match role {
            Some(KnownHtmlElement::Meta) => Some("content"),
            Some(KnownHtmlElement::Data | KnownHtmlElement::Meter) => Some("value"),
            Some(KnownHtmlElement::Time) => Some("datetime"),
            _ => None,
        };
        if let Some(local) = attr {
            let raw = self.md_attribute(e, local)?;
            if raw.is_some() || role != Some(KnownHtmlElement::Time) {
                self.charge(1)?;
                return Ok(HtmlSemanticValue::Known(HtmlMicrodataValue::String(
                    raw.map_or(&self.context.empty_language, Attribute::value)
                        .clone(),
                )));
            }
        }
        Ok(self
            .control_text_content(
                id,
                if role == Some(KnownHtmlElement::Time) {
                    HtmlControlTextOrigin::ChildText
                } else {
                    HtmlControlTextOrigin::DescendantText
                },
            )?
            .map(HtmlMicrodataValue::String))
    }
    pub(super) fn microdata_property(
        &mut self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMicrodataProperty>, HtmlSemanticFailure> {
        let Some(e) = self.md_element(id)? else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        let names = self.md_tokens(e, HtmlMicrodataTokenKind::ItemProp)?;
        if names.raw().is_none() {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let value = self.md_property_value(id, e)?;
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlMicrodataProperty {
            subject: id.clone(),
            names,
            value,
        }))
    }
    fn md_graph(&mut self, root: &HtmlCaptureSubjectId) -> Result<Graph, HtmlSemanticFailure> {
        let mut graph = Graph {
            items: Vec::new(),
            edges: Vec::new(),
            properties: Vec::new(),
            missing: None,
            revisit: None,
        };
        let mut frames = Vec::new();
        let result = (|| {
            let mut current = Some(root.clone());
            while let Some(id) = current {
                let node = self.node(&id)?;
                if self.md_scope(&id)? {
                    self.md_push(&mut graph.items, id.clone())?;
                }
                current = match self.ordinary_tree_next(node, &mut frames)? {
                    Ok(next) => next,
                    Err(m) => {
                        graph.missing.get_or_insert(m);
                        self.ordinary_tree_resume(&mut frames)?
                    }
                };
            }
            for i in 0..graph.items.len() {
                self.charge(1)?;
                let item = graph.items[i].clone();
                let crawl = self.md_crawl(&item)?;
                match crawl.revisit {
                    HtmlMicrodataCondition::Violated(id) => {
                        graph.revisit.get_or_insert(id);
                    }
                    HtmlMicrodataCondition::Missing(m) => {
                        graph.missing.get_or_insert(m);
                    }
                    _ => {}
                }
                if let Some(m) = crawl.missing {
                    graph.missing.get_or_insert(m);
                }
                for id in crawl.elements {
                    self.md_push(&mut graph.properties, (item.clone(), id.clone()))?;
                    if self.md_scope(&id)? {
                        self.md_push(&mut graph.edges, (item.clone(), id))?;
                        #[cfg(test)]
                        tests::cancel_after_commit(tests::CancellationStage::Edge, self.usage());
                    }
                }
            }
            Ok(())
        })();
        self.release_scratch(frames.capacity());
        if let Err(error) = result {
            self.md_release_graph(&graph);
            return Err(error);
        }
        Ok(graph)
    }
    fn md_release_graph(&mut self, graph: &Graph) {
        self.release_scratch(graph.items.capacity());
        self.release_scratch(graph.edges.capacity());
        self.release_scratch(graph.properties.capacity());
    }
    fn md_type_context(
        &mut self,
        id: &HtmlCaptureSubjectId,
        own: HtmlMicrodataTokenReport,
        graph: Option<&Graph>,
    ) -> Result<HtmlMicrodataTypeContext, HtmlSemanticFailure> {
        let mut paths = Vec::new();
        let mut path = Vec::new();
        let mut frames = Vec::new();
        let mut cycle = None;
        let result = (|| {
            self.md_push(&mut path, id.clone())?;
            self.md_push(&mut frames, (id.clone(), 0usize, false))?;
            while !frames.is_empty() {
                self.charge(1)?;
                let index = frames.len() - 1;
                let (current, next, entered) = frames[index].clone();
                if !entered {
                    let types = if &current == id {
                        None
                    } else {
                        let e = self.md_element(&current)?.expect("graph item");
                        Some(self.md_tokens(e, HtmlMicrodataTokenKind::ItemType)?)
                    };
                    let types = types.as_ref().unwrap_or(&own);
                    if !types.tokens().is_empty() {
                        let mut items = Vec::new();
                        for item in &path {
                            self.reserve_nested_output(&mut items, 1)?;
                            self.charge(1)?;
                            items.push(item.clone());
                        }
                        // Reconstruct token evidence through its controlled owner,
                        // rather than cloning the variable token vector.
                        let e = self.md_element(&current)?.expect("typed item");
                        let types = self.md_tokens(e, HtmlMicrodataTokenKind::ItemType)?;
                        self.reserve_nested_output(&mut paths, 1)?;
                        self.charge(1)?;
                        paths.push(HtmlMicrodataTypePath { items, types });
                        self.charge(2)?;
                        frames.pop();
                        path.pop();
                        continue;
                    }
                    self.charge(1)?;
                    frames[index].2 = true;
                }
                let mut parent = None;
                let mut cursor = next;
                if let Some(graph) = graph {
                    while cursor < graph.edges.len() {
                        self.charge(1)?;
                        let (from, to) = &graph.edges[cursor];
                        cursor += 1;
                        if to == &current {
                            parent = Some(from.clone());
                            break;
                        }
                    }
                }
                self.charge(1)?;
                frames[index].1 = cursor;
                if let Some(parent) = parent {
                    if self.md_contains(&path, &parent)? {
                        cycle.get_or_insert(parent);
                        continue;
                    }
                    self.md_push(&mut path, parent.clone())?;
                    self.md_push(&mut frames, (parent, 0, false))?;
                } else {
                    self.charge(2)?;
                    frames.pop();
                    path.pop();
                }
            }
            let missing = graph.and_then(|g| g.missing.clone());
            let typing = if !paths.is_empty() {
                HtmlMicrodataTyping::Typed
            } else if let Some(m) = &missing {
                HtmlMicrodataTyping::Missing(m.clone())
            } else if cycle.is_some() {
                HtmlMicrodataTyping::Cyclic
            } else {
                HtmlMicrodataTyping::Untyped
            };
            let paths = if let Some(m) = missing {
                HtmlSemanticValue::Missing(m)
            } else {
                HtmlSemanticValue::Known(paths)
            };
            Ok(HtmlMicrodataTypeContext {
                typing,
                paths,
                cycle,
            })
        })();
        self.release_scratch(path.capacity());
        self.release_scratch(frames.capacity());
        result
    }
    pub(super) fn microdata_item(
        &mut self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMicrodataItem>, HtmlSemanticFailure> {
        let Some(e) = self.md_element(id)? else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if self.md_attribute(e, "itemscope")?.is_none() {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let own_types = self.md_tokens(e, HtmlMicrodataTokenKind::ItemType)?;
        let top_level = self.md_attribute(e, "itemprop")?.is_none();
        let types = if !own_types.tokens().is_empty() {
            let fresh = self.md_tokens(e, HtmlMicrodataTokenKind::ItemType)?;
            self.md_type_context(id, fresh, None)?
        } else {
            match self.ordinary_tree_root(id)? {
                Ok(root) => {
                    let graph = self.md_graph(&root)?;
                    let fresh = self.md_tokens(e, HtmlMicrodataTokenKind::ItemType);
                    let types = match fresh {
                        Ok(fresh) => self.md_type_context(id, fresh, Some(&graph)),
                        Err(error) => Err(error),
                    };
                    self.md_release_graph(&graph);
                    types?
                }
                Err(m) => HtmlMicrodataTypeContext {
                    typing: HtmlMicrodataTyping::Missing(m.clone()),
                    paths: HtmlSemanticValue::Missing(m),
                    cycle: None,
                },
            }
        };
        let identifier = if self.md_attribute(e, "itemid")?.is_none() {
            HtmlSemanticValue::Known(None)
        } else {
            match self.md_url(id, HtmlMicrodataRawSource::ItemId, UrlUse::Identifier)? {
                Some(HtmlMicrodataUrlResult::Parsed(value)) => {
                    self.charge(1)?;
                    HtmlSemanticValue::Known(value.clone())
                }
                _ => HtmlSemanticValue::Missing(self.md_missing(id)),
            }
        };
        let crawl = self.md_crawl(id)?;
        let properties = match crawl.missing {
            None => {
                let mut rows = Vec::new();
                for id in crawl.elements {
                    let e = self.md_element(&id)?.expect("qualified property");
                    let names = self.md_tokens(e, HtmlMicrodataTokenKind::ItemProp)?;
                    let value = self.md_property_value(&id, e)?;
                    self.reserve_nested_output(&mut rows, 1)?;
                    self.charge(1)?;
                    rows.push(HtmlMicrodataProperty {
                        subject: id,
                        names,
                        value,
                    });
                }
                HtmlSemanticValue::Known(rows)
            }
            Some(m) => HtmlSemanticValue::Missing(m),
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlMicrodataItem {
            subject: id.clone(),
            own_types,
            top_level,
            types,
            identifier,
            properties,
            revisits: crawl.revisit,
        }))
    }
}

impl crate::bounded::HtmlMicrodataTokenMeter for HtmlSemanticSession<'_> {
    type Error = HtmlSemanticFailure;
    fn work(&mut self, units: usize) -> Result<(), Self::Error> {
        self.charge(units)
    }
    fn reserve(
        &mut self,
        reservation: &mut crate::bounded::HtmlMicrodataTokenReservation<'_>,
    ) -> Result<(), Self::Error> {
        self.reserve_record_capacity(
            reservation.len(),
            reservation.capacity(),
            reservation.record_size(),
            1,
            HtmlSemanticResource::NestedOutputRecords,
            || {
                reservation.try_reserve().map_err(|_| ())?;
                Ok(reservation.capacity())
            },
        )?;
        Ok(())
    }
}
