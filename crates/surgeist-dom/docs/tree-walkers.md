# Live TreeWalker cursors

Create a TreeWalker on the canonical `Dom` with a live root, a `whatToShow` mask
and an optional callback filter. Its current node starts at the root.
`set_tree_walker_current` accepts any live node from the same owner, including a
node outside the original root. Detachment and adoption preserve the walker's
root and current identities. Destruction of either tracked node retires it.
`DomLimits::with_tree_walkers` bounds retained slots and retirement allows reuse
with a new generation.

`navigate_tree_walker` reads current ordinary relations in the requested
direction. At a `Filter` request, the host can run its callback and edit the DOM
before responding with the original converted unsigned-short result. The
continuation resumes at the reached filtering step against the live tree. A
recursive operation on the same walker that reaches filtering refuses while
the callback is active. Dropping the request clears the active flag and keeps
the effects the callback already published.

At `Match`, prepare the result before calling `accept` to publish the new
current node. A dropped match leaves the current node unchanged. Acceptance
requires the original live Source scope and exact canonical publication.
Each reached traversal step uses the same caller admission and cancellation
across suspension. DOM composes native Source admission separately.

[Pinned TreeWalker algorithms](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#interface-treewalker)
