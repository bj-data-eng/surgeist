# surgeist-dom

`surgeist-dom` owns the canonical live document tree: stable identities, atomic
edits, immutable snapshots and complete change coverage. It supports native
Document, Element, DocumentFragment, ShadowRoot, Text, Comment,
ProcessingInstruction and DocumentType nodes, with bounded HTML family adapters.

The crate directly consumes HTML language values and decisions, completed
HTML-parser reports, HTML semantic queries, CSS values and CSSOM identities.
DOM owns its adapters to those producers; Runtime owns scheduling, service
delivery and host/resource orchestration. Runtime and the facade are consumers,
not dependencies of this crate. See [Cargo.toml](Cargo.toml) and the
[workspace architecture](../../docs/explanation.md) for dependency boundaries.

## Publish an edit and retain a snapshot

This example comes from the [crate-level documentation](src/lib.rs). Choose
finite limits, build a batch, and resolve its creation tickets after publication:

```rust
use std::sync::atomic::AtomicBool;
use surgeist_dom::{Dom, DomBatch, DomLimits};

fn main() -> Result<(), surgeist_dom::DomError> {
    let names = surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)?;
    let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192, names)?)?;
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let comment = batch.create_comment(&document, "retained");
    batch.insert_before(&comment, &document, None);
    let committed = dom.apply(batch, &AtomicBool::new(false))?;
    let comment = committed.created(&comment)?.expect("surviving creation");
    let snapshot = dom.snapshot();
    assert!(snapshot.node(comment)?.is_connected());
    Ok(())
}
```

The snapshot owns its captured state after later edits or destruction. Failed
or cancelled batches preserve live state; final semantic no-ops do not advance
the revision. The [canonical tree contract](docs/canonical-tree.md) explains
capacity accounting, exact UTF-16 values and publication behavior.

## Read the contract for the affected domain

| Work | Owning explanation |
| --- | --- |
| Tree, values, identities and publication | [Canonical tree](docs/canonical-tree.md), [storage/admission](docs/storage.md), [document creation/import](docs/document-creation.md), [Ranges/designMode](docs/ranges-and-editing.md) |
| Native controls | [Forms/Select](docs/forms-and-select.md), [Input](docs/input.md), [Output/Textarea](docs/text-controls.md) |
| Embedded content and scripts | [Canvas/Marquee/Frame/Iframe/Img](docs/embedded-content.md), [HTML/SVG Script](docs/scripts.md) |
| Document environment | [Base/Link/A/Area/Meta](docs/urls-and-metadata.md), [host observations and hosted documents](docs/hosted-documents.md) |
| Owned trees and semantic queries | [Shadow/Template](docs/shadow-and-template.md), [Slot/checkpoints](docs/slots-and-checkpoints.md), [semantic captures](docs/semantic-captures.md) |
| External work and interaction | [Source/Style](docs/source-and-style.md), [Dialogs/focus/top layer](docs/dialogs-and-focus.md), [Details tasks](docs/details-tasks.md), [fullscreen](docs/fullscreen.md) |
| Other structural state and installation | [View Transitions](docs/view-transitions.md), [native HTML/parser reports](docs/html-installation.md) |

## Capability boundaries

This crate does not claim complete DOM or HTML conformance. A missing native
family or unsupported historical/environment profile returns its precise
requirement; an admitted structural state does not certify rendering, resource
readiness, script execution, platform success or event delivery. Ordinary atomic
batches refuse reached external-service boundaries; Source operations retain
original continuations and mandatory tails across publication and retry.

Each topic keeps its exact supported paths, limits and failure behavior beside
the relevant API. Completed parser installation retains sealed pending stylesheet
association history on the canonical owner, independently of output packets.
[Pending association custody](docs/html-installation.md#pending-stylesheet-associations)
explains discovery, finite backing admission and native crossing refusals. The
[whole parsed-Meta consumer](docs/html-installation.md#whole-parsed-meta-consumption)
joins authentic supported history with real CSSOM owners. Original Style
processing retains its exact synchronous and asynchronous obligations until
[explicit whole settlement](docs/html-installation.md#whole-original-style-settlement)
publishes authenticated readiness and removes that contribution's guards.

Inline-style identity is an explicit lazy capability. After installing
`DomInlineStyleLimits`, `Dom::inline_style` binds a live HTML, SVG or MathML
Element to the actual same-object `CssStyleProperties` in this DOM owner's
private CSSOM store. DOM checks the current Document observation and parser
mode, derives canonical owner/base facts, and supplies the untouched UTF-16
attribute to CSSOM. The caller supplies explicit support/profile inputs and any
required declaration decisions. `capture_inline_style` couples immutable DOM
and declaration observations, which remain readable after owner destruction.

`apply_with_inline_styles` synchronizes ordinary batches with explicit
Element/Document profiles qualified by that exact batch and canonical basis.
Actual attribute notifications and relevant context barriers retain invocation
order, so moving a block changes its current packet without reparsing earlier
checked values. One CSSOM batch commits after all fallible DOM formation and
sharing, followed by infallible canonical publication. The result retains both
owners' actual publication evidence. Created Document operands use real batch
tickets; DOM derives their staged mode and base without a tentative public ID.

`begin_source_edit_with_inline_styles` retains published-Element/Document
profiles for one authentic Source root. Each `advance_with_inline_styles` borrows
its pure resolver for that call. Actual hooks record inputs during speculative
admission; parsing and resolution run only after full segment/rebase/sharing
formation. The single CSSOM commit precedes infallible DOM publication, and the
advance returns the original `DomChanges`/`CssomCommit` pair once at that boundary.
Drop/resume preserves the original invocation, profiles, cumulative work and
mandatory tails. An authenticated callback can create the first binding without
replacing the original Source capability. The context-bearing fullscreen handoff
service joins an admitted scheduling job with the original paired publication;
the context-free service refuses reached inline inputs before publishing that
prefix. DOM admits wrapper and continuation custody; CSSOM receipts retain their
independent producer output contract, with no composed physical-memory bound.

Context-bearing destruction retires bound declarations in the same CSSOM batch
as its canonical destruction segment. Real child-removal hooks run first and
retain their live context profiles; retirement itself requires no parsing or
declaration decision. Current retired handles become missing, surviving bindings
keep their identities, and released binding capacity can be reused. Historical
captures remain readable; a reused node slot never inherits a retired binding.

A context-free ordinary or Source operation that reaches a bound style
notification, a relevant context change or retirement returns
`InlineStyleContextRequired` before that publication. Same-value style setters
still reach the notification barrier; missing removals and unrelated attributes
retain their ordinary behavior. Same-scope moves maintain sparse membership.
The standard `set_inline_style_css_text`, `set_inline_style_property` and
`remove_inline_style_property` fronts operate on the same bound Properties
object with its checked current inputs. They run CSSOM's actual standard
executor, then synchronously prepare and publish canonical feedback. A refusal
after CSSOM commits returns `DomInlineStyleMutation::Incomplete`, retaining the
original return value, publication, diagnostics, effect and cumulative feedback
work. Dropping the operation parks it; `resume_inline_style_feedback` retries
only the owner write. Its consuming `complete` returns either the original
completed components or the same incomplete operation. Effect-free standard
no-ops complete without a DOM publication or changing raw attribute spelling.

Pending feedback serializes inline-store commits and refuses reached bound
context/attribute changes before a Source request can escape. Unrelated DOM and
root stylesheet work remain independent, and other bindings can still be
captured. If feedback originated in a genuine Source callback, its exact dormant
observation can finish that retained owner write after dispatch scope ends; that
observation's mandatory tail cannot advance until its feedback drains. This
does not grant general caller reentry. Only an exactly classified echo skips
inline ingestion; other native hooks still execute. Historical captures remain
readable, while the incomplete binding refuses a latest coupled capture.
Terminal owner effects remain failed. The consuming operation's `reconcile`
returns the exact displaced terminal evidence plus a fresh Pending lease before
attempting canonical feedback. It requests a real CSSOM synchronization commit
for the current exact binding; it never repeats the standard setter, parser or
resolver. `DomInlineStyleOutcome` distinguishes the original `Standard` return,
publication and diagnostics from a `Synchronization` commit and its detached
affine effects. Completing the fresh lease applies only that new effect; it does
not report the old failed attempt as applied. A refused reconciliation parks the
same old outcome for `resume_inline_style_feedback`.

Reconciliation preserves the original work counter, observer custody and inline
policy. Its four DOM feedback limits may only increase. Each emitted terminal
outcome consumes one cumulative output, and admission reserves the eventual
completion as well; completion also counts its actual DOM change records. Actual
work spent on refused attempts remains spent. Old terminal custody and the
replacement holder overlap during admission, before the final fallible CSSOM
commit. There is one resident attempt and no retained receipt history. Returned
terminal evidence reports work at displacement. Pending and terminal residents
continue to gate bound retirement and inline-store commits. Inline formation has
explicit DOM custody and work limits alongside CSSOM's independent logical/source
quotas; CSSOM private receipt backing has no DOM physical tariff, and neither
limit is a process memory measurement.

For contribution routing and focused verification commands, read the
[crate guide](AGENTS.md). The [source](src/lib.rs) and [tests](tests/) establish
current behavior; [source editions](../../references/SOURCE-EDITIONS.md) and the
[DOM/XML source basis](docs/canonical-tree.md#source-basis) retain provenance.
