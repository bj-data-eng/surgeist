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

Native constructors establish a private immutable closed SVG processing witness:
all network/data/same-document script acquisition and inline/external execution
are disabled, and no admitted SVG load/error recipient, executable handler,
callback, retained task, event or resource output can observe omitted notification.
This follows the actual constructor/command/host/output boundary, independently
of null browsing context or HTML Disabled. Authored handler strings are inert
UTF-16 data; existing Slot intentions remain real and deliverable. The processing
step consumes that witness from the script's actual node Document, which can
differ from its shadow-connected host's Document. No URL is classified, fetched
or decoded, no success/error/event/promise is fabricated, and no execution occurs.
Resource-capable processing is not admitted. Retained-history installation uses
its actual qualified transition before using this closed premise.

Clone/import copies the actual true or false bit through the shared clone rule,
sets Clone cause after copied attributes and before parent insertion, and uses
the target native environment. Adoption preserves state/cause. An insert followed
by detach can therefore leave a required state-only ElementState change despite
unchanged final topology. Snapshots/captures retain their original state. Native
fragment installation uses fresh native cause and these same insertion hooks;
it does not certify parsed/resource history. Scalar state adds no retained string
cost. Processing reuses the finite canonical-node static traversal and fallible
buffers with per-node cancellation; it introduces no task queue or numeric quota.
SVG use-instance trees and resource-capable processing remain separate domains.

[Back to surgeist-dom](../README.md).
