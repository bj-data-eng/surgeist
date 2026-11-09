# Block clearance

This determination specifies Block's clearance admission and settlement in the
enclosing block formatting context (BFC). The
[selected CSS2 clearance rules](css2--visuren.html--3f334c530cf4.md#clearance) and
[adjoining-margin rules](css2--box.html--8875bbcdefe6.md#what-is-adjoining)
provide the model. The contact, amount and dependency-order choices below are
the adopted policy for [#2125](https://github.com/bj-data-eng/surgeist/issues/2125).

## Admission and settlement

Use one BFC-local logical block coordinate frame. Let H be the hypothetical
block-start border edge with the target's clear treated as None, and Fh the
greatest relevant earlier float's block-end margin edge in that same
hypothetical placement. Introduce clearance exactly when Fh exists and H < Fh.
Exact contact introduces none. No relevant float is an absent bound, including
for negative coordinates. Clear None, the other side, later floats and floats
in another BFC supply no bound.

A zero-height float retains its actual margin-edge authority. Its bottom edge
can precede, equal or follow H; margins affect that edge. Shape contour occupancy
does not determine clearance.

Introduced clearance separates the applicable adjoining margins. Keep its
introduced state when the used amount is zero or negative, or the used border
meets the float bottom. An empty box's own margins can still collapse together
and with following siblings; a resulting group containing clearance does not
collapse with the parent's bottom margin.

Save H before separating margins. Let S be the border coordinate after
separation but before clearance, and Fs the relevant float bottom in that
separated placement. The selected greater-of-two amount is
`C = max(Fs - S, H - S)`, giving border `max(Fs, H)`. Keep C signed. Local floats
can move with their owner during separation; inherited external floats remain
anchored. Fh is the admission bound, not a replacement for Fs. Parent movement
inside the BFC does not change the saved H. Translating the entire BFC leaves
these BFC-local coordinates unchanged.

For example, a zero-edge parent P contains a height5 left float F followed by B
with top margin10, height10 and clear Left. Hypothetical collapse gives P=10,
H=10 and Fh=15.
Clearance is introduced. Separation gives P=0, F=0..5 and S=10, so C=0 and B=10.
P's height is20. Rechecking admission against Fs=5 would lose the required
margin separation.

## Qualified source prefix

Resolve clearance offers in source/tree order within the affected adjoining
component. A target's contextual None probe preserves its fully qualified
earlier prefix. It can provisionally resolve later offers to obtain the complete
hypothetical ancestor position. Once the target's admission is established,
retain that decision and H, discard incompatible speculative suffix receipts,
and recompute affected later work. A later probe does not reopen that prefix.
A no-clearance decision alone does not fix an unresolved ancestor origin.

Fresh source, style, request, BFC or dependency inputs require fresh
qualification. A contextual None probe differs from a fresh layout with an
authored None value: the fresh input cannot inherit the previous prefix.
Hypothetical qualification supplies no completed child geometry or publication
effects. Actual child completion and imported floats use the accepted origin.

For example, earlier half-width left/right floats end at15 and25. A zero-edge
parent P contains empty E with top/bottom margins8/12 and clear Left, followed
by B with top margin20, height10 and clear Right. With E suppressed, provisional B
separates P to12, so E's H is12. Actual E introduces clearance7 from S=8,
putting its border at15. Its through-group base is7 and own end is19.
B's contextual None probe retains E: `H_B = 7 + max(8, 12, 20) = 27`.
Thus B introduces no clearance; P starts at0 and has height37. A right float
ending at27 gives the same result. At30, B introduces clearance with
`S_B = 19 + 20 = 39`, `C_B = -9`, border30 and P height40. E's border15 is not
the normal cursor for the next adjoining group.

## Evidence and policy boundaries

CSS2's clearance prose says “not past”; its worked example uses a strict
inequality. The [contact determination](https://github.com/bj-data-eng/surgeist/issues/2125#issuecomment-6728652731)
selects strict admission, supported by pinned Blink comparisons for
[ordinary blocks](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/block_layout_algorithm.cc#L190-L198)
and [self-collapsing children](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/block_layout_algorithm.cc#L2488-L2508).
Blink's [exclusion owner](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/exclusions/exclusion_space.cc#L262-L333)
retains clearance bounds without filtering out zero-height records.

The [coordinate determination](https://github.com/bj-data-eng/surgeist/issues/2125#issuecomment-6728787611)
selects CSS2's greater-of-two amount in the common BFC frame. The pinned
[introduced-zero test](https://github.com/web-platform-tests/wpt/blob/792c7512e35133a4481f267dbd6417fdedd0df2c/css/CSS2/floats-clear/clearance-006.xht)
distinguishes introduced zero amount from no clearance. The
[adjoining-float test](https://github.com/web-platform-tests/wpt/blob/792c7512e35133a4481f267dbd6417fdedd0df2c/css/CSS2/floats-clear/adjoining-float-before-clearance.html)
shows the shared hypothetical-origin dependency; its exact-bottom expectation
does not select Surgeist's amount policy.

The [prefix determination](https://github.com/bj-data-eng/surgeist/issues/2125#issuecomment-6733332381)
completes the dependency ordering not uniquely specified by CSS2.
[CSSWG #9013](https://github.com/w3c/csswg-drafts/issues/9013) records the
counterfactual-ordering question. Pinned Blink preserves established clearance
across [restart](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/block_layout_algorithm.cc#L2595-L2612);
WebKit carries a cleared empty sibling's margin state into the
[following sibling](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/RenderBlockFlow.cpp#L1668-L1708).
These are inspected implementation and test witnesses, not a CSSWG resolution
or browser execution results for the examples above.
