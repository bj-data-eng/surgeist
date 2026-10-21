# Base URLs, links and Meta processing

Canonical Document facts and retained element history qualify URL and metadata processing. [Hosted construction](hosted-documents.md) and [native document creation](document-creation.md) establish their respective environments.

## Base selection and bound external facts

Native HTML Base retains a nullable historical frozen URL record, its actual
freezing Document, full external influencing revision and response disposition.
Documents retain their actual first ordinary-tree Base-with-href and separately
qualified base facts. Becoming first or changing the current first's exact
null-namespace href freezes immediately, including selection revealed by removal.
Shadow roots and Template contents are separate trees. Losing first position,
removing href, detaching and adoption preserve old frozen history; ownerDocument
and freezing Document remain distinct. Clone/import starts unfrozen, then copied
attributes and actual insertion run fresh hooks. Native immutable installation
has the same fresh cause, without replaying parser history.

`DomBatch::bind_base_facts` consumes an exact `DomDocumentStamp` and the
producer's immutable `HtmlParseBaseFacts`. This explicitly binds external
observations to that actual native about:blank/UTF8 environment. The supplied
opaque encoding identity is bound to the known native encoding, rather than
interpreted as a charset label. The operation changes no URL/encoding metadata
and accepts no arbitrary parsed environment: srcdoc/about-base/nonblank or
contradictory known URL observations reject. Existing bindings reject foreign
producers and equal-revision fallback/encoding identity substitutions. A changed
same-producer full revision is an explicit newly qualified observation binding;
it does not itself refreeze historical state. Every supplied observation must
match the actual fallback record identity, encoding identity and full revision;
reached freezing additionally matches exact raw UTF16 href. Missing observations
or CSP permission refuse atomically. No URL parser or CSP interpreter is copied.

`DomLimits::with_base_processing` bounds aggregate actual retained observation
capacity and cumulative DOM qualification/scan/response work per batch, default0.
Canonical string admission counts fact-formation UTF16 extent and each retained
frozen URL serialization. Input facts are formed under the semantic producer's
own finite `HtmlBaseFactsLimits`; DOM does not claim that its work budget covers
work already performed by that external constructor. Scans are stackless and
poll every visit/comparison; no traversal bank is allocated. Failed/cancelled
binding or freezing preserves the old facts, first binding, attributes and
history together. Constructor-only paths need no configured Base processing.

`DomHtmlCaptureLimits::with_base_facts(records, utf16_units, validation_work)`
selects the separate producer fact-formation limits explicitly. Adapter vectors
obey the existing subject/work domains; record/unit limits apply to each retained
formation, and producer validation work is cumulative across initial capture and
Slot reconstruction. A newly supplied Slot ceiling includes already consumed
work. Semantic query sessions retain their own caller-selected limits. Canonical
Link history joins the actual capture. Frozen rows bind the current nodeDocument
and href as required by the producer, while the canonical historical freezing
Document stays separate. The producer accepts one full revision for document
rows; captures containing differently qualified bound Documents reject with
`RevisionMismatch` rather than inventing a shared revision. The independent
Base href getter still uses declared parse results even when CSP blocked the
historical frozen URL.

HTML's shared `select_frozen_base` owns failure/data/javascript/CSP-block fallback
and successful selection. Successful assignment precedes actual native response
qualification. Current Scripts have real `Uninitialized` results, including
speculationrules-typed Scripts: native Disabled preparation supplies no result
callback. The actual shadow-inclusive response walk reads those states. Native
Documents have never been active in a navigable, so node navigable is null and
HTML877d76574's top-level guard returns before flag/task access. Current batch
commands and host observations admit no UI recipient or selector/style executor.
DOM publishes actual `DocumentBase` invalidation and frozen `ElementState` changes;
`FactsPublished` certifies that bounded canonical publication, not UI,
CSSOM or task delivery. Identical qualified refreezes whose actual state/effects
remain unchanged are canonical no-ops, without fabricated history counters.

## Link state and processing boundaries

Native Link retains shared `HtmlLinkFlags` and its nullable creating-parser
Document. Only actual null-namespace disabled removal enables the historical flag;
readdition/adoption preserve it and clone/native installation start fresh. Native
creation also establishes an actual null associated stylesheet, and Documents
start with the source-initial empty render-blocking set (HTML877d7480–7481).
No admitted native command installs a Link sheet or enters that set; structural
View Transition CSSOM identities establish no Link association. The actual
href/rel/disabled/insertion guards read the current native no-BC environment:
stylesheet, modulepreload, preload, prefetch, preconnect, dns-prefetch and expect
processing triggers require browsing-context connectedness. Disabled-set has no
associated sheet to disable; id/anchor-name processing iterates the actual empty
render-blocking set. This proves no general render-blocked Boolean. Actual HTML
A name changes execute that hook; other Elements' ordinary name attributes
remain exclusion evidence.

Icon/manifest/alternate-feed/search UI, discovery and service uses have no admitted
native recipient; this owner advertises no supported processing-token registry.
Pingback's external processing remains unsupported, without a blanket no-BC
claim. Resource-bearing sheet/set/environment state refuses precisely through
`LinkProcessingUnavailable`. Retained parsed installation uses its own cause
and qualified facts, frozen history, flags and remapped Document references;
native fresh initialization cannot erase unknown resources/history. Current public Document destruction remains
unadmitted; this processing does not widen it or pin historical references.

## A and Area URL history

Native A/Area start with a null associated URL and execute HTML's real set-URL
creation steps, returning on absent href without external facts or extra query
admission. A changed null-namespace href first clears the cache, then encoding-
parses the exact UTF16 against the **current Document base URL** and actual
encoding. The shared Document-base selection uses the current first Base's
frozen record when present; it does not substitute the fallback or rerun Base's
CSP selection. Exact equal optional href values and namespaced attributes return
before parsing, including after adoption. Present-empty href still parses.

`DomBatch::bind_encoding_url_facts` binds immutable producer observations to
an exact initial `DomDocumentStamp`. The existing native Base-facts binding
establishes opaque URL/encoding identities against the real UTF8/about:blank
Document environment; identity binding never infers a charset from an arbitrary
label. The hyperlink bank validates the complete current parse revision, selected
base identity and encoding identity. A frozen base may retain an older revision.
A missing, stale, foreign or mismatched context refuses atomically; an actual
qualified failed parse instead retains its provenance with a null URL.
`DomNode::as_hyperlink` reads the stored URL and historical parse Document without
reinitializing an href IDL getter. Observation updates do not silently reparse.

`DomLimits::with_encoding_url_processing` bounds aggregate installed observation
capacity and cumulative batch work, separately from producer formation limits.
Both allowances default to zero. Canonical unit admission charges actual retained
raw/base/result string backing capacity for each bank and chosen cache, including
transient parses later discarded. Scans and raw comparisons poll cancellation;
Select composition also charges its actual enclosing work. All admission and
cancellation precede the single publication. Whole final semantic equality ignores
only derived aggregate unit accounting: a capacity-only candidate retains the
old canonical backing, usage and revision, including at exhausted revision.
Other real semantic changes publish the candidate's actual admitted usage and
complete changes; no per-node allocation restoration or synthetic revision occurs.

Adoption preserves URL history; a later changed href parses its actual new
Document context. Clone/import-as-clone and native immutable installation create
fresh null state, then run real copied-attribute/tree hooks in the destination.
Snapshots and captures preserve selected opaque parse identities and history.
Captures map the historical parse Document independently from current nodeDocument,
with producer string/relation retention and cancellable adapter work admission;
Slot supplements preserve those same observations. Explicit destruction retires
ordinary state without pinning Documents or following a reused generation.

A/Area href/rel/referrerpolicy and insertion/removal hooks consider speculative
loads using actual current/old-ancestor facts. The native Document's exact null
node-navigable guard returns before task/service processing. A name/id hooks use
the existing actual empty render-blocking-set proof. These steps provide no
navigation, download, loading service, Blob-store truth, origin/components or
href-IDL reinitialization. [Parsed installation](html-installation.md) transfers
its actual history under its own creation cause.

## Meta history and CSS selections

Native Meta processing keeps Document history and current CSS selections
separately. Content-language and referrer processing consume HTML-owned rules;
refresh retains its actual source flag and the native Do-nothing terminal action.
These histories are not reconstructed from currently attached nodes.
`metadata().meta_theme_color()` and `meta_color_scheme()` capture the independent
last completed selections at that publication. Like other raw canonical fields,
these getters can be read during incomplete Source execution; interpret them as
current-tree results only after the snapshot's Source operation is complete.
A known empty selection differs from unavailable producer
evidence, including in parsed Documents whose processing history alone does not
establish CSS results.

Ordinary Meta tree and attribute hooks select theme color in document order,
then independently select color-scheme. Theme candidates with missing, empty or
ASCII-whitespace-only media use the CSS-owned UTF16 whole-color operation after
ASCII edge trimming. Invalid colors
permit a later candidate; unavailable media or contextual color facts refuse the
batch before a later candidate can hide that missing computation. The selected
numerical color retains its alpha and color space. Color-scheme uses CSS-owned
component and property grammar, preserves symbolic values and original UTF16
provenance, and does not apply theme-color's media filter.

The native constructor establishes an actual empty set of Meta UI recipients,
independently of Document activity and focus/hover associations. Selection still
runs and applies its result to that empty set. These results do not establish
installed UI colors, computed root color-scheme or rendering resources.
Context-free preferred stylesheet-set and reached Meta CSP effects retain
explicit refusal; a later refusal rolls back earlier private selection and
history changes in the batch.

`prepare_meta_batch` is the ordinary native composition building block for an
actual association adapter. It records reached nonempty default-style names in
invocation order, preserving exact UTF-16 and the original qualified subject even
when that node is later destroyed. It borrows the actual canonical root
capabilities once, exposes their held current captures for the adapter's genuine
sheet-set flag edits, and stages optional inline ingestion with the original
profiles and a borrowed resolver. Every root and inline CSSOM owner prepares
before any publishes. `commit` is the final fallible operation; its published
custody retains the root locks while Runtime installs its prepared preference
history. Consuming `into_parts` unlocks and returns the genuine DOM and producer
commits plus CSSOM retirement backing. Drop before commit publishes nothing.
Retirement makes no new bounded-disposal guarantee for the existing DOM tail.

A genuine supplied owner can establish the first binding of a reached Document,
including an empty sheet list. Existing canonical ownership remains authoritative;
conflicting stores bearing the same root are refused independently of input order.
The first binding is staged in the existing canonical custody bank and publishes
only at the composed fence. It does not invent a DOM revision when the final tree
is unchanged, and a Document retired by the batch receives no new retained owner.
The native envelope admits the actual additional controlled-registry union and
its temporary peak alongside the other native holders and Runtime storage.

The ordinary capability requires idle Source, including no active authenticated
observer callback. An original Source uses the separate borrowed hold described
in [Source/Style](source-and-style.md#native-meta-composition); the ordinary prefix
does not authorize or prepay that operation. `DomMetaAtomicLimits` bounds extra actual holder/output
storage, UTF-16 backing and orchestration work alongside existing domain limits.
`DomMetaRuntimePrefix` declares the adapter's actual preadmitted bank storage and
already-spent formation work; preparation rechecks it against its actual limits
and starts with that work spent. Later Runtime usage reports describe the whole
simultaneous union and retain their high-water rather than adding repeated
reports. Bank growth admits old and replacement backing together; shared Meta
content is retained once per actual allocation, while every occurrence and
traversal is charged. Runtime owns preferred/last-selected history, including an
empty current sheet list. This DOM capability stores no preference mirror and
does not consume or bypass original parsed pending association contributions.

`DomLimits::with_meta_processing_work` admits cumulative Meta work.
`with_meta_theme_color_scratch_bytes` and `with_meta_color_scheme_limits` select
the corresponding finite CSS processing allowances; unconfigured processing
cannot silently acquire them. `with_meta_color_scheme_retained_bytes` separately
bounds aggregate selected CSS payload across the canonical Documents;
`DomUsage::meta_color_scheme_retained_bytes` reads their actual retained usage.
Source execution also admits actual simultaneous
parser and retained-result storage through its original controller. Equal
recomputations retain the previous checked CSS backing, while changed original
UTF16 remains observable even when preprocessing yields equal normalized text.

[Back to surgeist-dom](../README.md).
