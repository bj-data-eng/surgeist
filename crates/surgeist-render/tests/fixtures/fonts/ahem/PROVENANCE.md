# Ahem Test Font Fixture

Source: <https://www.w3.org/Style/CSS/Test/Fonts/Ahem/>

Downloaded files:

- `Ahem.ttf` from `ahem.ttf`
- `COPYING`
- `UPSTREAM-README` from upstream `README`

License: public domain with Creative Commons Zero fallback, as stated in
`COPYING` and `UPSTREAM-README`.

SHA-256:

- `Ahem.ttf`: `f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc`
- `COPYING`: `3100f58a80de8fcd9dee4280dc4e52a45cc25a2084006e6b8fb5750708a388f8`
- `UPSTREAM-README`: `44ae87023a6cce5014e8cdd49028da600ccc4e04f34dbb41c7f258ed4455da8a`

## Probe Result

Probe command:

```sh
cargo run --manifest-path tmp/ahem-probe/Cargo.toml -- tests/fixtures/fonts/ahem/Ahem.ttf
```

Font metrics:

- units per em: `1000`
- ascender: `800`
- descender: `-200`
- line gap: `0`
- glyph count: `281`

Stable glyphs for deterministic render tests:

| Character | Codepoint | Glyph id | Advance | Bounds | Use |
| --- | --- | ---: | ---: | --- | --- |
| `A` | `U+0041` | `35` | `1000` | `(0,-200)..(1000,800)` | full em square |
| `X` | `U+0058` | `58` | `1000` | `(0,-200)..(1000,800)` | full em square |
| `p` | `U+0070` | `82` | `1000` | `(0,-200)..(1000,0)` | descent-only box |
| `É` | `U+00C9` | `100` | `1000` | `(0,0)..(1000,800)` | ascent-only box |
| space | `U+0020` | `3` | `1000` | no outline | full-advance spacer |
| no-break space | `U+00A0` | `153` | `1000` | no outline | full-advance spacer |
| zero-width space | `U+200B` | `253` | `0` | no outline | zero-advance spacer |
| em space | `U+2003` | `247` | `1000` | no outline | full-advance spacer |
| Greek upsilon | `U+03A5` | `277` | `1000` | `(200,-200)..(400,800)` | vertical stripe |
| Greek chi | `U+03A7` | `274` | `1000` | `(0,400)..(1000,600)` | horizontal stripe |

Use character-to-glyph resolution through the text stack where possible. The
glyph IDs above are fixture-specific probe facts for tests that need to assert
the exact lowered glyph stream.

## Authored outline derivatives

`tests/prepared_outline.rs` builds an in-memory derivative from these licensed
bytes using authored OpenType table data. Its single `wght` axis has normalized
default 0 and endpoint +1. One actual gvar tuple moves X's two right-edge points
by +100 font units at +1; its other points, phantom points and y deltas remain
zero. The independent expected width at UPEM1000 is therefore
`1000 + normalized_coordinate * 100`. No downloaded font or upstream test-data
package is used, and the original `Ahem.ttf` bytes remain unchanged.

The same test builds literal simple/composite contour and dense/sparse delta
variants in memory. The off-curve contour's source points are OFF `(500,1000)`,
ON `(1000,0)`, ON `(0,0)`; HarfBuzz starts at the second point and closes through
the first control point. A wide dense delta variant uses +32768 on the right
edge to distinguish exact full-scalar f32 application from overflowing partial
Fixed conversion. Malformed/truncated variants deliberately test refusal; these
fixtures do not claim general font conformance or validate unexecuted payloads.
