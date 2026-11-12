# Live NodeIterator cursors

The canonical DOM creates an iterator for an exact live root and a `whatToShow`
mask. Its reference starts before the root. Traversal uses ordinary tree order
and stays within the root. Both `Reject` and `Skip` continue the walk through
descendants. A node excluded by the mask never reaches the filter.

Create and retire cursors through `Dom`. Each identity belongs to one owner and
one slot generation. Detaching or adopting a root preserves the cursor.
Destroying the root retires the iterator. `DomLimits::with_node_iterators` bounds
retained slots, including vacancies, and `DomUsage::node_iterators` counts live
cursors. Next and previous operations take a finite work allowance shared by all
filter calls within that traversal.

With a callback policy, traversal returns a request carrying the iterator and
the actual node to pass to its filter. The caller may edit the same DOM while
the request is live. Respond with the filter decision or drop the request on
abrupt completion. Any already-published edits survive. The request is affine
and recursive traversal that reaches another filter check on the same iterator
refuses while it is active.

Ordinary removal adjusts the committed reference and any active candidate
before unlinking, after live Range adjustments. If the filter accepts a node
that it detached, the result still identifies the node passed to the filter.
The committed cursor takes the adjusted candidate. Creation, retirement and
changes to the committed reference appear in DOM change history. A filter request
alone does not advance the revision. Cursor and tree edits publish together
through the [Source continuation](source-and-style.md).

[Pinned DOM traversal and pre-removal steps](https://raw.githubusercontent.com/whatwg/dom/3071e5f269448350fdea79bb284eedaf17bc431e/dom.bs#L10500-L10754)
