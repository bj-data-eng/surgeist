# Forms, controls and Select

This page owns Form association, native Fieldset/Button state, Select updates and weak reflected targets. Value-specific contracts belong to [Input](input.md) and [Output/Textarea](text-controls.md).

## Form association and custom validity

Native HTML Fieldset retains its actual form-owner relation, native unset parser
flag and initially empty custom-validity message. `fieldset_state()` reads the
intrinsic parser/message state; `DomNode::as_fieldset()` provides the checked
`DomFieldset::form_owner()` relation at the captured revision. Exact form/ID attributes, associated subtree insertion/removal,
fragment transfer, replacement and adoption drive HTML's shared form-owner reset
cursor against the canonical staged tree. Ordinary form lookup does not cross a
shadow host; connectedness uses the actual shadow-including root. First-ID lookup
observes the first exact nonempty DOM ID, including a non-Form blocker. A present
empty form attribute is distinct from absence. ID-bearing removal uses the actual
post-removal non-Document-tree predicate, without an old-connected prerequisite.
Native HTML Form retains HTML's three fresh false scalar values and an actual
past names map. `DomNode::as_form()` reads its scalars and retained exact-name
aliases. Native creation/clone starts the map empty. The current public fronts
cannot insert past-name history or perform named-property lookup; map cleanup
handles actual retained aliases without treating the map as permanently empty.
Every owner change removes all aliases for the control from its former Form at
the intermediate owner state, before later lookup or reassociation. Thus even an
A→null→A reset can change the Form map. Ordinary nearest-owner retention and
unrelated name edits preserve aliases. Retained name units count independently
of attributes; the public API has no growing-history front.

Form owners live once in the canonical Node/Draft relation, using real NodeIds
and staged Keys respectively. Same-batch/private imported Forms resolve through
the existing final allocation map. `DomButton::form_owner()` and
`DomFieldset::form_owner()` read that relation; scalar state values carry no
identity mirror. Native Form clone/import starts fresh scalars/map, while deep
copied controls acquire their new actual Form. Adoption preserves retained state
and runs actual split-tree resets. Explicit retirement clears surviving owners
and aliases for exact retired identities without following generation reuse.
Snapshots preserve the old relations/maps and ElementState covers their changes.

`set_custom_validity` applies to native Fieldset and Button. It uses HTML's unit-fed
CRLF/CR-to-LF rule before replacing the stored message, preserving every other
exact UTF-16 unit. Raw batch operands are caller-owned; normalized output scratch
is bounded by the admitted retained length, and the final normalized message
counts independently of attributes. Every raw-unit read and output write polls
cancellation. State-only edits report ElementState, equal final messages are
semantic no-ops, and snapshots/history retain their original values. Native
clone/import initializes a fresh empty message/null association and executes
copied-attribute hooks; adoption preserves the message and runs actual removal
and insertion/reset effects. The Fieldset payload allocates no control value,
selection, cached disabledness or fake form object.

Semantic capture publishes actual established-current Fieldset and Button associations,
closing actual Form owners and retained map targets. Both initial capture and
Slot supplementation establish negative custom-control classifications only from
actual HTML elements with an actual null definition. The finite subject budget
bounds these records; adapter work admits their context validation and duplicate
comparison envelope. No custom spelling or registry emptiness stands in for a
definition fact.
Fieldset collections, Legend delegation, first-Legend disabledness and constraint
candidacy remain HTML semantic queries over the current captured tree. Slot
supplementation rebuilds actual control facts into that same context, preserving
Slot directionality/inert facts. `DomHtmlCaptureLimits` bounds each ordinary adapter vector
by subjects (nullable reference facts use twice subjects) and adapter work by work; the same explicit finite work value
independently caps shared control validation including its context join. Checked
move-only facts are rebuilt from the retained snapshot/map, never cloned or
replaced with inferred ownership. This native control adapter does not execute constraint
validation, report/check validity, expose a complete ValidityState schema or
provide validation UI/events. A nonempty Fieldset custom error does not make it
a constraint-validation candidate.

Native HTML Button composes that same private native-control storage and reset
adapter with its own public `button_state()` / `DomNode::as_button()` view.
`set_custom_validity` also applies to Button. Actual form association, native
parser flag and custom message obey the same fresh clone/import and preserving
adoption rules. Disabledness, Auto submit role, source form settings and constraint
candidacy are current HTML semantic queries; no activation, submission or event
execution is introduced.

## Select, Option, Optgroup and Selectedcontent

Native Select, Option, Optgroup and Selectedcontent compose the same canonical
owner. Select retains shared native control/message and user-validity flags;
Option retains selectedness/dirtiness and its real cached nearest Select identity.
Optgroup disabled changes consume the same bounded nearest-Select ancestry rule,
including HTML barriers. Selectedcontent retains actual insertion-derived disabledness.
`set_option_selected`, `set_select_value`, `set_select_selected_index` and
`reset_select` execute the shared HTML decisions against the current staged tree.
Value/index setters preserve the first exact match and intentionally leave no
selection after no match. Both ordinary and Slot-supplemented semantic captures
retain actual Option flags and canonical Select form association.

Selectedcontent copies actual Option children, with fresh canonical identities,
through the shared iterative clone executor and real family/Template/Shadow hooks.
Each child finishes its deep clone before append to an actual temporary Fragment;
replace-all detaches old children without retiring them. The proven private empty
wrapper is reclaimed before publication. Source insertion, attribute, removal and
static postconnection order remains observable; updates run no host callbacks.

`DomLimits::with_select_processing` explicitly bounds pending Select updates,
DOM adapter/effect scratch bytes and cumulative DOM work. Unconfigured variable
work has zero capacity; native constructors need no variable work. Reached value
setters additionally require `with_select_value_queries(HtmlCaptureLimits,
HtmlSemanticLimits)`. One sequential producer capture has its own finite retention
limits. Producer semantic work and allocated bytes are cumulative over every
Option query in a private batch; other semantic limits apply per query. These
producer domains remain separate from DOM temporary/effect banks. The adapter
passes the caller's cancellation token through the original capture and query
algorithms, and preserves producer usage on both success and refusal. During a
composed Source edit, its remaining live-byte, UTF-16 and work allowance also
covers the synchronous producer graph, interleaved DOM mapping banks and the
returned value occurrence. The producer graph remains admitted until the
returned value has its own storage admission; resuming preserves spent work.

Real Option removal queues duplicate-preserving UpdateSelect intentions in the
same FIFO as one coalesced Slot marker. Checkpoint handoff leaves the Select head
pending and returns an immutable `DomSelectTask` with shared once-only authority.
`complete_select_task` executes current state under `DomNativeCheckpointLimits`;
failure preserves the head and permission, and dropping all handles permits
reacquisition. Foreign owners and replay reject. Exact Select retirement runs
cleanup first, cancels every captured occurrence with TargetDestroyed evidence,
and allows one historical SkippedTargetDestroyed completion of an issued handle.
`begin_select_completion` reports that same canonical skip through
`DomSelectCompletionStart::Skipped` before admitting live-only Source work.
A live target returns `Source` with its original lease; resuming uses that actual
Source operation. Final publication refusal preserves the captured queue head
and permission so raising the caller's limits can complete it exactly once.
Detach/adopt preserve intentions. No browser event loop, picker or rendering
resources are implied by these intrinsic checkpoints.

## Reflected Element targets

`set_command_for_element` and `set_popover_target_element` drive HTML's actual
nullable Element setter cursor. The two weak slots are independent and initially
null. A non-null write first sets its exact content attribute empty (including its
clearing hook), then installs the supplied Element; a null write clears the slot
before deleting content. Exact null-namespace commandfor/popovertarget changes,
including redundant sets, clear only their corresponding slot. Missing removal
runs no new attribute hook. Namespace/case mismatches remain distinct attributes.
Targets must be actual same-store Elements, and can be same-batch creations,
detached, self, other-document or outside getter scope. Staged references use
actual draft keys and map to final generation-safe identities at publication.
Two fixed slots per Button are covered by managed-node capacity, not dynamic Slot
assignment-reference capacity. No target IDs, attribute data or history records
are invented from ticket indices.

`DomButton::explicit_command_for` / `explicit_popover_target` read raw stored weak
references. Their scoped associated-element getters remain existing HTML semantic
queries, with first-ID content fallback only when the actual explicit slot is
null. An out-of-scope explicit target is retained and queries null without content
retargeting. Capture closes actual weak targets, node documents and ancestors
conservatively, without claiming unrelated supplemental children are empty or
complete, and joins existing checked reference/control facts. Each reference-fact
vector is bounded by twice the explicit subject bound (two per Button, one per
Input), with
finite producer validation work. Slot supplementation rebuilds these actual facts
and preserves its own directionality/inert facts. Native clone/import starts both
slots null and processes copied attributes; adoption preserves actual weak IDs.
Explicit destruction clears surviving references to the exact retired identity,
including same-batch targets, and never follows generation reuse. Old snapshots
retain their actual historical reference and target. State-only changes report
ElementState alongside separate attribute changes; failed/cancelled batches roll
back both domains.

[Back to surgeist-dom](../README.md).
