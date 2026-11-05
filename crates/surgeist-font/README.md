# Surgeist Font

`surgeist-font` owns immutable font resource admission, exact encoded-source
provenance, decoded-resource identity, and original collection-face selection.
It has no catalogue, matching, shaping, glyph-admission or rendering authority.

`FontResource::decode(bytes, limits)` accepts raw SFNT/OpenType, collections,
WOFF1 and WOFF2. Each result retains the exact source and the admitted decoded
backing; `face(index)` selects an original collection index. All faces are
admitted atomically. Strong face and identity handles keep their resources alive.

## Compressed containers

WOFF1 supports compressed and uncompressed tables, validates table checksums,
preserves physical table order and reconstructs SFNT search fields. Checksum
verification is an explicit decoder integrity policy; WOFF's normative producer
validation requirement is separate. An unsigned font's whole-font adjustment
is corrected when needed. A correction that would invalidate a retained `DSIG`
is refused instead of silently changing signed data.

WOFF2 supports single fonts and collections, transformed `glyf`/`loca`/`hmtx`,
shared outlines with separate metrics, and metrics inferred from untransformed
outlines. Original face order and shared table associations survive decoding.
Opaque shaping, variation and color tables are retained. Unknown transforms and
unsupported SFNT flavors are typed refusals. Decoding does not certify those
tables for a consumer's particular shaping or glyph backend.

The original encoded bytes retain optional metadata and private data. These
blocks' structure is checked but their contents are unexposed and never used to
load a font. Metadata XML is not inflated or interpreted. WOFF2's reserved value
and reference-only total size do not determine acceptance or allocation.
Reconstructed table sizes are independently derived and bounded.

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
- `workspace_bytes`: requested temporary validation/reconstruction payload and
  explicitly accounted fixed decompressor state.
- `live_bytes`: requested simultaneously live heap payload plus known owned
  metadata, including retained encoded capacity, distinct decoded capacity when
  present, source/resource control data and reference counters, retained face
  offsets, and temporary validation regions. Raw payload is charged once.

Count and byte arithmetic is checked before variable allocations. Admission
scans complete directories without allocating, then fallibly reserves the
face-offset and region arrays once using `Vec::try_reserve_exact`; the region
sort needs no further allocation. Limits charge these arrays' requested
element payload. Compressed decoding additionally charges all intermediate
buffers, replacement-buffer overlap and provider storage. Brotli's strict
decoder uses thirteen fully prepared cells, about 19.2 MiB of payload plus
state and stream output; insufficient workspace is refused before provider
entry. Callbacks recycle this storage without allocating. Allocator-granted
reported capacity excess is checked before retaining or initializing it.
Allocator bookkeeping and unreported allocator rounding,
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

The binary-resource API is host-independent, requires no system font discovery,
and has no optional corpus-only decoder. See [source provenance](PROVENANCE.md),
[dependency attribution](NOTICE.md) and [fixture sources](tests/fixtures/PROVENANCE.md).

From the repository root:

```sh
cargo test --offline --locked -j 1 -p surgeist-font
cargo clippy --offline --locked -j 1 -p surgeist-font --all-targets --no-deps -- -F unsafe-code -D warnings
```

Tests use independently authored wire cases, retained original Ahem WOFF2,
the repository's later Ahem TTF, and independently sourced CFF/variable-color
fonts. Full table contents, actual sharing, bounds and malformed input are
observable controls. New API behavior is checked alongside functional
implementation; no preimplementation regression RED is claimed.
