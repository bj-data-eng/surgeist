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
}
/// Contributor identities follow the actual Block collapse operations. The arena
/// retains shared groups without copying every ancestor's contributor list.
#[derive(Clone)]
pub(crate) struct State<Node> {
    groups: Vec<Group<Node>>,
    receipts: Vec<Receipt<Node>>,
}
impl<Node: Copy + Eq> State<Node> {
    pub(crate) fn new() -> Self {
        Self {
            groups: Vec::new(),
            receipts: Vec::new(),
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
        let own = self.edge(node, side);
        self.join(own, descendants)
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
        };
        if let Some(entry) = self.receipts.iter_mut().find(|entry| entry.node == node) {
            *entry = receipt;
        } else {
            self.receipts.push(receipt);
        }
    }
    pub(crate) fn boundary(&self, node: Node, start: bool, output: &mut Vec<MarginOwner<Node>>) {
        let Some(receipt) = self.receipts.iter().find(|entry| entry.node == node) else {
            return;
        };
        self.collect(if start { receipt.boundary_start } else { receipt.boundary_end }, output);
    }
    /// Descendant contributors actually reported through this box edge. A
    /// separating border, padding, or formatting boundary already removed its
    /// group when the ordinary Block producer recorded the receipt.
    pub(crate) fn reported(&self, node: Node, side: PhysicalSide, output: &mut Vec<MarginOwner<Node>>) {
        let Some(receipt) = self.receipts.iter().find(|entry| entry.node == node) else { return; };
        let group = if side == receipt.axes.block_start() { receipt.start }
            else if side == receipt.axes.block_end() { receipt.end } else { None };
        self.collect(group, output);
    }
    fn collect(&self, group: Option<usize>, output: &mut Vec<MarginOwner<Node>>) {
        let mut pending = Vec::new();
        pending.extend(group);
        while let Some(id) = pending.pop() {
            match self.groups[id] {
                Group::Edge(owner) => {
                    if !output.contains(&owner) { output.push(owner); }
                }
                Group::Join(left, right) => {
                    pending.push(left);
                    pending.push(right);
                }
            }
        }
    }
}
