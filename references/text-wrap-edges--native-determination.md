# Native box-edge wrapping

This determination selects the box-edge semantics consumed by #65/#72.
Crate documentation and source establish current implementation support.

## Applicability and propagation

Pinned [Text 4 §5.3](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#wrap-before)
defines noninherited `wrap-before`/`wrap-after` values on inline-level boxes and
flex items. `avoid` applies to both kinds of wrapping; `avoid-line` and
`avoid-flex` apply to their respective kind. `line` forces the applicable inline
line break. `flex` forces a flex-line break only for an actual flex item in a
multiline flex container. Applicability follows the box's actual role.

The imported [Break 3 propagation rule](css-break-3-layout-decisions--editor-capture-20261010--debb8c3e11e2.md#break-propagation)
uses the first/last **in-flow child box** and preserves computed declarations.
Layout owns effective propagation through the actual inline chain and its
formatting-context boundary. Each consumed step requires genuine child/parent
box identity, role, in-flow status, relevant child order and prefix/suffix
completeness. Before and after consume their respective side's evidence.

A zero-decoration, removed-only or explicitly inventoried empty inline box
remains a box. Out-of-flow children are excluded. An atomic child's own exterior
edge can participate; its internal formatting context is opaque to propagation.
Explicit native pairs identify their complete inline box, separately from its
endpoint nodes. Source-scope ancestry supports propagation when its validated
association also proves these generated-box facts. Missing box incidence or
ordering produces a precise refusal for the requested step.

## Independent avoidance constraints

[Text 4 §5.2](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#wrap-inside)
orders nested inside-avoidance, while §5.3 discourages the specified edge.
Neither clause supplies a universal edge-versus-inside priority.

The native completion retains independent constraints identified by actual
box/edge occurrences. Prefer a fitting candidate whose violated constraints are
a strict subset of another's. Preserve nested-inside ancestry preference where
edge constraints introduce no opposing preference. For incomparable mixed
constraints, retain the fitter's ordinary candidate preference. Select from the
nondominated candidates; declaration count and arbitrary weights supply no
priority. This mixed-conflict rule is explicit Surgeist policy.

Compare within the same admitted search stage and actual finalized fit context.
Avoidance preserves ordinary opportunities and their intrinsic effects. It cannot
create a cut, defeat mandatory completion or establish insufficiency merely by
hiding a fitting avoided point. Existing conditional and emergency stages retain
their own prerequisites.

## Early mandatory-boundary semantics

Layout owns box applicability, effective declarations and propagation. Text
owns authenticated mandatory-boundary causes, processing barriers, Unicode and
transformation analysis, shaping and issued forced-break capabilities. Numeric
and flex selection consume the same Layout-owned effective plan directly.

A computed inline force enters the existing `ContextualSourceInput` operation
through a distinct semantic boundary request. Its genuine input-local scope,
endpoint occurrence or supported atomic edge is validated during contextual
admission before processing executes once. Text retains computed-cause custody
separately from authored controls and original characters. Late promotion of a
prepared soft opportunity cannot supply these earlier semantics.

The selected native phase completion establishes the hard boundary before
transforms, typographic-unit formation and shaping. A base and combining mark
on opposite sides form separate hard-bounded segments; a contraction cannot
execute across that boundary. Preserve the original source contributions and
ordinary standalone-segment handling. This cross-unit treatment is bounded
Surgeist policy completing the immediate-force requirement.

Reject a late attempt to split an already prepared unit or executed contraction.
Reject an input edge inside an indivisible decoded scalar or document delimiter.
The effective force neither snaps to another cut nor divides a provider glyph.
Computed causes insert no source character; plaintext remains governed by its
existing source and whitespace contract.

## Phase and identity custody

The early producer supplies real input-local source associations and a borrowed
preparation-independent view of the host box inventory. Text admission remains
the sole owner of canonical source capture. Validate the resulting source,
effective targets and computed causes before issuing a resolved Layout plan.
Normal preparation then supplies actual issued participation.

Final Layout admission revalidates every influencing box role, declaration,
association, parent/order/context and consumed propagation proof against the
current normalized tree. It then binds the actual returned capture, causes and
prepared paragraph identities. A receipt alone cannot authenticate semantic
changes across preparation. Internal admission factoring preserves the existing
processing lifecycle; an independently reusable public capture stage requires
a concrete consumer need.
