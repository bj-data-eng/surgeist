use crate::{FlowAxes, PhysicalSide};

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
#[derive(Clone)]
pub(crate) struct State<Node> {
    groups: Vec<Group<Node>>,
    receipts: Vec<Receipt<Node>>,
    projections: Vec<(Node, crate::engine::block_scope::ProductionId)>,
}
impl<Node: Copy + Eq> State<Node> {
    pub(crate) fn new() -> Self {
        Self {
            groups: Vec::new(),
            receipts: Vec::new(),
            projections: Vec::new(),
        }
    }
    fn edge(&mut self, node: Node, side: PhysicalSide) -> Option<usize> {
        let id = self.groups.len();
        self.groups.push(Group::Edge(MarginOwner { node, side }));
        Some(id)
    }
    pub(super) fn join(&mut self, left: Option<usize>, right: Option<usize>) -> Option<usize> {
        match (left, right) {
            (Some(left), Some(right)) => {
                let id = self.groups.len();
                self.groups.push(Group::Join(left, right));
                Some(id)
            }
            (left, right) => left.or(right),
        }
    }
    pub(super) fn child(&mut self, node: Node, side: PhysicalSide) -> Option<usize> {
        let descendants = self
            .receipts
            .iter()
            .rev()
            .find(|entry| entry.node == node)
            .and_then(|entry| {
                if side == entry.axes.block_start() {
                    entry.start
                } else if side == entry.axes.block_end() {
                    entry.end
                } else {
                    None
                }
            });
        let descendants = descendants.or_else(|| {
            let projection = self.projections.iter().find(|(key, _)| *key == node)?.1;
            let id = self.groups.len();
            self.groups
                .push(Group::Projection(projection, MarginOwner { node, side }));
            Some(id)
        });
        let own = self.edge(node, side);
        self.join(own, descendants)
    }
    pub(crate) fn bind(
        &mut self,
        node: Node,
        projection: crate::engine::block_scope::ProductionId,
    ) {
        if let Some((_, old)) = self.projections.iter_mut().find(|(key, _)| *key == node) {
            *old = projection;
        } else {
            self.projections.push((node, projection));
        }
    }
    pub(super) fn record(
        &mut self,
        node: Node,
        axes: FlowAxes,
        flow: Flow,
        collapse_start: bool,
        collapse_end: bool,
    ) {
        let receipt = Receipt {
            node,
            axes,
            start: collapse_start.then_some(flow.top).flatten(),
            end: collapse_end.then_some(flow.active).flatten(),
            boundary_start: flow.initial,
            boundary_end: flow.active,
            entry: None,
        };
        if let Some(entry) = self.receipts.iter_mut().find(|entry| entry.node == node) {
            let placement_entry = entry.entry;
            *entry = receipt;
            entry.entry = placement_entry;
        } else {
            self.receipts.push(receipt);
        }
    }
    /// The contributor group actually consumed by this child's start reducer.
    /// It shares the same Join arena as the reported boundary groups.
    pub(super) fn record_entry(&mut self, node: Node, axes: FlowAxes, group: Option<usize>) {
        if let Some(receipt) = self.receipts.iter_mut().find(|entry| entry.node == node) {
            receipt.entry = group;
        } else {
            self.receipts.push(Receipt {
                node,
                axes,
                start: None,
                end: None,
                boundary_start: None,
                boundary_end: None,
                entry: group,
            });
        }
    }
    pub(crate) fn entry(&self, node: Node, output: &mut Vec<Contributor<Node>>) {
        if let Some(receipt) = self.receipts.iter().find(|entry| entry.node == node) {
            self.collect(receipt.entry, output);
        }
    }
    pub(crate) fn boundary(&self, node: Node, start: bool, output: &mut Vec<Contributor<Node>>) {
        let Some(receipt) = self.receipts.iter().find(|entry| entry.node == node) else {
            return;
        };
        self.collect(
            if start {
                receipt.boundary_start
            } else {
                receipt.boundary_end
            },
            output,
        );
    }
    /// Descendant contributors actually reported through this box edge. A
    /// separating border, padding, or formatting boundary already removed its
    /// group when the ordinary Block producer recorded the receipt.
    pub(crate) fn reported(
        &self,
        node: Node,
        side: PhysicalSide,
        output: &mut Vec<Contributor<Node>>,
    ) {
        let Some(receipt) = self.receipts.iter().find(|entry| entry.node == node) else {
            return;
        };
        let group = if side == receipt.axes.block_start() {
            receipt.start
        } else if side == receipt.axes.block_end() {
            receipt.end
        } else {
            None
        };
        self.collect(group, output);
    }
    fn collect(&self, group: Option<usize>, output: &mut Vec<Contributor<Node>>) {
        let mut pending = Vec::new();
        pending.extend(group);
        while let Some(id) = pending.pop() {
            match self.groups[id] {
                Group::Edge(owner) => {
                    output.push(Contributor::Edge(owner));
                }
                Group::Projection(id, owner) => output.push(Contributor::Projection(id, owner)),
                Group::Join(left, right) => {
                    pending.push(left);
                    pending.push(right);
                }
            }
        }
    }
}
