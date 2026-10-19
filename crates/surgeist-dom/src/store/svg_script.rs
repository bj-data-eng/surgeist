//! Native Tiny operation over the existing static post-connection owner.
use super::*;

#[cfg(test)]
pub(super) struct ProcessingTrace {
    writes: Vec<(Key, Key)>,
    cancel_after: usize,
}

impl Draft {
    pub(super) fn svg_script_post_connection(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let Some(state) = (match &self.node(key)?.kind {
            DomNodeKind::Element(element) => element.svg_script_state(),
            _ => None,
        }) else {
            return Ok(());
        };
        if state.flags.already_processed() || !self.shadow_connected(key, cancel)? {
            return Ok(());
        }
        // Connection follows shadow topology; qualification follows the actual
        // node document, which need not be the connected host document.
        let document = self.node(key)?.document;
        let environment = self
            .document_metadata(document)?
            .document_environment()
            .clone();
        check_cancel(cancel)?;
        self.with_element_fields(key, cancel, |element| {
            element
                .svg_script_state_mut()
                .expect("checked SVG Script")
                .mark_processed_in(environment)
        })??;
        #[cfg(test)]
        if let Some(trace) = &mut self.svg_processing_trace {
            // Private deterministic observer of the real write, absent from
            // production and never a retained event/notification channel.
            trace.writes.push((key, document));
            if trace.writes.len() == trace.cancel_after {
                cancel.store(true, Ordering::Release);
            }
        }
        check_cancel(cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn processed(draft: &Draft, key: Key) -> bool {
        let DomNodeKind::Element(element) = &draft.node(key).unwrap().kind else {
            unreachable!()
        };
        element
            .svg_script_state()
            .unwrap()
            .flags
            .already_processed()
    }

    #[test]
    fn static_shadow_including_order_reaches_the_real_three_bit_prefix_before_cancel() {
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                8192,
                128,
                0,
                0,
                8,
                65536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        let d = b.create_document();
        let parent = b.create_element_ns(&d, crate::Namespace::html(), "div");
        b.insert_before(&parent, &d, None);
        let fragment = b.create_fragment(&d);
        let host = b.create_element_ns(&d, crate::Namespace::html(), "span");
        let root = b.attach_shadow(
            &host,
            crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Open),
        );
        let first = b.create_element_ns(&d, crate::Namespace::svg(), "script");
        let nested = b.create_element_ns(&d, crate::Namespace::html(), "span");
        let inner = b.attach_shadow(
            &nested,
            crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Closed),
        );
        let second = b.create_element_ns(&d, crate::Namespace::svg(), "script");
        let third = b.create_element_ns(&d, crate::Namespace::svg(), "script");
        let fourth = b.create_element_ns(&d, crate::Namespace::svg(), "script");
        let fifth = b.create_element_ns(&d, crate::Namespace::svg(), "script");
        b.insert_before(&first, &root, None);
        b.insert_before(&nested, &root, None);
        b.insert_before(&second, &inner, None);
        b.insert_before(&third, &nested, None);
        b.insert_before(&fourth, &host, None);
        b.insert_before(&host, &fragment, None);
        b.insert_before(&fifth, &fragment, None);
        let c = dom.apply(b, &cancel).unwrap();
        let parent = c.created(&parent).unwrap().unwrap();
        let fragment = c.created(&fragment).unwrap().unwrap();
        let ids =
            [&first, &second, &third, &fourth, &fifth].map(|n| c.created(n).unwrap().unwrap());
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let keys = ids.map(|id| draft.resolve(id.into()).unwrap());
        let document = draft.resolve(c.document(&d).unwrap().into()).unwrap();
        draft.svg_processing_trace = Some(ProcessingTrace {
            writes: Vec::with_capacity(5),
            cancel_after: 3,
        });
        assert_eq!(
            draft.apply(
                &Command::Insert {
                    node: fragment.into(),
                    parent: parent.into(),
                    before: None
                },
                dom.limits,
                &cancel
            ),
            Err(DomError::Cancelled)
        );
        // The real static traversal visits each hosted root before its host's
        // ordinary children, and fragment roots in source order.
        assert_eq!(
            draft.svg_processing_trace.as_ref().unwrap().writes,
            vec![
                (keys[0], document),
                (keys[1], document),
                (keys[2], document)
            ]
        );
        assert_eq!(
            keys.map(|key| processed(&draft, key)),
            [true, true, true, false, false]
        );
        assert!(
            draft
                .node(draft.resolve(fragment.into()).unwrap())
                .unwrap()
                .children
                .is_empty()
        );
        drop(draft);
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }

    #[test]
    fn qualification_lookup_uses_the_actual_root_child_document_not_the_connected_host() {
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                4096,
                64,
                0,
                0,
                8,
                65536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        let d1 = b.create_document();
        let host = b.create_element_ns(&d1, crate::Namespace::html(), "div");
        b.insert_before(&host, &d1, None);
        let root = b.attach_shadow(
            &host,
            crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Open),
        );
        let d2 = b.create_document();
        let fragment = b.create_fragment(&d2);
        let old = b.create_comment(&d2, "old");
        b.insert_before(&old, &fragment, None);
        let c = dom.apply(b, &cancel).unwrap();
        let d1 = c.document(&d1).unwrap();
        let d2 = c.document(&d2).unwrap();
        let root = c.created(&root).unwrap().unwrap();
        let fragment = c.created(&fragment).unwrap().unwrap();
        let old = c.created(&old).unwrap().unwrap();
        let mut b = DomBatch::new();
        b.replace_child(root, fragment, old);
        let script = b.create_element_ns(d2, crate::Namespace::svg(), "script");
        let c = dom.apply(b, &cancel).unwrap();
        let script = c.created(&script).unwrap().unwrap();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        draft.svg_processing_trace = Some(ProcessingTrace {
            writes: Vec::with_capacity(1),
            cancel_after: usize::MAX,
        });
        let key = draft.resolve(script.into()).unwrap();
        let actual = draft.resolve(d2.into()).unwrap();
        let host_document = draft.resolve(d1.into()).unwrap();
        draft
            .apply(
                &Command::Insert {
                    node: script.into(),
                    parent: root.into(),
                    before: None,
                },
                dom.limits,
                &cancel,
            )
            .unwrap();
        assert!(draft.shadow_connected(key, &cancel).unwrap());
        assert_ne!(actual, host_document);
        assert_eq!(
            draft.svg_processing_trace.as_ref().unwrap().writes,
            vec![(key, actual)]
        );
        assert!(processed(&draft, key));
    }

    #[test]
    fn prepared_processed_bit_then_detach_cannot_publish_after_cancel_or_revision_exhaustion() {
        for exhausted in [false, true] {
            let mut dom = Dom::new(
                DomLimits::new(
                    16,
                    2048,
                    32,
                    0,
                    0,
                    8,
                    65536,
                    crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let cancel = AtomicBool::new(false);
            let mut b = DomBatch::new();
            let d = b.create_document();
            let root = b.create_element_ns(&d, crate::Namespace::html(), "div");
            b.insert_before(&root, &d, None);
            let script = b.create_element_ns(&d, crate::Namespace::svg(), "script");
            let c = dom.apply(b, &cancel).unwrap();
            let root = c.created(&root).unwrap().unwrap();
            let script = c.created(&script).unwrap().unwrap();
            if exhausted {
                dom.revision = crate::DomRevision(u64::MAX);
            }
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut b = DomBatch::new();
            b.insert_before(script, root, None);
            b.detach(script);
            let prepared = dom.prepare(b, &cancel).unwrap();
            let actual = prepared.state.node(dom.owner, script).unwrap();
            let DomNodeKind::Element(element) = &actual.kind else {
                unreachable!()
            };
            assert!(
                element
                    .svg_script_state()
                    .unwrap()
                    .flags
                    .already_processed()
            );
            assert_eq!(actual.parent, None);
            if !exhausted {
                cancel.store(true, Ordering::Release);
            }
            assert_eq!(
                dom.publish(prepared, &cancel).unwrap_err(),
                if exhausted {
                    DomError::RevisionExhausted
                } else {
                    DomError::Cancelled
                }
            );
            assert_eq!(*dom.state, *before.state);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
            assert!(
                dom.changes_since(before.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        }
    }
}
