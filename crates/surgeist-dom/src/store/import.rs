//! Supported native DOM clone/import over a checked immutable source capture.
use super::*;

struct Frame {
    source: NodeId,
    copy: Key,
    next_child: usize,
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
        if !subtree {
            return Ok(root);
        }
        let mut frames = Vec::new();
        reserve(&mut frames, 1)?;
        frames.push(Frame {
            source,
            copy: root,
            next_child: 0,
        });
        while let Some(frame) = frames.last_mut() {
            check_cancel(cancelled)?;
            let original = snapshot.node(frame.source)?;
            let Some(&source) = original.children().get(frame.next_child) else {
                frames.pop();
                continue;
            };
            frame.next_child += 1;
            let parent = frame.copy;
            let copy = self.clone_single(snapshot, source, document, limits, cancelled)?;
            // DOM clones/initializes a child and appends it before traversing
            // that child's own descendants. No source parent IDs enter Draft.
            self.pre_insert(copy, parent, None, cancelled)?;
            reserve(&mut frames, 1)?;
            frames.push(Frame {
                source,
                copy,
                next_child: 0,
            });
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
            DomNodeKind::Document(_) => return Err(DomError::NotSupported),
            DomNodeKind::Element(element) => {
                // Recreate actual common initialization from the captured name.
                // Special families cannot enter this canonical surface yet.
                // Attribute change steps precede source nonce cloning below.
                DomNodeKind::Element(element.clone_initialized(cancelled)?)
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
        if let Some(element) = original.as_element() {
            for attribute in element.attributes().iter() {
                check_cancel(cancelled)?;
                self.append_cloned_attribute(copy, attribute.clone(), limits, cancelled)?;
            }
            if let Some(nonce) = element.nonce() {
                self.set_nonce(copy, nonce.clone(), limits, cancelled)?;
            }
        }
        Ok(copy)
    }
}
