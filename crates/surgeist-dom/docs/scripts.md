# HTML and SVG Script state

Native Script preparation and SVG processing use different source rules. Their retained document history also participates in [semantic capture](semantic-captures.md) and [parser-report installation](html-installation.md).

## HTML Script

Native HTML Script has a checked `DomNode::as_html_script()` view over its real
canonical flags, optional type, parser/preparation Document identities and shared
Uninitialized result/null-ready state. Fresh native creation has null documents,
force-async true and other flags false. Actual native node-document browsing
context proves scripting Disabled, independently of XML/HTML class. Preparation
still stores recognized type, already-started and actual preparation document
before that source return. No fetch, compilation, error result, ready callback
or JavaScript execution is manufactured.

Script source hooks run at each actual operation: async Added, connected nonnull
src writes (including redundant values), parent children-changed after insertion/
removal and CharacterData replacement, then source static descendant
post-connection order. Only immediate nonempty Text contributes source content;
whitespace and isolated UTF-16 units are preserved. Type/language interpretation
uses HTML's admitted borrowed cursor. Type edits alone do not prepare; equal data
edits or empty-fragment insertion can subsequently trigger a state-only change.
Template contents remain disconnected through their host. Shadow connectivity
does not replace actual node-document identity when a root differs from its host.

Clone/import-as-clone applies actual copied attributes, then copies only source
already-started; type/documents/result and other flags start fresh. Adoption,
detach and immutable snapshot copying preserve established preparation history.
Historic preparation documents stay in capture association closure, without
declaring their unrelated trees complete or adding a fake semantic Script fact.
Native APIs admit no arbitrary parser/processed-history installation; that cause
must preserve/remap real metadata separately. Resource-capable preparation
is not admitted by this native HTML profile.

## SVG Script

Native SVG Script has a separate checked `DomNode::as_svg_script()` view over
HTML's `SvgScriptFlags` and its actual `DomSvgScriptOrigin::{Native, Clone}`.
Fresh native creation is unprocessed. The selected Tiny four-step operation uses
DOM3071 shadow-including connectedness and the existing static post-connection
list: hosted roots precede a host's ordinary descendants, and inserted roots keep
source order. An eligible false bit becomes true even for empty content, unusual
type, absent/empty/data/same-document/external href or inert handler attributes.
No HTML empty/type/async/preparation guards or restart hooks are imported.
Ordinary Template contents and detached hosts remain disconnected; Slot assignment
alone does not connect them. Removal and later href/content edits do not restart.

Native creation qualifies disabled script acquisition and execution independently
of null browsing context or HTML Disabled. Before changing an eligible false bit,
processing also checks the actual staged Load/Error paths for executable
recipients. Runtime registrations retain DOM-issued affine capabilities across
all registries using the owner. Capture listeners count at ancestors; noncapture
listeners count at actual at-target invocations. Registration before creation,
later insertion or adoption cannot bypass this check. An applicable recipient
causes atomic `SvgScriptProcessingUnavailable`, preserving the unprocessed bit.
Removing the last applicable registration permits a later processing attempt.

Configure `DomEventRecipientLimits` explicitly to admit Load/Error registrations;
its default is zero. Its storage and work are separate from already-started
dispatch custody. Unused owners need no recipient allocation or route traversal.
Removal releases live registration capacity; reusable endpoint index backing
can remain charged until the DOM owner is dropped.
The actual node Document still qualifies processing even when it differs from
the shadow-connected host's Document. Authored handler strings remain inert data.
No URL, acquisition result, event, promise or task is fabricated. Retained-history
installation preserves its original qualified processing history; registration
after processing does not replay it.

Clone/import copies the actual true or false bit through the shared clone rule,
sets Clone cause after copied attributes and before parent insertion, and uses
the target native environment. Adoption preserves state/cause. An insert followed
by detach can therefore leave a required state-only ElementState change despite
unchanged final topology. Snapshots/captures retain their original state. Native
fragment installation uses fresh native cause and these same insertion hooks;
it does not certify parsed/resource history. Scalar state adds no retained string
cost. Processing reuses the finite canonical-node static traversal and fallible
buffers with per-node cancellation, plus finite recipient lookup when needed.
SVG use-instance trees and resource-capable processing remain separate domains.

[Back to surgeist-dom](../README.md).
