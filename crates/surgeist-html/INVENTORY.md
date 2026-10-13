# HTML language inventory associations

This reference describes the HTML-owned values and intrinsic interpretation
implemented in the current crate source. It associates original index
identities with defining rules, real producers and source-qualified limits.
Association coverage does not certify all listed algorithms or whole HTML
conformance. Active unpublished implementation drafts are excluded.

[Source editions](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
selects WHATWG HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`.
The standard's defining clauses govern the non-normative printed indexes.
[Source attribution](NOTICE.md) and the [WHATWG license/notice](../../licenses/whatwg/NOTICE.md)
cover the retained source and incorporation basis.
[The retained index](../../references/html-language-elements-attributes--877d6146487f--2ca83b5ac984.md)
and [foundations](../../references/html-language-foundations--877d6146487f--b55e73c31978.md)
retain the source. Crate source governs what is implemented; this reference
neither substitutes default tables for those owners nor records Project status.

## Read the dispositions

Every E/A row retains the [raw and construction](#raw-and-construction) contract.
Every HTML E row can consume applicable [captured global states](#captured-global-states).
The contextual bindings below add narrower rules. **Implemented** always means
the named contract, not an entire element. **Required absent** means a specific
HTML-defined value/static operation has no corresponding producer in the current
crate source. **External** names actual delegated grammar or execution; HTML surrounding
rules remain separately bound. A source-defined grouping/concept does not need a
fake parsing API. No unchecked external text is called Valid.

Original identities are E001–E111, A001–A180 and M:source-anchor for all 28 child
headings of §2.3. E039 retains six headings, while foreign/open families retain
original rows; A143 retains both input and select. All A rows describe the indexed
no-namespace content attribute identities on the printed HTML contexts, including actual
form-associated custom classification where specified. Qualified XML attributes,
SVG/MathML attributes and open families are separate [supplemental source domains](#supplemental-source-domains).
Printed context/syntax is discovery; defining state/parent requirements control.

## Element identities

Each row's ordinary element is represented, with exact attributes/text and no
missing-element default. A textual/structural role is retained as data without a
promise of whole-element author/content-model validation. Attributes are linked
through the contextual A table and global bindings; the described consumer does
not assert all child/content constraints are machine checked.

| Identity | Exact item and namespace | Defining source | Intrinsic association and execution boundary |
| --- | --- | --- | --- |
| <a id="e001"></a>E001 | a; HTML | [the-a-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-a-element) | Hyperlink references, rel/target/referrer settings, captured TabIndex and [Anchor text](#anchor-text); navigation/activation and remaining author obligations separate. |
| <a id="e002"></a>E002 | abbr; HTML | [the-abbr-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-abbr-element) | Exact abbreviation role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e003"></a>E003 | address; HTML | [the-address-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-address-element) | [Address descendant restriction](#address-descendant-restriction): captured AddressHasForbiddenDescendant observes the explicit ordinary descendant prohibition. Nearest article/body contact scope, contact meaning, flow-content membership and full author conformance remain separate; applicable global states are independently queryable. |
| <a id="e004"></a>E004 | area; HTML | [the-area-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-area-element) | [Area geometry](#area-geometry), map membership and hyperlink settings; image association/hit testing not performed. |
| <a id="e005"></a>E005 | article; HTML | [the-article-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-article-element) | Exact self-contained syndicatable or reusable composition role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e006"></a>E006 | aside; HTML | [the-aside-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-aside-element) | Exact sidebar for tangentially related content role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e007"></a>E007 | audio; HTML | [the-audio-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-audio-element) | Own media Boolean/fetch/preload/loading states; playback/resources/track lifecycle host-owned. |
| <a id="e008"></a>E008 | b; HTML | [the-b-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-b-element) | Exact keywords role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e009"></a>E009 | base; HTML | [the-base-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-base-element) | [Document base](#document-base): independent first href/target selection, qualified fallback/frozen/current getters and actual native first/href/removal/adoption/clone hooks; URL and navigation execution external. |
| <a id="e010"></a>E010 | bdi; HTML | [the-bdi-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-bdi-element) | [Language and direction](#language-and-direction): source auto-direction and exclusion rules; no CSS bidi layout. |
| <a id="e011"></a>E011 | bdo; HTML | [the-bdo-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-bdo-element) | Required ltr/rtl own author rule plus captured Directionality; no rendering engine. |
| <a id="e012"></a>E012 | blockquote; HTML | [the-blockquote-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-blockquote-element) | Exact a section quoted from another source role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e013"></a>E013 | body; HTML | [the-body-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-body-element) | Exact document body role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e014"></a>E014 | br; HTML | [the-br-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-br-element) | Exact line break, e.g. in poem or postal address role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e015"></a>E015 | button; HTML | [the-button-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-button-element) | [Control facts](#control-facts), [Form keywords and submission](#form-keywords-and-submission), default-button membership; command target/classification gap distinct from event dispatch. |
| <a id="e016"></a>E016 | canvas; HTML | [the-canvas-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-canvas-element) | [Numeric content](#numeric-content) width/height defaults 300/150; context creation/drawing state belongs rendering/host. |
| <a id="e017"></a>E017 | caption; HTML | [the-caption-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-caption-element) | [Tables](#tables) supplies independent captured TableCaption selection and retains model caption data; caption setters/create/delete and author content conformance remain separate. |
| <a id="e018"></a>E018 | cite; HTML | [the-cite-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-cite-element) | Exact title of a work role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e019"></a>E019 | code; HTML | [the-code-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-code-element) | Exact computer code role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e020"></a>E020 | col; HTML | [the-col-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-col-element) | [Numeric content](#numeric-content) span and [Tables](#tables) semantic columns; layout geometry external. |
| <a id="e021"></a>E021 | colgroup; HTML | [the-colgroup-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-colgroup-element) | [Numeric content](#numeric-content) span and [Tables](#tables) explicit children/columns; geometry external. |
| <a id="e022"></a>E022 | data; HTML | [the-data-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-data-element) | Exact machine-readable value string; no universal numeric parser or value default. |
| <a id="e023"></a>E023 | datalist; HTML | [the-datalist-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-datalist-element) | [Datalist](#datalist) options/suggestions and input same-tree list relation; picker/UI external. |
| <a id="e024"></a>E024 | dd; HTML | [the-dd-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-dd-element) | [Description lists](#description-lists): captured grouping retains ordered dd value identities, including value-only groups; member content and full conformance remain separate. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e025"></a>E025 | del; HTML | [the-del-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-del-element) | [Modification datetime](#modification-datetime) and exact citation reference; no edit operation. |
| <a id="e026"></a>E026 | details; HTML | [the-details-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-details-element) | [Details](#details) first direct summary/name group, Boolean open and private construction exclusivity; live toggle/tracker mutation external. |
| <a id="e027"></a>E027 | dfn; HTML | [the-dfn-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-dfn-element) | [Defining term](#defining-term): captured DfnDefiningTerm selects exact own title, the sole qualifying abbr child title, or ordinary descendant text, retaining branch provenance and influencing Missing. Definition adequacy and content-model conformance remain separate; applicable global facts stay independently queryable. |
| <a id="e028"></a>E028 | dialog; HTML | [the-dialog-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-dialog-element) | Own closedby/open, qualified blocking-modal facts consumed by [Inertness](#inertness); show/close/top-layer transitions external. |
| <a id="e029"></a>E029 | div; HTML | [the-div-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-div-element) | Generic flow content remains retained; [description-list grouping](#description-lists) processes direct HTML div children one level, without splitting state at div boundaries. No generic div getter or full content-conformance certificate is implied. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e030"></a>E030 | dl; HTML | [the-dl-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-dl-element) | [Description lists](#description-lists): complete pinned dt/dd grouping returns source-ordered qualified identities, including empty-side groups; content conformance and human-name uniqueness remain separate. |
| <a id="e031"></a>E031 | dt; HTML | [the-dt-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-dt-element) | [Description lists](#description-lists): captured grouping retains ordered dt name identities, including name-only groups; human-name uniqueness, member content and full conformance remain separate. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e032"></a>E032 | em; HTML | [the-em-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-em-element) | Exact stress emphasis role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e033"></a>E033 | embed; HTML | [the-embed-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-embed-element) | Exact src/type/dimensions; plugin/resource representation external; common numbers do not define element sizing policy. |
| <a id="e034"></a>E034 | fieldset; HTML | [the-fieldset-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-fieldset-element) | [Control facts](#control-facts) disabled/first-legend and fieldset/form collections; current form pointer vs reset remains distinct. |
| <a id="e035"></a>E035 | figcaption; HTML | [the-figcaption-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-figcaption-element) | Exact caption for figure role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e036"></a>E036 | figure; HTML | [the-figure-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-figure-element) | [Figure caption](#figure-caption): captured FigureCaption selects the first direct HTML figcaption child at any position, or established no caption, preserving qualification and influencing Missing. First/last author placement, content conformance and exposure remain separate. |
| <a id="e037"></a>E037 | footer; HTML | [the-footer-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-footer-element) | [Footer scope](#footer-scope): captured FooterScope selects the nearest ordinary sectioning-content ancestor or the actual document body, including null, preserving qualification and influencing Missing. Author content conformance and accessibility/exposure remain separate. |
| <a id="e038"></a>E038 | form; HTML | [the-form-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) | [Labels and form ownership](#labels-and-form-ownership), qualified listed controls/length/default button/implicit condition, own method/enctype/autocomplete. Entry-list construction/submission execution outside query. |
| <a id="e039"></a>E039 | h1, h2, h3, h4, h5, h6; HTML | [the-h1,-h2,-h3,-h4,-h5,-and-h6-elements](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements) | [Captured global states](#captured-global-states) ComputedHeadingLevel; no rendering/outline inference from font size. |
| <a id="e040"></a>E040 | head; HTML | [the-head-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-head-element) | Exact container for document metadata role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e041"></a>E041 | header; HTML | [the-header-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-header-element) | Exact introductory or navigational-aids role/content retained; no HeaderScope query or full content-conformance certificate is advertised. |
| <a id="e042"></a>E042 | hgroup; HTML | [the-hgroup-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-hgroup-element) | Exact heading container role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e043"></a>E043 | hr; HTML | [the-hr-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-hr-element) | Exact thematic break role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e044"></a>E044 | html; HTML | [the-html-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-html-element) | Exact root element role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e045"></a>E045 | i; HTML | [the-i-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-i-element) | Exact alternate voice role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e046"></a>E046 | iframe; HTML | [the-iframe-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element) | Sandbox/allow/loading/referrer/target raw states; child navigable, srcdoc parsing/instantiation and permissions execution external. |
| <a id="e047"></a>E047 | img; HTML | [the-img-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-img-element) | [Responsive images](#responsive-images), [Hash and map reference](#hash-and-map-reference), ismap/controls author-context; [captured image dimensions](#captured-image-dimensions) interpret qualified getter results; source selection/decoder and CSS layout remain separate. |
| <a id="e048"></a>E048 | input; HTML | [the-input-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) | State-specific content producers, current flags/candidacy/Checkbox-Radio and selected text-state missingness/autocomplete/list suggestions in [Control facts](#control-facts); [captured image dimensions](#captured-image-dimensions) interpret state-selected getter results; bounded parser value sanitization uses the shared rules in [Control facts](#control-facts); live validity/stepping/file/color/text UI remain separate. |
| <a id="e049"></a>E049 | ins; HTML | [the-ins-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-ins-element) | [Modification datetime](#modification-datetime) and exact citation reference; no edit transition. |
| <a id="e050"></a>E050 | kbd; HTML | [the-kbd-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-kbd-element) | Exact user input role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e051"></a>E051 | label; HTML | [the-label-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-label-element) | [Labels and form ownership](#labels-and-form-ownership): labeled control, associated form, control labels; activation/event forwarding external. |
| <a id="e052"></a>E052 | legend; HTML | [the-legend-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-legend-element) | [Legend form owner](#legend-form-owner) supplies the immediate fieldset parent's actual form getter; first-legend disabledness remains in [Control facts](#control-facts). Caption selection and author content conformance remain separate. |
| <a id="e053"></a>E053 | li; HTML | [the-li-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-li-element) | [Lists](#lists) owner/value/ordinal consumes actual list participation; not CSS box geometry. |
| <a id="e054"></a>E054 | link; HTML | [the-link-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-link-element) | [Link destinations](#link-destinations), [Link relationships](#link-relationships) and [Icon sizes](#icon-sizes) supply local reports, qualified relationship facts, native flags/history and document selection; fetch/referrer/loading/srcset and CSSOM association execution remain with consumers. |
| <a id="e055"></a>E055 | main; HTML | [the-main-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-main-element) | Exact container for the dominant contents of the document role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e056"></a>E056 | map; HTML | [the-map-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-map-element) | [Map names](#map-names), MapAreas ordinary membership; usemap processing not full map-name conformance or hit testing. |
| <a id="e057"></a>E057 | mark; HTML | [the-mark-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-mark-element) | Exact highlight role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e058"></a>E058 | MathML math; MathML | [the-top-level-math-element](https://w3c.github.io/mathml-core/#the-top-level-math-element) | MathML namespace/open names retained; ordinary private HTML foreign construction/integration implemented, MathML language semantics/runtime delegated. |
| <a id="e059"></a>E059 | menu; HTML | [the-menu-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-menu-element) | ListOwner source list domain is actual source-selected owner; menu command/UI execution separate from preserving li children. |
| <a id="e060"></a>E060 | meta; HTML | [the-meta-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-meta-element) | [Metadata](#metadata) and [Encoding labels](#encoding-labels): captured description pairs/uniqueness, independent discriminator/content/media/charset-population structure, and static pragma content/same-state/charset-conflict observations; execution, other metadata and full element conformance remain separate. |
| <a id="e061"></a>E061 | meter; HTML | [the-meter-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-meter-element) | [Meter and progress](#meter-and-progress): coupled author constraints and captured effective state; rendering separate. |
| <a id="e062"></a>E062 | nav; HTML | [the-nav-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-nav-element) | Exact section with navigational links role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e063"></a>E063 | noscript; HTML | [the-noscript-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-noscript-element) | Exact ordinary content plus parser scripting-mode-specific handling; no inferred caller scripting fact. |
| <a id="e064"></a>E064 | object; HTML | [the-object-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-object-element) | Actual non-resettable form association/collections supported; data/type resource/navigable/plugin execution external. |
| <a id="e065"></a>E065 | ol; HTML | [the-ol-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-ol-element) | [Lists](#lists) exact start/reversed/type plus running ordinals; list CSS participation explicitly supplied. |
| <a id="e066"></a>E066 | optgroup; HTML | [the-optgroup-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-optgroup-element) | [Options and selection](#options-and-selection) pruning/barriers and disabledness; label text retained. |
| <a id="e067"></a>E067 | option; HTML | [the-option-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-option-element) | [Options and selection](#options-and-selection) text/label/value/index/form owner/current selectedness; live mutations/dirtiness transitions external. |
| <a id="e068"></a>E068 | output; HTML | [the-output-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-output-element) | [Control facts](#control-facts) current/default values and associations; output for-ID author relation separate gap. |
| <a id="e069"></a>E069 | p; HTML | [the-p-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-p-element) | Exact paragraph role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e070"></a>E070 | picture; HTML | [the-picture-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-picture-element) | [Responsive images](#responsive-images): source candidate parsing exists, complete source/media/sizes choice requires image/environment consumer. |
| <a id="e071"></a>E071 | pre; HTML | [the-pre-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-pre-element) | Exact block of preformatted text role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e072"></a>E072 | progress; HTML | [the-progress-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-progress-element) | [Meter and progress](#meter-and-progress): native author constraints and captured indeterminate/determinate state; presentation separate. |
| <a id="e073"></a>E073 | q; HTML | [the-q-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-q-element) | Exact quotation role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e074"></a>E074 | rp; HTML | [the-rp-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-rp-element) | Exact parenthesis for ruby annotation text role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e075"></a>E075 | rt; HTML | [the-rt-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-rt-element) | Exact ruby annotation text role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e076"></a>E076 | ruby; HTML | [the-ruby-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-ruby-element) | Exact ruby annotation(s) role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e077"></a>E077 | s; HTML | [the-s-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-s-element) | Exact inaccurate text role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e078"></a>E078 | samp; HTML | [the-samp-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-samp-element) | Exact computer output role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e079"></a>E079 | script; HTML | [the-script-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-script-element) | [Script type](#script-type); private Disabled-script preparation metadata retains source effects only, not a JS runtime or complete parser-conformance certificate. |
| <a id="e080"></a>E080 | search; HTML | [the-search-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-search-element) | Exact container for search controls role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e081"></a>E081 | section; HTML | [the-section-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-section-element) | Exact generic document or application section role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e082"></a>E082 | select; HTML | [the-select-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-select-element) | [Options and selection](#options-and-selection) options/index/value/placeholder/valueMissing, size/multiple and current flags; live reset/user-selection transitions external. |
| <a id="e083"></a>E083 | selectedcontent; HTML | [the-selectedcontent-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-selectedcontent-element) | Native ordinary selectedcontent children remain representable; [private standalone parser construction](README.md) performs source-qualified cloning/mirroring; live selection-driven mutation needs DOM owner. |
| <a id="e084"></a>E084 | slot; HTML | [the-slot-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-slot-element) | Exact slot/name; actual shadow/assigned/flat relationships supplied by DOM-owned capture, not rebuilt in a semantic query. Missing name selects empty slot name; assignment execution is external. |
| <a id="e085"></a>E085 | small; HTML | [the-small-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-small-element) | Exact side comment role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e086"></a>E086 | source; HTML | [the-source-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-source-element) | [Responsive images](#responsive-images) source srcset/context; media/image source resource choice and media-query environment external. |
| <a id="e087"></a>E087 | span; HTML | [the-span-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-span-element) | Exact generic phrasing container role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e088"></a>E088 | strong; HTML | [the-strong-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-strong-element) | Exact importance role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e089"></a>E089 | style; HTML | [the-style-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-style-element) | Exact stylesheet text/type/media/title/blocking; [Blocking tokens](#blocking-tokens) supplies local authored/explicit processing; CSS grammar/document CSSOM association and render-blocking lifecycle separate. |
| <a id="e090"></a>E090 | sub; HTML | [the-sub-and-sup-elements](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-sub-and-sup-elements) | Exact subscript role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e091"></a>E091 | summary; HTML | [the-summary-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-summary-element) | [Details](#details) first direct summary predicate and captured TabIndex default; disclosure toggle external. |
| <a id="e092"></a>E092 | sup; HTML | [the-sub-and-sup-elements](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-sub-and-sup-elements) | Exact superscript role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e093"></a>E093 | SVG svg; SVG | [elementdef-svg](https://w3c.github.io/svgwg/svg2-draft/struct.html#elementdef-svg) | SVG namespace/open names retained; ordinary private HTML foreign construction/integration and [restricted SVG script processing](README.md#private-svg-script) implemented, broader SVG language semantics/runtime delegated. |
| <a id="e094"></a>E094 | table; HTML | [the-table-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-table-element) | [Tables](#tables): independent TableCaption/TableHead/TableFoot getters, static bodies/rows/cells and native collection indices, separately from the slot/column/header model; no live collections/mutation/layout or full author-conformance certificate. |
| <a id="e095"></a>E095 | tbody; HTML | [the-tbody-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-tbody-element) | [Tables](#tables) direct row membership; geometric layout and live DOM collection external. |
| <a id="e096"></a>E096 | td; HTML | [the-td-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-td-element) | [Tables](#tables) cell/span/header model and native TableCellIndex in the immediate row's cells; geometry, live getters and mutation remain separate. |
| <a id="e097"></a>E097 | template; HTML | [the-template-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-template-element) | Native separate template contents and explicit declarative shadow effects; [private fresh ordinary templates](README.md#private-ordinary-templates) implement contents/patching. [Private context fragments](README.md#private-context-fragments) retain qualified template caches and shadow construction effects. [Template marker reference](#template-marker-reference) reports captured same-tree name witnesses and first selection. [Bounded document/fragment reports](README.md#decoded-unicode-parsing) are available; live DOM shadow attachment remains unfinished. |
| <a id="e098"></a>E098 | textarea; HTML | [the-textarea-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) | [Control lengths](#control-lengths), cols/rows/wrap, captured defaultValue/autocomplete/text assistance/current flags and TextareaValueMissing in [Control facts](#control-facts); editing/reset/validation execution separate. |
| <a id="e099"></a>E099 | tfoot; HTML | [the-tfoot-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-tfoot-element) | [Tables](#tables) supplies independent first direct TableFoot selection and direct row membership/footer-last tableRows; no live mutation or full author-conformance certificate. |
| <a id="e100"></a>E100 | th; HTML | [the-th-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-th-element) | [Tables](#tables) cell/span/header model and native TableCellIndex, plus [Table header scope](#table-header-scope); live getters, mutation and layout remain separate. |
| <a id="e101"></a>E101 | thead; HTML | [the-thead-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-thead-element) | [Tables](#tables) supplies independent first direct TableHead selection and direct row membership/header-first tableRows; no live mutation or full author-conformance certificate. |
| <a id="e102"></a>E102 | time; HTML | [the-time-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-time-element) | [Time element](#time-element) ten-format author/processing and captured explicit-value/text selection; inverse/host-zone conversion not supplied. |
| <a id="e103"></a>E103 | title; HTML | [the-title-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-title-element) | [Document title](#document-title): exact TitleText child-text getter and qualified DocumentTitle selection with ASCII whitespace normalization; setters and lifecycle notification remain separate. |
| <a id="e104"></a>E104 | tr; HTML | [the-tr-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-tr-element) | [Tables](#tables) direct td/th membership and native TableRowIndex/TableSectionRowIndex; live collection, WebIDL conversion and insertion/deletion remain separate. |
| <a id="e105"></a>E105 | track; HTML | [the-track-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-track-element) | Own kind/default/src/srclang/label retained; WebVTT parsing, media readiness and track selection external. |
| <a id="e106"></a>E106 | u; HTML | [the-u-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-u-element) | Exact unarticulated annotation role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e107"></a>E107 | ul; HTML | [the-ul-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-ul-element) | [Lists](#lists): ListOwner recognizes ul/ol/menu and ListOrdinal interprets participating li children with qualified display/box facts. No ul starting-value getter or full content-conformance certificate is implied. Applicable global facts remain independently queryable. |
| <a id="e108"></a>E108 | var; HTML | [the-var-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-var-element) | Exact variable role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e109"></a>E109 | video; HTML | [the-video-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-video-element) | Own media Boolean/fetch/preload/loading/dimensions/poster; decoding/playback/media UI external. |
| <a id="e110"></a>E110 | wbr; HTML | [the-wbr-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-wbr-element) | Exact line breaking opportunity role/ordinary content is retained by raw construction; no source-specific computed getter or full content-conformance certificate is advertised. Applicable global language/editability/heading facts remain independently queryable. |
| <a id="e111"></a>E111 | autonomous custom elements; HTML | [autonomous-custom-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autonomous-custom-element) | [Custom element names](#custom-element-names): checked local name and exact HTML-name emission, plus captured actual state/association and independent `is` author obligations. Registry execution and complete element conformance remain separate. |

## Contextual attribute identities

Each row inherits exact supplied expanded-name/raw-state retention and standalone
per-value admission from [raw and construction](#raw-and-construction). Missing
is not empty, and wrong namespace/name does not select that grammar. An absent
specialized overlay does not erase raw values or prohibit generic construction.
The final column is an authored rule association; it is not inferred from the
index's syntax cell. The source column preserves all printed links, including
cross-language references, so a contextual defining link is not lost.

| Identity | Exact no-namespace item; printed HTML context | Source links | Raw/author/processing/consumer association |
| --- | --- | --- | --- |
| <a id="a001"></a>A001 | abbr; th | [attr-th-abbr](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-th-abbr); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | Text abbreviation for header cell; exact text retained, no missing/invalid default. TableModel binds semantic headers; abbreviation prose is not inferred. |
| <a id="a002"></a>A002 | accept; input | [attr-input-accept](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-accept); [file-upload-state-(type=file)](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#file-upload-state-(type=file)); [set-of-comma-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#set-of-comma-separated-tokens); [valid-mime-type-with-no-parameters](https://mimesniff.spec.whatwg.org/#valid-mime-type-with-no-parameters) | [File accept](#file-accept): input File author/context and token processing; no MIME certification or picker execution. |
| <a id="a003"></a>A003 | accept-charset; form | [attr-form-accept-charset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-form-accept-charset); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [ascii-case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) | [Encoding labels](#encoding-labels): form_accept_charset checks exact UTF-8 spelling and retains accepted casing; submission encoding selection remains separate. |
| <a id="a004"></a>A004 | accesskey; HTML elements | [the-accesskey-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-accesskey-attribute); [ordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ordered-set-of-unique-space-separated-tokens); [string-is](https://infra.spec.whatwg.org/#string-is) | [Access keys](#access-keys): checked unique single-code-point author set and ordered processing candidates; keyboard assignment/focus/activation separate. |
| <a id="a005"></a>A005 | action; form | [attr-fs-action](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-action); [concept-url](https://url.spec.whatwg.org/#concept-url); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [valid-non-empty-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-empty-url-potentially-surrounded-by-spaces) | [URLs and references](#urls-and-references): form action nonempty URL author condition; actual document/submitter fallback and submission are contextual, no guessed base. |
| <a id="a006"></a>A006 | allow; iframe | [attr-iframe-allow](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-allow); [permissions-policy](https://w3c.github.io/webappsec-feature-policy/#permissions-policy); [the-iframe-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element); [serialized-permissions-policy](https://w3c.github.io/webappsec-feature-policy/#serialized-permissions-policy) | Permissions Policy delegation; iframe allow string retained. HTML iframe declaration integration and host policy/fetch are distinct; no policy validator advertised. |
| <a id="a007"></a>A007 | allowfullscreen; iframe | [attr-iframe-allowfullscreen](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-allowfullscreen); [the-iframe-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element); [dom-element-requestfullscreen](https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): AllowFullscreen. Permissions/fullscreen transition remains host-owned. |
| <a id="a008"></a>A008 | alpha; input | [attr-input-alpha](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-alpha); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Alpha with actual input state; [controlled Color conversion](#color-input) is separate from attribute interpretation and live well updates. |
| <a id="a009"></a>A009 | alt; area; img; input | [attr-area-alt](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-area-alt); [attr-img-alt](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-alt); [attr-input-alt](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-alt); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | Exact alternative text for area/img/input; raw text retained. [Input Image and embedded dimensions](#input-image-and-embedded-dimensions) supplies the Image-state input alt required-presence/nonempty report. Label adequacy remains an independent source requirement. |
| <a id="a010"></a>A010 | as; link | [attr-link-as](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-as); [attr-link-rel](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-rel); [link-type-preload](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#link-type-preload); [link-type-modulepreload](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#link-type-modulepreload); [preload-destination](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#preload-destination); [module-preload-destination](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#module-preload-destination) | [Link destinations](#link-destinations): native thirteen-state as vocabulary, whole ASCII-insensitive content report and independent actual-rel author requirements; symbolic branch decisions retain no-state recovery separately from requests/execution. |
| <a id="a011"></a>A011 | async; script | [attr-script-async](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-async); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Async with script-kind applicability; scheduling/fetch not computed. |
| <a id="a012"></a>A012 | autocapitalize; HTML elements | [attr-autocapitalize](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize); [attr-autocapitalize-on](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize-on); [attr-autocapitalize-off](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize-off); [attr-autocapitalize-none](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize-none); [attr-autocapitalize-sentences](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize-sentences); [attr-autocapitalize-words](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize-words); [attr-autocapitalize-characters](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize-characters) | [Enumerations](#enumerations) and [Captured global states](#captured-global-states): default hint, used hint and inheritance are distinct. |
| <a id="a013"></a>A013 | autocomplete; form | [attr-form-autocomplete](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-form-autocomplete) | [Enumerations](#enumerations): form autocomplete defaults On; not the control token grammar. |
| <a id="a014"></a>A014 | autocomplete; input; select; textarea | [attr-fe-autocomplete](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fe-autocomplete); [autofill-field](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autofill-field) | [Control autocomplete](#control-autocomplete): input/select/textarea token grammar and mantle processing, with supplied input state and actual form owner where required. |
| <a id="a015"></a>A015 | autocorrect; HTML elements | [attr-autocorrect](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect); [attr-autocorrect-on](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect-on); [attr-autocorrect-off](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect-off) | [Enumerations](#enumerations) and [Captured global states](#captured-global-states): On default vs inherited used state. |
| <a id="a016"></a>A016 | autofocus; HTML elements | [attr-fe-autofocus](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fe-autofocus); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Autofocus. Actual focus candidate collection, focusability and focus transition require host/DOM owner. |
| <a id="a017"></a>A017 | autoplay; audio; video | [attr-media-autoplay](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-autoplay); [media-resource](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#media-resource); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Autoplay. Media playback eligibility/transition remains host media owner. |
| <a id="a018"></a>A018 | blocking; link; script; style | [attr-link-blocking](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-blocking); [attr-script-blocking](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-blocking); [attr-style-blocking](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-style-blocking); [potentially-render-blocking](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#potentially-render-blocking); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens) | [Blocking tokens](#blocking-tokens): exact authored set, case-folded explicit Render/Empty processing and actual link/script context; render-blocking lifecycle separate. |
| <a id="a019"></a>A019 | charset; meta | [attr-meta-charset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-charset); [character-encoding-declaration](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#character-encoding-declaration) | [Encoding labels](#encoding-labels): meta_charset checks exact UTF-8 spelling with independently Unchecked declaration context; [Metadata](#metadata) supplies specified-charset ordinary-Document multiplicity and independent content-type pragma/charset-presence conflict, not broader encoding-declaration conformance. |
| <a id="a020"></a>A020 | checked; input | [attr-input-checked](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-checked); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Checked, restricted input states. CurrentCheckedness/DirtyCheckednessFlag and CheckableInputValueMissing use actual checkedness, not authored checked as current state. |
| <a id="a021"></a>A021 | cite; blockquote; del; ins; q | [attr-blockquote-cite](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-blockquote-cite); [attr-mod-cite](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-mod-cite); [attr-q-cite](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-q-cite); [valid-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-url-potentially-surrounded-by-spaces) | [URLs and references](#urls-and-references): citation references on blockquote/q/ins/del; absent remains absent, no base is invented. |
| <a id="a022"></a>A022 | class; HTML elements | [classes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#classes); [set-of-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#set-of-space-separated-tokens) | [Token sequences](#token-sequences): unordered space-token class set, comparison according to document mode/source. No selector matching or reflected live token list. |
| <a id="a023"></a>A023 | closedby; dialog | [attr-dialog-closedby](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dialog-closedby); [attr-dialog-closedby-any](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dialog-closedby-any); [attr-dialog-closedby-closerequest](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dialog-closedby-closerequest); [attr-dialog-closedby-none](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dialog-closedby-none) | [Enumerations](#enumerations): closedby missing/invalid Auto. Close-watcher/request-close/modal transitions are external execution. |
| <a id="a024"></a>A024 | color; link | [attr-link-color](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-color); [attr-link-rel](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-rel); [typedef-color](https://drafts.csswg.org/css-color/#typedef-color) | Link mask-icon rel author restriction remains an HTML condition; CSS color grammar delegated. No UA color processing requirement is supplied by this clause. |
| <a id="a025"></a>A025 | colorspace; input | [attr-input-colorspace](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-colorspace); [attr-input-colorspace-limited-srgb](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-colorspace-limited-srgb); [attr-input-colorspace-display-p3](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-colorspace-display-p3) | [Color input](#color-input): input_colorspace supplies limited-srgb/display-p3 recognition and missing/invalid Limited sRGB fallback; the actual-input-state companion independently observes Color-only author presence. CSS owns color grammar; [controlled Color construction](README.md#controlled-color-input) consumes it separately from attribute interpretation. Interactive wells remain external. |
| <a id="a026"></a>A026 | cols; textarea | [attr-textarea-cols](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-cols); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | [Numeric content](#numeric-content): textarea cols positive author quantity; missing/failed/zero recovery 20. |
| <a id="a027"></a>A027 | colspan; td; th | [attr-tdth-colspan](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-tdth-colspan); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | [Numeric content](#numeric-content): cell colspan author 1..1000; missing/failed/zero recovery 1, clamp 1000. [Tables](#tables) owns slots. |
| <a id="a028"></a>A028 | command; button | [attr-button-command](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command); [attr-button-command-toggle-popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command-toggle-popover); [attr-button-command-show-popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command-show-popover); [attr-button-command-hide-popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command-hide-popover); [attr-button-command-close](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command-close); [attr-button-command-request-close](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command-request-close); [attr-button-command-show-modal](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command-show-modal); [attr-button-command-custom](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command-custom) | [Commands and popovers](#commands-and-popovers): standard/Custom/Unknown content, checked native emission and retained Custom payload; [captured command validity](#command-target-relations); activation remains distinct. |
| <a id="a029"></a>A029 | commandfor; button | [attr-button-commandfor](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-commandfor); [concept-id](https://dom.spec.whatwg.org/#concept-id) | [Command target relations](#command-target-relations): actual pointer scope and first ordinary-tree ID readback; author same-tree reference conformance and activation remain distinct. |
| <a id="a030"></a>A030 | content; meta | [attr-meta-content](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-content); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Metadata](#metadata): captured description preserves optional exact content/effective missing empty independently of required presence; static pragma content separates required presence, HTML author framing and lexical recovery from delegated grammar/execution. Descriptive appropriateness and other metadata-name value rules remain separate. |
| <a id="a031"></a>A031 | contenteditable; HTML elements | [attr-contenteditable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable) | [Enumerations](#enumerations) and [Captured global states](#captured-global-states): editing host/editable inheritance plus designMode facts; no editing engine. |
| <a id="a032"></a>A032 | controls; audio; video img; | [attr-media-controls](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-controls); [attr-img-controls](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-controls); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): media controls; img controls has nonempty alt author-context readback. UI creation/playback external. |
| <a id="a033"></a>A033 | coords; area | [attr-area-coords](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-area-coords); [image-map](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-map); [valid-list-of-floating-point-numbers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-list-of-floating-point-numbers) | [Area geometry](#area-geometry): coords exact author mathematical quantities and independent recovery geometry coupled to shape. |
| <a id="a034"></a>A034 | crossorigin; audio; img; link; script; video | [attr-media-crossorigin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-crossorigin); [attr-img-crossorigin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-crossorigin); [attr-link-crossorigin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-crossorigin); [attr-script-crossorigin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-crossorigin); [attr-crossorigin-anonymous-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-crossorigin-anonymous-keyword); [attr-crossorigin-use-credentials-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-crossorigin-use-credentials-keyword) | [Enumerations](#enumerations): crossorigin no CORS when missing, Anonymous on invalid/empty; script context explicit. Fetch/CORS enforcement external. |
| <a id="a035"></a>A035 | data; object | [attr-object-data](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-object-data); [valid-non-empty-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-empty-url-potentially-surrounded-by-spaces) | [URLs and references](#urls-and-references): object data URL and resource representation depend on document/resource context; immutable value does not load it. |
| <a id="a036"></a>A036 | datetime; del; ins | [attr-mod-datetime](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-mod-datetime); [valid-date-string-with-optional-time](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-date-string-with-optional-time) | [Modification datetime](#modification-datetime): ins/del strict optional-time date author grammar; associated timestamp excludes Time-only recovery. |
| <a id="a037"></a>A037 | datetime; time | [attr-time-datetime](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-time-datetime); [valid-month-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-month-string); [valid-date-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-date-string); [valid-yearless-date-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-yearless-date-string); [valid-time-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-time-string); [valid-local-date-and-time-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-local-date-and-time-string); [valid-time-zone-offset-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-time-zone-offset-string); [valid-global-date-and-time-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-global-date-and-time-string); [valid-week-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-week-string); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer); [valid-duration-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-duration-string) | [Time element](#time-element): time datetime explicit value wins; otherwise actual text/descendants and source child restrictions. |
| <a id="a038"></a>A038 | decoding; img | [attr-img-decoding](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-decoding); [attr-img-decoding-sync](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-decoding-sync); [attr-img-decoding-async](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-decoding-async); [attr-img-decoding-auto](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-decoding-auto) | [Enumerations](#enumerations): decoding missing/invalid Auto; image decoder execution external. |
| <a id="a039"></a>A039 | default; track | [attr-track-default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-default); [text-track](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text-track); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): TrackDefault. Same-parent uniqueness and track-selection execution are separate, not proved by presence. |
| <a id="a040"></a>A040 | defer; script | [attr-script-defer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-defer); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Defer with script context; execution scheduling external. |
| <a id="a041"></a>A041 | dir; HTML elements | [attr-dir](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dir); [the-directionality](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-directionality); [attr-dir-ltr](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dir-ltr); [attr-dir-rtl](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dir-rtl); [attr-dir-auto](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dir-auto) | [Enumerations](#enumerations) and [Language and direction](#language-and-direction): own dir missing/invalid Undefined; actual inheritance/auto scanning separately bound. |
| <a id="a042"></a>A042 | dir; bdo | [the-bdo-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-bdo-element); [the-directionality](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-directionality); [attr-dir-ltr](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dir-ltr); [attr-dir-rtl](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dir-rtl) | Same dir owner as A041, with bdo required ltr/rtl author constraint; Undefined recovery is not author conformity. |
| <a id="a043"></a>A043 | dirname; input; textarea | [attr-fe-dirname](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fe-dirname); [the-directionality](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-directionality); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Direction submission name](#direction-submission-name): checked exact nonempty native/report values and complete input-state presence checks; captured Directionality exists, submission entry assembly remains separate. |
| <a id="a044"></a>A044 | disabled; button; input; optgroup; option; select; textarea; form-associated custom elements | [attr-fe-disabled](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fe-disabled); [attr-optgroup-disabled](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-optgroup-disabled); [attr-option-disabled](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-option-disabled); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence) and [Control facts](#control-facts): disabled own content vs inherited fieldset/optgroup and actual custom classification. |
| <a id="a045"></a>A045 | disabled; fieldset | [attr-fieldset-disabled](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fieldset-disabled); [the-legend-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-legend-element); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | Fieldset disabled presence plus first-legend ancestry algorithm in [Control facts](#control-facts). |
| <a id="a046"></a>A046 | disabled; link | [attr-link-disabled](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-disabled); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): stylesheet_disabled requires actual stylesheet rel context; captured StylesheetDisabledAttribute uses it. CSSOM lifecycle separate. |
| <a id="a047"></a>A047 | download; a; area | [attr-hyperlink-download](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-download) | [URLs and references](#urls-and-references): download filename string is retained independently of href presence; download/navigation/security execution external. |
| <a id="a048"></a>A048 | draggable; HTML elements | [attr-draggable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-draggable) | [Enumerations](#enumerations): draggable missing/invalid Auto. Computation/drag data and event dispatch not implemented by the own-state report. |
| <a id="a049"></a>A049 | enctype; form | [attr-fs-enctype](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-enctype); [entry-list](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#entry-list); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [attr-fs-enctype-urlencoded](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-enctype-urlencoded); [attr-fs-enctype-formdata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-enctype-formdata); [attr-fs-enctype-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-enctype-text) | [Enumerations](#enumerations): form enctype defaults UrlEncoded; submitter override/report and actual encoding/submission distinct. |
| <a id="a050"></a>A050 | enterkeyhint; HTML elements | [attr-enterkeyhint](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-enterkeyhint); [attr-enterkeyhint-keyword-enter](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-enterkeyhint-keyword-enter); [attr-enterkeyhint-keyword-done](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-enterkeyhint-keyword-done); [attr-enterkeyhint-keyword-go](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-enterkeyhint-keyword-go); [attr-enterkeyhint-keyword-next](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-enterkeyhint-keyword-next); [attr-enterkeyhint-keyword-previous](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-enterkeyhint-keyword-previous); [attr-enterkeyhint-keyword-search](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-enterkeyhint-keyword-search); [attr-enterkeyhint-keyword-send](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-enterkeyhint-keyword-send) | [Enumerations](#enumerations): enterkeyhint has no missing/invalid state; keyboard UI suggestion external. |
| <a id="a051"></a>A051 | fetchpriority; img; link; script | [attr-img-fetchpriority](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-fetchpriority); [attr-link-fetchpriority](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-fetchpriority); [attr-script-fetchpriority](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-fetchpriority); [request-priority](https://fetch.spec.whatwg.org/#request-priority); [concept-fetch](https://fetch.spec.whatwg.org/#concept-fetch); [attr-fetchpriority-auto](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fetchpriority-auto); [attr-fetchpriority-high](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fetchpriority-high); [attr-fetchpriority-low](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fetchpriority-low) | [Enumerations](#enumerations): fetchpriority defaults Auto; actual resource priority external. |
| <a id="a052"></a>A052 | for; label | [attr-label-for](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-label-for); [concept-id](https://dom.spec.whatwg.org/#concept-id) | [Labels and form ownership](#labels-and-form-ownership): label for first appropriate same-tree ID/labeled-control relation; omission selects descendant rule. |
| <a id="a053"></a>A053 | for; output | [attr-output-for](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-output-for); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens) | Output for unordered unique ID tokens: common splitting/sets available, complete same-tree output-for relation author check absent; OutputValue is a different query. |
| <a id="a054"></a>A054 | for; template | [attr-template-for](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-template-for) | [Template marker reference](#template-marker-reference): captured same-tree named marker/start PI relation and independent first selection; private fresh parser implements actual patching; not label/output grammar. |
| <a id="a055"></a>A055 | form; button; fieldset; input; object; output; select; textarea; form-associated custom elements | [attr-fae-form](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fae-form); [the-form-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element); [concept-id](https://dom.spec.whatwg.org/#concept-id) | [Labels and form ownership](#labels-and-form-ownership): current established pointer differs reset interpretation; actual same-tree ID/default ancestor and custom facts. |
| <a id="a056"></a>A056 | formaction; button; input | [attr-fs-formaction](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-formaction); [concept-url](https://url.spec.whatwg.org/#concept-url); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [valid-non-empty-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-empty-url-potentially-surrounded-by-spaces) | [URLs and references](#urls-and-references): formaction submitter applicability/URL+actual-owner fallback not provided by method/enctype overrides. |
| <a id="a057"></a>A057 | formenctype; button; input | [attr-fs-formenctype](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-formenctype); [entry-list](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#entry-list); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [attr-fs-enctype-urlencoded](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-enctype-urlencoded); [attr-fs-enctype-formdata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-enctype-formdata); [attr-fs-enctype-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-enctype-text) | [Form keywords and submission](#form-keywords-and-submission): formenctype missing None, invalid UrlEncoded, actual form fallback in captured query. |
| <a id="a058"></a>A058 | formmethod; button; input | [attr-fs-formmethod](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-formmethod); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [attr-fs-method-get-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-method-get-keyword); [attr-fs-method-post-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-method-post-keyword); [attr-fs-method-dialog-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-method-dialog-keyword) | [Form keywords and submission](#form-keywords-and-submission): formmethod missing None, invalid Get; actual owner fallback independently qualified. |
| <a id="a059"></a>A059 | formnovalidate; button; input | [attr-fs-formnovalidate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-formnovalidate); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence) plus SubmitterNoValidate; actual submit-button classification/owner needed, no constraint-validation execution. |
| <a id="a060"></a>A060 | formtarget; button; input | [attr-fs-formtarget](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-formtarget); [navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [valid-navigable-target-name-or-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-navigable-target-name-or-keyword) | [Navigable target](#navigable-target): formtarget lexical target; actual input/button submitter eligibility remains unchecked by this local report, and navigable selection external. |
| <a id="a061"></a>A061 | headers; td; th | [attr-tdth-headers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-tdth-headers); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens) | [Tables](#tables): author unordered unique tokens must identify same-table th elements; retained raw/token encounter order and TableModel header association are separate processing observations. |
| <a id="a062"></a>A062 | headingoffset; HTML elements | Discovery: [attr-tabindex](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-tabindex); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer); defining: [attr-headingoffset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-headingoffset); [computed heading level](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-an-element's-computed-heading-level) | [Heading offset](#heading-offset): local author 0..8 report/emission preserves exact recovery; captured level traversal remains separate. |
| <a id="a063"></a>A063 | headingreset; HTML elements | Discovery: [attr-tabindex](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-tabindex); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute); defining: [attr-headingreset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-headingreset); [computed heading offset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-an-element's-computed-heading-offset) | [Boolean presence](#boolean-presence) and ComputedHeadingLevel traversal reset in [Captured global states](#captured-global-states). |
| <a id="a064"></a>A064 | height; canvas; embed; iframe; img; input; object; source (in picture); video | [attr-canvas-height](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-canvas-height); [attr-dim-height](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dim-height); [the-picture-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-picture-element); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | Canvas height content defaults 150 via [Numeric content](#numeric-content). [Input Image and embedded dimensions](#input-image-and-embedded-dimensions) supplies other roles' local grammar/recovery and source-parent/input-state qualification. The [qualified natural-dimension query](#input-image-and-embedded-dimensions) supplies independent omission and exact paired-author checks; [captured image dimensions](#captured-image-dimensions) interpret img/input getter results over completed facts. Other reflection roles remain separate. |
| <a id="a065"></a>A065 | hidden; HTML elements | [attr-hidden](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hidden); [attr-hidden-until-found](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hidden-until-found) | [Enumerations](#enumerations): hidden missing NotHidden, invalid/empty Hidden; no rendered/layout boolean derived from it. |
| <a id="a066"></a>A066 | high; meter | [attr-meter-high](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meter-high); [valid-floating-point-number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-floating-point-number) | [Meter and progress](#meter-and-progress): meter high author pair constraints; effective high defaults maximum then clamps at low/maximum. |
| <a id="a067"></a>A067 | href; a; area | [attr-hyperlink-href](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-href); [hyperlink](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hyperlink); [valid-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-url-potentially-surrounded-by-spaces) | [URLs and references](#urls-and-references): a/area href defines hyperlink only when present; actual resolution/activation requires URL/document context. |
| <a id="a068"></a>A068 | href; link | [attr-link-href](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-href); [hyperlink](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hyperlink); [valid-non-empty-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-empty-url-potentially-surrounded-by-spaces) | [URLs and references](#urls-and-references): link href resource reference; rel-dependent requirements/fetch not established by storage. |
| <a id="a069"></a>A069 | href; base | [attr-base-href](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-base-href); [document-base-url](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-base-url); [valid-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-url-potentially-surrounded-by-spaces) | [Document base](#document-base): qualified document fallback and first-base frozen URL; independent current BaseHref getter, source-required native freezing and pending-response refusal; no inferred URL from source identity. |
| <a id="a070"></a>A070 | hreflang; a; link | [attr-hyperlink-hreflang](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-hreflang); [attr-link-hreflang](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-hreflang) | BCP47 hreflang string is retained unchecked; advisory destination language does not override captured lang. No full external grammar validator. |
| <a id="a071"></a>A071 | http-equiv; meta | [attr-meta-http-equiv](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-http-equiv); [attr-meta-http-equiv-keyword-content-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-http-equiv-keyword-content-type); [attr-meta-http-equiv-keyword-default-style](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-http-equiv-keyword-default-style); [attr-meta-http-equiv-keyword-refresh](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-http-equiv-keyword-refresh); [attr-meta-http-equiv-keyword-x-ua-compatible](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-http-equiv-keyword-x-ua-compatible); [attr-meta-http-equiv-keyword-content-security-policy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-http-equiv-keyword-content-security-policy) | [Metadata](#metadata): exact seven-state recognition with no invented defaults, five conforming native author keywords, static content candidates/framing/refresh recovery and independent ordinary-Document same-state/charset-presence observations; delegated URL/CSP grammar, processing history and execution remain separate. |
| <a id="a072"></a>A072 | id; HTML elements | [the-id-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-id-attribute); [concept-id](https://dom.spec.whatwg.org/#concept-id); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [IDs](#ids): native/local nonempty/no-ASCII-whitespace evidence plus captured `IdAuthor` for independent ordinary-tree NoId/Unique/Duplicate/Missing; local present context remains Unchecked, with no overall conformance certificate. |
| <a id="a073"></a>A073 | imagesizes; link | [attr-link-imagesizes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-imagesizes); [attr-link-rel](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-rel); [link-type-preload](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#link-type-preload); [valid-source-size-list](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-source-size-list) | [Responsive images](#responsive-images): imagesizes requires preload/image context and width-descriptor relation; sizes-list parser absent. |
| <a id="a074"></a>A074 | imagesrcset; link | [attr-link-imagesrcset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-imagesrcset); [attr-link-rel](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-rel); [link-type-preload](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#link-type-preload); [image-candidate-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-candidate-string) | [Responsive images](#responsive-images): link_imagesrcset author+recovery, actual supplied rel/as/sizes context; no resource choice. |
| <a id="a075"></a>A075 | inert; HTML elements | [the-inert-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-inert-attribute); [inert](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#inert); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence) plus Inertness actual flat facts/modal qualification in [Inertness](#inertness). |
| <a id="a076"></a>A076 | inputmode; HTML elements | [attr-inputmode](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode); [attr-inputmode-keyword-none](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode-keyword-none); [attr-inputmode-keyword-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode-keyword-text); [attr-inputmode-keyword-tel](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode-keyword-tel); [attr-inputmode-keyword-email](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode-keyword-email); [attr-inputmode-keyword-url](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode-keyword-url); [attr-inputmode-keyword-numeric](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode-keyword-numeric); [attr-inputmode-keyword-decimal](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode-keyword-decimal); [attr-inputmode-keyword-search](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-inputmode-keyword-search) | [Enumerations](#enumerations): inputmode no missing/invalid state; virtual keyboard selection external. |
| <a id="a077"></a>A077 | integrity; link; script | [attr-link-integrity](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-integrity); [attr-script-integrity](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-integrity); [refsSRI](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSRI); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | SRI grammar/cryptographic verification delegated; exact integrity metadata retained. HTML link/script placement and fetch integration need actual context, not success from string storage. |
| <a id="a078"></a>A078 | is; HTML elements | [attr-is](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-is); [customized-built-in-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#customized-built-in-element); [valid-custom-element-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-custom-element-name) | [Custom element names](#custom-element-names): raw-first local `is` report remains context Unchecked for present content; captured `IsAttributeAuthorContext` separately observes current definition matching and the actual autonomous-element prohibition. |
| <a id="a079"></a>A079 | ismap; img | [attr-img-ismap](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-ismap); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): ismap own true independent of prohibited author context; ImageMapAttribute checks actual a[href] ancestor. Server-side navigation coordinates external. |
| <a id="a080"></a>A080 | itemid; HTML elements | [attr-itemid](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-itemid); [global-identifier](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-identifier); [valid-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-url-potentially-surrounded-by-spaces) | [Microdata](#microdata): separate itemid/itemscope/itemtype author eligibility and current-context identifier; qualified actual URL failure gives no identifier, while missing facts remain Missing. |
| <a id="a081"></a>A081 | itemprop; HTML elements | [names:-the-itemprop-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#names:-the-itemprop-attribute); [property-names](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#property-names); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens); [syntax-url-absolute](https://url.spec.whatwg.org/#syntax-url-absolute); [defined-property-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#defined-property-name) | [Microdata](#microdata): exact property-name occurrences, processing deduplication, typed/untyped receiver paths and independently qualified vocabulary/value checks. |
| <a id="a082"></a>A082 | itemref; HTML elements | [attr-itemref](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-itemref); [referenced](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#referenced); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens) | [Microdata](#microdata): author uniqueness/itemscope eligibility and actual same-tree first-ID lookup; processing retains occurrences and reports crawl revisits independently. |
| <a id="a083"></a>A083 | itemscope; HTML elements | [attr-itemscope](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-itemscope); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): canonical Itemscope presence; [Microdata](#microdata) separately owns captured item/property extraction. |
| <a id="a084"></a>A084 | itemtype; HTML elements | [attr-itemtype](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-itemtype); [item-types](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#item-types); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens); [syntax-url-absolute](https://url.spec.whatwg.org/#syntax-url-absolute) | [Microdata](#microdata): exact own types and receiving-item type paths; local nonempty/uniqueness and external absolute-URL/definition/same-vocabulary checks stay separate. |
| <a id="a085"></a>A085 | kind; track | [attr-track-kind](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-kind); [attr-track-kind-keyword-subtitles](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-kind-keyword-subtitles); [attr-track-kind-keyword-captions](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-kind-keyword-captions); [attr-track-kind-keyword-descriptions](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-kind-keyword-descriptions); [attr-track-kind-keyword-chapters](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-kind-keyword-chapters); [attr-track-kind-keyword-metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-kind-keyword-metadata) | [Enumerations](#enumerations): kind missing Subtitles, invalid Metadata; timed-track parsing/selection external. |
| <a id="a086"></a>A086 | label; optgroup; option; track | [attr-optgroup-label](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-optgroup-label); [attr-option-label](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-option-label); [attr-track-label](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-label); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | OptionLabel uses the present label attribute value (including empty) or absent-label text fallback in [Options and selection](#options-and-selection); optgroup label and track label remain exact text without inherited numeric/default grammar. Required label/context constraints are not certified. |
| <a id="a087"></a>A087 | lang; HTML elements | [attr-lang](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-lang); [language](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#language) | [Language and direction](#language-and-direction): exact lang plus qualified XML lang precedence and unknown/empty/inherited distinction; BCP47 validation external. |
| <a id="a088"></a>A088 | list; input | [attr-input-list](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-list); [concept-id](https://dom.spec.whatwg.org/#concept-id) | [Datalist](#datalist): input list same-tree ID first matching datalist and suggestions filter; actual current values/facts where requested. |
| <a id="a089"></a>A089 | loading; iframe; img; audio; video | [attr-iframe-loading](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-loading); [attr-img-loading](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-loading); [attr-media-loading](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-loading); [attr-loading-lazy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-loading-lazy); [attr-loading-eager](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-loading-eager) | [Enumerations](#enumerations): loading Eager default for iframe/img/audio/video; resource/lazy-load execution external. |
| <a id="a090"></a>A090 | loop; audio; video | [attr-media-loop](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-loop); [media-resource](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#media-resource); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Loop; media repeat transition external. |
| <a id="a091"></a>A091 | low; meter | [attr-meter-low](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meter-low); [valid-floating-point-number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-floating-point-number) | [Meter and progress](#meter-and-progress): low author pair constraints; effective default minimum then clamp. |
| <a id="a092"></a>A092 | max; input | [attr-input-max](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-max) | [Input quantities](#input-quantities): max Number/Range and five temporal states; other input states inapplicable, current constraint validation not implemented. |
| <a id="a093"></a>A093 | max; meter; progress | [attr-meter-max](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meter-max); [attr-progress-max](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-progress-max); [valid-floating-point-number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-floating-point-number) | [Meter and progress](#meter-and-progress): meter max default 1 raised to minimum; progress max default 1 when invalid/nonpositive; distinct author checks. |
| <a id="a094"></a>A094 | maxlength; input; textarea | [attr-input-maxlength](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-maxlength); [attr-textarea-maxlength](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-maxlength); [string-length](https://infra.spec.whatwg.org/#string-length); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | [Control lengths](#control-lengths): maxlength content bound and independent reflected signed-long fallback -1; actual input state/initial textarea text qualifiers. |
| <a id="a095"></a>A095 | media; link; meta; source; style | [attr-link-media](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-media); [attr-meta-media](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-media); [attr-source-media](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-source-media); [attr-style-media](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-style-media); [valid-media-query-list](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-media-query-list) | [Media queries](#media-queries): [Metadata](#metadata) supplies meta theme-color-only media placement; link/source/style surrounding HTML applicability and empty/whitespace handling remain separate. Grammar/environment are delegated, with no universal media-match API. |
| <a id="a096"></a>A096 | method; form | [attr-fs-method](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-method); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [attr-fs-method-get-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-method-get-keyword); [attr-fs-method-post-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-method-post-keyword); [attr-fs-method-dialog-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-method-dialog-keyword) | [Enumerations](#enumerations): method defaults Get; submitter override and host submission separate. |
| <a id="a097"></a>A097 | min; input | [attr-input-min](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-min) | [Input quantities](#input-quantities): Number min absent no bound, Range default 0, five temporal-state native conversion; no sanitization. |
| <a id="a098"></a>A098 | min; meter | [attr-meter-min](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meter-min); [valid-floating-point-number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-floating-point-number) | [Meter and progress](#meter-and-progress): meter min default 0, coupled constraints distinct from input. |
| <a id="a099"></a>A099 | minlength; input; textarea | [attr-input-minlength](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-minlength); [attr-textarea-minlength](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-minlength); [string-length](https://infra.spec.whatwg.org/#string-length); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | [Control lengths](#control-lengths): minlength missing/failure no content bound, reflected -1; min/max pair author relation independent of current dirty validation. |
| <a id="a100"></a>A100 | multiple; input; select | [attr-input-multiple](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-multiple); [attr-select-multiple](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-select-multiple); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Multiple actual input-state restriction/select presence; selection/upload execution separate. |
| <a id="a101"></a>A101 | muted; audio; video | [attr-media-muted](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-muted); [media-resource](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#media-resource); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Muted default content; current muted audio state/transition external. |
| <a id="a102"></a>A102 | name; button; fieldset; input; output; select; textarea; form-associated custom elements | [attr-fe-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fe-name); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [dom-form-elements](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-form-elements); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Labels and form ownership](#labels-and-form-ownership): name used by qualified collection predicates; entry-list construction/HTML named-property lookup is separate from immutable membership. Radio group missingness uses exact nonempty name plus actual ordinary tree/form owner, not name alone. |
| <a id="a103"></a>A103 | name; details | [attr-details-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-details-name); [the-details-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-details-element); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Details](#details): nonempty native name and exact same-ordinary-tree group, absent/empty singleton; private construction applies CloseGiven. |
| <a id="a104"></a>A104 | name; form | [attr-form-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-form-name); [dom-document-forms](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-document-forms); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Form names](#form-names): exact nonempty native/local report and captured `FormNameAuthor` distinguish actual Document-collection uniqueness, no collection and influencing Missing; local absence/empty stay independent, with no overall conformance certificate. |
| <a id="a105"></a>A105 | name; iframe; object | [attr-iframe-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-name); [attr-object-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-object-name); [content-navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable); [valid-navigable-target-name-or-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-navigable-target-name-or-keyword) | [Navigable target](#navigable-target) names identify browsing contexts; actual iframe/object navigable name registration/current state belongs host. |
| <a id="a106"></a>A106 | name; map | [attr-map-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-map-name); [image-map](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-map); [referenced](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#referenced); [attr-hyperlink-usemap](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-usemap); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Map names](#map-names): checked local emission/report and captured `MapNameAuthor` independently supply local syntax, own-id relation and same-tree uniqueness or Missing; no overall conformance or map-processing certificate. |
| <a id="a107"></a>A107 | name; meta | [attr-meta-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-name); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Metadata](#metadata): captured MetaDescriptionAuthor uses exact ASCII-insensitive description selection and ordinary-Document uniqueness; other metadata-name algorithms remain separate. |
| <a id="a108"></a>A108 | name; slot | [attr-slot-name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-slot-name); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | Slot name exact string, missing means empty slot name under shadow slotting source; named/manual slot assignment is DOM capture/slot owner, not inferred flattening. |
| <a id="a109"></a>A109 | nomodule; script | [attr-script-nomodule](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-nomodule); [module-script](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#module-script); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Nomodule with actual script kind; execution guard separate. |
| <a id="a110"></a>A110 | nonce; HTML elements | [attr-nonce](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-nonce); [refsCSP](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSP); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Nonce](#nonce): exact content and independent cryptographic slot; native attribute/removal/insertion/clone hooks, captured slot getter and qualified Document observations. CSP parsing/enforcement external. |
| <a id="a111"></a>A111 | novalidate; form | [attr-fs-novalidate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-novalidate); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Novalidate; submission skips are interpreted through actual submitter context, validation execution external. |
| <a id="a112"></a>A112 | open; details | [attr-details-open](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-details-open); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): DetailsOpen; private construction applies presence transitions and group exclusivity; live user toggle remains external. |
| <a id="a113"></a>A113 | open; dialog | [attr-dialog-open](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dialog-open); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): DialogOpen; connected modal state supplied independently for inertness, no show/close transition. |
| <a id="a114"></a>A114 | optimum; meter | [attr-meter-optimum](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meter-optimum); [valid-floating-point-number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-floating-point-number) | [Meter and progress](#meter-and-progress): optimum author bounds, effective default midpoint(min,max) then clamp. |
| <a id="a115"></a>A115 | pattern; input | [attr-input-pattern](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-pattern); [prod-Pattern](https://tc39.es/ecma262/#prod-Pattern) | [Pattern](#pattern): JS UnicodeSets grammar external; HTML compilation/anchoring/current mismatch contextual, no compiled provider currently advertised. |
| <a id="a116"></a>A116 | ping; a; area | [ping](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ping); [concept-url](https://url.spec.whatwg.org/#concept-url); [set-of-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#set-of-space-separated-tokens); [valid-non-empty-url](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-empty-url) | [URLs and references](#urls-and-references): ping whitespace URL list primitive available; no attribute-specific token/URL/host ping processing report. |
| <a id="a117"></a>A117 | placeholder; input; textarea | [attr-input-placeholder](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-placeholder); [attr-textarea-placeholder](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-placeholder); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Placeholder](#placeholder): input no-CR/LF author/native check and separate stripped content projection; textarea exact multiline text, complete input-state presence checks, actual hint presentation separate. |
| <a id="a118"></a>A118 | playsinline; video | [attr-video-playsinline](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-video-playsinline); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): PlaysInline; media presentation external. |
| <a id="a119"></a>A119 | popover; HTML elements | [attr-popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover); [attr-popover-auto](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover-auto); [attr-popover-manual](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover-manual); [attr-popover-hint](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover-hint) | [Enumerations](#enumerations): popover missing NoPopover, invalid Manual, empty Auto; target/toggle/open transitions separate. |
| <a id="a120"></a>A120 | popovertarget; button; input | [attr-popovertarget](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popovertarget); [concept-id](https://dom.spec.whatwg.org/#concept-id) | [Command target relations](#command-target-relations): generic pointer/ID association and guarded static popover target; author reference conformance and live transitions remain distinct. |
| <a id="a121"></a>A121 | popovertargetaction; button; input | [attr-popovertargetaction](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popovertargetaction); [attr-popovertargetaction-toggle](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popovertargetaction-toggle); [attr-popovertargetaction-show](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popovertargetaction-show); [attr-popovertargetaction-hide](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popovertargetaction-hide) | [Commands and popovers](#commands-and-popovers): Toggle/Show/Hide content, missing/invalid Toggle and independent input-state author presence; actual transition external. |
| <a id="a122"></a>A122 | poster; video | [attr-video-poster](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-video-poster); [valid-non-empty-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-empty-url-potentially-surrounded-by-spaces) | [URLs and references](#urls-and-references): poster author nonempty reference; video poster loading/display state external. |
| <a id="a123"></a>A123 | preload; audio; video | [attr-media-preload](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-preload); [media-resource](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#media-resource); [attr-media-preload-none](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-preload-none); [attr-media-preload-metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-preload-metadata); [attr-media-preload-auto](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-preload-auto) | [Enumerations](#enumerations): preload empty Auto, missing/invalid implementation-defined; no provider choice invented. |
| <a id="a124"></a>A124 | readonly; input; textarea | [attr-input-readonly](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-readonly); [attr-textarea-readonly](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-readonly); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Readonly input-state/textarea own content, validation candidacy and selected text-control mutability in [Control facts](#control-facts); editing/dirty transitions external. |
| <a id="a125"></a>A125 | readonly; form-associated custom elements | [attr-face-readonly](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-face-readonly); [dom-elementinternals-willvalidate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-elementinternals-willvalidate); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): custom readonly requires actual form-associated classification; ElementInternals implementation external. |
| <a id="a126"></a>A126 | referrerpolicy; a; area; iframe; img; link; script | [attr-hyperlink-referrerpolicy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-referrerpolicy); [attr-iframe-referrerpolicy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-referrerpolicy); [attr-img-referrerpolicy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-referrerpolicy); [attr-link-referrerpolicy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-referrerpolicy); [attr-script-referrerpolicy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-referrerpolicy); [referrer-policy](https://w3c.github.io/webappsec-referrer-policy/#referrer-policy); [concept-fetch](https://fetch.spec.whatwg.org/#concept-fetch) | [Enumerations](#enumerations): referrerpolicy own state and hyperlink href author context; request effective policy not inferred. |
| <a id="a127"></a>A127 | rel; a; area | [attr-hyperlink-rel](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-rel); [hyperlink](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hyperlink); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens) | [Link relationships](#link-relationships): role-qualified grammar, keyword/effect/author reports and captured extension/support/URL/bookmark facts; formed noopener evaluation preserves qualified inputs, activation remains separate. |
| <a id="a128"></a>A128 | rel; link | [attr-link-rel](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-rel); [hyperlink](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hyperlink); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens) | [Link relationships](#link-relationships): role-qualified keywords/effects, no-links/rel-itemprop author constraints, actual capability facts and native history; separate document feed/manifest/icon selection retains qualified external suitability and source ordering. |
| <a id="a129"></a>A129 | required; input; select; textarea | [attr-input-required](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-required); [attr-select-required](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-select-required); [attr-textarea-required](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-required); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Required actual state/content; SelectValueMissing, CheckableInputValueMissing, InputTextValueMissing and TextareaValueMissing in [Control facts](#control-facts) are source-static supplied-current-state observations. Other state predicates and broader validity execution remain separate. |
| <a id="a130"></a>A130 | reversed; ol | [attr-ol-reversed](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-reversed); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence) and [Lists](#lists): reversed start default counts participating list items; not a stored false from missing CSS facts. |
| <a id="a131"></a>A131 | rows; textarea | [attr-textarea-rows](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-rows); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | [Numeric content](#numeric-content): textarea rows positive author quantity; missing/failed/zero recovery2. |
| <a id="a132"></a>A132 | rowspan; td; th | [attr-tdth-rowspan](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-tdth-rowspan); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | [Numeric content](#numeric-content): rowspan 0..65534 author, default 1, clamp65534, zero means remaining group. [Tables](#tables) owns slot allocation. |
| <a id="a133"></a>A133 | sandbox; iframe | [attr-iframe-sandbox](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens); [ascii-case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive); [attr-iframe-sandbox-allow-downloads](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-downloads); [attr-iframe-sandbox-allow-forms](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-forms); [attr-iframe-sandbox-allow-modals](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-modals); [attr-iframe-sandbox-allow-orientation-lock](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-orientation-lock); [attr-iframe-sandbox-allow-pointer-lock](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-pointer-lock); [attr-iframe-sandbox-allow-popups](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-popups); [attr-iframe-sandbox-allow-popups-to-escape-sandbox](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-popups-to-escape-sandbox); [attr-iframe-sandbox-allow-presentation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-presentation); [attr-iframe-sandbox-allow-same-origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-same-origin); [attr-iframe-sandbox-allow-scripts](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-scripts); [attr-iframe-sandbox-allow-top-navigation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-top-navigation); [attr-iframe-sandbox-allow-top-navigation-by-user-activation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-top-navigation-by-user-activation); [attr-iframe-sandbox-allow-top-navigation-to-custom-protocols](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-top-navigation-to-custom-protocols) | [Sandbox](#sandbox): token vocabulary/diagnostics and flag-set complement; no browsing-context sandbox enforcement. |
| <a id="a134"></a>A134 | scope; th | [attr-th-scope](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-th-scope); [attr-th-scope-row](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-th-scope-row); [attr-th-scope-col](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-th-scope-col); [attr-th-scope-rowgroup](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-th-scope-rowgroup); [attr-th-scope-colgroup](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-th-scope-colgroup) | [Table header scope](#table-header-scope): four native keywords and local author/default report; missing/invalid Auto, group anchoring Unchecked; TableModel owns actual associations. |
| <a id="a135"></a>A135 | selected; option | [attr-option-selected](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-option-selected); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): Selected content default independent of CurrentSelectedness/OptionDirtiness. |
| <a id="a136"></a>A136 | shadowrootclonable; template | [attr-template-shadowrootclonable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-template-shadowrootclonable); [shadowroot-clonable](https://dom.spec.whatwg.org/#shadowroot-clonable); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): ShadowRootClonable. Actual template attachment is a construction/parser effect, not proven by presence. |
| <a id="a137"></a>A137 | shadowrootcustomelementregistry; template | [attr-template-shadowrootcustomelementregistry](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-template-shadowrootcustomelementregistry); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): ShadowRootCustomElementRegistry; registry behavior external and actual construction effects separately represented. |
| <a id="a138"></a>A138 | shadowrootdelegatesfocus; template | [attr-template-shadowrootdelegatesfocus](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-template-shadowrootdelegatesfocus); [shadowroot-delegates-focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): ShadowRootDelegatesFocus; focus dispatch external. |
| <a id="a139"></a>A139 | shadowrootmode; template | [attr-template-shadowrootmode](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-template-shadowrootmode) | [Enumerations](#enumerations): shadowrootmode none-state missing/invalid; actual shadow attachment/context permission independent. |
| <a id="a140"></a>A140 | shadowrootserializable; template | [attr-template-shadowrootserializable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-template-shadowrootserializable); [shadowroot-serializable](https://dom.spec.whatwg.org/#shadowroot-serializable); [boolean-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attribute) | [Boolean presence](#boolean-presence): ShadowRootSerializable; serialization is outside immutable construction read contract. |
| <a id="a141"></a>A141 | shadowrootslotassignment; template | [attr-template-shadowrootslotassignment](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-template-shadowrootslotassignment); [shadowroot-slot-assignment](https://dom.spec.whatwg.org/#shadowroot-slot-assignment) | [Enumerations](#enumerations): shadowrootslotassignment Named default; actual slot assignment external DOM owner. |
| <a id="a142"></a>A142 | shape; area | [attr-area-shape](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-area-shape); [image-map](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-map); [attr-area-shape-keyword-circle](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-area-shape-keyword-circle); [attr-area-shape-keyword-default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-area-shape-keyword-default); [attr-area-shape-keyword-poly](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-area-shape-keyword-poly); [attr-area-shape-keyword-rect](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-area-shape-keyword-rect) | [Area geometry](#area-geometry): shape default Rect, aliases recovered nonconforming, own grammar+coords coupled. |
| <a id="a143"></a>A143 | size; input; select | [attr-input-size](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-size); [attr-select-size](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-select-size); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | [Control lengths](#control-lengths) input size positive author/getter20 vs [Numeric content](#numeric-content) select size display fallback1/4; recovered select0 retained, reflected0 differs display. |
| <a id="a144"></a>A144 | sizes; link | [attr-link-sizes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-sizes); [attr-link-rel](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-rel); [rel-icon](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#rel-icon); [unordered-set-of-unique-space-separated-tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unordered-set-of-unique-space-separated-tokens); [ascii-case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) | [Icon sizes](#icon-sizes): native exact positive bitmap/scalable set, per-keyword processing and actual rel eligibility; linked-resource availability separate. |
| <a id="a145"></a>A145 | sizes; img; source | [attr-img-sizes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-sizes); [attr-source-sizes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-source-sizes); [valid-source-size-list](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-source-size-list) | [Responsive images](#responsive-images): img/source sizes-list+auto applicability/parser absent; CSS expressions/environment external, HTML list selection still needs owner. |
| <a id="a146"></a>A146 | slot; HTML elements | Discovery: [the-id-attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-id-attribute); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text); defining: [attr-slot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-slot); [DOM slottable names](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md#light-tree-slotables) | Slot attribute exact string, with absent/empty slottable name empty. HTML permits it on all HTML elements and delegates user-agent requirements for any namespace to DOM. Actual assigned slot comes from DOM-owned shadow/slot capture; HTML queries do not construct the flat tree. |
| <a id="a147"></a>A147 | span; col; colgroup | [attr-col-span](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-col-span); [attr-colgroup-span](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-colgroup-span); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | [Numeric content](#numeric-content): col/colgroup positive span clamp1000, default 1; TableModel respects explicit column semantics independent geometry. |
| <a id="a148"></a>A148 | spellcheck; HTML elements | [attr-spellcheck](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck); [attr-spellcheck-true](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-true); [attr-spellcheck-false](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-false) | [Enumerations](#enumerations) and [Captured global states](#captured-global-states): own Default vs actual user-agent default/inherited Spellcheck query. |
| <a id="a149"></a>A149 | src; audio; embed; iframe; img; input; script; source (in video or audio); track; video | [attr-media-src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-src); [attr-embed-src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-embed-src); [attr-iframe-src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-src); [attr-img-src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-src); [attr-input-src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-src); [attr-script-src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-src); [attr-source-src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-source-src); [the-video-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-video-element); [the-audio-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-audio-element); [attr-track-src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-src); [valid-non-empty-url-potentially-surrounded-by-spaces](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-empty-url-potentially-surrounded-by-spaces) | [URLs and references](#urls-and-references): media/embed/iframe/img/input/script/source/track src remains exact. [Input Image and embedded dimensions](#input-image-and-embedded-dimensions) supplies the Image-state required-presence and ASCII-edge-stripped nonempty report. URL validity and resource restrictions remain Unchecked; resource selection/loading is separate. |
| <a id="a150"></a>A150 | srcdoc; iframe | [attr-iframe-srcdoc](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-srcdoc); [the-iframe-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element); [an-iframe-srcdoc-document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#an-iframe-srcdoc-document) | Iframe srcdoc raw HTML text retained; the decoded-Unicode parser does not instantiate iframe documents. Child navigable construction/parsing/security require separate host/document owner, no reparse native authoring. |
| <a id="a151"></a>A151 | srclang; track | [attr-track-srclang](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-track-srclang) | BCP47 srclang string retained unchecked; track-kind placement/required-language constraints HTML-owned and not certified by generic language storage. |
| <a id="a152"></a>A152 | srcset; img; source | [attr-img-srcset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-srcset); [attr-source-srcset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-source-srcset); [image-candidate-string](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-candidate-string) | [Responsive images](#responsive-images): img/source candidate author+recovery, descriptor/source context evidence; URL validation/resource selection separate. |
| <a id="a153"></a>A153 | start; ol | [attr-ol-start](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-start); [concept-ol-start](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-ol-start); [valid-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-integer) | [Lists](#lists): ol_start exact signed author/prefix recovery; source list starting default depends on reversed/participation. |
| <a id="a154"></a>A154 | step; input | [attr-input-step](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-step); [valid-floating-point-number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-floating-point-number) | [Input quantities](#input-quantities): any exact ASCII-insensitive keyword or finite-positive recovered float; default step/scaling/base by state, min→value→source default. |
| <a id="a155"></a>A155 | style; HTML elements | [attr-style](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-style) | CSS declaration grammar/validation delegated, exact style text retained. Inline association owner is document/CSSOM (#840), computed style/layout is not HTML interpretation. |
| <a id="a156"></a>A156 | tabindex; HTML elements | [attr-tabindex](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-tabindex); [focusable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#focusable); [sequentially-focusable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sequentially-focusable); [sequential-focus-navigation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sequential-focus-navigation); [valid-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-integer) | [Numeric content](#numeric-content) content exact integer/null vs captured signed-i32/element-default TabIndex in [Tab index](#tab-index); no focus navigation. |
| <a id="a157"></a>A157 | target; a; area | [attr-hyperlink-target](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-target); [navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable); [hyperlink](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hyperlink); [navigate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate); [valid-navigable-target-name-or-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-navigable-target-name-or-keyword) | [Navigable target](#navigable-target): a/area target lexical restriction incl special keywords; actual selected navigable context external. |
| <a id="a158"></a>A158 | target; base | [attr-base-target](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-base-target); [navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable); [hyperlink](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hyperlink); [navigate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [valid-navigable-target-name-or-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-navigable-target-name-or-keyword) | [Navigable target](#navigable-target) and [Document base](#document-base): source target recovery and actual default-target selection after explicit/own overrides, including empty; qualified form submission precedence. |
| <a id="a159"></a>A159 | target; form | [attr-fs-target](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fs-target); [navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [valid-navigable-target-name-or-keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-navigable-target-name-or-keyword) | [Navigable target](#navigable-target): form target exact/default name processing; host submission navigable creation external. |
| <a id="a160"></a>A160 | title; HTML elements | [attr-title](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-title); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Advisory title](#advisory-title): AdvisoryInformation selects own/ordinary-parent title, with exact newline normalization and qualified empty/missing outcomes. |
| <a id="a161"></a>A161 | title; abbr; dfn | [attr-abbr-title](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-abbr-title); [attr-dfn-title](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dfn-title); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | Exact abbr expansion/dfn title text remains raw. [Dfn defining-term selection](#defining-term) consumes specified own or qualifying child title including empty, without ancestor inheritance or normalization. Expansion/definition adequacy and contextual author conformance remain separate. |
| <a id="a162"></a>A162 | title; input | [attr-input-title](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-title); [attr-input-pattern](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-pattern); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | Input title describes pattern when provided; exact text retained, compiled pattern mismatch/feedback external; no inferred Valid from text. |
| <a id="a163"></a>A163 | title; link | [attr-link-title](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-title); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | [Advisory title](#advisory-title) records the selected link/style noninheritance boundary; title-of-link and stylesheet set names remain separate document/CSSOM operations. |
| <a id="a164"></a>A164 | title; link; style | [attr-link-title](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-title); [attr-style-title](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-style-title); [css-style-sheet-set-name](https://drafts.csswg.org/cssom/#css-style-sheet-set-name); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | Link/style CSS stylesheet set title association belongs document/CSSOM, exact text retained. Style empty title author restriction/context is separate from CSS text grammar. |
| <a id="a165"></a>A165 | translate; HTML elements | [attr-translate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-translate) | [Enumerations](#enumerations) and [Captured global states](#captured-global-states): own Inherit vs captured TranslationMode; translation execution external. |
| <a id="a166"></a>A166 | type; a; link | [attr-hyperlink-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-type); [attr-link-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-type); [valid-mime-type](https://mimesniff.spec.whatwg.org/#valid-mime-type) | MIME type advisory a/link string retained unchecked; HTML rel/processing context separate from MIME lexical conformance. |
| <a id="a167"></a>A167 | type; button | [attr-button-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-type); [attr-button-type-submit](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-type-submit); [attr-button-type-reset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-type-reset); [attr-button-type-button](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-type-button) | [Enumerations](#enumerations): button type missing/invalid Auto, actual submit-button classification context-dependent; execution separate. |
| <a id="a168"></a>A168 | type; embed; object; source | [attr-embed-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-embed-type); [attr-object-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-object-type); [attr-source-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-source-type); [valid-mime-type](https://mimesniff.spec.whatwg.org/#valid-mime-type) | MIME type embed/object/source string retained unchecked; supported resource representation needs actual resource capability, no fake enum or shared script-type dispatch. |
| <a id="a169"></a>A169 | type; input | [attr-input-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-type) | [Enumerations](#enumerations): input type missing/invalid Text; state-specific content/attribute/semantics still explicit per binding. |
| <a id="a170"></a>A170 | type; ol | [attr-ol-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-type); [attr-ol-type-keyword-decimal](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-type-keyword-decimal); [attr-ol-type-keyword-lower-alpha](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-type-keyword-lower-alpha); [attr-ol-type-keyword-upper-alpha](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-type-keyword-upper-alpha); [attr-ol-type-keyword-lower-roman](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-type-keyword-lower-roman); [attr-ol-type-keyword-upper-roman](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-type-keyword-upper-roman) | [Enumerations](#enumerations): ol type exact case-sensitive 1/a/A/i/I, missing/invalid Decimal; ordinal algorithm in Lists. |
| <a id="a171"></a>A171 | type; script | [attr-script-type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-type); [valid-mime-type](https://mimesniff.spec.whatwg.org/#valid-mime-type); [javascript-mime-type-essence-match](https://mimesniff.spec.whatwg.org/#javascript-mime-type-essence-match) | [Script type](#script-type): public canonical keywords, content meaning/advice and actual type/language classification; external MIME/body/context checks and execution remain separate. |
| <a id="a172"></a>A172 | usemap; img | [attr-hyperlink-usemap](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-usemap); [image-map](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-map); [valid-hash-name-reference](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-hash-name-reference) | [Hash and map reference](#hash-and-map-reference): own syntax vs same-tree first id/name match and separate existential name author witness; positive witness past deferred gaps retained. |
| <a id="a173"></a>A173 | value; button; option | [attr-button-value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-value); [attr-option-value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-option-value); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | OptionValue fallback text via [Options and selection](#options-and-selection); button exact value content retained, entry-list/submission contribution external execution. |
| <a id="a174"></a>A174 | value; data | [attr-data-value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-data-value); [attribute-text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attribute-text) | Data value exact machine-readable string retained; HTML defines no shared numeric/default conversion for all data values. |
| <a id="a175"></a>A175 | value; input | [attr-input-value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-value) | [Input quantities](#input-quantities) covers numeric/temporal content and existing common owners; the separate [Control facts](#control-facts) owner supplies supported parser current/default modes, strict temporal and Number sanitizers, empty File selection and policy-qualified Image construction. Range construction, [controlled Color conversion](#color-input) and live file/image resources remain separate from local attribute interpretation. |
| <a id="a176"></a>A176 | value; li | [attr-li-value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-li-value); [ordinal-value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ordinal-value); [valid-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-integer) | [Lists](#lists): li_value exact signed syntax/prefix recovery with no invented value default; actual ordinal inherits list owner/running state. |
| <a id="a177"></a>A177 | value; meter; progress | [attr-meter-value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meter-value); [attr-progress-value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-progress-value); [valid-floating-point-number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-floating-point-number) | [Meter and progress](#meter-and-progress): separate required meter value/constrained progress value author states; progress absence Indeterminate vs invalid present determinate0. |
| <a id="a178"></a>A178 | width; canvas; embed; iframe; img; input; object; source (in picture); video | [attr-canvas-width](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-canvas-width); [attr-dim-width](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dim-width); [the-picture-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-picture-element); [valid-non-negative-integer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-non-negative-integer) | Canvas width content defaults 300 via [Numeric content](#numeric-content). [Input Image and embedded dimensions](#input-image-and-embedded-dimensions) supplies other roles' local grammar/recovery and source-parent/input-state qualification. Canvas fallback is not reused. The [qualified natural-dimension query](#input-image-and-embedded-dimensions) supplies independent omission and exact paired-author checks; [captured image dimensions](#captured-image-dimensions) interpret img/input getter results over completed facts. Other reflection roles remain separate. |
| <a id="a179"></a>A179 | wrap; textarea | [attr-textarea-wrap](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-wrap); [form-submission-2](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-submission-2); [attr-textarea-wrap-soft](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-wrap-soft); [attr-textarea-wrap-hard](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-wrap-hard) | [Textarea wrap](#textarea-wrap): soft/hard keyword/default and hard-with-cols context; text wrapping/entry-list linebreak mutation separate. |
| <a id="a180"></a>A180 | writingsuggestions; HTML elements | [attr-writingsuggestions](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions); [attr-writingsuggestions-true](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions-true); [attr-writingsuggestions-false](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions-false) | [Enumerations](#enumerations) and [Captured global states](#captured-global-states): own Default/True vs computed ancestor/control-specific writing-suggestions state. |

## Common microsyntax identities

This section covers the 28 child headings under §2.3, including the Numbers and
Dates grouping headings. It excludes the parent §2.3 itself. Every actual value
producer retains [raw and construction](#raw-and-construction) admission and
[common grammar](#common-grammar) author/recovery distinction; context defaults
are selected only by the defining attribute/element consumer.

| Identity | Source heading | Native/processing association and limits |
| --- | --- | --- |
| <a id="m-common-parser-idioms"></a>M:common-parser-idioms | [2.3.1 Common parser idioms](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#common-parser-idioms) | Private owned scanners in numeric/temporal/tokens reuse source pointer/ASCII operations; no standalone semantic value/default. Evidence: interpretation and temporal_recovery public tests. |
| <a id="m-boolean-attributes"></a>M:boolean-attributes | [2.3.2 Boolean attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#boolean-attributes) | [Boolean presence](#boolean-presence): native omission/empty syntax and independent presence/spelling; no inherited state from common grammar alone. |
| <a id="m-keywords-and-enumerated-attributes"></a>M:keywords-and-enumerated-attributes | [2.3.3 Keywords and enumerated attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#keywords-and-enumerated-attributes) | [Enumerations](#enumerations): per-attribute tables determine own missing/invalid/empty states; there is no universal enum default. |
| <a id="m-numbers"></a>M:numbers | [2.3.4 Numbers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#numbers) | Grouping identity for numeric child clauses, not a standalone parser. Actual native arbitrary integer, finite float and dimension domains follow their specific subheadings. |
| <a id="m-signed-integers"></a>M:signed-integers | [2.3.4.1 Signed integers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#signed-integers) | HtmlInteger; HtmlValueInterpreter::integer whole author syntax versus signed prefix recovery. Leading signs/zeros and arbitrary magnitude retained; element-specific defaults separate. |
| <a id="m-non-negative-integers"></a>M:non-negative-integers | [2.3.4.2 Non-negative integers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#non-negative-integers) | HtmlNonNegativeInteger; nonnegative_integer whole author syntax versus prefix recovery. Native conversion rejects negative, content processing can recover -0; positive constraints separate. |
| <a id="m-floating-point-numbers"></a>M:floating-point-numbers | [2.3.4.3 Floating-point numbers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#floating-point-numbers) | HtmlFloatingPointNumber/floating_point_number: strict whole grammar vs recovery, finite binary64 native domain and grammar-valid Unrepresentable distinct. No shared default or universal mathematical-quantity equality. |
| <a id="m-percentages-and-dimensions"></a>M:percentages-and-dimensions | [2.3.4.4 Percentages and lengths](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#percentages-and-dimensions) | HtmlDimension/dimension strict author grammar and source percentage/length recovery; neither CSS dimensions nor every element sizing rule. |
| <a id="m-nonzero-percentages-and-lengths"></a>M:nonzero-percentages-and-lengths | [2.3.4.5 Nonzero percentages and lengths](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nonzero-percentages-and-lengths) | nonzero_dimension rejects source zero where selected; general HtmlDimension is not a nonzero proof, and unrelated attributes do not inherit that constraint. |
| <a id="m-lists-of-floating-point-numbers"></a>M:lists-of-floating-point-numbers | [2.3.4.6 Lists of floating-point numbers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#lists-of-floating-point-numbers) | HtmlFloatingPointList/floating_point_list strict author list plus source permissive delimiter recovery. Area author exact mathematical keys/processing geometry are a distinct consumer. |
| <a id="m-lists-of-dimensions"></a>M:lists-of-dimensions | [2.3.4.7 Lists of dimensions](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#lists-of-dimensions) | HtmlDimensionList/dimension_list strict list and legacy processing; no default layout sizing engine. |
| <a id="m-dates-and-times"></a>M:dates-and-times | [2.3.5 Dates and times](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dates-and-times) | Grouping identity for concrete date/time domains; native Gregorian checks and exact precision belong temporal owner, not external timezone guessing. |
| <a id="m-months"></a>M:months | [2.3.5.1 Months](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#months) | HtmlMonth/month author grammar and recovery; positive arbitrary year preserved. Numeric input coordinate conversion belongs separate consumer. |
| <a id="m-dates"></a>M:dates | [2.3.5.2 Dates](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dates) | HtmlDate/date Gregorian calendar validity and recovery; no current host date/timezone. |
| <a id="m-yearless-dates"></a>M:yearless-dates | [2.3.5.3 Yearless dates](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#yearless-dates) | HtmlYearlessDate/yearless_date validates month/day without inventing a year. |
| <a id="m-times"></a>M:times | [2.3.5.4 Times](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#times) | HtmlTime/time author value, HtmlParsedTime recovery retains additional source precision; conversion to native precision is fallible, no timezone. |
| <a id="m-local-dates-and-times"></a>M:local-dates-and-times | [2.3.5.5 Local dates and times](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#local-dates-and-times) | HtmlLocalDateTime/local_date_time and normalized_local_date_time; local value has no implied timezone, normalized form is distinct. |
| <a id="m-time-zones"></a>M:time-zones | [2.3.5.6 Time zones](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#time-zones) | HtmlTimeZoneOffset/time_zone_offset source offset domain; not a time zone database or host locale. |
| <a id="m-global-dates-and-times"></a>M:global-dates-and-times | [2.3.5.7 Global dates and times](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-dates-and-times) | HtmlGlobalDateTime/global_date_time, HtmlParsedGlobalDateTime exact recovery and UTC arithmetic; host zone resolution is not performed. |
| <a id="m-weeks"></a>M:weeks | [2.3.5.8 Weeks](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#weeks) | HtmlWeek/week ISO week validity and native coordinates; source Week default base is not Unix zero guessed for every temporal state. |
| <a id="m-durations"></a>M:durations | [2.3.5.9 Durations](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#durations) | HtmlDuration/duration and exact parsed precision; no elapsed wall-clock promise or scheduling engine. |
| <a id="m-vaguer-moments-in-time"></a>M:vaguer-moments-in-time | [2.3.5.10 Vaguer moments in time](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#vaguer-moments-in-time) | HtmlDateOrTime/date_or_time versus stricter HtmlDateWithOptionalTime/date_with_optional_time; modification datetime and time-element union remain distinct consumers. |
| <a id="m-colours"></a>M:colours | [2.3.6 Legacy colors](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#colours) | HtmlLegacyColor/legacy_color source recovery, not CSS color grammar or input Color modern color-space rules. |
| <a id="m-space-separated-tokens"></a>M:space-separated-tokens | [2.3.7 Space-separated tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#space-separated-tokens) | HtmlSpaceToken/HtmlSpaceSeparatedTokens; ordered/unordered unique sets add explicit comparison/uniqueness. Clause-specific allowed tokens/nonempty/single-code-point restrictions are not inferred. |
| <a id="m-comma-separated-tokens"></a>M:comma-separated-tokens | [2.3.8 Comma-separated tokens](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#comma-separated-tokens) | HtmlCommaToken/HtmlCommaSeparatedTokens/comma_separated_tokens: empty token allowed, boundary ASCII trimming distinct from exact raw; accept grammar adds own constraints. |
| <a id="m-syntax-references"></a>M:syntax-references | [2.3.9 References](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#syntax-references) | [Hash and map reference](#hash-and-map-reference): common scanner native/report and actual same-tree UsemapReference; lexical form alone cannot prove existence. |
| <a id="m-mq"></a>M:mq | [2.3.10 Media queries](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#mq) | [Media queries](#media-queries): Media Queries delegated grammar/environment; HTML empty/ASCII-whitespace branch must remain in actual consumer. |
| <a id="m-unique-values"></a>M:unique-values | [2.3.11 Unique internal values](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unique-values) | Source identity concept, not an authored string grammar. Opaque source/capture/result identities serve their own Rust qualification contracts; they do not certify every HTML algorithm's internal-value creation. Uses belong the actual selected algorithm owner, with no script-exposed factory or fake parser. |

## Raw and construction

[Shared values](src/values.rs) provide `DocumentString` exact UTF-16 (including
NULL, isolated surrogates and whitespace), `Namespace`, `QualifiedName`,
`Attribute` and `OrderedAttributes`. [Vocabulary](src/vocabulary.rs) supplies
known expanded identities and original contextual rows, while unknown/custom
and foreign names remain constructible. `HtmlDocumentBuilder` and
`HtmlFragmentBuilder` [append exact element/text records](src/construction.rs)
and freeze immutable results with result-qualified IDs, ordered children and
separate template contents. [Values tests](tests/values.rs),
[construction tests](tests/construction.rs) and [vocabulary tests](tests/vocabulary.rs)
cover those contracts. Successful generic construction is not a lexical or
whole-element conformance certificate; no serialize/reparse path is used.

For every attribute association, raw missing means no supplied Attribute,
present-empty means an existing value with zero units, and present-invalid text
remains unchanged. A supplied wrong expanded name/element context can yield
Unapplied; it must not disappear. `HtmlValueInterpreter` admits every requested
raw value against its per-value UTF-16 cap before applicability/processing,
including all sides of compound reports. Standalone native constructors and
reports use ordinary allocation: that cap is not cancellation, aggregate work,
fallible-allocation, physical-memory or RSS control. Native types prove only
their documented domain. Parsed raw DOM values have undergone required HTML
normalization; original lexemes/quotes/entity spelling belong provenance.

## Common grammar

[Interpretation](src/interpretation.rs), [numeric](src/numeric.rs),
[temporal](src/temporal.rs), [token](src/tokens.rs) and [legacy color](src/color.rs)
own the shared grammars. Reports separate strict whole-value author syntax from
processing recovery and preserve exact raw state. Missing is not a syntax error;
recovery does not automatically certify authored text. Grammar-valid but
unrepresentable native values have `HtmlValueAuthorState::Unrepresentable`.
Author positivity/range conditions belong contextual consumers, not a global
reclassification of all representation failures as lexical errors.

[Interpretation](tests/interpretation.rs), [temporal](tests/temporal.rs),
[temporal recovery](tests/temporal_recovery.rs), [tokens](tests/tokens.rs) and
[colors](tests/color.rs) test those domains. Arbitrary native decimal integers
and exact temporal precision do not turn every HTML quantity into exact-real
arithmetic; processing floating-point contracts retain their binary64 boundary.
Legacy-color processing explicitly reports no defined author grammar.
Source grouping/identity headings in the M table are not independent parse values.

## Boolean presence

`HtmlBooleanAttribute::into_attribute` emits conforming empty syntax for true
and omits false. `HtmlValueInterpreter::boolean_attribute` observes presence
independently of author spelling: arbitrary present text still means true while
conforming spelling is empty or the exact attribute name ASCII-insensitively.
[The owner](src/attributes.rs) has role/state/context-specific methods for input,
script, form-associated custom elements, submit buttons, img and link. A
prohibited author context does not suppress source-defined content presence.
[Content](tests/attributes_boolean_content.rs),
[completion](tests/attributes_boolean_completion.rs),
[hyperlink/context](tests/attributes_hyperlink_context.rs) and
[captured context](tests/semantics_attribute_context.rs) are separate evidence.
Missing required context is not an invented false/invalid result. Flags such as
checked/selected/open/muted default content are not live current-state snapshots.

## Enumerations

`HtmlValueInterpreter` and checked native keyword enums in
[attributes.rs](src/attributes.rs) own exact applicable keyword tables.
[Attribute tests](tests/attributes.rs) retain wrong names, missing/empty/invalid
text, independent author conformance and intrinsic values. Unless specified
below, an empty string follows keyword/invalid processing rather than a newly
invented empty default. Canonical native emission is separate from exact raw.

| Rule/method | Missing | Invalid | Explicit empty behavior |
| --- | --- | --- | --- |
| dir | Undefined | Undefined | Undefined; bdo author restriction separate |
| contenteditable | Inherit | Inherit | True |
| draggable | Auto | Auto | Auto |
| hidden | NotHidden | Hidden | Hidden |
| popover | NoPopover | Manual | Auto |
| command | Unknown | Unknown | Unknown |
| popovertargetaction | Toggle | Toggle | Toggle |
| th scope | Auto | Auto | Auto |
| translate | Inherit | Inherit | Yes |
| spellcheck | Default | Default | True |
| autocapitalize | Default | Sentences | Invalid processing Sentences |
| autocorrect | On | On | On |
| writingsuggestions | Default | True | True |
| inputmode / enterkeyhint | No state | No state | No state |
| shadowrootmode | No state | No state | No state |
| shadowrootslotassignment | Named | Named | Named |
| closedby | Auto | Auto | Auto |
| form_autocomplete | On | On | On |
| decoding | Auto | Auto | Auto |
| loading | Eager | Eager | Eager |
| preload | Implementation-defined | Implementation-defined | Auto |
| form_method | Get | Get | Get |
| form_enctype | UrlEncoded | UrlEncoded | UrlEncoded |
| track_kind | Subtitles | Metadata | Metadata |
| input_type | Text | Text | Text |
| button_type | Auto | Auto | Auto |
| crossorigin | NoCors | Anonymous | Anonymous |
| fetchpriority | Auto | Auto | Auto |
| referrerpolicy | Empty | Empty | Empty keyword |
| ol_type | Decimal | Decimal | Decimal; keywords 1/a/A/i/I are case-sensitive |

Actual button Auto classification, script-kind placement, link rel membership,
href applicability, preload policy and effective request policy are additional
conditions. Reports supply no fetch/navigation/editing/render execution.
[Form settings](tests/semantics_form_settings.rs) and
[hyperlink unqualified](tests/attributes_hyperlink_unqualified.rs) exercise this
phase separation; source-required context methods must be selected explicitly.

## Numeric content

[Scalar](src/attributes/scalar.rs) binds shared integers to element rules:
canvas width/height default 300/150 after missing/failed nonnegative recovery,
including valid zero; textarea cols/rows default 20/2 after missing/failure/zero.
Cell colspan/col/colgroup span default 1 and clamp1000; cell rowspan defaults1,
clamps65534 and preserves zero. Whole author ranges are independent of recovered
prefixes/defaults. Select size requires positive author syntax, preserves
recovered0/-0, and only missing/failure consults actual multiple presence for
1/4 display size; its reflected default 0 is a different fact.
`tabindex` content reports arbitrary recovered integer or Null; the captured
signed-long getter is [Tab index](#tab-index), not this common content report.
[Scalar](tests/attributes_scalar.rs), [select size](tests/attributes_select_size.rs),
[cell spans](tests/attributes_cell_spans.rs),
[column spans](tests/attributes_column_spans.rs) verify exact defaults/caps.
Canvas fallback is not an implementation of all embedded element dimensions.

## Control lengths

[Control length owner](src/attributes/control_length.rs) exposes `input_size`,
`input_size_for_input` and `control_length_bounds`; native positive size/bound
constructors serialize canonical integer content. Input size's own positive
getter defaults 20, while content supplies no positive request for rejected/zero
recovery. Content min/max length absent/failure means NoBound; signed-long
reflected getters independently default −1 on missing/failure/out-of-range.
Actual input state, min/max author relation and supplied initial text (textarea
newline normalization distinct from input) are explicit. It does not derive
current dirty-value validation, edit truncation or DOM setters.
[Control lengths](tests/attributes_control_length.rs) and
[select size](tests/attributes_select_size.rs) preserve those different facts.

## Direction submission name

A043 uses [control_text.rs](src/attributes/control_text.rs) for the
[specified nonempty dirname constraint](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fe-dirname).
`HtmlDirectionSubmissionName` accepts every nonempty exact UTF-16 string,
including spaces, NULL and isolated surrogates, and emits the canonical
no-namespace attribute. Missing and present empty have separate author
observations and no fabricated default name.

`dirname` checks input/textarea local content; `dirname_for_input` additionally
checks the supplied actual `HtmlInputType`. The complete normative input matrix
allows Hidden, Text, Search, Telephone, URL, Email, Password and Submit.
Forbidden authored presence is distinct from local text validity. Present
input content without that state remains Unchecked; textarea needs no input
state, and the input-only companion is Unapplied there. Raw admission precedes
expanded-name and text checks. Ordinary allocation and the original per-value
UTF-16 cap do not establish aggregate work, physical-memory or cancellation
bounds. Captured Directionality exists; submitted direction-entry assembly
still requires the form-submission consumer.

## Placeholder

A117 uses the same owner for separate
[input](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-placeholder)
and [textarea](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-placeholder)
text rules. `HtmlInputPlaceholder` checks exact text without CR/LF and emits
the canonical attribute; textarea needs no redundant native wrapper.
`placeholder` retains missing/empty/raw/local-author phases. Input CR/LF is an
author error at the first UTF-16 unit offset, while its independent content
projection removes only those units. Textarea retains conforming multiline
text exactly; empty is conforming and absence invents no hint.

`placeholder_for_input` checks present authored eligibility independently for
Text, Search, Telephone, URL, Email, Password and Number. Content-only input
remains Unchecked. The original raw cap applies before qualification or
projection, with ordinary allocation and no aggregate resource guarantee.
[Fifteen public tests](tests/attributes_control_text.rs) cover both declarations,
all 22 input states, exact UTF-16/native emission, simultaneous local/context
violations, source rows, expanded names and admission boundaries. Actual
value/focus/autofocus-dependent hint presentation remains a separate consumer;
the local projection does not imply a shown hint or applied ignored attribute.

## Input quantities

[Input bounds](src/attributes/input_bounds.rs) exposes
`input_numeric_constraints` for Number/Range and Date/Month/Week/Time/LocalDateTime
with native [temporal coordinates](src/temporal.rs). Number absent bounds remain
None; Range uses0/100 fallback. Temporal quantities use their actual state units;
Time permits periodic reversed bounds, nonperiodic inverted relations remain
independent author observations. Step uses exact any or parsing to finite number
>0, state scaling/default, and successful min content→value content→source
default base (Week's source base differs zero). Empty Number/temporal value author
syntax is allowed; empty Range value is not. Reports retain syntax failures and
converted/recovered quantities; no clamp/sanitize/current validity/stepUp occurs.
[Number/Range](tests/attributes_input_bounds.rs),
[temporal constraints](tests/attributes_input_temporal_constraints.rs) and
[coordinates](tests/temporal_input_coordinates.rs) are direct behavior evidence.

## Meter and progress

[Meter author owner](src/attributes/meter.rs) and
[progress author owner](src/attributes/progress.rs) expose coupled raw/common/
author reports and checked native attribute bundles. Exact authored decimal
relations are distinguished from binary64 representation; meter missing required
value is not a fabricated zero-author success. Progress authored maximum must be
positive, value nonnegative and no greater than actual max/default 1. Meter adds
value/min/max/low/high/optimum and low≤high relations.
[Author meter](tests/attributes_meter.rs) and
[author progress](tests/attributes_progress.rs) test these constraints.

Captured `Meter`/`Progress` in [range.rs](src/semantics/range.rs) use processing
recovery: meter min0, max1 raised to min, value0 clamped, low=min, high=max then
low/high clamps, optimum midpoint then clamp. Progress missing value is
Indeterminate; present invalid/negative value yields determinate0; invalid or
nonpositive max uses1 and value is capped there. Position follows that state.
[Range tests](tests/semantics_ranges.rs) include coupled and finite edge cases.
Neither family supplies rendering or task lifecycle.

## Area geometry

`area_coordinates` and native checked area shape/point/rectangle/circle/polygon
values live in [area.rs](src/attributes/area.rs). Shape's missing/invalid default
is Rect; standard historical aliases recover but are nonconforming keywords.
Authored coords use exact mathematical integrality/order constraints rather than
binary64-rounded integer tests; native circle zero radius is valid authored
syntax. Independent processing descriptors use recovery binary64, required
coordinate counts, rectangle normalization, odd polygon tail and nonpositive
radius empty region. Default shape remains symbolic. All requested raw slots
are admitted first; [area tests](tests/attributes_area.rs) cover literal behavior.
These area-coordinate reports do not compute image-map association, image
dimensions or hit testing; the separate [captured image dimension query](#captured-image-dimensions)
interprets qualified img/input getter results.

## File accept

`input_accept` / `input_accept_for_input`, `HtmlAcceptSpecifier` and
`HtmlAcceptValue` in [accept.rs](src/attributes/accept.rs) bind comma fields,
HTML audio/*/video/*/image/*, filename-extension rules, MIME symbolic roles,
processing comma splitting with boundary ASCII-space handling and duplicate/context observations. Full MIME
conformance is explicitly separate. [Accept tests](tests/attributes_accept.rs)
include raw/author/recovery/context/default distinctions. It does not choose a
file, validate selected resources or implement a picker.

## Control autocomplete

[Autocomplete](src/attributes/autocomplete.rs) owns exact token order, recognized
field/mode/contact/section/webauthn grammar, actual input-state/anchor mantle
restrictions and field identity/scope/defaults. `ControlAutocomplete` in the
[captured owner](src/semantics/autocomplete.rs) joins actual form association to
own/missing field processing; unavailable owner is not an invented default.
[Attribute](tests/attributes_autocomplete.rs) and
[captured](tests/semantics_autocomplete.rs) tests cover qualification. Browser
stored data, credential retrieval and UI autofill are external transitions.

## Responsive images

[srcset.rs](src/attributes/srcset.rs) implements img/source/link candidate
strict author fields and HTML recovery parsing, width/density exact descriptors,
URL token/range retention, diagnostics and supplied context author observations.
[Srcset tests](tests/attributes_srcset.rs) cover those contracts without external
URL validity or resource acquisition. A complete sizes/imagesizes list parser,
auto-size applicability, source-size selection and actual image candidate
selection are not supplied by this report. CSS length/media expressions and
environment require a source-qualified CSS/host consumer; their surrounding HTML
list grammar/ordering remains HTML-owned work, not excused by CSS delegation.

## Sandbox

`iframe_sandbox` and native sandbox tokens/flags in
[sandbox.rs](src/attributes/sandbox.rs) apply ASCII token processing, independent
unknown/duplicate/redundancy author observations and source sandbox flag-set
complement. Missing/empty have different flag consequences;
[sandbox tests](tests/attributes_sandbox.rs) exercise them. Current inherited
navigable sandbox restrictions, policy and navigation enforcement require
actual document/host state, not inference from a lexical token bundle.

## Navigable target

[navigable_target.rs](src/attributes/navigable_target.rs) provides checked name
and name-or-four-keyword values and `navigable_target_attribute` roles. Name-only
iframe/object attributes differ from target name-or-keyword attributes; leading
underscore/empty and combined TAB/LF/CR plus less-than restrictions are explicit.
[Target tests](tests/attributes_navigable_target.rs) cover the lexical contract.
Actual hyperlink href/submitter eligibility/placement and effective target choice
are not certified by this report. Choosing/registering a navigable belongs the
consumer requiring actual document/navigation context.

## IDs

[id_attribute](src/attributes/id.rs) and native `HtmlSpaceToken` prove local
nonempty/no-ASCII-whitespace id author syntax, preserving exact units; missing,
empty, invalid and differently named slots remain distinct.
[ID tests](tests/attributes_id.rs) cover local behavior. Same-tree/document
uniqueness is a contextual author condition; collection lookups and form/label
algorithms use actual IDs but do not certify global uniqueness.

[Captured IdAuthor](src/semantics/id_author.rs) now independently retains that
local report and observes ordinary-tree DOM-ID uniqueness. The
[overview](README.md#captured-id-author) and
[literal tests](tests/semantics_id_author.rs) distinguish NoId, Unique, an actual
capture-qualified Duplicate witness and influencing Missing. Applicable present
local author context remains Unchecked; absent and supplied-empty values both
have no DOM ID without erasing their different local author states.

HTML subjects may be open/custom; foreign subjects are Unsupported and
non-elements Inapplicable. Competing nonempty lowercase no-namespace id values
on any element namespace count, even when HTML local syntax fails. Equality is
exact UTF-16, with no trim/fold/normalization. Only ordinary parent/child edges
participate, including detached/fragment trees; template/shadow/slot/host/
node-document associations do not substitute for that tree. Incomplete coverage
prevents Unique; a known equal-ID witness can establish Duplicate despite
nonterminal gaps. Whole consulted values are admitted before assembly/comparison;
terminal raw/work/cancellation/resource failure cannot be rescued by a witness.

The actual borrowed traversal, one retained nested data record and compiled
outer result capacities use the existing qualified semantic ledger, with
scratch cleanup and atomic batch failure. Input backing remains shared and
capture/standalone/cloning allocation stays ordinary. No live ID mutation/index,
DOM named properties, URL fragment/selector behavior, whole author conformance,
linear-work or complete process-memory guarantee follows. The closed Query/Fact
extensions are intentionally breaking; no compatibility adapter is provided.

## Hash and map reference

[Hash scanner](src/attributes/hash_name.rs) exposes exact first-hash/tail/local
author form with native `HtmlUsemapReference` over a checked map-name token.
[Usemap owner](src/semantics/image_map.rs) computes the first same-tree map with
matching id/name for processing independently from the existential author
name-only witness. Positive witnesses after deferred later gaps are retained;
unresolved influencing gaps remain Missing rather than false. No tree is cloned
per query. `MapAreas` separately returns immutable ordinary area-descendant
membership including nested maps/foreign wrappers; it does not select an image.
[Hash](tests/attributes_hash_name.rs), [usemap](tests/semantics_usemap_reference.rs)
and [areas](tests/semantics_map_areas.rs) cover these distinct results.

## Modification datetime

[modification.rs](src/attributes/modification.rs) binds ins/del datetime to strict
`HtmlDateWithOptionalTime` author syntax, independent `HtmlDateOrTime` common
recovery and associated date/global timestamp only. Time-only recovery has no
associated timestamp; missing/empty/invalid/unapplied remain distinct. Exact
offset/precision/huge year reuse the temporal owner.
[Modification tests](tests/attributes_modification.rs) cover the native overlay.
This does not turn a time-element value into a modification timestamp.

## Time element

[Temporal](src/temporal.rs) and [interpretation](src/interpretation.rs) supply
`HtmlTimeElementValue`, `HtmlMachineReadableTime` and `time_element_value` for
ordered Month/Date/Yearless/Time/Local/Offset/Global/Week/Year/Duration alternatives;
strict year author syntax differs positive-year processing. Actual captured
`TimeElement` in [time_element.rs](src/semantics/time_element.rs) selects explicit
datetime or source-qualified text/content conditions, preserving provenance.
[Standalone](tests/time_element_values.rs) and
[captured](tests/semantics_time_element.rs) verify it. No host-zone, inverse,
stepping or complete content-conformance checker is advertised.

## Textarea wrap

[textarea_wrap.rs](src/attributes/textarea_wrap.rs) observes Soft missing/invalid
and Hard keyword, with separate hard-requires-cols author context; actual cols
common report is reused. [Wrap tests](tests/attributes_textarea_wrap.rs) cover
raw/state/context. It does not run wrapping layout or mutate the submission
entry-list value/linebreaks.

## Captured global states

`HtmlCapture`, `HtmlSemanticContext`, checked six-domain `HtmlSemanticLimits` and
`HtmlSemanticSession` [own one immutable input and finite ledger](src/semantics.rs).
Outputs retain capture/configuration/host/source qualifications and subject IDs.
`Known`, `Missing`, `Unsupported` and `Inapplicable` are distinct; terminal
resource/cancellation errors yield no successful partial requested batch. Current
host/configuration facts cannot be proved by a DOM revision alone. Dependency
output is conservative recomputation, not fabricated precise dependencies.

[Inheritance](src/semantics/inheritance.rs),
[text assistance](src/semantics/text_assistance.rs) and explicit producer facts
supply EditingHost/Editable/IsContentEditable (delegated editable source pin),
TranslationMode, Spellcheck, ComputedHeadingLevel, capitalization/correction/
writing-suggestions states. Parent versus shadow-host traversal is selected by
its actual clause; missing relations/default policy remain typed. No edit,
translation, spellcheck engine or virtual keyboard is executed.
[Inheritance](tests/semantics_inheritance.rs),
[text assistance](tests/semantics_text_assistance.rs),
[semantics](tests/semantics.rs) and [capture](tests/capture.rs) cover qualification,
budgets, cancellation, source/default branches and unchanged raw storage.

## Language and direction

[language.rs](src/semantics/language.rs) and [directionality.rs](src/semantics.rs)
consume actual qualified lang/XML-lang and ordinary/host ancestry, unknown empty
versus unknown nonempty tag, document fallback, source auto scanning exclusions
and current control-value facts. BCP47 lexical validation is not inferred.
Directionality's Unicode/UAX qualification is separate from document revision.
[Language tests](tests/semantics_language.rs) and [semantics tests](tests/semantics.rs)
cover those native outputs, not CSS bidi layout or host language transitions.

## Inertness

[inert.rs](src/semantics/inert.rs) consumes actual qualified flat relation,
connectedness and topmost blocking modal facts. Own inert wins; only the source
blocking modal escapes inherited inertness. Known Excluded ancestry is different
from Missing and does not prove noninherited inertness. Document is inherently
connected. [Inert tests](tests/semantics_inert.rs) exercise source cases and
qualification. DOM owns flat relationships; host/runtime owns modal/top-layer
transitions. No live state or generic flattening is rebuilt here.

## Legend form owner

`LegendFormOwner` in [control.rs](src/semantics/control.rs) returns the actual
form getter of an immediate HTML fieldset parent, otherwise known null. An
established current pointer, including null, differs from unavailable association
or parent evidence; explicit reset interpretation reuses the existing form-owner
algorithm. This applies to every direct legend without requiring first-child or
ordered-child facts. Generic `FormOwner(legend)` stays inapplicable.
[The source clause](../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#dom-legend-form),
[canonical explanation](README.md#captured-legend-form-owner) and
[tests](tests/semantics_legend_form.rs) cover qualified results, consulted raw
admission, actual session resource bounds, cancellation and atomic batches.
No runtime association transition, caption selection or author content
conformance is certified.

## Control facts

Shared [input state/rules](src/input.rs) and
[textarea/select/option flags](src/control_values.rs) provide the HTML-owned
fixed intrinsic vocabulary consumed by the actual private parser. Input rules
include Checkbox/Radio DefaultOn content readback, Submit/Reset/Button Default
content readback with absent empty, checked/indeterminate transitions and the
source-ordered reset/type/clone requirements. Radio's five trigger
decisions are shared rules; the private parser owns admitted peer normalization
over actual name, ordinary tree and form-owner identity.
Email/URL use a borrowed, stepwise sanitizer; Multiple Email retains its actual
produced value count independently of the joined current string. These rules
do not establish address/URL validity or live setter execution.
Date/Month/Week/Time/LocalDateTime use the shared strict grammar: invalid current
values become empty, valid Date/Month/Week/Time spellings survive, and LocalDateTime
uses `T` with the shortest valid time while native default content stays independent.
The controlled parser preserves actual source history and explicitly marks changed
separators as InputValueNormalization; [public report readers](README.md#decoded-unicode-parsing)
lend the resulting metadata and associations. Ordinary content/quantity recovery,
constraint validation, stepping, live setters and UI remain separate domains.
[The overview](README.md#shared-intrinsic-control-rules) explains trigger order,
bounded recognition/comparison and adapter responsibilities;
[input tests](tests/input_rules.rs), [Checkbox tests](tests/input_checkbox_rules.rs),
[Radio tests](tests/input_radio_rules.rs), [Email/URL tests](tests/input_email_url_rules.rs),
[temporal input tests](tests/input_temporal_rules.rs) and [family tests](tests/control_values.rs)
verify literal local decisions. Strings, selection, form/tree references,
resource admission and canonical DOM mutation remain with their actual adapters.
These pure rules do not replace the separately qualified captured observations
below or establish complete control execution/conformance.

[control.rs](src/semantics/control.rs) supplies Disabledness, current checkedness/
selectedness, dirty flags, indeterminateness and validation candidacy from actual
captured state/context. Own readonly/disabled content and actual dynamic state
are different. First legend/fieldset, optgroup/select, datalist and form-associated
custom classification are source predicates, not generic host booleans.
[Controls](tests/semantics_controls.rs) and
[candidacy](tests/semantics_validation_candidate.rs) verify them. Static candidacy
is not current validity, reporting UI, ElementInternals or constraint-validation
execution. `OutputValue`/`OutputDefaultValue` and `TextareaDefaultValue` use
[control text](src/semantics/control_text.rs) exact current/default/text facts,
with [tests](tests/semantics_control_text.rs); mutations/resets are not performed.


`CheckableInputValueMissing` in [checkable.rs](src/semantics/checkable.rs) is a
source-static Checkbox/Radio observation using required presence and actual
checkedness. Radio self belongs even with absent/empty name; other members need
exact nonempty name, same ordinary tree and actual form owner. A disabled required
member still contributes. A checked witness proves false despite unrelated gaps;
otherwise genuinely influencing possible membership/checkedness remains Missing.
[Checkable tests](tests/semantics_checkable_input_missing.rs) cover these conditions,
scratch release, admission, budgets, cancellation and atomicity. This performs
neither validity execution nor radio-state normalization/transition.

`InputTextValueMissing` and `TextareaValueMissing` in
[value_missing.rs](src/semantics/value_missing.rs) observe required presence,
mutability and empty supplied current value for Text/Search/Telephone/URL/Email/
Password inputs and textarea. They reuse the checked current-string supplement
and ordinary disabledness/fieldset/first-legend owner; authored input value and
textarea descendant default text do not substitute for current state. Required
and readonly use exact attribute presence. Known nonempty current value or known
immutability proves false without unrelated unavailable facts; otherwise the
first influencing missing fact remains explicit. [Twenty-one public tests](tests/semantics_text_value_missing.rs)
cover roles, literals, disabled branches, unknowns, qualification, exact work/
output limits, cancellation and atomic batches. Current strings retain their
supplement aggregate UTF-16 limit separately from selected attribute admission;
these queries add no traversal scratch or value copying. They do not perform
validation, sanitization or a live control transition. Source:
[input required](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-required) and
[textarea required](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-required).

## Form names

A104's [local owner](src/attributes/form_name.rs) supplies exact nonempty
`HtmlFormName`, canonical no-namespace name emission and optional `form_name`
reports. Absence is Missing observation/recognition but locally Conforming;
specified empty is a local violation. Whitespace, case, NULL and isolated UTF-16
surrogates remain exact. Applicable present context remains Unchecked.

[FormNameAuthor](src/semantics/form_name.rs) independently retains that local
report and actual Document forms-collection uniqueness: NoSpecifiedName,
NoFormsCollection, Unique, an actual qualified Duplicate witness, or influencing
Missing. Absence skips topology/other values. An established ordinary Element/
Fragment root supplies no forms collection; node-document and form-owner facts
are not membership. Unknown ancestry cannot be resolved by an equal form in the
known component. After Document membership is established, a known duplicate
can resolve unrelated child gaps; Unique requires complete influencing coverage.
Only exact HTML forms with specified lowercase no-namespace name compete.
Specified-empty comparisons preserve local failure; missing name reflection is
not an authored competitor. Ordinary foreign wrappers permit HTML descendants,
while template/shadow/host/slot relations do not join collections.

The [overview](README.md#captured-form-name-author),
[local tests](tests/attributes_form_name.rs) and
[captured tests](tests/semantics_form_name.rs) explain exact raw admission,
borrowed traversal and the existing qualified session's work/storage/cancellation
and atomic-batch contract. Known(report) is not overall conformance; a terminal
failure cannot be rescued by a witness. Capture/standalone/cloning allocation
stays ordinary, with no live-collection/DOM named-property/reflection/submission,
linear-work, complete allocator/RSS or public parse/report guarantee. Closed
Query/Fact/local-error additions are intentionally breaking.

## Labels and form ownership

[control.rs](src/semantics/control.rs), [labels](src/semantics/control.rs),
[collections](src/semantics/control.rs) and
[form defaults](src/semantics/form_default.rs) implement actual form owner versus
reset interpretation, labeled control, label-associated form, control label
membership and listed form/fieldset membership/length. Source uses ordinary tree,
not flattened labels/form ancestry. Custom classification and actual current
association remain qualified facts; unavailable history is not silently reset.
[Controls](tests/semantics_controls.rs), [labels](tests/semantics_labels.rs),
[label form](tests/semantics_label_form.rs),
[collections](tests/semantics_collections.rs) and
[form defaults](tests/semantics_form_default.rs) are actual public evidence.
These immutable collections are not live HTMLCollections, named-property APIs
or entry-list/form submission execution.

## Form keywords and submission

`formmethod`/`formenctype` [scalar overlays](src/attributes/scalar.rs) intentionally
have no own missing override; invalid defaults Get/UrlEncoded differ from None.
Captured SubmitterFormMethod/SubmitterFormEnctype/SubmitterNoValidate in
[form_settings.rs](src/semantics/form_settings.rs) join supplied actual owner/
submitter facts, with [settings](tests/semantics_form_settings.rs) and
[no-validate](tests/semantics_no_validate.rs) tests. FormDefaultButton and
FormImplicitSubmissionCondition are static source conditions, not submission
execution. Action URL resolution, target selection, encoding/entry-list assembly,
events, network and navigable transitions require real consumer context; their
absence is not repaired by lexical Get/UrlEncoded values.

## Options and selection

[select.rs](src/semantics/select.rs) and [option_text.rs](src/semantics/option_text.rs)
provide source-pruned option lists, selected lists/index/value, option index/form
owner/text/label/value, source placeholder and SelectValueMissing. They use actual
supplied selectedness/value/relations, not content selected as current state.
Only influencing unknowns block; source-sufficient false witnesses remain sound.
Actual explicit size1 placeholder rule is edition-qualified even with multiple.
[Select](tests/semantics_select.rs), [placeholder](tests/semantics_select_placeholder.rs),
[value missing](tests/semantics_select_value_missing.rs),
[option text](tests/semantics_option_text.rs) and
[option form](tests/semantics_option_form.rs) cover those contracts. These captured
queries do not mutate selection. The private standalone parser's
[select construction owner](src/construction/parser_session/select.rs) performs
its own source-qualified creation/reset, option-pop, topology and selectedcontent
effects, with [managed cloning](src/construction/parser_session/clone.rs) and
[private tests](src/construction/parser_session/select_tests.rs). Its pending-work
freeze refusal and source-defined FIFO checkpoints precede the separate
[public completed-report boundary](README.md#decoded-unicode-parsing). Live DOM
selection mutations, UI and event dispatch remain separate.

## Datalist

[datalist.rs](src/semantics/datalist.rs) binds actual same-tree input list lookup,
immutable datalist option membership and suggestion filter using actual option
value/label/disabledness; unrelated option details are not mandatory to every
membership query. [Datalist tests](tests/semantics_datalist.rs) exercise those
source branches. Host presentation/autocomplete choices remain external.

## Description lists

[DescriptionListGroups](src/semantics/description_list.rs) computes dl's pinned
[dt/dd grouping](../../references/html-language-elements-text-links--877d6146487f--8833a85fd242.md#process-dt-or-dd):
ordered qualified identities, one-level direct HTML div processing and one
seenDd state across div boundaries. Leading value-only/trailing name-only groups
and Known empty remain distinct. Complete dl/direct-div ordered children
influence the result; their unavailability yields owner-qualified Missing rather
than partial membership. Ancestry and member/ignored/nested-wrapper contents do
not influence grouping, and no attribute value or text is flattened.

The [overview](README.md#captured-description-list-groups) and
[literal tests](tests/semantics_description_list.rs) distinguish borrowed
iteration with zero scratch from controlled nested output capacities and atomic
session failure. E024/E029/E030/E031 gain these associations, not whole dl/div/
dt/dd content conformance, human-name uniqueness, live DOM or parser behavior.

## Lists

[list.rs](src/semantics.rs) owns ListOwner/ListStartingValue/ListOrdinal
using exact signed start/value, reversed and explicit CSS display/box participation
revision-qualified supplement. Own li content value does not invent an ordinal
outside a list; missing participation is not false. Native/content integer
reports live in [scalar](src/attributes/scalar.rs);
[list integer tests](tests/attributes_list_integers.rs) and
[semantics tests](tests/semantics.rs) cover real source/default behavior. CSS
layout/marker rendering and update publication belong their consuming owners.

## Tables

The independent `TableCaption` query selects the first direct HTML caption
child or known null without forming TableModel or requiring row/header facts.
Influencing direct-child order remains Missing; selected-caption descendants
and later names after the first witness are uninspected. It shares the existing
charged borrowed child scan and qualified atomic session, without new scratch
or nested output. The [canonical getter explanation](README.md#captured-table-caption)
and [literal/resource tests](tests/semantics_table_caption.rs) retain its exact
limits. Caption setters/create/delete and table author conformance are separate.

`TableHead` and `TableFoot` additionally select the first direct HTML thead
or tfoot independently of row membership/model formation, including empty
sections and known null. Selection follows ordinary source order, not TableRows
phase order. Their [canonical explanation](README.md#captured-table-sections)
and [paired tests](tests/semantics_table_sections.rs) describe influencing Missing,
shared qualified admission and first-witness read limits. Setter/create/delete
and full table author conformance remain separate.

[table.rs](src/semantics/table.rs) owns slot/column/cell/header model and errors,
including malformed spans/overlap, separately from static
[collections](src/semantics/table/collections.rs): bodies direct tbody; rows
header-first then direct/body then footer; section direct tr; row direct td/th.
Immutable membership does not require irrelevant cell/spans/attributes or imply
live collections, mutation or geometric layout.

`TableRowIndex`, `TableSectionRowIndex` and `TableCellIndex` use the same
[collection owner](src/semantics/table/collections.rs) to return native zero-based
positions; None represents source −1, separately from influencing Missing.
The row's immediate table parent, or section parent plus table grandparent,
selects table.rows; sectionRowIndex needs only its immediate table/section
parent, and cellIndex needs an immediate tr parent. Direct table rows use
header-first/body/footer-last order. Spans and implied slots do not contribute
ordinals. No nearest ancestor or template/shadow host substitutes for these
ordinary relationships. Target projection retains no member vector or scratch;
actual traversal, scalar output, physical outer capacity and terminal atomic
failure use the existing qualified session.
[Canonical explanation](README.md#captured-table-indices) and
[tests](tests/semantics_table_indices.rs) preserve the source/null/Missing,
qualification and resource distinctions. This does not provide live collection
or mutation, a WebIDL long binding or full table author conformance.

The [headers author rule](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-tdth-headers) is an unordered set
of unique space-separated tokens, each identifying a th participating in the
same table. That author relation is distinct from retaining exact raw/token
encounter order and processing header association in TableModel; processing
order does not turn the author grammar into an ordered set.
[Table model](tests/semantics_tables.rs) and
[collections](tests/semantics_table_collections.rs) verify these separate contracts.

## Table header scope

[table_header_scope.rs](src/attributes/table_header_scope.rs) owns A134 local
scope interpretation on exact HTML th. Row/Column/RowGroup/ColumnGroup select
the four recognized keywords; missing, empty and invalid content select Auto
processing independently of raw observation and author conformance. Literal
`auto` is not a conforming author keyword. Native values emit only row, col,
rowgroup and colgroup.

RowGroup and ColumnGroup retain Unchecked author anchoring: actual table slots
and row/column groups must establish the separate source constraint. This
local result does not infer anchoring from a parent name or reconstruct a
table. Existing TableModel scoped header association is unchanged; a complete
anchoring author observation remains required. [Nine public tests](tests/attributes_table_header_scope.rs)
cover literal spelling/defaults, exact UTF-16/raw retention, native emission,
expanded-name roles, source metadata and original raw-cap boundaries.
Ordinary allocation and the per-value cap do not establish aggregate work,
cancellation or physical-memory guarantees.

## Details

[details.rs](src/semantics/details.rs) exposes first direct HTML summary optional
ID, SummaryForParentDetails predicate and same-ordinary-tree exact nonempty name
group including subject; absent/empty yields singleton without unneeded topology.
[details_name](src/attributes/details_name.rs) adds local nonempty native author
syntax. [Name tests](tests/attributes_details_name.rs) and
[captured tests](tests/semantics_details.rs) cover those observations.
[details_rules.rs](src/details_rules.rs) owns allocation-free attribute effects;
[rule tests](tests/details_rules.rs) distinguish name/open presence and namespaces.
Private construction applies exclusive-open changes at ordered creation, cloning
and insertion, with controlled native attribute removal and retained source
history. Its closed ordinary producer qualifies notification elimination;
unqualified fragment notification branches refuse without consuming input.
The private eligibility marker is not published as authored metadata. Live
toggle/tracker operations, group author-context validation, UI rendering fallback
and whole content conformance remain separate.

## Tab index

[tabindex.rs](src/semantics/tabindex.rs) uses signed integer recovery only within
i32; otherwise source exact element/namespace defaults (including obsolete HTML
frame, SVG/MathML a and first-summary predicate). Explicit recovered values bypass
unneeded topology. [TabIndex tests](tests/semantics_tabindex.rs) distinguish
raw syntax, recovery/null/content magnitude, defaults/Missing and qualification.
Focusability, navigation order and focus effects require actual host algorithms.

## Encoding labels

A003/A019 use the published [encoding_label.rs](src/attributes/encoding_label.rs)
owner: `HtmlUtf8EncodingLabel` accepts only exact ASCII-case-insensitive UTF-8,
preserves supplied casing and moves it into canonical accept-charset/charset
attributes. `form_accept_charset` and `meta_charset` admit the entire selected
raw UTF-16 value before expanded-name applicability. Missing, present-empty,
invalid and unapplied observations remain distinct; missing/invalid content
selects no checked label or default. [Nine public tests](tests/attributes_encoding_label.rs)
cover casing, rejected aliases/padding, isolated units, emission, provenance,
roles, exact caps and retained reports.

Applicable present meta charset keeps `context_author` independently `Unchecked`
even when local spelling fails. Whole declaration exclusivity/content, document
uniqueness, first 1024 serialized bytes and character-reference constraints need
separate facts; this local proof does not certify them. Encoding sniff/restart,
decoding and form submission encoding selection remain separate operations using
actual document/Encoding inputs. Ordinary shared allocation and interpreter
per-value UTF-16 admission apply, without aggregate work/cancellation/physical
memory guarantees. Source: [form accept-charset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-form-accept-charset) and [meta charset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-charset).

## Color input

A025 uses [input_colorspace.rs](src/attributes/input_colorspace.rs):
`HtmlInputColorSpace` emits canonical limited-srgb/display-p3 attributes.
`input_colorspace` keeps raw observation, keyword recognition and author spelling
separate from the Limited sRGB missing/invalid fallback. Present applicable
content leaves its Color-state author restriction independently Unchecked.
`input_colorspace_for_input` accepts the actual `HtmlInputType`, observing Color
presence as Conforming and other-state presence as Nonconforming without
changing lexical evidence or intrinsic fallback/state. Missing adds no forbidden
presence violation; wrong expanded identities stay Unapplied.

The whole selected raw UTF-16 value is admitted before applicability. Reports
and native attributes use ordinary allocation; reports retain the per-value
interpreter cap, without aggregate work, cancellation or physical-memory accounting. [Ten public
tests](tests/attributes_input_colorspace.rs) cover canonical emission, literal
keywords/defaults, all non-Color states, exact raw/provenance and exact/short caps.
[Colorspace definition](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-colorspace)
and the [Color state](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#color-state-(type=color))
are the defining rules. Shared legacy_color is unrelated.

[Controlled Color construction](README.md#controlled-color-input) consumes these
same colorspace decisions and actual alpha presence through the canonical
`HtmlInputType::Color` and shared input effects. It delegates CSS parsing,
conversion and serialization to `surgeist-css` under the original parse
controller, retaining raw defaults independently from generated current values.
Its admission, cancellation and generated-value provenance belong to that
controlled adapter, rather than this ordinary attribute interpreter.
[Public Color construction tests](tests/color_input_construction.rs) exercise
that integration separately. Mutable well updates and live reflection remain
external operations.

## Heading offset

A062's [headingoffset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-headingoffset)
uses [scalar.rs](src/attributes/scalar.rs). `heading_offset` separates whole
nonnegative author syntax constrained to 0..=8 from exact successful recovery,
including values above eight. Missing and failed recovery contribute zero with
distinct origins; lexical failures are not overwritten by range errors.
`HtmlNonNegativeInteger::into_heading_offset_attribute` emits canonical 0..=8
values and rejects larger exact magnitudes. [Nine public tests](tests/attributes_heading_offset.rs)
cover ranges, huge recovery, origins, applicability, provenance and raw caps.

Both operations use ordinary allocation. Reports admit the entire original
selected UTF-16 value before applicability through the per-value interpreter cap;
native emission has no captured budget. Neither operation claims aggregate work,
physical-memory or cancellation accounting, summed offset or live reflection.
[Computed heading offset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-an-element's-computed-heading-offset)
and [computed heading level](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-an-element's-computed-heading-level)
ancestor/reset/shadow-host traversal, including the separate final level limit
nine, remain with the captured global owner. A063's Boolean
[headingreset](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-headingreset)
stops traversal beyond that element. Defining clauses remain distinct from the
retained labeled printed-index discovery links.

## Advisory title

A160 uses [advisory_title.rs](src/semantics/advisory_title.rs) and the captured
`AdvisoryInformation` query. `HtmlAdvisoryInformation` retains normalized
UTF-16 and the actual selecting subject: an own title, including empty, wins;
otherwise ordinary parent-Element recursion selects the nearest HTML title.
CRLF and remaining CR become LF; other code units and original captured raw
remain exact. Selected empty has a source; known empty without an attribute
selection does not. Influencing unavailable ancestry remains Missing.

The [generic advisory-information algorithm](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#advisory-information)
and specific [link](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-title)/
[style](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-style-title)
clauses are reconciled by the [selected interpretation](https://github.com/bj-data-eng/surgeist/issues/1134#issuecomment-6116993954):
title-less exact HTML link/style stops with known empty; own title wins first,
independently of rel or connectedness. This is a selected reconciliation, not
an upstream clarification. Foreign ancestors bridge ordinary Element parents
without supplying HTML title/exception; Document, Fragment and ShadowRoot stop
recursion without host/slot substitution. Missing evidence beyond a proven
selection or boundary is irrelevant.

Whole selected raw admission precedes normalization. Reads, actual variable
capacity reservations and writes use the shared semantic ledger; admission is
cumulative, failures sticky and batches atomic. No traversal scratch is added.
The existing fixed Arc wrapper/provider limitation remains explicit.
[Twenty-two public tests](tests/semantics_advisory_title.rs) cover qualification,
selection, boundaries, exact UTF-16, work/storage/cancellation and retained reports.
A163 title-of-link and stylesheet-set names still require their distinct
document/CSSOM association. Abbr/dfn expansion and pattern descriptions retain
contextual author meaning; this query is no semantic-adequacy or exposure proof.

## Address descendant restriction

E003 supplies `AddressHasForbiddenDescendant` through
[address.rs](src/semantics/address.rs), using the existing qualified ordinary
tree walker. The pinned [address content restriction](../../references/html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-address-element)
prohibits heading and sectioning content, header, footer and address descendants.
Heading-free hgroup is allowed by this restriction; a heading-bearing hgroup
contains a prohibited h1–h6 descendant. Ordinary foreign wrappers are traversed;
only exact expanded HTML names supply witnesses. Associated template/shadow
trees and the subject itself are excluded.

Known true is an established witness; known false requires complete influencing
coverage. A later witness can resolve an earlier unavailable branch, but without
one the first unresolved dependency remains Missing. Terminal failures cannot
be bypassed. The [canonical explanation](README.md#captured-address-restriction)
and [tests](tests/semantics_address.rs) describe qualified scalar results,
borrowed-iterator scratch, cumulative capacity admission, cleanup and atomic
batches. Flow content, contact-information meaning and the nearest article/body
contact scope remain separate; no full content-conformance or live DOM claim
follows.

## Anchor text

E001 additionally supplies the captured `AnchorText` getter through
[anchor.rs](src/semantics/anchor.rs) and the shared ordinary descendant-text
owner. Exact UTF-16 Text data is concatenated in tree order without requiring
href, trimming, rendered-text rules or associated template/shadow traversal.
Known empty text differs from influencing unavailable ordered children.
The [canonical explanation](README.md#captured-anchor-text) and
[literal/resource tests](tests/semantics_anchor_text.rs) describe qualification,
shared admission and atomic batches. Setter mutation, navigation/activation
and remaining anchor author obligations are separate.

## Figure caption

E036 uses [figure.rs](src/semantics/figure.rs) and the captured
`FigureCaption` query. The
[defining clause](../../references/html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-figure-element)
selects the first direct HTML figcaption element child at any position.
First/last placement belongs to the separate author content model; neither a
middle nor repeated caption defeats source selection. Complete absence is
`Known(None)`, distinct from unavailable ordered children. Exact namespace/local
identity ignores prefix; nested and template/shadow-associated content is not
searched. Selected-child descendants and later branches are irrelevant.

The same [borrowed direct-child scan](src/semantics.rs) serves the existing
Details/summary algorithms without changing their resource sequence. Reads/name
recognition and actual outer result capacity use the existing qualified ledger;
there is no scratch, nested output, content-string materialization or raw-content
scan. First stops are sticky and requested batches atomic. Fixed immutable
handle/provider limits remain. [Literal tests](tests/semantics_figure_caption.rs)
cover membership, qualification and resource boundaries; see the
[capability explanation](README.md#captured-figure-caption). E035's figcaption
role/raw content remains retained without a new figcaption-subject query.
Caption adequacy, author content conformance, mutation, rendering and exposure
remain separate requirements.

## Footer scope

E037 uses [footer.rs](src/semantics/footer.rs) and the captured `FooterScope`
query. It selects the nearest ordinary HTML sectioning-content ancestor, or the
actual associated document's body, including established null. Influencing
unavailable parent/child order remains Missing; template/shadow hosts supply no
ordinary ancestry. See the [capability explanation](README.md#captured-footer-scope)
for exact document-element/body selection, resource limits and qualification.
E041 retains the header's introductory/navigation role without a HeaderScope
query. Author content conformance, accessibility/exposure and live consequences
remain separate.

## Defining term

E027 and A161 use [dfn.rs](src/semantics/dfn.rs) and the captured
`DfnDefiningTerm` query. The
[source algorithm](../../references/html-language-elements-text-links--877d6146487f--8833a85fd242.md#defining-term)
first selects a specified own no-namespace lowercase title, including empty.
Otherwise a complete direct-child sequence with exactly one element and no Text
can select an HTML abbr child's specified title. Comments and processing
instructions are irrelevant to this predicate; even an empty Text node defeats
it. Exact namespace/local-name identity ignores prefixes.

Otherwise the existing [ordinary descendant-text owner](src/semantics/control_text.rs)
concatenates Text in tree order, including ordinary script/img/foreign-element
descendants, without separators or template/shadow association traversal.
`HtmlDefiningTerm` retains exact UTF-16 and own-title, abbreviation-child or
descendant-text origin. Ancestor titles do not affect it. No branch normalizes
newlines or collapses whitespace; advisory-information normalization does not
apply.
Influencing incomplete child evidence is Missing; proved selected titles skip
irrelevant descendants and ancestry.

Whole selected raw title admission precedes retaining shared backing. Actual
fallback text and traversal capacities use the qualified semantic ledger,
with scratch released on every exit, cumulative allocation, sticky stops and
atomic batches. Fixed shared-wrapper limits remain explicit. The
[literal tests](tests/semantics_dfn.rs) exercise selection, exact units,
qualification and resource failure; the [capability explanation](README.md#captured-defining-term)
retains definition/expansion adequacy, content-model, hyperlink and exposure
limits. This getter does not certify those separate source requirements.

## Document title

E103 uses [document_title.rs](src/semantics/document_title.rs), the existing
child-text owner and the captured `TitleText`/`DocumentTitle` queries.
[Title text](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-title-text)
is exact direct-child Text data, excluding nested element text. The
[document getter](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document.title)
selects the first HTML title in ordinary document tree order and strips and
collapses only TAB/LF/FF/CR/SPACE. Other UTF-16 units remain exact.
`HtmlDocumentTitle` retains the actual selected subject, including when its
text is empty. Complete absence is known empty without a source; influencing
unavailable ordered children remain Missing. Shadow trees, template contents
and detached nodes are excluded. More than one direct document Element is
query-local Unsupported rather than a rejected capture.

An exact SVG `svg` document element selects only its first direct SVG `title`.
The [narrow source determination](../../references/SOURCE-EDITIONS.md#absent-svg-document-title)
fills HTML's omitted no-title case with known empty and no source when direct
children are complete. Deeper SVG or HTML fallback titles are not selected.
This is a selected correction, not an upstream erratum or adoption of WebKit's
text extraction, canonicalization or lifecycle machinery.

Both actual text buffers, ordinary traversal capacity and cumulative output
are admitted through the shared semantic ledger before writes. Failures are
sticky, batches atomic and traversal scratch released on every stop. The
existing fixed Arc wrapper allocation limitation remains explicit.
[Twenty-four public tests](tests/semantics_document_title.rs) cover exact
selection, missing/empty distinctions, namespaces, excluded trees, UTF-16,
literal work and capacity limits, cancellation and retained qualified reports.
Title setters, author placement/content conformance and browser title-update
notifications remain required distinct operations.

## Commands and popovers

[command.rs](src/attributes/command.rs) supplies six standard command keywords,
Custom with an exact retained UTF-16 payload, and missing/invalid Unknown.
Checked native standard and Custom values emit command attributes; Unknown is
a processing state without an author emitter. Native Custom requires two exact
hyphens and no trailing ASCII whitespace; processing still retains a
prefix-qualified value with that author violation. The
[selected boundary interpretation](https://github.com/bj-data-eng/surgeist/issues/1133#issuecomment-6117207035)
uses Infra TAB/LF/FF/CR/SPACE, preserving bare `--`, internal whitespace and
non-ASCII tails. It is a selected reading, not an upstream clarification.

Popover target action is a separate Toggle/Show/Hide domain with missing and
invalid Toggle defaults. Its input companion retains local content while
independently checking author presence for Submit/Image/Reset/Button; other
states report a context violation, unavailable context stays Unchecked and
absence has no presence violation. Exact expanded names and complete raw
admission precede these operations. [Twenty-two public tests](tests/attributes_command.rs)
cover literal keywords/defaults, native emission, Custom/raw retention,
all input states, qualification, cap boundaries and existing presence-based
Auto submitter behavior. Ordinary allocation and the per-original-value cap do
not establish work/cancellation or physical-memory bounds.

[Captured target operations](#command-target-relations) consume actual pointer
facts and ordinary-tree relationships independently of local classification.
Live dispatch, activation and dialog/popover transitions remain required
distinct consumers.

## Command target relations

[command_targets.rs](src/semantics/command_targets.rs) supplies checked,
capture/host-qualified explicit Element slots for CommandFor and PopoverTarget.
Missing or Unavailable slots stay missing; Null alone selects the content ID
branch. Actual pointers use the source shadow-including ancestor scope test,
with ordinary target ancestry. An out-of-scope pointer produces known null
without falling back to raw content. The ID branch selects the first exact
match in the source's ordinary tree, including a foreign Element; hosted
template/shadow content and flat-tree relationships are not substituted.

`CommandForElement` and `PopoverTargetAssociatedElement` retain raw
attributes, pointer state, selected target and origin. `CommandValidity`
combines the actual association with the existing command report: Unknown is
false, Custom supports any Element, popover commands support HTML elements,
and dialog commands require an HTML dialog. A known null association has no
validity value. `PopoverTargetElement` additionally observes actual input
type, disabledness, form owner and submitter role. A selected foreign target
returns known null before popover inspection, following the
[selected interface guard](../../references/SOURCE-EDITIONS.md#foreign-popover-target-interface-guard),
and does not search for a later matching ID.

[Thirty-nine public tests](tests/semantics_command_targets.rs) cover pointer
states/scope, first duplicate ID, detached ordinary roots, foreign and shadow
boundaries, missing influencing information, command support, form/control
guards, exact UTF-16, shared capacities/work, cancellation and sticky atomic
batch failure. The supplement bounds moved capacity and validation work,
without allocator/RSS or cancellation claims of its own. Queries consume the
existing session controller. Same-tree author-reference conformance, live
command dispatch, popover show/hide and dialog transitions remain required
distinct operations; captured results execute none of them.

## Access keys

[token_declarations.rs](src/attributes/token_declarations.rs) supplies
`HtmlAccessKeys` and the `accesskey` report for A004. Local author checking uses
an ordered case-sensitive unique set with exactly one code point per token.
UTF-16 pairs count once; isolated surrogates, NULL and non-ASCII whitespace retain
their exact local spelling. Processing filters only tokens with the wrong
cardinality, retaining eligible repetitions and order. These are candidate
tokens; keyboard correspondence, host fallback, shortcut assignment, focus and
activation remain required host operations.

## Blocking tokens

For A018, the same owner checks an exact unique authored `render` set and
separately supplies Empty/Render after ASCII-insensitive processing. An uppercase
or duplicated declaration can fail local author checking while retaining Render.
Content-only link/script reports leave context unchecked when present; style
needs no extra presence condition. `blocking_for_link` observes the complete
actual attribute set and stylesheet/expect rel membership. `blocking_for_script`
retains the actual supplied script kind, permitting authored presence only for
external classic/module scripts. No kind is inferred from type/src. These
results concern explicit tokens, not potential, implicit or current render
blocking; resource lifecycle remains a separate consumer.

## Icon sizes

For A144, `HtmlIconSizes` checks an ASCII-insensitive unique set of Scalable or
exact positive bitmap dimensions in raw/device pixels. Author spelling allows
`any` or one x/X between ASCII decimal dimensions without leading zeroes.
`icon_sizes` retains each keyword's result in source order, including repetitions
and invalid neighbors; valid dimensions remain exact beyond u128.
`icon_sizes_for_link` separately checks actual icon/apple-touch-icon rel
membership. Linked-resource size availability remains Unchecked for a specified
applicable declaration; these are neither responsive source-size/CSS lengths
nor an icon-selection operation.

All three reports reuse common token/string/integer owners and preserve exact
raw attributes, absence versus specified empty, duplicate evidence, applicability
and independent author/context results. Wrong contexts have no applied token
projection. The complete original raw value is admitted before checking;
link companions admit both actual declaration and rel slots independently.
[Twenty public tests](tests/attributes_token_declarations.rs) cover all these
boundaries, UTF-16, native emission, companion roles, huge integers and exact
capacity precedence. Ordinary allocation and the per-original UTF-16 cap do
not establish aggregate physical-memory, work/cancellation or RSS guarantees.

## Script type

A171: `HtmlScriptTypeKeyword` emits canonical module/importmap/speculationrules
attributes. `HtmlValueInterpreter::script_type` retains exact raw evidence,
role applicability and presence independently of current content meaning,
HTML-defined author state and omission advice. Missing/empty or an exact
ASCII-insensitive JavaScript essence selects nominal Classic; untrimmed keywords
select their kinds, and other values select DataBlock with
`ExternalMimeUnchecked`. `HtmlDefined` establishes only the local type-value
branch. Specified applicable context remains Unchecked.

`script_type_with_attributes` reads complete actual ordered attributes and
retains type/language slots or known absence. Empty type, or absent type with
empty/absent language, selects Classic; otherwise explicit type overrides
language, or the virtual `text/` language branch supplies the classification
input. Only explicit JavaScript essence matching adds ASCII boundary stripping.
Keywords and language remain untrimmed, with no MIME parameter parsing.
`NoScript` records the source classification step's return without a kind;
it is independent of DataBlock meaning and unchecked MIME author grammar.

Both selected original values are admitted before role/override processing.
Standalone reports use ordinary allocation and a per-value UTF-16 cap. The sole
[borrowed classifier](src/script_type.rs) also serves the private parser under its
actual controller; public `HtmlScriptKind` does not set saved preparation
metadata. Public Boolean script operations still require their explicit context.
External MIME, script/JSON body, wider placement/source conditions and language
author conformance remain unasserted; no execution or whole-element validity
follows. The [overview](README.md#script-type-values),
[public owner](src/attributes/script_type.rs),
[source-derived tests](tests/attributes_script_type.rs) and
[pinned MIME reference](../../references/mimesniff-javascript-essence--39aa53511b13.md)
record this boundary. Private checkpoint provenance remains in
[PARSER-ORIGINS](PARSER-ORIGINS.md).

<a id="html-local-operations-with-no-corresponding-complete-producer"></a>

## HTML intrinsic operations and external capability boundaries

The following entries describe implemented HTML operations and the inputs or
execution retained by other owners. They do not imply that every obligation
needs its own wrapper. Shared primitives can be consumed directly where their
domain suffices; context-dependent algorithms need actual qualified input rather
than guessed facts. Each entry distinguishes its producer from any unavailable
grammar, live state or external action. The individual source links in the E/A
tables govern each entry.

### Input Image and embedded dimensions

A009/A149 require source-state qualification before applying input Image rules.
For an input in the [Image Button state](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-button-state-(type=image)),
[alt](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-alt) must be present and nonempty, with a label
appropriate for an equivalent button without the image. [src](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-src)
must be present and contain a valid nonempty URL potentially surrounded by
spaces, referencing a non-interactive, optionally animated image that is neither
paged nor scripted.
The [local image values](src/attributes/image_values.rs) supply required-presence
and nonempty reports selected by the actual InputType classification, plus an
exact-string native bundle that emits `type=image`. Content-only input reports
retain Unchecked state requirements. Label adequacy, URL validity and actual
resource restrictions remain independently Unchecked and need their real
consumers; the local report does not validate or resolve URLs.

A064/A178's [dimension attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dimension-attributes) apply to
img/iframe/embed/object/video, source only with a picture parent, and input only
in Image Button state. Specified values must be valid nonnegative integers.
When both are specified, either width is within 0.5 of height times the target
ratio, height is within 0.5 of width divided by that ratio, or both are zero.
The target ratio is the resource's natural width divided by natural height.
Both attributes must be omitted if the resource lacks either natural dimension.
The [local dimension reports](src/attributes/image_values.rs) reuse the common
nonnegative-integer owner for author grammar and exact prefix recovery, with
actual input-state and source-parent companions. Content-only conditional roles
remain Unchecked; unavailable source-parent evidence does not establish
inapplicability. These reports do not apply the [canvas report's](src/attributes/scalar.rs)
300/150 defaults. The [captured natural-dimension query](src/semantics/natural_dimensions.rs)
uses the same applicability owner and strict whole-value author grammar.
`EmbeddedDimensionAuthorConformance` independently reports omission and the
inclusive half-unit pair alternatives, including the authored-both-zero case.
The resource consumer supplies intrinsic CSS-pixel axes through independently
expected selection bindings and coherent observations; capture, host, producer,
configuration, selection, resource, content and metadata identities are checked.
Known absence, unavailable evidence and known present but unrepresentable axes
remain distinct. Supported exact nonnegative rational axes use bounded decimal
cross products through the existing semantic ledger and cancellation boundary.
Fact construction separately bounds incoming record capacities, retained bytes
and validation work. Degenerate or unsupported arithmetic remains explicit.
No natural dimensions or ratio are inferred from raw width/height or canvas
defaults.

### Captured image dimensions

The captured `ImageDimensions` query in the
[getter owner](src/semantics/image_dimensions.rs) interprets HTML img/input
width/height getters over qualified completed producer observations. Its
[canonical explanation](README.md#captured-image-dimensions) distinguishes
non-Image input zero, actual rendered content-box axes and unrendered independent
attribute-prefix overrides. Uninspected attributes are not known absent.

Known image unavailability supplies a null common argument and zero only for
unoverridden axes. Missing availability, selection or completed results never
mint that zero. Img density-corrected/preferred/default-sized results and Input
default-sized results retain their distinct operation keys; missing/unsupported
axes remain independent. Intrinsic author conformance above remains separate.
Exact rational outputs are a supported domain, not a universal CSS representation.

Capture/host/resource/operation/rendering revision joins and incoming
capacity/byte/work admission precede context use. Queries retain the sole
semantic ledger and sticky atomic-batch failure; public fact/native allocation
remains ordinary. CSS sizing production, decoding/resource loading, DOM
unsigned-long conversion/setters and other elements' reflection are external.

### Link destinations

A010 `as` uses the six HTML preload destinations and nine modulepreload
destinations, with thirteen distinct states in their union. The
[local owner](src/attributes/link_destination.rs) consumes the shared enumeration
and token scanners. Native `HtmlLinkDestination` emits canonical spelling;
`link_as` preserves whole-value ASCII-insensitive recognition without trimming,
with no state for missing, empty or invalid values and no enum default.

`link_as_with_attributes` admits actual original `as` and `rel` slots before
context checks. Preload requires a present preload destination; modulepreload
permits absence but requires its own destination domain when present. If both
rel members occur, both author requirements remain independent. Presence with
neither rel member is forbidden. Content-only reports retain Unchecked context.

Symbolic processing reports keep preload no state distinct from modulepreload's
later script selection for no state, including invalid author input. Recognized
image remains a modulepreload nonmember, not a recovered script. The projection
does not assert the real algorithm reached this step: it does not inspect href,
translate Fetch destinations, create requests, load resources or execute modules.
It does not certify all rel grammar or full element conformance. The vocabulary
retains the [fixed Fetch definition](../../references/fetch-script-like-destinations--e9460d1b22ea.md);
xslt is excluded. Ordinary allocation and per-original-value UTF-16 admission
apply; no aggregate work, physical-memory or cancellation control is claimed.

### Metadata

[MetaAttributeAuthorStructure](src/semantics/meta_author.rs) supplies independent
exactly-one discriminator, content-presence, theme-color-only media-placement
and specified-charset ordinary-Document multiplicity observations. Specified
empty discriminators count; content is required for name/http-equiv/itemprop
and omitted otherwise, independently of exclusivity failure. Allowed media
placement uses whole ASCII-insensitive theme-color, not CSS validation.
Own six raw fields remain exact and admitted before processing/delivery.

Absent own charset is NotRequired without topology. Otherwise actual Document
membership precedes Unique/Duplicate/Missing; detached roots have
NoDocumentMetadataScope. Empty/invalid competitor charset counts by presence
without reading its value, and unrelated author failures do not remove it.
A qualified duplicate can resolve a nonterminal child gap after membership is
proved; unknown ancestry and terminal failure cannot be rescued. Separate
Documents and template/shadow/host/slot relations do not join populations.
The [overview](README.md#captured-meta-attribute-author-structure) and
[literal tests](tests/semantics_meta_author.rs) explain the existing shared
ledger, scratch cleanup, nested/outer capacities and atomic batch contract.
Known structural evidence is not whole meta or encoding conformance.

E060/A030/A107's captured
[MetaDescriptionAuthor](src/semantics/meta_description.rs) observes exact
original name/optional content, required-content presence and independent
ordinary-Document description uniqueness. Only exact HTML meta and an
ASCII-insensitive whole `description` name select it; no trim or Unicode fold is
performed. Missing content stays absent with MissingContent and effective
prebuilt empty; present empty satisfies presence. Whether the prose describes
the page appropriately remains explicitly Unchecked.

NoDocumentMetadataScope distinguishes established detached Element/Fragment
roots from a Document population. Actual ordinary Document membership precedes
Duplicate witnesses; unknown ancestry remains Missing. Every matching HTML meta
counts, including body descendants and missing-content competitors, without
consulting competitor content. Unique needs complete influencing coverage;
an actual qualified witness can resolve unrelated nonterminal child gaps.
Template/shadow/host/slot relations and node-document identity do not substitute
for ordinary membership. A terminal stop cannot be rescued by a witness.

The [overview](README.md#captured-meta-description-author) and
[literal tests](tests/semantics_meta_description.rs) explain whole influencing
raw admission, shared exact UTF-16 backing, the existing qualified ledger's
scratch/nested/outer capacities and atomic batch failure. Known(report) does not
certify overall conformance, descriptive appropriateness or live metadata use.
Closed Query/Fact additions are intentionally breaking. Ordinary capture and
fixed shared-wrapper allocation limitations remain; repeated scans may be
quadratic under finite work, with no linear or complete process-memory claim.

A071's [local pragma report](src/attributes/meta_pragma.rs) recognizes all seven
source states with exact ASCII-insensitive whole keywords and no invented
missing/empty/invalid defaults. Content-language and set-cookie remain recognized
but nonconforming; `HtmlMetaPragma` emits the five conforming native author
keywords. Exact raw admission precedes processing/applicability, and applicable
present context remains independently Unchecked.

[MetaPragmaContent](src/semantics/meta_pragma.rs) retains original classification,
optional content and independent author/content/population observations.
Missing content is not an empty default. Static content-language selects the
first ASCII-space token only when no comma occurs; default-style returns an
exact nonempty name candidate. EncodingDeclaration checks the source's exact
text/html/UTF-8 framing, not general MIME or decoding. X-UA-Compatible and
set-cookie are ignored independently of author observations. CSP supplies
actual direct HTML head relation and nonempty-content facts with delegated
grammar Unchecked; no policy is parsed or enforced.

Refresh keeps strict HTML author framing separate from source-ordered lexical
recovery. Its arbitrary exact nonnegative delay and shared pre-URL substring
preserve skipped digit/period runs, original prefix fallback and quote
truncation. DocumentUrl does not supply the existing address; UrlString is not
validated or resolved. URL grammar remains delegated, and lexical Parsed is not
a successful refresh or execution fact.

Actual ordinary Document membership precedes same-state uniqueness; missing or
invalid content does not remove competitors. Independent encoding charset
conflict counts exact specified charset by presence, including empty/invalid
values and the subject. Detached roots and influencing Missing stay distinct;
qualified positive witnesses resolve only their own nonterminal gaps. Other
Documents and template/shadow/host/slot relations do not join populations.
Terminal raw/work/resource/cancel stops cannot be rescued by later witnesses.

The [overview](README.md#captured-meta-pragma-content) and
[local](tests/attributes_meta_pragma.rs) and [captured tests](tests/semantics_meta_pragma.rs)
describe exact shared UTF-16, the existing numeric/session owners, actual
scratch/nested/outer capacity admission, cleanup and sticky atomic batches.
Closed Query/Fact additions are intentionally breaking. Ordinary capture and
fixed-wrapper limitations remain; no linear or complete process-memory bound is
claimed. Known(report) is not whole meta/pragma conformance.

Other metadata-name algorithms and actual pragma insertion/lifecycle effects
remain separate. Charset spelling/population and content-type conflict do not
supply HTML-vs-XML document classification, actual encoding/decoding, unescaped
serialization within the first 1024 bytes or broader encoding-declaration
conformance. CSS media/environment processing, viewport, processed language,
stylesheet mutation, refresh history/timers/navigation and CSP grammar/filtering/
enforcement still need their defining owners. No generic string parser or
whole-meta completion follows.

### Template marker reference

A054 for targets a same-tree named marker/start ProcessingInstruction.
`HtmlSemanticQuery::TemplateForReference` reports the exact raw attribute,
an independent existential name relation and the first marker/start selection.
An established later name witness can satisfy the relation while earlier
unavailable data keeps the first selection Missing. Absent for skips topology;
empty for retains the independent author requirement while preparation returns
EmptyName. Exact case-sensitive names and ordinary tree relations govern the
search; host/template/shadow associations supply no same-tree proof. A selected
start counts following sibling start/end PIs regardless of their names, and
complete exhaustion retains the start without an end.

The [captured explanation](README.md#captured-template-marker-selection),
[semantic owner](src/semantics/template.rs) and
[public tests](tests/semantics_template_for.rs) describe admission, missing
evidence, actual cancellation and atomic batches. Native and captured readers
share the [complete XML pseudo-attribute grammar](src/pi_pseudo_attributes.rs)
and [marker selection rules](src/template_markers.rs); duplicate names or any
malformed suffix invalidate the entire map. This is independent of label/output
for grammar. The [private fresh ordinary parser](README.md#private-ordinary-templates)
executes insertion/replacement effects over actual PI maps and template state.
The captured query reports immutable facts; imported patch facts, live DOM
mutation and shadow attachment retain their separate owners.

### Microdata

A080–A084: [local token observations](src/attributes/microdata.rs) preserve exact
backing, occurrence ranges, duplicate positions and separate processing/author
rules. Native `HtmlMicrodataTokenList` enforces local uniqueness and required
nonempty lists; URL/vocabulary context is independently qualified. Itemscope's
existing Boolean presence remains the canonical local owner.

The [captured queries](src/semantics/microdata.rs) expose item/property values,
itemid/itemscope/itemtype/itemref eligibility, same-tree first-ID lookup,
property collection/order, receiving-item type paths, vocabulary/receiver checks
and document cycles/orphans. Actual nested itemscope prunes descendant discovery;
foreign wrappers and first-ID matches retain their specified roles without
joining shadow/template/host relations. Processing keeps duplicate itemref
occurrences while deduplicating itemprop names. Missing influencing coverage
remains distinct from complete absence. Proved type seeds, contributions and
cycles survive unrelated incomplete coverage, while complete collections/type
paths and absence-based results still require coverage.

Finite `HtmlMicrodataFacts` separately qualify original raw subject/token,
capture/host, independently expected URL/vocabulary revisions and actual URL
operation. Property URLs retain attribute-set-time document/base/encoding/event
context; identifiers use their current-document operation. Supplied facts do not
implement URL parsing or establish vocabulary truth. Ordinary input construction
limits are separate from the existing session's work, scratch, nested output,
cancellation and atomic publication. The [overview](README.md#captured-microdata)
and [local](tests/attributes_microdata.rs) and
[captured tests](tests/semantics_microdata.rs) describe these boundaries.
Live DOM collections, mutation, physical RSS and universal work-fit/linearity
claims remain outside this immutable contribution.

### Map names

A106: [HtmlValueInterpreter::map_name](src/attributes/map_name.rs) reports required
presence and the sole `HtmlSpaceToken` nonempty/no-ASCII-whitespace grammar.
Original raw admission precedes expanded-name applicability. Missing, Empty,
invalid and Unapplied remain distinct without a default; present local reports
leave author context Unchecked. `HtmlSpaceToken::into_map_name_attribute` emits
canonical no-namespace `name` with exact UTF-16, without trimming or normalization.

The captured [MapNameAuthor](src/semantics/map_name.rs) report separately observes
local syntax, specified own-id equality and other-map specified-name uniqueness
in the same ordinary tree. Empty or invalid names still participate; own-id
equality does not certify global id syntax or uniqueness. Absent name skips
unrelated id/tree facts. A known duplicate witness can resolve earlier
nonterminal unknown coverage; Unique requires complete influencing traversal,
otherwise uniqueness retains inner Missing. Actual shadow/template host
associations do not replace ordinary tree edges. Known(report) is not overall
conformance, and terminal failures cannot be hidden by a later witness.

Native/standalone operations retain ordinary allocation and the standalone
per-raw-value cap. The captured query uses the sole session's qualified ledger,
borrowed precharged local assembly, whole consulted name/id admission, one
fallibly retained data record and actual scratch capacities. Shared input backing
and fixed wrappers are excluded from new backing-allocation charges. Current
scratch is released while cumulative usage remains charged; terminal failures
are sticky and atomically discard the requested batch. [The overview](README.md#captured-map-name-author),
[local tests](tests/attributes_map_name.rs) and
[captured tests](tests/semantics_map_name.rs) describe the boundary.
UsemapReference and MapAreas remain separate; no map processing, URL, geometry,
activation or live-collection guarantee is added.

### Nonce

A110: [shared decisions](src/nonce.rs) preserve exact UTF-16 and the distinct
content attribute and cryptographic slot for HTML/SVG/MathML elements. Native
eligible elements initialize an admitted registered empty slot. The actual
[construction hooks](src/construction/parser_session/nonce.rs) handle content
changes/removal, independent slot writes, clone completion and conditioned
insertion. Hiding uses current browsing-context connection and header-CSP of the
actual shadow-including root Document; it saves the current slot, empties the
content attribute and restores the saved value. Standalone construction retains
content because it has no browsing context. Influencing unavailable context
refuses; a known false conjunct discharges unrelated gaps.

The [capture owner](src/capture.rs) establishes actual slot and Document facts
with existing identity/retention admission. Unavailable differs from established
empty. `HtmlSemanticQuery::Nonce` observes the slot independently of content and
the content-attribute input cap; completed parse metadata lends its registered
raw backing/history through `cryptographic_nonce`. Existing control, effects,
provenance, cleanup and atomic publication remain authoritative. The
[overview](README.md#nonce-slot-and-content), [local/capture tests](tests/nonce.rs),
[semantic tests](tests/semantics_nonce.rs) and [public parse tests](tests/public_nonce.rs)
describe the boundaries. CSP parsing/enforcement and live DOM setters remain
with their consuming owners; content storage alone does not prove slot state.

### Pattern

A115 delegates JS UnicodeSets regexp grammar, while HTML defines anchoring and
missing/compile-error behavior for actual compiled patterns. No source-qualified
compiled provider/current pattern-mismatch operation is advertised. Raw strings
are retained; unavailable compilation is not Invalid or a match success.

### Link relationships

A127/A128: [local reports](src/rel.rs) preserve exact rel/rev fields and token
occurrences, distinguishing role applicability, relationship effects, no-links
cases and independent rel/itemprop author constraints. Captured
[relationship facts](src/semantics/link_relationships.rs) qualify extensions,
supported-token observations, URL grammar, bookmark/settings context and actual
native link history separately. Recognition and registration do not establish
execution support. Supplemental form interpretation follows its own defining
clause without changing the printed row identities.

[Document operations](src/semantics/link_relationships/selectors.rs) expose feed
discovery/default selection, first manifest and icon ordering/retry over ordinary
Document children. Feed types match exactly; discovery includes alternate
stylesheet declarations independently of their per-element meaning. Manifest
selection does not skip the first declaration for downstream readiness. Icon
selection consumes qualified suitability groups, breaks ties by last actual
tree order, and retains qualified actual attempted-use rejection history.
Unknown influencing evidence stays Missing; raw declarations and diagnostics
remain independent. [Independent cases](tests/semantics_rel_selectors.rs) cover
these operations. Network/navigation/CSS association, feed retrieval, manifest
installation and icon decoding/display remain with consuming execution owners.

### Document base

E009/A069/A158: [captured operations](src/semantics/base.rs) separately select
first href/target candidates in ordinary Document order. `DocumentBaseUrl` uses
actual fallback or the first candidate's historical frozen URL; unavailable first
facts remain Missing rather than selecting a later candidate. `BaseHref` is the
current getter, independent of frozen/CSP state. `ElementTarget` and
`FormSubmissionTarget` preserve overrides, own empty values and qualified form
precedence before the default target, using the shared TAB/LF/CR/less-than recovery.

[Qualified base facts](src/base.rs) distinguish unavailable from known-null
about-base and retain actual URL/encoding/parse/CSP observations and revisions.
The [native hook](src/construction/parser_session/base.rs) handles becoming first,
actual href changes, removal/reorder/adoption and clone/destination insertion in
the canonical effects owner. Non-href appends preserve an established response;
pending stopped responses cannot publish. Completed metadata retains actual
frozen state. Formation and semantic/native execution keep their respective
finite admission and original-stop/cleanup owners. Source labels are not URLs;
URL parsing, CSP enforcement, live browser response and navigation remain external.
The [overview](README.md#document-base-and-target), [public tests](tests/base_target.rs)
and [captured tests](tests/semantics_base.rs) qualify the current producer.

### Custom element names

E111/A078: [HtmlCustomElementName](src/custom_names.rs) owns the complete
selected valid-custom-element-name predicate and preserves exact UTF-16.
Moving conversions emit an unprefixed HTML local name or canonical no-namespace
`is` attribute; raw qualified storage remains open. The selected DOM ASCII-first
local-name branch permits less-than, equals, quotes, noncharacters and isolated
surrogates after the required lowercase ASCII first letter, subject to HTML's
other exclusions. This is local-name syntax proof, not serialized HTML syntax.

[HtmlValueInterpreter::is_attribute](src/attributes/custom_name.rs) admits the
original raw value before expanded-name applicability and uses the same checked
name owner. Missing, Empty, locally nonconforming and Unapplied states remain
distinct; no default name is manufactured. Every applicable present value leaves
context Unchecked independently of lexical success. The separate captured
[IsAttributeAuthorContext](src/semantics/custom_element_context.rs) observation
supplies local syntax, current matching-definition and autonomous-prohibition
results independently. Its Known result can retain a nonconforming or unavailable
obligation; it does not certify complete author conformance. Absent content skips
unrelated context facts, and invalid syntax does not suppress a known autonomous
prohibition. Unavailable registry, definition set, state or association is typed
evidence rather than a guessed mismatch.

The [capture producer](src/capture.rs) retains actual state and associated
definition, including independently known records in an incomplete registry.
Such records do not make current lookup complete. Current content lookup uses
the current registry and autonomous-first source order; saved runtime `is` and an
old association after adoption do not substitute for it. Autonomous prohibition
uses actual Custom state and association separately. Customized Custom joins
check a known saved name; autonomous Custom associations may retain a different
saved Value. Neither a hyphen nor a local report proves registration or
form-associated classification.

Native checking and raw interpretation use ordinary allocation, with a
per-original-value UTF-16 cap for raw interpretation. Capture construction has
ordinary allocation and logical retention limits. The captured semantic query
uses the existing qualified session ledger, shared borrowed local predicate,
charged definition cursor and actual result layout; retained capture backing is
shared after admission. Work, output and cancellation failures are sticky and
discard the whole batch. No process-RSS, universal polling-quantum or linear-work
guarantee is supplied. Registry execution, construction, upgrade and callbacks
remain with their execution owner. Content checking does not set saved runtime
`is` or change an element's behavior. [The crate overview](README.md#captured-custom-element-author-context),
[local tests](tests/custom_names.rs) and
[captured tests](tests/semantics_custom_element_context.rs) describe these
separate surfaces.

### Media queries

M:mq/A095: [media.rs](src/media.rs) owns the borrowed UTF-16 cursor and HTML's
specified-empty/ASCII-whitespace unconditional match. Specified `all` remains a
CSS observation; its boundary-trimmed spelling separately controls picture
source author requirements. [Local reports](src/attributes/media.rs) retain exact
fields, expanded-name applicability and independent theme-color placement.

Captured `MediaAttributeRules` exposes conditional link advice/resource gates,
style gates, source filtering/picture requirements and theme-color uniqueness
over the actual ordinary HTML Document. Missing media gives link/style DefaultAll
or source/theme NoFilter; unavailable influencing environment is Missing, never
a mismatch. [Qualified facts](src/semantics/media_facts.rs) bind the actual media
ordinal/capture/host and independent grammar/environment revisions, with bounded
incoming record capacity and validation work. HTML does not execute CSS matching,
resource application or theme selection. Link readers need actual relationship
classification from their consumer.

The [semantic owner](src/semantics/media.rs) uses the existing session's raw
admission, work, nested output, scratch cleanup and atomic first-stop behavior.
[Public literal tests](tests/media_rules.rs) cover exact values, qualification,
picture order, ordinary-tree uniqueness and resource/batch boundaries.

## Supplemental source domains

The [obsolete source](../../references/html-language-obsolete--877d6146487f--66870e9cdabe.md),
[syntax/foreign integration](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md),
[event/global/open families](../../references/html-language-foundations--877d6146487f--b55e73c31978.md)
and [custom elements/templates](../../references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md)
remain relevant beyond the modern indexes. Unknown attributes, data-* and ARIA
are exact raw storage; ARIA grammar/accessibility mapping is delegated, while
HTML surrounding role/context remains defined by its clause. Event handler text
is retained; JS grammar/event compilation/dispatch is not executed here.

Ordinary private foreign construction implements the HTML-defined MathML/SVG
name/attribute adjustment, integration, CDATA, recovery and qualified native
cloning through the existing controlled parser owner. Exact SVG script elements
also have [restricted private processing](README.md#private-svg-script): actual
processed state, parser-end wrapper, nonparser connection and qualified clone
bit preservation. Separate no-loader and Disabled capabilities suppress
acquisition and execution; this is not literal SVG Tiny Disabled behavior or
whole SVG/MathML language support. [Private contextual fragments](README.md#private-context-fragments)
retain qualified context/effect facts in
[bounded public reports](README.md#decoded-unicode-parsing); live DOM import remains
unfinished. [Parser origins](PARSER-ORIGINS.md) records the selected upstream
adaptation and HTML differences; [the crate overview](README.md) records actual
admission, cleanup and capability limits.

The fresh ordinary driver has [private EOF completion](README.md#private-ordinary-eof):
source-initial Complete readiness is distinct from parsing completion; Interactive
precedes canonical pops, two source checkpoints run, and the post-load milestone
precedes execution of the retained Complete-readiness continuation. The whole
completed owner
retains construction evidence. [Contextual fragment execution](README.md#private-context-fragments)
has a separately qualified internal EOF return. The public entry transfers
completed native storage and committed diagnostics/history into its source-owning
report; no browser events or queue are synthesized.

Immutable constructed template/shadow/PI content and explicit metadata/effects
are not a second live DOM. Internal production parser machinery supplies
[decoded-Unicode document/fragment requests](README.md#decoded-unicode-parsing)
with source-limited profiles, explicit unsupported branches and retained
provenance; this reference does not certify complete HTML parser conformance.
DOM owns canonical identity/mutation/ordinary-shadow-flat captures and
adapts them once for HTML queries. CSS/layout own grammar/computed presentation/
geometry; runtime/host and domain consumers own lifecycle/publication/execution.
Planned owners are identified as planned, not available provider APIs. No generic
provider protocol, per-query tree copy, compatibility shim or runtime metadata
surface is required to read this reference.

## Token sequences

[Tokens](src/tokens.rs) provide checked individual space/comma tokens, sequences,
and ordered/unordered unique sets with explicit comparison. Native valid tokens
retain exact unit values; normalized/split views never rewrite raw Attribute.
[Token tests](tests/tokens.rs) and [interpretation tests](tests/interpretation.rs)
cover boundaries, empty comma tokens, uniqueness and comparison. Class content
can use unordered space tokens; output for/headers/itemref/accesskey/rel/sandbox
add separate source conditions listed in their contextual rows. Shared splitting
does not automatically certify allowed keywords, IDs, nonempty-set or
single-code-point requirements. Live DOMTokenList, CSS selector matching and
application mutation are not supplied by this immutable primitive.

## URLs and references

Each URL-bearing row retains exact authored `DocumentString`/Attribute and its
HTML role/source. Relative URL text is representable without a base. This surface
does not expose a source-qualified URL parser/resolver or complete lexical URL
validator; raw storage is unchecked rather than Valid. Actual consumer context
must supply document/base URL and URL capability before source parsing/fallback
algorithms are applied. Empty/absent author requirements remain those of the
individual clauses; no global URL invalid/missing default is inferred. Common
hash-name syntax is independently [implemented](#hash-and-map-reference) and is
not URL resolution. MIME/language/ARIA/CSS role storage likewise does not certify
external conformance. External fetch/navigation/submission lifecycle does not
remove HTML presence/token/list/applicability requirements.
