# Typed Windows Bridge Guide

This crate owns safe, typed Windows query mechanics and owned native facts.
Consumers own policy and native-to-domain conversion. Load `src/lib.rs`,
`src/fonts.rs`, and the README for the public boundary; `src/platform.rs` owns
safe WinSafe calls and `src/native_preferences.rs` owns fixed-action font queries.

The explicitly authorized unsafe exception applies only to the two private,
typed `SystemParametersInfoForDpi` getters. The crate root denies unsafe;
only `native_preferences` allows it. Keep raw bindings, action selection,
pointers, native handles and handle constructors private. No blanket upstream
reexports. No font catalogue, realization, loading, shaping or rendering.

Preserve complete owned observations, native ambiguity, logical-unit signs,
reference DPI, flags and names. Fail whole queries on native failure or bounded
enumeration overflow. Font data is a logical request, not a realized identity.
Before root v1 there are no compatibility shims, per the root guide.

Run focused crate tests, warning-denied all-target Clippy, and no-dependency docs
through `pisct run` with offline/locked Cargo and one build job. See README for
commands. Native integration tests require Windows 10 version 1607 or newer.
The root owns workspace membership, lockfiles, API generation and publication.
