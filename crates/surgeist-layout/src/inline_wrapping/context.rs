use super::*;
use crate::{InlineBoundaryKind, LayoutTree, engine::validation::ValidationGraph};

/// Canonical metadata; only requested Avoid scopes activate their full region.
#[derive(Clone, Debug)]
pub(crate) struct CanonicalScopeProof<Node> {
    pub(crate) scope: CapturedScope,
    pub(crate) flow: Node,
    pub(crate) pair: Option<(Node, Node)>,
    pub(crate) order: Option<Arc<CanonicalOrderBindings<Node>>>,
}
/// Endpoint evidence retains the explicit native identity; it does not invent
/// a canonical scope for an additional native box.
#[derive(Clone, Debug)]
pub(crate) enum WrappingScopeProof<Node> {
    Captured(CanonicalScopeProof<Node>),
    NativePair {
        flow: Node,
        pair: (Node, Node),
        start: surgeist_text::CapturedControl,
        end: surgeist_text::CapturedControl,
        order: Option<Arc<CanonicalOrderBindings<Node>>>,
    },
}
impl<Node> WrappingScopeProof<Node> {
    fn canonical(&self) -> Option<&CanonicalScopeProof<Node>> {
        match self {
            Self::Captured(proof) => Some(proof),
            Self::NativePair { .. } => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum SourceExtent {
    Captured(CapturedScope),
    Native {
        start: surgeist_text::CapturedControl,
        end: surgeist_text::CapturedControl,
    },
}
impl SourceExtent {
    fn canonical(scope: &CapturedScope) -> Self {
        Self::Captured(scope.clone())
    }
    fn native(
        start: &surgeist_text::CapturedControl,
        end: &surgeist_text::CapturedControl,
    ) -> Result<Self, WrappingScopeError> {
        let first = start.event_position();
        let last = end.event_position();
        let domain = start.paragraph().domain();
        if domain != end.paragraph().domain()
            || !first.same_capture(&last)
            || first.partial_cmp(&last) != Some(std::cmp::Ordering::Less)
            || !matches!(
                start.kind(),
                surgeist_text::ParagraphControlKind::InlineBoundaryStart
            )
            || !matches!(
                end.kind(),
                surgeist_text::ParagraphControlKind::InlineBoundaryEnd
            )
        {
            return Err(WrappingScopeError::CaptureAssociation);
        }
        Ok(Self::Native {
            start: start.clone(),
            end: end.clone(),
        })
    }
    fn domain(&self) -> CapturedScope {
        match self {
            Self::Captured(scope) => scope.domain(),
            Self::Native { start, .. } => start.paragraph().domain(),
        }
    }
    fn start(&self) -> surgeist_text::CapturedEventPosition {
        match self {
            Self::Captured(scope) => scope.enter_position(),
            Self::Native { start, .. } => start.event_position(),
        }
    }
    fn end(&self) -> surgeist_text::CapturedEventPosition {
        match self {
            Self::Captured(scope) => scope.exit_position(),
            Self::Native { end, .. } => end.event_position(),
        }
    }
    fn owner(&self) -> Option<CapturedScope> {
        match self {
            Self::Captured(scope) => Some(scope.clone()),
            Self::Native { start, end } => {
                let owner = start.scope();
                (owner == end.scope()).then_some(owner)
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativeSpan {
    pub(crate) first: usize,
    pub(crate) last: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScopeRow<Node> {
    pub(crate) flow: Node,
    pub(crate) capture: Option<CapturedScope>,
    pub(crate) pair: Option<(Node, Node)>,
    pub(crate) native: Option<NativeSpan>,
    pub(crate) policy: WrapInside,
    pub(crate) edges: WrapEdges,
    pub(crate) metric_contribution: Option<InlineBoxMetricContribution>,
    source: Option<SourceExtent>,
}
impl<Node> ScopeRow<Node> {
    /// Actual source owner, independently of this native box's identity and of
    /// whichever selected scope frame ultimately inherits the numeric leaf.
    pub(crate) fn source_owner(&self) -> Option<CapturedScope> {
        self.source.as_ref()?.owner()
    }
}
#[derive(Clone, Debug)]
pub(crate) struct WrappingContext<Node> {
    rows: Vec<ScopeRow<Node>>,
    ancestry: Arc<Ancestry>,
    has_avoid: bool,
    pub(crate) edge_plan: super::edge_plan::EdgePlan<Node>,
}
impl<Node> PartialEq for WrappingContext<Node> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ancestry, &other.ancestry)
    }
}
impl<Node> Eq for WrappingContext<Node> {}
impl<Node> Default for WrappingContext<Node> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            ancestry: Arc::default(),
            has_avoid: false,
            edge_plan: super::edge_plan::EdgePlan {
                locations: Vec::new(),
                dependencies: Vec::new(),
                subtrees: Vec::new(),
            },
        }
    }
}
fn validate_metrics(
    value: Option<InlineBoxMetricContribution>,
) -> Result<Option<InlineBoxMetricContribution>, WrappingScopeError> {
    Ok(match value {
        Some(InlineBoxMetricContribution::Strut(mut strut)) => {
            let metrics = &mut strut.metrics;
            if !metrics.baseline.is_finite()
                || !metrics.line_extent.is_finite()
                || metrics.baseline < 0.0
                || metrics.line_extent < metrics.baseline
            {
                return Err(WrappingScopeError::InvalidBoxMetrics);
            }
            if metrics.baseline == 0.0 {
                metrics.baseline = 0.0;
            }
            if metrics.line_extent == 0.0 {
                metrics.line_extent = 0.0;
            }
            Some(InlineBoxMetricContribution::Strut(strut))
        }
        other => other,
    })
}
fn captured_contains(a: &CapturedScope, b: &CapturedScope) -> bool {
    let mut next = b.parent();
    while let Some(scope) = next {
        if &scope == a {
            return true;
        }
        next = scope.parent();
    }
    false
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Relation {
    Before,
    After,
    Contains,
    Within,
    Crossing,
}
pub(super) fn interval_relation<T: PartialOrd>(a: &T, b: &T, c: &T, d: &T) -> Relation {
    if b < c {
        Relation::Before
    } else if d < a {
        Relation::After
    } else if a < c && d < b {
        Relation::Contains
    } else if c < a && b < d {
        Relation::Within
    } else {
        Relation::Crossing
    }
}
/// Combines independent actual proofs, refusing disagreement rather than
/// treating different proof representations as unrelated boxes.
fn proven_relation<Node: Eq>(
    a: &ScopeRow<Node>,
    b: &ScopeRow<Node>,
) -> Result<Option<Relation>, WrappingScopeError> {
    if a.flow != b.flow {
        return Ok(None);
    }
    let native = a
        .native
        .as_ref()
        .zip(b.native.as_ref())
        .map(|(a, b)| interval_relation(&a.first, &a.last, &b.first, &b.last));
    let source = a.source.as_ref().zip(b.source.as_ref()).and_then(|(a, b)| {
        (a.domain() == b.domain() && a.start().same_capture(&b.start()))
            .then(|| interval_relation(&a.start(), &a.end(), &b.start(), &b.end()))
    });
    if native == Some(Relation::Crossing) || source == Some(Relation::Crossing) {
        return Err(WrappingScopeError::CrossingScopes);
    }
    if native.zip(source).is_some_and(|(a, b)| a != b) {
        return Err(WrappingScopeError::CaptureAssociation);
    }
    let canonical = a
        .capture
        .as_ref()
        .zip(b.capture.as_ref())
        .and_then(|(a, b)| {
            if captured_contains(a, b) {
                Some(Relation::Contains)
            } else if captured_contains(b, a) {
                Some(Relation::Within)
            } else {
                None
            }
        });
    if a.capture.is_some() && b.capture.is_some() {
        for evidence in [native, source].into_iter().flatten() {
            if canonical.is_some_and(|known| known != evidence)
                || canonical.is_none() && matches!(evidence, Relation::Contains | Relation::Within)
            {
                return Err(WrappingScopeError::CaptureAssociation);
            }
        }
    }
    Ok(canonical.or(source).or(native))
}
pub(crate) fn native_span<Tree: LayoutTree>(
    tree: &Tree,
    graph: &ValidationGraph<Tree::Node>,
    start: Tree::Node,
    end: Tree::Node,
    counts: &[u8],
) -> Result<(Tree::Node, NativeSpan), WrappingScopeError> {
    native_span_with(graph, start, end, counts, |node| {
        tree.layout_input(node)
            .as_inline_boundary()
            .map(|v| v.kind())
    })
}

pub(super) fn native_span_with<Node: Copy + Eq>(
    graph: &ValidationGraph<Node>,
    start: Node,
    end: Node,
    counts: &[u8],
    kind: impl Fn(Node) -> Option<InlineBoundaryKind>,
) -> Result<(Node, NativeSpan), WrappingScopeError> {
    let a = graph
        .index(start)
        .ok_or(WrappingScopeError::NodeNotReachable)?;
    let b = graph
        .index(end)
        .ok_or(WrappingScopeError::NodeNotReachable)?;
    if counts[a] != 1 || counts[b] != 1 {
        return Err(WrappingScopeError::AmbiguousNode);
    }
    if kind(start) != Some(InlineBoundaryKind::Start) || kind(end) != Some(InlineBoundaryKind::End)
    {
        return Err(WrappingScopeError::EndpointRole);
    }
    let parent = *graph.parents[a]
        .first()
        .ok_or(WrappingScopeError::DifferentFlow)?;
    if graph.parents[b].as_slice() != [parent] {
        return Err(WrappingScopeError::DifferentFlow);
    }
    let children = &graph.children[parent];
    let first = children
        .iter()
        .position(|&i| i == a)
        .ok_or(WrappingScopeError::EndpointOrder)?;
    let last = children
        .iter()
        .position(|&i| i == b)
        .ok_or(WrappingScopeError::EndpointOrder)?;
    if first >= last {
        return Err(WrappingScopeError::EndpointOrder);
    }
    Ok((graph.nodes[parent], NativeSpan { first, last }))
}
fn ordered_ranks<Node: Copy + Eq>(
    graph: &ValidationGraph<Node>,
    owner: usize,
    nodes: &[Node],
) -> Result<Vec<usize>, WrappingScopeError> {
    let children = &graph.children[owner];
    let mut cursor = 0;
    let mut ranks = Vec::with_capacity(nodes.len());
    for &node in nodes {
        let Some(offset) = children[cursor..]
            .iter()
            .position(|&i| graph.nodes[i] == node)
        else {
            return Err(if children.iter().any(|&i| graph.nodes[i] == node) {
                WrappingScopeError::EndpointOrder
            } else {
                WrappingScopeError::DifferentFlow
            });
        };
        let rank = cursor + offset;
        ranks.push(rank);
        cursor = rank + 1;
    }
    Ok(ranks)
}
impl<Node: Copy + Eq> WrappingContext<Node> {
    pub(crate) fn new<Tree: LayoutTree<Node = Node>>(
        tree: &Tree,
        graph: &ValidationGraph<Node>,
        inputs: &[WrappingScopeInput<Node>],
        proofs: &[WrappingScopeProof<Node>],
        child_order: &[InlineChildOrderInput<Node>],
    ) -> Result<Self, WrappingScopeError> {
        if inputs.is_empty()
            && child_order.is_empty()
            && graph
                .nodes
                .iter()
                .all(|&n| tree.node_input(n).wrap_edges.is_auto())
        {
            return Ok(Self::default());
        }
        let counts = graph.path_counts();
        let mut rows: Vec<ScopeRow<Node>> = Vec::new();
        let mut forms: Vec<u8> = Vec::new();
        for input in inputs {
            let (row, form) = match input {
                WrappingScopeInput::NativePair {
                    start,
                    end,
                    wrap_inside,
                    edges,
                    metric_contribution,
                } => {
                    let (flow, native) = native_span(tree, graph, *start, *end, &counts)?;
                    let proof = proofs
                        .iter()
                        .filter_map(WrappingScopeProof::canonical)
                        .find(|p| p.pair == Some((*start, *end)));
                    if proofs
                        .iter()
                        .filter_map(WrappingScopeProof::canonical)
                        .any(|p| {
                            p.pair.is_some_and(|(a, b)| {
                                (a == *start || b == *end) && (a, b) != (*start, *end)
                            })
                        })
                    {
                        return Err(WrappingScopeError::CaptureAssociation);
                    }
                    if proof.is_some_and(|p| p.flow != flow) {
                        return Err(WrappingScopeError::CaptureAssociation);
                    }
                    (
                        ScopeRow {
                            flow,
                            capture: proof.map(|p| p.scope.clone()),
                            pair: Some((*start, *end)),
                            native: Some(native),
                            policy: *wrap_inside,
                            edges: *edges,
                            metric_contribution: validate_metrics(metric_contribution.clone())?,
                            source: if let Some(proof) = proof {
                                Some(SourceExtent::canonical(&proof.scope))
                            } else {
                                proofs
                                    .iter()
                                    .find_map(|proof| match proof {
                                        WrappingScopeProof::NativePair {
                                            pair,
                                            flow: actual,
                                            start: a,
                                            end: b,
                                            ..
                                        } if *pair == (*start, *end) => Some(if *actual == flow {
                                            SourceExtent::native(a, b)
                                        } else {
                                            Err(WrappingScopeError::CaptureAssociation)
                                        }),
                                        _ => None,
                                    })
                                    .transpose()?
                            },
                        },
                        1,
                    )
                }
                WrappingScopeInput::Captured {
                    scope,
                    wrap_inside,
                    edges,
                    metric_contribution,
                } => {
                    if scope.role() != surgeist_text::ScopeRole::Inline {
                        return Err(WrappingScopeError::NonInlineScope);
                    }
                    let proof = proofs
                        .iter()
                        .filter_map(WrappingScopeProof::canonical)
                        .find(|p| p.scope == *scope)
                        .ok_or(WrappingScopeError::MissingCapture)?;
                    let native = proof
                        .pair
                        .map(|(a, b)| native_span(tree, graph, a, b, &counts))
                        .transpose()?;
                    if native.as_ref().is_some_and(|(flow, _)| *flow != proof.flow) {
                        return Err(WrappingScopeError::CaptureAssociation);
                    }
                    if graph.index(proof.flow).is_none() {
                        return Err(WrappingScopeError::NodeNotReachable);
                    }
                    (
                        ScopeRow {
                            flow: proof.flow,
                            capture: Some(scope.clone()),
                            pair: proof.pair,
                            native: native.map(|(_, s)| s),
                            policy: *wrap_inside,
                            edges: *edges,
                            metric_contribution: validate_metrics(metric_contribution.clone())?,
                            source: Some(SourceExtent::canonical(scope)),
                        },
                        2,
                    )
                }
            };
            if let Some(i) = rows.iter().position(|p| {
                p.capture
                    .as_ref()
                    .zip(row.capture.as_ref())
                    .is_some_and(|(a, b)| a == b)
                    || p.pair.zip(row.pair).is_some_and(|(a, b)| a == b)
            }) {
                if let Some(value) = row.metric_contribution {
                    if rows[i]
                        .metric_contribution
                        .as_ref()
                        .is_some_and(|prior| prior != &value)
                    {
                        return Err(WrappingScopeError::ConflictingBoxMetrics);
                    }
                    rows[i].metric_contribution = Some(value);
                }
                if rows[i].policy != row.policy || rows[i].edges != row.edges {
                    return Err(WrappingScopeError::ConflictingScope);
                }
                if forms[i] & form != 0 {
                    return Err(WrappingScopeError::DuplicateScope);
                }
                forms[i] |= form;
                continue;
            }
            for prior in &rows {
                proven_relation(prior, &row)?;
            }
            rows.push(row);
            forms.push(form);
        }
        // Scope references are already owner-issued positive generated-box
        // inventory. They do not acquire fabricated authored declarations or
        // metric witnesses merely because they have no declaration row.
        #[expect(
            clippy::mutable_key_type,
            reason = "Text handle hashing uses immutable capture identity and occurrence index; lazy cache contents never affect the key"
        )]
        let mut inventory = std::collections::HashSet::new();
        for scope in child_order
            .iter()
            .flat_map(|order| {
                std::iter::once(&order.parent).chain(order.children.iter().map(|child| &child.box_))
            })
            .filter_map(|box_| {
                if let GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope)) = box_ {
                    Some(scope)
                } else {
                    None
                }
            })
        {
            if !inventory.insert(scope.clone())
                || rows.iter().any(|row| row.capture.as_ref() == Some(scope))
            {
                continue;
            }
            if scope.role() != surgeist_text::ScopeRole::Inline {
                return Err(WrappingScopeError::NonInlineScope);
            }
            let proof = proofs
                .iter()
                .filter_map(WrappingScopeProof::canonical)
                .find(|proof| proof.scope == *scope)
                .ok_or(WrappingScopeError::MissingCapture)?;
            let native = proof
                .pair
                .map(|(start, end)| native_span(tree, graph, start, end, &counts))
                .transpose()?;
            if native.as_ref().is_some_and(|(flow, _)| *flow != proof.flow) {
                return Err(WrappingScopeError::CaptureAssociation);
            }
            let row = ScopeRow {
                flow: proof.flow,
                capture: Some(scope.clone()),
                pair: proof.pair,
                native: native.map(|(_, span)| span),
                policy: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: None,
                source: Some(SourceExtent::canonical(scope)),
            };
            for prior in &rows {
                proven_relation(prior, &row)?;
            }
            rows.push(row);
        }
        let mut parents: Vec<Option<usize>> = vec![None; rows.len()];
        for i in 0..rows.len() {
            for j in 0..rows.len() {
                if i == j || proven_relation(&rows[j], &rows[i])? != Some(Relation::Contains) {
                    continue;
                }
                if let Some(parent) = parents[i]
                    && proven_relation(&rows[parent], &rows[j])? != Some(Relation::Contains)
                {
                    continue;
                }
                parents[i] = Some(j);
            }
        }
        let mut intervals = vec![(0, 0); rows.len()];
        let mut time = 1;
        let mut stack = Vec::new();
        for i in (0..rows.len()).rev() {
            if parents[i].is_none() {
                stack.push((i, false));
            }
        }
        while let Some((i, exit)) = stack.pop() {
            if exit {
                intervals[i].1 = time;
                continue;
            }
            intervals[i].0 = time;
            time += 1;
            stack.push((i, true));
            for j in (0..rows.len()).rev() {
                if parents[j] == Some(i) {
                    stack.push((j, false));
                }
            }
        }
        let has_avoid = rows.iter().any(|row| row.policy == WrapInside::Avoid);
        let edge_plan = if child_order.is_empty()
            && rows.iter().all(|r| r.edges.is_auto())
            && graph
                .nodes
                .iter()
                .all(|&n| tree.node_input(n).wrap_edges.is_auto())
        {
            super::edge_plan::EdgePlan {
                locations: Vec::new(),
                dependencies: Vec::new(),
                subtrees: Vec::new(),
            }
        } else {
            super::edge_plan::EdgePlan::new(tree, graph, &rows, child_order, proofs)?
        };
        Ok(Self {
            rows,
            ancestry: Arc::new(Ancestry { intervals }),
            has_avoid,
            edge_plan,
        })
    }
    pub(crate) fn has_effects(&self) -> bool {
        self.has_avoid
            || !self.edge_plan.dependencies.is_empty()
            || self
                .rows
                .iter()
                .any(|row| row.metric_contribution.is_some())
    }
    pub(crate) fn scope_pair(&self, scope: &CapturedScope) -> Option<(Node, Node)> {
        self.rows
            .iter()
            .find(|r| r.capture.as_ref() == Some(scope))
            .and_then(|r| r.pair)
    }
    pub(crate) fn edge_locations(
        &self,
        flow: Node,
    ) -> impl Iterator<Item = (usize, &super::edge_plan::EdgeLocation<Node>)> {
        self.edge_plan
            .locations
            .iter()
            .enumerate()
            .filter(move |(_, e)| e.flow == flow)
    }
    pub(crate) fn requires_source(&self, flow: Node) -> bool {
        self.edge_plan
            .locations
            .iter()
            .any(|e| e.flow == flow && e.effect.forced)
    }
    pub(crate) fn whole_subtree(&self, row: usize) -> Result<bool, WrappingScopeError> {
        use super::edge_plan::SubtreeRealization;
        match self.edge_plan.subtrees.get(row) {
            Some(SubtreeRealization::Empty) => Ok(true),
            Some(SubtreeRealization::Participating) => Ok(false),
            Some(SubtreeRealization::Unavailable) | None => {
                Err(WrappingScopeError::MissingChildOrder)
            }
        }
    }
    pub(crate) fn box_rows(&self, flow: Node) -> impl Iterator<Item = (usize, &ScopeRow<Node>)> {
        self.rows
            .iter()
            .enumerate()
            .filter(move |(_, row)| row.flow == flow)
    }
    pub(crate) fn rows(&self) -> impl Iterator<Item = &ScopeRow<Node>> {
        self.rows
            .iter()
            .filter(|row| row.policy == WrapInside::Avoid)
    }
    pub(crate) fn classify<E>(
        &self,
        flow: Node,
        mut inside: impl FnMut(&ScopeRow<Node>) -> Result<bool, E>,
    ) -> Result<AvoidClass, E>
    where
        E: From<WrappingScopeError>,
    {
        let mut deepest = None;
        for (i, row) in self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.flow == flow && row.policy == WrapInside::Avoid)
        {
            if !inside(row)? {
                continue;
            }
            if let Some(prior) = deepest {
                let (a, b) = self.ancestry.intervals[prior];
                let (c, d) = self.ancestry.intervals[i];
                if a <= c && d <= b {
                    deepest = Some(i);
                } else if !(c <= a && b <= d) {
                    return Err(WrappingScopeError::InconsistentIncidence.into());
                }
            } else {
                deepest = Some(i);
            }
        }
        Ok(AvoidClass {
            ancestry: Arc::clone(&self.ancestry),
            scope: deepest,
            edges: Vec::new(),
        })
    }
    pub(crate) fn numeric_projection(
        &self,
        graph: &ValidationGraph<Node>,
        flow: Node,
        nodes: &[Node],
    ) -> Result<NumericWrapping, WrappingScopeError> {
        let owner = graph
            .index(flow)
            .ok_or(WrappingScopeError::NodeNotReachable)?;
        let ranks = ordered_ranks(graph, owner, nodes)?;
        let requested: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.flow == flow && r.policy == WrapInside::Avoid)
            .collect();
        let unsupported = requested.iter().any(|(_, row)| row.native.is_none());
        let mut starts: Vec<_> = requested
            .iter()
            .filter_map(|(i, row)| row.native.as_ref().map(|s| (s.first, *i)))
            .collect();
        let mut ends: Vec<_> = requested
            .iter()
            .filter_map(|(i, row)| row.native.as_ref().map(|s| (s.last, *i)))
            .collect();
        starts.sort_unstable();
        ends.sort_unstable();
        let mut active = std::collections::BTreeSet::new();
        let (mut entered, mut exited) = (0, 0);
        let mut classes = Vec::with_capacity(nodes.len() + 1);
        for cut in 0..=nodes.len() {
            let rank = ranks
                .get(cut)
                .copied()
                .unwrap_or_else(|| ranks.last().map_or(0, |i| i + 1));
            while entered < starts.len() && starts[entered].0 < rank {
                let i = starts[entered].1;
                active.insert((self.ancestry.intervals[i].0, i));
                entered += 1;
            }
            while exited < ends.len() && ends[exited].0 < rank {
                let i = ends[exited].1;
                active.remove(&(self.ancestry.intervals[i].0, i));
                exited += 1;
            }
            classes.push(if unsupported {
                Err(WrappingScopeError::UnsupportedIncidence)
            } else {
                Ok(AvoidClass {
                    ancestry: Arc::clone(&self.ancestry),
                    scope: active.last().map(|&(_, i)| i),
                    edges: Vec::new(),
                })
            });
        }
        let mut forced = vec![false; nodes.len() + 1];
        for (id, edge) in self
            .edge_plan
            .locations
            .iter()
            .enumerate()
            .filter(|(_, e)| e.flow == flow)
        {
            let Some(span) = edge.native.as_ref() else {
                if edge.effect.forced {
                    return Err(WrappingScopeError::UnsupportedIncidence);
                }
                classes.fill(Err(WrappingScopeError::UnsupportedIncidence));
                continue;
            };
            // The box endpoint owns this edge, not a shared inter-run gap.
            // Qualify that actual emitted slot before projecting Before/After;
            // partitioning an outside rank would clamp it to this run's ends.
            let owner_rank = if edge.side == surgeist_text::SourceBoxSide::Before {
                span.first
            } else {
                span.last
            };
            let owner = match ranks.binary_search(&owner_rank) {
                Ok(owner) => owner,
                Err(_) => {
                    if ranks
                        .first()
                        .zip(ranks.last())
                        .is_some_and(|(first, last)| *first < owner_rank && owner_rank < *last)
                    {
                        if edge.effect.forced {
                            return Err(WrappingScopeError::UnsupportedIncidence);
                        }
                        classes.fill(Err(WrappingScopeError::UnsupportedIncidence));
                    }
                    continue;
                }
            };
            let cut = owner + usize::from(edge.side == surgeist_text::SourceBoxSide::After);
            if edge.effect.forced {
                forced[cut] = true;
            }
            if edge.effect.avoided
                && let Ok(class) = &mut classes[cut]
            {
                class.edges.push(id);
            }
        }
        let mut metrics = Vec::new();
        for (_, row) in self.box_rows(flow) {
            let Some(contribution) = row.metric_contribution.clone() else {
                continue;
            };
            let span = row
                .native
                .as_ref()
                .ok_or(WrappingScopeError::UnsupportedIncidence)?;
            if ranks
                .first()
                .zip(ranks.last())
                .is_none_or(|(first, last)| span.last < *first || *last < span.first)
            {
                continue;
            }
            let first = ranks.binary_search(&span.first).ok();
            let last = ranks.binary_search(&span.last).ok();
            if first.is_none() && ranks.first().is_some_and(|rank| *rank <= span.first)
                || last.is_none() && ranks.last().is_some_and(|rank| span.last <= *rank)
            {
                return Err(WrappingScopeError::UnsupportedIncidence);
            }
            metrics.push(NumericBoxMetrics {
                first: first.unwrap_or(0),
                after: last.map_or(nodes.len(), |index| index + 1),
                endpoints: [first, last],
                contribution,
            });
        }
        Ok(NumericWrapping {
            metrics,
            forced,
            ancestry: Arc::clone(&self.ancestry),
            classes,
        })
    }
    pub(crate) fn native_pair_projection(
        &self,
        graph: &ValidationGraph<Node>,
        flow: Node,
        nodes: &[Node],
        pair: (Node, Node),
    ) -> Result<NativePairProjection<Node>, WrappingScopeError> {
        let owner = graph
            .index(flow)
            .ok_or(WrappingScopeError::NodeNotReachable)?;
        let rank = |node| {
            graph.children[owner]
                .iter()
                .position(|&i| graph.nodes[i] == node)
                .ok_or(WrappingScopeError::DifferentFlow)
        };
        let ranks = ordered_ranks(graph, owner, nodes)?;
        if ranks.is_empty() {
            return Err(WrappingScopeError::EndpointOrder);
        }
        let project = |node| {
            let at = rank(node)?;
            if nodes.contains(&node) {
                Ok(ScopeEndpoint::Local(node))
            } else if at < ranks[0] {
                Ok(ScopeEndpoint::BeforeRun)
            } else if at > *ranks.last().ok_or(WrappingScopeError::EndpointOrder)? {
                Ok(ScopeEndpoint::AfterRun)
            } else {
                Err(WrappingScopeError::UnsupportedIncidence)
            }
        };
        Ok(NativePairProjection {
            start: project(pair.0)?,
            end: project(pair.1)?,
        })
    }
    pub(crate) fn dependencies(&self) -> Vec<Node> {
        let mut result = self.edge_plan.dependencies.clone();
        for row in self.rows.iter().filter(|row| {
            row.policy == WrapInside::Avoid
                || !row.edges.is_auto()
                || row.metric_contribution.is_some()
        }) {
            for n in [Some(row.flow), row.pair.map(|p| p.0), row.pair.map(|p| p.1)]
                .into_iter()
                .flatten()
            {
                if !result.contains(&n) {
                    result.push(n);
                }
            }
        }
        result
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NumericBoxMetrics {
    pub(crate) first: usize,
    pub(crate) after: usize,
    pub(crate) endpoints: [Option<usize>; 2],
    pub(crate) contribution: InlineBoxMetricContribution,
}
#[derive(Clone, Debug)]
pub(crate) struct NumericWrapping {
    ancestry: Arc<Ancestry>,
    classes: Vec<Result<AvoidClass, WrappingScopeError>>,
    forced: Vec<bool>,
    metrics: Vec<NumericBoxMetrics>,
}
impl PartialEq for NumericWrapping {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ancestry, &other.ancestry)
            && self.classes == other.classes
            && self.forced == other.forced
            && self.metrics == other.metrics
    }
}
impl NumericWrapping {
    pub(crate) fn has_metrics(&self) -> bool {
        !self.metrics.is_empty()
    }
    pub(crate) fn metrics(
        &self,
        start: usize,
        end: usize,
    ) -> impl Iterator<Item = &NumericBoxMetrics> {
        self.metrics.iter().filter(move |metrics| {
            start < metrics.after && metrics.first < end
                || start == end && metrics.first < start && start < metrics.after
        })
    }

    pub(crate) fn without_avoidance(mut self) -> Self {
        for class in self.classes.iter_mut().flatten() {
            class.scope = None;
            class.edges.clear();
        }
        self
    }
    pub(crate) fn forced(&self, cut: usize) -> bool {
        self.forced.get(cut).copied().unwrap_or(false)
    }
    pub(crate) fn next_forced(&self, start: usize) -> Option<usize> {
        self.forced
            .iter()
            .enumerate()
            .skip(start + 1)
            .find_map(|(i, &f)| f.then_some(i))
    }
    pub(crate) fn class(&self, cut: usize) -> Result<AvoidClass, WrappingScopeError> {
        self.classes
            .get(cut)
            .ok_or(WrappingScopeError::UnsupportedIncidence)?
            .clone()
    }
    pub(crate) fn frontier<T>(&self) -> PreferenceFrontier<T> {
        PreferenceFrontier {
            ancestry: Arc::clone(&self.ancestry),
            entries: FrontierEntries::Inside(BTreeMap::new()),
            serial: 0,
        }
    }
}
