//! Captured CSS tab values and resource demand.
use crate::{ParagraphBaseDirection, Result, Style, TextNumericError};
pub(crate) mod cell;

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
    /// Actual signed spacing-derived interval; nonzero grids use its magnitude.
    pub(crate) period: f64,
    pub(crate) threshold: f64,
    pub(crate) font: Option<crate::engine::layout::data::PreparedRun>,
    pub(crate) _block_font: Option<crate::engine::layout::data::PreparedRun>,
}
fn capability(field: crate::ProtocolContextField) -> crate::Error {
    crate::protocol::protocol_error(crate::ProtocolError::UnsupportedContext { field })
}
pub(crate) fn multiply(field: &'static str, a: f64, b: f64) -> Result<f64> {
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
    alternatives: &[crate::protocol::prepared::PreparedIndicationFonts],
    options: crate::PreparationOptions,
    scratch: &mut crate::engine::shape::ShapeContext,
    ledger: &mut crate::engine::scope_alignment::accounting::Ledger,
) -> Result<Vec<Option<ResolvedTab>>> {
    use std::collections::BTreeSet;
    let mut output: Vec<_> = (0..styles.len()).map(|_| None).collect();
    if options.tabs() != TabPreparation::CssStops {
        return Ok(output);
    }
    let mut needed = BTreeSet::new();
    for (projection, analysis, original) in
        std::iter::once((&analyzed.projection, &analyzed.analysis, true)).chain(
            alternatives
                .iter()
                .map(|alt| (&alt.projection, &alt.analysis, false)),
        )
    {
        for (i, atom) in projection.atoms.iter().enumerate() {
            if atom.code_point == 9 {
                if original && cell::owns_atom(analyzed, i, ledger)? {
                    continue;
                }
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
                formatting_breaks: &[],
                scope_custody: None,
                characters: None,
                separator_geometry: false,
                profile: crate::engine::upright::original_profile(
                    options.orientation(),
                    &[],
                    analyzed.typography,
                    &analyzed.styles,
                ),
                baseline_reference: options.resolved_baseline_reference(),
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
                crate::engine::shape::CharacterProbe {
                    input,
                    style_index: 0,
                    ch: ' ',
                    direction: reference.direction(),
                    content: crate::UprightContent::TabMeasurement {
                        role: crate::UprightTabMeasurement::SpaceInterval,
                        owner: crate::TextStyleOwner::TabReference,
                    },
                },
                scratch,
                &mut None,
                &[],
            )?
            .ok_or_else(|| capability(crate::ProtocolContextField::TabMetrics))?;
            // The actual hypothetical SPACE operation already applied the
            // shared intrinsic-zero gate and authored spacing exactly once.
            metric.period = multiply("tab interval", size.value, advance)?;
            metric._block_font = Some(font);
        } else {
            metric.period = multiply("tab length", size.value, f64::from(options.scale()))?;
        }
        if metric.period != 0.0 {
            let input = crate::engine::ShapingInput {
                formatting_breaks: &[],
                scope_custody: None,
                characters: None,
                separator_geometry: false,
                profile: crate::engine::upright::original_profile(
                    options.orientation(),
                    &[],
                    analyzed.typography,
                    &analyzed.styles,
                ),
                baseline_reference: options.resolved_baseline_reference(),
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
                crate::engine::shape::CharacterProbe {
                    input,
                    style_index: index,
                    ch: '0',
                    direction: crate::ParagraphBaseDirection::LeftToRight,
                    content: crate::UprightContent::TabMeasurement {
                        role: crate::UprightTabMeasurement::ZeroThreshold,
                        owner: crate::engine::upright::style_owner(analyzed.styles[index].owner),
                    },
                },
                scratch,
                &mut None,
                &[],
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

/// One checked consumed tab-grid evaluator, shared by outer and Cell layout.
pub(crate) struct Grid {
    phase: crate::TextNumericPhase,
}
impl Grid {
    pub(crate) fn new(phase: crate::TextNumericPhase) -> Self {
        Self { phase }
    }
    pub(crate) fn stop(
        &self,
        origin: f64,
        pen: f64,
        sign: f64,
        period: f64,
        threshold: f64,
    ) -> Result<f64> {
        if period == 0.0 {
            return Ok(0.0);
        }
        let period = period.abs();
        let mut distance = self.distance(origin, pen, sign, period)?;
        if distance < threshold {
            distance = self.add("tab threshold bump", distance, period)?;
        }
        Ok(distance)
    }
    pub(crate) fn add(&self, field: &'static str, base: f64, delta: f64) -> Result<f64> {
        let actual = crate::engine::layout::add(field, self.phase, base, delta)?;
        if base != 0.0 && actual == delta {
            return Err(crate::error::text_numeric_error(
                field,
                self.phase,
                TextNumericError::NonZeroLost,
                base,
            ));
        }
        Ok(actual)
    }

    fn distance(&self, origin: f64, pen: f64, sign: f64, period: f64) -> Result<f64> {
        const FIELD: &str = "tab grid displacement";
        // Reduce each coordinate before subtracting: a large whole-period offset
        // must not round away a small, representable distance to the next stop.
        // Keep signed remainders; normalizing a tiny negative remainder to a huge
        // positive period first could itself erase the required contribution.
        let origin = (sign * origin) % period;
        let pen = (sign * pen) % period;
        let cyclic_sum = |a: f64, b: f64, reverse: bool| -> Result<f64> {
            let (large, small) = if a >= b { (a, b) } else { (b, a) };
            if large < period * 0.5 {
                let sum = self.add(FIELD, large, small)?;
                return if reverse && sum != 0.0 {
                    self.add(FIELD, period, -sum)
                } else {
                    Ok(sum)
                };
            }
            // In the upper half, period-large is an exact binary64 subtraction.
            // Compare against that gap before adding, avoiding both overflow and
            // rounding a value just beyond the period back onto the period.
            let gap = self.add(FIELD, period, -large)?;
            let difference = self.add(FIELD, small, -gap)?;
            // Compute the requested direction directly. Its opposite may be an
            // unrepresentable period-small value even when this distance is exact.
            let directed = if reverse { -difference } else { difference };
            if directed < 0.0 {
                self.add(FIELD, period, directed)
            } else {
                Ok(directed)
            }
        };
        let forward_difference = |following: f64, preceding: f64| -> Result<f64> {
            if following >= preceding {
                return self.add(FIELD, following, -preceding);
            }
            let distance = if preceding >= period * 0.5 {
                let gap = self.add(FIELD, period, -preceding)?;
                self.add(FIELD, gap, following)?
            } else {
                let backwards = self.add(FIELD, preceding, -following)?;
                self.add(FIELD, period, -backwards)?
            };
            if distance == period {
                return Err(crate::error::text_numeric_error(
                    FIELD,
                    self.phase,
                    TextNumericError::NonZeroLost,
                    preceding - following,
                ));
            }
            Ok(distance)
        };
        let distance = if origin >= 0.0 && pen < 0.0 {
            cyclic_sum(origin, -pen, false)?
        } else if origin < 0.0 && pen >= 0.0 {
            cyclic_sum(-origin, pen, true)?
        } else if origin < 0.0 {
            forward_difference(-pen, -origin)?
        } else {
            forward_difference(origin, pen)?
        };
        Ok(if distance == 0.0 { period } else { distance })
    }
}
