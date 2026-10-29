use std::hash::{Hash, Hasher};

use super::{
    Brush, Decoration, DecorationBrush, DecorationOffset, DecorationThickness, Font,
    FontCatalogueIdentity, Id, Indent, InlineBox, LineHeight, Options, Size, Slant, Source,
    SourceRevision, Style, ValidatedOptions, ValidatedSource, ValidatedStyle, VerticalAlign,
    Weight, Width,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FontGeneration(u64);

impl FontGeneration {
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[cfg(test)]
impl FontGeneration {
    #[must_use]
    pub(crate) const fn exhausted_for_test() -> Self {
        Self(u64::MAX)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Stats {
    layout_hits: usize,
    layout_misses: usize,
    font_refreshes: usize,
    invalidations: usize,
}

impl Stats {
    #[must_use]
    pub const fn layout_hits(self) -> usize {
        self.layout_hits
    }

    #[must_use]
    pub const fn layout_misses(self) -> usize {
        self.layout_misses
    }

    #[must_use]
    pub const fn font_refreshes(self) -> usize {
        self.font_refreshes
    }

    #[must_use]
    pub const fn invalidations(self) -> usize {
        self.invalidations
    }

    pub(crate) fn record_layout_hit(&mut self) {
        self.layout_hits = self.layout_hits.saturating_add(1);
    }

    pub(crate) fn record_layout_miss(&mut self) {
        self.layout_misses = self.layout_misses.saturating_add(1);
    }

    pub(crate) fn record_font_refresh(&mut self) {
        self.font_refreshes = self.font_refreshes.saturating_add(1);
    }

    pub(crate) fn record_invalidations(&mut self, count: usize) {
        self.invalidations = self.invalidations.saturating_add(count);
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Key {
    source: SourceKey,
    styles: StyleKey,
    basis: RequestBasisKey,
    font_catalogue: FontCatalogueIdentity,
}

impl Key {
    #[must_use]
    pub fn from_validated(
        source: &ValidatedSource,
        style: &ValidatedStyle,
        options: ValidatedOptions,
        font_catalogue: &FontCatalogueIdentity,
    ) -> Self {
        Self::from_parts(source.source(), style.authored(), options, font_catalogue)
    }

    pub(crate) fn from_analyzed(
        analyzed: &crate::engine::AnalyzedData,
        options: ValidatedOptions,
        font_catalogue: &FontCatalogueIdentity,
    ) -> Self {
        Self::from_parts(
            analyzed.processed.source(),
            analyzed.paragraph.base_style(&analyzed.processed),
            options,
            font_catalogue,
        )
    }

    fn from_parts(
        source: &Source,
        style: &Style,
        options: ValidatedOptions,
        font_catalogue: &FontCatalogueIdentity,
    ) -> Self {
        let authored_options = options.authored();
        Self {
            source: SourceKey::new(source.id(), source.revision(), stable_hash_source(source)),
            styles: StyleKey::new(0, stable_hash_style(style, source)),
            basis: RequestBasisKey::Standalone {
                options: OptionsKey::new(stable_hash_options(authored_options)),
                width: authored_options.width.map(OrderedF64),
            },
            font_catalogue: font_catalogue.clone(),
        }
    }

    #[must_use]
    pub const fn source(&self) -> SourceKey {
        self.source
    }

    #[must_use]
    pub const fn styles(&self) -> StyleKey {
        self.styles
    }

    #[must_use]
    pub const fn options(&self) -> Option<OptionsKey> {
        match &self.basis {
            RequestBasisKey::Standalone { options, .. } => Some(*options),
            RequestBasisKey::ExplicitPlacement { .. } => None,
        }
    }

    pub(crate) fn from_placement(request: &crate::PlacementRequest) -> Self {
        let analyzed = &request.prepared.inner.analyzed;
        let source = analyzed.processed.source();
        let style = analyzed.paragraph.base_style(&analyzed.processed);
        Self {
            source: SourceKey::new(source.id(), source.revision(), stable_hash_source(source)),
            styles: StyleKey::new(0, stable_hash_style(style, source)),
            font_catalogue: request.prepared.inner.font_catalogue.clone(),
            basis: RequestBasisKey::ExplicitPlacement {
                prepared: request.prepared.clone(),
                coverage: match &request.coverage {
                    crate::PlacementCoverage::WholeParagraph => ExplicitCoverageKey::Whole,
                    crate::PlacementCoverage::Range { start, end } => ExplicitCoverageKey::Range {
                        start: start.clone(),
                        end: end.clone(),
                    },
                },
                lines: request.entries.iter().map(explicit_entry_key).collect(),
            },
        }
    }

    #[must_use]
    pub const fn font_generation(&self) -> FontGeneration {
        self.font_catalogue.generation()
    }

    /// Returns the authentic catalogue publication used by this request.
    #[must_use]
    pub const fn font_catalogue(&self) -> &FontCatalogueIdentity {
        &self.font_catalogue
    }
}

/// Standalone options and explicit placement are different actual requests.
/// No default Options or content hash manufactures explicit phase authority.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum RequestBasisKey {
    Standalone {
        options: OptionsKey,
        width: Option<OrderedF64>,
    },
    ExplicitPlacement {
        prepared: crate::PreparedText,
        coverage: ExplicitCoverageKey,
        lines: Vec<ExplicitLineKey>,
    },
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum ExplicitCoverageKey {
    Whole,
    Range {
        start: crate::PreparedBoundary,
        end: crate::PreparedBoundary,
    },
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ExplicitLineKey {
    line: ExplicitReceiptKey,
    anchor: [OrderedF64; 2],
    orientation: crate::LineOrientation,
    atomics: Vec<ExplicitAtomicKey>,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum ExplicitReceiptKey {
    Formatted(ExplicitFinalizationKey),
    NonFormatted(ExplicitFinalizationKey),
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ExplicitFinalizationKey {
    start: crate::PreparedBoundary,
    end: crate::PreparedBoundary,
    termination: crate::PreparedTermination,
    incoming: Option<crate::HyphenationContinuation>,
    alternative: Option<crate::PreparedAlternative>,
    alignment: crate::Alignment,
    context: Vec<u64>,
    spacing: Vec<OrderedF64>,
    profile: u8,
}
fn explicit_finalization_key(line: &crate::FinalizedLine) -> ExplicitFinalizationKey {
    let request = &line.inner.selected.request;
    ExplicitFinalizationKey {
        start: request.start.clone(),
        end: request.end.clone(),
        termination: request.termination.clone(),
        incoming: request.incoming.clone(),
        alternative: request.alternative.clone(),
        alignment: request.context.alignment,
        context: crate::protocol::request_context(request.into()),
        spacing: line
            .inner
            .data
            .spacing_amounts
            .iter()
            .copied()
            .map(OrderedF64)
            .collect(),
        profile: line.inner.selected.metric_policy as u8,
    }
}

fn explicit_entry_key(entry: &crate::PlacementEntry) -> ExplicitLineKey {
    let (line, anchor, orientation, atomics) = match entry {
        crate::PlacementEntry::Formatted(placement) => (
            ExplicitReceiptKey::Formatted(explicit_finalization_key(&placement.line)),
            [
                OrderedF64(placement.anchor.inline_origin),
                OrderedF64(placement.anchor.baseline),
            ],
            placement.orientation,
            &placement.atomics,
        ),
        crate::PlacementEntry::NonFormatted(placement) => (
            ExplicitReceiptKey::NonFormatted(explicit_finalization_key(
                placement.line.finalized_line(),
            )),
            [
                OrderedF64(placement.origin.x),
                OrderedF64(placement.origin.y),
            ],
            placement.orientation,
            &placement.atomics,
        ),
    };
    ExplicitLineKey {
        line,
        anchor,
        orientation,
        atomics: atomics
            .iter()
            .map(|atomic| ExplicitAtomicKey {
                occurrence: atomic.occurrence.clone(),
                border: rect_key(atomic.border),
                content: rect_key(atomic.content),
                baseline: OrderedF64(atomic.baseline),
            })
            .collect(),
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ExplicitAtomicKey {
    occurrence: crate::PreparedAtomic,
    border: [OrderedF64; 4],
    content: [OrderedF64; 4],
    baseline: OrderedF64,
}
fn rect_key(rect: crate::Rect) -> [OrderedF64; 4] {
    [
        OrderedF64(rect.origin.x),
        OrderedF64(rect.origin.y),
        OrderedF64(rect.size.width),
        OrderedF64(rect.size.height),
    ]
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceKey {
    id: Option<Id>,
    revision: u64,
    hash: u64,
}

impl SourceKey {
    #[must_use]
    pub(crate) const fn new(id: Option<Id>, revision: u64, hash: u64) -> Self {
        Self { id, revision, hash }
    }

    #[must_use]
    pub const fn id(self) -> Option<Id> {
        self.id
    }

    #[must_use]
    pub const fn revision(self) -> SourceRevision {
        SourceRevision::new(self.revision)
    }

    #[must_use]
    pub const fn hash(self) -> u64 {
        self.hash
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StyleKey {
    revision: u64,
    hash: u64,
}

impl StyleKey {
    #[must_use]
    pub(crate) const fn new(revision: u64, hash: u64) -> Self {
        Self { revision, hash }
    }

    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn hash(self) -> u64 {
        self.hash
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OptionsKey {
    hash: u64,
}

impl OptionsKey {
    #[must_use]
    pub(crate) const fn new(hash: u64) -> Self {
        Self { hash }
    }

    #[must_use]
    pub const fn hash(self) -> u64 {
        self.hash
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OrderedF64(pub(crate) f64);

impl PartialEq for OrderedF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrderedF64 {}

impl std::hash::Hash for OrderedF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

fn stable_hash_source(source: &Source) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.text.hash(&mut hasher);
    source.boxes.len().hash(&mut hasher);
    for box_ in &source.boxes {
        hash_inline_box(*box_, &mut hasher);
    }
    hasher.finish()
}

fn stable_hash_style(style: &Style, source: &Source) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_style(style, &mut hasher);
    source.spans().len().hash(&mut hasher);
    for span in source.spans() {
        span.range().hash(&mut hasher);
        hash_style(span.style(), &mut hasher);
    }
    hasher.finish()
}

fn stable_hash_options(options: Options) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_option_f64(options.width, &mut hasher);
    hash_f32(options.scale, &mut hasher);
    options.alignment.hash(&mut hasher);
    options.text_overflow.hash(&mut hasher);
    options.text_align_last.hash(&mut hasher);
    hash_indent(options.indent, &mut hasher);
    options.quantize.hash(&mut hasher);
    hasher.finish()
}

fn hash_inline_box<H: Hasher>(box_: InlineBox, hasher: &mut H) {
    box_.id.hash(hasher);
    box_.kind.hash(hasher);
    box_.index.hash(hasher);
    hash_size(box_.size, hasher);
    hash_vertical_align(box_.vertical_align, hasher);
    hash_f64(box_.letter_spacing, hasher);
}

fn hash_style<H: Hasher>(style: &Style, hasher: &mut H) {
    style.hanging_punctuation.hash(hasher);
    hash_f64(style.tab_size.value(), hasher);
    style.tab_size.is_spaces().hash(hasher);
    hash_font(&style.font, hasher);
    hash_f32(style.size, hasher);
    hash_line_height(style.line_height, hasher);
    hash_f64(style.letter_spacing, hasher);
    hash_f64(style.word_spacing, hasher);
    hash_brush(style.brush, hasher);
    hash_decoration(style.underline, hasher);
    hash_decoration(style.strikethrough, hasher);
    style.locale.hash(hasher);
    style.direction.hash(hasher);
    style.white_space.hash(hasher);
    style.word_break.hash(hasher);
    style.line_break.hash(hasher);
    style.wrap.hash(hasher);
    style.overflow_wrap.hash(hasher);
    style.text_transform.hash(hasher);
    style.word_space_transform.hash(hasher);
    style.hyphens.hash(hasher);
    style.hyphenate_limit_chars.hash(hasher);
    style.hyphenate_character.hash(hasher);
    style.native_hyphenation_convention.hash(hasher);
}

fn hash_font<H: Hasher>(font: &Font, hasher: &mut H) {
    font.family.hash(hasher);
    hash_weight(font.weight, hasher);
    hash_width(font.width, hasher);
    hash_slant(font.slant, hasher);
    font.variant.hash(hasher);
    font.features.hash(hasher);
    font.variations.hash(hasher);
}

fn hash_weight<H: Hasher>(weight: Weight, hasher: &mut H) {
    std::mem::discriminant(&weight).hash(hasher);
    if let Weight::Number(value) = weight {
        hash_f32(value.get(), hasher);
    }
}

fn hash_width<H: Hasher>(width: Width, hasher: &mut H) {
    std::mem::discriminant(&width).hash(hasher);
    if let Width::Ratio(value) = width {
        hash_f32(value.get(), hasher);
    }
}

fn hash_vertical_align<H: Hasher>(vertical_align: VerticalAlign, hasher: &mut H) {
    std::mem::discriminant(&vertical_align).hash(hasher);
    if let VerticalAlign::Shift(value) = vertical_align {
        hash_f64(value.get(), hasher);
    }
}

fn hash_slant<H: Hasher>(slant: Slant, hasher: &mut H) {
    std::mem::discriminant(&slant).hash(hasher);
    if let Slant::Oblique(angle) = slant {
        hash_option_f32(angle, hasher);
    }
}

fn hash_line_height<H: Hasher>(line_height: LineHeight, hasher: &mut H) {
    std::mem::discriminant(&line_height).hash(hasher);
    match line_height {
        LineHeight::MetricsRelative(value)
        | LineHeight::FontSizeRelative(value)
        | LineHeight::Absolute(value) => hash_f64(value, hasher),
    }
}

fn hash_decoration<H: Hasher>(decoration: Decoration, hasher: &mut H) {
    decoration.enabled().hash(hasher);
    hash_decoration_offset(decoration.offset(), hasher);
    hash_decoration_thickness(decoration.thickness(), hasher);
    hash_decoration_brush(decoration.brush(), hasher);
}

fn hash_decoration_offset<H: Hasher>(offset: DecorationOffset, hasher: &mut H) {
    std::mem::discriminant(&offset).hash(hasher);
    if let DecorationOffset::Absolute(value) = offset {
        hash_f64(value.get(), hasher);
    }
}

fn hash_decoration_thickness<H: Hasher>(thickness: DecorationThickness, hasher: &mut H) {
    std::mem::discriminant(&thickness).hash(hasher);
    if let DecorationThickness::Absolute(value) = thickness {
        hash_f64(value.get(), hasher);
    }
}

fn hash_decoration_brush<H: Hasher>(brush: DecorationBrush, hasher: &mut H) {
    std::mem::discriminant(&brush).hash(hasher);
    if let DecorationBrush::Color(brush) = brush {
        hash_brush(brush, hasher);
    }
}

fn hash_brush<H: Hasher>(brush: Brush, hasher: &mut H) {
    hash_f32(brush.r, hasher);
    hash_f32(brush.g, hasher);
    hash_f32(brush.b, hasher);
    hash_f32(brush.a, hasher);
}

fn hash_indent<H: Hasher>(indent: Indent, hasher: &mut H) {
    hash_f64(indent.amount, hasher);
    indent.first_line.hash(hasher);
    indent.each_line.hash(hasher);
    indent.hanging.hash(hasher);
}

fn hash_size<H: Hasher>(size: Size, hasher: &mut H) {
    hash_f64(size.width, hasher);
    hash_f64(size.height, hasher);
}

fn hash_option_f32<H: Hasher>(value: Option<f32>, hasher: &mut H) {
    match value {
        Some(value) => {
            true.hash(hasher);
            hash_f32(value, hasher);
        }
        None => false.hash(hasher),
    }
}

fn hash_f32<H: Hasher>(value: f32, hasher: &mut H) {
    value.to_bits().hash(hasher);
}

fn hash_option_f64<H: Hasher>(value: Option<f64>, hasher: &mut H) {
    match value {
        Some(value) => {
            1_u8.hash(hasher);
            hash_f64(value, hasher);
        }
        None => 0_u8.hash(hasher),
    }
}

fn hash_f64<H: Hasher>(value: f64, hasher: &mut H) {
    value.to_bits().hash(hasher);
}
