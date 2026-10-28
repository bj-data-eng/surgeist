use std::{collections::HashMap, sync::Arc};

use super::{
    FontCatalogueIdentity, FontCatalogueMode, FontFace, Id, InlineBox, Key, Layout, Options, Range,
    Result, Source, Span, Stats, Style, Text, ValidatedOptions, ValidatedSource, ValidatedStyle,
};
use crate::{catalogue::Catalogue, engine, projection::SourceProjection};

/// Shared font and layout system.
pub struct System {
    catalogue: Catalogue,
    shape_context: engine::shape::ShapeContext,
    cache: HashMap<Key, Layout>,
    stats: Stats,
}

impl System {
    #[cfg(test)]
    pub(crate) fn shape_cache_hits(&self) -> [usize; 3] {
        self.shape_context.cache_hits()
    }

    pub fn new(options: SystemOptions) -> Result<Self> {
        Ok(Self {
            catalogue: Catalogue::new(options.font_catalogue),
            shape_context: engine::shape::ShapeContext::default(),
            cache: HashMap::new(),
            stats: Stats::default(),
        })
    }

    pub fn builder(&mut self, text: impl Into<Text>) -> Builder<'_> {
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

    /// Shapes and places an identity-processing standalone source analysis.
    /// Explicit paragraph captures use `prepare` and the composed line protocol.
    /// A cache result from an independently issued equal analysis cannot substitute.
    pub fn layout_analyzed(
        &mut self,
        analyzed: &crate::AnalyzedSource,
        options: Options,
    ) -> Result<Layout> {
        if analyzed.inner.paragraph.base_direction.is_some() {
            return Err(crate::protocol::protocol_error(
                crate::ProtocolError::UnsupportedStandaloneAnalysis,
            ));
        }
        let options = ValidatedOptions::try_from(options)?;
        let key = Key::from_analyzed(&analyzed.inner, options, self.catalogue.identity());
        if let Some(layout) = self.cache.get(&key)
            && Arc::ptr_eq(&layout.inner.prepared.analyzed, &analyzed.inner)
            && layout.matches_request(
                analyzed.source(),
                analyzed.style(),
                options.authored(),
                self.catalogue.identity(),
            )
        {
            self.stats.record_layout_hit();
            return Ok(layout.clone());
        }
        self.place_analyzed(Arc::clone(&analyzed.inner), key, options)
    }

    /// Prepares the exact immutable analysis, retaining its actual source and
    /// selected resources without choosing, finalizing, or placing a line.
    pub fn prepare(
        &mut self,
        analyzed: &crate::AnalyzedSource,
        options: crate::PreparationOptions,
    ) -> Result<crate::PreparedText> {
        let inner = engine::prepare(
            &mut self.catalogue,
            Arc::clone(&analyzed.inner),
            options,
            engine::PreparationProfile::Native,
            &mut self.shape_context,
        )?;
        Ok(crate::PreparedText { inner })
    }

    fn place_analyzed(
        &mut self,
        analyzed: Arc<engine::AnalyzedData>,
        key: Key,
        options: ValidatedOptions,
    ) -> Result<Layout> {
        let prepared = engine::prepare(
            &mut self.catalogue,
            analyzed,
            crate::PreparationOptions::try_new(
                options.authored().scale,
                crate::TabPreparation::FontAdvance,
            )?,
            engine::PreparationProfile::Standalone,
            &mut self.shape_context,
        )?;
        let placed = engine::layout::layout(prepared, options, &mut self.shape_context)?;
        let layout = Layout::from_placed(
            placed,
            key.clone(),
            crate::layout::LayoutBasis::Standalone(options.authored()),
        );
        layout.qualify_decorations()?;
        self.cache.insert(key, layout.clone());
        self.stats.record_layout_miss();
        Ok(layout)
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

    /// Borrows only actual registered faces, in registration then file-face order.
    /// This performs no system discovery, family matching, fallback or glyph layout.
    pub fn registered_font_faces(&self) -> impl Iterator<Item = FontFace<'_>> {
        self.catalogue.registered_faces()
    }

    fn install_catalogue(&mut self, candidate: Catalogue) {
        let old_catalogue = std::mem::replace(&mut self.catalogue, candidate);
        self.stats.record_invalidations(self.cache.len());
        self.cache = HashMap::new();
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
        SourceProjection::admit(validated_source.source())?;
        let key = Key::from_validated(
            &validated_source,
            &default_style,
            validated_options,
            self.system.catalogue.identity(),
        );
        if let Some(layout) = self.system.cache.get(&key)
            && layout.matches_request(
                validated_source.source(),
                &self.default_style,
                self.options,
                self.system.catalogue.identity(),
            )
        {
            self.system.stats.record_layout_hit();
            return Ok(layout.clone());
        }

        let analyzed = engine::analyze(
            validated_source,
            default_style,
            crate::AnalysisTypography::HorizontalComposition,
        )?;
        self.system.place_analyzed(analyzed, key, validated_options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decoration_system() -> System {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        system
    }

    fn decoration_style(offset: f64, thickness: f64, strike: bool) -> Style {
        let decoration = crate::Decoration::solid()
            .with_offset(crate::DecorationOffset::try_absolute(offset).unwrap())
            .with_thickness(crate::DecorationThickness::try_absolute(thickness).unwrap());
        Style {
            font: crate::Font::new().family("Ahem"),
            size: 20.0,
            underline: if strike {
                crate::Decoration::none()
            } else {
                decoration
            },
            strikethrough: if strike {
                decoration
            } else {
                crate::Decoration::none()
            },
            ..Style::default()
        }
    }

    fn assert_decoration_refusal_preserves_publication(source: &str, strike: bool) {
        let mut system = decoration_system();
        let options = Options {
            quantize: false,
            ..Options::default()
        };
        let prior = system
            .layout(
                Source::new("A"),
                decoration_style(2.0, 2.0, strike),
                options,
            )
            .unwrap();
        let prior_key = prior.key();
        let prior_geometry = prior.decorations();
        let stats = system.stats();
        let cache_len = system.cache.len();
        let result = system.layout(
            Source::new(source),
            // Strike top overflows; underline's bottom endpoint overflows.
            decoration_style(if strike { f64::MAX } else { -f64::MAX }, f64::MAX, strike),
            options,
        );
        let error = match result {
            Err(error) => error,
            Ok(layout) => panic!(
                "unrepresentable enabled decoration was published: {:?}",
                layout.decorations()
            ),
        };
        assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
        assert!(matches!(
            error.detail(),
            Some(crate::ErrorDetail::InvalidTextNumeric {
                category: crate::TextNumericError::NonFinite,
                ..
            })
        ));
        assert_eq!(system.stats(), stats);
        assert_eq!(system.cache.len(), cache_len);
        assert!(system.cache.contains_key(&prior_key));
        assert_eq!(prior.decorations(), prior_geometry);
    }

    #[test]
    fn decoration_geometry_strike_overflow_refuses_before_cache_publication() {
        assert_decoration_refusal_preserves_publication("A", true);
    }

    #[test]
    fn decoration_geometry_underline_endpoint_overflow_refuses_before_cache_publication() {
        assert_decoration_refusal_preserves_publication("A", false);
    }

    #[test]
    fn decoration_geometry_metric_only_overflow_refuses_before_cache_publication() {
        assert_decoration_refusal_preserves_publication("", true);
    }

    #[test]
    fn decoration_geometry_disabled_extremes_emit_no_rectangles() {
        let mut system = decoration_system();
        let disabled = crate::Decoration::none()
            .with_offset(crate::DecorationOffset::try_absolute(f64::MAX).unwrap())
            .with_thickness(crate::DecorationThickness::try_absolute(f64::MAX).unwrap());
        let layout = system
            .layout(
                Source::new("A"),
                Style {
                    underline: disabled,
                    strikethrough: disabled,
                    ..decoration_style(2.0, 2.0, true)
                },
                Options::default(),
            )
            .unwrap();
        assert_eq!(layout.decoration_views().count(), 0);
    }

    #[test]
    fn decoration_geometry_finite_extremes_preserve_literal_bounds() {
        let mut system = decoration_system();
        let options = Options {
            quantize: false,
            ..Options::default()
        };
        for source in ["A", ""] {
            for strike in [true, false] {
                let quarter = f64::MAX / 4.0;
                let layout = system
                    .layout(
                        Source::new(source),
                        decoration_style(quarter, quarter, strike),
                        options,
                    )
                    .unwrap();
                let decorations = layout.decorations();
                assert_eq!(decorations.len(), 1);
                let rect = decorations[0].rect();
                assert_eq!(rect.origin.x, 0.0);
                assert_eq!(
                    rect.origin.y,
                    if strike {
                        -quarter - quarter / 2.0
                    } else {
                        -quarter
                    }
                );
                assert_eq!(rect.size.width, if source.is_empty() { 0.0 } else { 20.0 });
                assert_eq!(rect.size.height, quarter);
                assert!((rect.origin.y + rect.size.height).is_finite());
            }
        }
    }

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
            .next()
            .map(|run| run.font().clone())
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
