# Range geometry: CSSWG and browser-test evidence

Checked 22 October 2026 against Surgeist `6ea7a8b13f15d78e3919f0a52b54b4168abb4617`. This is a bounded reference and factual Markdown synopsis, not another policy owner or a claim of executed browser conformance. The [Text/Layout contract](../docs/text-layout-contract.md#nonempty-text-range-fragments) owns implementation determinations.

## Current specifications

The [current CSSOM View §9 capture](cssom-view-range--editor-capture-20261022.md) retains the complete selected section from the editor draft dated 8 October 2026. Its relevant requirements remain font-metric and advance geometry, partial typographic-unit expansion, content ordering and ancestor transforms. Bounding aggregation has separate zero-rectangle rules. It does not specify an exact provider-cluster partition, deleted-only endpoint projection, or independent PDI position.

The current [CSS Text 4 editor draft](https://drafts.csswg.org/css-text-4/) displays 14 August 2026. Checked §1.4, §4.3 and §6.3 against the [retained dated edition](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md): the relevant EGC-based operation-specific unit, duplicate-space zero-advance, removal and hyphenation/source distinctions remain. These clauses do not by themselves prescribe the missing Range projections. This check does not select the whole floating draft as a new implementation baseline.

## CSSWG issue and resolution check

Searched `w3c/csswg-drafts` issues (open and closed) for `getClientRects`, then combinations with `RESOLVED`, `white`, `ligature`, `newline`, `hyphen` and `isolate`; also searched `getBoundingClientRect hyphen` and `typographic range`. Read the following issue bodies and every returned comment. None contains an adopted resolution settling the nonempty deleted-only, preserved-newline, SHY-only or PDI-only rectangle policies. This is a bounded negative result, not a claim to have exhausted all WG minutes or browser tests. Search indexing and later changes can affect discovery.

| Issue | State at capture | Relevant evidence and limit |
| --- | --- | --- |
| [1156](https://github.com/w3c/csswg-drafts/issues/1156) | Open | Collapsed-range ambiguity; a comment points to the explicit identical-boundary wording. Not a decision on nonempty deleted source. |
| [2514](https://github.com/w3c/csswg-drafts/issues/2514) | Open | Empty elements/text and normalization; detailed proposed wording is a participant proposal, not a WG resolution. Its old browser observations are not current test results. |
| [2861](https://github.com/w3c/csswg-drafts/issues/2861) | Open | Document membership/shadow-tree ambiguity; comments propose rendered-box wording. No adopted resolution in the captured discussion. |
| [3456](https://github.com/w3c/csswg-drafts/issues/3456) | Open | Meaning of selected element remains disputed. Do not claim every DOM consumer membership detail is settled. |
| [7155](https://github.com/w3c/csswg-drafts/issues/7155) | Open | Reports divergent nested Text rectangle enumeration and points to Blink's test. No adopted resolution in the captured discussion. |
| [12331](https://github.com/w3c/csswg-drafts/issues/12331) | Open | Selected parent versus overflowing descendant geometry. Does not authorize replacing the specified element rule with arbitrary descendant union. |

Closed hits about zoom or Highlight APIs do not settle these text-unit cases. Open status alone is not the evidence: the captured bodies/comments were inspected for decisions.

## Browser test assertions

These are inspected upstream test expectations, not results of running browsers. WPT tests can expose intended behavior without proving interoperability. Browser-specific tests are implementation witnesses, not CSSWG resolutions.

WPT snapshot: `69f2bba61ce7e9469ccbce53de4644a89f6b7dff`. Blink snapshot: `daf70f9ddee4b45d4f8870f574fbde61c0db4ca7`.

| Test | Assertions inspected | Limit |
| --- | --- | --- |
| [range-client-rects-surrogate-indexing.html](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/cssom-view/range-client-rects-surrogate-indexing.html) | Nonempty ranges beginning/ending within surrogate pairs have the same rectangle widths as the full-pair ranges, including two Text origins. | Supports partial-pair expansion; does not decide optional ligature or deleted-source geometry. |
| [range-bounding-client-rect-rtl-ligature-split.html](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/cssom-view/range-bounding-client-rect-rtl-ligature-split.html) | Collapsed positions inside an RTL span beginning within an Arabic ligature stay within the full-line horizontal bounds (one-pixel tolerance). | Does not assert equal shares, nonempty partial ligature geometry, or an exact caret position. |
| [range-bounding-client-rect-with-nested-text.html](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/cssom-view/range-bounding-client-rect-with-nested-text.html) | Six total rectangles and four nonempty rectangles for its nested Text fixture. | Supports retaining nested Text geometry; not proof that open issue7155 is resolved or that all engines pass. |
| [range-bounding-client-rect-with-shadow-dom.html](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/cssom-view/range-bounding-client-rect-with-shadow-dom.html) | Its fixture expects detached ranges empty, a range wholly inside the attached shadow tree nonempty, and a collapsed Text range nonempty; also has a cross-root endpoint case. | Do not reduce document membership to root == Document. Cross-root Range endpoint setters affect the resulting range; this is not a composed-range definition. |
| [getClientRects-leading-trailing-whitespaces.html](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/web_tests/fast/dom/Range/getClientRects-leading-trailing-whitespaces.html) | Leading offsets share a position; trailing offsets share the terminal position. | Only setStart is called on a fresh Range: these are collapsed ranges. They do not validate empty observations for nonempty removed-only selections. |
| [getBoundingClientRect-linebreak-character.html](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/web_tests/fast/dom/Range/getBoundingClientRect-linebreak-character.html) | With preformatted a LF b LF LF LF c, individually selected characters/newlines have the asserted increasing vertical positions. | Supports newline placement ordering; does not assert width, font height, terminal LF, CRLF halves, or client-rectangle multiplicity. |
| [getClientRects.html](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/web_tests/fast/dom/Range/getClientRects.html) | Fixtures compare rectangle counts, positions and sizes for nested content, multiline, transforms and vertical writing. | Broader consumer witness; not the exact SHY/PDI/deleted-only cases. |

Searches of WPT and Chromium code for `getClientRects hyphen`, and WPT for `getClientRects whitespace`, did not establish an exact SHY-only or PDI-only test. The reviewed directory inventories were WPT `css/cssom-view` and Blink `fast/dom/Range`. Absence from this bounded search is not absence from all tests.

## Consequences for the determinations

- The earlier whole-provider-ligature Range rule is withdrawn. CSS units are operation-specific; float closure does not define Range closure. The [existing Blink shaping witness](blink-formatting-unit-range--daf70f9ddee4.md) supports grapheme-based advance partitioning, but the inspected RTL caret WPT is not an exact partition oracle.
- Equal-share allocation is an explicit bounded fallback, not a standard requirement. Preserve any existing authentic unit allocation and separately owned spacing/justification; never redistribute them by dividing a whole finalized fragment width.
- Empty observations for suppressed duplicates/deleted-only source remain a native fallback policy, not a tested interoperable web rule. In particular, the collapsed whitespace test must not be cited as its validation. Keep ordinary zero-advance retained units distinct.
- The preserved-LF terminating-line choice has limited browser-test support for vertical ordering. Its exact zero advance, font association, terminal-emptyline exclusion and admitted CRLF mapping remain explicit policy details, not assertions proven by that test.
- Source-defined selected SHY attribution remains explicit native policy, supported by an authentic defining-source relation. A control anchor is insufficient. The browser generated-hyphen path is not evidence that every automatic hyphen replaces a nonempty source interval.
- The PDI x24 choice remains an explicit policy with an existing observed Chrome x72 disagreement. No reviewed resolution or test removes that disagreement or makes either coordinate universally mandated.
- Native capability refusals are not web Range return semantics. The web consumer must separately implement legitimate empty results, collapsed Text requests, selected-element contributions, nested Text enumeration and shadow-tree membership. The new WPT shadow test strengthens the warning against a root-equals-Document shortcut; the open membership issue remains relevant.

These results retain honest source gaps instead of promoting proposals or partial browser assertions into standards. They do not approve full web Range conformance or replace strict browser comparisons.

## Capture provenance

The standards were captured as HTML; §9 is converted separately. Issue API JSON is already Markdown-bearing; the table above is a factual synopsis rather than a redistribution of discussion text. Test sources are represented by immutable locators, hashes and assertion summaries; no test code is ingested or compiled. Existing Chromium attribution remains in [licenses/chromium](../licenses/chromium/NOTICE.md); WPT source licensing remains at its [pinned LICENSE.md](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/LICENSE.md). W3C document licensing is retained in the converted standard excerpt. Source artifacts below are identified without making the ignored working captures implementation dependencies.

| Captured source | Bytes | SHA-256 |
| --- | ---: | --- |
| [cssom-view.html](https://drafts.csswg.org/cssom-view/) | 1087595 | `71d9ce94f21ec00a5a333ee5a20d2b0f999a862d3543fbdb85cd4d1d3dcac176` |
| [css-text-4.html](https://drafts.csswg.org/css-text-4/) | 3355016 | `c13b32c627840bd2e010b793f1f750ef0a00b62f207c8e59276adeda5301126d` |
| [issue-1156.json](https://api.github.com/repos/w3c/csswg-drafts/issues/1156) | 5002 | `c65fec28f5e866a6174152214e46f069e80c7d78cbafd459337dc5fab9166029` |
| [issue-1156-comments.json](https://api.github.com/repos/w3c/csswg-drafts/issues/1156/comments?per_page=100) | 2169 | `a13df570d7c4d280cce24f9ab7bd611fdabb69b16a175256871daad52337cd1d` |
| [issue-2514.json](https://api.github.com/repos/w3c/csswg-drafts/issues/2514) | 4825 | `3309f272367bae313263f3291d0366964fd6868e4f9e7338bdfcc0ef5752b1a0` |
| [issue-2514-comments.json](https://api.github.com/repos/w3c/csswg-drafts/issues/2514/comments?per_page=100) | 13080 | `e22334ce47c9606ea18506fc5f923d161579bcc86749be28e545ef0905b5cd51` |
| [issue-2861.json](https://api.github.com/repos/w3c/csswg-drafts/issues/2861) | 3221 | `171936544a642cc2dde99306039985d110722b27100e742493e923a2506fbab4` |
| [issue-2861-comments.json](https://api.github.com/repos/w3c/csswg-drafts/issues/2861/comments?per_page=100) | 8715 | `b2447246827ca01e28fc77ed99b0a60b374fdc5a95f5488d776b6e4259e33400` |
| [issue-3456.json](https://api.github.com/repos/w3c/csswg-drafts/issues/3456) | 3833 | `2839e8b28353210bdf2b38cc7b35c60369d48444f5e1fbf5dc1f3aab5fb400f5` |
| [issue-3456-comments.json](https://api.github.com/repos/w3c/csswg-drafts/issues/3456/comments?per_page=100) | 5 | `2ba33ca0557f1bb5b7ba88d67f9d0093c7185a36ec51fe2b7bd9372d3e001d6d` |
| [issue-7155.json](https://api.github.com/repos/w3c/csswg-drafts/issues/7155) | 4484 | `79e71cd1e42660221ebcb10c33e5bd1ad5974bd73f5dab96a2293f95687d7aa5` |
| [issue-7155-comments.json](https://api.github.com/repos/w3c/csswg-drafts/issues/7155/comments?per_page=100) | 5 | `2ba33ca0557f1bb5b7ba88d67f9d0093c7185a36ec51fe2b7bd9372d3e001d6d` |
| [issue-12331.json](https://api.github.com/repos/w3c/csswg-drafts/issues/12331) | 4343 | `71ea21b746cb97cafade5667ec472becf6c1143302dc3a6568ca94e17e3054c5` |
| [issue-12331-comments.json](https://api.github.com/repos/w3c/csswg-drafts/issues/12331/comments?per_page=100) | 5 | `2ba33ca0557f1bb5b7ba88d67f9d0093c7185a36ec51fe2b7bd9372d3e001d6d` |
| [range-client-rects-surrogate-indexing.html](https://raw.githubusercontent.com/web-platform-tests/wpt/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/cssom-view/range-client-rects-surrogate-indexing.html) | 2423 | `a1703fcbcfb71b5fd3b51b89d8ded25ad695f251e41ea0200d170e0fb4e8bce2` |
| [range-bounding-client-rect-rtl-ligature-split.html](https://raw.githubusercontent.com/web-platform-tests/wpt/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/cssom-view/range-bounding-client-rect-rtl-ligature-split.html) | 1474 | `40445e2e06620e1bdb2b0c6e9b766bbb0d6a8aae5d498d4988b6c0abc42cec9c` |
| [range-bounding-client-rect-with-nested-text.html](https://raw.githubusercontent.com/web-platform-tests/wpt/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/cssom-view/range-bounding-client-rect-with-nested-text.html) | 1509 | `583a7707151fc2f14ec593dce1474d26bb03129f2b33499c3cd4970c1a61609b` |
| [range-bounding-client-rect-with-shadow-dom.html](https://raw.githubusercontent.com/web-platform-tests/wpt/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/cssom-view/range-bounding-client-rect-with-shadow-dom.html) | 2674 | `5c5823409e73577efebbae3035cba3df5a1bb5e93cee8e4e395e57557461f119` |
| [getClientRects-leading-trailing-whitespaces.html](https://raw.githubusercontent.com/chromium/chromium/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/web_tests/fast/dom/Range/getClientRects-leading-trailing-whitespaces.html) | 1128 | `15c17c23dc364b6629e4e86876684b60c9de6eb51efb13ff96e1f1a61d19ea25` |
| [getBoundingClientRect-linebreak-character.html](https://raw.githubusercontent.com/chromium/chromium/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/web_tests/fast/dom/Range/getBoundingClientRect-linebreak-character.html) | 1524 | `6bafec9bcfed4df66ee9c0f2151f038f912063cd6811aa455d59cbe83738f5cf` |
| [getClientRects.html](https://raw.githubusercontent.com/chromium/chromium/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/web_tests/fast/dom/Range/getClientRects.html) | 21477 | `5ec312c5299b2f61940365b8e5d89437a60b3cabbd6164e6984d43f5de34532c` |
