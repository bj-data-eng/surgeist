use std::{borrow::Cow, collections::HashMap, ops::Range as StdRange};

use parley::{
    AlignmentOptions, FontFamily, FontFeatures, FontVariations, GenericFamily, LayoutContext,
    StyleProperty,
};

use super::{
    Brush, FontCatalogueIdentity, FontCatalogueMode, Id, InlineBox, Key, Layout, Options, Range,
    Result, Source, Span, Stats, Style, ValidatedOptions, ValidatedSource, ValidatedStyle,
};

use crate::catalogue::Catalogue;

/// Shared font and layout system.
pub struct System {
    catalogue: Catalogue,
    layout_context: LayoutContext<Brush>,
    cache: HashMap<Key, Layout>,
    stats: Stats,
}

impl System {
    pub fn new(options: SystemOptions) -> Result<Self> {
        Ok(Self {
            catalogue: Catalogue::new(options.font_catalogue),
            layout_context: LayoutContext::new(),
            cache: HashMap::new(),
            stats: Stats::default(),
        })
    }

    pub fn builder(&mut self, text: impl Into<String>) -> Builder<'_> {
        Builder {
            system: self,
            source: Source::new(text),
            default_style: Style::default(),
            options: Options::default(),
        }
    }

    pub fn layout(&mut self, source: Source, style: Style, options: Options) -> Result<Layout> {
        let mut builder = Builder {
            system: self,
            source,
            default_style: style,
            options,
        };
        builder.build()
    }

    /// Registers every face in an owned font file, or leaves the system unchanged.
    ///
    /// Registrations are appended in request order. This does not replace an
    /// existing family or promise coverage of every requested character.
    pub fn register_fonts(&mut self, bytes: Vec<u8>) -> Result<()> {
        let candidate = self.catalogue.with_registration(bytes)?;
        self.install_catalogue(candidate);
        Ok(())
    }

    /// Rediscovers configured system fonts and replays registered font bytes.
    ///
    /// A failure preserves the catalogue, cache, and statistics. Historical
    /// layouts keep their actual selected font resources after a refresh.
    pub fn refresh_fonts(&mut self) -> Result<()> {
        let candidate = self.catalogue.refreshed()?;
        self.install_catalogue(candidate);
        self.stats.record_font_refresh();
        Ok(())
    }

    /// Returns the current registration and discovery publication identity.
    #[must_use]
    pub const fn font_catalogue_identity(&self) -> &FontCatalogueIdentity {
        self.catalogue.identity()
    }

    fn install_catalogue(&mut self, candidate: Catalogue) {
        let old_catalogue = std::mem::replace(&mut self.catalogue, candidate);
        let old_cache = std::mem::take(&mut self.cache);
        self.stats.record_invalidations(old_cache.len());
        drop(old_cache);
        drop(old_catalogue);
    }

    #[must_use]
    pub const fn stats(&self) -> Stats {
        self.stats
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new(SystemOptions::default()).expect("default text system should initialize")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SystemOptions {
    /// Selects system discovery or registered-only matching.
    pub font_catalogue: FontCatalogueMode,
}

/// Builds one layout.
pub struct Builder<'a> {
    system: &'a mut System,
    source: Source,
    default_style: Style,
    options: Options,
}

impl Builder<'_> {
    pub fn identity(&mut self, id: Id, revision: u64) -> &mut Self {
        self.source.set_identity(Some(id), revision);
        self
    }

    pub fn options(&mut self, options: Options) -> &mut Self {
        self.options = options;
        self
    }

    pub fn default_style(&mut self, style: Style) -> &mut Self {
        self.default_style = style;
        self
    }

    pub fn span(&mut self, range: Range, style: Style) -> &mut Self {
        self.source.spans.push(Span::new(range, style));
        self
    }

    pub fn inline_box(&mut self, box_: InlineBox) -> &mut Self {
        self.source.boxes.push(box_);
        self
    }

    pub fn build(&mut self) -> Result<Layout> {
        let validated_source = ValidatedSource::try_from(self.source.clone())?;
        let validated_options = ValidatedOptions::try_from(self.options)?;
        let default_style = ValidatedStyle::try_from(self.default_style.clone())?;
        let span_styles = validated_source
            .span_styles()
            .iter()
            .map(|span| (span.range(), span.style()))
            .collect::<Vec<_>>();

        let key = Key::from_validated(
            &validated_source,
            &default_style,
            validated_options,
            self.system.catalogue.identity(),
        );
        if let Some(layout) = self.system.cache.get(&key) {
            self.system.stats.record_layout_hit();
            return Ok(layout.clone());
        }

        let layout_source = validated_source.source().clone();
        let mut builder = self.system.layout_context.ranged_builder(
            self.system.catalogue.context_mut(),
            &layout_source.text,
            validated_options.authored().scale,
            validated_options.authored().quantize,
        );

        push_style_defaults(&mut builder, &default_style);
        for (range, style) in &span_styles {
            push_style_span(&mut builder, *range, style);
        }
        for box_ in &layout_source.boxes {
            builder.push_inline_box(parley::InlineBox {
                id: box_.id.as_u64(),
                kind: box_.kind.into(),
                index: box_.index,
                width: box_.size.width,
                height: box_.size.height,
            });
        }

        let mut layout = builder.build(&layout_source.text);
        if let Some((amount, indent_options)) = validated_options.parley_indent() {
            layout.set_text_indent(amount, indent_options);
        }
        layout.break_all_lines(validated_options.authored().width);
        layout.align(
            validated_options.authored().alignment.into(),
            AlignmentOptions::default(),
        );

        self.system.stats.record_layout_miss();

        let layout = Layout {
            inner: layout,
            source: layout_source,
            default_style: self.default_style.clone(),
            key: key.clone(),
        };
        self.system.cache.insert(key, layout.clone());
        Ok(layout)
    }
}

fn push_style_defaults(builder: &mut parley::RangedBuilder<'_, Brush>, style: &ValidatedStyle) {
    push_style_properties(builder, style, None)
}

fn push_style_span(
    builder: &mut parley::RangedBuilder<'_, Brush>,
    range: Range,
    style: &ValidatedStyle,
) {
    push_style_properties(builder, style, Some(range.into()))
}

fn push_style_properties(
    builder: &mut parley::RangedBuilder<'_, Brush>,
    style: &ValidatedStyle,
    range: Option<StdRange<usize>>,
) {
    let mut push = |property: StyleProperty<'static, Brush>| {
        if let Some(range) = range.clone() {
            builder.push(property, range);
        } else {
            builder.push_default(property);
        }
    };
    let authored = style.authored();

    let family = style
        .parley_font_family()
        .map_or(FontFamily::from(GenericFamily::SansSerif), |families| {
            FontFamily::List(Cow::Owned(families.to_vec()))
        });

    push(StyleProperty::FontFamily(family));
    push(StyleProperty::FontSize(authored.size));
    push(StyleProperty::FontWeight(authored.font.weight.into()));
    push(StyleProperty::FontWidth(authored.font.width.into()));
    push(StyleProperty::FontStyle(authored.font.slant.into()));
    push(StyleProperty::FontFeatures(
        style
            .parley_font_features()
            .map_or_else(FontFeatures::empty, |features| {
                FontFeatures::Source(Cow::Owned(features.to_owned()))
            }),
    ));
    push(StyleProperty::FontVariations(
        style
            .parley_font_variations()
            .map_or_else(FontVariations::empty, |variations| {
                FontVariations::Source(Cow::Owned(variations.to_owned()))
            }),
    ));
    push(StyleProperty::Locale(style.parley_locale()));
    push(StyleProperty::Brush(authored.brush));
    push(StyleProperty::LineHeight(authored.line_height.into()));
    push(StyleProperty::LetterSpacing(authored.letter_spacing));
    push(StyleProperty::WordSpacing(authored.word_spacing));
    push(StyleProperty::WordBreak(authored.word_break.into()));
    push(StyleProperty::OverflowWrap(authored.overflow_wrap.into()));
    push(StyleProperty::TextWrapMode(authored.wrap.into()));
    push(StyleProperty::Underline(authored.underline.enabled()));
    push(StyleProperty::UnderlineOffset(
        authored.underline.offset().to_parley(),
    ));
    push(StyleProperty::UnderlineSize(
        authored.underline.thickness().to_parley(),
    ));
    push(StyleProperty::UnderlineBrush(
        authored.underline.brush().to_parley(),
    ));
    push(StyleProperty::Strikethrough(
        authored.strikethrough.enabled(),
    ));
    push(StyleProperty::StrikethroughOffset(
        authored.strikethrough.offset().to_parley(),
    ));
    push(StyleProperty::StrikethroughSize(
        authored.strikethrough.thickness().to_parley(),
    ));
    push(StyleProperty::StrikethroughBrush(
        authored.strikethrough.brush().to_parley(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_registration_refuses_before_font_validation_or_publication() {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .expect("registered-only system initializes");
        system.catalogue.exhaust_generation_for_test();
        let identity = system.font_catalogue_identity().clone();
        let stats = system.stats();
        let error = system
            .register_fonts(b"invalid font".to_vec())
            .expect_err("issuance headroom is checked before font parsing");
        assert_eq!(error.code, crate::ErrorCode::FontGenerationExhausted);
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::FontGenerationExhausted {
                generation: u64::MAX
            })
        );
        assert_eq!(system.font_catalogue_identity(), &identity);
        assert_eq!(system.stats(), stats);
        assert!(system.cache.is_empty());
    }

    #[test]
    fn exhausted_font_refresh_preserves_cached_layout_stats_and_retained_fonts() {
        let mut system = System::default();
        system.catalogue.exhaust_generation_for_test();
        let source = Source::new("retained font refresh");
        let layout = system
            .layout(source.clone(), Style::default(), Options::default())
            .expect("the current catalogue builds the exhaustion fixture");
        let key = layout.key();
        assert_eq!(
            key.font_generation(),
            system.font_catalogue_identity().generation()
        );
        assert_eq!(key.font_generation().get(), u64::MAX);
        assert_eq!(system.cache.len(), 1);

        let lines = layout.lines();
        let metrics = layout.metrics().size();
        let retained_font = layout
            .glyph_runs()
            .into_iter()
            .find_map(|run| run.font().data().cloned())
            .expect("the real layout retains a resolved font");
        let font_bytes = retained_font.bytes().to_vec();
        let font_index = retained_font.index();
        assert!(!font_bytes.is_empty());
        let stats = system.stats();

        let refresh = system.refresh_fonts();
        assert!(
            refresh.is_err(),
            "an exhausted font generation must refuse refresh rather than reuse its identity: {refresh:?}"
        );
        assert_eq!(
            system.font_catalogue_identity().generation(),
            key.font_generation()
        );
        assert_eq!(system.stats(), stats);
        assert_eq!(system.cache.len(), 1);
        let cached = system
            .cache
            .get(&key)
            .expect("the refused refresh preserves the actual cached entry");
        assert_eq!(cached.lines(), lines);
        assert_eq!(cached.metrics().size(), metrics);
        assert_eq!(cached.source().text(), source.text());

        let reused = system
            .layout(source, Style::default(), Options::default())
            .expect("the unchanged catalogue still serves the same layout");
        assert_eq!(reused.key(), key);
        assert_eq!(reused.lines(), lines);
        assert_eq!(system.stats().layout_hits(), stats.layout_hits() + 1);
        assert_eq!(system.stats().layout_misses(), stats.layout_misses());
        assert_eq!(system.stats().font_refreshes(), stats.font_refreshes());
        assert_eq!(system.stats().invalidations(), stats.invalidations());

        drop(reused);
        drop(system);
        assert_eq!(layout.key(), key);
        assert_eq!(layout.lines(), lines);
        assert_eq!(layout.metrics().size(), metrics);
        drop(layout);
        assert_eq!(retained_font.index(), font_index);
        assert_eq!(retained_font.bytes(), font_bytes);
    }
}
