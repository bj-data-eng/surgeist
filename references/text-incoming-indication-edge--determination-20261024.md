# Incoming indication custody at inline edges

Determination for [#64](https://github.com/bj-data-eng/surgeist/issues/64),
24 October 2026. This is explicit Surgeist policy completing a source gap,
not a CSSWG resolution or a claim of browser interoperability.

## Evidence

[CSS Text 4](https://drafts.csswg.org/css-text-4/), checked at its 14 August
2026 edition, is retained in
[the full reference](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md).
Hyphenation rules preserve the indication's style ownership and language
placement convention. The
[shaping boundary rule](https://drafts.csswg.org/css-text-4/#shaping-across-element-boundaries)
requires a shaping break when nonzero inline-axis margin, border or padding
separates typographic character units. Neither specifies the ordering of a
native incoming LogicalAfter indication and a selected source-free Start.

[CSSWG #2975's adopted resolution](https://github.com/w3c/csswg-drafts/issues/2975#issuecomment-1403956955)
distinguishes the inserted string from its placement convention, without
settling endpoint custody. Retrieved discussion in
[#13688](https://github.com/w3c/csswg-drafts/issues/13688) supplies no endpoint
ordering resolution.
[Pinned WebKit evidence](webkit-hyphen-indication--73aa6c89e2cb-metadata.md)
at 73aa6c89e2cb77c46184a81aec944e4ab99d114d appends a Before indication;
it does not establish native After behavior.

Source inspection at HEAD 0a1612e42c95999b5f2170520062b6d1877ab951 with
active implementation drafts found that presentation construction authenticates
After against the receiving original, then prepends it ahead of all selected
controls. Receiver-based cut transport disagrees with that placement when a
positive Start precedes the receiver. The coordinator's diagnostic demonstrates
the mismatch. No tests were executed for this determination.

## Decision

Insert the whole incoming indication immediately before its authenticated
receiver in the selected logical mixed-event sequence. Preserve all genuine
prefix controls in captured order. The reported case becomes Start, indication,
receiver, remaining originals. Use the same insertion basis for zero and positive
edges: edge magnitude controls the shaping barrier, not attachment identity.

The receiver supplies direction and geometric attachment, not style, font,
language convention or DOM ownership. Preserve the established indication
owner and empty generated source association. Endpoints remain source-free;
do not invent original characters, endpoints, scopes or equal-offset anchors.

Keep original L1 processing independent of generated strings. Insert the local
indication bidi domain at the receiver's full authenticated level, preserving
original order and each control's own level. Reorder the actual mixed sequence;
do not reanalyze paragraph bidi or relocate coordinates after shaping. Same-level
RTL fixture geometry is not a general mixed-level placement rule.

Use this attachment for original-context seeds, transported presentation cuts,
shaping and placement. An eligible positive Start is before the whole
indication-plus-receiver side and cannot divide their joining. It still blocks
genuine original context across itself. Preserve computed/original joining
semantics; concatenation is not a replacement algorithm. Whole-indication custody
does not introduce an atomic box or mandatory shaping barrier: retain local
bidi, shared graphemes and compatible joint GSUB/GPOS.

Authenticate the actual participant and scalar, including genuine formatting
controls or deleted originals with directional authority, rather than the first
visible glyph. Keep the existing unsupported atomic-receiver error and
MissingReceivingBidiAuthority refusal. Empty indications retain obligation
identity without visible insertion. Nested starts, ends and empty metadata
retain captured order. Outgoing Before custody is unchanged.

## Implementation and acceptance

Text owns internal presentation insertion and consistent cut transport. No
public API addition is needed. Layout supplies authentic endpoints and consumes
placement; it must not compensate by moving glyphs. Integrate this determination
into the active Text/Layout contract draft.

- Exercise the five-BEH/SHY fixture at Start widths zero and five: receiver raw
  advances remain 40 and 45. The indication and first receiver occupy the
  receiver side of the positive Start and preserve expected computed/original
  joining behavior.
- Cover LTR, RTL and differing nested levels without reducing custody to
  endpoint parity; preserve original bidi order and local containment.
- Cover nested prefix controls, formatting/deleted receivers, empty indications,
  missing receivers and unsupported receivers with existing precise refusals.
- Verify owner style/font/convention, source maps, continuation identity,
  source-free endpoint readback, paint placement and unchanged Before behavior.

Independent read-only review accepted this policy with these qualifications.
That review and source inspection are not executed test evidence.
