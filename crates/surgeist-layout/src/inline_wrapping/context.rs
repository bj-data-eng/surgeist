use super::*;
use crate::{InlineBoundaryKind, LayoutTree, engine::validation::ValidationGraph};

/// Canonical metadata; only requested Avoid scopes activate their full region.
#[derive(Clone, Debug)]
pub(crate) struct CanonicalScopeProof<Node> {
    pub(crate) scope: CapturedScope,
    pub(crate) flow: Node,
    pub(crate) pair: Option<(Node, Node)>,
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
struct SourceExtent {
    domain: CapturedScope,
    start: surgeist_text::CapturedEventPosition,
    end: surgeist_text::CapturedEventPosition,
}
impl SourceExtent {
    fn canonical(scope: &CapturedScope) -> Self {
        Self {
            domain: scope.domain(),
            start: scope.enter_position(),
            end: scope.exit_position(),
        }
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
        Ok(Self {
            domain,
            start: first,
            end: last,
        })
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
    native: Option<NativeSpan>,
    policy: WrapInside,
    source: Option<SourceExtent>,
}
#[derive(Clone, Debug)]
pub(crate) struct WrappingContext<Node> {
    rows: Vec<ScopeRow<Node>>,
    ancestry: Arc<Ancestry>,
    has_avoid: bool,
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
        }
    }
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
enum Relation {
    Before,
    After,
    Contains,
    Within,
    Crossing,
}
fn interval_relation<T: PartialOrd>(a: &T, b: &T, c: &T, d: &T) -> Relation {
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
        (a.domain == b.domain && a.start.same_capture(&b.start))
            .then(|| interval_relation(&a.start, &a.end, &b.start, &b.end))
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
    let a = graph
        .index(start)
        .ok_or(WrappingScopeError::NodeNotReachable)?;
    let b = graph
        .index(end)
        .ok_or(WrappingScopeError::NodeNotReachable)?;
    if counts[a] != 1 || counts[b] != 1 {
        return Err(WrappingScopeError::AmbiguousNode);
    }
    if tree
        .layout_input(start)
        .as_inline_boundary()
        .is_none_or(|v| v.kind() != InlineBoundaryKind::Start)
        || tree
            .layout_input(end)
            .as_inline_boundary()
            .is_none_or(|v| v.kind() != InlineBoundaryKind::End)
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
    ) -> Result<Self, WrappingScopeError> {
        if inputs.is_empty() {
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
                WrappingScopeInput::Captured { scope, wrap_inside } => {
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
                if rows[i].policy != row.policy {
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
        if rows.iter().all(|row| row.policy == WrapInside::Auto) {
            return Ok(Self::default());
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
        Ok(Self {
            rows,
            ancestry: Arc::new(Ancestry { intervals }),
            has_avoid,
        })
    }
    pub(crate) fn has_avoid(&self) -> bool {
        self.has_avoid
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
                })
            });
        }
        Ok(NumericWrapping {
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
        let mut result = Vec::new();
        for row in self
            .rows
            .iter()
            .filter(|row| row.policy == WrapInside::Avoid)
        {
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
#[derive(Clone, Debug)]
pub(crate) struct NumericWrapping {
    ancestry: Arc<Ancestry>,
    classes: Vec<Result<AvoidClass, WrappingScopeError>>,
}
impl PartialEq for NumericWrapping {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ancestry, &other.ancestry) && self.classes == other.classes
    }
}
impl NumericWrapping {
    pub(crate) fn class(&self, cut: usize) -> Result<AvoidClass, WrappingScopeError> {
        self.classes
            .get(cut)
            .ok_or(WrappingScopeError::UnsupportedIncidence)?
            .clone()
    }
    pub(crate) fn frontier<T>(&self) -> PreferenceFrontier<T> {
        PreferenceFrontier {
            ancestry: Arc::clone(&self.ancestry),
            entries: BTreeMap::new(),
            serial: 0,
        }
    }
}
