# Font metric selection provenance

Text's private `line_origin` predicates in [metrics.rs](../src/font/metrics.rs)
are adapted from the static selection branch in Skrifa 0.42.1
[metrics.rs](https://github.com/googlefonts/fontations/blob/ba086017d5bc36ff0b0e9502f6ea4f30efa70a37/skrifa/src/metrics.rs).
The retained package records that exact VCS revision. Its source file SHA256 is
`81623be46fe3406785a30780f9f4745caeb8d64492aa28952b35176373ea249f`.
Applicable licenses are [MIT](../licenses/fontations/LICENSE-MIT) or
[Apache License 2.0](../licenses/fontations/LICENSE-APACHE), retained with the
crate's [notice](../NOTICE.md).

The provider still supplies the unscaled base numbers. The small adapted
predicates identify which stored fields those numbers came from: explicit typo,
hhea, fallback typo or Windows ascent/descent with retained hhea leading. Text
does not rerun selection after applying variation. These predicates do not copy
the provider's whole metric engine or its permissive MVAR wrapper.

[variation.rs](../src/font/metrics/variation.rs) owns strict used-table/index/row
completeness, nullable-slot semantics, checked accumulation and exact widening.
It calls the safe retained read-fonts 0.39.2 parsing and region-scalar APIs through
Skrifa's public reexport; no generated parser or interpolation implementation is
copied. Canonical coordinate conversion is scoped scratch; returned metrics
retain the original resource/instance loan. The [current metric
contract](reference.md#glyph-free-associated-font-metrics) states the supported
format and output conventions.
