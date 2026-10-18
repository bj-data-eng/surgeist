# Structural View Transitions

This capability stores admitted transition identities, generated pseudo relations and CSSOM associations. Runtime owns the rendering resources and composed execution described at the end of this page.

## Initialization and participant identities

Canonical View Transitions state is an explicitly initialized structural
capability in the same atomic owner. Configure finite transition, participant
record and generated-pseudo capacities with `DomLimits::with_view_transitions`;
these capacities start at zero. Participant names count toward stored UTF-16
units. Hidden, inactive records and pseudos remain admitted until explicit
release; retaining caller snapshots does not pin live nodes or consume live
capacity. Node, transition and record creation tickets are separate and resolve
only after publication. Transition/record generations are checked and never
reissued.

Every feature edit requires an exact already-published `DomDocumentStamp` for
its pre-batch basis. Native document creation leaves this feature uninitialized.
`initialize_view_transitions` admits an actual captured CSSOM sheet and establishes
source defaults: absent active transition, rendering suppression false and
show-tree false. An initialized binding always contains a real sheet identity;
uninitialized is not the source's default null stylesheet. This structural
association does not establish the source-required fresh UA stylesheet's actual
cascade-origin placement or document installation.

## Generated relation and CSSOM associations

A transition owns an ordered map of case-sensitive checked participant names,
with pending, old-only, new-only or paired structural branches. An old branch
certifies no image. A newly assigned new-element reference must resolve to an
actual same-document Element, and one element cannot occupy two names in the
same transition. Pending records cannot be set up. Setup is one-shot and creates
the separate generated relation: root, groups in record order, one image pair
per group, then old before new when present. This never changes ordinary node
children. Actual Elements and all owned VT pseudos have independent captured
flags, initially false, through a typed subject domain. Other CSS pseudo families
are not admitted; the selected source's broader CSS-element capture requirement
remains outside this family.

Complete pseudo relation reads use this snapshot's actual document element as
the mandatory ultimate origin. Replacing it changes current origin readback;
removing it produces `ViewTransitionOriginUnavailable` without rejecting the
ordinary tree edit. Identity/captured-flag reads remain available for cleanup.
Old snapshots keep their old actual origins. Each pseudo's originating pseudo
is its parent in the generated tree.

Rule associations retain real CSSOM identities qualified by the snapshot
revision used to admit them. Ordinary slots require actual directly installed
Style rules in the bound sheet; the keyframes slot requires Keyframes. The
image-animation group contains zero, one old/new rule, or two distinct ordinary
rules ordered old then new. DOM validates kinds, membership, owner and known
revision order; it does not validate rule content or install/remove the rules.
An old retained association is historical evidence, not proof of current CSSOM
state. Clearing association records performs no CSSOM mutation.

## Cleanup, lifetime and composition boundary

`clear_view_transition_state` follows retained new-element identities and clears
still-live captured flags, active state and show-tree atomically. It preserves
suppression, records, generated identities and rule associations. Reset
suppression separately; release an inactive transition separately. Repeated
cleanup is a no-op; cleanup of an old transition rejects if a newer transition
is active, before touching its potentially reused participants.

Detaching or adopting a participant preserves its full `NodeId`, including when
it moves to another Document. Clear follows that identity across documents.
Explicit native destruction retains the existing detached-root/subtree rules
and is not blocked by active or inactive records. The record retains its exact
historical identity; `resolve_new_element` distinguishes never assigned
`Ok(None)` from destroyed `Err(StaleNode)`. Clear skips destroyed generations
and never follows a reused slot. This is the selected native-destruction policy,
not a browser object-lifetime claim. Runtime must requalify current participation
after canonical revisions.

Runtime owns actual
images, geometry/style snapshots, phase timing and the exact resource join to
these DOM transition/record identities and revisions. DOM stores no image or
resource handle and certifies no usable composed transition, rendering, pointer
hit testing, accessibility behavior or CSSOM effect. The selected
[View Transitions capture](../../../references/css-view-transitions-1--CRD-css-view-transitions-1-20240328--7f8e537956b6.md)
defines the structural source basis. Complete composed execution is not admitted
by this DOM state alone.

[Back to surgeist-dom](../README.md).
