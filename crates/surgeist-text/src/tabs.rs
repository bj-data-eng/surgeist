//! Captured CSS tab values and resource demand.
use crate::{ParagraphBaseDirection, Result, Style};

/// Finite, nonnegative CSS `tab-size`, kept symbolic until preparation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabSize {
    pub(crate) value: f64,
    pub(crate) spaces: bool,
}
impl TabSize {
    pub fn spaces(number: f64) -> Result<Self> {
        Self::new(number, true)
    }
    pub fn length(absolute: f64) -> Result<Self> {
        Self::new(absolute, false)
    }
    fn new(value: f64, spaces: bool) -> Result<Self> {
        if !value.is_finite() || value < 0.0 {
            return Err(crate::error::text_numeric_error(
                "tab size",
                crate::TextNumericPhase::Preparation,
                if value.is_finite() {
                    crate::TextNumericError::OutOfRange
                } else {
                    crate::TextNumericError::NonFinite
                },
                value,
            ));
        }
        Ok(Self {
            value: if value == 0.0 { 0.0 } else { value },
            spaces,
        })
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }
    #[must_use]
    pub const fn is_spaces(self) -> bool {
        self.spaces
    }
}
impl Default for TabSize {
    fn default() -> Self {
        Self {
            value: 8.0,
            spaces: true,
        }
    }
}

/// Actual nearest block style and its used direction, independent of paragraph bidi.
#[derive(Clone, Debug, PartialEq)]
pub struct ParagraphTabReference {
    style: Style,
    direction: ParagraphBaseDirection,
}
impl ParagraphTabReference {
    pub fn new(style: Style, direction: ParagraphBaseDirection) -> Result<Self> {
        crate::ValidatedStyle::for_processed(style.clone())?;
        Ok(Self { style, direction })
    }
    #[must_use]
    pub fn style(&self) -> &Style {
        &self.style
    }
    #[must_use]
    pub const fn direction(&self) -> ParagraphBaseDirection {
        self.direction
    }
}

/// Which real tab operation preparation must make ready for every selectable tab.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TabPreparation {
    #[default]
    FontAdvance,
    CssStops,
}

/// One actual style's resolved operation; metric resources remain strongly owned.
pub(crate) struct ResolvedTab {
    pub(crate) period: f64,
    pub(crate) threshold: f64,
    pub(crate) font: Option<crate::engine::layout::data::PreparedRun>,
    pub(crate) _block_font: Option<crate::engine::layout::data::PreparedRun>,
}
fn capability(field: crate::ProtocolContextField) -> crate::Error {
    crate::protocol::protocol_error(crate::ProtocolError::UnsupportedContext { field })
}
fn multiply(field: &'static str, a: f64, b: f64) -> Result<f64> {
    let value = a * b;
    if !value.is_finite() || (a != 0.0 && b != 0.0 && value == 0.0) {
        return Err(crate::error::text_numeric_error(
            field,
            crate::TextNumericPhase::Preparation,
            if value.is_finite() {
                if a > 0.0 && b > 0.0 {
                    crate::TextNumericError::PositiveLost
                } else {
                    crate::TextNumericError::NonZeroLost
                }
            } else {
                crate::TextNumericError::NonFinite
            },
            value,
        ));
    }
    Ok(value)
}
pub(crate) fn prepare(
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: &crate::engine::AnalyzedData,
    styles: &[crate::engine::resolve::ResolvedStyleRun],
    alternatives: &[crate::protocol::prepared::PreparedAlternativeShape],
    options: crate::PreparationOptions,
    scratch: &mut crate::engine::shape::ShapeContext,
) -> Result<Vec<Option<ResolvedTab>>> {
    use std::collections::BTreeSet;
    let mut output: Vec<_> = (0..styles.len()).map(|_| None).collect();
    if options.tabs() != TabPreparation::CssStops {
        return Ok(output);
    }
    let mut needed = BTreeSet::new();
    for (projection, analysis) in std::iter::once((&analyzed.projection, &analyzed.analysis)).chain(
        alternatives
            .iter()
            .map(|alt| (&alt.projection, &alt.analysis)),
    ) {
        for (i, atom) in projection.atoms.iter().enumerate() {
            if atom.code_point == 9 {
                needed.insert(analysis.infos[i].1);
            }
        }
    }
    let catalogue_identity = catalogue.identity().clone();
    for index in needed {
        let size = analyzed.styles[index].style.authored().tab_size;
        let mut metric = ResolvedTab {
            period: 0.0,
            threshold: 0.0,
            font: None,
            _block_font: None,
        };
        if size.value == 0.0 {
            output[index] = Some(metric);
            continue;
        }
        let reference = analyzed
            .paragraph
            .tab_reference
            .as_ref()
            .ok_or_else(|| capability(crate::ProtocolContextField::TabReference))?;
        if size.spaces {
            let block = crate::ValidatedStyle::for_processed(reference.style().clone())?;
            let block_style = crate::engine::resolve::resolve_default(
                catalogue.collection_mut(),
                &block,
                options.scale(),
            )?;
            let block_styles = [block_style];
            let input = crate::engine::ShapingInput {
                projection: &analyzed.projection,
                analysis: &analyzed.analysis,
                styles: &block_styles,
                boxes: &[],
                font_catalogue: &catalogue_identity,
                paragraph: &analyzed.paragraph,
                css_tabs: true,
            };
            let (advance, font) = crate::engine::shape::measure_character(
                catalogue.query(),
                input,
                0,
                ' ',
                reference.direction(),
                scratch,
            )?
            .ok_or_else(|| capability(crate::ProtocolContextField::TabMetrics))?;
            let adjusted = crate::engine::layout::add(
                "tab block letter spacing",
                crate::TextNumericPhase::Preparation,
                advance,
                block_styles[0].style.letter_spacing,
            )?;
            let adjusted = crate::engine::layout::add(
                "tab block word spacing",
                crate::TextNumericPhase::Preparation,
                adjusted,
                block_styles[0].style.word_spacing,
            )?;
            metric.period = multiply("tab interval", size.value, adjusted)?;
            metric._block_font = Some(font);
        } else {
            metric.period = multiply("tab length", size.value, f64::from(options.scale()))?;
        }
        if metric.period < 0.0 {
            return Err(capability(crate::ProtocolContextField::NegativeTabInterval));
        }
        if metric.period != 0.0 {
            let input = crate::engine::ShapingInput {
                projection: &analyzed.projection,
                analysis: &analyzed.analysis,
                styles,
                boxes: &[],
                font_catalogue: &catalogue_identity,
                paragraph: &analyzed.paragraph,
                css_tabs: true,
            };
            let measured = crate::engine::shape::measure_character(
                catalogue.query(),
                input,
                index,
                '0',
                crate::ParagraphBaseDirection::LeftToRight,
                scratch,
            )?;
            let ch = if let Some((advance, font)) = measured {
                metric.font = Some(font);
                advance
            } else {
                multiply(
                    "tab ch fallback",
                    f64::from(styles[index].style.font_size),
                    0.5,
                )?
            };
            if ch < 0.0 {
                return Err(capability(crate::ProtocolContextField::TabMetrics));
            }
            metric.threshold = multiply("tab half ch", ch, 0.5)?;
        }
        output[index] = Some(metric);
    }
    Ok(output)
}
