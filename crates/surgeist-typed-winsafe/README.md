# surgeist-typed-winsafe

Safe, typed Windows observations for Surgeist host adapters. This crate owns
native query mechanics; consuming adapters own interpretation and policy.
It depends directly on pinned [WinSafe](https://github.com/rodrigocfd/winsafe)
0.0.29 `user` APIs on Windows only. It has no facade dependency.

`monitors` returns owned GDI device names, frame edges and work-area edges.
Every native monitor must be observed successfully or the whole query fails.
`display_interfaces` returns complete owned device-interface strings, preserving
empty, duplicate and multiple identities for consumers to evaluate. Its bounded
enumeration rejects overflow and clears last error before every iterator step.

`top_level_windows` returns opaque borrowed window identities with query-only
title, process, monitor, client geometry and DPI methods. Windows may destroy
or reuse an identity between calls. No native handles, pointer constructors,
generic action selectors or WinSafe reexports escape this crate.

`ui_fonts_for_dpi` reads the actual caption, small-caption, menu, status and
message preferences from `SPI_GETNONCLIENTMETRICS`, and icon-label preferences
from `SPI_GETICONTITLELOGFONT`. The individually typed getters are also available.
Every query uses an explicit nonzero DPI with
[`SystemParametersInfoForDpi`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfofordpi),
which requires Windows 10 version 1607 or newer. No thread DPI state changes.
The `LogicalFont` model retains the complete
[`LOGFONTW`](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-logfontw)
preference, including native flags, defaults, signed dimensions, and UTF-16 name.
Invalid UTF-16 or missing string termination fails without replacement text.
Names identify font-mapper requests; the crate does not claim realized font
identity, typographic metrics or a point size. Font discovery, loading, matching,
shaping and rendering belong to consuming owners.

All public APIs are safe. A crate-specific authorized exception confines unsafe
code to a private module containing exactly two fixed-action preference calls.
Each call documents its buffer layout, initialization, size and lifetime.
The remaining code denies unsafe; unsafe operations in unsafe functions are
forbidden throughout. Non-Windows queries return `UnsupportedPlatform`, while
owned observation types remain available for portable adapters.

Focused checks from the repository root:

```sh
pisct run --timeout-seconds 900 -- cargo test --offline --locked -j 1 -p surgeist-typed-winsafe
pisct run --timeout-seconds 900 -- cargo clippy --offline --locked -j 1 -p surgeist-typed-winsafe --all-targets -- -D warnings
pisct run --timeout-seconds 900 -- cargo doc --offline --locked -j 1 -p surgeist-typed-winsafe --no-deps
```

The normal tests verify preservation, decoding and complete collection without
assuming particular host preferences. `tests/native_observations.rs` exercises
the real Windows query paths. Native preferences are observations rather than
an atomic OS snapshot. See [NOTICE.md](NOTICE.md) and [LICENSE](LICENSE).
