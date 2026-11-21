//! The ordinary ancestry rule for disabled HTML form controls.

/// The ordinary parent observation for a qualified node.
#[derive(Debug)]
pub enum HtmlFieldsetParent<Node, Missing> {
    /// The qualified ordinary parent in the same tree.
    Parent(Node),
    /// This ordinary node is at the root of its tree.
    Root,
    /// The provider lacks the required parent fact.
    Missing(Missing),
}

/// Supplies the qualified ordinary tree facts needed for the Fieldset rule.
///
/// The provider admits each reached observation as it is read. A full live
/// tree can use [`core::convert::Infallible`] for missing facts.
pub trait HtmlFieldsetDisablednessReader {
    /// A node in the provider's qualified ordinary topology.
    type Node: Copy;
    /// A missing fact in a partial representation.
    type Missing;
    /// The provider's first refusal or failure.
    type Error;

    /// Read and qualify this node's ordinary parent, if present.
    fn ordinary_parent(
        &mut self,
        node: Self::Node,
    ) -> Result<HtmlFieldsetParent<Self::Node, Self::Missing>, Self::Error>;

    /// Observe whether the node is an HTML Fieldset with its disabled attribute.
    fn disabled_fieldset(&mut self, node: Self::Node) -> Result<bool, Self::Error>;

    /// Observe whether this branch is an exact HTML Legend element.
    fn html_legend(&mut self, node: Self::Node) -> Result<bool, Self::Error>;

    /// Observe whether the qualified branch is the first HTML Legend child.
    fn first_legend_branch(
        &mut self,
        fieldset: Self::Node,
        branch: Self::Node,
    ) -> Result<Result<bool, Self::Missing>, Self::Error>;
}

/// Determine whether a disabled HTML Fieldset ancestor disables this node.
///
/// An exempt first Legend branch still takes part in every outer ancestor's
/// rule. A proven disabling ancestor establishes the result even when a
/// nearer Fieldset's Legend order was unavailable.
///
/// [Pinned HTML disabledness rule](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-disabled).
pub fn disabled_by_fieldset<R: HtmlFieldsetDisablednessReader>(
    reader: &mut R,
    node: R::Node,
) -> Result<Result<bool, R::Missing>, R::Error> {
    let mut branch = node;
    let mut missing_children = None;
    loop {
        let ancestor = match reader.ordinary_parent(branch)? {
            HtmlFieldsetParent::Parent(ancestor) => ancestor,
            HtmlFieldsetParent::Root => return Ok(missing_children.map_or(Ok(false), Err)),
            HtmlFieldsetParent::Missing(missing) => return Ok(Err(missing)),
        };
        if reader.disabled_fieldset(ancestor)? {
            if !reader.html_legend(branch)? {
                return Ok(Ok(true));
            }
            match reader.first_legend_branch(ancestor, branch)? {
                Ok(true) => {}
                Ok(false) => return Ok(Ok(true)),
                Err(missing) => missing_children = Some(missing),
            }
        }
        branch = ancestor;
    }
}
