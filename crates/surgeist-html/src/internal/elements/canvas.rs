//! Native canvas state restricted to the source's `none` context mode.
//!
//! HTML 877d6146487f #the-canvas-element supplies these initial values and
//! #attr-canvas-width/#attr-canvas-height supply the exact bitmap dimensions.

use crate::internal::microsyntax::numeric::HtmlNonNegativeInteger;

pub(crate) const CANVAS_DEFAULT_WIDTH: u32 = 300;
pub(crate) const CANVAS_DEFAULT_HEIGHT: u32 = 150;
const MAX_REFLECTED_DIMENSION: u32 = i32::MAX as u32;
const MAX_REFLECTED_DIGITS: usize = 10;

/// Selects the content attribute and default for a Canvas IDL dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCanvasDimension {
    /// The reflected width, defaulting to 300.
    Width,
    /// The reflected height, defaulting to 150.
    Height,
}

impl HtmlCanvasDimension {
    const fn default_value(self) -> u32 {
        match self {
            Self::Width => CANVAS_DEFAULT_WIDTH,
            Self::Height => CANVAS_DEFAULT_HEIGHT,
        }
    }

    /// Applies the unsigned-long reflection cap and the default on overflow.
    pub const fn reflected_value(self, value: u32) -> u32 {
        if value <= MAX_REFLECTED_DIMENSION {
            value
        } else {
            self.default_value()
        }
    }
}

/// An exact bitmap dimension, in CSS pixels. This is not an unsigned-long IDL
/// reflection: that separate getter defaults above 2147483647, whereas the
/// bitmap's content-attribute rule has no machine-integer magnitude limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCanvasBitmapDimension<'a> {
    /// The defining attribute rule's default numeric value. An explicit equal
    /// value is normalized here too; this does not report attribute provenance.
    Default(u128),
    /// An actual successfully parsed non-default integer, including zero.
    Parsed(&'a HtmlNonNegativeInteger),
}

/// Actual native canvas state while its context mode remains `none`.
///
/// The [canvas rules](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-canvas-element)
/// give this state no bound context, an origin-clean flag of true and a
/// transparent-black bitmap whose dimensions follow the actual attributes.
/// This stores a bitmap description, never pixels, a renderer or a resource
/// handle. No method changes context mode or admits retained/unknown context
/// state. A consumer that installs a context must leave this type's domain.
///
/// Fresh defaults allocate nothing. Parsed inputs are moved without allocation;
/// each input inspects at most three canonical digit bytes to normalize an
/// explicit default. Attribute lookup, raw admission, parsing and subsequent
/// ownership/accounting of the exact integers belong to the caller. Cloning
/// this Rust value may allocate for its integers; it is not DOM cloning steps.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCanvasNoneState {
    width: Option<HtmlNonNegativeInteger>,
    height: Option<HtmlNonNegativeInteger>,
}

fn normalized_dimension(
    value: Option<HtmlNonNegativeInteger>,
    default: u32,
) -> Option<HtmlNonNegativeInteger> {
    value.filter(|value| {
        // Both normative defaults have three digits. The existing exact
        // projection therefore reads at most three bytes, never a large input.
        value.decimal_digits().len() > 3 || value.to_u128().ok() != Some(u128::from(default))
    })
}

impl HtmlCanvasNoneState {
    /// Native creation before any width/height attributes exist.
    pub const fn fresh() -> Self {
        Self {
            width: None,
            height: None,
        }
    }

    /// Native initialization after actual (including copied) attributes.
    ///
    /// Each `Some` must be the actual non-negative integer parsing result for
    /// that exact no-namespace attribute. `None` means actual absence or parse
    /// failure, never an unavailable value, wrong attribute or unapplied rule.
    /// Reuse HTML's canvas attribute rules or non-negative integer kernel;
    /// author-invalid prefix recovery can still provide an actual dimension.
    /// This does not copy a source bitmap or execute element cloning steps.
    pub fn from_parsed_attributes(
        width: Option<HtmlNonNegativeInteger>,
        height: Option<HtmlNonNegativeInteger>,
    ) -> Self {
        Self {
            width: normalized_dimension(width, CANVAS_DEFAULT_WIDTH),
            height: normalized_dimension(height, CANVAS_DEFAULT_HEIGHT),
        }
    }

    /// The bitmap width in CSS pixels: the default 300 unless an attribute supplied a different
    /// parsed value. Never the IDL `width` reflection.
    pub fn width(&self) -> HtmlCanvasBitmapDimension<'_> {
        self.width.as_ref().map_or(
            HtmlCanvasBitmapDimension::Default(u128::from(CANVAS_DEFAULT_WIDTH)),
            HtmlCanvasBitmapDimension::Parsed,
        )
    }

    /// The bitmap height in CSS pixels: the default 150 unless an attribute supplied a different
    /// parsed value. Never the IDL `height` reflection.
    pub fn height(&self) -> HtmlCanvasBitmapDimension<'_> {
        self.height.as_ref().map_or(
            HtmlCanvasBitmapDimension::Default(u128::from(CANVAS_DEFAULT_HEIGHT)),
            HtmlCanvasBitmapDimension::Parsed,
        )
    }

    /// Reads the IDL dimension from the exact, recovered bitmap dimension.
    ///
    /// Absence, failed recovery and values above 2147483647 return the
    /// dimension's default. Admit the selection and each digit of a bounded
    /// numeric projection. A longer canonical value falls back by length.
    pub fn reflected_dimension_with<E>(
        &self,
        dimension: HtmlCanvasDimension,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<u32, E> {
        step()?;
        let parsed = match dimension {
            HtmlCanvasDimension::Width => &self.width,
            HtmlCanvasDimension::Height => &self.height,
        };
        let default = dimension.default_value();
        let Some(parsed) = parsed else {
            return Ok(default);
        };
        let digits = parsed.decimal_digits().len();
        if digits > MAX_REFLECTED_DIGITS {
            return Ok(default);
        }
        for _ in 0..digits {
            step()?;
        }
        let value = parsed
            .to_u64()
            .ok()
            .and_then(|value| u32::try_from(value).ok());
        Ok(value.map_or(default, |value| dimension.reflected_value(value)))
    }

    /// Always `true`: this type exists only while the context mode is `none`.
    pub const fn context_is_none(&self) -> bool {
        true
    }

    /// Always `false`: no rendering context is bound while the context mode is `none`.
    pub const fn has_bound_context(&self) -> bool {
        false
    }

    /// Always `true`: the origin-clean flag starts true and nothing here can clear it.
    pub const fn origin_clean(&self) -> bool {
        true
    }

    /// Always `true`: the bitmap of a `none`-mode canvas is transparent black and never drawn on.
    pub const fn bitmap_is_transparent_black(&self) -> bool {
        true
    }

    /// Actual width setting, change, removal or redundant set in `none` mode.
    /// The bitmap follows the new parsed dimension; the context-mode table has
    /// no additional action. Input qualification is as in from_parsed_attributes.
    pub fn width_attribute_changed(&mut self, parsed: Option<HtmlNonNegativeInteger>) {
        self.width = normalized_dimension(parsed, CANVAS_DEFAULT_WIDTH);
    }

    /// Actual height setting, change, removal or redundant set in `none` mode.
    /// No other dimension, context, origin-clean or bitmap-content state changes.
    pub fn height_attribute_changed(&mut self, parsed: Option<HtmlNonNegativeInteger>) {
        self.height = normalized_dimension(parsed, CANVAS_DEFAULT_HEIGHT);
    }
}
