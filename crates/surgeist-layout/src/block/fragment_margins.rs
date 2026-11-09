use crate::text_layout::scope_alignment::{EntryBacking, EntryFunding};
use crate::{FlowAxes, LayoutScalar, PhysicalSide};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MarginOwner<Node> {
    pub(crate) node: Node,
    pub(crate) side: PhysicalSide,
}
#[derive(Clone)]
enum Group<Node> {
    Edge(MarginOwner<Node>),
    Join(usize, usize),
    Projection(crate::engine::block_scope::ProductionId, MarginOwner<Node>),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Contributor<Node> {
    Edge(MarginOwner<Node>),
    Projection(crate::engine::block_scope::ProductionId, MarginOwner<Node>),
}
#[derive(Clone, Copy, Default)]
pub(super) struct Flow {
    pub(super) top: Option<usize>,
    pub(super) active: Option<usize>,
    pub(super) initial: Option<usize>,
    pub(super) started: bool,
}
#[derive(Clone)]
struct Receipt<Node> {
    node: Node,
    axes: FlowAxes,
    start: Option<usize>,
    end: Option<usize>,
    boundary_start: Option<usize>,
    boundary_end: Option<usize>,
    entry: Option<usize>,
}
/// Contributor identities follow the actual Block collapse operations. The arena
/// retains shared groups without copying every ancestor's contributor list.
pub(crate) struct State<Node, S: LayoutScalar> {
    groups: Vec<Group<Node>>,
    receipts: Vec<Receipt<Node>>,
    projections: Vec<(Node, crate::engine::block_scope::ProductionId)>,
    group_backing: EntryBacking,
    receipt_backing: EntryBacking,
    projection_backing: EntryBacking,
    funding: Option<EntryFunding<S>>,
}

/// A query's contributor rows retain their backing until the consumer drops them.
pub(crate) struct MarginContributors<Node> {
    rows: Vec<Contributor<Node>>,
    backing: EntryBacking,
}
impl<Node> std::ops::Deref for MarginContributors<Node> {
    type Target = [Contributor<Node>];
    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}
pub(crate) struct ContributorIter<Node> {
    rows: std::vec::IntoIter<Contributor<Node>>,
    _backing: EntryBacking,
}
impl<Node> Iterator for ContributorIter<Node> {
    type Item = Contributor<Node>;
    fn next(&mut self) -> Option<Self::Item> {
        self.rows.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rows.size_hint()
    }
}
impl<Node> IntoIterator for MarginContributors<Node> {
    type Item = Contributor<Node>;
    type IntoIter = ContributorIter<Node>;
    fn into_iter(self) -> Self::IntoIter {
        ContributorIter {
            rows: self.rows.into_iter(),
            _backing: self.backing,
        }
    }
}

fn copy_rows<T: Clone, S: LayoutScalar>(
    source: &[T],
    funding: Option<&EntryFunding<S>>,
) -> surgeist_text::Result<(Vec<T>, EntryBacking)> {
    let Some(funding) = funding else {
        return Ok((source.to_vec(), EntryBacking::default()));
    };
    if source.is_empty() {
        return Ok((Vec::new(), EntryBacking::default()));
    }
    let (lease, mut values) = funding
        .buffer(source.len())
        .map(|(rows, lease)| (lease, rows))?;
    for row in source {
        funding.visit(1)?;
        values.push(row.clone());
    }
    EntryBacking::from_buffer(values, source.len(), lease, funding)
}

impl<Node: Copy + Eq, S: LayoutScalar> State<Node, S> {
    pub(crate) fn new() -> Self {
        Self {
            groups: Vec::new(),
            receipts: Vec::new(),
            projections: Vec::new(),
            group_backing: EntryBacking::default(),
            receipt_backing: EntryBacking::default(),
            projection_backing: EntryBacking::default(),
            funding: None,
        }
    }
    fn visit(&self, count: usize) -> surgeist_text::Result<()> {
        if let Some(funding) = &self.funding {
            funding.visit(count)?;
        }
        Ok(())
    }
    fn replace(&self) -> surgeist_text::Result<()> {
        if let Some(funding) = &self.funding {
            funding.materialize(1)?;
        }
        Ok(())
    }
    pub(crate) fn fork_entry(
        &self,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Self> {
        if let Some(funding) = funding {
            funding.visit(1)?;
        }
        let mut state = Self::new();
        state.funding = funding.cloned();
        (state.groups, state.group_backing) = copy_rows(&self.groups, funding)?;
        (state.receipts, state.receipt_backing) = copy_rows(&self.receipts, funding)?;
        (state.projections, state.projection_backing) = copy_rows(&self.projections, funding)?;
        Ok(state)
    }
    fn edge(&mut self, node: Node, side: PhysicalSide) -> surgeist_text::Result<Option<usize>> {
        let id = self.groups.len();
        self.group_backing
            .reserve_push(&mut self.groups, self.funding.as_ref())?;
        self.groups.push(Group::Edge(MarginOwner { node, side }));
        Ok(Some(id))
    }
    pub(super) fn join(
        &mut self,
        left: Option<usize>,
        right: Option<usize>,
    ) -> surgeist_text::Result<Option<usize>> {
        match (left, right) {
            (Some(left), Some(right)) => {
                let id = self.groups.len();
                self.group_backing
                    .reserve_push(&mut self.groups, self.funding.as_ref())?;
                self.groups.push(Group::Join(left, right));
                Ok(Some(id))
            }
            (left, right) => Ok(left.or(right)),
        }
    }
    pub(super) fn child(
        &mut self,
        node: Node,
        side: PhysicalSide,
    ) -> surgeist_text::Result<Option<usize>> {
        let mut descendants = None;
        for entry in self.receipts.iter().rev() {
            self.visit(1)?;
            if entry.node == node {
                descendants = if side == entry.axes.block_start() {
                    entry.start
                } else if side == entry.axes.block_end() {
                    entry.end
                } else {
                    None
                };
                break;
            }
        }
        if descendants.is_none() {
            let mut projection = None;
            for (key, value) in &self.projections {
                self.visit(1)?;
                if *key == node {
                    projection = Some(*value);
                    break;
                }
            }
            if let Some(projection) = projection {
                let id = self.groups.len();
                self.group_backing
                    .reserve_push(&mut self.groups, self.funding.as_ref())?;
                self.groups
                    .push(Group::Projection(projection, MarginOwner { node, side }));
                descendants = Some(id);
            }
        }
        let own = self.edge(node, side)?;
        self.join(own, descendants)
    }
    pub(crate) fn bind(
        &mut self,
        node: Node,
        projection: crate::engine::block_scope::ProductionId,
    ) -> surgeist_text::Result<()> {
        for index in 0..self.projections.len() {
            self.visit(1)?;
            if self.projections[index].0 == node {
                self.replace()?;
                self.projections[index].1 = projection;
                return Ok(());
            }
        }
        self.projection_backing
            .reserve_push(&mut self.projections, self.funding.as_ref())?;
        self.projections.push((node, projection));
        Ok(())
    }
    pub(super) fn record(
        &mut self,
        node: Node,
        axes: FlowAxes,
        flow: Flow,
        collapse_start: bool,
        collapse_end: bool,
    ) -> surgeist_text::Result<()> {
        let mut receipt = Receipt {
            node,
            axes,
            start: collapse_start.then_some(flow.top).flatten(),
            end: collapse_end.then_some(flow.active).flatten(),
            boundary_start: flow.initial,
            boundary_end: flow.active,
            entry: None,
        };
        for index in 0..self.receipts.len() {
            self.visit(1)?;
            if self.receipts[index].node == node {
                self.replace()?;
                receipt.entry = self.receipts[index].entry;
                self.receipts[index] = receipt;
                return Ok(());
            }
        }
        self.receipt_backing
            .reserve_push(&mut self.receipts, self.funding.as_ref())?;
        self.receipts.push(receipt);
        Ok(())
    }
    /// The contributor group actually consumed by this child's start reducer.
    /// It shares the same Join arena as the reported boundary groups.
    pub(super) fn record_entry(
        &mut self,
        node: Node,
        axes: FlowAxes,
        group: Option<usize>,
    ) -> surgeist_text::Result<()> {
        for index in 0..self.receipts.len() {
            self.visit(1)?;
            if self.receipts[index].node == node {
                self.replace()?;
                self.receipts[index].entry = group;
                return Ok(());
            }
        }
        self.receipt_backing
            .reserve_push(&mut self.receipts, self.funding.as_ref())?;
        self.receipts.push(Receipt {
            node,
            axes,
            start: None,
            end: None,
            boundary_start: None,
            boundary_end: None,
            entry: group,
        });
        Ok(())
    }
    fn receipt(&self, node: Node) -> surgeist_text::Result<Option<&Receipt<Node>>> {
        for receipt in &self.receipts {
            self.visit(1)?;
            if receipt.node == node {
                return Ok(Some(receipt));
            }
        }
        Ok(None)
    }
    pub(crate) fn entry(&self, node: Node) -> surgeist_text::Result<MarginContributors<Node>> {
        self.collect(self.receipt(node)?.and_then(|receipt| receipt.entry))
    }
    pub(crate) fn boundary(
        &self,
        node: Node,
        start: bool,
    ) -> surgeist_text::Result<MarginContributors<Node>> {
        self.collect(self.receipt(node)?.and_then(|receipt| {
            if start {
                receipt.boundary_start
            } else {
                receipt.boundary_end
            }
        }))
    }
    /// Descendant contributors actually reported through this box edge. A
    /// separating border, padding, or formatting boundary already removed its
    /// group when the ordinary Block producer recorded the receipt.
    pub(crate) fn reported(
        &self,
        node: Node,
        side: PhysicalSide,
    ) -> surgeist_text::Result<MarginContributors<Node>> {
        let group = self.receipt(node)?.and_then(|receipt| {
            if side == receipt.axes.block_start() {
                receipt.start
            } else if side == receipt.axes.block_end() {
                receipt.end
            } else {
                None
            }
        });
        self.collect(group)
    }
    fn collect(&self, group: Option<usize>) -> surgeist_text::Result<MarginContributors<Node>> {
        let mut result = MarginContributors {
            rows: Vec::new(),
            backing: EntryBacking::default(),
        };
        let mut pending_backing = EntryBacking::default();
        let mut pending = Vec::new();
        if let Some(group) = group {
            pending_backing.reserve_push(&mut pending, self.funding.as_ref())?;
            pending.push(group);
        }
        while let Some(id) = pending.pop() {
            self.visit(1)?;
            match self.groups[id] {
                Group::Edge(owner) => {
                    result
                        .backing
                        .reserve_push(&mut result.rows, self.funding.as_ref())?;
                    result.rows.push(Contributor::Edge(owner));
                }
                Group::Projection(id, owner) => {
                    result
                        .backing
                        .reserve_push(&mut result.rows, self.funding.as_ref())?;
                    result.rows.push(Contributor::Projection(id, owner));
                }
                Group::Join(left, right) => {
                    pending_backing.reserve_push(&mut pending, self.funding.as_ref())?;
                    pending.push(left);
                    pending_backing.reserve_push(&mut pending, self.funding.as_ref())?;
                    pending.push(right);
                }
            }
        }
        drop(pending);
        drop(pending_backing);
        Ok(result)
    }
}
