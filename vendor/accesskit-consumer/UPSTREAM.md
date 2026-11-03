# AccessKit consumer source

This copy retains accesskit_consumer 0.36.0 from the published package at its
[exact upstream revision](https://github.com/AccessKit/accesskit/tree/f40dfc01a0c0e76de535969f82fb35e19513737d/consumer).
The unmodified text.rs SHA256 is
205b9794ebe28de282602fb953f7d8bf317cabeb0bd6920b1c40485ceb9cfed8.

The published normalized Cargo.toml is retained unchanged. Cargo.toml.orig is
retained as reference with one redundant terminal blank line removed. Resolution
uses the consuming workspace lockfile, not the package's
upstream development lockfile. The AccessKit core 0.24 protocol family remains
unchanged.

Local correction: is_word_start, forward_to_word_start and
backward_to_word_start compare stored word starts after lossless widening to
usize, preserving original positions beyond 255. No producer text, selectable
units, word-start encoding or public protocol types are changed.
The maintained library also forbids executable unsafe code in src/lib.rs.

MIT, Apache-2.0, Chromium BSD and AUTHORS are copied unchanged from the exact
upstream revision; existing source copyright and derivation headers remain.
