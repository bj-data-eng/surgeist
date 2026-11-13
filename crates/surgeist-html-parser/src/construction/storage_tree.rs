//! Tree navigation over the arena: containers, ordinary roots, connectivity and effect walks.

use super::*;
use surgeist_html::construction::{ConstructionNodeKind, HtmlFragmentOwner};

impl SessionStorage {
    #[cfg(test)]
    pub(super) fn template_contents(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let index = self.index(control, element)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.construction
            .node_at(index)
            .ok_or_else(|| latch_registry_row(control))?
            .template_contents()
            .map(|contents| self.node(control, contents.position()))
            .transpose()
    }

    pub(super) fn container(
        &self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<(), SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?
            .check_container()
            .map_err(|error| construction_error(control, error))
    }

    /// Qualifies the intended parent of an element being created. Creation
    /// reads only the parent's node document and registry and never adds a
    /// child, so a template element qualifies, unlike for [`Self::container`]:
    /// the adoption agency algorithm creates replacements with the template
    /// as the intended parent and later inserts them into its contents.
    pub(super) fn creation_parent(
        &self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<(), SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        match self
            .construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        {
            ConstructionNodeKind::Document(_)
            | ConstructionNodeKind::Fragment
            | ConstructionNodeKind::Element(_) => Ok(()),
            _ => Err(SessionError::NotContainer),
        }
    }

    /// Actual entry into an associated document leaves the closed ordinary
    /// producer inventory. This qualifies topology mutation, not preparation's
    /// source-ordered ordinary connectivity condition.
    pub(super) fn associated_document_parent(
        &mut self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<bool, SessionError> {
        let Self {
            construction,
            payloads,
            fragment,
            ..
        } = self;
        associated_document_parent_of(payloads, fragment, control, construction, index)
    }

    /// Only ordinary parents participate in form association. The single
    /// override represents the source removal intermediate after a move's
    /// coherent final topology has committed; it is not another tree.
    pub(super) fn ordinary_parent(
        &self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        ordinary_view_parent(
            ConstructionView::Active(&self.construction),
            control,
            node,
            detached,
        )
    }

    pub(super) fn ordinary_root(
        &self,
        control: &mut ParseControl,
        mut node: usize,
        detached: Option<usize>,
    ) -> Result<usize, SessionError> {
        if detached.is_none() {
            let storage = &self.construction;
            return self.payloads.tree_root_uncached(control, storage, node);
        }
        for _ in 0..self.construction.len() {
            match self.ordinary_parent(control, node, detached)? {
                Some(parent) => node = parent,
                None => return Ok(node),
            }
        }
        Err(SessionError::Cycle)
    }

    pub(super) fn html_local(
        &self,
        control: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<bool, SessionError> {
        let storage = &self.construction;
        Self::html_local_in(control, storage, node, local)
    }
    pub(super) fn html_local_in(
        control: &mut ParseControl,
        storage: &surgeist_html::bounded::construction::HtmlConstructionStorage,
        node: usize,
        local: &str,
    ) -> Result<bool, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
        let record = storage.node_at(node).ok_or(SessionError::ForeignNode)?;
        let ConstructionNodeKind::Element(element) = record.kind() else {
            return Ok(false);
        };
        Ok(exact_ascii(
            control,
            element.name().namespace().uri(),
            "http://www.w3.org/1999/xhtml",
        )? && exact_ascii(control, Some(element.name().local_name()), local)?)
    }

    /// Source tree order without traversal storage. The virtual removal view
    /// both detaches the root's parent and hides that child from outside trees.
    /// Its own subtree retains its exact internal child order.
    pub(super) fn next_view_node(
        &self,
        control: &mut ParseControl,
        root: usize,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        next_ordinary_view_node(
            ConstructionView::Active(&self.construction),
            control,
            root,
            node,
            detached,
        )
    }

    /// Whether no node follows `node` in `root`'s subtree in tree order, apart
    /// from `node`'s own descendants: it ends every ancestor's children up to
    /// `root`.
    pub(super) fn ends_subtree(
        &self,
        c: &mut ParseControl,
        root: usize,
        node: usize,
    ) -> Result<bool, SessionError> {
        let mut node = node;
        for _ in 0..self.construction.len() {
            let Some(parent) = self.ordinary_parent(c, node, None)? else {
                return Ok(false);
            };
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            let record = self
                .construction
                .node_at(parent)
                .ok_or_else(|| latch_registry_row(c))?;
            let last = record.last_child().map(|child| child.position());
            if last != Some(node) {
                return Ok(false);
            }
            if parent == root {
                return Ok(true);
            }
            node = parent;
        }
        Err(SessionError::Cycle)
    }
    pub(super) fn next_effect_node(
        &self,
        control: &mut ParseControl,
        root: usize,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        self.next_view_node(control, root, node, None)
    }

    pub(super) fn connected(
        &self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        let root = self.ordinary_root(control, node, detached)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(matches!(
            self.construction
                .node_at(root)
                .ok_or(SessionError::ForeignNode)?
                .kind(),
            ConstructionNodeKind::Document(_)
        ))
    }

    /// DOM3071 connected is a shadow-including root, not ordinary tree membership.
    /// The virtual removed root applies to every owned host hop as well.
    pub(super) fn shadow_including_connected(
        &self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        let mut root = self.ordinary_root(control, node, detached)?;
        'roots: for _ in 0..self.construction.len() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if matches!(
                self.construction
                    .node_at(root)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                ConstructionNodeKind::Document(_)
            ) {
                return Ok(true);
            }
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(HtmlFragmentOwner::ShadowHost(host)) = &self
                .construction
                .node_at(root)
                .ok_or_else(|| latch_registry_row(control))?
                .fragment_owner()
            {
                control.charge(
                    HtmlParseWorkCategory::CopyByte,
                    mem::size_of::<usize>() as u64,
                )?;
                control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                let host_position = self
                    .construction
                    .node_by_result_id(host)
                    .map_err(|_| SessionError::ForeignNode)?
                    .position();
                root = self.ordinary_root(control, host_position, detached)?;
                continue;
            }
            // A retained captured attachment is an influencing host, not proof
            // of disconnection. The adapter cannot invent its connectedness.
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(fragment) = &self.fragment {
                for effect in fragment.effects.as_slice() {
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if let context::ContextEffect::AttachShadow {
                        contents: Some(contents),
                        host,
                        ..
                    } = effect
                        && *contents == root
                    {
                        match host {
                            context::ShadowHostKey::Owned(host) => {
                                control.charge(
                                    HtmlParseWorkCategory::CopyByte,
                                    mem::size_of::<usize>() as u64,
                                )?;
                                root = self.ordinary_root(control, *host, detached)?;
                                continue 'roots;
                            }
                            context::ShadowHostKey::Captured(host) => {
                                return match self.captured_shadow_including_document(control, host)
                                {
                                    Ok(document) => Ok(document.is_some()),
                                    Err(error) => Err(end_session_on(control, error)),
                                };
                            }
                        }
                    }
                }
            }
            return Ok(false);
        }
        Err(SessionError::Cycle)
    }
}
