//! Independent version-2 XML source capture consumer. This module deliberately
//! imports neither generator wire types nor browser observation structures.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use surgeist_text as text;

use super::{Error, layout, pinned_font};

#[derive(Clone, Debug)]
pub(super) struct Capture {
    raw: Value,
    pub prepared: text::PreparedText,
    pub origins: Vec<(usize, text::PreparedOrigin)>,
    pub atomics: Vec<(usize, text::PreparedAtomic)>,
    pub controls: Vec<(usize, text::PreparedControl)>,
    children: usize,
}
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

fn fail(message: impl Into<String>) -> Error {
    Error::new(format!("prepared source: {}", message.into()))
}
fn object<'a>(value: &'a Value, keys: &[&str]) -> Result<&'a Map<String, Value>, Error> {
    let object = value
        .as_object()
        .ok_or_else(|| fail("expected a closed object"))?;
    if object.len() != keys.len() || keys.iter().any(|key| !object.contains_key(*key)) {
        return Err(fail(format!("expected exactly fields {}", keys.join(","))));
    }
    Ok(object)
}
fn array(value: &Value) -> Result<&[Value], Error> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| fail("expected an array"))
}
fn index(value: &Value) -> Result<usize, Error> {
    usize::try_from(
        value
            .as_u64()
            .ok_or_else(|| fail("expected a nonnegative integer"))?,
    )
    .map_err(|_| fail("index out of range"))
}
fn string(value: &Value) -> Result<&str, Error> {
    value.as_str().ok_or_else(|| fail("expected a string"))
}
fn number(value: &Value) -> Result<f64, Error> {
    value
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| fail("expected a finite number"))
}
fn extent(value: &Value) -> Result<f64, Error> {
    let value = number(value)?;
    if value < 0.0 {
        Err(fail("negative extent"))
    } else {
        Ok(value)
    }
}
fn wrap(value: &Value) -> Result<text::Wrap, Error> {
    match string(value)? {
        "word" => Ok(text::Wrap::Word),
        "none" => Ok(text::Wrap::None),
        _ => Err(fail("unsupported wrapping profile")),
    }
}
fn style(value: &Value) -> Result<text::Style, Error> {
    let value = object(
        value,
        &[
            "fontSize",
            "lineHeight",
            "whitespace",
            "wrap",
            "letterSpacing",
            "wordSpacing",
            "color",
            "locale",
        ],
    )?;
    let size = extent(&value["fontSize"])?;
    let canonical = size as f32;
    if size <= 0.0 || !canonical.is_finite() || f64::from(canonical) != size {
        return Err(fail("font size must be an exact canonical f32 value"));
    }
    let color = array(&value["color"])?;
    if color.len() != 4 {
        return Err(fail("color must have four canonical components"));
    }
    let color = color
        .iter()
        .map(|value| {
            number(value).and_then(|value| {
                if (0.0..=1.0).contains(&value) && f64::from(value as f32) == value {
                    Ok(value as f32)
                } else {
                    Err(fail("color component out of range"))
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(text::Style {
        font: text::Font::new().family("Ahem"),
        brush: text::Brush::color(color[0], color[1], color[2], color[3]),
        size: canonical,
        locale: if value["locale"].is_null() {
            None
        } else {
            Some(string(&value["locale"])?.to_owned())
        },
        line_height: text::LineHeight::Absolute(extent(&value["lineHeight"])?),
        white_space: match string(&value["whitespace"])? {
            "collapse" => text::WhiteSpace::Collapse,
            "preserve" => text::WhiteSpace::Preserve,
            "preserve-spaces" => text::WhiteSpace::PreserveSpaces,
            _ => return Err(fail("unsupported whitespace profile")),
        },
        wrap: wrap(&value["wrap"])?,
        letter_spacing: number(&value["letterSpacing"])?,
        word_spacing: number(&value["wordSpacing"])?,
        ..text::Style::default()
    })
}

impl Capture {
    pub fn parse(raw: &str, children: usize) -> Result<Self, Error> {
        // serde_json's Value parser alone permits duplicate keys, so use its
        // streaming visitor before admitting the independently owned schema.
        let raw = unique_json(raw)?;
        let value = object(
            &raw,
            &[
                "font",
                "direction",
                "writingMode",
                "baseStyle",
                "origins",
                "connections",
                "atomics",
                "controls",
                "order",
            ],
        )?;
        let font = object(
            &value["font"],
            &[
                "id",
                "sha256",
                "faceIndex",
                "family",
                "style",
                "weight",
                "stretch",
                "features",
                "variations",
                "synthesis",
                "loaded",
            ],
        )?;
        let actual = format!("{:x}", Sha256::digest(pinned_font::BYTES));
        if actual != pinned_font::SHA256
            || string(&font["id"])? != pinned_font::ID
            || string(&font["sha256"])? != actual
            || index(&font["faceIndex"])? != 0
            || string(&font["family"])? != "SurgeistParityAhem"
            || string(&font["style"])? != "normal"
            || index(&font["weight"])? != 400
            || string(&font["stretch"])? != "normal"
            || string(&font["features"])? != "normal"
            || string(&font["variations"])? != "normal"
            || string(&font["synthesis"])? != "none"
            || font["loaded"] != Value::Bool(true)
        {
            return Err(fail("pinned resource/descriptors/loading proof mismatch"));
        }
        let vertical = match string(&value["writingMode"])? {
            "horizontal-tb" => false,
            "vertical-lr" | "vertical-rl" => true,
            _ => return Err(fail("unsupported writing mode")),
        };
        let direction = match string(&value["direction"])? {
            "ltr" => text::ParagraphBaseDirection::LeftToRight,
            "rtl" => text::ParagraphBaseDirection::RightToLeft,
            _ => return Err(fail("unresolved base direction")),
        };
        let mut source = text::Source::new("");
        let mut origins = Vec::new();
        let mut origin_children = Vec::new();
        let mut origin_wraps = Vec::new();
        let mut roles = std::collections::BTreeSet::new();
        for row in array(&value["origins"])? {
            let row = object(row, &["child", "units", "start", "end", "style"])?;
            let child = index(&row["child"])?;
            if child >= children || !roles.insert(child) {
                return Err(fail("origin child is missing or repeated"));
            }
            let units = array(&row["units"])?
                .iter()
                .map(|unit| {
                    u16::try_from(index(unit)?).map_err(|_| fail("UTF16 unit out of range"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if !pinned_font::covers(&units) {
                return Err(fail(
                    "original source lacks pinned nominal font coverage or supported glyph-free semantics",
                ));
            }
            let original = text::Source::new(text::Text::from_code_units(units));
            let aggregate = source.push(original.text().clone());
            if aggregate.start != index(&row["start"])? || aggregate.end != index(&row["end"])? {
                return Err(fail("original range does not match exact UTF16 units"));
            }
            let resolved = style(&row["style"])?;
            if vertical
                && (resolved.white_space != text::WhiteSpace::Collapse
                    || original
                        .text()
                        .code_units()
                        .iter()
                        .any(|unit| !matches!(unit, 10 | 13 | 32)))
            {
                return Err(fail(
                    "vertical source admits only actual collapsing LF/CR/SPACE formatting occurrences",
                ));
            }
            origin_wraps.push(resolved.wrap);
            source.span(aggregate, resolved.clone());
            origin_children.push(child);
            origins.push(text::ParagraphOrigin {
                text_node: None,
                source: original,
                local: text::Range::new(0, aggregate.len()),
                aggregate,
                kind: text::ParagraphOriginKind::Authored,
            });
        }
        let mut connections = Vec::new();
        let mut policies = Vec::new();
        let rows = array(&value["connections"])?;
        if rows.len() != origins.len().saturating_sub(1) {
            return Err(fail("incomplete incident connection inventory"));
        }
        for (index_, row) in rows.iter().enumerate() {
            let row = object(
                row,
                &[
                    "preceding",
                    "following",
                    "kind",
                    "precedingWrap",
                    "followingWrap",
                    "ancestorWrap",
                ],
            )?;
            if index(&row["preceding"])? != index_ || index(&row["following"])? != index_ + 1 {
                return Err(fail("connection is not between actual consecutive origins"));
            }
            let kind = match string(&row["kind"])? {
                "continuous" => text::ParagraphConnectionKind::Continuous,
                "barrier" => text::ParagraphConnectionKind::Disconnected,
                _ => return Err(fail("invalid connection role")),
            };
            connections.push(text::ParagraphConnection {
                at: origins[index_ + 1].aggregate.start,
                kind,
            });
            if kind == text::ParagraphConnectionKind::Continuous {
                policies.push(text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(index_),
                    preceding_wrap: wrap(&row["precedingWrap"])?,
                    following_wrap: wrap(&row["followingWrap"])?,
                    ancestor_wrap: wrap(&row["ancestorWrap"])?,
                });
            }
        }
        let mut atomic_children = Vec::new();
        let mut control_children = Vec::new();
        let mut controls = Vec::new();
        for (occurrence, row) in array(&value["atomics"])?.iter().enumerate() {
            let row = object(
                row,
                &[
                    "child",
                    "at",
                    "kind",
                    "width",
                    "height",
                    "precedingWrap",
                    "followingWrap",
                    "ancestorWrap",
                ],
            )?;
            let child = index(&row["child"])?;
            if child >= children || !roles.insert(child) {
                return Err(fail("atomic child is missing or repeated"));
            }
            let kind = match string(&row["kind"])? {
                "in-flow" => text::InlineBoxKind::InFlow,
                "out-of-flow" => text::InlineBoxKind::OutOfFlow,
                _ => return Err(fail("invalid atomic role")),
            };
            if vertical && kind != text::InlineBoxKind::InFlow {
                return Err(fail(
                    "vertical source supports only actual in-flow opaque atomics",
                ));
            }
            let at = index(&row["at"])?;
            if at > source.text().len() {
                return Err(fail("atomic anchor exceeds original source"));
            }
            source.inline_box(text::InlineBox::new(
                text::Id::from_u64(
                    u64::try_from(child).map_err(|_| fail("child identity out of range"))?,
                ),
                kind,
                at,
                text::Size::new(extent(&row["width"])?, extent(&row["height"])?),
            ));
            if kind == text::InlineBoxKind::InFlow {
                for definition in [
                    text::ParagraphBoundaryDefinition::AtomicBefore(occurrence),
                    text::ParagraphBoundaryDefinition::AtomicAfter(occurrence),
                ] {
                    policies.push(text::ParagraphBoundaryPolicy {
                        definition,
                        preceding_wrap: wrap(&row["precedingWrap"])?,
                        following_wrap: wrap(&row["followingWrap"])?,
                        ancestor_wrap: wrap(&row["ancestorWrap"])?,
                    });
                }
            }
            atomic_children.push(child);
        }
        for row in array(&value["controls"])? {
            let row = object(row, &["child", "at", "kind"])?;
            let child = index(&row["child"])?;
            if child >= children || !roles.insert(child) {
                return Err(fail("control child is missing or repeated"));
            }
            let kind = match string(&row["kind"])? {
                "forced-break" => text::ParagraphControlKind::ForcedBreak,
                "inline-boundary-start" => text::ParagraphControlKind::InlineBoundaryStart,
                "inline-boundary-end" => text::ParagraphControlKind::InlineBoundaryEnd,
                _ => return Err(fail("invalid actual control role")),
            };
            if vertical && !matches!(&kind, text::ParagraphControlKind::ForcedBreak) {
                return Err(fail(
                    "vertical source does not admit independent boundary controls",
                ));
            }
            controls.push(text::ParagraphControl {
                at: index(&row["at"])?,
                kind,
            });
            control_children.push(child);
        }
        if roles.len() != children {
            return Err(fail("incomplete actual participant inventory"));
        }
        let mut anchors = Vec::new();
        // Prepared control/atomic views follow actual anchor-event order, not
        // the raw definition-array index. The captured mixed order owns the
        // association with each real XML child, including equal-offset rows.
        let mut ordered_control_children = Vec::new();
        let mut ordered_atomic_children = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        let mut cursor = 0;
        for (position, row) in array(&value["order"])?.iter().enumerate() {
            let row = object(row, &["kind", "index", "affinity"])?;
            let ordinal = index(&row["index"])?;
            let affinity = match string(&row["affinity"])? {
                "before" => text::Affinity::Before,
                "after" => text::Affinity::After,
                _ => return Err(fail("invalid captured affinity")),
            };
            let (role, child, at, end, item) = match string(&row["kind"])? {
                "origin" => {
                    let origin = origins
                        .get(ordinal)
                        .ok_or_else(|| fail("missing original occurrence"))?;
                    (
                        0,
                        *origin_children
                            .get(ordinal)
                            .ok_or_else(|| fail("missing original child"))?,
                        origin.aggregate.start,
                        origin.aggregate.end,
                        origin
                            .aggregate
                            .is_empty()
                            .then_some(text::ParagraphAnchorItem::EmptyOrigin(ordinal)),
                    )
                }
                "atomic" => {
                    let box_ = source
                        .boxes()
                        .get(ordinal)
                        .ok_or_else(|| fail("missing atomic occurrence"))?;
                    (
                        1,
                        atomic_children[ordinal],
                        box_.index(),
                        box_.index(),
                        Some(text::ParagraphAnchorItem::Atomic(ordinal)),
                    )
                }
                "control" => {
                    let control = controls
                        .get(ordinal)
                        .ok_or_else(|| fail("missing control occurrence"))?;
                    (
                        2,
                        control_children[ordinal],
                        control.at,
                        control.at,
                        Some(text::ParagraphAnchorItem::Control(ordinal)),
                    )
                }
                _ => return Err(fail("invalid occurrence role")),
            };
            if child != position || at != cursor || !seen.insert((role, ordinal)) {
                return Err(fail(
                    "original occurrence sequence/anchor does not match the actual child order",
                ));
            }
            cursor = end;
            if role == 1 {
                ordered_atomic_children.push(child);
            }
            if role == 2 {
                ordered_control_children.push(child);
            }
            if let Some(item) = item {
                anchors.push(text::ParagraphAnchor { item, affinity });
            }
        }
        if seen.len() != children || cursor != source.text().len() {
            return Err(fail("incomplete original sequence"));
        }
        let base = style(&value["baseStyle"])?;
        let analyzed = text::AnalyzedSource::from_paragraph(
            text::ParagraphInput {
                terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source,
                base_style: base,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: direction,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
                origins,
                connections,
                controls,
                boundary_policies: policies,
                replacements: Vec::new(),
                anchor_order: anchors,
            },
            if vertical {
                text::AnalysisTypography::Mixed
            } else {
                text::AnalysisTypography::HorizontalComposition
            },
        )
        .map_err(|error| fail(error.to_string()))?;
        // Each origin captures one actual DOM text node's containing style.
        // It owns NCA permission only for a pair inside that same origin;
        // paragraph base style cannot stand in for unknown cross-origin ancestry.
        let same_origin = analyzed
            .wrapping_relations()
            .filter(|relation| relation.ancestor_wrap().is_none())
            .filter_map(|relation| match (relation.before(), relation.after()) {
                (
                    text::WrappingNeighbor::Text { origin: before, .. },
                    text::WrappingNeighbor::Text { origin: after, .. },
                ) if before == after => Some(text::WrappingPolicy {
                    relation: relation.handle(),
                    ancestor_wrap: origin_wraps[before],
                }),
                _ => None,
            })
            .collect();
        let analyzed = analyzed
            .with_wrapping_policies(same_origin)
            .map_err(|error| fail(error.to_string()))?;
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .map_err(|error| fail(error.to_string()))?;
        system
            .register_fonts(pinned_font::BYTES.to_vec())
            .map_err(|error| fail(error.to_string()))?;
        let prepared = system
            .prepare(
                &analyzed,
                text::PreparationOptions::default().with_orientation(if vertical {
                    text::LineOrientation::Vertical
                } else {
                    text::LineOrientation::Horizontal
                }),
            )
            .map_err(|error| fail(error.to_string()))?;
        for unit in prepared.unit_views() {
            for instance in unit.font_instances() {
                let instance = instance.map_err(|error| fail(error.to_string()))?;
                if instance.face().bytes() != pinned_font::BYTES
                    || instance.face().index() != 0
                    || instance.axis_count() != 0
                    || instance.synthesis() != text::FontSynthesis::default()
                {
                    return Err(fail(
                        "actual Text matching escaped the exact pinned face/instance",
                    ));
                }
            }
        }
        Ok(Self {
            raw,
            origins: prepared
                .origin_views()
                .zip(origin_children)
                .map(|(view, child)| (child, view.handle()))
                .collect(),
            atomics: prepared
                .atomic_views()
                .zip(ordered_atomic_children)
                .map(|(view, child)| (child, view.handle()))
                .collect(),
            controls: prepared
                .control_views()
                .zip(ordered_control_children)
                .map(|(view, child)| (child, view.handle()))
                .collect(),
            prepared,
            children,
        })
    }
    pub(super) fn validate_container(
        &self,
        attrs: &super::StyleAttrs,
        children: &[super::Node],
    ) -> Result<(), Error> {
        let native = super::to_node_input(attrs)?;
        let mode_name = string(&self.raw["writingMode"])?;
        let direction_name = string(&self.raw["direction"])?;
        let mode = super::parse_writing_mode(Some(mode_name))?;
        let direction = super::parse_direction(direction_name)?;
        if native.writing_mode != mode || native.direction != direction {
            return Err(fail(
                "captured paragraph mode/direction differs from its actual XML container",
            ));
        }
        let vertical = self.prepared.orientation() == text::LineOrientation::Vertical;
        let has_break = self.controls.iter().any(|(_, control)| {
            matches!(
                control.view().kind(),
                text::ParagraphControlKind::ForcedBreak
            )
        });
        if vertical
            && (native.display != layout::Display::Block
                || attrs
                    .get("text-orientation")
                    .is_some_and(|value| value != "mixed"))
        {
            return Err(fail(
                "vertical source requires the supported mixed block context",
            ));
        }
        if !vertical && !has_break {
            return Ok(());
        }
        let font_size = super::font_size(attrs)?.ok_or_else(|| {
            fail("captured BR/vertical context needs its actual container font size")
        })?;
        let line_height = super::line_height(attrs)?.ok_or_else(|| {
            fail("captured BR/vertical context needs its actual container line height")
        })?;
        let family = super::font_family(attrs)?.unwrap_or(super::FontFamily::Ahem);
        if family != super::FontFamily::Ahem
            || f64::from(font_size) != number(&self.raw["baseStyle"]["fontSize"])?
            || f64::from(line_height) != number(&self.raw["baseStyle"]["lineHeight"])?
        {
            return Err(fail(
                "container font context differs from the actual captured default-font strut",
            ));
        }
        let flow = layout::FlowAxes::new(mode, direction);
        for (child, control) in &self.controls {
            if !matches!(
                control.view().kind(),
                text::ParagraphControlKind::ForcedBreak
            ) {
                continue;
            }
            let child = &children[*child];
            let input = super::to_node_input_in_flow(&child.style, Some(flow))?;
            let same_frame = child
                .style
                .get("writing-mode")
                .is_none_or(|value| value == mode_name)
                && child
                    .style
                    .get("direction")
                    .is_none_or(|value| value == direction_name);
            let unshifted = [
                input.inset.top,
                input.inset.right,
                input.inset.bottom,
                input.inset.left,
            ]
            .iter()
            .all(|offset| offset.is_auto() || *offset == layout::LengthAuto::ZERO);
            let zero_box_edges = [
                input.margin.top,
                input.margin.right,
                input.margin.bottom,
                input.margin.left,
            ]
            .iter()
            .all(|edge| *edge == layout::LengthAuto::ZERO)
                && [
                    input.padding.top,
                    input.padding.right,
                    input.padding.bottom,
                    input.padding.left,
                    input.border.top,
                    input.border.right,
                    input.border.bottom,
                    input.border.left,
                ]
                .iter()
                .all(|edge| *edge == layout::Length::ZERO);
            if child.style.get("source-tag") != Some("br")
                || child.style.get("display") != Some("inline")
                || !child.children.is_empty()
                || child.text.is_some()
                || child.inline_boundary.is_some()
                || input.position != layout::Position::Relative
                || !input.float.is_none()
                || input.clear != layout::Clear::None
                || input.vertical_align != layout::VerticalAlign::Baseline
                || !same_frame
                || !unshifted
                || !zero_box_edges
                || child.style.get("inline-baseline").is_some()
                || child.style.get("inline-line-height").is_some()
                || super::font_family(&child.style)?.unwrap_or(family) != family
                || super::font_size(&child.style)?.unwrap_or(font_size) != font_size
                || super::line_height(&child.style)?.unwrap_or(line_height) != line_height
            {
                return Err(fail(
                    "captured BR requires an unchanged visible font/frame, baseline alignment, clear:none, zero box edges and no second metrics or displacement",
                ));
            }
        }
        if vertical {
            for row in array(&self.raw["origins"])? {
                let own = &row["style"];
                if number(&own["fontSize"])? != f64::from(font_size)
                    || number(&own["lineHeight"])? != f64::from(line_height)
                    || number(&own["letterSpacing"])? != 0.0
                    || number(&own["wordSpacing"])? != 0.0
                {
                    return Err(fail(
                        "vertical formatting occurrences require the actual unchanged paragraph font/spacing context",
                    ));
                }
            }
            for row in array(&self.raw["atomics"])? {
                let child = &children[index(&row["child"])?];
                let input = super::to_node_input_in_flow(&child.style, Some(flow))?;
                let width = child
                    .style
                    .get("width")
                    .ok_or_else(|| fail("vertical atomic needs authored fixed-px width"))?;
                let height = child
                    .style
                    .get("height")
                    .ok_or_else(|| fail("vertical atomic needs authored fixed-px height"))?;
                if input.display != layout::Display::InlineBlock
                    || input.position != layout::Position::Relative
                    || !input.float.is_none()
                    || input.writing_mode != mode
                    || input.direction != direction
                    || !child.children.is_empty()
                    || child.text.is_some()
                    || f64::from(super::parse_px_dimension(width, "width")?)
                        != number(&row["width"])?
                    || f64::from(super::parse_px_dimension(height, "height")?)
                        != number(&row["height"])?
                {
                    return Err(fail(
                        "vertical source requires empty same-frame fixed-px inline-blocks",
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn binding(
        &self,
        container: usize,
        children: &[usize],
    ) -> Result<layout::TextParagraphBinding<usize>, Error> {
        if children.len() != self.children {
            return Err(fail(
                "actual tree child count changed after immutable capture",
            ));
        }
        Ok(layout::TextParagraphBinding {
            inline_box_fragments: layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: self.prepared.clone(),
            container,
            nodes: children.to_vec(),
            origins: self
                .origins
                .iter()
                .map(|(child, origin)| layout::TextOriginBinding {
                    node: children[*child],
                    origin: origin.clone(),
                })
                .collect(),
            atomics: self
                .atomics
                .iter()
                .map(|(child, occurrence)| layout::TextAtomicBinding {
                    node: children[*child],
                    occurrence: occurrence.clone(),
                })
                .collect(),
            controls: self
                .controls
                .iter()
                .map(|(child, occurrence)| layout::TextControlBinding {
                    node: children[*child],
                    occurrence: occurrence.clone(),
                })
                .collect(),
            line_strut: None,
        })
    }
}

/// The independent reader rejects duplicate fields before a Value can erase
/// their occurrence. Serde's own parser retains its normal nesting bound.
fn unique_json(raw: &str) -> Result<Value, Error> {
    use serde::{
        Deserialize, Deserializer,
        de::{self, MapAccess, SeqAccess, Visitor},
    };
    struct Unique(Value);
    impl<'de> Deserialize<'de> for Unique {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct Json;
            impl<'de> Visitor<'de> for Json {
                type Value = Unique;
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("JSON without duplicate object fields")
                }
                fn visit_bool<E: de::Error>(self, value: bool) -> Result<Unique, E> {
                    Ok(Unique(Value::Bool(value)))
                }
                fn visit_i64<E: de::Error>(self, value: i64) -> Result<Unique, E> {
                    Ok(Unique(value.into()))
                }
                fn visit_u64<E: de::Error>(self, value: u64) -> Result<Unique, E> {
                    Ok(Unique(value.into()))
                }
                fn visit_f64<E: de::Error>(self, value: f64) -> Result<Unique, E> {
                    serde_json::Number::from_f64(value)
                        .map(|value| Unique(Value::Number(value)))
                        .ok_or_else(|| E::custom("nonfinite JSON number"))
                }
                fn visit_str<E: de::Error>(self, value: &str) -> Result<Unique, E> {
                    Ok(Unique(value.into()))
                }
                fn visit_string<E: de::Error>(self, value: String) -> Result<Unique, E> {
                    Ok(Unique(value.into()))
                }
                fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
                    Ok(Unique(Value::Null))
                }
                fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Unique, A::Error> {
                    let mut values = Vec::new();
                    while let Some(Unique(value)) = access.next_element()? {
                        values.push(value);
                    }
                    Ok(Unique(Value::Array(values)))
                }
                fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Unique, A::Error> {
                    let mut values = Map::new();
                    while let Some(key) = access.next_key::<String>()? {
                        if values.contains_key(&key) {
                            return Err(de::Error::custom(format!("duplicate field {key}")));
                        }
                        let Unique(value) = access.next_value()?;
                        values.insert(key, value);
                    }
                    Ok(Unique(Value::Object(values)))
                }
            }
            deserializer.deserialize_any(Json)
        }
    }
    serde_json::from_str::<Unique>(raw)
        .map(|value| value.0)
        .map_err(|error| fail(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn original_capture() -> Value {
        let style = json!({"fontSize":10.0,"lineHeight":12.0,"whitespace":"preserve","wrap":"word","letterSpacing":0.0,"wordSpacing":0.0,"color":[0.0,0.0,0.0,1.0],"locale":"en"});
        json!({"font":{"id":pinned_font::ID,"sha256":pinned_font::SHA256,"faceIndex":0,"family":"SurgeistParityAhem","style":"normal","weight":400,"stretch":"normal","features":"normal","variations":"normal","synthesis":"none","loaded":true},"direction":"ltr","writingMode":"horizontal-tb","baseStyle":style,"origins":[{"child":0,"units":[88,32,88],"start":0,"end":3,"style":style}],"connections":[],"atomics":[],"controls":[],"order":[{"kind":"origin","index":0,"affinity":"before"}]})
    }

    #[test]
    fn permuted_control_array_keeps_explicit_event_order_and_placed_owner() {
        for (raw_children, kinds, order, xml) in [
            (
                vec![1, 0],
                vec!["inline-boundary-end", "inline-boundary-start"],
                vec![1, 0],
                "<inline-boundary kind=\"start\"/><inline-boundary kind=\"end\"/>",
            ),
            (
                vec![1, 0, 2, 3],
                vec![
                    "inline-boundary-start",
                    "inline-boundary-start",
                    "inline-boundary-end",
                    "inline-boundary-end",
                ],
                vec![1, 0, 2, 3],
                "<inline-boundary kind=\"start\"/><inline-boundary kind=\"start\"/><inline-boundary kind=\"end\"/><inline-boundary kind=\"end\"/>",
            ),
        ] {
            let mut raw = original_capture();
            raw["origins"] = json!([]);
            raw["controls"] = Value::Array(
                raw_children
                    .iter()
                    .zip(kinds)
                    .map(|(child, kind)| json!({"child":child,"at":0,"kind":kind}))
                    .collect(),
            );
            raw["order"] = Value::Array(
                order
                    .iter()
                    .map(|index| json!({"kind":"control","index":index,"affinity":"before"}))
                    .collect(),
            );
            let encoded = raw
                .to_string()
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;");
            let golden=super::super::Golden::parse(&format!("<test name=\"permuted-control-owner\" measurement-version=\"2\" use-rounding=\"false\"><viewport width=\"100\" height=\"max-content\"/><input><div display=\"block\" layout-ready-inline-root=\"true\"><prepared-paragraph capture=\"{encoded}\"/>{xml}</div></input><expectations><node/></expectations></test>"))
                .expect("actual explicit order must bind native XML leaves independently of raw array indices");
            let prepared = &golden.root.prepared.as_ref().unwrap().prepared;
            let expected = prepared
                .control_views()
                .map(|view| view.handle())
                .collect::<Vec<_>>();
            let tree = super::super::TestTree::from_golden(&golden.root).unwrap();
            let capture = layout::LayoutFlowInput::try_new(
                &tree,
                0,
                tree.paragraphs.clone(),
                layout::WrappingInput {
                    native_inline_boxes: Vec::new(),
                    gaps: Vec::new(),
                    scopes: Vec::new(),
                    child_order: Vec::new(),
                },
                Vec::new(),
            )
            .unwrap();
            let request = layout::LayoutRootRequest::viewport(layout::Size::new(
                layout::Available::definite(100.0),
                layout::Available::MaxContent,
            ))
            .unwrap();
            let batch = layout::compute_layout_with_flow(&capture, 0, request).unwrap();
            assert!(batch.unrounded_inline_fragments().is_empty());
            assert!(batch.final_inline_fragments().is_empty());
            let placed = batch.placed_text()[0].layout();
            assert!(placed.lines().is_empty());
            let receipts = placed.nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(receipts.len(), 1);
            let line = receipts[0].line.finalized_line();
            assert_eq!(line.prepared_text(), *prepared);
            assert!(!line.has_in_flow_content());
            assert_eq!(line.fragments().len(), raw_children.len());
            assert_eq!(
                line.selected_line()
                    .request()
                    .controls
                    .iter()
                    .map(|row| row.occurrence.clone())
                    .collect::<Vec<_>>(),
                expected,
                "the nonvisual selection retains the actual explicit event order"
            );
            for fragment in line.fragments() {
                let text::FinalizedFragmentOwner::Control(control) = fragment.view().owner() else {
                    panic!("genuine control must remain Control-owned")
                };
                let binding = tree.paragraphs[0]
                    .controls
                    .iter()
                    .find(|row| row.occurrence == control)
                    .expect("the actual retained control still binds its genuine native child");
                assert_eq!(
                    control,
                    expected[binding.node - 1],
                    "native child must retain its actual event owner, including same-kind same-offset controls"
                );
                assert!(fragment.view().origin().is_none());
            }
        }
    }

    #[test]
    fn original_units_create_retained_real_preparation_without_observation_inputs() {
        let capture = Capture::parse(&original_capture().to_string(), 1).unwrap();
        assert_eq!(capture.origins.len(), 1);
        assert_eq!(
            capture
                .prepared
                .analyzed_source()
                .source()
                .text()
                .code_units(),
            &[88, 32, 88]
        );
        assert!(
            capture
                .prepared
                .unit_views()
                .any(|unit| unit.font_instances().next().is_some())
        );
        assert!(capture.binding(4, &[7]).is_ok());
        assert!(capture.binding(4, &[]).is_err());
    }

    #[test]
    fn same_origin_permission_uses_its_containing_style_instead_of_paragraph_base() {
        let captures = [("none", "word"), ("word", "none")].map(|(base, origin)| {
            let mut raw = original_capture();
            raw["baseStyle"]["wrap"] = json!(base);
            raw["origins"][0]["style"]["wrap"] = json!(origin);
            Capture::parse(&raw.to_string(), 1).unwrap()
        });
        for (capture, expected) in captures.iter().zip([1, 0]) {
            assert_eq!(
                capture
                    .prepared
                    .analyzed_source()
                    .source()
                    .text()
                    .code_units(),
                &[88, 32, 88]
            );
            assert_eq!(
                capture
                    .prepared
                    .opportunity_views()
                    .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
                    .count(),
                expected
            );
            assert!(
                capture
                    .prepared
                    .analyzed_source()
                    .wrapping_relations()
                    .all(|relation| relation.ancestor_wrap().is_some())
            );
        }
    }

    #[test]
    fn same_origin_binding_preserves_captured_cross_origin_permission() {
        let mut raw = original_capture();
        let style = raw["origins"][0]["style"].clone();
        raw["origins"] = json!([
            {"child":0,"units":[88,32],"start":0,"end":2,"style":style},
            {"child":1,"units":[88],"start":2,"end":3,"style":style}
        ]);
        raw["connections"] = json!([{"preceding":0,"following":1,"kind":"continuous","precedingWrap":"word","followingWrap":"word","ancestorWrap":"none"}]);
        raw["order"] = json!([{"kind":"origin","index":0,"affinity":"before"},{"kind":"origin","index":1,"affinity":"before"}]);
        let capture = Capture::parse(&raw.to_string(), 2).unwrap();
        assert_eq!(
            capture
                .prepared
                .analyzed_source()
                .source()
                .text()
                .code_units(),
            &[88, 32, 88]
        );
        assert!(
            !capture
                .prepared
                .opportunity_views()
                .any(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
        );
        assert!(
            capture
                .prepared
                .analyzed_source()
                .wrapping_relations()
                .any(|relation| matches!(
                    (relation.before(), relation.after()),
                    (
                        text::WrappingNeighbor::Text { origin: 0, .. },
                        text::WrappingNeighbor::Text { origin: 1, .. }
                    )
                ) && relation.ancestor_wrap() == Some(text::Wrap::None))
        );
    }

    #[test]
    fn missing_cross_origin_ancestry_is_not_inferred_across_an_empty_origin() {
        let mut raw = original_capture();
        let style = raw["origins"][0]["style"].clone();
        raw["origins"] = json!([
            {"child":0,"units":[88,32],"start":0,"end":2,"style":style},
            {"child":1,"units":[],"start":2,"end":2,"style":style},
            {"child":2,"units":[88],"start":2,"end":3,"style":style}
        ]);
        raw["connections"] = json!([
            {"preceding":0,"following":1,"kind":"continuous","precedingWrap":"word","followingWrap":"word","ancestorWrap":"word"},
            {"preceding":1,"following":2,"kind":"continuous","precedingWrap":"word","followingWrap":"word","ancestorWrap":"word"}
        ]);
        raw["order"] = json!([{"kind":"origin","index":0,"affinity":"before"},{"kind":"origin","index":1,"affinity":"before"},{"kind":"origin","index":2,"affinity":"before"}]);
        assert!(Capture::parse(&raw.to_string(), 3).is_err());
    }

    #[test]
    fn source_font_identity_and_coverage_cannot_be_forged_by_geometry() {
        for (field, value) in [
            ("sha256", json!("wrong")),
            ("loaded", json!(false)),
            ("faceIndex", json!(1)),
        ] {
            let mut raw = original_capture();
            raw["font"][field] = value;
            assert!(Capture::parse(&raw.to_string(), 1).is_err());
        }
        for units in [json!([0x05d0]), json!([0xd800]), json!([9])] {
            let mut raw = original_capture();
            raw["origins"][0]["units"] = units;
            raw["origins"][0]["end"] = json!(1);
            assert!(
                Capture::parse(&raw.to_string(), 1)
                    .unwrap_err()
                    .to_string()
                    .contains("coverage")
            );
        }
        let mut raw = original_capture();
        raw["origins"][0]["advance"] = json!(30.0);
        assert!(Capture::parse(&raw.to_string(), 1).is_err());
    }

    #[test]
    fn closed_capture_rejects_duplicate_unknown_order_and_numeric_loss() {
        assert!(unique_json(r#"{"font":0,"\u0066ont":1}"#).is_err());
        for raw in [
            {
                let mut v = original_capture();
                v["outputLevel"] = json!(0);
                v
            },
            {
                let mut v = original_capture();
                v["order"][0]["index"] = json!(1);
                v
            },
            {
                let mut v = original_capture();
                v["origins"][0]["start"] = json!(1);
                v
            },
            {
                let mut v = original_capture();
                v["baseStyle"]["fontSize"] = json!(0.0);
                v
            },
            {
                let mut v = original_capture();
                v["baseStyle"]["color"][0] = json!(1.0000000000000002);
                v
            },
        ] {
            assert!(Capture::parse(&raw.to_string(), 1).is_err());
        }
    }

    fn opaque_break_capture(writing_mode: &str, direction: &str, line_height: f64) -> Value {
        let mut raw = original_capture();
        raw["writingMode"] = json!(writing_mode);
        raw["direction"] = json!(direction);
        raw["baseStyle"]["fontSize"] = json!(20.0);
        raw["baseStyle"]["lineHeight"] = json!(line_height);
        raw["origins"] = json!([]);
        raw["connections"] = json!([]);
        raw["atomics"] = json!([
            {"child":0,"at":0,"kind":"in-flow","width":10.0,"height":10.0,"precedingWrap":"word","followingWrap":"word","ancestorWrap":"word"},
            {"child":2,"at":0,"kind":"in-flow","width":10.0,"height":10.0,"precedingWrap":"word","followingWrap":"word","ancestorWrap":"word"}
        ]);
        raw["controls"] = json!([{"child":1,"at":0,"kind":"forced-break"}]);
        raw["order"] = json!([
            {"kind":"atomic","index":0,"affinity":"before"},
            {"kind":"control","index":0,"affinity":"before"},
            {"kind":"atomic","index":1,"affinity":"before"}
        ]);
        raw
    }

    fn assert_empty_source_opaque_break_flow(writing_mode: &str, direction: &str) {
        let vertical = writing_mode != "horizontal-tb";
        for line_height in [30.0, 40.0] {
            let raw = opaque_break_capture(writing_mode, direction, line_height);
            let encoded = raw
                .to_string()
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;");
            let inline_size = if vertical { "height" } else { "width" };
            let golden = super::super::Golden::parse(&format!(
                r#"<test name="empty-source-opaque-break" measurement-version="2" use-rounding="false">
                    <viewport width="120px" height="120px"/>
                    <input><div display="block" direction="{direction}" writing-mode="{writing_mode}" {inline_size}="120px" font-size="20px" line-height="{line_height}px" layout-ready-inline-root="true">
                        <prepared-paragraph capture="{encoded}"/>
                        <div source-tag="span" display="inline-block" direction="{direction}" writing-mode="{writing_mode}" width="10px" height="10px"/>
                        <div source-tag="br" display="inline" direction="{direction}" writing-mode="{writing_mode}"/>
                        <div source-tag="span" display="inline-block" direction="{direction}" writing-mode="{writing_mode}" width="10px" height="10px"/>
                    </div></input><expectations><node/></expectations>
                </test>"#
            )).unwrap();
            let capture = golden.root.prepared.as_ref().unwrap();
            assert!(
                capture
                    .prepared
                    .analyzed_source()
                    .source()
                    .text()
                    .is_empty()
            );
            assert_eq!(
                capture.prepared.analyzed_source().paragraph_strut(),
                text::ParagraphStrut::DefaultFont
            );
            assert_eq!(capture.prepared.origin_views().len(), 0);
            assert_eq!(capture.prepared.unit_views().len(), 0);
            assert_eq!(
                capture.prepared.orientation(),
                if vertical {
                    text::LineOrientation::Vertical
                } else {
                    text::LineOrientation::Horizontal
                }
            );
            let atomics = capture.prepared.atomic_views().collect::<Vec<_>>();
            assert_eq!(atomics.len(), 2);
            assert_eq!(atomics[0].original_box().id().as_u64(), 0);
            assert_eq!(atomics[1].original_box().id().as_u64(), 2);
            let controls = capture.prepared.control_views().collect::<Vec<_>>();
            assert_eq!(controls.len(), 1);
            assert!(matches!(
                controls[0].kind(),
                text::ParagraphControlKind::ForcedBreak
            ));

            let tree = super::super::TestTree::from_golden(&golden.root).unwrap();
            assert_eq!(tree.paragraphs.len(), 1);
            let binding = &tree.paragraphs[0];
            assert_eq!(binding.nodes, [1, 2, 3]);
            assert!(binding.origins.is_empty());
            assert!(binding.line_strut.is_none());
            assert_eq!(binding.atomics[0].node, 1);
            assert_eq!(binding.atomics[0].occurrence, atomics[0].handle());
            assert_eq!(binding.atomics[1].node, 3);
            assert_eq!(binding.atomics[1].occurrence, atomics[1].handle());
            assert_eq!(binding.controls[0].node, 2);
            assert_eq!(binding.controls[0].occurrence, controls[0].handle());

            let flow = layout::LayoutFlowInput::try_new(
                &tree,
                0,
                tree.paragraphs.clone(),
                layout::WrappingInput::default(),
                Vec::new(),
            )
            .unwrap();
            let request = layout::LayoutRootRequest::viewport(layout::Size::new(
                layout::Available::definite(120.0),
                layout::Available::definite(120.0),
            ))
            .unwrap();
            let batch = layout::compute_layout_with_flow(&flow, 0, request).unwrap();
            assert_eq!(batch.placed_text().len(), 1);
            let placed = batch.placed_text()[0].layout();
            assert!(placed.source().text().is_empty());
            assert_eq!(
                placed.analyzed_source().identity(),
                binding.prepared.analyzed_source().identity()
            );
            assert!(placed.glyph_runs().is_empty());
            assert_eq!(placed.lines().len(), 2);
            let container_origin = batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output()
                .location;
            let boxes = placed.inline_boxes();
            assert_eq!(boxes.len(), 2);
            for (atomic, node, id, line) in [(boxes[0], 1, 0, 0), (boxes[1], 3, 2, 1)] {
                assert_eq!(atomic.id().as_u64(), id);
                assert_eq!(atomic.kind(), text::InlineBoxKind::InFlow);
                assert_eq!(atomic.line(), line);
                assert_eq!(atomic.rect().size, text::Size::new(10.0, 10.0));
                let native = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output();
                // Node outputs are parent-local; placed Text is in the actual
                // committed world frame. These are direct container children.
                assert_eq!(
                    f64::from(container_origin.x) + f64::from(native.location.x),
                    atomic.rect().origin.x
                );
                assert_eq!(
                    f64::from(container_origin.y) + f64::from(native.location.y),
                    atomic.rect().origin.y
                );
                assert_eq!(native.size, layout::Size::new(10.0, 10.0));
            }
            let control_fragment = batch
                .final_inline_fragments()
                .iter()
                .find(|entry| entry.owner() == &layout::InlineFragmentOwner::Node(2))
                .unwrap();
            let line = control_fragment
                .fragment()
                .finalized_fragment()
                .finalized_line();
            assert_eq!(line.prepared_text(), binding.prepared);
            assert_eq!(line.feedback().advance, 10.0);
            // No glyph run, caller strut, or BR metrics can donate this extent.
            assert_eq!(line.feedback().line_height, line_height);
            assert!(line.selected_line().request().line_strut.is_none());
            assert!(
                line.selected_line()
                    .request()
                    .controls
                    .iter()
                    .all(|allocation| allocation.strut.is_none())
            );
            assert_eq!(
                line.atomic_slots().next().unwrap().occurrence,
                binding.atomics[0].occurrence
            );
            assert_eq!(
                line.control_slots().next().unwrap().control(),
                binding.controls[0].occurrence
            );
            let root = batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output();
            assert_eq!(
                f64::from(if vertical {
                    root.size.width
                } else {
                    root.size.height
                }),
                2.0 * line_height
            );
        }
    }

    #[test]
    fn empty_source_opaque_break_horizontal_ltr_uses_font_owned_lines() {
        assert_empty_source_opaque_break_flow("horizontal-tb", "ltr");
    }

    #[test]
    fn empty_source_opaque_break_horizontal_rtl_uses_font_owned_lines() {
        assert_empty_source_opaque_break_flow("horizontal-tb", "rtl");
    }

    #[test]
    fn empty_source_opaque_break_vertical_lr_ltr_uses_font_owned_lines() {
        assert_empty_source_opaque_break_flow("vertical-lr", "ltr");
    }

    #[test]
    fn empty_source_opaque_break_vertical_lr_rtl_uses_font_owned_lines() {
        assert_empty_source_opaque_break_flow("vertical-lr", "rtl");
    }

    #[test]
    fn empty_source_opaque_break_vertical_rl_ltr_uses_font_owned_lines() {
        assert_empty_source_opaque_break_flow("vertical-rl", "ltr");
    }

    #[test]
    fn empty_source_opaque_break_vertical_rl_rtl_uses_font_owned_lines() {
        assert_empty_source_opaque_break_flow("vertical-rl", "rtl");
    }

    #[test]
    fn empty_source_opaque_break_rejects_wrong_occurrence_order_and_font() {
        let raw = opaque_break_capture("horizontal-tb", "ltr", 30.0);
        let prior = Capture::parse(&raw.to_string(), 3).unwrap();
        let mut wrong_order = raw.clone();
        wrong_order["order"] = json!([
            {"kind":"control","index":0,"affinity":"before"},
            {"kind":"atomic","index":0,"affinity":"before"},
            {"kind":"atomic","index":1,"affinity":"before"}
        ]);
        assert!(Capture::parse(&wrong_order.to_string(), 3).is_err());
        for (field, value) in [("sha256", json!("wrong")), ("loaded", json!(false))] {
            let mut wrong_font = raw.clone();
            wrong_font["font"][field] = value;
            assert!(Capture::parse(&wrong_font.to_string(), 3).is_err());
        }
        assert!(prior.prepared.analyzed_source().source().text().is_empty());
        assert_eq!(
            prior.binding(0, &[1, 2, 3]).unwrap().controls[0].occurrence,
            prior.controls[0].1
        );
        assert_eq!(
            Capture::parse(&raw.to_string(), 3).unwrap().atomics.len(),
            2
        );
    }

    fn source_context_xml(raw: &Value, root_attrs: &str, children: &str) -> String {
        let encoded = raw
            .to_string()
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;");
        format!(
            r#"<test name="source-context" measurement-version="2" use-rounding="false"><viewport width="120px" height="120px"/><input><div display="block" layout-ready-inline-root="true" {root_attrs}><prepared-paragraph capture="{encoded}"/>{children}</div></input><expectations><node/></expectations></test>"#
        )
    }

    #[test]
    fn captured_paragraph_context_cannot_be_replaced_by_xml_container_context() {
        let raw = opaque_break_capture("horizontal-tb", "ltr", 30.0);
        let children = r#"<div source-tag="span" display="inline-block" width="10px" height="10px"/><div source-tag="br" display="inline"/><div source-tag="span" display="inline-block" width="10px" height="10px"/>"#;
        for root_attrs in [
            r#"direction="rtl" writing-mode="horizontal-tb" font-size="20px" line-height="30px""#,
            r#"direction="ltr" writing-mode="vertical-lr" font-size="20px" line-height="30px""#,
            r#"direction="ltr" writing-mode="horizontal-tb" font-size="10px" line-height="30px""#,
            r#"direction="ltr" writing-mode="horizontal-tb" font-size="20px" line-height="20px""#,
        ] {
            assert!(
                super::super::Golden::parse(&source_context_xml(&raw, root_attrs, children))
                    .is_err()
            );
        }
        let valid = source_context_xml(
            &raw,
            r#"direction="ltr" writing-mode="horizontal-tb" font-size="20px" line-height="30px""#,
            children,
        );
        assert!(super::super::Golden::parse(&valid).is_ok());
    }

    #[test]
    fn captured_br_cannot_silently_drop_its_own_font_frame_metrics_or_displacement() {
        let raw = opaque_break_capture("horizontal-tb", "ltr", 30.0);
        let root_attrs =
            r#"direction="ltr" writing-mode="horizontal-tb" font-size="20px" line-height="30px""#;
        for extra in [
            r#"font-family="monospace""#,
            r#"font-size="10px""#,
            r#"line-height="20px""#,
            r#"direction="rtl""#,
            r#"writing-mode="vertical-rl""#,
            r#"vertical-align="top""#,
            r#"clear="both""#,
            r#"position="absolute""#,
            r#"top="1px""#,
            r#"inline-baseline="12px" inline-line-height="16px""#,
        ] {
            let children = format!(
                r#"<div source-tag="span" display="inline-block" width="10px" height="10px"/><div source-tag="br" display="inline" {extra}/><div source-tag="span" display="inline-block" width="10px" height="10px"/>"#
            );
            assert!(
                super::super::Golden::parse(&source_context_xml(&raw, root_attrs, &children))
                    .is_err(),
                "BR context {extra} must remain explicit"
            );
        }
        for br in [
            r#"<div source-tag="br" display="none"/>"#,
            r#"<div source-tag="br" display="inline">unexpected</div>"#,
        ] {
            let children = format!(
                r#"<div source-tag="span" display="inline-block" width="10px" height="10px"/>{br}<div source-tag="span" display="inline-block" width="10px" height="10px"/>"#
            );
            assert!(
                super::super::Golden::parse(&source_context_xml(&raw, root_attrs, &children))
                    .is_err()
            );
        }
        assert!(Capture::parse(&raw.to_string(), 3).is_ok());
    }

    #[test]
    fn vertical_capture_keeps_original_formatting_units_and_real_atomic_break_owners() {
        for mode in ["vertical-lr", "vertical-rl"] {
            for direction in ["ltr", "rtl"] {
                let mut raw = opaque_break_capture(mode, direction, 30.0);
                raw["baseStyle"]["whitespace"] = json!("collapse");
                let own = raw["baseStyle"].clone();
                raw["origins"] = json!([
                    {"child":0,"units":[10,32,32],"start":0,"end":3,"style":own},
                    {"child":4,"units":[10],"start":3,"end":4,"style":own}
                ]);
                raw["connections"] = json!([{"preceding":0,"following":1,"kind":"continuous","precedingWrap":"word","followingWrap":"word","ancestorWrap":"word"}]);
                raw["atomics"][0]["child"] = json!(1);
                raw["atomics"][0]["at"] = json!(3);
                raw["atomics"][0]["width"] = json!(30.0);
                raw["atomics"][0]["height"] = json!(30.0);
                raw["atomics"][1]["child"] = json!(3);
                raw["atomics"][1]["at"] = json!(3);
                raw["atomics"][1]["width"] = json!(30.0);
                raw["atomics"][1]["height"] = json!(16.0);
                raw["controls"][0]["child"] = json!(2);
                raw["controls"][0]["at"] = json!(3);
                raw["order"] = json!([
                    {"kind":"origin","index":0,"affinity":"before"},
                    {"kind":"atomic","index":0,"affinity":"before"},
                    {"kind":"control","index":0,"affinity":"before"},
                    {"kind":"atomic","index":1,"affinity":"before"},
                    {"kind":"origin","index":1,"affinity":"before"}
                ]);
                let root_attrs = format!(
                    r#"direction="{direction}" writing-mode="{mode}" width="80px" font-size="20px" line-height="30px""#
                );
                let children = format!(
                    r#"<text layout-input="prepared-text" occurrence="0"/><div source-tag="span" display="inline-block" direction="{direction}" writing-mode="{mode}" width="30px" height="30px"/><div source-tag="br" display="inline" direction="{direction}" writing-mode="{mode}"/><div source-tag="span" display="inline-block" direction="{direction}" writing-mode="{mode}" width="30px" height="16px"/><text layout-input="prepared-text" occurrence="1"/>"#
                );
                let golden =
                    super::super::Golden::parse(&source_context_xml(&raw, &root_attrs, &children))
                        .unwrap();
                let prepared = &golden.root.prepared.as_ref().unwrap().prepared;
                assert_eq!(
                    prepared.analyzed_source().source().text().code_units(),
                    &[10, 32, 32, 10]
                );
                assert_eq!(
                    prepared.analyzed_source().typography(),
                    text::AnalysisTypography::Mixed
                );
                assert_eq!(prepared.orientation(), text::LineOrientation::Vertical);
                // Preparation retains deleted-source custody units as well as
                // whitespace clusters. Visibility is qualified after placement.
                let origins = prepared.origin_views().collect::<Vec<_>>();
                assert_eq!(origins[0].source().text().code_units(), &[10, 32, 32]);
                assert_eq!(origins[0].local_range(), text::Range::new(0, 3));
                assert_eq!(origins[1].source().text().code_units(), &[10]);
                assert_eq!(origins[1].local_range(), text::Range::new(0, 1));
                let tree = super::super::TestTree::from_golden(&golden.root).unwrap();
                let binding = &tree.paragraphs[0];
                assert_eq!((binding.origins[0].node, binding.origins[1].node), (1, 5));
                assert_eq!((binding.atomics[0].node, binding.atomics[1].node), (2, 4));
                assert_eq!(binding.controls[0].node, 3);
                let flow = layout::LayoutFlowInput::try_new(
                    &tree,
                    0,
                    tree.paragraphs.clone(),
                    layout::WrappingInput::default(),
                    Vec::new(),
                )
                .unwrap();
                let request = layout::LayoutRootRequest::viewport(layout::Size::new(
                    layout::Available::definite(120.0),
                    layout::Available::definite(120.0),
                ))
                .unwrap();
                let batch = layout::compute_layout_with_flow(&flow, 0, request).unwrap();
                let placed = batch.placed_text()[0].layout();
                assert_eq!(placed.source().text().code_units(), &[10, 32, 32, 10]);
                assert!(placed.run_views().all(|run| run.glyphs().next().is_none()));
                assert_eq!(placed.lines().len(), 2);
                let boxes = placed.inline_boxes();
                assert_eq!((boxes[0].id().as_u64(), boxes[0].line()), (1, 0));
                assert_eq!((boxes[1].id().as_u64(), boxes[1].line()), (3, 1));
                assert_eq!(boxes[0].rect().size, text::Size::new(30.0, 30.0));
                assert_eq!(boxes[1].rect().size, text::Size::new(30.0, 16.0));
                let control = batch
                    .final_inline_fragments()
                    .iter()
                    .find(|entry| entry.owner() == &layout::InlineFragmentOwner::Node(3))
                    .unwrap();
                let line = control.fragment().finalized_fragment().finalized_line();
                assert_eq!(
                    line.control_slots().next().unwrap().control(),
                    binding.controls[0].occurrence
                );
                assert_eq!(line.feedback().advance, 30.0);
                assert!(
                    line.selected_line()
                        .request()
                        .controls
                        .iter()
                        .all(|allocation| allocation.strut.is_none())
                );
            }
        }
    }

    #[test]
    fn vertical_capture_refuses_visible_or_preserved_source_and_other_writing_profiles() {
        for (mode, whitespace, units) in [
            ("vertical-lr", "collapse", json!([88])),
            ("vertical-rl", "preserve", json!([10])),
            ("vertical-rl", "collapse", json!([9])),
            ("sideways-rl", "collapse", json!([10])),
        ] {
            let mut raw = original_capture();
            raw["writingMode"] = json!(mode);
            raw["origins"][0]["style"]["whitespace"] = json!(whitespace);
            raw["origins"][0]["units"] = units;
            raw["origins"][0]["end"] = json!(1);
            assert!(Capture::parse(&raw.to_string(), 1).is_err());
        }
        let mut raw = opaque_break_capture("vertical-lr", "ltr", 30.0);
        raw["atomics"][0]["kind"] = json!("out-of-flow");
        assert!(Capture::parse(&raw.to_string(), 3).is_err());
    }

    #[test]
    fn captured_br_refuses_real_box_edges_but_accepts_zero_edges_and_relative_zero_offsets() {
        let raw = opaque_break_capture("horizontal-tb", "ltr", 30.0);
        let root_attrs = r#"direction="ltr" writing-mode="horizontal-tb" font-size="20px" line-height="30px" width="120px""#;
        let fields = [
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
            "border-top",
            "border-right",
            "border-bottom",
            "border-left",
        ];
        for field in fields {
            let children = format!(
                r#"<div source-tag="span" display="inline-block" width="10px" height="10px"/><div source-tag="br" display="inline" {field}="1px"/><div source-tag="span" display="inline-block" width="10px" height="10px"/>"#
            );
            assert!(
                super::super::Golden::parse(&source_context_xml(&raw, root_attrs, &children))
                    .is_err(),
                "real BR {field} must not be discarded"
            );
        }
        let zero_edges = fields
            .iter()
            .map(|field| format!(r#"{field}="0px""#))
            .collect::<Vec<_>>()
            .join(" ");
        let children = format!(
            r#"<div source-tag="span" display="inline-block" width="10px" height="10px"/><div source-tag="br" display="inline" position="relative" top="0px" right="0px" bottom="0px" left="0px" {zero_edges}/><div source-tag="span" display="inline-block" width="10px" height="10px"/>"#
        );
        let golden =
            super::super::Golden::parse(&source_context_xml(&raw, root_attrs, &children)).unwrap();
        let tree = super::super::TestTree::from_golden(&golden.root).unwrap();
        let flow = layout::LayoutFlowInput::try_new(
            &tree,
            0,
            tree.paragraphs.clone(),
            layout::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let request = layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::definite(120.0),
            layout::Available::definite(120.0),
        ))
        .unwrap();
        let batch = layout::compute_layout_with_flow(&flow, 0, request).unwrap();
        assert_eq!(batch.placed_text()[0].layout().lines().len(), 2);
        let control = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| entry.owner() == &layout::InlineFragmentOwner::Node(2))
            .unwrap();
        let line = control.fragment().finalized_fragment().finalized_line();
        assert_eq!(line.feedback().line_height, 30.0);
        assert!(
            line.selected_line()
                .request()
                .controls
                .iter()
                .all(|allocation| allocation.strut.is_none())
        );
    }

    #[test]
    fn vertical_capture_refuses_otherwise_valid_transparent_boundary_controls() {
        let mut raw = original_capture();
        raw["origins"] = json!([]);
        raw["atomics"] = json!([]);
        raw["controls"] = json!([
            {"child":0,"at":0,"kind":"inline-boundary-start"},
            {"child":1,"at":0,"kind":"inline-boundary-end"}
        ]);
        raw["order"] = json!([
            {"kind":"control","index":0,"affinity":"before"},
            {"kind":"control","index":1,"affinity":"before"}
        ]);
        assert!(Capture::parse(&raw.to_string(), 2).is_ok());
        for mode in ["vertical-lr", "vertical-rl"] {
            raw["writingMode"] = json!(mode);
            assert!(Capture::parse(&raw.to_string(), 2).is_err());
        }
    }

    fn opaque_break_observation_golden() -> super::super::Golden {
        let raw = opaque_break_capture("horizontal-tb", "ltr", 30.0);
        let children = r#"<div source-tag="span" display="inline-block" width="10px" height="10px"/><div source-tag="br" display="inline"/><div source-tag="span" display="inline-block" width="10px" height="10px"/>"#;
        let xml = source_context_xml(
            &raw,
            r#"direction="ltr" writing-mode="horizontal-tb" width="120px" font-size="20px" line-height="30px""#,
            children,
        );
        // Ahem20 has ascent16/descent4; LH30 adds5 on each side.
        let expectations = r#"<expectations><node x="0" y="0" width="120" height="60"><node x="0" y="11" width="10" height="10"/><node><browser-control source_index="1" terminal_visual_slot="1" previous_line="same" next_line="later"/><browser-br-font-band x="10" y="5" width="0" height="20"/></node><node x="0" y="41" width="10" height="10"/></node></expectations>"#;
        super::super::Golden::parse(
            &xml.replace("<expectations><node/></expectations>", expectations),
        )
        .unwrap()
    }

    #[test]
    fn source_bound_br_band_retains_geometry_but_refuses_only_the_unavailable_query() {
        let golden = opaque_break_observation_golden();
        let error = super::super::assert_surgeist_matches(&golden)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("unsupported browser BR font-band observation"),
            "{error}"
        );
        assert!(error.contains("x=10, y=5, width=0, height=20"));
        let mut relations = golden.clone();
        relations.expectations.children[1].browser_br_font_band = None;
        super::super::assert_surgeist_matches(&relations).unwrap();
        let mut wrong_geometry = golden.clone();
        wrong_geometry.expectations.children[2].width = Some(11.0);
        let error = super::super::assert_surgeist_matches(&wrong_geometry)
            .unwrap_err()
            .to_string();
        assert!(error.contains("width mismatch"), "{error}");
        assert!(!error.contains("unsupported browser BR font-band observation"));
        let mut wrong_relation = golden.clone();
        wrong_relation.expectations.children[1]
            .browser_control
            .as_mut()
            .unwrap()
            .next_line = super::super::BrowserNeighborLine::Same;
        let error = super::super::assert_surgeist_matches(&wrong_relation)
            .unwrap_err()
            .to_string();
        assert!(error.contains("next neighbor line mismatch"), "{error}");
        assert!(!error.contains("unsupported browser BR font-band observation"));
        let mut misbound = golden.clone();
        let control = misbound.expectations.children[1].clone();
        misbound.expectations.children[0] = control;
        let actual = misbound.expectations.children[0]
            .browser_control
            .as_mut()
            .unwrap();
        actual.source_index = 0;
        actual.terminal_visual_slot = None;
        actual.previous_line = super::super::BrowserNeighborLine::Absent;
        actual.next_line = super::super::BrowserNeighborLine::Unobserved;
        let error = super::super::assert_surgeist_matches(&misbound)
            .unwrap_err()
            .to_string();
        assert!(error.contains("browser control source mismatch"), "{error}");
        assert!(!error.contains("unsupported browser BR font-band observation"));
        super::super::assert_surgeist_matches(&relations).unwrap();
    }
}
