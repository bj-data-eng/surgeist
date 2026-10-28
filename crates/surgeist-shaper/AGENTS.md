# surgeist-shaper Crate Guide

Read the [repository guide](../../AGENTS.md) for shared authority, compatibility,
subagent, safe Rust and process policy. This supplement grants no additional
authority.

## Product Boundary

Shaper owns OpenType/AAT substitution and positioning, completion/refusal,
joining correspondence, ordinary-symbol presentation, default-ignorable custody
and supplied vertical positioning. Text owns its adapter, paragraph
interpretation, font selection and composition. Shaper has no production
dependency on Text, Layout, Runtime or the facade. `surgeist-shape` is the
separate geometry crate.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package, library, MSRV, features, dependencies and inherited lints | [Cargo.toml](Cargo.toml) |
| Public front door and typed errors | [src/lib.rs](src/lib.rs), reexported owners |
| Provider mechanisms and test registrations | [src/hb/mod.rs](src/hb/mod.rs) |
| Maintained extensions, resource bounds, origins and licenses | [PROVENANCE.md](PROVENANCE.md), [NOTICE.md](NOTICE.md) |
| Current use and focused commands | [README.md](README.md) |
| Original package documentation and manifests | [upstream/](upstream/) |

## Generated Artifacts

Generated machines/tables and their Ragel/Unicode source data retain their
original headers and generation commands. Do not hand-edit them. Public API
artifacts belong to the root [audit procedures](../../docs/how-to.md#check-public-api-audits).

## Command Inventory

Use the [README commands](README.md#features-and-verification) from the root
under its process/resource policy. Default `std` and alloc-only
`--no-default-features` both require qualification. Tests borrow Text's retained
Ahem font by compile-time path; they add no production dependency on Text.
