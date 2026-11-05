//! Source and metric evidence from the browser's measured Grid leaf.

use serde::Deserialize;

use super::{
    Attributes, Context, ElementScrollSize, MeasurementError, MeasurementErrorKind, WireNode,
};

pub(crate) const FONT_RESOURCE: &str =
    "e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c";
const SOURCE_UNITS: [u16; 9] = [72, 72, 72, 72, 8203, 72, 72, 72, 72];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireMeasuredLeaf {
    node_path: String,
    font_resource_sha256: String,
    source_units: [u16; 9],
    advances: [u32; 3],
    direction: String,
    box_sizing: String,
    client_width: u32,
    scroll_width: u32,
}

impl WireMeasuredLeaf {
    pub(super) fn validate(
        &self,
        root: &WireNode,
        sizes: &[ElementScrollSize],
        context: &Context<'_>,
    ) -> Result<Attributes, MeasurementError> {
        let field = "measuredLeaf";
        let child_index = self
            .node_path
            .strip_prefix("0.")
            .and_then(|suffix| suffix.parse::<usize>().ok())
            .filter(|index| self.node_path == format!("0.{index}"))
            .ok_or_else(|| {
                context.error(
                    field,
                    MeasurementErrorKind::InvalidValue,
                    "expected the path of a direct Grid child",
                )
            })?;
        let leaf = root
            .children
            .as_deref()
            .and_then(|children| children.get(child_index))
            .ok_or_else(|| {
                context.error(
                    field,
                    MeasurementErrorKind::InvalidValue,
                    "observed child is absent",
                )
            })?;
        let style = context.required(leaf.style.as_ref(), "measuredLeaf.style")?;
        let root_style = context.required(root.style.as_ref(), "style")?;
        let (variant_box, variant_direction) =
            context.variant.rsplit_once('_').ok_or_else(|| {
                context.error(
                    field,
                    MeasurementErrorKind::InvalidValue,
                    "unregistered box variant",
                )
            })?;
        context.compatible(
            root_style.display.as_deref() == Some("grid")
                && leaf.children.as_ref().is_some_and(Vec::is_empty)
                && self.direction == variant_direction
                && self.box_sizing == variant_box.replace('_', "-")
                && style.direction.as_deref() == Some(self.direction.as_str())
                && style.box_sizing.as_deref() == Some(self.box_sizing.as_str()),
            field,
            "measured leaf role or captured variant changed",
        )?;
        context.compatible(
            self.source_units == SOURCE_UNITS
                && leaf
                    .text_content
                    .as_ref()
                    .is_some_and(|text| text.encode_utf16().eq(self.source_units)),
            field,
            "measured source does not match the captured leaf",
        )?;
        context.compatible(
            self.font_resource_sha256 == FONT_RESOURCE && self.advances == [40, 40, 80],
            field,
            "measured font resource or advances changed",
        )?;
        context.compatible(
            sizes.iter().any(|row| {
                row.node_path == self.node_path
                    && row.client_width == self.client_width
                    && row.scroll_width == self.scroll_width
            }),
            field,
            "measured leaf widths disagree with its element observations",
        )?;
        let [first, second, full] = self.advances;
        Ok(vec![
            ("profile", "ahem-10px-zwsp".into()),
            ("node-path", self.node_path.clone()),
            ("font-resource-sha256", self.font_resource_sha256.clone()),
            ("first-advance", first.to_string()),
            ("second-advance", second.to_string()),
            ("full-advance", full.to_string()),
        ])
    }
}
