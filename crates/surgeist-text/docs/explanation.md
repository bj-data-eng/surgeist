# Explanation

## The layout boundary

`surgeist-text` receives authored text with resolved styles and produces text
layout facts. `Source` carries the string, style spans, optional identity and
revision, and inline boxes. `Style` supplies the default resolved text style;
`Options` supplies paragraph constraints. [System](../src/system.rs) validates
these inputs, derives a cache key, and either returns an exact matching cached layout or uses owned analysis,
font preparation, selected-line finalization and placement. The private standalone
chooser calls the same finalizer; source and fonts stay retained by immutable phases.

The resulting [Layout](../src/layout.rs) is an immutable snapshot containing
authored source and the shaped layout. It exposes line metrics, resolved font
data, glyphs, clusters, decorations, inline boxes, hit tests, selection geometry,
and low-level movement. These facts let a caller build its own surface or editor
without making the text crate own widgets or application commands.

[The public front door](../src/lib.rs) excludes rendering, widgets, document
identity, style resolution and application commands. Each consuming subsystem
owns its direct adapter: Render consumes Text's borrowed actual run/face/instance
readback under its optional `render-text` feature. Text has no Render dependency.
The adapter borrows the original immutable Layout for paint/interaction association;
it does not choose breaks or create future selected-line identities. The root
facade provides optional composition and shared API auditing.

## Source coordinates remain authored coordinates

Text ranges count exact UTF-16 code units. Immutable source preserves isolated
surrogates and endpoints within pairs for edits and extraction. Original atoms retain raw Unicode properties and exact source associations.
Separate private segmentation and glyph projections map their actual boundaries back to units.
Caret geometry uses legal grapheme/provider stops and affinity, while partial-unit
selection geometry covers the complete typographic unit. Isolated units have original bidi class L; their glyph-only U+FFFD replacement
does not supply original property analysis. Host UTF-8 accessibility refuses
ill-formed original text atomically rather than repairing the source.

The separate [source-processing phase](../src/processing.rs) executes lexical
Collapse, Discard, Preserve and PreserveSpaces with exact authored correspondence
before fonts. It retains actual deletion reasons and original scalar context;
its partial plaintext consumer borrows the same result. Paragraph layout still
rejects Collapse because it has not integrated this phase with its own wrapping,
selected-line trimming and finalization. Having a source map does not supply
those missing semantic steps. Similarly, style resolution and symbolic colors require caller
policy beyond the concrete values accepted here. The
[support matrix](reference.md) makes those limits observable instead of silently
substituting behavior.

Inline-box alignment illustrates another distinction. Authored alignment and
baseline-shift requests survive validation and appear in metric facts, while the
standalone path retains its current bottom-on-baseline geometry without
applying those CSS alignment requests. Consumers must distinguish retained intent from completed placement.

## Identity, edits, and caching

A source identity is optional caller-supplied metadata; the crate does not define
a document identity system. The [cache key](../src/cache.rs) includes source ID,
revision, text and inline-box content, default and span styles, layout options,
and the authentic font catalogue publication (issuer identity and checked generation). Identity alone therefore does not stand in for the
contents of a layout request. A key selects a cache candidate; captured inputs
are compared before reuse. Explicitly supplied analyses additionally require the
same opaque analysis identity, so equal content cannot substitute another owner.

[TextEdit](../src/edit.rs) normalizes insertion, replacement, or deletion to a
validated range and replacement text. Applying it rechecks the range against
the target source, projects span endpoints and inline-box anchors, and increments
the source revision once with checked arithmetic. Exhaustion refuses the edit
before changing source data. It does not require that
the target source have the identity or revision of the source used to construct
the edit. A caller that needs stale-edit detection must enforce that separately.

Projection retains the existing spans and boxes. Span starts inside replaced
text move to the edit start; span ends inside or at the replaced end move to the
end of inserted text. Box anchors at or before the edit start stay put, while
anchors inside the replaced interval move to the inserted end. Later coordinates
shift by the code-unit length difference. Applying an edit returns a `Source`; a new
layout request performs validation and shaping for that version.

Each `System` owns direct provider scratch and an in-memory layout cache. Repeated identical
requests reuse cached layouts; counters expose hits, misses, font refreshes, and
invalidations. Registration validates all faces off-side before publishing any
of them. `refresh_fonts()` constructs a fresh configured font collection and
source cache, then replays the actual retained registered bytes. Both operations
check generation exhaustion first and invalidate layouts only on success.
Failures preserve the current catalogue, cache and statistics.

Registered-only mode excludes system discovery and fallback without inventing a
generic default family. System mode keeps the provider's normal matching and
fallback behavior. A publication identifies registrations and discovery; it is
not an eager snapshot of every lazily loaded system font file. Each retained
layout owns the actual selected font bytes, so refresh or System disposal does
not invalidate historical output. No cache size limit is imposed. See
[src/catalogue.rs](../src/catalogue.rs) and [src/system.rs](../src/system.rs).

## Font-independent and positioned accessibility

`AnalyzedSource` owns exact source, actual span winners, original Unicode facts
and logical grapheme boundaries without font selection. Its accessibility front
publishes source text and language with unsupported glyph/font/line/character/parent
geometry omitted. A font preparation and resulting Layout retain that same analysis.
Positioned projection enriches the shared semantic builder from actual finalized
rows. Replacement rebuilds host nodes so optional fields clear coherently.

Source-only queries require their exact analyzed binding; placed queries require
both the same analysis and exact placed identity. Updates can enrich or clear a
binding, while stale queries return a typed mismatch. Original isolated units stay
analyzable, but AccessKit's scalar UTF-8 text domain refuses them without changing
an existing binding. See [source provenance](owned-engine-provenance.md).
