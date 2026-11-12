# Named compound mutations

These [Source operations](source-and-style.md) execute the pinned DOM3071
algorithms over the [canonical tree](canonical-tree.md). Bindings supply already
converted `DomNodeOrString` operands containing live native identities or exact
UTF-16 strings. Native DOM owns the remaining algorithm, not an adapter replay
queue or a collection of independently completed calls.

## Methods and source order

`DomSourceEdit` exposes `append`, `prepend`, `replace_children`, `before`, `after`,
`replace_with`, `set_text_content` and `insert_adjacent_text`.

Variadic conversion creates every string's Text before appending any operand.
Exactly one operand is used directly; otherwise one DocumentFragment receives
the original operands in order. Duplicates therefore move their actual identity
again, and fragment operands transfer their real children. Empty variadic strings
still create Text. The zero-argument case creates an empty conversion fragment.

ChildNode methods capture the original parent and viable sibling before
conversion. A detached receiver returns before creating Text or moving nodes.
`replace_with` checks whether the receiver still has that original parent after
conversion. `prepend` reads the first child after conversion. `replace_children`
validates its replacement while excluding the original target children, then
runs the shared replace-all removal/insertion continuation. Hierarchy checking,
adoption, native hooks and Range maintenance retain their canonical owners.

`set_text_content` replaces all children of Element/DocumentFragment, including
ShadowRoot; an empty string creates no replacement Text. Text, Comment and PI use
replace-data. Document and DocumentType explicitly do nothing. Attr's separate
identity/value producer is not implemented by this NodeId front (#2274).

`insert_adjacent_text` creates Text before interpreting the four ASCII
case-insensitive position names. It preserves an empty Text and isolated UTF-16
surrogates. An invalid position produces `InvalidAdjacentPosition`; an outside
position on a detached Element leaves its tree unchanged.

## Suspension, failure and cleanup

One admitted invocation retains its converted identities, conversion cursor,
original parent/viable sibling, final action and mandatory native effects.
Actual external requests can interrupt fragment transfer before a later operand
is inserted. Dropping a published lease preserves that exact continuation;
resumption does not repeat acknowledged work. Cancellation cannot discard its
mandatory tail.

A late ordinary algorithm error can follow accepted conversion detaches. These
effects are published with `DomSourceProgress::Failed(DomSourceFailure)`, carrying
the original operation, original error and authentic terminal commit. A failure
is distinct from `Complete`. Selection of that ordinary error retains the same
invocation before any fallible cleanup, even if no external phase published yet.
Dropping and publicly resuming that lease keeps its error and accepted conversion
cursor. Final admission refusal or cancellation does not replace the first error.
Resource/service refusals remain errors on the
retained invocation; existing simple Source operations keep their rollback
contract. Inline and native Meta fronts preserve their actual paired publication
alongside the failed terminal progress.

Admission includes original operand capacity and UTF-16 backing, converted-key
storage, checkpoint overlap and temporary node peaks. Cleanup reclaims only
unpublished detached temporary Text and empty conversion fragments. Issued identities remain canonical. A
nonempty conversion fragment after failure remains observable through the moved
nodes' parent links; no garbage-collection policy is guessed.

## Separate move operation

`DomSourceEdit::move_before` retains one invocation of DOM3071's state-preserving
move. It validates the shared shadow-including root, ancestry, reference child,
and node kind before relinking the existing identity. The old position supplies
Range and NodeIterator adjustments, Slot updates, and transient registrations.
The new position supplies Range and Slot adjustments. Dedicated HTML moving
hooks run in shadow-including order, followed by the removal and insertion
records. The operation skips ordinary insertion and removal hooks.

Option moving hooks queue each reached Select update in order. Selectedcontent
moving hooks queue a single update with the captured Select and Selectedcontent
identities. The native checkpoint executes each task against current state.
Each captured identity contributes an obligation in its current Document.
Retiring either captured identity cancels that task and preserves the original
identities in the cancellation evidence.

Reached media resource selection and hosted hyperlink speculation retain their
typed refusals pending their respective producers. [Media resource selection](https://github.com/bj-data-eng/surgeist/issues/2657) · [Hyperlink speculation](https://github.com/bj-data-eng/surgeist/issues/2658)

## Source basis

The retained [DOM3071 capture](../../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md)
defines replace-all (lines716–736), convert-nodes-into-a-node (850–860),
ParentNode methods (939–963), ChildNode methods (1036–1078), textContent
(1593–1621), insert-adjacent/Text (3601–3633), and the independent move algorithm
(604–674).
