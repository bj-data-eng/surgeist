# HTML hosted initial Document, global, agent and settings clauses

Bounded source retention for the native hosted lifecycle contribution. The
original HTML definitions and algorithms below remain authoritative as source;
the separately selected Rust/native execution projection does not rewrite them.
This reference establishes no hosted implementation or JavaScript execution.

## Source basis and attribution

Source: [immutable WHATWG HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/),
revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026).
Extracted offline on 16 October 2026 from the already-retained complete HTML,
14,930,614 bytes, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
No new retrieval, edition, executable or software acquisition is used.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is
[Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/).
The [complete WHATWG license](../licenses/whatwg/LICENSE) and
[pinned HTML attribution](../licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11)
apply. The license SHA-256 is
`85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`.
The upstream BSD 3-Clause provision applies to portions incorporated into source
code, not as an alternative license for retained specification prose.

Changes by Surgeist on 16 October 2026: bounded offline extraction, explanatory
headings/provenance and absolute same-edition fragment links. Source bodies are
literal semantic HTML in Markdown. Every original word, condition, branch,
ordering, ID, definition/list boundary, note, example and source issue marker is
preserved. Only `href` values beginning with `#` are rewritten to the immutable
snapshot, with normal quoted attribute spelling. No source script is executed;
no endorsement or complete browser-conformance claim is implied.

The original fragments below, in ascending source order and joined by one extra
LF, have SHA-256 `03a314e0aa5534a3fc2e1aa15d51c876f718912479ebd33074937b07e7214148`.
The filename suffix identifies that bounded original input, not the final file.

| Original raw lines | Complete retained unit | Original fragment SHA-256 |
| --- | --- | --- |
| 64436–64444 | Obtain a site, including the opaque-origin branch | `b157fe021b101b600ec39c4250934b1e58f8b988b1b88c24069828f854b01267` |
| 65437–65473 | Embedder-policy values, compatibility and record defaults | `4e0b0950fda128a3b71869de59b0c761b9101e68757e7d23dab6772392c83d79` |
| 65797–65810 | Complete iframe referrer-policy selection and adjacent note | `0b11ca16bf70c580e1999dc552fb2016643f0569503538e9dead0ac7423b1a5c` |
| 65813–65842 | Policy-container fields and complete clone algorithm | `3febded3c0f96cb2bca171dcbe0f8d88198fc8e8f5b6f9be6df0e64bdecda54b` |
| 66211–66227 | Window associated Document, browsing context and navigable | `e4be75f2fc59dd81c0eaf4f7f6d1d6f60498cc26bc45eb8cd51cb837b3aab5a2` |
| 66833–66900 | Complete Window environment-settings setup, including all branches | `f517d33e7d3eb2e419fd01ba58a2f696a3133849d5fe0e63c6f184c53a121173` |
| 66905–66927 | WindowProxy definition, internal-method rule and wrapped Window slot | `588efb00968b8a047b6fb5fa769f32d8d666f5a584a3992b7ccc0eaac4d1de00` |
| 71587–71649 | Browsing-context fields, active Window, relations and notes | `ec5d04a8aa051217857e97a625c0be238c975bf0eecf806ad0a099ba375991cf` |
| 71651–71727 | Complete BC/Document and top-level BC/Document creation | `c4deb9bc20d0a0df83e91052ae15d2b23616e7bae4cd885be895de749d61d70c` |
| 71743–71760 | Complete determine-origin algorithm and shared-origin note | `d76f3a964fc1e8e0560d9ab24debcfde9660fc179eebb8dcef7f82c190cc1a57` |
| 71829–71916 | Group fields, complete creation/append/remove and lifetime clauses | `32fe046e23288cb29ce1b99a49d4b76e48d1ea02f0e13a88de847e4a2d6bbe77` |
| 75414–75428 | Complete html/head/body skeleton population | `ab6ea3b653fd2ea2036d2cfe5a50e00af6260ca5ece57c44aedcc7f7ab5b97d5` |
| 75686–75722 | Loaded-time state and complete finish-loading algorithm | `737be51a65880595b86e6e9eeeda2bcab0e276b76c69360b975ae5629ef7b723` |
| 77310–77451 | Agent kinds, create-agent, relevant agent, cluster fields and complete similar-origin acquisition | `0cac919ce87a712b5ce87e9fc1731c6b6e14ae4206896a3a443501f4b03a93b6` |
| 77544–77716 | Environment/settings/global fields and complete create-new-realm algorithm | `d4ff1361bf0727ca271536e96f1f58b4c41664d50937b328fdbcb003487391dd` |
| 78096–78157 | Relevant realm/settings/global and complete scripting enabled/disabled clauses | `cc1d10dcecc20ff267f42bb782b6272516c6f9007d8e000920bd199ecfe950fc` |
| 81077–81189 | Complete event-loop definitions, queues, initial state and same-loop Windows | `c8823100e7b6976017a2874c796a9709850237a673cbc2506bf6a2476cfb9631` |

## Reused source and execution boundary

The separate [navigable lifecycle reference](html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md)
retains the top-level/child caller, actual entry/Document-state/nested-history
definitions, destruction callers and make-active. Reuse
[sandbox flags and creation-sandbox selection](html-sandboxing--877d6146487f.md),
[Document defaults and ancestor-origin algorithms](html-language-foundations--877d6146487f--b55e73c31978.md),
[registry defaults, Window.customElements and lookup](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md),
[active and fully-active relations](html-hosted-document-activity--877d6146487f--f8564ea4758e.md),
[task/global/element queue algorithms](html-element-task-association--877d6146487f.md)
and [Permissions Policy native creation](permissions-policy-native-creation--29398046f1e0.md).
Those retained bodies are not replaced by empty/default guessed results.

The constructor bodies retain all their branches even though the first native
creation profile has no opener and null creator/embedder. In particular,
InitializeHostDefinedRealm, JavaScript execution-context stack operations,
SharedArrayBuffer deletion, realm/global-this customization and [[HostDefined]]
remain literal source below. They are external/JavaScript-only operations outside
the admitted native execution projection, not excised steps or claims that
non-executable Rust identity records implement ECMAScript. The Architecture
Coordinator and owning issue select that distinct native projection. This
reference does not pin/acquire ECMAScript or implement JavaScript/WindowProxy
exotic internal methods. It retains the WindowProxy definition and [[Window]]
relation used by the constructor and make-active; its other internal algorithms
remain separate source dependencies if execution reaches them.

Creating a real agent also creates its event loop. The full event-loop defining
state is retained here, including task fields already available in the smaller
task reference so this definition section has no omitted middle. The continually
running processing model at raw81279 onward, microtask checkpoint and spin/wait
algorithms are outside this excerpt. Native task processing, destruction waits
or other later execution must retain/qualify the actual reached procedure; these
definitions do not certify a complete browser event loop.

HR-Time unsafe shared current time and coarsen time remain external constructor
calls. The source-defined settings cross-origin-isolated capability is retained
in full. A host clock/provider needs its actual source/contract and result;
neither zero nor a generic Rust timestamp is an implicit acknowledgment. No
HR-Time algorithm source or clock implementation is selected by this retention.

The initial no-opener constructor does not run fresh-top-level navigation,
replacement or Document reactivation. Initial registry identity, real history
state, source-ordered creation notifications and later top-level destruction
remain separate obligations. The destruction caller already retained in the
lifecycle reference invokes complete Document/descendant destruction
(raw75871–75923), which calls abort-Document (75927–75949) and unloading cleanup
(75829–75869), and queues real navigation/traversal work. Those bodies and the
reached history application/Navigation API helpers require a separate bounded
retention followup before that destruction execution is accepted; they are not
waived because the initial profile has no JavaScript. External WebDriver,
cleanup, clock and platform services retain their actual availability contracts.
This file is not a transitive closure of its links.

## Obtain a site, including the opaque-origin branch

<!-- selected-source-fragment:64436-64444 -->
  <div data-algorithm="">
  <p>To <dfn id=obtain-a-site data-export="">obtain a site</dfn>, given an origin <var>origin</var>, run these steps:</p>

  <ol><li><p>If <var>origin</var> is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-opaque" id=sites:concept-origin-opaque-2>opaque origin</a>,
   then return <var>origin</var>.<li><p>If <var>origin</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-host" id=sites:concept-origin-host>host</a>'s <a id=sites:registrable-domain href=https://url.spec.whatwg.org/#host-registrable-domain data-x-internal=registrable-domain>registrable
   domain</a> is null, then return (<var>origin</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-scheme" id=sites:concept-origin-scheme>scheme</a>, <var>origin</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-host" id=sites:concept-origin-host-2>host</a>).<li><p>Return (<var>origin</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-scheme" id=sites:concept-origin-scheme-2>scheme</a>,
   <var>origin</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-host" id=sites:concept-origin-host-3>host</a>'s <a id=sites:registrable-domain-2 href=https://url.spec.whatwg.org/#host-registrable-domain data-x-internal=registrable-domain>registrable
   domain</a>).</ol>
  </div>


## Embedder-policy values, compatibility and record defaults

<!-- selected-source-fragment:65437-65473 -->
  <p>An <dfn id=embedder-policy-value data-export="">embedder policy value</dfn> is one of three strings that controls the fetching
  of cross-origin resources without explicit permission from resource owners.</p>

  <dl><dt>"<dfn data-dfn-for="embedder policy value" id=coep-unsafe-none data-export=""><code>unsafe-none</code></dfn>"<dd><p>This is the default value. When this value is used, cross-origin resources can be fetched
   without giving explicit permission through the <a id=coep:cors-protocol href=https://fetch.spec.whatwg.org/#http-cors-protocol data-x-internal=cors-protocol>CORS protocol</a> or the
   `<code id=coep:cross-origin-resource-policy><a data-x-internal=cross-origin-resource-policy href=https://fetch.spec.whatwg.org/#http-cross-origin-resource-policy>Cross-Origin-Resource-Policy</a></code>` header.<dt>"<dfn data-dfn-for="embedder policy value" id=coep-require-corp data-export=""><code>require-corp</code></dfn>"<dd><p>When this value is used, fetching cross-origin resources requires the server's
   explicit permission through the <a id=coep:cors-protocol-2 href=https://fetch.spec.whatwg.org/#http-cors-protocol data-x-internal=cors-protocol>CORS protocol</a> or the
   `<code id=coep:cross-origin-resource-policy-2><a data-x-internal=cross-origin-resource-policy href=https://fetch.spec.whatwg.org/#http-cross-origin-resource-policy>Cross-Origin-Resource-Policy</a></code>` header.<dt>"<dfn data-dfn-for="embedder policy value" id=coep-credentialless data-export=""><code>credentialless</code></dfn>"<dd><p>When this value is used, fetching cross-origin no-CORS resources omits credentials. In
   exchange, an explicit `<code id=coep:cross-origin-resource-policy-3><a data-x-internal=cross-origin-resource-policy href=https://fetch.spec.whatwg.org/#http-cross-origin-resource-policy>Cross-Origin-Resource-Policy</a></code>` header is not required. Other
   requests sent with credentials require the server's explicit permission through the <a id=coep:cors-protocol-3 href=https://fetch.spec.whatwg.org/#http-cors-protocol data-x-internal=cors-protocol>CORS
   protocol</a> or the `<code id=coep:cross-origin-resource-policy-4><a data-x-internal=cross-origin-resource-policy href=https://fetch.spec.whatwg.org/#http-cross-origin-resource-policy>Cross-Origin-Resource-Policy</a></code>` header.</dl>

  <div class=warning>
   <p>Before supporting "<code id=coep:coep-credentialless><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#coep-credentialless">credentialless</a></code>", implementers are
   strongly encouraged to support both:

   <ul class=brief><li><a href=https://wicg.github.io/private-network-access/>Private Network Access</a><li><a href=https://github.com/annevk/orb>Opaque Response Blocking</a></ul>

   <p>Otherwise, it would allow attackers to leverage the client's network position to read non
   public resources, using the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-cross-origin-isolated-capability" id=coep:concept-settings-object-cross-origin-isolated-capability>cross-origin isolated
   capability</a>.</p>
  </div>

  <div data-algorithm="">
  <p>An <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#embedder-policy-value" id=coep:embedder-policy-value>embedder policy value</a> is <dfn id=compatible-with-cross-origin-isolation>compatible with cross-origin isolation</dfn> if
  it is "<code id=coep:coep-credentialless-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#coep-credentialless">credentialless</a></code>" or "<code id=coep:coep-require-corp><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#coep-require-corp">require-corp</a></code>".</p>
  </div>

  

  <p>An <dfn id=embedder-policy data-export="">embedder policy</dfn> consists of:</p>

  <ul><li><p>A <dfn data-dfn-for="embedder policy" id=embedder-policy-value-2 data-export="">value</dfn>, which is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#embedder-policy-value" id=coep:embedder-policy-value-2>embedder policy value</a>, initially "<code id=coep:coep-unsafe-none><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#coep-unsafe-none">unsafe-none</a></code>".<li><p>A <dfn data-dfn-for="embedder policy" id=embedder-policy-reporting-endpoint data-export="">reporting endpoint</dfn> string, initially the empty string.<li><p>A <dfn data-dfn-for="embedder policy" id=embedder-policy-report-only-value data-export="">report-only value</dfn>, which is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#embedder-policy-value" id=coep:embedder-policy-value-3>embedder policy value</a>, initially
   "<code id=coep:coep-unsafe-none-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#coep-unsafe-none">unsafe-none</a></code>".<li><p>A <dfn data-dfn-for="embedder
   policy" id=embedder-policy-report-only-reporting-endpoint data-export="">report-only reporting endpoint</dfn> string, initially the empty
   string.</ul>



## Complete iframe referrer-policy selection and adjacent note

<!-- selected-source-fragment:65797-65810 -->
  <h4 id=iframe-element-referrer-policy><span class=secno>7.1.6</span> <code id=iframe-element-referrer-policy:the-iframe-element><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element">iframe</a></code> element referrer policy<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#iframe-element-referrer-policy" class=self-link></a></h4>

  <div data-algorithm="">
  <p>To <dfn id=determining-the-iframe-element-referrer-policy>determine the
  <code>iframe</code> element referrer policy</dfn> given an element-or-null
  <var>embedder</var>:</p>

  <ol><li><p>If <var>embedder</var> is an <code id=iframe-element-referrer-policy:the-iframe-element-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element">iframe</a></code> element, then return
   <var>embedder</var>'s <code id=iframe-element-referrer-policy:attr-iframe-referrerpolicy><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-referrerpolicy">referrerpolicy</a></code> attribute's
   state's corresponding keyword.<li><p>Return the empty string.</ol>
  </div>

  <p class=note>This is used for allowing masking of some <a href=https://dom.spec.whatwg.org/#concept-document-origin id=iframe-element-referrer-policy:concept-document-origin data-x-internal=concept-document-origin>origins</a> in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#internal-ancestor-origin-objects-list-creation-steps" id=iframe-element-referrer-policy:internal-ancestor-origin-objects-list-creation-steps>internal ancestor origin objects list
  creation steps</a>.</p>


## Policy-container fields and complete clone algorithm

<!-- selected-source-fragment:65813-65842 -->
  <h4 id=policy-containers><span class=secno>7.1.7</span> Policy containers<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-containers" class=self-link></a></h4>

  <p>A <dfn id=policy-container data-export="">policy container</dfn> is a <a id=policy-containers:struct href=https://infra.spec.whatwg.org/#struct data-x-internal=struct>struct</a> containing policies that apply to
  a <code id=policy-containers:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>, a <code id=policy-containers:workerglobalscope><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#workerglobalscope">WorkerGlobalScope</a></code>, or a <code id=policy-containers:workletglobalscope><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#workletglobalscope">WorkletGlobalScope</a></code>.
  It has the following <a href=https://infra.spec.whatwg.org/#struct-item id=policy-containers:struct-item data-x-internal=struct-item>items</a>:</p>

  

  <ul><li><p>A <dfn data-dfn-for="policy container" id=policy-container-csp-list data-export="">CSP list</dfn>,
   which is a <a href=https://w3c.github.io/webappsec-csp/#csp-list id=policy-containers:concept-csp-list data-x-internal=concept-csp-list>CSP list</a>. It is initially empty.<li><p>An <dfn data-dfn-for="policy container" id=policy-container-embedder-policy data-export="">embedder
   policy</dfn>, which is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#embedder-policy" id=policy-containers:embedder-policy>embedder policy</a>. It is initially a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#embedder-policy" id=policy-containers:embedder-policy-2>embedder
   policy</a>.<li><p>A <dfn data-dfn-for="policy container" id=policy-container-referrer-policy data-export="">referrer
   policy</dfn>, which is a <a id=policy-containers:referrer-policy href=https://w3c.github.io/webappsec-referrer-policy/#referrer-policy data-x-internal=referrer-policy>referrer policy</a>. It is initially the <a id=policy-containers:default-referrer-policy href=https://w3c.github.io/webappsec-referrer-policy/#default-referrer-policy data-x-internal=default-referrer-policy>default referrer
   policy</a>.<li><p>An <dfn data-dfn-for="policy container" id=policy-container-integrity-policy data-export="">integrity
   policy</dfn>, which is an <a href=https://w3c.github.io/webappsec-subresource-integrity/#integrity-policy id=policy-containers:integrity-policy data-x-internal=integrity-policy>integrity policy</a>, initially a new
   <a href=https://w3c.github.io/webappsec-subresource-integrity/#integrity-policy id=policy-containers:integrity-policy-2 data-x-internal=integrity-policy>integrity policy</a>.<li><p>A <dfn data-dfn-for="policy container" id=policy-container-report-only-integrity-policy data-export="">report-only integrity policy</dfn>, which
   is an <a href=https://w3c.github.io/webappsec-subresource-integrity/#integrity-policy id=policy-containers:integrity-policy-3 data-x-internal=integrity-policy>integrity policy</a>, initially a new <a href=https://w3c.github.io/webappsec-subresource-integrity/#integrity-policy id=policy-containers:integrity-policy-4 data-x-internal=integrity-policy>integrity policy</a>.</ul>

  <p class=XXX>Move other policies into the policy container.</p>

  <div data-algorithm="">
  <p>To <dfn id=clone-a-policy-container data-export="">clone a policy container</dfn> given a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container" id=policy-containers:policy-container>policy container</a>
  <var>policyContainer</var>:</p>

  <ol><li><p>Let <var>clone</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container" id=policy-containers:policy-container-2>policy container</a>.<li><p><a href=https://infra.spec.whatwg.org/#list-iterate id=policy-containers:list-iterate data-x-internal=list-iterate>For each</a> <var>policy</var> in
   <var>policyContainer</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-csp-list" id=policy-containers:policy-container-csp-list>CSP list</a>, <a href=https://infra.spec.whatwg.org/#list-append id=policy-containers:list-append data-x-internal=list-append>append</a> a copy of <var>policy</var> into <var>clone</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-csp-list" id=policy-containers:policy-container-csp-list-2>CSP list</a>.<li><p>Set <var>clone</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-embedder-policy" id=policy-containers:policy-container-embedder-policy>embedder
   policy</a> to a copy of <var>policyContainer</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-embedder-policy" id=policy-containers:policy-container-embedder-policy-2>embedder policy</a>.<li><p>Set <var>clone</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-referrer-policy" id=policy-containers:policy-container-referrer-policy>referrer
   policy</a> to <var>policyContainer</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-referrer-policy" id=policy-containers:policy-container-referrer-policy-2>referrer policy</a>.<li><p>Set <var>clone</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-integrity-policy" id=policy-containers:policy-container-integrity-policy>integrity
   policy</a> to a copy of <var>policyContainer</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-integrity-policy" id=policy-containers:policy-container-integrity-policy-2>integrity policy</a>.<li><p>Return <var>clone</var>.</ol>
  </div>


## Window associated Document, browsing context and navigable

<!-- selected-source-fragment:66211-66227 -->
  <p>The <code id=the-window-object:window-5><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> object has an <dfn id=concept-document-window data-export="">associated
  <code>Document</code></dfn>, which is a <code id=the-window-object:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> object. It is set when the
  <code id=the-window-object:window-6><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> object is created, and only ever changed during <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate" id=the-window-object:navigate>navigation</a> from the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-initial-about:blank" id=the-window-object:is-initial-about:blank>initial
  <code>about:blank</code></a> <code id=the-window-object:document-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>.</p>

  <div data-algorithm="">
  <p>A <code id=the-window-object:window-7><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code>'s <dfn data-dfn-for=Window id=window-bc data-export="">browsing context</dfn> is
  its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=the-window-object:concept-document-window>associated <code>Document</code></a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=the-window-object:concept-document-bc>browsing context</a>. <span class=note>It is either null or a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=the-window-object:browsing-context>browsing context</a>.</span></p>
  </div>

  <div data-algorithm="">
  <p>A <code id=the-window-object:window-8><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code>'s <dfn data-dfn-for=Window id=window-navigable data-export="">navigable</dfn> is
  the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=the-window-object:navigable>navigable</a> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=the-window-object:nav-document>active document</a> is the
  <code id=the-window-object:window-9><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=the-window-object:concept-document-window-2>associated
  <code>Document</code></a>'s, or null if there is no such <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=the-window-object:navigable-2>navigable</a>.</p>
  </div>


## Complete Window environment-settings setup, including all branches

<!-- selected-source-fragment:66833-66900 -->
  <h5 id=script-settings-for-window-objects><span class=secno>7.2.2.6</span> Script settings for <code id=script-settings-for-window-objects:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> objects<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#script-settings-for-window-objects" class=self-link></a></h5>

  <div data-algorithm="">
  <p>To <dfn id=set-up-a-window-environment-settings-object>set up a window environment settings object</dfn>, given a <a id=script-settings-for-window-objects:url href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a>
  <var>creationURL</var>, a <a id=script-settings-for-window-objects:javascript-execution-context href=https://tc39.es/ecma262/#sec-execution-contexts data-x-internal=javascript-execution-context>JavaScript execution context</a> <var>execution context</var>,
  null or an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=script-settings-for-window-objects:environment>environment</a> <var>reservedEnvironment</var>, a <a id=script-settings-for-window-objects:url-2 href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a>
  <var>topLevelCreationURL</var>, and an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=script-settings-for-window-objects:concept-origin>origin</a> <var>topLevelOrigin</var>, run these
  steps:</p>

  <ol><li><p>Let <var>realm</var> be the value of <var>execution context</var>'s Realm
   component.<li><p>Let <var>window</var> be <var>realm</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-realm-global" id=script-settings-for-window-objects:concept-realm-global>global
   object</a>.<li>
    <p>Let <var>settings object</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=script-settings-for-window-objects:environment-settings-object>environment settings object</a> whose
    algorithms are defined as follows:</p>

    <dl><dt>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#realm-execution-context" id=script-settings-for-window-objects:realm-execution-context>realm execution context</a><dd>
      <p>Return <var>execution context</var>.</p>
     <dt>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-module-map" id=script-settings-for-window-objects:concept-settings-object-module-map>module map</a><dd>
      <p>Return the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-module-map" id=script-settings-for-window-objects:concept-document-module-map>module map</a> of
      <var>window</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=script-settings-for-window-objects:concept-document-window>associated
      <code>Document</code></a>.</p>
     <dt>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#api-base-url" id=script-settings-for-window-objects:api-base-url>API base URL</a><dd>
      <p>Return the current <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-base-url" id=script-settings-for-window-objects:document-base-url>base URL</a> of <var>window</var>'s
      <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=script-settings-for-window-objects:concept-document-window-2>associated <code>Document</code></a>.</p>
     <dt>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-origin" id=script-settings-for-window-objects:concept-settings-object-origin>origin</a><dd>
      <p>Return the <a href=https://dom.spec.whatwg.org/#concept-document-origin id=script-settings-for-window-objects:concept-document-origin data-x-internal=concept-document-origin>origin</a> of <var>window</var>'s
      <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=script-settings-for-window-objects:concept-document-window-3>associated <code>Document</code></a>.</p>
     <dt>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-has-cross-site-ancestor" id=script-settings-for-window-objects:concept-settings-object-has-cross-site-ancestor>has cross-site
     ancestor</a><dd>
      <ol><li><p>If <var>window</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=script-settings-for-window-objects:navigable>navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=script-settings-for-window-objects:nav-parent>parent</a> is null, then return false.<li><p>Let <var>parentDocument</var> be <var>window</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=script-settings-for-window-objects:navigable-2>navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=script-settings-for-window-objects:nav-parent-2>parent</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=script-settings-for-window-objects:nav-document>active
       document</a>.<li><p>If <var>parentDocument</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object" id=script-settings-for-window-objects:relevant-settings-object>relevant settings object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-has-cross-site-ancestor" id=script-settings-for-window-objects:concept-settings-object-has-cross-site-ancestor-2>has cross-site ancestor</a> is
       true, then return true.<li><p>If <var>parentDocument</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=script-settings-for-window-objects:concept-document-origin-2 data-x-internal=concept-document-origin>origin</a>
       is not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-site" id=script-settings-for-window-objects:same-site>same site</a> with <var>window</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=script-settings-for-window-objects:concept-document-window-4>associated <code>Document</code></a>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=script-settings-for-window-objects:concept-document-origin-3 data-x-internal=concept-document-origin>origin</a>, then return true.<li><p>Return false.</ol>
     <dt>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-policy-container" id=script-settings-for-window-objects:concept-settings-object-policy-container>policy container</a><dd>
      <p>Return the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-policy-container" id=script-settings-for-window-objects:concept-document-policy-container>policy container</a> of
      <var>window</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=script-settings-for-window-objects:concept-document-window-5>associated
      <code>Document</code></a>.</p>
     <dt>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-cross-origin-isolated-capability" id=script-settings-for-window-objects:concept-settings-object-cross-origin-isolated-capability>cross-origin
     isolated capability</a><dd>
      <p>Return true if both of the following hold, and false otherwise:</p>
      <ul><li><p><var>realm</var>'s <a id=script-settings-for-window-objects:agent-cluster href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-cross-origin-isolation" id=script-settings-for-window-objects:agent-cluster-cross-origin-isolation>cross-origin-isolation mode</a> is "<code id=script-settings-for-window-objects:cross-origin-isolation-concrete><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-concrete">concrete</a></code>", and<li><p><var>window</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=script-settings-for-window-objects:concept-document-window-6>associated
       <code>Document</code></a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#allowed-to-use" id=script-settings-for-window-objects:allowed-to-use>allowed to use</a> the "<code id=script-settings-for-window-objects:cross-origin-isolated-feature><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolated-feature">cross-origin-isolated</a></code>" feature.</ul>
     <dt>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-time-origin" id=script-settings-for-window-objects:concept-settings-object-time-origin>time origin</a><dd>
      <p>Return <var>window</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=script-settings-for-window-objects:concept-document-window-7>associated
      <code>Document</code></a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#load-timing-info" id=script-settings-for-window-objects:load-timing-info>load timing info</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-start-time" id=script-settings-for-window-objects:navigation-start-time>navigation start
      time</a>.</p>
     </dl>
   <li>
    <p>If <var>reservedEnvironment</var> is non-null:</p>

    <ol><li><p>Set <var>settings object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-id" id=script-settings-for-window-objects:concept-environment-id>id</a> to
     <var>reservedEnvironment</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-id" id=script-settings-for-window-objects:concept-environment-id-2>id</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-target-browsing-context" id=script-settings-for-window-objects:concept-environment-target-browsing-context>target browsing context</a> to
     <var>reservedEnvironment</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-target-browsing-context" id=script-settings-for-window-objects:concept-environment-target-browsing-context-2>target browsing context</a>, and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-active-service-worker" id=script-settings-for-window-objects:concept-environment-active-service-worker>active service worker</a> to
     <var>reservedEnvironment</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-active-service-worker" id=script-settings-for-window-objects:concept-environment-active-service-worker-2>active service worker</a>.<li>
      <p>Set <var>reservedEnvironment</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-id" id=script-settings-for-window-objects:concept-environment-id-3>id</a> to
      the empty string.</p>

      <p class=note>The identity of the reserved environment is considered to be fully
      transferred to the created <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=script-settings-for-window-objects:environment-settings-object-2>environment settings object</a>. The reserved environment
      is not searchable by the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=script-settings-for-window-objects:environment-2>environment</a>’s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-id" id=script-settings-for-window-objects:concept-environment-id-4>id</a> from this point on.</p>
     </ol>
   <li><p>Otherwise, set <var>settings object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-id" id=script-settings-for-window-objects:concept-environment-id-5>id</a> to a new unique opaque string, <var>settings
   object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-target-browsing-context" id=script-settings-for-window-objects:concept-environment-target-browsing-context-3>target browsing
   context</a> to null, and <var>settings object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-active-service-worker" id=script-settings-for-window-objects:concept-environment-active-service-worker-3>active service worker</a> to null.<li><p>Set <var>settings object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-creation-url" id=script-settings-for-window-objects:concept-environment-creation-url>creation
   URL</a> to <var>creationURL</var>, <var>settings object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-top-level-creation-url" id=script-settings-for-window-objects:concept-environment-top-level-creation-url>top-level creation
   URL</a> to <var>topLevelCreationURL</var>, and <var>settings object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-top-level-origin" id=script-settings-for-window-objects:concept-environment-top-level-origin>top-level
   origin</a> to <var>topLevelOrigin</var>.<li><p>Set <var>realm</var>'s [[HostDefined]] field to <var>settings object</var>.</ol>
  </div>


## WindowProxy definition, internal-method rule and wrapped Window slot

<!-- selected-source-fragment:66905-66927 -->
  <h4 id=the-windowproxy-exotic-object><span class=secno>7.2.3</span> The <code id=the-windowproxy-exotic-object:windowproxy><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> exotic object<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-windowproxy-exotic-object" class=self-link></a></h4>

  <p>A <dfn id=windowproxy data-dfn-type=interface><code>WindowProxy</code></dfn> is an exotic object that wraps a
  <code id=the-windowproxy-exotic-object:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> ordinary object, indirecting most operations through to the wrapped object.
  Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=the-windowproxy-exotic-object:browsing-context>browsing context</a> has an associated <code id=the-windowproxy-exotic-object:windowproxy-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> object. When the
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=the-windowproxy-exotic-object:browsing-context-2>browsing context</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate" id=the-windowproxy-exotic-object:navigate>navigated</a>, the <code id=the-windowproxy-exotic-object:window-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code>
  object wrapped by the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=the-windowproxy-exotic-object:browsing-context-3>browsing context</a>'s associated <code id=the-windowproxy-exotic-object:windowproxy-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> object
  is changed.</p>

  

  <p>The <code id=the-windowproxy-exotic-object:windowproxy-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> exotic object must use the ordinary internal methods except where
  it is explicitly specified otherwise below.</p>

  <p>There is no <code id=the-windowproxy-exotic-object:windowproxy-5><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> <a id=the-windowproxy-exotic-object:interface-object href=https://webidl.spec.whatwg.org/#dfn-interface-object data-x-internal=interface-object>interface object</a>.</p>

  <p>Every <code id=the-windowproxy-exotic-object:windowproxy-6><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> object has a <dfn id=concept-windowproxy-window data-export="">[[Window]]</dfn> internal slot representing the wrapped <code id=the-windowproxy-exotic-object:window-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> object.</p>

  <p class=note>Although <code id=the-windowproxy-exotic-object:windowproxy-7><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> is named as a "proxy", it does not do
  polymorphic dispatch on its target's internal methods as a real proxy would, due to a desire to
  reuse machinery between <code id=the-windowproxy-exotic-object:windowproxy-8><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> and <code id=the-windowproxy-exotic-object:location><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#location">Location</a></code> objects. As long as the
  <code id=the-windowproxy-exotic-object:window-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> object remains an ordinary object this is unobservable and can be implemented
  either way.</p>


## Browsing-context fields, active Window, relations and notes

<!-- selected-source-fragment:71587-71649 -->
  <h4 id=windows><span class=secno>7.3.2</span> Browsing contexts<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windows" class=self-link></a></h4>

  <p>A <dfn id=browsing-context data-export="">browsing context</dfn> is a programmatic representation of a series of documents,
  multiple of which can live within a single <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=windows:navigable>navigable</a>. Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context>browsing
  context</a> has a corresponding <code id=windows:windowproxy><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> object, as well as the following:</p>

  <ul><li><p>An <dfn id=active-document>active document</dfn>, a <code id=windows:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> or null, initially null.<li><p>An <dfn id=opener-browsing-context data-export="">opener browsing context</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-2>browsing context</a> or null,
   initially null.<li><p>An <dfn id=opener-origin-at-creation>opener origin at creation</dfn>, an
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=windows:concept-origin>origin</a> or null, initially null.</p>

   <li>
    <p>An <dfn id=is-popup>is popup</dfn> boolean, initially false.</p>

    <p class=note>The only mandatory impact in this specification of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-popup" id=windows:is-popup>is popup</a> is on
    the <code id=windows:dom-barprop-visible><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-barprop-visible">visible</a></code> getter of the relevant
    <code id=windows:barprop><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#barprop">BarProp</a></code> objects. However, user agents might also use it for <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-traversal-ui">user interface considerations</a>.</p>
   <li><p>An <dfn id=is-auxiliary>is auxiliary</dfn> boolean, initially false.<li><p>An <dfn id=browsing-context-initial-url>initial URL</dfn>, a <a id=windows:url href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a> or
   null, initially null.<li><p>A <dfn id=virtual-browsing-context-group-id>virtual browsing context group ID</dfn>
   integer, initially 0. This is used by <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#coop-struct-report-only-value" id=windows:coop-struct-report-only-value>opener policy
   reporting</a>, to keep track of the browsing context group switches that would have happened
   if the report-only policy had been enforced.</ul>

  <div data-algorithm="">
  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-3>browsing context</a>'s <dfn id=active-window>active window</dfn> is its <code id=windows:windowproxy-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code>
  object's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-windowproxy-window" id=windows:concept-windowproxy-window>[[Window]]</a> internal slot value.</p>
  </div>

  <div data-algorithm="">
  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-4>browsing context</a>'s <dfn data-dfn-for="browsing context" id=bc-traversable data-export="">top-level traversable</dfn> is its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-document" id=windows:active-document>active document</a>'s
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=windows:node-navigable>node navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-top" id=windows:nav-top>top-level traversable</a>.</p>
  </div>

  <div data-algorithm="">
  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-5>browsing context</a> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-auxiliary" id=windows:is-auxiliary>is auxiliary</a> is true is known as an <dfn id=auxiliary-browsing-context data-export="">auxiliary browsing context</dfn>. Auxiliary browsing contexts are always <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=windows:top-level-browsing-context>top-level browsing contexts</a>.</p>
  </div>

  <p class=XXX>It's unclear whether a separate <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-auxiliary" id=windows:is-auxiliary-2>is auxiliary</a> concept is necessary. In
  <a href=https://github.com/whatwg/html/issues/5680>issue #5680</a>, it is indicated that we may
  be able to simplify this by using whether or not the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#opener-browsing-context" id=windows:opener-browsing-context>opener browsing context</a> is
  null.</p>

  <p id=warning-avoid-using-bcs class=warning>Modern specifications should avoid using the
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-6>browsing context</a> concept in most cases, unless they are dealing with the subtleties
  of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group-switches-due-to-cross-origin-opener-policy">browsing context
  group switches</a> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-map" id=windows:agent-cluster-map>agent cluster allocation</a>. Instead,
  the <code id=windows:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=windows:navigable-2>navigable</a> concepts are usually more appropriate.</p>

  <hr>

  <p>A <dfn data-export="" data-lt="browsing context" id=concept-document-bc data-dfn-for=Document><code>Document</code>'s browsing context</dfn> is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-7>browsing context</a> or null,
  initially null.</p>

  <p class=note>A <code id=windows:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> does not necessarily have a non-null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=windows:concept-document-bc>browsing context</a>. In particular, data mining tools are likely
  to never instantiate browsing contexts. A <code id=windows:document-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> created using an API such as <code id=windows:dom-domimplementation-createdocument><a data-x-internal=dom-domimplementation-createdocument href=https://dom.spec.whatwg.org/#dom-domimplementation-createdocument>createDocument()</a></code> never has a non-null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=windows:concept-document-bc-2>browsing context</a>. And the <code id=windows:document-5><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> originally
  created for an <code id=windows:the-iframe-element><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element">iframe</a></code> element, which has since been <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#remove-an-element-from-a-document" id=windows:remove-an-element-from-a-document>removed from the document</a>, has no associated browsing context, since that
  browsing context was <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document" id=windows:destroy-a-document>nulled out</a>.</p>

  <p class=note>In general, there is a 1-to-1 mapping from the <code id=windows:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> object to the
  <code id=windows:document-6><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> object, as long as the <code id=windows:document-7><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> object has a non-null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=windows:concept-document-bc-3>browsing context</a>. There is one exception. A
  <code id=windows:window-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> can be reused for the presentation of a second <code id=windows:document-8><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> in the
  same <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-8>browsing context</a>, such that the mapping is then 1-to-2. This occurs when a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-9>browsing context</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate" id=windows:navigate>navigated</a> from the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-initial-about:blank" id=windows:is-initial-about:blank>initial <code>about:blank</code></a> <code id=windows:document-9><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> to
  another, which will be done with <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigationhistorybehavior-replace" id=windows:navigationhistorybehavior-replace>replacement</a>.</p>


## Complete BC/Document and top-level BC/Document creation

<!-- selected-source-fragment:71651-71727 -->
  <h5 id=creating-browsing-contexts><span class=secno>7.3.2.1</span> Creating browsing contexts<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-browsing-contexts" class=self-link></a></h5>

  <div data-algorithm="">
  <p>To <dfn id=creating-a-new-browsing-context>create a new browsing context and document</dfn>, given null or a <code id=creating-browsing-contexts:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>
  object <var>creator</var>, null or an element <var>embedder</var>, and a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=creating-browsing-contexts:browsing-context-group>browsing context
  group</a> <var>group</var>:</p>

  

  <ol><li><p>Let <var>browsingContext</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=creating-browsing-contexts:browsing-context>browsing context</a>.<li><p>Let <var>unsafeContextCreationTime</var> be the
   <a id=creating-browsing-contexts:unsafe-shared-current-time href=https://w3c.github.io/hr-time/#dfn-unsafe-shared-current-time data-x-internal=unsafe-shared-current-time>unsafe shared current time</a>.<li><p>Let <var>creatorOrigin</var> be null.</p>

   <li><p>Let <var>creatorBaseURL</var> be null.<li id=creator-browsing-context>
    <p>If <var>creator</var> is non-null:</p>

    <ol><li><p>Set <var>creatorOrigin</var> to <var>creator</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=creating-browsing-contexts:concept-document-origin data-x-internal=concept-document-origin>origin</a>.<li><p>Set <var>creatorBaseURL</var> to <var>creator</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-base-url" id=creating-browsing-contexts:document-base-url>document base
     URL</a>.<li><p>Set <var>browsingContext</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#virtual-browsing-context-group-id" id=creating-browsing-contexts:virtual-browsing-context-group-id>virtual browsing context group ID</a> to
     <var>creator</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=creating-browsing-contexts:concept-document-bc>browsing context</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bc-tlbc" id=creating-browsing-contexts:bc-tlbc>top-level browsing context</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#virtual-browsing-context-group-id" id=creating-browsing-contexts:virtual-browsing-context-group-id-2>virtual browsing context group ID</a>.</ol>
   <li><p>Let <var>sandboxFlags</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#determining-the-creation-sandboxing-flags" id=creating-browsing-contexts:determining-the-creation-sandboxing-flags>determining the creation sandboxing
   flags</a> given <var>browsingContext</var> and <var>embedder</var>.<li id=about-blank-origin><p>Let <var>origin</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#determining-the-origin" id=creating-browsing-contexts:determining-the-origin>determining the
   origin</a> given <code id=creating-browsing-contexts:about:blank><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#about:blank">about:blank</a></code>, <var>sandboxFlags</var>, and
   <var>creatorOrigin</var>.<li>
    <p>Let <var>permissionsPolicy</var> be the result of <a id=creating-browsing-contexts:creating-a-permissions-policy href=https://w3c.github.io/webappsec-feature-policy/#create-for-navigable data-x-internal=creating-a-permissions-policy>creating a permissions policy</a>
    given <var>embedder</var> and <var>origin</var>. <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsPERMISSIONSPOLICY">[PERMISSIONSPOLICY]</a></p>
   <li><p>Let <var>agent</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#obtain-similar-origin-window-agent" id=creating-browsing-contexts:obtain-similar-origin-window-agent>obtaining a similar-origin window agent</a> given
   <var>origin</var>, <var>group</var>, and false.<li>
    <p>Let <var>realm execution context</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-javascript-realm" id=creating-browsing-contexts:creating-a-new-javascript-realm>creating a new realm</a>
    given <var>agent</var> and the following customizations:</p>

    <ul><li><p>For the global object, create a new <code id=creating-browsing-contexts:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> object.<li><p>For the global <b>this</b> binding, use <var>browsingContext</var>'s
     <code id=creating-browsing-contexts:windowproxy><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> object.</ul>
   <li><p>Let <var>topLevelCreationURL</var> be <code id=creating-browsing-contexts:about:blank-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#about:blank">about:blank</a></code> if <var>embedder</var> is
   null; otherwise <var>embedder</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object" id=creating-browsing-contexts:relevant-settings-object>relevant settings object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-top-level-creation-url" id=creating-browsing-contexts:concept-environment-top-level-creation-url>top-level
   creation URL</a>.<li><p>Let <var>topLevelOrigin</var> be <var>origin</var> if <var>embedder</var> is null;
   otherwise <var>embedder</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object" id=creating-browsing-contexts:relevant-settings-object-2>relevant settings object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-top-level-origin" id=creating-browsing-contexts:concept-environment-top-level-origin>top-level
   origin</a>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#set-up-a-window-environment-settings-object" id=creating-browsing-contexts:set-up-a-window-environment-settings-object>Set up a window environment settings object</a> with <code id=creating-browsing-contexts:about:blank-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#about:blank">about:blank</a></code>,
   <var>realm execution context</var>, null, <var>topLevelCreationURL</var>, and
   <var>topLevelOrigin</var>.<li><p>Let <var>loadTimingInfo</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-load-timing-info" id=creating-browsing-contexts:document-load-timing-info>document load timing info</a> with
   its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-start-time" id=creating-browsing-contexts:navigation-start-time>navigation start time</a> set to the result of calling <a id=creating-browsing-contexts:coarsen-time href=https://w3c.github.io/hr-time/#dfn-coarsen-time data-x-internal=coarsen-time>coarsen time</a>
   with <var>unsafeContextCreationTime</var> and the new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=creating-browsing-contexts:environment-settings-object>environment settings object</a>'s
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-cross-origin-isolated-capability" id=creating-browsing-contexts:concept-settings-object-cross-origin-isolated-capability>cross-origin isolated
   capability</a>.<li>
    <p>Let <var>document</var> be a new <code id=creating-browsing-contexts:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>, with:</p>

    <dl class=props><dt><a href=https://dom.spec.whatwg.org/#concept-document-type id=creating-browsing-contexts:concept-document-type data-x-internal=concept-document-type>type</a><dd>"<code>html</code>"<dt><a href=https://dom.spec.whatwg.org/#concept-document-content-type id=creating-browsing-contexts:concept-document-content-type data-x-internal=concept-document-content-type>content type</a><dd>"<code id=creating-browsing-contexts:text/html><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/html">text/html</a></code>"<dt><a href=https://dom.spec.whatwg.org/#concept-document-mode id=creating-browsing-contexts:concept-document-mode data-x-internal=concept-document-mode>mode</a><dd>"<code>quirks</code>"<dt><a href=https://dom.spec.whatwg.org/#concept-document-origin id=creating-browsing-contexts:concept-document-origin-2 data-x-internal=concept-document-origin>origin</a><dd><var>origin</var><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=creating-browsing-contexts:concept-document-bc-2>browsing context</a><dd><var>browsingContext</var>

     <dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-permissions-policy" id=creating-browsing-contexts:concept-document-permissions-policy>permissions policy</a><dd><var>permissionsPolicy</var><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-sandboxing-flag-set" id=creating-browsing-contexts:active-sandboxing-flag-set>active sandboxing flag set</a><dd><var>sandboxFlags</var><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#load-timing-info" id=creating-browsing-contexts:load-timing-info>load timing info</a><dd><var>loadTimingInfo</var><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-initial-about:blank" id=creating-browsing-contexts:is-initial-about:blank>is initial <code>about:blank</code></a><dd>true<dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-about-base-url" id=creating-browsing-contexts:concept-document-about-base-url>about base URL</a><dd><var>creatorBaseURL</var><dt><a href=https://dom.spec.whatwg.org/#concept-document-allow-declarative-shadow-roots id=creating-browsing-contexts:concept-document-allow-declarative-shadow-roots data-x-internal=concept-document-allow-declarative-shadow-roots>allow declarative shadow
     roots</a><dd>true<dt><a href=https://dom.spec.whatwg.org/#document-custom-element-registry id=creating-browsing-contexts:document-custom-element-registry data-x-internal=document-custom-element-registry>custom element registry</a><dd>a new <code id=creating-browsing-contexts:customelementregistry><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#customelementregistry">CustomElementRegistry</a></code> object</dl>
   <li><p>Let <var>iframeReferrerPolicy</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#determining-the-iframe-element-referrer-policy" id=creating-browsing-contexts:determining-the-iframe-element-referrer-policy>determining the
   <code>iframe</code> element referrer policy</a> given <var>embedder</var>.<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-internal-ancestor-origin-objects-list" id=creating-browsing-contexts:concept-document-internal-ancestor-origin-objects-list>internal ancestor origin objects
   list</a> to the result of running the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#internal-ancestor-origin-objects-list-creation-steps" id=creating-browsing-contexts:internal-ancestor-origin-objects-list-creation-steps>internal ancestor origin objects list creation
   steps</a> given <var>document</var> and <var>iframeReferrerPolicy</var>.<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-ancestor-origins-list" id=creating-browsing-contexts:concept-document-ancestor-origins-list>ancestor
   origins list</a> to the result of running the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ancestor-origins-list-creation-steps" id=creating-browsing-contexts:ancestor-origins-list-creation-steps>ancestor origins list creation
   steps</a> given <var>document</var>.<li>
    <p>If <var>creator</var> is non-null:</p>

    <ol><li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-document's-referrer" id="creating-browsing-contexts:the-document's-referrer">referrer</a> to the
     <a href=https://url.spec.whatwg.org/#concept-url-serializer id=creating-browsing-contexts:concept-url-serializer data-x-internal=concept-url-serializer>serialization</a> of <var>creator</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-url id="creating-browsing-contexts:the-document's-address" data-x-internal="the-document's-address">URL</a>.<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-policy-container" id=creating-browsing-contexts:concept-document-policy-container>policy
     container</a> to a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#clone-a-policy-container" id=creating-browsing-contexts:clone-a-policy-container>clone</a> of
     <var>creator</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-policy-container" id=creating-browsing-contexts:concept-document-policy-container-2>policy
     container</a>.<li><p>If <var>creator</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=creating-browsing-contexts:concept-document-origin-3 data-x-internal=concept-document-origin>origin</a> is
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin" id=creating-browsing-contexts:same-origin>same origin</a> with <var>creator</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object" id=creating-browsing-contexts:relevant-settings-object-3>relevant settings object</a>'s
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-top-level-origin" id=creating-browsing-contexts:concept-environment-top-level-origin-2>top-level origin</a>, then set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-coop" id=creating-browsing-contexts:concept-document-coop>opener policy</a> to <var>creator</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=creating-browsing-contexts:concept-document-bc-3>browsing context</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bc-tlbc" id=creating-browsing-contexts:bc-tlbc-2>top-level
     browsing context</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-document" id=creating-browsing-contexts:active-document>active document</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-coop" id=creating-browsing-contexts:concept-document-coop-2>opener policy</a>.</ol>
   <li><p><a id=creating-browsing-contexts:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>document</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-url id="creating-browsing-contexts:the-document's-address-2" data-x-internal="the-document's-address">URL</a>
   and <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object" id=creating-browsing-contexts:relevant-settings-object-4>relevant settings object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-creation-url" id=creating-browsing-contexts:concept-environment-creation-url>creation URL</a> are
   <code id=creating-browsing-contexts:about:blank-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#about:blank">about:blank</a></code>.<li><p>Mark <var>document</var> as <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ready-for-post-load-tasks" id=creating-browsing-contexts:ready-for-post-load-tasks>ready for post-load tasks</a>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#populate-with-html/head/body" id=creating-browsing-contexts:populate-with-html/head/body>Populate with <code>html</code>/<code>head</code>/<code>body</code></a> given
   <var>document</var>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#make-active" id=creating-browsing-contexts:make-active>Make active</a> <var>document</var>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#completely-finish-loading" id=creating-browsing-contexts:completely-finish-loading>Completely finish loading</a> <var>document</var>.<li><p>Return <var>browsingContext</var> and <var>document</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=creating-a-new-top-level-browsing-context>create a new top-level browsing context and document</dfn>:</p>

  <ol><li><p>Let <var>group</var> and <var>document</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-browsing-context-group-and-document" id=creating-browsing-contexts:creating-a-new-browsing-context-group-and-document>creating a new
   browsing context group and document</a>.<li><p>Return <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-set" id=creating-browsing-contexts:browsing-context-set>browsing context set</a>[0] and
   <var>document</var>.</ol>
  </div>


## Complete determine-origin algorithm and shared-origin note

<!-- selected-source-fragment:71743-71760 -->
  <div data-algorithm="">
  <p>To <dfn id=determining-the-origin>determine the origin</dfn>, given a <a id=creating-browsing-contexts:url href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a>
  <var>url</var>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing-flag-set" id=creating-browsing-contexts:sandboxing-flag-set>sandboxing flag set</a> <var>sandboxFlags</var>, and an
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=creating-browsing-contexts:concept-origin>origin</a>-or-null <var>sourceOrigin</var>:</p>

  <ol><li id=sandboxOrigin><p>If <var>sandboxFlags</var> has its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-origin-browsing-context-flag" id=creating-browsing-contexts:sandboxed-origin-browsing-context-flag>sandboxed origin browsing
   context flag</a> set, then return a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-opaque" id=creating-browsing-contexts:concept-origin-opaque>opaque
   origin</a>.<li><p>If <var>url</var> is null, then return a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-opaque" id=creating-browsing-contexts:concept-origin-opaque-2>opaque
   origin</a>.<li>
    <p>If <var>url</var> is <code id=creating-browsing-contexts:about:srcdoc><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#about:srcdoc">about:srcdoc</a></code>:</p>

    <ol><li><p><a id=creating-browsing-contexts:assert-3 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>sourceOrigin</var> is non-null.<li><p>Return <var>sourceOrigin</var>.</ol>
   <li><p>If <var>url</var> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#matches-about:blank" id=creating-browsing-contexts:matches-about:blank>matches <code>about:blank</code></a> and
   <var>sourceOrigin</var> is non-null, then return <var>sourceOrigin</var>.<li><p>Return <var>url</var>'s <a href=https://url.spec.whatwg.org/#concept-url-origin id=creating-browsing-contexts:concept-url-origin data-x-internal=concept-url-origin>origin</a>.</ol>

  <p class=note>The cases that return <var>sourceOrigin</var> result in two
  <code id=creating-browsing-contexts:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s that end up with the same underlying <a href=https://dom.spec.whatwg.org/#concept-document-origin id=creating-browsing-contexts:concept-document-origin-5 data-x-internal=concept-document-origin>origin</a>, meaning that <code id=creating-browsing-contexts:dom-document-domain><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-document-domain">document.domain</a></code> affects both.</p>
  </div>


## Group fields, complete creation/append/remove and lifetime clauses

<!-- selected-source-fragment:71829-71916 -->
  <h5 id=groupings-of-browsing-contexts><span class=secno>7.3.2.3</span> Groupings of browsing contexts<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#groupings-of-browsing-contexts" class=self-link></a></h5>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=groupings-of-browsing-contexts:top-level-browsing-context>top-level browsing context</a> has an associated <dfn id=tlbc-group>group</dfn> (null or a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group>browsing context group</a>). It is initially null.</p>

  <p>A user agent holds a <dfn id=browsing-context-group-set>browsing context group set</dfn> (a <a id=groupings-of-browsing-contexts:set href=https://infra.spec.whatwg.org/#ordered-set data-x-internal=set>set</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-2>browsing context groups</a>).</p>

  <p>A <dfn id=browsing-context-group>browsing context group</dfn> holds a <dfn id=browsing-context-set>browsing context set</dfn> (a <a id=groupings-of-browsing-contexts:set-2 href=https://infra.spec.whatwg.org/#ordered-set data-x-internal=set>set</a>
  of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=groupings-of-browsing-contexts:top-level-browsing-context-2>top-level browsing contexts</a>).</p>

  <p class=note>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=groupings-of-browsing-contexts:top-level-browsing-context-3>top-level browsing context</a> is added to the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-3>group</a> when the group is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-browsing-context-group-and-document" id=groupings-of-browsing-contexts:creating-a-new-browsing-context-group-and-document>created</a>. All subsequent <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=groupings-of-browsing-contexts:top-level-browsing-context-4>top-level
  browsing contexts</a> added to the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-4>group</a> will be
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context" id=groupings-of-browsing-contexts:auxiliary-browsing-context>auxiliary browsing contexts</a>.</p>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-5>browsing context group</a> has an associated <dfn id=agent-cluster-map>agent cluster map</dfn> (a weak
  <a href=https://infra.spec.whatwg.org/#ordered-map id=groupings-of-browsing-contexts:ordered-map data-x-internal=ordered-map>map</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-key" id=groupings-of-browsing-contexts:agent-cluster-key>agent cluster
  keys</a> to <a href=https://tc39.es/ecma262/#sec-agent-clusters id=groupings-of-browsing-contexts:agent-cluster data-x-internal=agent-cluster>agent clusters</a>). User agents are responsible
  for collecting agent clusters when it is deemed that nothing can access them anymore.</p>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-6>browsing context group</a> has an associated <dfn id=historical-agent-cluster-key-map>historical agent cluster key
  map</dfn>, which is a <a href=https://infra.spec.whatwg.org/#ordered-map id=groupings-of-browsing-contexts:ordered-map-2 data-x-internal=ordered-map>map</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=groupings-of-browsing-contexts:concept-origin>origins</a> to <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-key" id=groupings-of-browsing-contexts:agent-cluster-key-2>agent cluster keys</a>. This
  map is used to ensure the consistency of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#origin-keyed-agent-clusters">origin-keyed
  agent clusters</a> feature by recording what agent cluster keys were previously used for a given
  origin.</p>

  <p class=note>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#historical-agent-cluster-key-map" id=groupings-of-browsing-contexts:historical-agent-cluster-key-map>historical agent cluster key map</a> only ever gains entries over the
  lifetime of the browsing context group.</p>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-7>browsing context group</a> has a <dfn id=bcg-cross-origin-isolation>cross-origin isolation mode</dfn>, which is a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-mode" id=groupings-of-browsing-contexts:cross-origin-isolation-mode>cross-origin isolation mode</a>. It is initially "<code id=groupings-of-browsing-contexts:cross-origin-isolation-none><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-none">none</a></code>".</p>

  <p>A <dfn id=cross-origin-isolation-mode>cross-origin isolation mode</dfn> is one of three possible values: "<dfn id=cross-origin-isolation-none><code>none</code></dfn>", "<dfn id=cross-origin-isolation-logical><code>logical</code></dfn>", or "<dfn id=cross-origin-isolation-concrete><code>concrete</code></dfn>".</p>

  <div class=note>
   <p>"<code id=groupings-of-browsing-contexts:cross-origin-isolation-logical><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-logical">logical</a></code>" and "<code id=groupings-of-browsing-contexts:cross-origin-isolation-concrete><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-concrete">concrete</a></code>" are similar. They are both used for
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-8>browsing context groups</a> where:</p>

   <ul><li><p>every top-level <code id=groupings-of-browsing-contexts:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> has `<code><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-opener-policy-2" id=groupings-of-browsing-contexts:cross-origin-opener-policy-2>Cross-Origin-Opener-Policy</a>: <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#coop-same-origin" id=groupings-of-browsing-contexts:coop-same-origin>same-origin</a></code>`, and<li><p>every <code id=groupings-of-browsing-contexts:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> has a `<code id=groupings-of-browsing-contexts:cross-origin-embedder-policy><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-embedder-policy">Cross-Origin-Embedder-Policy</a></code>` header
    whose value is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#compatible-with-cross-origin-isolation" id=groupings-of-browsing-contexts:compatible-with-cross-origin-isolation>compatible with cross-origin isolation</a>.</ul>

   <p>On some platforms, it is difficult to provide the security properties required to grant safe
   access to the APIs gated by the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-cross-origin-isolated-capability" id=groupings-of-browsing-contexts:concept-settings-object-cross-origin-isolated-capability>cross-origin isolated
   capability</a>. As a result, only "<code id=groupings-of-browsing-contexts:cross-origin-isolation-concrete-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-concrete">concrete</a></code>" can grant access that capability.
   "<code id=groupings-of-browsing-contexts:cross-origin-isolation-logical-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-logical">logical</a></code>" is used on platform not supporting
   this capability, where various restrictions imposed by cross-origin isolation will still apply,
   but the capability is not granted.</p>
  </div>

  <div data-algorithm="">
  <p id=creating-a-new-browsing-context-group>To <dfn id=creating-a-new-browsing-context-group-and-document>create a new browsing context group and document</dfn>:</p>

  <ol><li><p>Let <var>group</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-9>browsing context group</a>.<li><p><a href=https://infra.spec.whatwg.org/#set-append id=groupings-of-browsing-contexts:set-append data-x-internal=set-append>Append</a> <var>group</var> to the user agent's
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group-set" id=groupings-of-browsing-contexts:browsing-context-group-set>browsing context group set</a>.<li><p>Let <var>browsingContext</var> and <var>document</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-browsing-context" id=groupings-of-browsing-contexts:creating-a-new-browsing-context>creating a
   new browsing context and document</a> with null, null, and <var>group</var>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bcg-append" id=groupings-of-browsing-contexts:bcg-append>Append</a> <var>browsingContext</var> to
   <var>group</var>.<li><p>Return <var>group</var> and <var>document</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=bcg-append>append</dfn> a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=groupings-of-browsing-contexts:top-level-browsing-context-5>top-level browsing context</a>
  <var>browsingContext</var> to a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-10>browsing context group</a> <var>group</var>:</p>

  <ol><li><p><a href=https://infra.spec.whatwg.org/#set-append id=groupings-of-browsing-contexts:set-append-2 data-x-internal=set-append>Append</a> <var>browsingContext</var> to <var>group</var>'s
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-set" id=groupings-of-browsing-contexts:browsing-context-set>browsing context set</a>.<li><p>Set <var>browsingContext</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tlbc-group" id=groupings-of-browsing-contexts:tlbc-group>group</a> to
   <var>group</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=bcg-remove>remove</dfn> a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=groupings-of-browsing-contexts:top-level-browsing-context-6>top-level browsing context</a>
  <var>browsingContext</var>:</p>

  <ol><li><p><a id=groupings-of-browsing-contexts:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>browsingContext</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tlbc-group" id=groupings-of-browsing-contexts:tlbc-group-2>group</a>
   is non-null.<li><p>Let <var>group</var> be <var>browsingContext</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tlbc-group" id=groupings-of-browsing-contexts:tlbc-group-3>group</a>.<li><p>Set <var>browsingContext</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tlbc-group" id=groupings-of-browsing-contexts:tlbc-group-4>group</a> to null.<li><p><a href=https://infra.spec.whatwg.org/#list-remove id=groupings-of-browsing-contexts:list-remove data-x-internal=list-remove>Remove</a> <var>browsingContext</var> from
   <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-set" id=groupings-of-browsing-contexts:browsing-context-set-2>browsing context set</a>.<li><p>If <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-set" id=groupings-of-browsing-contexts:browsing-context-set-3>browsing context set</a> <a href=https://infra.spec.whatwg.org/#list-is-empty id=groupings-of-browsing-contexts:list-is-empty data-x-internal=list-is-empty>is
   empty</a>, then <a href=https://infra.spec.whatwg.org/#list-remove id=groupings-of-browsing-contexts:list-remove-2 data-x-internal=list-remove>remove</a> <var>group</var> from the user
   agent's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group-set" id=groupings-of-browsing-contexts:browsing-context-group-set-2>browsing context group set</a>.</ol>
  </div>

  <p class=note><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bcg-append" id=groupings-of-browsing-contexts:bcg-append-2>Append</a> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bcg-remove" id=groupings-of-browsing-contexts:bcg-remove>remove</a> are primitive operations that help define the lifetime of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=groupings-of-browsing-contexts:browsing-context-group-11>browsing
  context group</a>. They are called by higher-level creation and destruction operations for
  <code id=groupings-of-browsing-contexts:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=groupings-of-browsing-contexts:browsing-context>browsing contexts</a>.</p>

  <div data-algorithm="">
  <p id=a-browsing-context-is-discarded>When there are no <code id=groupings-of-browsing-contexts:document-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> objects whose
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=groupings-of-browsing-contexts:concept-document-bc>browsing context</a> equals a given <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=groupings-of-browsing-contexts:browsing-context-2>browsing
  context</a> (i.e., all such <code id=groupings-of-browsing-contexts:document-5><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s have been <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document" id=groupings-of-browsing-contexts:destroy-a-document>destroyed</a>), and that <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=groupings-of-browsing-contexts:browsing-context-3>browsing context</a>'s <code id=groupings-of-browsing-contexts:windowproxy><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code> is
  eligible for garbage collection, then the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=groupings-of-browsing-contexts:browsing-context-4>browsing context</a> will never be accessed
  again. If it is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=groupings-of-browsing-contexts:top-level-browsing-context-7>top-level browsing context</a>, then at this point the user agent must
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bcg-remove" id=groupings-of-browsing-contexts:bcg-remove-2>remove</a> it.</p>
  </div>


## Complete html/head/body skeleton population

<!-- selected-source-fragment:75414-75428 -->
  <div data-algorithm="">
  <p>To <dfn id=populate-with-html/head/body>populate with <code>html</code>/<code>head</code>/<code>body</code></dfn> given a
  <code id=shared-document-creation-infrastructure:document-11><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var>:</p>

  <ol><li><p>Let <var>html</var> be the result of <a href=https://dom.spec.whatwg.org/#concept-create-element id=shared-document-creation-infrastructure:create-an-element data-x-internal=create-an-element>creating an
   element</a> given <var>document</var>, "<code>html</code>", and the <a id=shared-document-creation-infrastructure:html-namespace-2 href=https://infra.spec.whatwg.org/#html-namespace data-x-internal=html-namespace-2>HTML
   namespace</a>.<li><p>Let <var>head</var> be the result of <a href=https://dom.spec.whatwg.org/#concept-create-element id=shared-document-creation-infrastructure:create-an-element-2 data-x-internal=create-an-element>creating an
   element</a> given <var>document</var>, "<code>head</code>", and the <a id=shared-document-creation-infrastructure:html-namespace-2-2 href=https://infra.spec.whatwg.org/#html-namespace data-x-internal=html-namespace-2>HTML
   namespace</a>.<li><p>Let <var>body</var> be the result of <a href=https://dom.spec.whatwg.org/#concept-create-element id=shared-document-creation-infrastructure:create-an-element-3 data-x-internal=create-an-element>creating an
   element</a> given <var>document</var>, "<code>body</code>", and the <a id=shared-document-creation-infrastructure:html-namespace-2-3 href=https://infra.spec.whatwg.org/#html-namespace data-x-internal=html-namespace-2>HTML
   namespace</a>.<li><p><a href=https://dom.spec.whatwg.org/#concept-node-append id=shared-document-creation-infrastructure:concept-node-append data-x-internal=concept-node-append>Append</a> <var>html</var> to
   <var>document</var>.<li><p><a href=https://dom.spec.whatwg.org/#concept-node-append id=shared-document-creation-infrastructure:concept-node-append-2 data-x-internal=concept-node-append>Append</a> <var>head</var> to
   <var>html</var>.<li><p><a href=https://dom.spec.whatwg.org/#concept-node-append id=shared-document-creation-infrastructure:concept-node-append-3 data-x-internal=concept-node-append>Append</a> <var>body</var> to
   <var>html</var>.</ol>
  </div>


## Loaded-time state and complete finish-loading algorithm

<!-- selected-source-fragment:75686-75722 -->
  <h4 id=loading-documents><span class=secno>7.5.8</span> Finishing the loading process<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#loading-documents" class=self-link></a></h4>

  <p>A <code id=loading-documents:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> has a <dfn id=completely-loaded-time>completely loaded time</dfn> (a time or null), which is
  initially null.</p>

  <div data-algorithm="">
  <p>A <code id=loading-documents:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> is considered <dfn id=completely-loaded>completely loaded</dfn> if its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#completely-loaded-time" id=loading-documents:completely-loaded-time>completely
  loaded time</a> is non-null.
  </div>

  <div data-algorithm="">
  <p>To <dfn id=completely-finish-loading>completely finish loading</dfn> a <code id=loading-documents:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var>:</p>

  <ol><li><p><a id=loading-documents:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=loading-documents:concept-document-bc>browsing
   context</a> is non-null.<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#completely-loaded-time" id=loading-documents:completely-loaded-time-2>completely loaded time</a> to the current
   time.<li>
    <p>Let <var>container</var> be <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=loading-documents:node-navigable>node navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container" id=loading-documents:nav-container>container</a>.</p>

    <div class=note>
     <p>This will be null in the case where <var>document</var> is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-initial-about:blank" id=loading-documents:is-initial-about:blank>initial <code>about:blank</code></a> <code id=loading-documents:document-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> in a
     <code id=loading-documents:frame><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#frame">frame</a></code> or <code id=loading-documents:the-iframe-element><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element">iframe</a></code>, since at the point of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-browsing-context" id=loading-documents:creating-a-new-browsing-context>browsing context creation</a> which calls this algorithm,
     the container relationship has not yet been established. (That happens in a subsequent step of
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#create-a-new-child-navigable" id=loading-documents:create-a-new-child-navigable>create a new child navigable</a>.)</p>

     <p>The consequence of this is that the following steps do nothing, i.e., we do not fire an
     asynchronous <code id=loading-documents:event-load><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load">load</a></code> event on the container element for such
     cases. Instead, a synchronous <code id=loading-documents:event-load-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load">load</a></code> event is fired in a special
     initial-insertion case when <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#process-the-iframe-attributes" id=loading-documents:process-the-iframe-attributes>processing the
     <code>iframe</code> attributes</a>.</p>
    </div>
   <li><p>If <var>container</var> is an <code id=loading-documents:the-iframe-element-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element">iframe</a></code> element, then <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-an-element-task" id=loading-documents:queue-an-element-task>queue an element
   task</a> on the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-manipulation-task-source" id=loading-documents:dom-manipulation-task-source>DOM manipulation task source</a> given <var>container</var> to run
   the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#iframe-load-event-steps" id=loading-documents:iframe-load-event-steps>iframe load event steps</a> given <var>container</var>.<li><p>Otherwise, if <var>container</var> is non-null, then <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-an-element-task" id=loading-documents:queue-an-element-task-2>queue an element task</a> on
   the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-manipulation-task-source" id=loading-documents:dom-manipulation-task-source-2>DOM manipulation task source</a> given <var>container</var> to <a href=https://dom.spec.whatwg.org/#concept-event-fire id=loading-documents:concept-event-fire data-x-internal=concept-event-fire>fire an event</a> named <code id=loading-documents:event-load-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load">load</a></code> at
   <var>container</var>.</p>
  </ol>
  </div>


## Agent kinds, create-agent, relevant agent, cluster fields and complete similar-origin acquisition

<!-- selected-source-fragment:77310-77451 -->
  <h4 id=agents-and-agent-clusters><span class=secno>8.1.2</span> Agents and agent clusters<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agents-and-agent-clusters" class=self-link></a></h4>

  <h5 id=integration-with-the-javascript-agent-formalism><span class=secno>8.1.2.1</span> Integration with the JavaScript agent formalism<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#integration-with-the-javascript-agent-formalism" class=self-link></a></h5>

  <p>JavaScript defines the concept of an <a id=integration-with-the-javascript-agent-formalism:agent href=https://tc39.es/ecma262/#sec-agents data-x-internal=agent>agent</a>. This section gives the mapping of that
  language-level concept on to the web platform.</p>

  <div class=note>
   <p>Conceptually, the <a id=integration-with-the-javascript-agent-formalism:agent-2 href=https://tc39.es/ecma262/#sec-agents data-x-internal=agent>agent</a> concept is an architecture-independent, idealized
   "thread" in which JavaScript code runs. Such code can involve multiple globals/<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-global-object-realm" id=integration-with-the-javascript-agent-formalism:concept-global-object-realm>realms</a> that can synchronously access each other, and
   thus needs to run in a single execution thread.</p>

   <p>Two <code id=integration-with-the-javascript-agent-formalism:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> objects having the same <a id=integration-with-the-javascript-agent-formalism:agent-3 href=https://tc39.es/ecma262/#sec-agents data-x-internal=agent>agent</a> does not indicate
   they can directly access all objects created in each other's realms. They would have to be
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin-domain" id=integration-with-the-javascript-agent-formalism:same-origin-domain>same origin-domain</a>; see <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#isplatformobjectsameorigin-(-o-)" id=integration-with-the-javascript-agent-formalism:isplatformobjectsameorigin-(-o-)>IsPlatformObjectSameOrigin</a>.</p>
  </div>

  <p>The following types of agents exist on the web platform:</p>

  <dl><dt><dfn id=similar-origin-window-agent data-export="">Similar-origin window agent</dfn><dd>
    <p>Contains various <code id=integration-with-the-javascript-agent-formalism:window-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> objects which can potentially reach each other, either
    directly or by using <code id=integration-with-the-javascript-agent-formalism:dom-document-domain><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-document-domain">document.domain</a></code>.</p>

    <p>If the encompassing <a id=integration-with-the-javascript-agent-formalism:agent-cluster href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-origin-keyed" id=integration-with-the-javascript-agent-formalism:is-origin-keyed>is origin-keyed</a> is true, then
    all the <code id=integration-with-the-javascript-agent-formalism:window-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> objects will be <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin" id=integration-with-the-javascript-agent-formalism:same-origin>same origin</a>, can reach each other
    directly, and <code id=integration-with-the-javascript-agent-formalism:dom-document-domain-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-document-domain">document.domain</a></code> will no-op.</p>

    <p class=note>Two <code id=integration-with-the-javascript-agent-formalism:window-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> objects that are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin" id=integration-with-the-javascript-agent-formalism:same-origin-2>same origin</a> can be in
    different <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#similar-origin-window-agent" id=integration-with-the-javascript-agent-formalism:similar-origin-window-agent>similar-origin window agents</a>, for
    instance if they are each in their own <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=integration-with-the-javascript-agent-formalism:browsing-context-group>browsing context group</a>.</p>
   <dt><dfn id=dedicated-worker-agent data-export="">Dedicated worker agent</dfn><dd><p>Contains a single <code id=integration-with-the-javascript-agent-formalism:dedicatedworkerglobalscope><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dedicatedworkerglobalscope">DedicatedWorkerGlobalScope</a></code>.<dt><dfn id=shared-worker-agent data-export="">Shared worker agent</dfn><dd><p>Contains a single <code id=integration-with-the-javascript-agent-formalism:sharedworkerglobalscope><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sharedworkerglobalscope">SharedWorkerGlobalScope</a></code>.<dt><dfn id=service-worker-agent data-export="">Service worker agent</dfn><dd><p>Contains a single <code id=integration-with-the-javascript-agent-formalism:serviceworkerglobalscope><a data-x-internal=serviceworkerglobalscope href=https://w3c.github.io/ServiceWorker/#serviceworkerglobalscope>ServiceWorkerGlobalScope</a></code>.<dt><dfn id=worklet-agent data-export="">Worklet agent</dfn><dd>
    <p>Contains a single <code id=integration-with-the-javascript-agent-formalism:workletglobalscope><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#workletglobalscope">WorkletGlobalScope</a></code> object.</p>

    <p class=note>Although a given worklet can have multiple realms, each such realm needs its own
    agent, as each realm can be executing code independently and at the same time as the others.</p>
   </dl>

  <p>Only <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#shared-worker-agent" id=integration-with-the-javascript-agent-formalism:shared-worker-agent>shared</a> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dedicated-worker-agent" id=integration-with-the-javascript-agent-formalism:dedicated-worker-agent>dedicated worker agents</a> allow the use of JavaScript <code id=integration-with-the-javascript-agent-formalism:atomics><a data-x-internal=atomics href=https://tc39.es/ecma262/#sec-atomics-object>Atomics</a></code> APIs to
  potentially <a href=https://tc39.es/ecma262/#sec-forward-progress id=integration-with-the-javascript-agent-formalism:forward-progress data-x-internal=forward-progress>block</a>.</p>

  

  <div data-algorithm="">
  <p>To <dfn id=create-an-agent>create an agent</dfn>, given a boolean <var>canBlock</var>:</p>

  <ol><li><p>Let <var>signifier</var> be a new unique internal value.<li><p>Let <var>candidateExecution</var> be a new <a id=integration-with-the-javascript-agent-formalism:candidate-execution href=https://tc39.es/ecma262/#sec-candidate-executions data-x-internal=candidate-execution>candidate execution</a>.<li><p>Let <var>agent</var> be a new <a id=integration-with-the-javascript-agent-formalism:agent-4 href=https://tc39.es/ecma262/#sec-agents data-x-internal=agent>agent</a> whose [[CanBlock]] is
   <var>canBlock</var>, [[Signifier]] is <var>signifier</var>, [[CandidateExecution]] is
   <var>candidateExecution</var>, and [[IsLockFree1]], [[IsLockFree2]], and [[LittleEndian]] are set
   at the implementation's discretion.<li><p>Set <var>agent</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-agent-event-loop" id=integration-with-the-javascript-agent-formalism:concept-agent-event-loop>event loop</a> to a new
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=integration-with-the-javascript-agent-formalism:event-loop>event loop</a>.<li><p>Return <var>agent</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>For a <a id=integration-with-the-javascript-agent-formalism:realm href=https://tc39.es/ecma262/#sec-code-realms data-x-internal=realm>realm</a> <var>realm</var>, the <a id=integration-with-the-javascript-agent-formalism:agent-5 href=https://tc39.es/ecma262/#sec-agents data-x-internal=agent>agent</a> whose [[Signifier]] is
  <var>realm</var>.[[AgentSignifier]] is <dfn data-export="" data-lt=agent id="realm's-agent" data-dfn-for=realm>the realm's agent</dfn>.</p>
  </div>

  <div data-algorithm="">
  <p>The <dfn id=relevant-agent data-export="">relevant agent</dfn> for a <a id=integration-with-the-javascript-agent-formalism:platform-object href=https://webidl.spec.whatwg.org/#dfn-platform-object data-x-internal=platform-object>platform object</a>
  <var>platformObject</var> is <var>platformObject</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-realm" id=integration-with-the-javascript-agent-formalism:concept-relevant-realm>relevant realm</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#realm's-agent" id="integration-with-the-javascript-agent-formalism:realm's-agent">agent</a>.</p>
  </div>

  <p class=note>The agent equivalent of the <a id=integration-with-the-javascript-agent-formalism:current-realm href=https://tc39.es/ecma262/#current-realm data-x-internal=current-realm>current realm</a> is the <a id=integration-with-the-javascript-agent-formalism:surrounding-agent href=https://tc39.es/ecma262/#surrounding-agent data-x-internal=surrounding-agent>surrounding
  agent</a>.</p>

  

  <h5 id=integration-with-the-javascript-agent-cluster-formalism><span class=secno>8.1.2.2</span> Integration with the JavaScript agent cluster formalism<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#integration-with-the-javascript-agent-cluster-formalism" class=self-link></a></h5>

  <p>JavaScript also defines the concept of an <a id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a>, which this standard maps
  to the web platform by placing agents appropriately when they are created using the
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#obtain-similar-origin-window-agent" id=integration-with-the-javascript-agent-cluster-formalism:obtain-similar-origin-window-agent>obtain a similar-origin window agent</a> or
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#obtaining-a-worker/worklet-agent" id=integration-with-the-javascript-agent-cluster-formalism:obtaining-a-worker/worklet-agent>obtain a worker/worklet agent</a>
  algorithms.</p>

  <p>The <a id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-2 href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a> concept is crucial for defining the JavaScript memory model, and
  in particular among which <a href=https://tc39.es/ecma262/#sec-agents id=integration-with-the-javascript-agent-cluster-formalism:agent data-x-internal=agent>agents</a> the backing data of
  <code id=integration-with-the-javascript-agent-cluster-formalism:sharedarraybuffer><a data-x-internal=sharedarraybuffer href=https://tc39.es/ecma262/#sec-sharedarraybuffer-objects>SharedArrayBuffer</a></code> objects can be shared.</p>

  <p class=note>Conceptually, the <a id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-3 href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a> concept is an
  architecture-independent, idealized "process boundary" that groups together multiple "threads"
  (<a href=https://tc39.es/ecma262/#sec-agents id=integration-with-the-javascript-agent-cluster-formalism:agent-2 data-x-internal=agent>agents</a>). The <a href=https://tc39.es/ecma262/#sec-agent-clusters id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-4 data-x-internal=agent-cluster>agent clusters</a>
  defined by the specification are generally more restrictive than the actual process boundaries
  implemented in user agents. By enforcing these idealized divisions at the specification level, we
  ensure that web developers see interoperable behavior with regard to shared memory, even in the
  face of varying and changing user agent process models.</p>

  

  <p>An <a id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-5 href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a> has an associated <dfn id=agent-cluster-cross-origin-isolation>cross-origin isolation mode</dfn>, which is a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-mode" id=integration-with-the-javascript-agent-cluster-formalism:cross-origin-isolation-mode>cross-origin isolation mode</a>. It is initially "<code id=integration-with-the-javascript-agent-cluster-formalism:cross-origin-isolation-none><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-none">none</a></code>".</p>

  <p>An <a id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-6 href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a> has an associated <dfn id=is-origin-keyed>is origin-keyed</dfn> (a boolean), which
  is initially false.</p>

  <hr>

  <p>The following defines the allocation of the <a href=https://tc39.es/ecma262/#sec-agent-clusters id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-7 data-x-internal=agent-cluster>agent clusters</a>
  of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#similar-origin-window-agent" id=integration-with-the-javascript-agent-cluster-formalism:similar-origin-window-agent>similar-origin window agents</a>.</p>

  <p>An <dfn id=agent-cluster-key>agent cluster key</dfn> is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#site" id=integration-with-the-javascript-agent-cluster-formalism:site>site</a> or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-tuple" id=integration-with-the-javascript-agent-cluster-formalism:concept-origin-tuple>tuple origin</a>. Without web developer action to achieve <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#origin-keyed-agent-clusters">origin-keyed agent clusters</a>, it will be a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#site" id=integration-with-the-javascript-agent-cluster-formalism:site-2>site</a>.</p>

  <p class=note>An equivalent formulation is that an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-key" id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-key>agent cluster key</a> can be a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#scheme-and-host" id=integration-with-the-javascript-agent-cluster-formalism:scheme-and-host>scheme-and-host</a> or an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=integration-with-the-javascript-agent-cluster-formalism:concept-origin>origin</a>.</p>

  <div data-algorithm="">
  <p>To <dfn id=obtain-similar-origin-window-agent>obtain a similar-origin window agent</dfn>,
  given an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=integration-with-the-javascript-agent-cluster-formalism:concept-origin-2>origin</a> <var>origin</var>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context-group" id=integration-with-the-javascript-agent-cluster-formalism:browsing-context-group>browsing context group</a>
  <var>group</var>, and a boolean <var>requestsOAC</var>, run these steps:</p>

  <ol><li><p>Let <var>site</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#obtain-a-site" id=integration-with-the-javascript-agent-cluster-formalism:obtain-a-site>obtaining a site</a>
   with <var>origin</var>.<li><p>Let <var>key</var> be <var>site</var>.<li><p>If <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bcg-cross-origin-isolation" id=integration-with-the-javascript-agent-cluster-formalism:bcg-cross-origin-isolation>cross-origin isolation
   mode</a> is not "<code id=integration-with-the-javascript-agent-cluster-formalism:cross-origin-isolation-none-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-none">none</a></code>", then set
   <var>key</var> to <var>origin</var>.<li><p>Otherwise, if <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#historical-agent-cluster-key-map" id=integration-with-the-javascript-agent-cluster-formalism:historical-agent-cluster-key-map>historical agent cluster key
   map</a>[<var>origin</var>] <a href=https://infra.spec.whatwg.org/#map-exists id=integration-with-the-javascript-agent-cluster-formalism:map-exists data-x-internal=map-exists>exists</a>, then set <var>key</var> to
   <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#historical-agent-cluster-key-map" id=integration-with-the-javascript-agent-cluster-formalism:historical-agent-cluster-key-map-2>historical agent cluster key map</a>[<var>origin</var>].<li>
    <p>Otherwise:</p>

    <ol><li><p>If <var>requestsOAC</var> is true, then set <var>key</var> to
     <var>origin</var>.<li><p>Set <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#historical-agent-cluster-key-map" id=integration-with-the-javascript-agent-cluster-formalism:historical-agent-cluster-key-map-3>historical agent cluster key map</a>[<var>origin</var>]
     to <var>key</var>.</ol>
   <li>
    <p>If <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-map" id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-map>agent cluster map</a>[<var>key</var>] <a href=https://infra.spec.whatwg.org/#map-exists id=integration-with-the-javascript-agent-cluster-formalism:map-exists-2 data-x-internal=map-exists>does not exist</a>:</p>

    <ol><li><p>Let <var>agentCluster</var> be a new <a id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-8 href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a>.<li><p>Set <var>agentCluster</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-cross-origin-isolation" id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-cross-origin-isolation>cross-origin isolation mode</a> to
     <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bcg-cross-origin-isolation" id=integration-with-the-javascript-agent-cluster-formalism:bcg-cross-origin-isolation-2>cross-origin isolation
     mode</a>.<li>
      <p>If <var>key</var> is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=integration-with-the-javascript-agent-cluster-formalism:concept-origin-3>origin</a>:</p>

      <ol><li><p><a id=integration-with-the-javascript-agent-cluster-formalism:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>key</var> is <var>origin</var>.<li><p>Set <var>agentCluster</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-origin-keyed" id=integration-with-the-javascript-agent-cluster-formalism:is-origin-keyed>is origin-keyed</a> to true.</ol>
     <li><p>Add the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#create-an-agent" id=integration-with-the-javascript-agent-cluster-formalism:create-an-agent>creating an agent</a>, given false,
     to <var>agentCluster</var>.<li><p>Set <var>group</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-map" id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-map-2>agent cluster map</a>[<var>key</var>] to
     <var>agentCluster</var>.</ol>
   <li><p>Return the single <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#similar-origin-window-agent" id=integration-with-the-javascript-agent-cluster-formalism:similar-origin-window-agent-2>similar-origin window agent</a> contained in <var>group</var>'s
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-map" id=integration-with-the-javascript-agent-cluster-formalism:agent-cluster-map-3>agent cluster map</a>[<var>key</var>].</ol>
  </div>

  <p class=note>This means that there is only one <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#similar-origin-window-agent" id=integration-with-the-javascript-agent-cluster-formalism:similar-origin-window-agent-3>similar-origin window agent</a> per
  browsing context agent cluster. (However, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dedicated-worker-agent" id=integration-with-the-javascript-agent-cluster-formalism:dedicated-worker-agent>dedicated
  worker</a> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#worklet-agent" id=integration-with-the-javascript-agent-cluster-formalism:worklet-agent>worklet agents</a> might be in the same
  cluster.)</p>


## Environment/settings/global fields and complete create-new-realm algorithm

<!-- selected-source-fragment:77544-77716 -->
  <h5 id=environments><span class=secno>8.1.3.1</span> Environments<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environments" class=self-link></a></h5>

  <p>An <dfn id=environment data-export="">environment</dfn> is an object that identifies the settings of a current or
  potential execution environment (i.e., a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-params" id=environments:navigation-params>navigation
  params</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-params-reserved-environment" id=environments:navigation-params-reserved-environment>reserved environment</a>
  or a <a href=https://fetch.spec.whatwg.org/#concept-request id=environments:concept-request data-x-internal=concept-request>request</a>'s <a href=https://fetch.spec.whatwg.org/#concept-request-reserved-client id=environments:concept-request-reserved-client data-x-internal=concept-request-reserved-client>reserved client</a>). An <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=environments:environment>environment</a> has
  the following fields:</p>

  <dl><dt>An <dfn data-dfn-for=environment id=concept-environment-id data-export="">id</dfn><dd><p>An opaque string that uniquely identifies this <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=environments:environment-2>environment</a>.<dt>A <dfn data-dfn-for=environment id=concept-environment-creation-url data-export="">creation URL</dfn><dd>
    <p>A <a id=environments:url href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a> that represents the location of the resource with which this
    <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=environments:environment-3>environment</a> is associated.</p>

    <p class=note>In the case of a <code id=environments:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=environments:environment-settings-object>environment settings object</a>,
    this URL might be distinct from its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-global" id=environments:concept-settings-object-global>global
    object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=environments:concept-document-window>associated
    <code>Document</code></a>'s <a href=https://dom.spec.whatwg.org/#concept-document-url id="environments:the-document's-address" data-x-internal="the-document's-address">URL</a>, due to
    mechanisms such as <code id=environments:dom-history-pushstate><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-history-pushstate">history.pushState()</a></code> which modify
    the latter.</p>
   <dt>A <dfn data-dfn-for=environment id=concept-environment-top-level-creation-url data-export="">top-level creation URL</dfn><dd><p>Null or a <a id=environments:url-2 href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a> that represents the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-creation-url" id=environments:concept-environment-creation-url>creation URL</a> of the "top-level"
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=environments:environment-4>environment</a>. It is null for workers and worklets.<dt>A <dfn data-dfn-for=environment id=concept-environment-top-level-origin data-export="">top-level origin</dfn><dd>
    <p>A <span class=XXX>for now</span> <a id=environments:implementation-defined href=https://infra.spec.whatwg.org/#implementation-defined data-x-internal=implementation-defined>implementation-defined</a> value, null,
    or an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=environments:concept-origin>origin</a>. For a "top-level" potential execution environment it is null (i.e.,
    when there is no response yet); otherwise it is the "top-level" <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=environments:environment-5>environment</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-origin" id=environments:concept-settings-object-origin>origin</a>. For a dedicated worker or worklet it is
    the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-top-level-origin" id=environments:concept-environment-top-level-origin>top-level origin</a> of its creator. For a shared or service worker it is an
    <a id=environments:implementation-defined-2 href=https://infra.spec.whatwg.org/#implementation-defined data-x-internal=implementation-defined>implementation-defined</a> value.</p>

    <p class=note>This is distinct from the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-top-level-creation-url" id=environments:concept-environment-top-level-creation-url>top-level creation URL</a>'s <a href=https://url.spec.whatwg.org/#concept-url-origin id=environments:concept-url-origin data-x-internal=concept-url-origin>origin</a> when sandboxing, workers, and worklets are
    involved.</p>
   <dt>A <dfn data-dfn-for=environment id=concept-environment-target-browsing-context data-export="">target browsing context</dfn><dd><p>Null or a target <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=environments:browsing-context>browsing context</a> for a <a href=https://fetch.spec.whatwg.org/#navigation-request id=environments:navigation-request data-x-internal=navigation-request>navigation request</a>.<dt>An <dfn data-dfn-for=environment id=concept-environment-active-service-worker data-export="">active service worker</dfn><dd><p>Null or a <a href=https://w3c.github.io/ServiceWorker/#dfn-service-worker id=environments:dfn-service-worker data-x-internal=dfn-service-worker>service worker</a> that <a href=https://w3c.github.io/ServiceWorker/#dfn-control id=environments:dfn-control data-x-internal=dfn-control>controls</a> the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=environments:environment-6>environment</a>.<dt>An <dfn data-dfn-for=environment id=concept-environment-execution-ready-flag data-export="">execution ready flag</dfn><dd><p>A flag that indicates whether the environment setup is done. It is initially
   unset.</dl>

  <p>Specifications may define <dfn id=environment-discarding-steps data-export="">environment discarding steps</dfn> for environments. The
  steps take an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=environments:environment-7>environment</a> as input.</p>

  <p class=note>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-discarding-steps" id=environments:environment-discarding-steps>environment discarding
  steps</a> are run for only a select few environments: the ones that will
  never become execution ready because, for example, they failed to load.</p>

  <h5 id=environment-settings-objects><span class=secno>8.1.3.2</span> Environment settings objects<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-objects" class=self-link></a></h5>

  <p>An <dfn id=environment-settings-object data-export="">environment settings object</dfn> is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment" id=environment-settings-objects:environment>environment</a> that
  additionally specifies algorithms for:</p>

  <dl><dt>A <dfn data-dfn-for="environment settings object" id=realm-execution-context data-export="">realm execution
   context</dfn><dd>
    <p>A <a id=environment-settings-objects:javascript-execution-context href=https://tc39.es/ecma262/#sec-execution-contexts data-x-internal=javascript-execution-context>JavaScript execution context</a> shared by all
    <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-script-element" id=environment-settings-objects:the-script-element>scripts</a> that use this settings object, i.e., all scripts in a given
    <a id=environment-settings-objects:realm href=https://tc39.es/ecma262/#sec-code-realms data-x-internal=realm>realm</a>. When we <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#run-a-classic-script" id=environment-settings-objects:run-a-classic-script>run a classic script</a> or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#run-a-module-script" id=environment-settings-objects:run-a-module-script>run a module
    script</a>, this execution context becomes the top of the <a id=environment-settings-objects:javascript-execution-context-stack href=https://tc39.es/ecma262/#execution-context-stack data-x-internal=javascript-execution-context-stack>JavaScript execution context
    stack</a>, on top of which another execution context specific to the script in question is
    pushed. (This setup ensures <a id=environment-settings-objects:source-text-module-record href=https://tc39.es/ecma262/#sec-source-text-module-records data-x-internal=source-text-module-record>Source Text Module Record</a>'s
    <a href=https://tc39.es/ecma262/#sec-moduleevaluation id=environment-settings-objects:js-evaluate data-x-internal=js-evaluate>Evaluate</a> knows which realm to use.)</p>
   <dt>A <dfn data-dfn-for="environment
   settings object" id=concept-settings-object-module-map data-export="">module map</dfn><dd>
    <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#module-map" id=environment-settings-objects:module-map>module map</a> that is used when importing JavaScript modules.</p>
   <dt>An <dfn data-dfn-for="environment settings object" id=api-base-url data-export="">API base URL</dfn><dd>
    <p>A <a id=environment-settings-objects:url href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a> used by APIs called by scripts that use this <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=environment-settings-objects:environment-settings-object>environment settings
    object</a> to <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#parse-a-url" id=environment-settings-objects:parse-a-url>parse URLs</a>.</p>
   <dt>An <dfn data-dfn-for="environment
   settings object" id=concept-settings-object-origin data-export="">origin</dfn><dd>
    <p>An <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=environment-settings-objects:concept-origin>origin</a> used in security checks.</p>
   <dt>A <dfn data-dfn-for="environment settings object" id=concept-settings-object-has-cross-site-ancestor data-export="">has cross-site ancestor</dfn><dd><p>A boolean used in security checks.<dt>A <dfn data-dfn-for="environment settings object" id=concept-settings-object-policy-container data-export="">policy container</dfn><dd>
    <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container" id=environment-settings-objects:policy-container>policy container</a> containing policies used for security checks.</p>
   <dt>A <dfn data-dfn-for="environment settings object" id=concept-settings-object-cross-origin-isolated-capability data-export="">cross-origin isolated capability</dfn><dd><p>A boolean representing whether scripts that use this <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=environment-settings-objects:environment-settings-object-2>environment settings
   object</a> are allowed to use APIs that require cross-origin isolation.<dt>A <dfn data-dfn-for="environment settings object" id=concept-settings-object-time-origin data-export="">time origin</dfn><dd>A number used as the baseline for performance-related timestamps. <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsHRT">[HRT]</a></dl>

  <div data-algorithm="">
  <p>An <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=environment-settings-objects:environment-settings-object-3>environment settings object</a>'s <dfn data-dfn-for="environment settings
  object" id=responsible-event-loop data-export="">responsible event loop</dfn> is its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-global" id=environment-settings-objects:concept-settings-object-global>global
  object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-agent" id=environment-settings-objects:relevant-agent>relevant agent</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-agent-event-loop" id=environment-settings-objects:concept-agent-event-loop>event
  loop</a>.</p>
  </div>

  <h5 id=realms-settings-objects-global-objects><span class=secno>8.1.3.3</span> Realms, settings objects, and global objects<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#realms-settings-objects-global-objects" class=self-link></a></h5>

  <div data-algorithm="">
  <p>A <dfn id=global-object data-export="">global object</dfn> is a JavaScript object that is the [[GlobalObject]] field of
  a <a id=realms-settings-objects-global-objects:realm href=https://tc39.es/ecma262/#sec-code-realms data-x-internal=realm>realm</a>.</p>
  </div>

  <p class=note>In this specification, all <a href=https://tc39.es/ecma262/#sec-code-realms id=realms-settings-objects-global-objects:realm-2 data-x-internal=realm>realms</a> are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-javascript-realm" id=realms-settings-objects-global-objects:creating-a-new-javascript-realm>created</a> with <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object>global
  objects</a> that are either <code id=realms-settings-objects-global-objects:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code>, <code id=realms-settings-objects-global-objects:workerglobalscope><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#workerglobalscope">WorkerGlobalScope</a></code>, or
  <code id=realms-settings-objects-global-objects:workletglobalscope><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#workletglobalscope">WorkletGlobalScope</a></code> objects.</p>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-2>global object</a> has an <dfn data-dfn-for="global object" id=in-error-reporting-mode>in error reporting mode</dfn>
  boolean, which is initially false.</p>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-3>global object</a> has an <dfn id=outstanding-rejected-promises-weak-set>outstanding rejected promises weak set</dfn>, a
  <a id=realms-settings-objects-global-objects:set href=https://infra.spec.whatwg.org/#ordered-set data-x-internal=set>set</a> of <code id=realms-settings-objects-global-objects:idl-promise><a data-x-internal=idl-promise href=https://webidl.spec.whatwg.org/#idl-promise>Promise</a></code> objects, initially empty. This set
  must not create strong references to any of its members, and implementations are free to limit
  its size in an <a id=realms-settings-objects-global-objects:implementation-defined href=https://infra.spec.whatwg.org/#implementation-defined data-x-internal=implementation-defined>implementation-defined</a> manner, e.g., by removing old entries from it
  when new ones are added.</p>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-4>global object</a> has an <dfn id=about-to-be-notified-rejected-promises-list>about-to-be-notified rejected promises list</dfn>, a
  <a id=realms-settings-objects-global-objects:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <code id=realms-settings-objects-global-objects:idl-promise-2><a data-x-internal=idl-promise href=https://webidl.spec.whatwg.org/#idl-promise>Promise</a></code> objects, initially empty.</p>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-5>global object</a> has an <dfn id=concept-global-import-map>import map</dfn>,
  initially an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#empty-import-map" id=realms-settings-objects-global-objects:empty-import-map>empty import map</a>.</p>

  <p class=note>For now, only <code id=realms-settings-objects-global-objects:window-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-6>global
  objects</a> have their <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-global-import-map" id=realms-settings-objects-global-objects:concept-global-import-map>import map</a> modified
  from the initial empty one. The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-global-import-map" id=realms-settings-objects-global-objects:concept-global-import-map-2>import map</a> is
  only accessed for the resolution of a root <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#module-script" id=realms-settings-objects-global-objects:module-script>module script</a>.</p>

  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-7>global object</a> has a <dfn id=resolved-module-set>resolved module set</dfn>, a <a id=realms-settings-objects-global-objects:set-2 href=https://infra.spec.whatwg.org/#ordered-set data-x-internal=set>set</a>
  of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#specifier-resolution-record" id=realms-settings-objects-global-objects:specifier-resolution-record>specifier resolution records</a>, initially
  empty.</p>

  <p class=note>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#resolved-module-set" id=realms-settings-objects-global-objects:resolved-module-set>resolved module set</a> ensures that module specifier resolution
  returns the same result when called multiple times with the same (referrer, specifier) pair. It
  does that by ensuring that <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#import-map" id=realms-settings-objects-global-objects:import-map>import map</a> rules that impact the specifier in its
  referrer's scope cannot be defined after its initial resolution. For now, only <code id=realms-settings-objects-global-objects:window-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code>
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-8>global objects</a> have their module set data structures modified
  from the initial empty one.</p>

  <hr>

  <p>There is always a 1-to-1-to-1 mapping between <a href=https://tc39.es/ecma262/#sec-code-realms id=realms-settings-objects-global-objects:realm-3 data-x-internal=realm>realms</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-9>global objects</a>, and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=realms-settings-objects-global-objects:environment-settings-object>environment settings objects</a>:</p>

  <ul><li>
    <div data-algorithm="">
    <p>A <a id=realms-settings-objects-global-objects:realm-4 href=https://tc39.es/ecma262/#sec-code-realms data-x-internal=realm>realm</a> has a [[HostDefined]] field, which contains <dfn data-export="" data-lt="settings object" id=concept-realm-settings-object data-dfn-for=realm>the realm's
    settings object</dfn>.</p>
    </div>
   <li>
    <div data-algorithm="">
    <p>A <a id=realms-settings-objects-global-objects:realm-5 href=https://tc39.es/ecma262/#sec-code-realms data-x-internal=realm>realm</a> has a [[GlobalObject]] field, which contains <dfn data-export="" data-lt="global object" id=concept-realm-global data-dfn-for=realm>the realm's global
    object</dfn>.</p>
    </div>
   <li><p>Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-10>global object</a> in this specification is created during the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-javascript-realm" id=realms-settings-objects-global-objects:creating-a-new-javascript-realm-2>creation</a> of a corresponding <a id=realms-settings-objects-global-objects:realm-6 href=https://tc39.es/ecma262/#sec-code-realms data-x-internal=realm>realm</a>, known as
   <dfn data-export="" data-lt=realm id=concept-global-object-realm data-dfn-for="global object">the global
   object's realm</dfn>.<li id=relevant-settings-object-for-a-global-object><p>Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-object" id=realms-settings-objects-global-objects:global-object-11>global object</a> in this
   specification is created alongside a corresponding <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=realms-settings-objects-global-objects:environment-settings-object-2>environment settings object</a>,
   known as its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object" id=realms-settings-objects-global-objects:relevant-settings-object>relevant settings object</a>.</p>

   <li>
    <div data-algorithm="">
    <p>An <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=realms-settings-objects-global-objects:environment-settings-object-3>environment settings object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#realm-execution-context" id=realms-settings-objects-global-objects:realm-execution-context>realm execution context</a>'s
    Realm component is <dfn data-export="" data-lt=realm id="environment-settings-object's-realm" data-dfn-for="environment settings object">the environment settings object's realm</dfn>.</p>
    </div>
   <li>
    <div data-algorithm="">
    <p>An <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=realms-settings-objects-global-objects:environment-settings-object-4>environment settings object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object's-realm" id="realms-settings-objects-global-objects:environment-settings-object's-realm">realm</a> then has a [[GlobalObject]] field, which contains <dfn data-export="" data-lt="global object" id=concept-settings-object-global data-dfn-for="environment settings
    object">the environment settings object's global object</dfn>.</p>
    </div>
   </ul>

  <div data-algorithm="">
  <p>To <dfn id=creating-a-new-javascript-realm data-export="">create a new
  realm</dfn> in an <a id=realms-settings-objects-global-objects:agent href=https://tc39.es/ecma262/#sec-agents data-x-internal=agent>agent</a> <var>agent</var>, optionally with instructions to create a
  global object or a global <b>this</b> binding (or both), the following steps are taken:</p>

  <ol><li><p>Perform <a href=https://tc39.es/ecma262/#sec-initializehostdefinedrealm id=realms-settings-objects-global-objects:js-initializehostdefinedrealm data-x-internal=js-initializehostdefinedrealm>InitializeHostDefinedRealm</a>()
   with the provided customizations for creating the global object and the global <b>this</b>
   binding.<li>
    <p>Let <var>realm execution context</var> be the <a id=realms-settings-objects-global-objects:running-javascript-execution-context href=https://tc39.es/ecma262/#running-execution-context data-x-internal=running-javascript-execution-context>running JavaScript execution
    context</a>.</p>

    <p class=note>This is the <a id=realms-settings-objects-global-objects:javascript-execution-context href=https://tc39.es/ecma262/#sec-execution-contexts data-x-internal=javascript-execution-context>JavaScript execution context</a> created in the previous
    step.</p>
   <li><p>Remove <var>realm execution context</var> from the <a id=realms-settings-objects-global-objects:javascript-execution-context-stack href=https://tc39.es/ecma262/#execution-context-stack data-x-internal=javascript-execution-context-stack>JavaScript execution context
   stack</a>.<li><p>Let <var>realm</var> be <var>realm execution context</var>'s Realm component.<li>
    <p>If <var>agent</var>'s <a id=realms-settings-objects-global-objects:agent-cluster href=https://tc39.es/ecma262/#sec-agent-clusters data-x-internal=agent-cluster>agent cluster</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#agent-cluster-cross-origin-isolation" id=realms-settings-objects-global-objects:agent-cluster-cross-origin-isolation>cross-origin isolation mode</a> is "<code id=realms-settings-objects-global-objects:cross-origin-isolation-none><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cross-origin-isolation-none">none</a></code>":</p>

    <ol><li><p>Let <var>global</var> be <var>realm</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-realm-global" id=realms-settings-objects-global-objects:concept-realm-global>global
     object</a>.<li><p>Let <var>status</var> be ! <var>global</var>.[[Delete]]("<code>SharedArrayBuffer</code>").<li><p><a id=realms-settings-objects-global-objects:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>status</var> is true.</ol>

    <p class=note>This is done for compatibility with web content and there is some hope that this
    can be removed in the future. Web developers can still get at the constructor through
    <code class=js>new WebAssembly.Memory({ shared:true, initial:0, maximum:0
    }).buffer.constructor</code>.</p>
   <li><p>Return <var>realm execution context</var>.</ol>
  </div>


## Relevant realm/settings/global and complete scripting enabled/disabled clauses

<!-- selected-source-fragment:78096-78157 -->
  <h6 id=relevant><span class=secno>8.1.3.3.4</span> Relevant<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant" class=self-link></a></h6>

  <div data-algorithm="">
  <p>The <dfn id=concept-relevant-realm data-export="">relevant realm</dfn> for a <a id=relevant:platform-object href=https://webidl.spec.whatwg.org/#dfn-platform-object data-x-internal=platform-object>platform
  object</a> is the value of <a href=https://webidl.spec.whatwg.org/#es-platform-objects id=relevant:realm-field-of-a-platform-object data-x-internal=realm-field-of-a-platform-object>its [[Realm]]
  field</a>.</p>
  </div>

  <div data-algorithm="">
  <p>Then, the <dfn id=relevant-settings-object data-export="">relevant settings object</dfn> for a <a id=relevant:platform-object-2 href=https://webidl.spec.whatwg.org/#dfn-platform-object data-x-internal=platform-object>platform object</a>
  <var>o</var> is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-realm-settings-object" id=relevant:concept-realm-settings-object>environment settings
  object</a> of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-realm" id=relevant:concept-relevant-realm>relevant realm</a> for
  <var>o</var>.</p>
  </div>

  <div data-algorithm="">
  <p>Similarly, the <dfn id=concept-relevant-global data-export="">relevant global object</dfn> for a
  <a id=relevant:platform-object-3 href=https://webidl.spec.whatwg.org/#dfn-platform-object data-x-internal=platform-object>platform object</a> <var>o</var> is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-realm-global" id=relevant:concept-realm-global>global
  object</a> of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-realm" id=relevant:concept-relevant-realm-2>relevant realm</a> for
  <var>o</var>.</p>
  </div>

  <h5 id=enabling-and-disabling-scripting><span class=secno>8.1.3.4</span> Enabling and disabling scripting<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#enabling-and-disabling-scripting" class=self-link></a></h5>

  <div data-algorithm="">
  <p id=concept-bc-script><dfn id=concept-environment-script>Scripting is enabled</dfn> for
  an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=enabling-and-disabling-scripting:environment-settings-object>environment settings object</a> <var>settings</var> when all of the following
  conditions are true:</p>

  <ul><li>The user agent supports scripting.<li>
   <a id=enabling-and-disabling-scripting:tracking-vector title="There is a tracking vector here." href=https://infra.spec.whatwg.org/#tracking-vector class=tracking-vector data-x-internal=tracking-vector><img width=46 alt="(This is a tracking vector.)" crossorigin="" class=darkmode-aware src=https://resources.whatwg.org/tracking-vector.svg height=64></a>
   The user has not disabled scripting for <var>settings</var> at this time. (User agents may
   provide users with the option to disable scripting globally, or in a finer-grained manner, e.g.,
   on a per-origin basis, down to the level of individual <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=enabling-and-disabling-scripting:environment-settings-object-2>environment settings objects</a>.)<li id=sandboxScriptBlocked>Either <var>settings</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-global" id=enabling-and-disabling-scripting:concept-settings-object-global>global object</a> is not a <code id=enabling-and-disabling-scripting:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> object,
   or <var>settings</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-global" id=enabling-and-disabling-scripting:concept-settings-object-global-2>global object</a>'s
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=enabling-and-disabling-scripting:concept-document-window>associated <code>Document</code></a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-sandboxing-flag-set" id=enabling-and-disabling-scripting:active-sandboxing-flag-set>active
   sandboxing flag set</a> does not have its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-scripts-browsing-context-flag" id=enabling-and-disabling-scripting:sandboxed-scripts-browsing-context-flag>sandboxed scripts browsing context flag</a>
   set.<li id=WebDriverBiDiScriptingIsEnabled>The result of <a id=enabling-and-disabling-scripting:webdriver-bidi-scripting-is-enabled href=https://w3c.github.io/webdriver-bidi/#webdriver-bidi-scripting-is-enabled data-x-internal=webdriver-bidi-scripting-is-enabled>WebDriver BiDi scripting is
   enabled</a> with <var>settings</var> is true.</ul>
  </div>

  <div data-algorithm="">
  <p id=concept-bc-noscript><dfn id=concept-environment-noscript>Scripting is disabled</dfn>
  for an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=enabling-and-disabling-scripting:environment-settings-object-3>environment settings object</a> when scripting is not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-script" id=enabling-and-disabling-scripting:concept-environment-script>enabled</a> for it, i.e., when any of the above conditions
  are false.</p>
  </div>

  <hr>

  <div data-algorithm="">
  <p><dfn id=concept-n-noscript>Scripting is disabled</dfn> for a <a id=enabling-and-disabling-scripting:platform-object href=https://webidl.spec.whatwg.org/#dfn-platform-object data-x-internal=platform-object>platform
  object</a> <var>object</var> if any of the following are true:</p>

  <ul><li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-noscript" id=enabling-and-disabling-scripting:concept-environment-noscript>Scripting is disabled</a> for
   <var>object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object" id=enabling-and-disabling-scripting:relevant-settings-object>relevant settings object</a>.<li><p>The <var>object</var> implements <code id=enabling-and-disabling-scripting:node><a data-x-internal=node href=https://dom.spec.whatwg.org/#interface-node>Node</a></code>, and <var>object</var>'s <a id=enabling-and-disabling-scripting:node-document href=https://dom.spec.whatwg.org/#concept-node-document data-x-internal=node-document>node
   document</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=enabling-and-disabling-scripting:concept-document-bc>browsing context</a> is null.<li><p>The <var>object</var> implements <code id=enabling-and-disabling-scripting:window-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> and <var>object</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=enabling-and-disabling-scripting:concept-document-window-2>associated <code>Document</code></a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=enabling-and-disabling-scripting:concept-document-bc-2>browsing context</a> is null.</ul>
  </div>

  <div data-algorithm="">
  <p><dfn id=concept-n-script>Scripting is enabled</dfn> for a <a id=enabling-and-disabling-scripting:platform-object-2 href=https://webidl.spec.whatwg.org/#dfn-platform-object data-x-internal=platform-object>platform object</a>
  <var>object</var>, when <var>object</var>'s scripting is not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-n-noscript" id=enabling-and-disabling-scripting:concept-n-noscript>disabled</a>.</p>
  </div>


## Complete event-loop definitions, queues, initial state and same-loop Windows

<!-- selected-source-fragment:81077-81189 -->
  <h4 id=event-loops><span class=secno>8.1.7</span> Event loops<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loops" class=self-link></a></h4>

  <h5 id=definitions-3><span class=secno>8.1.7.1</span> Definitions<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#definitions-3" class=self-link></a></h5>

  <p>To coordinate events, user interaction, scripts, rendering, networking, and so forth, user
  agents must use <dfn id=event-loop data-lt="event loop" data-export="">event loops</dfn> as
  described in this section. Each <a id=definitions-3:agent href=https://tc39.es/ecma262/#sec-agents data-x-internal=agent>agent</a> has an associated <dfn data-dfn-for=agent id=concept-agent-event-loop data-export="">event loop</dfn>, which is unique to that agent.</p>

  <div data-algorithm="">
  <p>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-agent-event-loop" id=definitions-3:concept-agent-event-loop>event loop</a> of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#similar-origin-window-agent" id=definitions-3:similar-origin-window-agent>similar-origin window
  agent</a> is known as a <dfn id=window-event-loop>window event loop</dfn>. The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-agent-event-loop" id=definitions-3:concept-agent-event-loop-2>event loop</a> of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dedicated-worker-agent" id=definitions-3:dedicated-worker-agent>dedicated worker agent</a>,
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#shared-worker-agent" id=definitions-3:shared-worker-agent>shared worker agent</a>, or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#service-worker-agent" id=definitions-3:service-worker-agent>service worker agent</a> is known as a <dfn id=worker-event-loop-2>worker
  event loop</dfn>. And the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-agent-event-loop" id=definitions-3:concept-agent-event-loop-3>event loop</a> of a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#worklet-agent" id=definitions-3:worklet-agent>worklet agent</a> is known as a <dfn id=worklet-event-loop>worklet event loop</dfn>.</p>
  </div>

  <div class=note>
   <p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=definitions-3:event-loop>Event loops</a> do not necessarily correspond to implementation
   threads. For example, multiple <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-event-loop" id=definitions-3:window-event-loop>window event loops</a> could
   be cooperatively scheduled in a single thread.</p>

   <p>However, for the various worker <a href=https://tc39.es/ecma262/#sec-agents id=definitions-3:agent-2 data-x-internal=agent>agents</a> that are allocated with
   [[CanBlock]] set to true, the JavaScript specification does place requirements on them regarding
   <a id=definitions-3:forward-progress href=https://tc39.es/ecma262/#sec-forward-progress data-x-internal=forward-progress>forward progress</a>, which effectively amount to requiring dedicated per-agent threads
   in those cases.</p>
  </div>

  

  <hr>

  <p>An <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=definitions-3:event-loop-2>event loop</a> has one or more <dfn id=task-queue>task queues</dfn>. A
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-queue" id=definitions-3:task-queue>task queue</a> is a <a id=definitions-3:set href=https://infra.spec.whatwg.org/#ordered-set data-x-internal=set>set</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=definitions-3:concept-task>tasks</a>.</p>

  <p class=note><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-queue" id=definitions-3:task-queue-2>Task queues</a> are <a href=https://infra.spec.whatwg.org/#ordered-set id=definitions-3:set-2 data-x-internal=set>sets</a>,
  not <a href=https://infra.spec.whatwg.org/#queue id=definitions-3:queue data-x-internal=queue>queues</a>, because the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop-processing-model">event
  loop processing model</a> grabs the first <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task-runnable" id=definitions-3:concept-task-runnable><em>runnable</em></a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=definitions-3:concept-task-2>task</a>
  from the chosen queue, instead of <a href=https://infra.spec.whatwg.org/#queue-dequeue id=definitions-3:dequeue data-x-internal=dequeue>dequeuing</a> the first task.</p>

  <p class=note>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microtask-queue" id=definitions-3:microtask-queue>microtask queue</a> is not a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-queue" id=definitions-3:task-queue-3>task queue</a>.</p>

  <p>Tasks encapsulate algorithms that are responsible for such work as:</p>

  <dl><dt>Events<dd>
    <p>Dispatching an <code id=definitions-3:event><a data-x-internal=event href=https://dom.spec.whatwg.org/#interface-event>Event</a></code> object at a particular
    <code id=definitions-3:eventtarget><a data-x-internal=eventtarget href=https://dom.spec.whatwg.org/#interface-eventtarget>EventTarget</a></code> object is often done by a dedicated task.</p>

    <p class=note>Not all events are dispatched using the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-queue" id=definitions-3:task-queue-4>task queue</a>; many are
    dispatched during other tasks.</p>
   <dt>Parsing<dd><p>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#html-parser" id=definitions-3:html-parser>HTML parser</a> tokenizing one or more bytes, and then processing any
   resulting tokens, is typically a task.<dt>Callbacks<dd><p>Calling a callback is often done by a dedicated task.<dt>Using a resource<dd><p>When an algorithm <a href=https://fetch.spec.whatwg.org/#concept-fetch id=definitions-3:concept-fetch data-x-internal=concept-fetch>fetches</a> a resource, if the fetching
   occurs in a non-blocking fashion then the processing of the resource once some or all of the
   resource is available is performed by a task.<dt>Reacting to DOM manipulation<dd><p>Some elements have tasks that trigger in response to DOM manipulation, e.g. when that
   element is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#insert-an-element-into-a-document" id=definitions-3:insert-an-element-into-a-document>inserted into the document</a>.</p>
  </dl>

  <p>Formally, a <dfn id=concept-task data-lt=task data-export="">task</dfn> is a
  <a id=definitions-3:struct href=https://infra.spec.whatwg.org/#struct data-x-internal=struct>struct</a> which has:</p>

  <dl><dt><dfn id=concept-task-steps>Steps</dfn><dd>A series of steps specifying the work to be done by the task.<dt>A <dfn id=concept-task-source>source</dfn><dd>One of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=definitions-3:task-source>task sources</a>, used to group and serialize
   related tasks.<dt>A <dfn id=concept-task-document>document</dfn><dd>A <code id=definitions-3:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> associated with the task, or null for tasks that are not in a
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-event-loop" id=definitions-3:window-event-loop-2>window event loop</a>.<dt>A <dfn id=script-evaluation-environment-settings-object-set data-export="">script evaluation environment settings object set</dfn><dd>A <a id=definitions-3:set-3 href=https://infra.spec.whatwg.org/#ordered-set data-x-internal=set>set</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id=definitions-3:environment-settings-object>environment settings
   objects</a> used for tracking script evaluation during the task.</dl>

  <div data-algorithm="">
  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=definitions-3:concept-task-3>task</a> is <dfn id=concept-task-runnable>runnable</dfn>
  if its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task-document" id=definitions-3:concept-task-document>document</a> is either null or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=definitions-3:fully-active>fully
  active</a>.</p>
  </div>

  <p>Per its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task-source" id=definitions-3:concept-task-source>source</a> field, each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=definitions-3:concept-task-4>task</a> is defined as coming from a specific <dfn id=task-source data-export="">task
  source</dfn>. For each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=definitions-3:event-loop-3>event loop</a>, every <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=definitions-3:task-source-2>task source</a> must be associated
  with a specific <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-queue" id=definitions-3:task-queue-5>task queue</a>.</p>

  <p class=note>Essentially, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=definitions-3:task-source-3>task sources</a> are used within
  standards to separate logically-different types of tasks, which a user agent might wish to
  distinguish between. <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-queue" id=definitions-3:task-queue-6>Task queues</a> are used by user agents to
  coalesce task sources within a given <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=definitions-3:event-loop-4>event loop</a>.</p>

  <p class=example>For example, a user agent could have one <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-queue" id=definitions-3:task-queue-7>task queue</a> for mouse and
  key events (to which the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#user-interaction-task-source" id=definitions-3:user-interaction-task-source>user interaction task source</a> is associated), and another to
  which all other <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=definitions-3:task-source-4>task sources</a> are associated. Then, using the
  freedom granted in the initial step of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop-processing-model">event loop
  processing model</a>, it could give keyboard and mouse events preference over other tasks
  three-quarters of the time, keeping the interface responsive but not starving other task queues.
  Note that in this setup, the processing model still enforces that the user agent would never
  process events from any one <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=definitions-3:task-source-5>task source</a> out of order.</p>

  <hr>

  <p>Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=definitions-3:event-loop-5>event loop</a> has a <dfn id=currently-running-task>currently running task</dfn>, which is either a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=definitions-3:concept-task-5>task</a> or null. Initially, this is null. It is used to handle
  reentrancy.</p>

  <p>Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=definitions-3:event-loop-6>event loop</a> has a <dfn id=microtask-queue>microtask queue</dfn>, which is a <a id=definitions-3:queue-2 href=https://infra.spec.whatwg.org/#queue data-x-internal=queue>queue</a> of
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microtask" id=definitions-3:microtask>microtasks</a>, initially empty. A <dfn id=microtask data-export="">microtask</dfn> is a
  colloquial way of referring to a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=definitions-3:concept-task-6>task</a> that was created via the
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-microtask" id=definitions-3:queue-a-microtask>queue a microtask</a> algorithm.</p>

  <p>Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=definitions-3:event-loop-7>event loop</a> has a <dfn id=performing-a-microtask-checkpoint>performing a microtask checkpoint</dfn> boolean,
  which is initially false. It is used to prevent reentrant invocation of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#perform-a-microtask-checkpoint" id=definitions-3:perform-a-microtask-checkpoint>perform a
  microtask checkpoint</a> algorithm.</p>

  <p>Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-event-loop" id=definitions-3:window-event-loop-3>window event loop</a> has a <code id=definitions-3:domhighrestimestamp><a data-x-internal=domhighrestimestamp href=https://w3c.github.io/hr-time/#dom-domhighrestimestamp>DOMHighResTimeStamp</a></code>
  <dfn id=last-render-opportunity-time>last render opportunity time</dfn>, initially set to zero.</p>

  <p>Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-event-loop" id=definitions-3:window-event-loop-4>window event loop</a> has a <code id=definitions-3:domhighrestimestamp-2><a data-x-internal=domhighrestimestamp href=https://w3c.github.io/hr-time/#dom-domhighrestimestamp>DOMHighResTimeStamp</a></code>
  <dfn id=last-idle-period-start-time>last idle period start time</dfn>, initially set to zero.</p>

  <div data-algorithm="">
  <p>To get the <dfn id=same-loop-windows>same-loop windows</dfn> for a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-event-loop" id=definitions-3:window-event-loop-5>window event loop</a> <var>loop</var>,
  return all <code id=definitions-3:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> objects whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-agent" id=definitions-3:relevant-agent>relevant agent</a>'s
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-agent-event-loop" id=definitions-3:concept-agent-event-loop-4>event loop</a> is <var>loop</var>.</p>
  </div>


