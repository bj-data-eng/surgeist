//! Version-2 original-source inputs. No browser geometry enters this model.

use serde::{Deserialize, Serialize};

use super::{Context, MeasurementError, nonnegative};

use crate::pinned_font;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Paragraph {
    pub font: Font,
    pub direction: Direction,
    pub writing_mode: WritingMode,
    pub base_style: TextStyle,
    pub origins: Vec<Origin>,
    pub connections: Vec<Connection>,
    pub atomics: Vec<Atomic>,
    pub controls: Vec<Control>,
    pub order: Vec<Occurrence>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Font {
    id: String,
    sha256: String,
    face_index: u32,
    family: String,
    style: String,
    weight: u16,
    stretch: String,
    features: String,
    variations: String,
    synthesis: String,
    loaded: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Direction {
    Ltr,
    Rtl,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum WritingMode {
    HorizontalTb,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Origin {
    pub child: usize,
    pub units: Vec<u16>,
    pub start: usize,
    pub end: usize,
    pub style: TextStyle,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TextStyle {
    font_size: f64,
    line_height: f64,
    whitespace: Whitespace,
    wrap: Wrap,
    letter_spacing: f64,
    word_spacing: f64,
    color: [f64; 4],
    locale: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Whitespace {
    Collapse,
    Preserve,
    PreserveSpaces,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Wrap {
    Word,
    None,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Connection {
    pub preceding: usize,
    pub following: usize,
    pub kind: ConnectionKind,
    pub preceding_wrap: Wrap,
    pub following_wrap: Wrap,
    pub ancestor_wrap: Wrap,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ConnectionKind {
    Continuous,
    Barrier,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Atomic {
    pub child: usize,
    pub at: usize,
    pub kind: AtomicKind,
    pub width: f64,
    pub height: f64,
    pub preceding_wrap: Wrap,
    pub following_wrap: Wrap,
    pub ancestor_wrap: Wrap,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum AtomicKind {
    InFlow,
    OutOfFlow,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Control {
    pub child: usize,
    pub at: usize,
    pub kind: ControlKind,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ControlKind {
    ForcedBreak,
    InlineBoundaryStart,
    InlineBoundaryEnd,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Occurrence {
    pub kind: OccurrenceKind,
    pub index: usize,
    pub affinity: Affinity,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum OccurrenceKind {
    Origin,
    Atomic,
    Control,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Affinity {
    Before,
    After,
}

impl Paragraph {
    pub(super) fn validate(
        &self,
        context: &Context<'_>,
        child_count: usize,
    ) -> Result<String, MeasurementError> {
        let font = &self.font;
        context.ensure(
            font.id == pinned_font::ID
                && font.sha256 == pinned_font::SHA256
                && font.face_index == 0
                && font.family == "SurgeistParityAhem"
                && font.style == "normal"
                && font.weight == 400
                && font.stretch == "normal"
                && font.features == "normal"
                && font.variations == "normal"
                && font.synthesis == "none"
                && font.loaded,
            "preparedParagraph.font",
            "the exact loaded pinned Ahem face and executable descriptor profile are required",
        )?;
        self.base_style.validate(context)?;
        let mut child_roles = std::collections::BTreeSet::new();
        let mut source_end = 0;
        for (index, origin) in self.origins.iter().enumerate() {
            context.ensure(
                origin.child < child_count && child_roles.insert(origin.child),
                format!("preparedParagraph.origins[{index}].child"),
                "each source occurrence binds one distinct actual child",
            )?;
            context.ensure(
                origin.start == source_end
                    && origin.end.checked_sub(origin.start) == Some(origin.units.len()),
                format!("preparedParagraph.origins[{index}]"),
                "original UTF16 ranges must exactly partition the captured source units",
            )?;
            // This first resource profile is scalar-only. The numeric encoding is
            // nevertheless lossless: no replacement or normalization is performed.
            context.ensure(pinned_font::covers(&origin.units),
                format!("preparedParagraph.origins[{index}].units"), "source units require actual pinned nominal coverage or an explicitly glyph-free source control")?;
            let style = &origin.style;
            style.validate(context)?;
            nonnegative(style.font_size, context, "preparedParagraph.fontSize")?;
            nonnegative(style.line_height, context, "preparedParagraph.lineHeight")?;
            super::finite(
                style.letter_spacing,
                context,
                "preparedParagraph.letterSpacing",
            )?;
            super::finite(style.word_spacing, context, "preparedParagraph.wordSpacing")?;
            source_end = origin.end;
        }
        for (index, atomic) in self.atomics.iter().enumerate() {
            context.ensure(
                atomic.child < child_count
                    && child_roles.insert(atomic.child)
                    && atomic.at <= source_end,
                format!("preparedParagraph.atomics[{index}]"),
                "atomic occurrences need distinct real children and an original anchor",
            )?;
            nonnegative(atomic.width, context, "preparedParagraph.atomic.width")?;
            nonnegative(atomic.height, context, "preparedParagraph.atomic.height")?;
        }
        for (index, control) in self.controls.iter().enumerate() {
            context.ensure(
                control.child < child_count
                    && child_roles.insert(control.child)
                    && control.at <= source_end,
                format!("preparedParagraph.controls[{index}]"),
                "controls need distinct actual children and an original anchor",
            )?;
        }
        context.ensure(
            child_roles.len() == child_count,
            "preparedParagraph",
            "the source capture must cover every actual direct inline child",
        )?;
        context.ensure(
            self.connections.len() == self.origins.len().saturating_sub(1),
            "preparedParagraph.connections",
            "each pair of incident original text occurrences needs an explicit connection",
        )?;
        for (index, connection) in self.connections.iter().enumerate() {
            context.ensure(
                connection.preceding == index && connection.following == index + 1,
                format!("preparedParagraph.connections[{index}]"),
                "connections must identify the actual consecutive origins",
            )?;
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut cursor = 0;
        for (position, occurrence) in self.order.iter().enumerate() {
            let (role, index, child, at, end) = match occurrence.kind {
                OccurrenceKind::Origin => {
                    let row = self.origins.get(occurrence.index);
                    context.ensure(
                        row.is_some(),
                        "preparedParagraph.order",
                        "origin occurrence is out of bounds",
                    )?;
                    let row = row.expect("validated above");
                    (0, occurrence.index, row.child, row.start, row.end)
                }
                OccurrenceKind::Atomic => {
                    let row = self.atomics.get(occurrence.index);
                    context.ensure(
                        row.is_some(),
                        "preparedParagraph.order",
                        "atomic occurrence is out of bounds",
                    )?;
                    let row = row.expect("validated above");
                    (1, occurrence.index, row.child, row.at, row.at)
                }
                OccurrenceKind::Control => {
                    let row = self.controls.get(occurrence.index);
                    context.ensure(
                        row.is_some(),
                        "preparedParagraph.order",
                        "control occurrence is out of bounds",
                    )?;
                    let row = row.expect("validated above");
                    (2, occurrence.index, row.child, row.at, row.at)
                }
            };
            context.ensure(position == child && at == cursor && seen.insert((role,index)),"preparedParagraph.order","captured occurrence order must exactly follow real direct-child and original source order")?;
            cursor = end;
        }
        context.ensure(
            seen.len() == child_count && cursor == source_end,
            "preparedParagraph.order",
            "every source/control/atomic occurrence must occur exactly once",
        )?;
        // The validated occurrence order, not control-array storage order,
        // determines the nesting of actual transparent inline containers.
        let mut depth = 0usize;
        for occurrence in &self.order {
            if let OccurrenceKind::Control = occurrence.kind {
                match self.controls[occurrence.index].kind {
                    ControlKind::InlineBoundaryStart => depth += 1,
                    ControlKind::InlineBoundaryEnd => {
                        context.ensure(
                            depth > 0,
                            "preparedParagraph.controls",
                            "an actual boundary end requires a preceding unmatched start",
                        )?;
                        depth -= 1;
                    }
                    ControlKind::ForcedBreak => {}
                }
            }
        }
        context.ensure(
            depth == 0,
            "preparedParagraph.controls",
            "every actual boundary start requires a matching end",
        )?;
        serde_json::to_string(self).map_err(|error| {
            context.error(
                "preparedParagraph",
                super::MeasurementErrorKind::InvalidValue,
                error.to_string(),
            )
        })
    }
}

impl TextStyle {
    fn validate(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        context.ensure(
            self.font_size > 0.0
                && self.font_size.is_finite()
                && f64::from(self.font_size as f32) == self.font_size,
            "preparedParagraph.fontSize",
            "font size must be a positive exact canonical f32 value",
        )?;
        context.ensure(
            self.color.iter().all(|value| {
                value.is_finite()
                    && (0.0..=1.0).contains(value)
                    && f64::from(*value as f32) == *value
            }),
            "preparedParagraph.color",
            "color must be canonical finite RGBA",
        )?;
        nonnegative(self.line_height, context, "preparedParagraph.lineHeight")?;
        super::finite(
            self.letter_spacing,
            context,
            "preparedParagraph.letterSpacing",
        )?;
        super::finite(self.word_spacing, context, "preparedParagraph.wordSpacing")?;
        Ok(())
    }
}
