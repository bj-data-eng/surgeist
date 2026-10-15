//! Native canvas state restricted to the source's `none` context mode.
//!
//! HTML 877d6146487f #the-canvas-element supplies these initial values and
//! #attr-canvas-width/#attr-canvas-height supply the exact bitmap dimensions.

use crate::HtmlNonNegativeInteger;

pub(crate) const CANVAS_DEFAULT_WIDTH: u128 = 300;
pub(crate) const CANVAS_DEFAULT_HEIGHT: u128 = 150;

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
    default: u128,
) -> Option<HtmlNonNegativeInteger> {
    value.filter(|value| {
        // Both normative defaults have three digits. The existing exact
        // projection therefore reads at most three bytes, never a large input.
        value.decimal_digits().len() > 3 || value.to_u128().ok() != Some(default)
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

    pub fn width(&self) -> HtmlCanvasBitmapDimension<'_> {
        self.width.as_ref().map_or(
            HtmlCanvasBitmapDimension::Default(CANVAS_DEFAULT_WIDTH),
            HtmlCanvasBitmapDimension::Parsed,
        )
    }

    pub fn height(&self) -> HtmlCanvasBitmapDimension<'_> {
        self.height.as_ref().map_or(
            HtmlCanvasBitmapDimension::Default(CANVAS_DEFAULT_HEIGHT),
            HtmlCanvasBitmapDimension::Parsed,
        )
    }

    pub const fn context_is_none(&self) -> bool {
        true
    }

    pub const fn has_bound_context(&self) -> bool {
        false
    }

    pub const fn origin_clean(&self) -> bool {
        true
    }

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
