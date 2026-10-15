//! Supported native DOM clone/import over a checked immutable source capture.
use super::*;

struct Frame {
    source: NodeId,
    copy: Key,
    document: Key,
    parent: Option<Key>,
    template_child: usize,
    appended: bool,
    next_child: usize,
    subtree: bool,
    shadow_started: bool,
}

impl Draft {
    pub(super) fn import_node(
        &mut self,
        snapshot: &DomSnapshot,
        source: NodeId,
        document: Key,
        subtree: bool,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        let root = self.clone_single(snapshot, source, document, limits, cancelled)?;
        let mut frames = Vec::new();
        reserve(&mut frames, 1)?;
        frames.push(Frame {
            source,
            copy: root,
            document,
            parent: None,
            template_child: 0,
            appended: false,
            next_child: 0,
            subtree,
            shadow_started: false,
        });
        while let Some(frame) = frames.last_mut() {
            check_cancel(cancelled)?;
            let original = snapshot.node(frame.source)?;
            // HTML's cloning extension precedes parent append and ordinary
            // descendants. Each contents frame has its actual inert document.
            if frame.subtree
                && let Some(template) = original.as_template()
            {
                let contents = snapshot.node(template.contents())?;
                if let Some(&source) = contents.children().get(frame.template_child) {
                    frame.template_child += 1;
                    let parent = self
                        .node(frame.copy)?
                        .template_contents
                        .expect("fresh contents");
                    let document = self.node(parent)?.document;
                    let copy = self.clone_single(snapshot, source, document, limits, cancelled)?;
                    reserve(&mut frames, 1)?;
                    frames.push(Frame {
                        source,
                        copy,
                        document,
                        parent: Some(parent),
                        template_child: 0,
                        appended: false,
                        next_child: 0,
                        subtree: true,
                        shadow_started: false,
                    });
                    continue;
                }
            }
            if !frame.appended {
                frame.appended = true;
                if let Some(parent) = frame.parent {
                    self.pre_insert(frame.copy, parent, None, limits, cancelled)?;
                }
            }
            if frame.subtree
                && let Some(&source) = original.children().get(frame.next_child)
            {
                frame.next_child += 1;
                let parent = frame.copy;
                let document = frame.document;
                let copy = self.clone_single(snapshot, source, document, limits, cancelled)?;
                reserve(&mut frames, 1)?;
                frames.push(Frame {
                    source,
                    copy,
                    document,
                    parent: Some(parent),
                    template_child: 0,
                    appended: false,
                    next_child: 0,
                    subtree: true,
                    shadow_started: false,
                });
                continue;
            }
            // DOM clone step 6 follows the ordinary-descendants branch and is
            // independent of subtree: a shallow host copy still copies a
            // clonable shadow root and all its children deeply.
            if !frame.shadow_started {
                frame.shadow_started = true;
                if let Some(source) = original.hosted_shadow_root() {
                    let state = snapshot.shadow_root(source)?.state();
                    if state.clonable() {
                        let root = self.attach_shadow(
                            frame.copy,
                            state.properties().into(),
                            limits,
                            cancelled,
                        )?;
                        let document = frame.document;
                        reserve(&mut frames, 1)?;
                        frames.push(Frame {
                            source,
                            copy: root,
                            document,
                            parent: None,
                            template_child: 0,
                            appended: true,
                            next_child: 0,
                            subtree: true,
                            shadow_started: false,
                        });
                        continue;
                    }
                }
            }
            frames.pop();
        }
        Ok(root)
    }

    fn clone_single(
        &mut self,
        snapshot: &DomSnapshot,
        source: NodeId,
        document: Key,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        check_cancel(cancelled)?;
        let original = snapshot.node(source)?;
        let kind = match original.kind() {
            DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_) => {
                return Err(DomError::NotSupported);
            }
            DomNodeKind::Element(element) => {
                // Recreate actual common/family initialization from the name.
                // Canvas dimensions follow copied attrs; Marquee starts on/0;
                // Frame starts with actual null child-content; Iframe with the
                // actual fresh native unprocessed/null-child tuple. This genuine
                // clone cause is not retained-state installation from current null.
                // Snapshot Rust Clone is deliberately a different operation.
                // Attribute change steps precede source nonce cloning below.
                DomNodeKind::Element(
                    element.clone_initialized(self.native_environment(document)?, cancelled)?,
                )
            }
            // These immutable value payloads contain exact intrinsic fields
            // and no foreign node/document references. In particular PI clone
            // copies current data (possibly edited to contain ?>), not new()'s
            // stricter initial-construction input. Its derived map stays coherent.
            DomNodeKind::DocumentFragment => DomNodeKind::DocumentFragment,
            DomNodeKind::Text(data) => DomNodeKind::Text(data.clone()),
            DomNodeKind::Comment(data) => DomNodeKind::Comment(data.clone()),
            DomNodeKind::ProcessingInstruction(value) => {
                DomNodeKind::ProcessingInstruction(value.clone())
            }
            DomNodeKind::DocumentType(value) => DomNodeKind::DocumentType(value.clone()),
        };
        let copy = self.allocate(kind, document, limits)?;
        self.initialize_native_element(copy, limits, cancelled)?;
        if let Some(element) = original.as_element() {
            let cause = if element
                .input_state()
                .is_some_and(|state| state.input_type() == surgeist_html::HtmlInputType::Color)
            {
                attributes::AttributeCause::CloneInputColor
            } else {
                attributes::AttributeCause::Change
            };
            for attribute in element.attributes().iter() {
                check_cancel(cancelled)?;
                self.append_attribute(copy, attribute.clone(), cause, limits, cancelled)?;
            }
            if let Some(nonce) = element.nonce() {
                self.set_nonce(copy, nonce.clone(), limits, cancelled)?;
            }
        }
        if let Some(original) = original.node.script {
            self.node_mut(copy)
                .script
                .as_mut()
                .expect("actual fresh HTML Script")
                .flags
                .clone_already_started_from(original.flags);
        }
        if let Some(original) = original.as_input() {
            self.clone_input_overlay(copy, original.state(), limits, cancelled)?;
        }
        if let Some(original) = original.as_textarea() {
            self.clone_textarea_overlay(copy, original.state(), limits)?;
        }
        Ok(copy)
    }
}
