use super::*;

pub(super) fn parent_can_have_children(kind: &DomNodeKind) -> bool {
    matches!(
        kind,
        DomNodeKind::Document(_)
            | DomNodeKind::DocumentFragment
            | DomNodeKind::ShadowRoot(_)
            | DomNodeKind::Element(_)
    )
}

impl Draft {
    pub(super) fn string_replace_all(
        &mut self,
        key: Key,
        raw: DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // DOM string-replace-all creates its Text before removing children.
        // Removed children remain real detached managed nodes, not retired IDs.
        let replacement = if raw.is_empty() {
            None
        } else {
            Some(self.allocate(DomNodeKind::Text(raw), self.node(key)?.document, limits)?)
        };
        while let Some(&child) = self.node(key)?.children.first() {
            self.effect_step(cancel)?;
            self.remove(child, limits, cancel)?;
        }
        if let Some(text) = replacement {
            self.pre_insert(text, key, None, limits, cancel)?;
        }
        Ok(())
    }

    pub(super) fn shadow_connected(
        &self,
        mut key: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        loop {
            self.effect_step(cancel)?;
            match self.shadow_including_parent(key)? {
                Some(parent) => key = parent,
                None => return Ok(matches!(self.node(key)?.kind, DomNodeKind::Document(_))),
            }
        }
    }

    fn native_post_connection_steps(
        &mut self,
        roots: &[Key],
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // DOM insert steps10–12 collect the whole static shadow-including list
        // before invoking any post-connection work. Template contents are owned
        // auxiliaries, not shadow-including descendants of their host.
        let mut list = self.effect_vec();
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, roots.len())?;
        pending.extend(roots.iter().rev().copied());
        while let Some(key) = pending.pop() {
            self.effect_step(cancel)?;
            self.select_reserve(&mut list, 1)?;
            list.push(key);
            let node = self.node(key)?;
            self.select_reserve(&mut pending, node.children.len())?;
            pending.extend(node.children.iter().rev().copied());
            if let Some(root) = node.shadow_root {
                self.select_reserve(&mut pending, 1)?;
                pending.push(root);
            }
        }
        self.push_effect(select::Effect::Post { list, index: 0 }, self.effect_scoped)?;
        self.run_effects(limits, cancel)
    }

    pub(super) fn adopt(
        &mut self,
        node: Key,
        document: Key,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        // Removal precedes the document test, including same-document adoption.
        // Admitted nonce/common/family state and manual intent survive adoption;
        // removal runs actual stored-assignment effects. Native null child-content
        // is preserved: adoption does not create a new element or reset its family.
        self.remove(node, limits, cancelled)?;
        if self.node(node)?.document == document {
            return Ok(());
        }
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1)?;
        pending.push((node, document));
        while let Some((node, document)) = pending.pop() {
            self.effect_step(cancelled)?;
            let children = &self.node(node)?.children;
            self.select_reserve(&mut pending, children.len())?;
            pending.extend(
                children
                    .iter()
                    .rev()
                    .copied()
                    .map(|child| (child, document)),
            );
            if let Some(root) = self.node(node)?.shadow_root {
                self.select_reserve(&mut pending, 1)?;
                pending.push((root, document));
            }
            self.node_mut(node).document = document;
            if let Some(contents) =
                self.template_adoption_target(node, document, limits, cancelled)?
            {
                self.select_reserve(&mut pending, 1)?;
                pending.push(contents);
            }
            // Attributes are value records owned by this Element, not Attr
            // nodes. Their node-document is consequently this same relation.
        }
        Ok(())
    }

    fn next_sibling(&self, node: Key) -> Option<Key> {
        let parent = self.node(node).expect("staged node").parent?;
        let children = &self.node(parent).expect("staged parent").children;
        let position = children
            .iter()
            .position(|child| *child == node)
            .expect("staged child");
        children.get(position + 1).copied()
    }

    fn ensure_pre_insert(
        &self,
        node: Key,
        parent: Key,
        child: Option<Key>,
        exclude: Option<Key>,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        if !parent_can_have_children(&self.node(parent)?.kind) {
            return Err(DomError::HierarchyRequest);
        }
        let mut ancestor = Some(parent);
        while let Some(key) = ancestor {
            self.effect_step(cancelled)?;
            if key == node {
                return Err(DomError::HierarchyRequest);
            }
            ancestor = self.host_including_parent(key)?;
        }
        if child
            .is_some_and(|child| self.node(child).expect("resolved child").parent != Some(parent))
        {
            return Err(DomError::NotFound);
        }
        let kind = &self.node(node)?.kind;
        if matches!(kind, DomNodeKind::Document(_)) {
            return Err(DomError::HierarchyRequest);
        }
        if !matches!(self.node(parent)?.kind, DomNodeKind::Document(_)) {
            return if matches!(kind, DomNodeKind::DocumentType(_)) {
                Err(DomError::HierarchyRequest)
            } else {
                Ok(())
            };
        }
        match kind {
            DomNodeKind::Text(_) => return Err(DomError::HierarchyRequest),
            DomNodeKind::Comment(_) | DomNodeKind::ProcessingInstruction(_) => return Ok(()),
            DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_) => {
                let mut elements = 0;
                for child in &self.node(node)?.children {
                    self.effect_step(cancelled)?;
                    match self.node(*child)?.kind {
                        DomNodeKind::Text(_) => return Err(DomError::HierarchyRequest),
                        DomNodeKind::Element(_) => elements += 1,
                        _ => {}
                    }
                    if elements > 1 {
                        return Err(DomError::HierarchyRequest);
                    }
                }
                if elements == 0 {
                    return Ok(());
                }
            }
            _ => {}
        }
        let children = &self.node(parent)?.children;
        let reference = child.map(|child| {
            children
                .iter()
                .position(|key| *key == child)
                .expect("validated reference")
        });
        for (position, candidate) in children.iter().copied().enumerate() {
            self.effect_step(cancelled)?;
            let candidate_kind = &self.node(candidate)?.kind;
            let invalid = if matches!(kind, DomNodeKind::DocumentType(_)) {
                (matches!(candidate_kind, DomNodeKind::DocumentType(_))
                    && Some(candidate) != exclude)
                    || (matches!(candidate_kind, DomNodeKind::Element(_))
                        && reference
                            .map_or(Some(candidate) != exclude, |reference| position < reference))
            } else {
                (matches!(candidate_kind, DomNodeKind::Element(_)) && Some(candidate) != exclude)
                    || (matches!(candidate_kind, DomNodeKind::DocumentType(_))
                        && reference.is_some_and(|reference| {
                            position > reference
                                || position == reference && Some(candidate) != exclude
                        }))
            };
            if invalid {
                return Err(DomError::HierarchyRequest);
            }
        }
        Ok(())
    }

    pub(super) fn pre_insert(
        &mut self,
        node: Key,
        parent: Key,
        mut before: Option<Key>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.ensure_pre_insert(node, parent, before, None, cancelled)?;
        if before == Some(node) {
            before = self.next_sibling(node);
        }
        self.insert_validated(node, parent, before, limits, cancelled)
    }

    pub(super) fn replace_child(
        &mut self,
        node: Key,
        parent: Key,
        child: Key,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.ensure_pre_insert(node, parent, Some(child), Some(child), cancelled)?;
        let mut before = self.next_sibling(child);
        if before == Some(node) {
            before = self.next_sibling(node);
        }
        // Unlike insert's per-child adoption, replace explicitly adopts its
        // supplied node first, including a DocumentFragment and its subtree.
        self.adopt(node, self.node(parent)?.document, limits, cancelled)?;
        self.remove(child, limits, cancelled)?;
        self.insert_validated(node, parent, before, limits, cancelled)
    }

    fn insert_validated(
        &mut self,
        node: Key,
        parent: Key,
        before: Option<Key>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let fragment = matches!(
            self.node(node)?.kind,
            DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_)
        );
        #[cfg(test)]
        let ledger_before_moving = self.select_temporary_bytes.get();
        let mut moving = self.effect_vec();
        if fragment {
            let children = &self.node(node)?.children;
            self.select_reserve(&mut moving, children.len())?;
            moving.extend(children.iter().copied());
        } else {
            self.select_reserve(&mut moving, 1)?;
            moving.push(node);
        }
        #[cfg(test)]
        let _moving_observation = scratch_observation::Moving::new(
            moving.capacity() * size_of::<Key>(),
            self.select_temporary_bytes.get() - ledger_before_moving,
        );
        if moving.is_empty() {
            self.children_changed(parent, limits, cancelled)?;
            return Ok(());
        }
        reserve(&mut self.node_mut(parent).children, moving.len())?;
        // Fragment removal, including source Slot effects, completes before
        // any insertion. Each actual inserted child then runs Slot and
        // applicable shadow-inclusive native insertion steps below.
        if fragment {
            for child in moving.iter() {
                self.effect_step(cancelled)?;
                self.remove(*child, limits, cancelled)?;
            }
        }
        let document = self.node(parent)?.document;
        for &child in moving.iter() {
            self.effect_step(cancelled)?;
            self.adopt(child, document, limits, cancelled)?;
            let target = self.node_mut(parent);
            let index = before.map_or(target.children.len(), |reference| {
                target
                    .children
                    .iter()
                    .position(|child| *child == reference)
                    .expect("retained reference")
            });
            target.children.insert(index, child);
            self.node_mut(child).parent = Some(parent);
            self.slot_insertion(child, parent, limits, cancelled)?;
            self.common_insertion_steps(child, limits, cancelled, &mut || check_cancel(cancelled))?;
        }
        // Parent children-changed observes the completed atomic insertion before
        // any descendant post-connection work, including nested Scripts.
        self.children_changed(parent, limits, cancelled)?;
        self.native_post_connection_steps(&moving, limits, cancelled)?;
        Ok(())
    }

    pub(super) fn native_removing_steps(
        &mut self,
        root: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1)?;
        pending.push(root);
        while let Some(key) = pending.pop() {
            self.effect_step(cancel)?;
            let node = self.node(key)?;
            self.select_reserve(&mut pending, node.children.len())?;
            pending.extend(node.children.iter().rev().copied());
            if let Some(root) = node.shadow_root {
                self.select_reserve(&mut pending, 1)?;
                pending.push(root);
            }
            self.navigable_removing_node(key, &mut || check_cancel(cancel))?;
            self.script_removing_node(key, cancel)?;
            self.select_removal(key, limits, cancel)?;
            self.control_removing_node(key, cancel)?;
        }
        Ok(())
    }

    fn common_insertion_steps(
        &mut self,
        root: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
        admit_step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        use surgeist_html::{
            HtmlBrowsingContextPresence, HtmlHeaderDeliveredCsp, HtmlNonceConnection,
            HtmlNonceInsertionEffect,
        };
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1)?;
        pending.push(root);
        while let Some(key) = pending.pop() {
            self.effect_step(cancel)?;
            admit_step()?;
            let node = self.node(key)?;
            self.select_reserve(&mut pending, node.children.len())?;
            pending.extend(node.children.iter().rev().copied());
            if let Some(root) = node.shadow_root {
                self.select_reserve(&mut pending, 1)?;
                pending.push(root);
            }
            let DomNodeKind::Element(element) = &node.kind else {
                continue;
            };
            let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
                unreachable!("canonical node document")
            };
            // This is an established native environment fact, independent of
            // class or connectedness. No CSP observation is invented.
            let connection = match document.native_environment().browsing_context() {
                HtmlBrowsingContextPresence::Absent => {
                    HtmlNonceConnection::NotBrowsingContextConnected
                }
                HtmlBrowsingContextPresence::Unavailable | HtmlBrowsingContextPresence::Present => {
                    HtmlNonceConnection::Unavailable
                }
            };
            let mut content = None;
            for attribute in element.attributes().iter() {
                self.effect_step(cancel)?;
                admit_step()?;
                if attribute.name().namespace().uri().is_none()
                    && attribute.name().local_name().code_units() == [0x6e, 0x6f, 0x6e, 0x63, 0x65]
                {
                    content = Some(attribute.value());
                    break;
                }
            }
            match surgeist_html::nonce_insertion_effect(
                element.name(),
                content,
                connection,
                HtmlHeaderDeliveredCsp::Unavailable,
            ) {
                HtmlNonceInsertionEffect::KeepContent => {}
                HtmlNonceInsertionEffect::Unavailable(missing) => {
                    return Err(DomError::NonceEnvironmentUnavailable(missing));
                }
                HtmlNonceInsertionEffect::HideContentPreservingSlot => {
                    unreachable!("native null browsing context cannot hide nonce content")
                }
            }
            self.control_insertion_node(key, cancel)?;
            self.select_insertion(key, limits, cancel)?;
            // Closed HTML popovers are hidden, so their removal/attribute hide
            // steps return without changing state. Never-navigable documents
            // are not fully active, discharging the autofocus insertion guard.
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_during_common_descendant_insertion_keeps_publication_unchanged() {
        let mut dom = Dom::new(DomLimits::new(16, 2048, 32, 0, 0, 8, 8192).unwrap()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let parent = batch.create_element_ns(
            &document,
            crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
            "div",
        );
        let child = batch.create_element_ns(
            &document,
            crate::Namespace::from_uri("http://www.w3.org/2000/svg"),
            "g",
        );
        batch.set_attribute(&child, "nonce", "child");
        batch.insert_before(&child, &parent, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let parent = commit.created(&parent).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &AtomicBool::new(false)).unwrap();
        let key = draft.resolve(parent.into()).unwrap();
        draft
            .set_nonce(key, "unpublished".into(), dom.limits(), &cancel)
            .unwrap();
        let mut visited = 0;
        assert_eq!(
            draft.common_insertion_steps(key, dom.limits(), &cancel, &mut || {
                visited += 1;
                if visited == 2 {
                    cancel.store(true, Ordering::Release);
                }
                check_cancel(&cancel)
            }),
            Err(DomError::Cancelled)
        );
        assert_eq!(visited, 2);
        drop(draft);
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
}

// A test-only independent observation of the actual moving allocation's full
// lifetime. Charge already present in the live ledger is subtracted once, so a
// tracked bank is never double-counted. This hook changes no admission decision.
#[cfg(test)]
pub(super) mod scratch_observation {
    use std::cell::Cell;
    thread_local! {
        static MISSING_MOVING: Cell<usize> = const { Cell::new(0) };
        static GAP: Cell<bool> = const { Cell::new(false) };
        static GROWTH_GAP: Cell<bool> = const { Cell::new(false) };
    }
    pub struct Moving(usize);
    impl Moving {
        pub fn new(capacity_bytes: usize, registered_bytes: usize) -> Self {
            let missing = capacity_bytes.saturating_sub(registered_bytes);
            MISSING_MOVING.set(MISSING_MOVING.get() + missing);
            Self(missing)
        }
    }
    impl Drop for Moving {
        fn drop(&mut self) {
            MISSING_MOVING.set(MISSING_MOVING.get() - self.0);
        }
    }
    pub fn observe(admitted: usize, limit: usize) {
        if admitted <= limit && admitted + MISSING_MOVING.get() > limit {
            GAP.set(true);
        }
    }
    pub fn observe_growth(before: usize, actual_new: usize, after: usize, limit: usize) {
        if after <= limit && before + actual_new > limit {
            GROWTH_GAP.set(true);
        }
    }
    pub fn reset() {
        assert_eq!(MISSING_MOVING.get(), 0);
        GAP.set(false);
        GROWTH_GAP.set(false);
    }
    pub fn gap() -> bool {
        GAP.get()
    }
    pub fn growth_gap() -> bool {
        GROWTH_GAP.get()
    }
}

#[cfg(test)]
mod scratch_lifetime_tests {
    use super::*;
    fn attempt(scratch: usize, deep: bool) -> (bool, bool, bool) {
        let cancel = AtomicBool::new(false);
        let configured = DomLimits::new(2048, 32768, 4096, 4096, 4096, 16, 1_000_000)
            .unwrap()
            .with_select_processing(128, 4_000_000, 10_000_000)
            .unwrap();
        let mut dom = Dom::new(configured).unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let select = b.create_element_ns(&d, crate::Namespace::html(), "select");
        let option = b.create_element_ns(&d, crate::Namespace::html(), "option");
        if deep {
            let mut chain = Vec::new();
            for _ in 0..32 {
                chain.push(b.create_element_ns(&d, crate::Namespace::html(), "span"));
            }
            b.insert_before(&chain[0], &option, None);
            for pair in chain.windows(2) {
                b.insert_before(&pair[1], &pair[0], None);
            }
            let text = b.create_text(&d, "x");
            b.insert_before(&text, chain.last().unwrap(), None);
        } else {
            for _ in 0..64 {
                let text = b.create_text(&d, "x");
                b.insert_before(&text, &option, None);
            }
        }
        let content = b.create_element_ns(&d, crate::Namespace::html(), "selectedcontent");
        b.insert_before(&option, &select, None);
        b.insert_before(&content, &select, None);
        b.insert_before(&select, &d, None);
        let c = dom.apply(b, &cancel).unwrap();
        let select = c.created(&select).unwrap().unwrap();
        let content = c.created(&content).unwrap().unwrap();
        dom.set_limits(
            configured
                .with_select_processing(128, scratch, 10_000_000)
                .unwrap(),
        )
        .unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        scratch_observation::reset();
        let mut b = DomBatch::new();
        b.set_select_selected_index(select, 0);
        let result = dom.apply(b, &cancel);
        match &result {
            Ok(_) => {
                let s = dom.snapshot();
                assert_eq!(
                    s.node(content).unwrap().children().len(),
                    if deep { 1 } else { 64 }
                );
                if deep {
                    let mut node = s.node(content).unwrap().children()[0];
                    for _ in 0..32 {
                        node = s.node(node).unwrap().children()[0];
                    }
                    assert_eq!(s.node(node).unwrap().kind(), &DomNodeKind::Text("x".into()));
                } else {
                    for id in s.node(content).unwrap().children() {
                        assert_eq!(s.node(*id).unwrap().kind(), &DomNodeKind::Text("x".into()));
                    }
                }
            }
            Err(error) => {
                assert_eq!(
                    *error,
                    DomError::ResourceLimit {
                        resource: DomResource::SelectScratch,
                        limit: scratch
                    }
                );
                assert_eq!(dom.snapshot().stamp(), before.stamp());
                assert_eq!(dom.usage(), usage);
                assert_eq!(
                    dom.snapshot().node(content).unwrap().children(),
                    before.node(content).unwrap().children()
                );
            }
        }
        (
            result.is_ok(),
            scratch_observation::gap(),
            scratch_observation::growth_gap(),
        )
    }
    fn boundary(deep: bool) {
        // Discover an actually accepted boundary rather than asserting an
        // incidental byte constant or requiring these allocations forever.
        let (mut low, mut high) = (0, 4_000_000);
        while low < high {
            let middle = low + (high - low) / 2;
            if attempt(middle, deep).0 {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        let (committed, moving_gap, growth_gap) = attempt(low, deep);
        eprintln!(
            "deep={deep}; minimum accepted scratch={low}; committed={committed}; moving gap={moving_gap}; actual frame growth gap={growth_gap}"
        );
        assert!(committed);
        assert!(
            !moving_gap,
            "committed while the actual live moving bank exceeded admission"
        );
        assert!(
            !growth_gap,
            "committed while old plus actual new frame capacity exceeded admission"
        );
        if low > 0 {
            assert!(!attempt(low - 1, deep).0);
        }
    }
    #[test]
    fn moving_capacity_remains_admitted_through_every_later_insert_reservation() {
        boundary(false);
    }
    #[test]
    fn nested_copy_admits_actual_effect_frame_growth_overlap() {
        boundary(true);
    }
}
