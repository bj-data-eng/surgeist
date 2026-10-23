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
        if string(&value["writingMode"])? != "horizontal-tb" {
            return Err(fail("unsupported writing mode"));
        }
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
            origin_wraps.push(resolved.wrap);
            source.span(aggregate, resolved.clone());
            origin_children.push(child);
            origins.push(text::ParagraphOrigin {
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
        let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
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
        })
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
            .prepare(&analyzed, text::PreparationOptions::default())
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
            let capture =
                layout::TextLayoutInput::try_new(&tree, 0, tree.paragraphs.clone()).unwrap();
            let request = layout::LayoutRootRequest::viewport(layout::Size::new(
                layout::Available::definite(100.0),
                layout::Available::MaxContent,
            ))
            .unwrap();
            let batch = layout::compute_layout_with_text(&capture, 0, request).unwrap();
            assert_eq!(batch.unrounded_inline_fragments().len(), raw_children.len());
            for entry in batch.unrounded_inline_fragments() {
                let text::FinalizedFragmentOwner::Control(control) =
                    entry.fragment().finalized_fragment().view().owner()
                else {
                    panic!("genuine control must remain Control-owned")
                };
                assert_eq!(
                    control,
                    expected[entry.node() - 1],
                    "placed child must retain its actual event owner, including same-kind same-offset controls"
                );
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
}
