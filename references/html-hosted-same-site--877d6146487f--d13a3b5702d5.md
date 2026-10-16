# HTML same-site comparison for hosted Settings

This bounded reference completes the same-site definitions reached by the
existing [Window Settings algorithms](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md).
It reuses the selected HTML edition and adds no source identity, URL engine,
public-suffix service, JavaScript runtime or implementation-conformance claim.

## Source and attribution

WHATWG, *HTML Living Standard*, immutable snapshot
`877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026), extracted offline
on 16 October 2026 from the already-retained complete HTML: 14,930,614 bytes,
SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
The [original sites section](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sites)
remains authoritative.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose uses
[Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/).
The unchanged complete [WHATWG license](../licenses/whatwg/LICENSE), SHA-256
`85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`,
and [supplemental notice](../licenses/whatwg/NOTICE.md#hosted-same-site-definitions--2026-10-16)
retain the original conditions and disclaimer. BSD 3-Clause applies only to
portions incorporated into source code, not as an alternative prose license.
No endorsement is implied.

Changes by Surgeist: bounded offline selection, Markdown headings, provenance
and the separately marked reading guide. The two complete algorithm divs below
are retained byte for byte inside inert fenced blocks, including their original
links, IDs, whitespace and branch order. No source script, renderer or acquired
tool ran. Their original bytes joined by one extra LF have SHA-256
`d13a3b5702d5a580a69546be2cc80a55cc29b7ce731552573716c07618f1cde5`;
the filename suffix identifies that input, not this Markdown representation.

| Original raw lines, inclusive | Complete unit | Bytes | Original SHA-256 |
| --- | --- | --- | --- |
| 64446–64454 | Sites are same site | 1,050 | `5db184d37a6d47d31f09945c2d9c1dd636b222dc368b4722aff24643b66049f9` |
| 64486–64493 | Origins are same site | 701 | `8d09ed975cf9382a86f22acb0d61e70c413842899e2fb74e227a2161cc1627e5` |

## Reading guide — explanatory, not replacement source

The existing Settings reference retains obtain-a-site, Window Settings setup
and the initial constructor in full. Obtaining a site from an opaque origin
returns that original origin. The definitions below then distinguish the same
opaque origin from every different opaque origin; equal serialized spellings
such as `null` do not establish identity.

Settings' cross-site-ancestor algorithm follows the current parent navigable's
active Document and relevant Settings. Its top-level-origin field instead
retains the operand supplied during construction. These are distinct source
roles. The bounded native implementation and its admission and availability
requirements remain with the owning issues and consuming subsystems.

## Sites are same site

[Original definition](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-site-same-site).

```html
  <div data-algorithm="">
  <p>Two <a href=#site id=sites:site>sites</a>, <var>A</var> and <var>B</var>, are said to be <dfn data-dfn-for=site id=concept-site-same-site data-export="">same site</dfn> if the following algorithm
  returns true:</p>

  <ol><li><p>If <var>A</var> and <var>B</var> are the same <a href=#concept-origin-opaque id=sites:concept-origin-opaque-3>opaque
   origin</a>, then return true.<li><p>If <var>A</var> or <var>B</var> is an <a href=#concept-origin-opaque id=sites:concept-origin-opaque-4>opaque
   origin</a>, then return false.<li><p>If <var>A</var>'s and <var>B</var>'s <a href=#concept-scheme-and-host-scheme id=sites:concept-scheme-and-host-scheme>scheme</a> values are different, then return
   false.<li><p>If <var>A</var>'s and <var>B</var>'s <a href=#concept-scheme-and-host-host id=sites:concept-scheme-and-host-host>host</a> values are not <a href=https://url.spec.whatwg.org/#concept-host-equals id=sites:host-equals data-x-internal=host-equals>equal</a>, then return false.<li><p>Return true.</ol>
  </div>
```

## Origins are same site

[Original definition](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-site).

```html
  <div data-algorithm="">
  <p>Two <a href=#concept-origin id=sites:concept-origin-2>origins</a>, <var>A</var> and <var>B</var>, are said to be <dfn id=same-site data-export="">same site</dfn> if the following algorithm returns true:</p>

  <ol><li><p>Let <var>siteA</var> be the result of <a href=#obtain-a-site id=sites:obtain-a-site>obtaining a
   site</a> given <var>A</var>.<li><p>Let <var>siteB</var> be the result of <a href=#obtain-a-site id=sites:obtain-a-site-2>obtaining a
   site</a> given <var>B</var>.<li><p>If <var>siteA</var> is <a href=#concept-site-same-site id=sites:concept-site-same-site>same site</a> with
   <var>siteB</var>, then return true.<li><p>Return false.</ol>
  </div>
```
