//! Resolve authentic capture facts against one immutable Layout graph.

use std::collections::{HashMap, HashSet};

use super::{
    TextBindingError as E, TextParagraphBinding, TextScopeAssociation, TextScopeBinding,
    binding_error,
};
use crate::engine::validation::ValidationGraph;
use surgeist_text::{
    CapturedControl, CapturedDomainDefinition, CapturedEventPosition, CapturedFlowEvent as Event,
    CapturedParagraph, CapturedScope, PreparedScope, PreparedText, ScopeRole,
};

type ResultOf<Tree, T> = crate::LayoutResultOf<
    <Tree as crate::Traverse>::Node,
    T,
    <Tree as crate::Traverse>::Scalar,
    <Tree as crate::LayoutTree>::MeasureError,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParagraphClass {
    Flow,
    AssociationOnly,
}

pub(crate) fn classify<Node>(paragraph: &TextParagraphBinding<Node>) -> Result<ParagraphClass, E> {
    if !paragraph.nodes.is_empty() {
        return Ok(ParagraphClass::Flow);
    }
    if paragraph.prepared.captured_paragraph().is_some()
        && paragraph.prepared.origin_views().next().is_none()
        && paragraph.prepared.control_views().next().is_none()
        && paragraph.prepared.atomic_views().next().is_none()
        && paragraph.prepared.participants().next().is_none()
        && paragraph.prepared.scopes().next().is_none()
        && paragraph.prepared.virtual_separators().next().is_none()
        && paragraph.origins.is_empty()
        && paragraph.controls.is_empty()
        && paragraph.atomics.is_empty()
        && paragraph.line_strut.is_none()
    {
        Ok(ParagraphClass::AssociationOnly)
    } else {
        Err(E::EmptyRun)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScopeAssociations<Node> {
    inline: Vec<InlineScope<Node>>,
    paragraphs: Vec<ParagraphResolution<Node>>,
    domains: Vec<Domain<Node>>,
    regions: Vec<CapturedScope>,
    structural: Vec<CapturedScope>,
}
#[derive(Clone, Debug, PartialEq)]
struct InlineScope<Node> {
    scope: CapturedScope,
    start: Node,
    end: Node,
}
#[derive(Clone, Debug, PartialEq)]
struct Domain<Node> {
    scope: CapturedScope,
    container: Node,
}
#[derive(Clone, Debug, PartialEq)]
struct ParagraphResolution<Node> {
    prepared: PreparedText,
    container: Node,
    captured: Option<CapturedParagraph>,
    class: ParagraphClass,
    scopes: Vec<(PreparedScope, Association)>,
}
#[derive(Clone, Debug, PartialEq)]
enum Association {
    Domain,
    Inline(usize),
}
impl<Node> Default for ScopeAssociations<Node> {
    fn default() -> Self {
        Self {
            inline: Vec::new(),
            paragraphs: Vec::new(),
            domains: Vec::new(),
            regions: Vec::new(),
            structural: Vec::new(),
        }
    }
}
impl<Node: Copy + Eq> ScopeAssociations<Node> {
    pub fn paragraph_is_flow(&self, index: usize) -> bool {
        self.paragraphs[index].class == ParagraphClass::Flow
    }
    pub fn binding(&self, scope: &PreparedScope) -> Option<TextScopeBinding<Node>> {
        for paragraph in &self.paragraphs {
            if let Some((_, association)) = paragraph.scopes.iter().find(|(s, _)| s == scope) {
                let association = match association {
                    Association::Domain => {
                        TextScopeAssociation::ParagraphContainer(paragraph.container)
                    }
                    Association::Inline(index) => {
                        let row = &self.inline[*index];
                        TextScopeAssociation::Inline {
                            start: row.start,
                            end: row.end,
                        }
                    }
                };
                return Some(TextScopeBinding {
                    scope: scope.clone(),
                    association,
                });
            }
        }
        None
    }
}
struct Control<Node> {
    captured: CapturedControl,
    node: Node,
    paragraph: usize,
}
struct Atomic<Node> {
    position: CapturedEventPosition,
    node: Node,
    paragraph: usize,
    kind: surgeist_text::InlineBoxKind,
}
struct Origin<Node> {
    position: CapturedEventPosition,
    node: Node,
    paragraph: usize,
}
struct Actual<Node> {
    event: Event,
    node: Node,
    key: (usize, u8),
}

#[derive(Default)]
struct NodeEvents {
    start: Vec<(CapturedScope, Event)>,
    body: Vec<(CapturedScope, Event)>,
    end: Vec<(CapturedScope, Event)>,
}
struct PositionedEvent {
    position: CapturedEventPosition,
    event: Event,
}
struct ExpectedFlow {
    events: Vec<PositionedEvent>,
    paragraphs: HashMap<CapturedParagraph, (usize, usize)>,
    previous: Vec<Option<usize>>,
    next: Vec<Option<usize>>,
}
struct GapScan<'a> {
    previous: Option<&'a Event>,
    next: Option<&'a Event>,
    inside: bool,
    complete: bool,
}

struct Proof<'a, Tree: crate::LayoutTree> {
    tree: &'a Tree,
    graph: &'a ValidationGraph<Tree::Node>,
    paragraphs: &'a [TextParagraphBinding<Tree::Node>],
    result: ScopeAssociations<Tree::Node>,
    controls: Vec<Control<Tree::Node>>,
    atomics: Vec<Atomic<Tree::Node>>,
    origins: Vec<Origin<Tree::Node>>,
    scopes: Vec<CapturedScope>,
    domains: Vec<CapturedScope>,
    paths: Vec<u8>,
    enter: Vec<usize>,
    exit: Vec<usize>,
    depth: Vec<usize>,
    node_events: Vec<NodeEvents>,
    node_domains: Vec<Vec<usize>>,
    expected: HashMap<CapturedScope, ExpectedFlow>,
    paragraph_indices: HashMap<CapturedParagraph, usize>,
    metadata: HashSet<CapturedParagraph>,
    nearest: HashMap<CapturedScope, Option<CapturedScope>>,
    enclosing: HashMap<CapturedScope, Option<CapturedScope>>,
}

pub(super) fn resolve<Tree: crate::LayoutTree>(
    tree: &Tree,
    graph: &ValidationGraph<Tree::Node>,
    paragraphs: &[TextParagraphBinding<Tree::Node>],
) -> ResultOf<Tree, ScopeAssociations<Tree::Node>> {
    let (enter, exit) = graph_order(graph);
    let mut proof = Proof {
        tree,
        graph,
        paragraphs,
        result: ScopeAssociations::default(),
        controls: Vec::new(),
        atomics: Vec::new(),
        origins: Vec::new(),
        scopes: Vec::new(),
        domains: Vec::new(),
        paths: graph.path_counts(),
        enter,
        exit,
        depth: vec![0; graph.nodes.len()],
        node_events: (0..graph.nodes.len())
            .map(|_| NodeEvents::default())
            .collect(),
        node_domains: vec![Vec::new(); graph.nodes.len()],
        expected: HashMap::new(),
        paragraph_indices: HashMap::new(),
        metadata: HashSet::new(),
        nearest: HashMap::new(),
        enclosing: HashMap::new(),
    };
    proof.index()?;
    proof.prepare_observations()?;
    // A is selected from producer topology before resolving its supplied rows.
    let regions = proof.result.regions.clone();
    for region in &regions {
        for event in proof
            .selected(region)
            .iter()
            .map(|entry| entry.event.clone())
            .collect::<Vec<_>>()
        {
            proof.require_expected(&event)?;
            if let Event::BlockStart(child) | Event::BlockEnd(child) = event {
                push_unique(&mut proof.domains, child);
            }
        }
    }
    proof.close_associations()?;
    // Unexpected real observations inside a region are retained too. Their
    // structural identities need validation but never activate another A region.
    for region in &regions {
        for child in proof.region_domains(region)? {
            push_unique(&mut proof.domains, child);
        }
    }
    proof.close_associations()?;
    proof.validate_pairs()?;
    proof.validate_metadata()?;
    for region in &regions {
        proof.compare_region(region)?;
    }
    for (index, paragraph) in paragraphs.iter().enumerate() {
        if proof.result.paragraphs[index].class == ParagraphClass::AssociationOnly {
            proof.metadata_slot(index)?;
        }
        for scope in paragraph.prepared.scopes() {
            let canonical = scope.captured_scope();
            let association = match canonical.role() {
                ScopeRole::Domain => Association::Domain,
                ScopeRole::Inline => Association::Inline(
                    proof
                        .result
                        .inline
                        .iter()
                        .position(|row| row.scope == canonical)
                        .ok_or_else(|| binding_error(paragraph.container, E::ScopeCoverage))?,
                ),
            };
            proof.result.paragraphs[index]
                .scopes
                .push((scope, association));
        }
        validate_virtuals(paragraph)
            .map_err(|reason| binding_error(paragraph.container, reason))?;
    }
    proof.result.structural = proof.domains;
    Ok(proof.result)
}

impl<Tree: crate::LayoutTree> Proof<'_, Tree> {
    fn index(&mut self) -> ResultOf<Tree, ()> {
        for (index, paragraph) in self.paragraphs.iter().enumerate() {
            let captured = paragraph.prepared.captured_paragraph();
            let class =
                classify(paragraph).map_err(|reason| binding_error(paragraph.container, reason))?;
            if let Some(captured) = &captured {
                if self
                    .result
                    .paragraphs
                    .iter()
                    .any(|p| p.captured.as_ref() == Some(captured))
                {
                    return Err(binding_error(paragraph.container, E::OverlappingParagraphs));
                }
                let domain = captured.domain();
                if let Some(old) = self.result.domains.iter().find(|d| d.scope == domain) {
                    if old.container != paragraph.container {
                        return Err(binding_error(paragraph.container, E::ScopeAssociation));
                    }
                } else {
                    if self.result.domains.iter().any(|d| {
                        d.container == paragraph.container && same_capture(&d.scope, &domain)
                    }) {
                        return Err(binding_error(paragraph.container, E::ScopeAssociation));
                    }
                    self.result.domains.push(Domain {
                        scope: domain.clone(),
                        container: paragraph.container,
                    });
                }
                if class == ParagraphClass::AssociationOnly {
                    push_unique(&mut self.domains, domain);
                }
            }
            for row in &paragraph.controls {
                if let Some(control) = row.occurrence.captured_control() {
                    if captured.as_ref() != Some(&control.paragraph())
                        || self.controls.iter().any(|old| old.captured == control)
                    {
                        return Err(binding_error(row.node, E::ControlCoverage));
                    }
                    self.controls.push(Control {
                        captured: control,
                        node: row.node,
                        paragraph: index,
                    });
                }
            }
            for row in &paragraph.atomics {
                if let Some(position) = row.occurrence.view().capture_position() {
                    if self.atomics.iter().any(|old| old.position == position) {
                        return Err(binding_error(row.node, E::AtomicCoverage));
                    }
                    self.atomics.push(Atomic {
                        position,
                        node: row.node,
                        paragraph: index,
                        kind: row.occurrence.view().original_box().kind(),
                    });
                }
            }
            for row in &paragraph.origins {
                if let Some(position) = row.origin.view().capture_position() {
                    if self.origins.iter().any(|old| old.position == position) {
                        return Err(binding_error(row.node, E::OriginCoverage));
                    }
                    self.origins.push(Origin {
                        position,
                        node: row.node,
                        paragraph: index,
                    });
                }
            }
            for scope in paragraph.prepared.scopes() {
                let canonical = scope.captured_scope();
                let Some(captured) = &captured else {
                    return Err(binding_error(paragraph.container, E::ScopeCoverage));
                };
                if canonical.domain() != captured.domain()
                    || !captured
                        .scope_intersections()
                        .any(|i| i.scope() == canonical)
                {
                    return Err(binding_error(
                        paragraph.container,
                        E::ScopeOccurrenceMembership,
                    ));
                }
                push_unique(&mut self.scopes, canonical.clone());
                push_unique(&mut self.domains, canonical.domain());
                add_region(&mut self.result.regions, canonical);
            }
            self.result.paragraphs.push(ParagraphResolution {
                prepared: paragraph.prepared.clone(),
                container: paragraph.container,
                captured,
                class,
                scopes: Vec::new(),
            });
        }
        Ok(())
    }
    fn paragraph(&self, captured: &CapturedParagraph) -> Option<usize> {
        self.paragraph_indices.get(captured).copied()
    }
    fn domain(&self, scope: &CapturedScope) -> ResultOf<Tree, Domain<Tree::Node>> {
        self.result
            .domains
            .iter()
            .find(|d| &d.scope == scope)
            .cloned()
            .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))
    }
    fn require_node(&self, node: Tree::Node) -> ResultOf<Tree, usize> {
        let index = self
            .graph
            .index(node)
            .ok_or_else(|| binding_error(node, E::ScopeAssociation))?;
        if self.paths[index] != 1 {
            return Err(binding_error(node, E::ScopeTreeAmbiguity));
        }
        Ok(index)
    }
    fn require_expected(&self, event: &Event) -> ResultOf<Tree, ()> {
        let (node, paragraph) = match event {
            Event::ParagraphStart(p) | Event::ParagraphEnd(p) => {
                let index = self
                    .paragraph(p)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                (self.paragraphs[index].container, Some(index))
            }
            Event::Origin {
                paragraph,
                position,
            } => {
                let row = self
                    .origins
                    .iter()
                    .find(|o| &o.position == position)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                if self.result.paragraphs[row.paragraph].captured.as_ref() != Some(paragraph) {
                    return Err(binding_error(row.node, E::OriginCoverage));
                }
                (row.node, Some(row.paragraph))
            }
            Event::Atomic {
                paragraph,
                position,
            } => {
                let row = self
                    .atomics
                    .iter()
                    .find(|a| &a.position == position)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                if self.result.paragraphs[row.paragraph].captured.as_ref() != Some(paragraph) {
                    return Err(binding_error(row.node, E::AtomicCoverage));
                }
                (row.node, Some(row.paragraph))
            }
            Event::Control(control) => {
                let row = self
                    .controls
                    .iter()
                    .find(|c| &c.captured == control)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                (row.node, Some(row.paragraph))
            }
            Event::BlockStart(scope) | Event::BlockEnd(scope) => {
                (self.domain(scope)?.container, None)
            }
        };
        self.require_node(node)?;
        if let Some(index) = paragraph {
            self.require_node(self.paragraphs[index].container)?;
        }
        Ok(())
    }
    fn close_associations(&mut self) -> ResultOf<Tree, ()> {
        let mut b = 0;
        let mut c = 0;
        while b < self.scopes.len() || c < self.domains.len() {
            while b < self.scopes.len() {
                let scope = self.scopes[b].clone();
                b += 1;
                self.scope_association(scope)?;
            }
            while c < self.domains.len() {
                let domain = self.domains[c].clone();
                c += 1;
                self.domain_association(domain)?;
            }
        }
        Ok(())
    }
    fn ancestors(
        &mut self,
        mut scope: Option<CapturedScope>,
        domain: &CapturedScope,
        nominee: Option<&CapturedEventPosition>,
        site: Tree::Node,
    ) -> ResultOf<Tree, Option<CapturedScope>> {
        // Capture construction proves nested canonical extents. Containment in
        // the immediate retained inline therefore proves all its ancestors too.
        if let (Some(parent), Some(position)) = (&scope, nominee)
            && parent.domain() == *domain
            && parent.role() == ScopeRole::Inline
            && !(parent.enter_position() < *position && *position < parent.exit_position())
        {
            return Err(binding_error(site, E::ScopeAssociation));
        }
        let mut trail = Vec::new();
        let nearest = loop {
            let Some(parent) = scope else {
                break None;
            };
            if &parent.domain() != domain || parent.role() == ScopeRole::Domain {
                break None;
            }
            if parent.entry_control().is_some() && parent.exit_control().is_some() {
                break Some(parent);
            }
            if let Some(known) = self.nearest.get(&parent) {
                break known.clone();
            }
            scope = parent.parent();
            trail.push(parent);
        };
        for parent in trail {
            self.nearest.insert(parent, nearest.clone());
        }
        if let Some(parent) = &nearest {
            push_unique(&mut self.scopes, parent.clone());
        }
        // Processing that B work item closes the remaining complete ancestors.
        Ok(nearest)
    }
    fn scope_association(&mut self, scope: CapturedScope) -> ResultOf<Tree, ()> {
        let domain = self.domain(&scope.domain())?;
        push_unique(&mut self.domains, scope.domain());
        let parent = self.ancestors(scope.parent(), &scope.domain(), None, domain.container)?;
        self.enclosing.insert(scope.clone(), parent);
        if scope.role() == ScopeRole::Domain || self.result.inline.iter().any(|s| s.scope == scope)
        {
            return Ok(());
        }
        let (Some(entry), Some(exit)) = (scope.entry_control(), scope.exit_control()) else {
            return Err(binding_error(domain.container, E::ScopeBoundaryCapture));
        };
        let start = self
            .controls
            .iter()
            .find(|c| c.captured == entry)
            .ok_or_else(|| binding_error(domain.container, E::ScopeAssociation))?;
        let end = self
            .controls
            .iter()
            .find(|c| c.captured == exit)
            .ok_or_else(|| binding_error(domain.container, E::ScopeAssociation))?;
        let exact = |c: &Control<Tree::Node>, kind| {
            c.captured.paragraph().domain() == scope.domain()
                && self.paragraphs[c.paragraph].container == domain.container
                && matches!(self.tree.layout_input(c.node), crate::LayoutInputViewOf::InlineBoundary(input) if input.kind() == kind)
        };
        if start.node == end.node
            || !exact(start, crate::InlineBoundaryKind::Start)
            || !exact(end, crate::InlineBoundaryKind::End)
        {
            return Err(binding_error(domain.container, E::ScopeAssociation));
        }
        self.require_node(start.node)?;
        self.require_node(end.node)?;
        self.result.inline.push(InlineScope {
            scope,
            start: start.node,
            end: end.node,
        });
        Ok(())
    }
    fn domain_association(&mut self, scope: CapturedScope) -> ResultOf<Tree, ()> {
        let domain = self.domain(&scope)?;
        self.require_node(domain.container)?;
        if self.tree.layout_input(domain.container).as_box().is_none() {
            return Err(binding_error(domain.container, E::ScopeAssociation));
        }
        let parent = scope.parent().map(|p| p.domain());
        let definition = scope
            .domain_definition()
            .ok_or_else(|| binding_error(domain.container, E::ScopeAssociation))?;
        match definition {
            CapturedDomainDefinition::Block => {
                if self.atomics.iter().any(|a| {
                    a.node == domain.container && a.position.same_capture(&scope.enter_position())
                }) {
                    return Err(binding_error(domain.container, E::ScopeAssociation));
                }
                if domain.container == self.graph.nodes[0] {
                    return Ok(());
                }
                if let Some(parent) = parent {
                    let actual_parent = self.domain(&parent)?;
                    self.require_node(actual_parent.container)?;
                    if !descendant(self.graph, actual_parent.container, domain.container) {
                        return Err(binding_error(domain.container, E::ScopeAssociation));
                    }
                    self.no_intervening(&scope, actual_parent.container, domain.container)?;
                    let enclosing =
                        self.ancestors(scope.parent(), &parent, None, domain.container)?;
                    self.enclosing.insert(scope.clone(), enclosing);
                    push_unique(&mut self.domains, parent);
                }
            }
            CapturedDomainDefinition::AtomicBlock { atomic }
            | CapturedDomainDefinition::OutOfFlowBlock { atomic } => {
                let row = self
                    .atomics
                    .iter()
                    .find(|a| a.position == atomic)
                    .ok_or_else(|| binding_error(domain.container, E::ScopeAssociation))?;
                let expected = if matches!(
                    scope.domain_definition(),
                    Some(CapturedDomainDefinition::AtomicBlock { .. })
                ) {
                    surgeist_text::InlineBoxKind::InFlow
                } else {
                    surgeist_text::InlineBoxKind::OutOfFlow
                };
                if row.node != domain.container || row.kind != expected {
                    return Err(binding_error(domain.container, E::ScopeAssociation));
                }
                let captured = self.result.paragraphs[row.paragraph]
                    .captured
                    .as_ref()
                    .ok_or_else(|| binding_error(row.node, E::ScopeAssociation))?;
                let parent = parent.ok_or_else(|| binding_error(row.node, E::ScopeAssociation))?;
                if captured.domain() != parent {
                    return Err(binding_error(row.node, E::ScopeAssociation));
                }
                let actual_parent = self.domain(&parent)?;
                if !descendant(self.graph, actual_parent.container, domain.container) {
                    return Err(binding_error(domain.container, E::ScopeAssociation));
                }
                self.no_intervening(&scope, actual_parent.container, domain.container)?;
                let enclosing =
                    self.ancestors(scope.parent(), &parent, Some(&atomic), domain.container)?;
                self.enclosing.insert(scope.clone(), enclosing);
                push_unique(&mut self.domains, parent);
            }
        }
        Ok(())
    }
    fn no_intervening(
        &self,
        scope: &CapturedScope,
        parent: Tree::Node,
        node: Tree::Node,
    ) -> ResultOf<Tree, ()> {
        for other in &self.result.domains {
            if other.container != parent
                && other.container != node
                && same_capture(scope, &other.scope)
                && descendant(self.graph, parent, other.container)
                && descendant(self.graph, other.container, node)
                && !scope_ancestor(&other.scope, scope)
            {
                return Err(binding_error(node, E::ScopeAssociation));
            }
        }
        Ok(())
    }
    fn validate_pairs(&self) -> ResultOf<Tree, ()> {
        let positions: Vec<_> = self
            .result
            .inline
            .iter()
            .map(|row| {
                Ok((
                    self.enter[self.require_node(row.start)?],
                    self.enter[self.require_node(row.end)?],
                ))
            })
            .collect::<ResultOf<Tree, _>>()?;
        for (index, row) in self.result.inline.iter().enumerate() {
            let (start, end) = positions[index];
            if start >= end {
                return Err(binding_error(row.start, E::ScopeBoundaryOrder));
            }
            for (other_index, other) in self.result.inline[..index].iter().enumerate() {
                if row.scope.domain() != other.scope.domain() {
                    continue;
                }
                let (first, last) = positions[other_index];
                if interval_relation(
                    &row.scope.enter_position(),
                    &row.scope.exit_position(),
                    &other.scope.enter_position(),
                    &other.scope.exit_position(),
                ) != interval_relation(&start, &end, &first, &last)
                {
                    return Err(binding_error(row.start, E::ScopeBoundaryOrder));
                }
            }
        }
        for scope in &self.domains {
            let Some(Some(parent)) = self.enclosing.get(scope) else {
                continue;
            };
            let index = self
                .result
                .inline
                .iter()
                .position(|s| &s.scope == parent)
                .expect("closed enclosing B association");
            let (first, last) = positions[index];
            let domain = self.domain(scope)?;
            let node = self.require_node(domain.container)?;
            if !(first < self.enter[node] && self.exit[node] < last) {
                return Err(binding_error(domain.container, E::ScopeAssociation));
            }
        }
        Ok(())
    }
    fn roots(&self, region: &CapturedScope) -> ResultOf<Tree, Vec<usize>> {
        let domain = self.domain(&region.domain())?;
        let node = self.require_node(domain.container)?;
        if region.role() == ScopeRole::Domain {
            return Ok(self.graph.children[node].clone());
        }
        let pair = self
            .result
            .inline
            .iter()
            .find(|s| &s.scope == region)
            .ok_or_else(|| binding_error(domain.container, E::ScopeCoverage))?;
        let start = self.graph.index(pair.start).expect("resolved boundary");
        let end = self.graph.index(pair.end).expect("resolved boundary");
        let first = self.graph.children[node]
            .iter()
            .position(|&n| n == start)
            .ok_or_else(|| binding_error(pair.start, E::ScopeAssociation))?;
        let last = self.graph.children[node]
            .iter()
            .position(|&n| n == end)
            .ok_or_else(|| binding_error(pair.end, E::ScopeAssociation))?;
        if first > last {
            return Err(binding_error(pair.start, E::ScopeBoundaryOrder));
        }
        Ok(self.graph.children[node][first..=last].to_vec())
    }
    fn region_domains(&self, region: &CapturedScope) -> ResultOf<Tree, Vec<CapturedScope>> {
        let mut result = Vec::new();
        let mut seen = vec![false; self.graph.nodes.len()];
        let mut pending = self.roots(region)?;
        while let Some(index) = pending.pop() {
            if seen[index] {
                continue;
            }
            seen[index] = true;
            if let Some(domain) = self.node_domains[index]
                .iter()
                .map(|&i| &self.result.domains[i])
                .find(|d| d.scope != region.domain() && same_capture(&d.scope, region))
            {
                self.require_node(domain.container)?;
                push_unique(&mut result, domain.scope.clone());
            } else {
                pending.extend(self.graph.children[index].iter().copied());
            }
        }
        Ok(result)
    }
    fn actual(
        &self,
        domain: &CapturedScope,
        roots: Vec<usize>,
        strict: bool,
        mut gap: Option<GapScan<'_>>,
    ) -> ResultOf<Tree, Vec<Actual<Tree::Node>>> {
        let mut result = Vec::new();
        // A region has one bounded traversal. For an empty slot, a shared node
        // seen before its authentic left anchor must be revisited if encountered
        // inside the candidate gap. Two epochs remain bounded by graph size.
        let mut seen = vec![false; self.graph.nodes.len() * 2];
        let mut pending: Vec<_> = roots.into_iter().rev().map(|i| (i, false, false)).collect();
        while let Some((index, exiting, opaque)) = pending.pop() {
            let node = self.graph.nodes[index];
            if exiting {
                for (_, event) in self.node_events[index]
                    .end
                    .iter()
                    .filter(|(d, _)| d == domain)
                {
                    let item = Actual {
                        event: event.clone(),
                        node,
                        key: (
                            if opaque {
                                self.enter[index]
                            } else {
                                self.exit[index]
                            },
                            4,
                        ),
                    };
                    if self.observe(item, strict, &mut gap, &mut result)? {
                        return Ok(result);
                    }
                }
                continue;
            }
            let epoch = usize::from(gap.as_ref().is_some_and(|g| g.inside));
            let slot = index + epoch * self.graph.nodes.len();
            if seen[slot] {
                continue;
            }
            seen[slot] = true;
            for (_, event) in self.node_events[index]
                .start
                .iter()
                .filter(|(d, _)| d == domain)
            {
                let item = Actual {
                    event: event.clone(),
                    node,
                    key: (self.enter[index], 0),
                };
                if self.observe(item, strict, &mut gap, &mut result)? {
                    return Ok(result);
                }
            }
            for (_, event) in self.node_events[index]
                .body
                .iter()
                .filter(|(d, _)| d == domain)
            {
                let item = Actual {
                    event: event.clone(),
                    node,
                    key: (self.enter[index], 1),
                };
                if self.observe(item, strict, &mut gap, &mut result)? {
                    return Ok(result);
                }
            }
            let child = self.node_domains[index]
                .iter()
                .map(|&i| &self.result.domains[i])
                .find(|d| &d.scope != domain && same_capture(&d.scope, domain));
            if let Some(child) = child {
                if strict {
                    self.require_node(node)?;
                }
                if matches!(
                    child.scope.domain_definition(),
                    Some(CapturedDomainDefinition::Block)
                ) && child.scope.parent().is_some_and(|p| &p.domain() == domain)
                {
                    for (event, key) in [
                        (
                            Event::BlockStart(child.scope.clone()),
                            (self.enter[index], 2),
                        ),
                        (Event::BlockEnd(child.scope.clone()), (self.exit[index], 0)),
                    ] {
                        if self.observe(
                            Actual { event, node, key },
                            strict,
                            &mut gap,
                            &mut result,
                        )? {
                            return Ok(result);
                        }
                    }
                }
                pending.push((index, true, true));
            } else {
                pending.push((index, true, false));
                pending.extend(
                    self.graph.children[index]
                        .iter()
                        .rev()
                        .map(|&i| (i, false, false)),
                );
            }
        }
        if gap
            .as_ref()
            .is_some_and(|g| !g.inside || g.next.is_some() && !g.complete)
        {
            return Err(binding_error(
                self.domain(domain)?.container,
                E::ScopeAssociation,
            ));
        }
        Ok(result)
    }
    fn observe(
        &self,
        item: Actual<Tree::Node>,
        strict: bool,
        gap: &mut Option<GapScan<'_>>,
        result: &mut Vec<Actual<Tree::Node>>,
    ) -> ResultOf<Tree, bool> {
        if let Some(gap) = gap {
            if !gap.inside {
                if gap.previous == Some(&item.event) {
                    self.require_node(item.node)?;
                    gap.inside = true;
                    result.push(item);
                }
            } else {
                self.require_node(item.node)?;
                if gap.next != Some(&item.event) {
                    return Err(binding_error(item.node, E::ScopeAssociation));
                }
                gap.complete = true;
                result.push(item);
                return Ok(true);
            }
        } else {
            if strict {
                self.require_node(item.node)?;
            }
            result.push(item);
        }
        Ok(false)
    }
    fn compare_region(&self, region: &CapturedScope) -> ResultOf<Tree, ()> {
        let mut expected = Vec::new();
        for entry in self.selected(region) {
            let event = entry.event.clone();
            if let Event::ParagraphStart(p) | Event::ParagraphEnd(p) = &event {
                let index = self
                    .paragraph(p)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                if self.result.paragraphs[index].class == ParagraphClass::AssociationOnly {
                    self.metadata_slot(index)?;
                    continue;
                }
            }
            expected.push(event);
        }
        let mut actual = self.actual(&region.domain(), self.roots(region)?, true, None)?;
        if region.role() == ScopeRole::Inline {
            let pair = self
                .result
                .inline
                .iter()
                .find(|s| &s.scope == region)
                .expect("resolved region endpoints");
            let lower = (self.enter[self.require_node(pair.start)?], 1);
            let upper = (self.enter[self.require_node(pair.end)?], 1);
            actual.retain(|observation| lower <= observation.key && observation.key <= upper);
        }
        if expected.iter().ne(actual.iter().map(|a| &a.event)) {
            return Err(binding_error(
                self.domain(&region.domain())?.container,
                E::ParticipantOrder,
            ));
        }
        Ok(())
    }
    fn metadata_slot(&self, index: usize) -> ResultOf<Tree, ()> {
        let paragraph = self.result.paragraphs[index]
            .captured
            .as_ref()
            .expect("classified metadata");
        if !self.metadata.contains(paragraph) {
            return Err(binding_error(
                self.paragraphs[index].container,
                E::ScopeAssociation,
            ));
        }
        Ok(())
    }
    fn validate_metadata(&mut self) -> ResultOf<Tree, ()> {
        let mut proven = HashSet::new();
        for (index, paragraph) in self.result.paragraphs.iter().enumerate() {
            if paragraph.class != ParagraphClass::AssociationOnly {
                continue;
            }
            let captured = paragraph
                .captured
                .as_ref()
                .expect("classified captured metadata");
            let domain = captured.domain();
            let flow = &self.expected[&domain];
            let &(first, last) = flow
                .paragraphs
                .get(captured)
                .ok_or_else(|| binding_error(paragraph.container, E::ScopeAssociation))?;
            if last != first + 1 {
                return Err(binding_error(paragraph.container, E::ScopeAssociation));
            }
            let previous = flow.previous[first];
            let next = flow.next[last];
            if proven.insert((domain.clone(), previous, next)) {
                let before = previous.map(|i| &flow.events[i].event);
                let after = next.map(|i| &flow.events[i].event);
                let roots = self.gap_roots(&domain, before, after)?;
                self.actual(
                    &domain,
                    roots,
                    false,
                    Some(GapScan {
                        previous: before,
                        next: after,
                        inside: before.is_none(),
                        complete: false,
                    }),
                )?;
            }
            self.metadata.insert(captured.clone());
            // Every supplied identity is checked even when several share a slot.
            self.require_node(self.paragraphs[index].container)?;
        }
        Ok(())
    }
    fn event_node(&self, event: &Event) -> ResultOf<Tree, Tree::Node> {
        self.require_expected(event)?;
        let node = match event {
            Event::ParagraphStart(p) | Event::ParagraphEnd(p) => {
                let index = self.paragraph(p).expect("required paragraph");
                let nodes = &self.paragraphs[index].nodes;
                if matches!(event, Event::ParagraphStart(_)) {
                    nodes.first()
                } else {
                    nodes.last()
                }
                .copied()
                .ok_or_else(|| {
                    binding_error(self.paragraphs[index].container, E::ScopeAssociation)
                })?
            }
            Event::Origin { position, .. } => {
                self.origins
                    .iter()
                    .find(|o| &o.position == position)
                    .expect("required origin")
                    .node
            }
            Event::Atomic { position, .. } => {
                self.atomics
                    .iter()
                    .find(|a| &a.position == position)
                    .expect("required atomic")
                    .node
            }
            Event::Control(control) => {
                self.controls
                    .iter()
                    .find(|c| &c.captured == control)
                    .expect("required control")
                    .node
            }
            Event::BlockStart(scope) | Event::BlockEnd(scope) => self.domain(scope)?.container,
        };
        self.require_node(node)?;
        Ok(node)
    }
    fn gap_roots(
        &self,
        domain: &CapturedScope,
        previous: Option<&Event>,
        next: Option<&Event>,
    ) -> ResultOf<Tree, Vec<usize>> {
        let container = self.require_node(self.domain(domain)?.container)?;
        let left = previous
            .map(|e| self.event_node(e).and_then(|n| self.require_node(n)))
            .transpose()?;
        let right = next
            .map(|e| self.event_node(e).and_then(|n| self.require_node(n)))
            .transpose()?;
        let (Some(mut first), Some(mut last)) = (left, right) else {
            let anchor = left.or(right);
            if let Some(mut node) = anchor {
                while self.graph.parents[node].first().copied() != Some(container) {
                    node = *self.graph.parents[node].first().ok_or_else(|| {
                        binding_error(self.graph.nodes[node], E::ScopeAssociation)
                    })?;
                }
                let position = self.graph.children[container]
                    .iter()
                    .position(|&i| i == node)
                    .expect("unique child edge");
                return Ok(if left.is_some() {
                    self.graph.children[container][position..].to_vec()
                } else {
                    self.graph.children[container][..=position].to_vec()
                });
            }
            return Ok(self.graph.children[container].clone());
        };
        if first == last {
            return Ok(vec![first]);
        }
        while self.depth[first] > self.depth[last] {
            first = self.graph.parents[first][0];
        }
        while self.depth[last] > self.depth[first] {
            last = self.graph.parents[last][0];
        }
        while first != last {
            first = self.graph.parents[first][0];
            last = self.graph.parents[last][0];
        }
        let ancestor = first;
        let branch = |mut node: usize| {
            if node == ancestor {
                return None;
            }
            while self.graph.parents[node][0] != ancestor {
                node = self.graph.parents[node][0];
            }
            Some(node)
        };
        let (Some(first), Some(last)) = (
            branch(left.expect("left anchor")),
            branch(right.expect("right anchor")),
        ) else {
            return Ok(vec![ancestor]);
        };
        let first = self.graph.children[ancestor]
            .iter()
            .position(|&i| i == first)
            .expect("left branch");
        let last = self.graph.children[ancestor]
            .iter()
            .position(|&i| i == last)
            .expect("right branch");
        if first > last {
            return Err(binding_error(
                self.graph.nodes[ancestor],
                E::ScopeAssociation,
            ));
        }
        Ok(self.graph.children[ancestor][first..=last].to_vec())
    }
    fn selected(&self, scope: &CapturedScope) -> &[PositionedEvent] {
        let events = &self.expected[&scope.domain()].events;
        let start = scope.enter_position();
        let end = scope.exit_position();
        let first = events.partition_point(|event| event.position <= start);
        let last = events.partition_point(|event| event.position < end);
        &events[first..last]
    }
    fn prepare_observations(&mut self) -> ResultOf<Tree, ()> {
        for &node in self.graph.finish.iter().rev() {
            for &child in &self.graph.children[node] {
                if self.paths[child] == 1 {
                    self.depth[child] = self.depth[node] + 1;
                }
            }
        }
        for (index, paragraph) in self.result.paragraphs.iter().enumerate() {
            let Some(captured) = &paragraph.captured else {
                continue;
            };
            self.paragraph_indices.insert(captured.clone(), index);
            if paragraph.class == ParagraphClass::Flow {
                let row = &self.paragraphs[index];
                let first = self.graph.index(row.nodes[0]).expect("validated run node");
                let last = self
                    .graph
                    .index(*row.nodes.last().expect("flow run"))
                    .expect("validated run node");
                self.node_events[first]
                    .start
                    .push((captured.domain(), Event::ParagraphStart(captured.clone())));
                self.node_events[last]
                    .end
                    .push((captured.domain(), Event::ParagraphEnd(captured.clone())));
            }
        }
        for row in &self.origins {
            let p = self.result.paragraphs[row.paragraph]
                .captured
                .as_ref()
                .expect("captured origin");
            self.node_events[self.graph.index(row.node).expect("validated origin")]
                .body
                .push((
                    p.domain(),
                    Event::Origin {
                        paragraph: p.clone(),
                        position: row.position.clone(),
                    },
                ));
        }
        for row in &self.controls {
            self.node_events[self.graph.index(row.node).expect("validated control")]
                .body
                .push((
                    row.captured.paragraph().domain(),
                    Event::Control(row.captured.clone()),
                ));
        }
        for row in &self.atomics {
            let p = self.result.paragraphs[row.paragraph]
                .captured
                .as_ref()
                .expect("captured atomic");
            self.node_events[self.graph.index(row.node).expect("validated atomic")]
                .body
                .push((
                    p.domain(),
                    Event::Atomic {
                        paragraph: p.clone(),
                        position: row.position.clone(),
                    },
                ));
        }
        for (index, domain) in self.result.domains.iter().enumerate() {
            self.node_domains[self
                .graph
                .index(domain.container)
                .expect("validated container")]
            .push(index);
        }
        let mut domains = Vec::new();
        for region in &self.result.regions {
            push_unique(&mut domains, region.domain());
        }
        for paragraph in &self.result.paragraphs {
            if paragraph.class == ParagraphClass::AssociationOnly {
                push_unique(
                    &mut domains,
                    paragraph
                        .captured
                        .as_ref()
                        .expect("captured metadata")
                        .domain(),
                );
            }
        }
        for domain in domains {
            let events: Vec<_> = domain
                .flow_events()
                .expect("domain flow")
                .map(|event| PositionedEvent {
                    position: event_position(&event),
                    event,
                })
                .collect();
            let mut paragraphs = HashMap::new();
            for (index, event) in events.iter().enumerate() {
                match &event.event {
                    Event::ParagraphStart(p) => {
                        paragraphs.insert(p.clone(), (index, index));
                    }
                    Event::ParagraphEnd(p) => {
                        paragraphs.get_mut(p).expect("owner paragraph nesting").1 = index;
                    }
                    _ => {}
                }
            }
            let visible = |event: &Event| match event {
                Event::ParagraphStart(p) | Event::ParagraphEnd(p) => self.paragraph(p).map_or_else(
                    || {
                        let (first, last) = paragraphs[p];
                        last != first + 1
                    },
                    |i| self.result.paragraphs[i].class == ParagraphClass::Flow,
                ),
                _ => true,
            };
            let mut previous = vec![None; events.len()];
            let mut next = previous.clone();
            let mut last = None;
            for (index, event) in events.iter().enumerate() {
                previous[index] = last;
                if visible(&event.event) {
                    last = Some(index);
                }
            }
            last = None;
            for (index, event) in events.iter().enumerate().rev() {
                next[index] = last;
                if visible(&event.event) {
                    last = Some(index);
                }
            }
            self.expected.insert(
                domain,
                ExpectedFlow {
                    events,
                    paragraphs,
                    previous,
                    next,
                },
            );
        }
        // Source-neighbor Block slots used to locate supplied empty incidence
        // are C obligations, not new A regions.
        for paragraph in &self.result.paragraphs {
            if paragraph.class != ParagraphClass::AssociationOnly {
                continue;
            }
            let captured = paragraph.captured.as_ref().expect("captured metadata");
            let flow = &self.expected[&captured.domain()];
            let (first, last) = flow.paragraphs[captured];
            for i in [flow.previous[first], flow.next[last]]
                .into_iter()
                .flatten()
            {
                if let Event::BlockStart(child) | Event::BlockEnd(child) = &flow.events[i].event {
                    push_unique(&mut self.domains, child.clone());
                }
            }
        }
        Ok(())
    }
}

fn push_unique<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}
fn same_capture(first: &CapturedScope, second: &CapturedScope) -> bool {
    first
        .enter_position()
        .same_capture(&second.enter_position())
}
fn scope_ancestor(ancestor: &CapturedScope, scope: &CapturedScope) -> bool {
    let mut parent = scope.parent();
    while let Some(p) = parent {
        if &p == ancestor {
            return true;
        }
        parent = p.parent();
    }
    false
}
fn add_region(regions: &mut Vec<CapturedScope>, scope: CapturedScope) {
    if regions.iter().any(|r| {
        r.domain() == scope.domain()
            && r.enter_position() <= scope.enter_position()
            && scope.exit_position() <= r.exit_position()
    }) {
        return;
    }
    regions.retain(|r| {
        r.domain() != scope.domain()
            || !(scope.enter_position() <= r.enter_position()
                && r.exit_position() <= scope.exit_position())
    });
    regions.push(scope);
}
fn event_position(event: &Event) -> CapturedEventPosition {
    match event {
        Event::ParagraphStart(p) => p.enter_position(),
        Event::ParagraphEnd(p) => p.exit_position(),
        Event::Origin { position, .. } | Event::Atomic { position, .. } => position.clone(),
        Event::Control(c) => c.event_position(),
        Event::BlockStart(s) => s.enter_position(),
        Event::BlockEnd(s) => s.exit_position(),
    }
}
fn descendant<Node: Copy + Eq>(graph: &ValidationGraph<Node>, parent: Node, child: Node) -> bool {
    let (Some(parent), Some(child)) = (graph.index(parent), graph.index(child)) else {
        return false;
    };
    if parent == child {
        return false;
    }
    let mut seen = vec![false; graph.nodes.len()];
    let mut pending = graph.parents[child].clone();
    while let Some(node) = pending.pop() {
        if node == parent {
            return true;
        }
        if !seen[node] {
            seen[node] = true;
            pending.extend(graph.parents[node].iter().copied());
        }
    }
    false
}
fn graph_order<Node: Copy + Eq>(graph: &ValidationGraph<Node>) -> (Vec<usize>, Vec<usize>) {
    let mut enter = vec![0; graph.nodes.len()];
    let mut exit = enter.clone();
    let mut seen = vec![false; graph.nodes.len()];
    let mut pending = vec![(0, false)];
    let mut clock = 0;
    while let Some((node, done)) = pending.pop() {
        if done {
            exit[node] = clock;
            clock += 1;
            continue;
        }
        if seen[node] {
            continue;
        }
        seen[node] = true;
        enter[node] = clock;
        clock += 1;
        pending.push((node, true));
        pending.extend(graph.children[node].iter().rev().map(|&i| (i, false)));
    }
    (enter, exit)
}
fn interval_relation<T: PartialOrd>(start: &T, end: &T, first: &T, last: &T) -> u8 {
    if end < first {
        1
    } else if last < start {
        2
    } else if start < first && last < end {
        3
    } else if first < start && end < last {
        4
    } else {
        0
    }
}

fn validate_virtuals<Node: Copy + Eq>(paragraph: &TextParagraphBinding<Node>) -> Result<(), E> {
    for occurrence in paragraph.prepared.virtual_separators() {
        let view = occurrence.view();
        let scope = view.scope().captured_scope();
        let Some(captured) = paragraph.prepared.captured_paragraph() else {
            return Err(E::ScopeOccurrenceMembership);
        };
        if captured.domain() != scope.domain()
            || !captured
                .scope_intersections()
                .any(|intersection| intersection.scope() == scope)
        {
            return Err(E::ScopeOccurrenceMembership);
        }
        let preceding: Vec<_> = view.preceding_controls().collect();
        let following: Vec<_> = view.following_controls().collect();
        let mut previous = None;
        for control in preceding.iter().chain(&following) {
            let Some(current) = paragraph
                .controls
                .iter()
                .find(|row| &row.occurrence == control)
                .and_then(|row| paragraph.nodes.iter().position(|node| *node == row.node))
            else {
                return Err(E::ScopeOccurrenceMembership);
            };
            if previous.is_some_and(|p| p >= current) {
                return Err(E::ScopeOccurrenceMembership);
            }
            previous = Some(current);
        }
        for (endpoint, local) in [
            (scope.entry_control(), preceding),
            (scope.exit_control(), following),
        ] {
            if let Some(endpoint) = endpoint
                && endpoint.paragraph() == captured
                && !local
                    .iter()
                    .any(|control| control.captured_control().as_ref() == Some(&endpoint))
            {
                return Err(E::ScopeOccurrenceMembership);
            }
        }
    }
    Ok(())
}
