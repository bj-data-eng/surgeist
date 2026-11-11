//! Browser source evidence for a bounded direct Flex text profile.

use serde::Deserialize;

use super::{Attributes, Context, LayoutInput, MeasurementError, WireNode};

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
