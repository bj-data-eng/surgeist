# Surgeist Font

`surgeist-font` owns immutable font resource admission, exact encoded-source
provenance, decoded-resource identity, and original collection-face selection.
It has no catalogue, matching, shaping, glyph-admission or rendering authority.

## Raw resource admission

`FontResource::from_sfnt` accepts standalone SFNT/OTF and version 1 or 2 TTC/OTC.
Supported scaler signatures are `0x00010000`, `OTTO`, `true`, and `typ1`.
Every face is validated before any resource can escape. Original collection
order is retained, even when directory offsets are in a different order.

The structural boundary checks complete headers and directories, checked
offset/length arithmetic, printable table tags with trailing-space padding,
strictly increasing unique tags, four-byte alignment, table ranges, and region
overlap. Exact same-tag table spans may be shared by different collection faces;
partial overlaps and aliases between different kinds of regions are rejected.
Repeated collection entries may reference the same complete directory. A TTC v2
signature declaration must be coherent and identify a disjoint final region;
the signature's cryptographic contents are not interpreted.

This is a deliberately strict directory policy, based on the
[OpenType container specification](https://learn.microsoft.com/en-us/typography/opentype/spec/otff).
The `true` and `typ1` signatures follow the
[Apple TrueType container definition](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html).
Admission does not verify checksums, table-padding contents, required tables,
outlines, glyphs, internal table offsets, or other OpenType semantics. Search
accelerator fields are ignored; lookup derives its bounds from the validated
directory. A structurally admitted resource is not a semantic font sanitizer or
a consumer capability certificate.

Raw resources retain the input `Vec` without copying its payload. `bytes()` is
the complete decoded container expected by SFNT consumers; `source().bytes()`
retains the exact encoded input, including unused trailing bytes. Their raw
payload allocation is shared, but source and decoded-resource identities have
different types and owners. Equal bytes never establish identity. Clones retain
identity; source and resource IDs are strong handles, and a `FontFace` retains
its complete resource. Face equality uses resource identity and original index.
IDs are process-local handles, with no serialization or ordering contract.

## Explicit limits and allocation accounting

Construct `DecodeLimits::new(encoded_bytes, decoded_bytes, table_bytes, tables,
faces, workspace_bytes, live_bytes)` with caller policy. There is no implicit
default. Zero is a valid cap. The limits mean:

- `encoded_bytes`: retained encoded `Vec` capacity, including unused capacity.
- `decoded_bytes`: logical decoded SFNT container length.
- `table_bytes`: logical length of each individual top-level table.
- `tables`: aggregate table count across all faces, counting shared tables for
  each reference.
- `faces`: collection-face count; a standalone resource has one face.
- `workspace_bytes`: requested temporary validation allocation payload.
- `live_bytes`: requested simultaneously live heap payload plus known owned
  metadata, including retained encoded capacity, distinct decoded capacity when
  present, source/resource control data and reference counters, retained face
  offsets, and temporary validation regions. Raw payload is charged once.

Count and byte arithmetic is checked before variable allocations. Admission
scans complete directories without allocating, then fallibly reserves the
face-offset and region arrays once using `Vec::try_reserve_exact`; the region
sort needs no further allocation. Limits charge these arrays' requested
element payload. Allocator bookkeeping, allocator rounding/excess capacity,
stack frames and caller-held handles are outside this accounting model.
The original transferred `Vec`'s full reported capacity is charged regardless
of how the caller obtained it. Bounds describe this admission operation, not a
global process budget or all resources held by an application.

`AllocationFailed` reports failed fallible variable reservations. Stable Rust's
fixed-size `Arc::new` control allocations use the standard allocation-failure
handler; this API cannot promise recoverable process-wide out-of-memory.
The caller's original allocation occurred before transfer and is outside this
call's allocation-failure guarantee. Rejection drops transferred ownership and
returns no partially admitted resource.

## Capability and checks

WOFF1 and WOFF2 decoding are separate implementation work; this raw entry point
does not accept either encoded format. It exposes no placeholder decoder.

From the repository root:

```sh
cargo test --offline --locked -j 1 -p surgeist-font
cargo clippy --offline --locked -j 1 -p surgeist-font --all-targets --no-deps -- -F unsafe-code -D warnings
```

The tests use small authored structural containers and the repository's existing
Ahem fixture by reference. New API behavior is checked alongside functional
implementation; no preimplementation regression RED is claimed.
