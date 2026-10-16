# HTML opener-policy values and default record

Bounded source retention for the actual default and inherited opener-policy
record used by initial native child Document construction. Complete selected
definitions remain authoritative; their retention does not implement JavaScript,
HTTP header parsing, COOP enforcement, reporting or browsing-context-group switches.

## Source basis and attribution

Source: [immutable WHATWG HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/),
revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026).
Extracted offline on 16 October 2026 from the already-retained complete HTML,
14,930,614 bytes, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
No new retrieval, edition, executable, renderer or software acquisition is used.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is
[Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/).
The existing [complete WHATWG license](../licenses/whatwg/LICENSE), SHA-256
`85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`,
and [pinned HTML attribution](../licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11)
apply. The BSD 3-Clause provision concerns specification portions incorporated
into source code; it is not an alternative license for this retained prose.

Changes by Surgeist on 16 October 2026: offline bounded extraction, Markdown
headings, provenance and separately identified editorial scope notes. The
original HTML is preserved **byte for byte** inside the text fence, including
source whitespace, complete defining containers, original variable/ID spelling,
all value definitions, informative notes and links. No source href is rewritten
and no source script is executed. No endorsement or browser-conformance claim
is implied; this is implementation-support material, not a replacement standard.

| Original raw lines | Complete retained unit | Original fragment SHA-256 |
| --- | --- | --- |
| 64771–64827 | Section heading, complete opener-policy value definition/list with its informative notes, and complete four-field opener-policy record | `d124556cde21fa872bee44e6a57bc1c98473b8a8f9a2c60b09283f1c2d77e9e5` |

The single original fragment has 8,104 bytes and 57 original lines.
The joined-fragment hash for this one-fragment selection is the same SHA-256
`d124556cde21fa872bee44e6a57bc1c98473b8a8f9a2c60b09283f1c2d77e9e5`. The filename suffix identifies
that original input, not the final Markdown. The value list's dl opens64777 and
closes64819, including its complete nested informative-note div/list. The record
paragraph is64823; its complete field list opens64825 and closes64827.
The next algorithm, matching opener-policy values, starts64829 and is excluded.

## Editorial initial-construction boundary

The existing [Document definitions](html-language-foundations--877d6146487f--b55e73c31978.md)
retain raw7086–7087: every Document's opener policy initially is a new opener
policy. The retained record below supplies its actual four defaults:

| Record field | Original initial value |
| --- | --- |
| value | unsafe-none |
| reporting endpoint | null |
| report-only value | unsafe-none |
| report-only reporting endpoint | null |

The existing [BC/Document creation body](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
retains raw71653–71719. For a non-null creator, after ancestor-list initialization
it sets referrer from the actual creator URL, clones the actual policy container
and conditionally sets the child's opener policy to the top-level browsing
context's active Document policy. This opener-policy record is distinct from
the embedder policy inside the policy container and from Permissions Policy.

For an actual initial parent and child without sandbox, determine-origin returns
the actual creator origin for about:blank. Its opaque origin identity is shared
with the original top-level settings origin, so the same-origin inheritance
condition is reached; equal `null` text is not an identity test. The child must
receive the actual source-selected existing opener policy, not substitute a new
default based solely on no-opener/no-JavaScript labels. In the closed initial
profile the actual inherited record has the four default values above; this
does not admit caller-provided policies or future header-derived changes.

The source's other opener-policy values and their complete informative security
note remain visible below to preserve the complete definition list. Their
retention does not admit navigation or execute their group-switch effects.
The selected initial-construction path reads/copies an established record;
it does not call matching-coop64829 onward, obtain-coop/header parsing,
response enforcement/reporting or group-switch algorithms. Those bodies are
not selected by this reference, and external links in the informative note
do not establish transitive source closure or implemented browser services.

The actual creator-origin equality, Document association, policy-container clone
and settings/top-level relations reuse their existing retained source owners.
No newly reached COOP matching/header algorithm is required just to establish
or inherit this default record. The separate Frame embedder/iframe-sandbox-slot
typing question belongs to #1124 and is not resolved by this source selection.

## Literal selected definitions

<!-- selected-source-fragment:64771-64827 -->
```text
  <h4 id=cross-origin-opener-policies><span class=secno>7.1.3</span> Cross-origin opener policies<a href=#cross-origin-opener-policies class=self-link></a></h4>

  <p>An <dfn id=cross-origin-opener-policy-value>opener policy value</dfn> allows a document which
  is navigated to in a <a href=#top-level-browsing-context id=cross-origin-opener-policies:top-level-browsing-context>top-level browsing context</a> to force the creation of a new
  <a href=#top-level-browsing-context id=cross-origin-opener-policies:top-level-browsing-context-2>top-level browsing context</a>, and a corresponding <a href=#tlbc-group id=cross-origin-opener-policies:tlbc-group>group</a>. The possible values are:</p>

  <dl><dt>"<dfn id=coop-unsafe-none><code>unsafe-none</code></dfn>"<dd><p>This is the (current) default and means that the document will occupy the same
   <a href=#top-level-browsing-context id=cross-origin-opener-policies:top-level-browsing-context-3>top-level browsing context</a> as its predecessor, unless that document specified a
   different <a href=#cross-origin-opener-policy id=cross-origin-opener-policies:cross-origin-opener-policy>opener policy</a>.<dt>"<dfn id=coop-same-origin-allow-popups><code>same-origin-allow-popups</code></dfn>"<dd><p>This forces the creation of a new <a href=#top-level-browsing-context id=cross-origin-opener-policies:top-level-browsing-context-4>top-level browsing context</a> for the
   document, unless its predecessor specified the same <a href=#cross-origin-opener-policy id=cross-origin-opener-policies:cross-origin-opener-policy-2>opener policy</a> and they are
   <a href=#same-origin id=cross-origin-opener-policies:same-origin>same origin</a>.<dt>"<dfn id=coop-same-origin><code>same-origin</code></dfn>"<dd><p>This behaves the same as "<code id=cross-origin-opener-policies:coop-same-origin-allow-popups><a href=#coop-same-origin-allow-popups>same-origin-allow-popups</a></code>", with the addition that
   any <a href=#auxiliary-browsing-context id=cross-origin-opener-policies:auxiliary-browsing-context>auxiliary browsing context</a> created needs to contain <a href=#same-origin id=cross-origin-opener-policies:same-origin-2>same origin</a>
   documents that also have the same <a href=#cross-origin-opener-policy id=cross-origin-opener-policies:cross-origin-opener-policy-3>opener policy</a> or it will appear closed to the
   opener.<dt>"<dfn id=coop-same-origin-plus-coep><code>same-origin-plus-COEP</code></dfn>"<dd>
    <p>This behaves the same as "<code id=cross-origin-opener-policies:coop-same-origin><a href=#coop-same-origin>same-origin</a></code>", with the
    addition that it sets the (new) <a href=#top-level-browsing-context id=cross-origin-opener-policies:top-level-browsing-context-5>top-level browsing context</a>'s <a href=#tlbc-group id=cross-origin-opener-policies:tlbc-group-2>group</a>'s <a href=#bcg-cross-origin-isolation id=cross-origin-opener-policies:bcg-cross-origin-isolation>cross-origin isolation
    mode</a> to one of "<code id=cross-origin-opener-policies:cross-origin-isolation-logical><a href=#cross-origin-isolation-logical>logical</a></code>" or "<code id=cross-origin-opener-policies:cross-origin-isolation-concrete><a href=#cross-origin-isolation-concrete>concrete</a></code>".</p>

    <p class=note>"<code id=cross-origin-opener-policies:coop-same-origin-plus-coep><a href=#coop-same-origin-plus-coep>same-origin-plus-COEP</a></code>" cannot
    be directly set via the `<code id=cross-origin-opener-policies:cross-origin-opener-policy-2-2><a href=#cross-origin-opener-policy-2>Cross-Origin-Opener-Policy</a></code>` header, but results from a
    combination of setting both `<code><a href=#cross-origin-opener-policy-2 id=cross-origin-opener-policies:cross-origin-opener-policy-2-3>Cross-Origin-Opener-Policy</a>: <a href=#coop-same-origin id=cross-origin-opener-policies:coop-same-origin-2>same-origin</a></code>` and a
    `<code id=cross-origin-opener-policies:cross-origin-embedder-policy><a href=#cross-origin-embedder-policy>Cross-Origin-Embedder-Policy</a></code>` header whose value is <a href=#compatible-with-cross-origin-isolation id=cross-origin-opener-policies:compatible-with-cross-origin-isolation>compatible with
    cross-origin isolation</a> together.</p>
   <dt>"<dfn id=coop-noopener-allow-popups><code>noopener-allow-popups</code></dfn>"<dd>
    <p>This forces the creation of a new <a href=#top-level-browsing-context id=cross-origin-opener-policies:top-level-browsing-context-6>top-level browsing context</a> for the document,
    regardless of its predecessor.</p>

    <div class=note>
     <p>While including a <code id=cross-origin-opener-policies:coop-noopener-allow-popups><a href=#coop-noopener-allow-popups>noopener-allow-popups</a></code> value severs the opener
     relationship between the document on which it is applied and its opener, it does not create a
     robust security boundary between those same-origin documents.</p>

     <p>Other risks from same-origin applications include:</p>

     <ul><li><p>Same-origin requests fetching the document's content — could be mitigated through
      Fetch Metadata filtering. <a href=#refsFETCHMETADATA>[FETCHMETADATA]</a><li><p>Same-origin framing - could be mitigated through <code id=cross-origin-opener-policies:x-frame-options><a href=#x-frame-options>X-Frame-Options</a></code> or CSP
      <code id=cross-origin-opener-policies:frame-ancestors-directive><a data-x-internal=frame-ancestors-directive href=https://w3c.github.io/webappsec-csp/#frame-ancestors>frame-ancestors</a></code>.<li><p>JavaScript accessible cookies - can be mitigated by ensuring all cookies are <code>httponly</code>.<li><p><code id=cross-origin-opener-policies:dom-localstorage><a href=#dom-localstorage>localStorage</a></code> access to sensitive data.<li><p>Service worker installation.<li><p><a href=https://w3c.github.io/ServiceWorker/#cache>Cache API</a> manipulation or
      access to sensitive data. <a href=#refsSW>[SW]</a><li><p><code>postMessage</code> or <code id=cross-origin-opener-policies:broadcastchannel><a href=#broadcastchannel>BroadcastChannel</a></code> messaging that
      exposes sensitive information.<li><p>Autofill which may not require user interaction for same-origin documents.</ul>

     <p>Developers using <code id=cross-origin-opener-policies:coop-noopener-allow-popups-2><a href=#coop-noopener-allow-popups>noopener-allow-popups</a></code>
     need to make sure that their sensitive applications don't rely on client-side features
     accessible to other same-origin documents, e.g., <code id=cross-origin-opener-policies:dom-localstorage-2><a href=#dom-localstorage>localStorage</a></code> and other client-side storage APIs,
     <code id=cross-origin-opener-policies:broadcastchannel-2><a href=#broadcastchannel>BroadcastChannel</a></code> and related same-origin communication mechanisms. They also need
     to make sure that their server-side endpoints don't return sensitive data to non-navigation
     requests, whose response content is accessible to same-origin
     documents.</p>
    </div>
   </dl>

  

  <p>An <dfn id=cross-origin-opener-policy>opener policy</dfn> consists of:</p>

  <ul><li><p>A <dfn id=coop-struct-value>value</dfn>, which is an <a href=#cross-origin-opener-policy-value id=cross-origin-opener-policies:cross-origin-opener-policy-value>opener policy value</a>, initially "<code id=cross-origin-opener-policies:coop-unsafe-none><a href=#coop-unsafe-none>unsafe-none</a></code>".<li><p>A <dfn id=coop-struct-report-endpoint>reporting endpoint</dfn>, which is string or
   null, initially null.<li><p>A <dfn id=coop-struct-report-only-value>report-only value</dfn>, which is an <a href=#cross-origin-opener-policy-value id=cross-origin-opener-policies:cross-origin-opener-policy-value-2>opener policy value</a>, initially "<code id=cross-origin-opener-policies:coop-unsafe-none-2><a href=#coop-unsafe-none>unsafe-none</a></code>".<li><p>A <dfn id=coop-struct-report-only-endpoint>report-only reporting endpoint</dfn>,
   which is a string or null, initially null.</ul>
```
