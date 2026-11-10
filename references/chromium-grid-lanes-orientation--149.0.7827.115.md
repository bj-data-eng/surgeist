# Chromium Grid Lanes orientation

This source witness explains the input difference in the Grid Lanes axis corpus.
It records source inspection, not an executed geometry comparison.

## Source identity

Chrome for Testing `149.0.7827.115` resolves to Chromium revision
`daf70f9ddee4b45d4f8870f574fbde61c0db4ca7`. The inspected file is
[`third_party/blink/renderer/core/style/computed_style.h`](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/style/computed_style.h#L1183),
with complete-file SHA-256
`34e8c9138514959dc36eb3e92f1b675beba810677e31747cda82a504551d0070`.
The bounded witness is `ComputedStyle::GridLanesTrackSizingDirection`, lines
1183–1201, inspected on 2026-11-10. No upstream source code is reproduced here.

## Input interpretation

Chromium selects the grid axis from `grid-lanes-direction`. Its `normal` branch
selects columns when columns are specified, including when both templates are
specified; it selects rows only when columns are absent and rows are present.
This function does not use `grid-auto-flow` to select orientation.

Surgeist's [adopted orientation contract](https://github.com/bj-data-eng/surgeist/issues/993)
selects rows for explicit `grid-auto-flow:column` and columns for explicit
`grid-auto-flow:row`, independently of template presence. The
[selected WebKit witness](webkit-grid-lanes-witnesses--73aa6c89e2cb--5b433979a8aa.md)
supports that choice. These are different interpretations of the same authored
input when both templates are present.

The [Grid 3 orientation clause](https://drafts.csswg.org/css-grid-3/#grid-lanes-orientation)
still leaves the controlling property undecided as inspected on 2026-11-10.
The [CSSWG resolution](https://github.com/w3c/csswg-drafts/issues/12803#issuecomment-3607869618)
establishes the template-dependent `normal` default, without selecting the
explicit controlling property. Neither source replaces Surgeist's selected
explicit-orientation policy.

## Corpus contract

[#2326](https://github.com/bj-data-eng/surgeist/issues/2326) requires distinct
authored controls that remove only the template on Surgeist's selected stacking
axis. This lets Chromium's default select the same grid axis. Original sources,
historical captures and raw differences remain separate evidence. Qualification
requires measured control equality and complete Native original/control equality;
source inspection alone establishes neither result.
