# Surgeist Font

This crate owns immutable encoded sources, structurally admitted decoded SFNT
resources, stable allocation identity, collection-face selection and admission
limits. Its [README](README.md) defines the current boundary and focused checks;
[`src/lib.rs`](src/lib.rs) is the public front door and
[`src/sfnt.rs`](src/sfnt.rs) owns raw structural validation.

Consumers own table semantics and their adapters. Admission is not catalogue,
shaping, glyph or render qualification. Retain exact source provenance and
actual collection order; equal content is not identity.
