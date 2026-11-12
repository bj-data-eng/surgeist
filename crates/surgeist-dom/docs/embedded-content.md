# Native embedded-content state

Canvas, Marquee, Frame, Iframe and Img retain their HTML-owned state under native creation and canonical mutation. [Hosted documents](hosted-documents.md) supply the separate controlled child-creation path.

## Canvas and Marquee

Native HTML Canvas retains HTML's actual none-context state: no bound context,
origin-clean true and a transparent-black bitmap description. Exact no-namespace
width/height attribute changes use shared HTML prefix recovery and update the
source bitmap dimensions, including zero and integers beyond machine magnitude.
Missing or failed values use the shared 300×150 defaults. These are bitmap
dimensions, distinct from unsigned-long reflected IDL values. Redundant sets
have no extra none-mode reset action. No pixels, drawing context, image handle
or width×height allocation is fabricated. `canvas_none_state()` exposes the
current shared value. Each retained non-default canonical dimension's ASCII
digits count as stored UTF-16 independently of the raw attribute; inline numeric
defaults own no digit string. Immutable Arc storage prevents transaction and
snapshot copies from cloning unbounded integer Strings.

In an HTML Document, non-NS `WIDTH`/`HEIGHT` inputs reach Canvas hooks after
ASCII lowercase. NS-created uppercase names remain distinct raw attributes and
do not resize the bitmap. The same case rule precedes applicable nonce hooks.

Native HTML Marquee retains HTML's actual initially-on state and closed current
loop index0. `start_marquee` and `stop_marquee` batch commands change only the
on/off state; repeated assignments are semantic no-ops. `marquee_state()` exposes
that current value. Rendering-layer loop increments and animation scheduling
have no admission API. Snapshots preserve stopped state, while native DOM
clone/import initializes a fresh on/index0 Marquee. Canvas clone/import similarly
initializes fresh none state and derives dimensions from copied attributes through
their actual change steps. Neither family adds tree/adoption/CharacterData hooks
beyond the existing applicable common effects. They remain coherent inside
ordinary and hosted shadow trees. New family state reports `ElementState`
independently of raw `Attributes`; equal recovered dimensions with changed raw
spelling report only Attributes. Failed/cancelled edits roll back both domains.

## Frame and Iframe

Native HTML Frame retains the actual initially null child-content relationship,
read through `frame_null_content()`. Its content document/window are null; it
potentially delays load by type but causes no actual child load delay. Frame
post-connection runs after insertion/common steps and only when still
shadow-including connected, using the actual node document's known-null browsing
context. That source guard returns before creating a child or processing src.
Exact null-namespace src changes and removing steps consume HTML's null-content
decisions without URL interpretation, child destruction or navigation effects.
Removal visits the actual ordinary and owned shadow subtree after detachment.
Clone/import creates fresh null family state and copies actual attributes;
adoption preserves state and IDs while changing the real node document. Frame
owns no additional strings and fabricates no Window, child identity or host
capability.

Native HTML Iframe retains HTML's actual fresh never-browsing-context,
unprocessed/null-child state, read through `iframe_native_state()`. Its lazy
navigation flag is false; pending timing fields and lazy-load resumption are
null. It potentially delays load but has no actual child load delay. Frame and
Iframe share the canonical connected-subtree and removing hooks, with each
family consuming its own HTML decision. The actual current node document's
absent browsing context returns before sandbox parsing and child creation.
Exact src/srcdoc/sandbox changes preserve the null-child phase; loading changes
consume the shared language interpreter and eager-transition guard before the
actual null resumption returns. Raw sandbox absence, empty content and tokens
remain distinct attributes, not a numeric live sandbox policy. This admits no
navigation, resource handle, active policy or context-creation transition.
Clone/import-as-clone creates a fresh element and applies copied
attributes; adoption and immutable copying preserve existing family state.
Current null content alone never justifies resetting processed retained history.
Semantic captures retain this state in their source snapshot; the semantic
schema has no child-navigable field to synthesize.

## Img and Picture/Source updates

Native standalone Img retains HTML's actual fresh request tuple: Unavailable,
empty current URL, density1, null preferred dimensions/pending request/last source/
last auto-sizes width/lazy resumption, and no acquired image data. The canonical
dimension-attribute source is the Img's own exact identity. `DomNode::as_img`
reads that tuple, its nullable Form owner and parser flag independently of
constraint-validation controls. Img is nonlisted; its `form` attribute does not
select a listed-control owner, and `set_custom_validity` remains inapplicable.
The existing shared reset cursor operates on its actual nearest Form/tree state.
Both initial and Slot semantic captures include the real established-current
association, without inventing loaded resources or external geometry.

Creation and every relevant mutation synchronously start HTML's update-image-data
algorithm. Its first step waits for the actual node Document to become fully
active. Closed native Documents and retired hosted Documents establish
inactivity through the canonical activity owner, so the wait occurs before the
later images-disabled abort/reset. Unavailable live authority does not establish
inactivity. The current request stays unchanged; no load/error event, request
reset, promise or continuation task is reported as completed. Native activation
and resource-history installation are
unadmitted phases, with `ImageProcessingUnavailable` at a reached unsupported
processing boundary.

`DomImg::suspended_update` reads the latest owned wholly inactive invocation.
Its opaque `DomImageUpdateId` contains the exact target generation and checked
nonwrapping invocation order; the readback transfers no execution authority or
work ownership. Every admitted start advances the high-water mark. A later start
precludes an older wait's post-activation continuation even when the later one
has already aborted. Under this native prefix, superseded waits have no resource,
callback, promise, queued continuation or independent task identity, so the owner
retains only the latest wait and its exact flags. Flags are replaced per start,
never ORed. Equal-value src sets request restart-animation; other admitted causes
use their actual shared decisions. Semantic crossorigin/referrerpolicy states
are compared through HTML's interpreter. Eager loading checks the actual null
resumption without manufacturing work.

Source srcset/sizes/media/type/width/height changes update only following direct
Img siblings of an actual Picture. Source insertion and root removal visit all
actual direct Img children of the current/old Picture in source order. Img
insertion into Picture and root removal from Picture start their own updates;
removal then clears the already-null auto-sizes width. Descendant hooks preserve
the subtree-root distinction. A Source insertion under unloaded Audio or Video
reaches the media resource selection operation described below.

Adoption into a different Document preserves the image request state and starts
a new update against that Document. Its actual removal and Form owner reset
hooks can change the Form association. Detachment preserves waiting ownership, while
executing any real removal mutation. Clone/import-as-clone and native immutable
installation start fresh requests/self dimension sources and run their actual
creation, copied-attribute and tree hooks. Parser history cannot enter through
that native cause. Old snapshots retain exact prior waits and associations.

The latest wait and high-water are fixed per-node storage admitted by existing
canonical node capacity; constructors require no new default-zero quota. Ordered
scans and comparisons use finite canonical node/string/command bounds and poll
cancellation per step. When nested in Select effects they consume that actual
work ledger too. Explicit destruction performs removal hooks first, then cancels
the latest still-owned exact-generation wait. `ImageUpdateCancelled` evidence
includes its operation, exact flags and `TargetDestroyed` reason, even if the
latest invocation started earlier in the same batch. Its fallible evidence bank
is bounded by managed-node capacity and joins the live Select scratch ledger;
all change output is built before atomic publication. Entirely private
create/destroy closures fabricate no public target or cancellation identity.
Detach/adopt do not cancel, pin or rebind old generations. No handoff or scheduler
API is exposed. Readback of an invocation does not establish its current liveness
or transfer its execution authority. ElementState, NodeDocument and ordinary tree/
attribute changes retain their distinct coverage; rejection restores all state,
waits, revision and history together.

## Initial Audio and Video state

The exact HTML expanded names `audio` and `video` establish the unloaded media
state through ordinary native creation, Source creation, cloning, and authored
immutable installation. Read it through `DomElement::initial_media_state`.
The network is empty, no media data is available, playback is paused, and the
show-poster and autoplay eligibility flags are set. The initial playback time
is zero, the current resource address is empty, and media does not delay the
Document load event. The canonical node owns this state and its lifetime.

The unloaded profile has a private, empty shadow boundary with no slots.
Fallback descendants keep their ordinary DOM identity and parentage, and the
CSS flat tree excludes them. The boundary creates no author-visible ShadowRoot.
[Pinned Video shadow tree](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L37209)
· [Pinned Audio shadow tree](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L37718).

Setting a null-namespace `src` reaches the load algorithm. A Source inserted
under unloaded media reaches resource selection. A nonempty Video `poster`
reaches poster determination. Each operation returns
`DomError::MediaResourceEffectUnavailable` with its exact effect before
publication. An empty or absent Video poster retains the absent frame and
null resumption.

Removing media from an ordinary Document tree reaches the stable-state
removal steps. The edit returns `DomError::MediaRemovalStableStateUnavailable`
before publication. Removing nodes from an ordinary detached fragment or
removing a host that contains media only in its shadow tree does not reach this
hook. Parser-created media requires its original lifecycle evidence, which the
current immutable parse report does not supply.
[Pinned HTML removal steps](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L41444)
· [Pinned parser lifecycle](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L42334).

[Back to surgeist-dom](../README.md).
