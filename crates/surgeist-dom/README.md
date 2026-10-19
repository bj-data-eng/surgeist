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
the relevant API.

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

A context-free ordinary or Source operation that reaches a bound style
notification, a relevant context change or retirement returns
`InlineStyleContextRequired` before that publication. Same-value style setters
still reach the notification barrier; missing removals and unrelated attributes
retain their ordinary behavior. Same-scope moves maintain sparse membership.
Bound retirement, retained context-bearing Source synchronization and CSSOM
owner feedback are unavailable. Inline formation has explicit original DOM
custody/work limits alongside CSSOM's independent logical/source quotas;
neither is a process memory measurement.

For contribution routing and focused verification commands, read the
[crate guide](AGENTS.md). The [source](src/lib.rs) and [tests](tests/) establish
current behavior; [source editions](../../references/SOURCE-EDITIONS.md) and the
[DOM/XML source basis](docs/canonical-tree.md#source-basis) retain provenance.
