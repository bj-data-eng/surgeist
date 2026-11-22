//! Atomic canonical application of the bounded local HTML construction profile.

use super::*;
use surgeist_html_parser::live::{HtmlLiveAttributes, HtmlLiveElement, HtmlLiveTree};

/// Original cumulative native work admission for one hosted local parser.
pub struct DomHostedLocalHtmlEdits {
    document: DocumentId,
    global: DomHostedGlobalId,
    work: Arc<Work>,
}

impl DomHostedSource {
    /// Requalify the resource-free local profile against exact current tree
    /// ownership and complete native changes since the caller's checkpoint.
    /// Only inert ordinary edits qualify. Missing history is a refusal.
    pub fn local_html_resource_free_changes_since(
        &self,
        dom: &Dom,
        document: DocumentId,
        global: DomHostedGlobalId,
        checkpoint: DomStamp,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedError> {
        self.local_html_current_document(dom, document, global, cancel)?;
        let work = self.work(cancel)?;
        resource_free_changes_since(dom, document, checkpoint, &work, cancel)
    }

    /// Check guest changes made at one parser script pause using the original
    /// canonical parser work owner, without extending its retained history.
    pub fn local_html_script_changes_since(
        &self,
        dom: &Dom,
        edits: &DomHostedLocalHtmlEdits,
        checkpoint: DomStamp,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedError> {
        self.qualify_hosted_local_html_parser(dom, edits.document, edits.global, cancel)?;
        if !current_script_registration(dom, edits, cancel)? {
            return Err(DomHostedError::ScriptingDisabled);
        }
        resource_free_changes_since(dom, edits.document, checkpoint, &edits.work, cancel)
    }

    /// Bind the native edit budget to the current loading Document and Global.
    pub fn begin_local_html_edits(
        &self,
        dom: &Dom,
        document: DocumentId,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedLocalHtmlEdits, DomHostedError> {
        self.qualify_hosted_local_html_parser(dom, document, global, cancel)?;
        Ok(DomHostedLocalHtmlEdits {
            document,
            global,
            work: self.work(cancel)?,
        })
    }

    /// Borrow the qualified canonical tree while the original parser advances.
    /// Every atomic edit shares the original native work admission, including
    /// copies of existing canonical data. Borrowing again cannot reset it.
    pub fn local_html_tree<'a>(
        &self,
        dom: &'a mut Dom,
        edits: &'a DomHostedLocalHtmlEdits,
        cancel: &'a AtomicBool,
    ) -> Result<impl HtmlLiveTree<Node = NodeId, Error = DomError> + 'a, DomHostedError> {
        self.qualify_hosted_local_html_parser(dom, edits.document, edits.global, cancel)?;
        let limits = crate::internal::source::DomSourceLimits::new(
            self.remaining_bytes()?.min(self.limits().scratch_bytes()),
            dom.limits.units,
            self.limits().work(),
            0,
            0,
            0,
            1,
        );
        Ok(LocalHtmlTree {
            dom,
            edits,
            limits,
            cancel,
        })
    }
}

fn current_script_registration(
    dom: &Dom,
    edits: &DomHostedLocalHtmlEdits,
    cancel: &AtomicBool,
) -> Result<bool, DomHostedError> {
    edits.work.step(cancel)?;
    let Some(registration) =
        dom.script_registration
            .as_ref()
            .map(|slot| &slot.0)
            .filter(|registration| {
                registration.is_active()
                    && registration.global == edits.global
                    && registration.key.document == edits.document
            })
    else {
        return Ok(false);
    };
    dom.dispatch_script_registration_current_with(registration, || edits.work.step(cancel))
}

fn resource_free_changes_since(
    dom: &Dom,
    document: DocumentId,
    checkpoint: DomStamp,
    work: &Work,
    cancel: &AtomicBool,
) -> Result<bool, DomHostedError> {
    work.step(cancel)?;
    if checkpoint.owner != dom.owner {
        return Err(DomHostedError::ForeignSource);
    }
    dom.history.changes_since_satisfy(
        checkpoint.revision,
        dom.revision,
        || work.step(cancel),
        |change| {
            let id = match change {
                DomChange::SourceOperation | DomChange::ChildListObservation => {
                    return Ok(true);
                }
                DomChange::Created(id)
                | DomChange::Data(id)
                | DomChange::Attributes(id)
                | DomChange::ElementState(id)
                | DomChange::Parent(id)
                | DomChange::Children(id) => *id,
                DomChange::AttrValue(id) | DomChange::AttrOwner(id) => id.0,
                _ => return Ok(false),
            };
            let node = match dom
                .state
                .node_with(dom.owner, id, &mut || work.step(cancel))
            {
                Ok(node) => node,
                Err(DomHostedError::Dom(DomError::StaleNode)) => return Ok(false),
                Err(error) => return Err(error),
            };
            if node.document() != document {
                return Ok(false);
            }
            Ok(match node.kind() {
                DomNodeKind::Element(element) => {
                    let name = element.name();
                    crate::internal::names::in_html_namespace_with(name, &mut || work.step(cancel))?
                        && ["html", "head", "body", "div", "span", "p"]
                            .iter()
                            .any(|tag| {
                                name.local_name()
                                    .code_units()
                                    .iter()
                                    .copied()
                                    .eq(tag.encode_utf16())
                            })
                }
                DomNodeKind::Attr(attribute) => {
                    let name = attribute.name();
                    name.namespace_uri().is_none()
                        && ["id", "class"].iter().any(|attr| {
                            name.local_name()
                                .code_units()
                                .iter()
                                .copied()
                                .eq(attr.encode_utf16())
                        })
                }
                DomNodeKind::Text(_) | DomNodeKind::Comment(_) => true,
                _ => false,
            })
        },
    )
}

/// One advance borrows the canonical DOM. The parser retains node identities
/// across advances and the next advance reads the current revision.
struct LocalHtmlTree<'a> {
    dom: &'a mut Dom,
    edits: &'a DomHostedLocalHtmlEdits,
    limits: crate::internal::source::DomSourceLimits,
    cancel: &'a AtomicBool,
}

enum ScriptEdit<'a> {
    None,
    Inserted(&'a CreatedNode),
    Prepare(NodeId),
}

impl LocalHtmlTree<'_> {
    fn apply(
        &mut self,
        batch: DomBatch,
        script: ScriptEdit<'_>,
    ) -> Result<(DomCommit, bool), DomError> {
        let result = (|| {
            let (mut draft, _) = Draft::new_source_root_with_work(
                self.dom,
                batch.owner,
                source::initial::CommandStorage {
                    commands: &batch.commands,
                    capacity: batch.commands.capacity(),
                },
                self.limits,
                source::initial::Replay {
                    work: self.edits.work.spent.clone(),
                    identity: self.edits.work.spent_id,
                    names: None,
                },
                self.cancel,
            )?;
            // This is an atomic batch. A reached asynchronous hook is refused
            // by the ordinary edit path before any publication.
            draft.phase_map = None;
            let document = Key::Existing(self.edits.document.node());
            for (index, command) in batch.commands.into_iter().enumerate() {
                draft.source_step(self.cancel)?;
                draft.apply(&command, self.dom.limits, self.cancel)?;
                if index == 0
                    && let ScriptEdit::Inserted(element) = script
                {
                    let element = draft.resolve(element.into())?;
                    draft.initialize_parser_inserted_script(element, document, self.cancel)?;
                }
            }
            let executable = match script {
                ScriptEdit::Prepare(script) => draft.prepare_local_parser_classic_script(
                    Key::Existing(script),
                    document,
                    self.cancel,
                )?,
                _ => false,
            };
            let prepared = draft.finish(self.dom, self.cancel)?;
            Ok((self.dom.publish(prepared, self.cancel)?, executable))
        })();
        self.dom.custody_registry.sweep_blocks();
        result
    }

    fn qualify_parent(&self, parent: NodeId) -> Result<(), DomError> {
        let snapshot = self.dom.snapshot();
        let node = snapshot.node(parent)?;
        if (parent == self.edits.document.node() && matches!(node.kind(), DomNodeKind::Document(_)))
            || (node.owner_document() == Some(self.edits.document)
                && matches!(node.kind(), DomNodeKind::Element(_)))
        {
            Ok(())
        } else {
            Err(DomError::WrongDocument)
        }
    }

    fn adjacent_text(&self, parent: NodeId) -> Result<Option<(NodeId, usize)>, DomError> {
        let snapshot = self.dom.snapshot();
        let parent = snapshot.node(parent)?;
        let Some(&last) = parent.children().last() else {
            return Ok(None);
        };
        Ok(match snapshot.node(last)?.kind() {
            DomNodeKind::Text(data) => Some((last, data.code_units().len())),
            _ => None,
        })
    }
}

impl HtmlLiveTree for LocalHtmlTree<'_> {
    type Node = NodeId;
    type Error = DomError;

    fn insert_doctype(&mut self, document: NodeId) -> Result<(), DomError> {
        if document != self.edits.document.node() {
            return Err(DomError::WrongDocument);
        }
        self.qualify_parent(document)?;
        let mut batch = DomBatch::new();
        let doctype = batch.create_document_type(document, "html", "", "");
        batch.insert_before(&doctype, document, None);
        self.apply(batch, ScriptEdit::None)?;
        Ok(())
    }

    fn insert_element(
        &mut self,
        parent: NodeId,
        name: HtmlLiveElement,
        attributes: HtmlLiveAttributes<'_>,
    ) -> Result<NodeId, DomError> {
        self.qualify_parent(parent)?;
        let mut batch = DomBatch::new();
        let element = batch.create_element(self.edits.document.node(), name.as_str());
        for (name, value) in attributes.iter() {
            batch.set_attribute(&element, name.as_str(), value.clone());
        }
        batch.insert_before(&element, parent, None);
        let script = if name == HtmlLiveElement::Script {
            ScriptEdit::Inserted(&element)
        } else {
            ScriptEdit::None
        };
        let (commit, _) = self.apply(batch, script)?;
        // The original batch creates this exact output and cannot retire it.
        Ok(commit
            .created(&element)
            .expect("the creation ticket belongs to this batch")
            .expect("the created element survives this batch"))
    }

    fn insert_comment(&mut self, parent: NodeId, text: &DocumentString) -> Result<(), DomError> {
        self.qualify_parent(parent)?;
        let mut batch = DomBatch::new();
        let node = batch.create_comment(self.edits.document.node(), text.clone());
        batch.insert_before(&node, parent, None);
        self.apply(batch, ScriptEdit::None)?;
        Ok(())
    }

    fn insert_text(&mut self, parent: NodeId, text: &DocumentString) -> Result<(), DomError> {
        self.qualify_parent(parent)?;
        if parent == self.edits.document.node() || text.code_units().is_empty() {
            return Ok(());
        }
        let mut batch = DomBatch::new();
        if let Some((node, end)) = self.adjacent_text(parent)? {
            batch.replace_data(node, end, 0, text.clone());
        } else {
            let node = batch.create_text(self.edits.document.node(), text.clone());
            batch.insert_before(&node, parent, None);
        }
        self.apply(batch, ScriptEdit::None)?;
        Ok(())
    }

    fn prepare_classic_script(&mut self, script: NodeId) -> Result<bool, DomError> {
        self.qualify_parent(script)?;
        if !current_script_registration(self.dom, self.edits, self.cancel).map_err(|error| {
            match error {
                DomHostedError::Dom(error) => error,
                DomHostedError::WorkLimit { limit } => DomError::SourceWorkLimit { limit },
                DomHostedError::RepresentationExhausted => DomError::RepresentationExhausted,
                _ => DomError::HostedSourceUnavailable,
            }
        })? {
            return Err(DomError::ScriptPreparationRequired);
        }
        let (_, executable) = self.apply(DomBatch::new(), ScriptEdit::Prepare(script))?;
        Ok(executable)
    }
}
