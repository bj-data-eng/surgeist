//! Browser source evidence for a bounded direct Flex text profile.

use serde::Deserialize;

use super::{Attributes, Context, LayoutInput, MeasurementError, MeasurementErrorKind, WireNode};
use crate::fri07_intrinsic_flex_profile as intrinsic;
use crate::fri08_grid_lanes_text_profile as grid_lanes;
use crate::fri08_wrapped_subgrid_profile as wrapped;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireFlexContentText {
    font_resource_sha256: String,
    rows: Vec<WireRow>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireRow {
    child: usize,
    source_units: Vec<u16>,
    direction: String,
}

#[derive(Debug)]
pub(crate) struct FlexContentText {
    pub attrs: Attributes,
    pub rows: Vec<Attributes>,
}

impl WireFlexContentText {
    pub(super) fn validate(
        &self,
        root: &WireNode,
        context: &Context<'_>,
    ) -> Result<FlexContentText, MeasurementError> {
        let field = "flexContentText";
        let root_style = context.required(root.style.as_ref(), "style")?;
        let children = context.required(root.children.as_deref(), "children")?;
        context.compatible(
            self.font_resource_sha256 == crate::pinned_font::AHEM_STYLESHEET_WOFF2_SHA256
                && root.layout_input == LayoutInput::Box
                && root_style.display.as_deref() == Some("flex")
                && root_style.writing_mode() == "horizontal-tb"
                && root.text_content.is_none()
                && !children.is_empty()
                && self.rows.len() == children.len(),
            field,
            "Flex content font or source census changed",
        )?;
        let rows = children
            .iter()
            .zip(&self.rows)
            .enumerate()
            .map(|(index, (child, row))| {
                let style = context.required(child.style.as_ref(), "flexContentText.style")?;
                context.compatible(
                    row.child == index
                        && matches!(row.direction.as_str(), "ltr" | "rtl")
                        && style.direction.as_deref() == Some(row.direction.as_str())
                        && child.layout_input == LayoutInput::Box
                        && style.display.as_deref() == Some("flex")
                        && style.writing_mode() == "horizontal-tb"
                        && style.has_direct_flex_text_style()
                        && child.tag_name.as_deref() == Some("div")
                        && child.children.as_ref().is_some_and(Vec::is_empty)
                        && child.text_content.as_ref().is_some_and(|text| {
                            text.split(' ').all(|word| {
                                !word.is_empty()
                                    && word.bytes().all(|byte| byte.is_ascii_alphabetic())
                            }) && text.encode_utf16().eq(row.source_units.iter().copied())
                        }),
                    field,
                    "Flex content occurrence or typography changed",
                )?;
                Ok(vec![
                    ("child", index.to_string()),
                    ("direction", row.direction.clone()),
                ])
            })
            .collect::<Result<_, _>>()?;
        Ok(FlexContentText {
            attrs: vec![
                ("profile", "ahem-10px-flex-direct-text".into()),
                ("font-resource-sha256", self.font_resource_sha256.clone()),
            ],
            rows,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireIntrinsicFlexText {
    font_resource_sha256: String,
    items: Vec<WireIntrinsicFlexItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireIntrinsicFlexItem {
    row: usize,
    item: usize,
    direction: String,
    source_units: Vec<u16>,
    words: Vec<intrinsic::WordRange>,
}

impl WireIntrinsicFlexText {
    pub(super) fn validate(
        &self,
        root: &WireNode,
        context: &Context<'_>,
    ) -> Result<FlexContentText, MeasurementError> {
        const FIELD: &str = "fri07IntrinsicFlexText";
        let style = context.required(root.style.as_ref(), "style")?;
        let root_children = context.required(root.children.as_deref(), "children")?;
        context.compatible(
            self.font_resource_sha256 == crate::pinned_font::AHEM_STYLESHEET_WOFF2_SHA256
                && root.layout_input == LayoutInput::Box
                && root.tag_name.as_deref() == Some("div")
                && style.display.as_deref() == Some("flex")
                && style.flex_direction.as_deref() == Some("column")
                && style.writing_mode() == "horizontal-tb"
                && root.text_content.is_none()
                && root_children.len() == intrinsic::LABELS.len()
                && self.items.len() == intrinsic::LABELS.len() * 2,
            FIELD,
            "intrinsic Flex source root, font or occurrence census changed",
        )?;

        let mut rows = Vec::new();
        for (row_index, (row, label)) in root_children.iter().zip(intrinsic::LABELS).enumerate() {
            let row_context = context.child(row_index);
            let style = row_context.required(row.style.as_ref(), "style")?;
            let children = row_context.required(row.children.as_deref(), "children")?;
            context.compatible(
                row.layout_input == LayoutInput::Box
                    && row.tag_name.as_deref() == Some("div")
                    && style.display.as_deref() == Some("flex")
                    && matches!(style.flex_direction.as_deref(), None | Some("row"))
                    && style.writing_mode() == "horizontal-tb"
                    && row.text_content.is_none()
                    && children.len() == 2,
                FIELD,
                "original Flex row topology changed",
            )?;
            for (item_index, item) in children.iter().enumerate() {
                let witness = &self.items[row_index * 2 + item_index];
                let item_context = row_context.child(item_index);
                let style = item_context.required(item.style.as_ref(), "style")?;
                item_context.compatible(
                    (witness.row, witness.item) == (row_index, item_index)
                        && matches!(witness.direction.as_str(), "ltr" | "rtl")
                        && style.direction.as_deref() == Some(witness.direction.as_str())
                        && item.layout_input == LayoutInput::Box
                        && style.display.as_deref() == Some("flex")
                        && style.writing_mode() == "horizontal-tb"
                        && style.has_intrinsic_flex_text_style(item_index)
                        && item.tag_name.as_deref() == Some("div")
                        && item.children.as_ref().is_some_and(Vec::is_empty)
                        && item.text_content.as_deref() == Some(label)
                        && label
                            .encode_utf16()
                            .eq(witness.source_units.iter().copied())
                        && intrinsic::valid_words(label, &witness.words),
                    FIELD,
                    "original Flex item, UTF-16 label, typography or word Range changed",
                )?;
                let words = serde_json::to_string(&witness.words).map_err(|error| {
                    item_context.error(FIELD, MeasurementErrorKind::InvalidValue, error.to_string())
                })?;
                rows.push(vec![
                    ("row", row_index.to_string()),
                    ("item", item_index.to_string()),
                    ("direction", witness.direction.clone()),
                    ("words", words),
                ]);
            }
        }
        Ok(FlexContentText {
            attrs: vec![
                ("profile", intrinsic::PROFILE.into()),
                ("font-resource-sha256", self.font_resource_sha256.clone()),
            ],
            rows,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireWrappedSubgrid {
    font_resource_sha256: String,
    items: Vec<WireWrappedItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireWrappedItem {
    card: usize,
    item: usize,
    direction: String,
    source_units: Vec<u16>,
    ranges: Vec<intrinsic::WordRange>,
    clip: Option<[u32; 4]>,
}

impl WireWrappedSubgrid {
    pub(super) fn validate(
        &self,
        root: &WireNode,
        context: &Context<'_>,
    ) -> Result<FlexContentText, MeasurementError> {
        const FIELD: &str = "fri08WrappedSubgrid";
        let style = context.required(root.style.as_ref(), "style")?;
        let cards = context.required(root.children.as_deref(), "children")?;
        context.compatible(
            self.font_resource_sha256 == crate::pinned_font::AHEM_STYLESHEET_WOFF2_SHA256
                && root.layout_input == LayoutInput::Box
                && root.tag_name.as_deref() == Some("div")
                && style.display.as_deref() == Some("grid")
                && style.writing_mode() == "horizontal-tb"
                && root.text_content.is_none()
                && cards.len() == 3
                && self.items.len() == cards.len() * wrapped::LABELS.len(),
            FIELD,
            "wrapped Subgrid root, font or occurrence census changed",
        )?;
        let mut rows = Vec::new();
        for (index, card) in cards.iter().enumerate() {
            let card_context = context.child(index);
            let children = subgrid_children(card, &card_context, 2)?;
            let footer_context = card_context.child(1);
            let footer = subgrid_children(&children[1], &footer_context, 1)?;
            for (item, (node, label)) in [&children[0], &footer[0]]
                .into_iter()
                .zip(wrapped::LABELS)
                .enumerate()
            {
                let witness = &self.items[index * wrapped::LABELS.len() + item];
                let item_context = if item == 0 {
                    card_context.child(0)
                } else {
                    footer_context.child(0)
                };
                let style = item_context.required(node.style.as_ref(), "style")?;
                let expected_overflow = if item == 0 { "visible" } else { "hidden" };
                item_context.compatible(
                    (witness.card, witness.item) == (index, item)
                        && matches!(witness.direction.as_str(), "ltr" | "rtl")
                        && style.direction.as_deref() == Some(witness.direction.as_str())
                        && style.overflow_x.as_deref().unwrap_or("visible") == expected_overflow
                        && style.overflow_y.as_deref().unwrap_or("visible") == expected_overflow
                        && node.layout_input == LayoutInput::Box
                        && style.display.as_deref() == Some("flex")
                        && style.writing_mode() == "horizontal-tb"
                        && style.has_direct_flex_text_style()
                        && node.tag_name.as_deref() == Some("div")
                        && node.children.as_ref().is_some_and(Vec::is_empty)
                        && node.text_content.as_deref() == Some(label)
                        && label
                            .encode_utf16()
                            .eq(witness.source_units.iter().copied())
                        && wrapped::valid_ranges(label, &witness.ranges)
                        && match witness.clip {
                            None => item == 0,
                            Some(clip) => item == 1 && wrapped::valid_clip(clip),
                        },
                    FIELD,
                    "wrapped source, typography, Range or clip observation changed",
                )?;
                let ranges = serde_json::to_string(&witness.ranges).map_err(|error| {
                    item_context.error(FIELD, MeasurementErrorKind::InvalidValue, error.to_string())
                })?;
                let mut attrs = vec![
                    ("card", index.to_string()),
                    ("item", item.to_string()),
                    ("direction", witness.direction.clone()),
                    ("ranges", ranges),
                ];
                if let Some(clip) = witness.clip {
                    let clip = serde_json::to_string(&clip).map_err(|error| {
                        item_context.error(
                            FIELD,
                            MeasurementErrorKind::InvalidValue,
                            error.to_string(),
                        )
                    })?;
                    attrs.push(("clip", clip));
                }
                rows.push(attrs);
            }
        }
        Ok(FlexContentText {
            attrs: vec![
                ("profile", wrapped::PROFILE.into()),
                ("font-resource-sha256", self.font_resource_sha256.clone()),
            ],
            rows,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireGridLanesText {
    row: usize,
    item: usize,
    direction: String,
    source_units: Vec<u16>,
    sizes: grid_lanes::Sizes,
}

pub(super) fn validate_grid_lanes(
    witnesses: &[WireGridLanesText],
    root: &WireNode,
    context: &Context<'_>,
) -> Result<FlexContentText, MeasurementError> {
    const FIELD: &str = "fri08GridLanesText";
    let style = context.required(root.style.as_ref(), "style")?;
    let groups = context.required(root.children.as_deref(), "children")?;
    context.compatible(
        root.layout_input == LayoutInput::Box
            && root.tag_name.as_deref() == Some("div")
            && style.display.as_deref() == Some("block")
            && style.writing_mode() == "horizontal-tb"
            && root.text_content.is_none()
            && groups.len() == grid_lanes::WIDTHS.len()
            && witnesses.len() == groups.len() * grid_lanes::LABELS.len(),
        FIELD,
        "original Grid Lanes text census changed",
    )?;
    let mut rows = Vec::with_capacity(witnesses.len());
    for (row, (group, width)) in groups.iter().zip(grid_lanes::WIDTHS).enumerate() {
        let group_context = context.child(row);
        let style = group_context.required(group.style.as_ref(), "style")?;
        let children = group_context.required(group.children.as_deref(), "children")?;
        group_context.compatible(
            group.layout_input == LayoutInput::Box
                && group.tag_name.as_deref() == Some("div")
                && style.display.as_deref() == Some("grid-lanes")
                && style.has_grid_lanes_text_style(width)
                && group.text_content.is_none()
                && children.len() == grid_lanes::LABELS.len(),
            FIELD,
            "original Grid Lanes sizing group changed",
        )?;
        for (item, (node, label)) in children.iter().zip(grid_lanes::LABELS).enumerate() {
            let item_context = group_context.child(item);
            let style = item_context.required(node.style.as_ref(), "style")?;
            let witness = &witnesses[row * grid_lanes::LABELS.len() + item];
            item_context.compatible(
                (witness.row, witness.item) == (row, item)
                    && matches!(witness.direction.as_str(), "ltr" | "rtl")
                    && style.direction.as_deref() == Some(witness.direction.as_str())
                    && style.display.as_deref() == Some("flex")
                    && style.writing_mode() == "horizontal-tb"
                    && style.has_grid_lanes_text_style(width)
                    && node.layout_input == LayoutInput::Box
                    && node.tag_name.as_deref() == Some("div")
                    && node.children.as_ref().is_some_and(Vec::is_empty)
                    && node.text_content.as_deref() == Some(label)
                    && label
                        .encode_utf16()
                        .eq(witness.source_units.iter().copied()),
                FIELD,
                "original Flex text changed",
            )?;
            if !witness.sizes.valid() {
                return Err(item_context.error(
                    FIELD,
                    MeasurementErrorKind::ContradictoryFields,
                    format!("independent text sizing changed: {:?}", witness.sizes),
                ));
            }
            let sizes = serde_json::to_string(&witness.sizes).map_err(|error| {
                item_context.error(FIELD, MeasurementErrorKind::InvalidValue, error.to_string())
            })?;
            rows.push(vec![
                ("row", row.to_string()),
                ("item", item.to_string()),
                ("direction", witness.direction.clone()),
                ("sizes", sizes),
            ]);
        }
    }
    Ok(FlexContentText {
        attrs: vec![("profile", grid_lanes::PROFILE.into())],
        rows,
    })
}

fn subgrid_children<'a>(
    node: &'a WireNode,
    context: &Context<'_>,
    count: usize,
) -> Result<&'a [WireNode], MeasurementError> {
    let children = context.required(node.children.as_deref(), "children")?;
    let style = context.required(node.style.as_ref(), "style")?;
    context.compatible(
        node.layout_input == LayoutInput::Box
            && node.tag_name.as_deref() == Some("div")
            && style.display.as_deref() == Some("grid")
            && style.writing_mode() == "horizontal-tb"
            && node.text_content.is_none()
            && children.len() == count,
        "children",
        "original nested Subgrid topology changed",
    )?;
    Ok(children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn original_intrinsic_flex_capture_uses_its_own_request() {
        const REQUEST: &[u8] =
            b"{\"schema_version\":1,\"text_source\":\"ahem-10px-fri07-intrinsic-flex-text\"}\n";
        let request = super::super::BoxObservationRequest::from_input_bytes(Some(REQUEST))
            .expect("the original nested Flex source has an exact Text capture request");
        assert_eq!(request.input_bytes(), Some(REQUEST));
        assert_eq!(
            request.proof_script().map(|(path, _)| path),
            Some("scripts/gentest/fri07_intrinsic_flex_text.js")
        );
    }

    #[test]
    fn flex_content_proof_binds_every_text_occurrence_and_style() {
        let wire = json!({
            "layoutInput": "box",
            "style": {"display": "flex"},
            "children": [{
                "layoutInput": "box", "tagName": "div", "children": [],
                "textContent": "A B",
                "style": {
                    "display": "flex", "direction": "rtl", "fontFamily": "ahem",
                    "fontSize": {"unit": "px", "value": 10},
                    "lineHeight": {"unit": "px", "value": 10}
                }
            }]
        });
        let witness = json!({
            "fontResourceSha256": crate::pinned_font::AHEM_STYLESHEET_WOFF2_SHA256,
            "rows": [{"child": 0, "sourceUnits": [65, 32, 66], "direction": "rtl"}]
        });
        let context = Context {
            case_id: "flex-source",
            variant: "border_box_rtl",
            node_path: "root".into(),
        };
        let root: WireNode = serde_json::from_value(wire.clone()).unwrap();
        let proof: WireFlexContentText = serde_json::from_value(witness.clone()).unwrap();
        assert_eq!(
            proof.validate(&root, &context).unwrap().rows,
            [vec![("child", "0".into()), ("direction", "rtl".into())]]
        );
        for (pointer, value) in [
            ("/fontResourceSha256", json!("foreign")),
            ("/rows", json!([])),
            ("/rows/0/child", json!(1)),
            ("/rows/0/sourceUnits", json!([65, 66])),
            ("/rows/0/direction", json!("ltr")),
        ] {
            let mut changed = witness.clone();
            *changed.pointer_mut(pointer).unwrap() = value;
            let changed: WireFlexContentText = serde_json::from_value(changed).unwrap();
            assert!(changed.validate(&root, &context).is_err(), "{pointer}");
        }
        let mut changed = wire;
        changed["children"][0]["style"]["lineHeight"]["value"] = json!(20);
        let changed = serde_json::from_value(changed).unwrap();
        assert!(proof.validate(&changed, &context).is_err());
    }
}
