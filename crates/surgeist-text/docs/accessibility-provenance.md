# Accessibility projection provenance

[The owned projection](../src/accessibility.rs) adapts font metadata, text-run
attributes, and same-line node links from Parley 0.9.0
`src/layout/accessibility.rs`, Copyright 2025 the Parley Authors, under
Apache-2.0 OR MIT. Its exact upstream file SHA-256 is
`8247c217b55e10d307df199c6de8b8c9d95c4796ce11b86b7e6ef1e96f0e860e`;
the registry package checksum is
`8fad031076f48f0d4d85ce1aea9b94b4e715a4d636a030a123038f8f5b5e4343`.
The [crate notice](../NOTICE.md) links the retained license texts.

Surgeist replaces the upstream layout-accessibility carrier with bindings to its
own immutable layout identity and exact UTF-16 source. Selectable units respect
source graphemes and actual provider clusters, rather than upstream style-index
splits. Host strings remain UTF-8 as required by AccessKit. Checked unit lengths
and node identities, off-side publication, mixed-style metadata summaries, and
source-unit affinity/selection mappings are owned adaptations. The owned engine
supplies original-atom analysis, actual shaping and finalized clusters. Host
strings use the same original semantic atoms as the host byte projection, so a
real source barrier cannot fuse surrogate halves during accessibility conversion.
Source-only and placed projection share that same analysis identity. Composed
lines retain their exact prepared, selected and placed associations; they do not
create another accessibility source owner.
See the [complete mechanism derivation](owned-engine-provenance.md).
