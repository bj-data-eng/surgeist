//! Explicit, lossless DOM element-size observations from the selected wire tuple.

use super::{
    Context, ElementScrollSize, LayoutInput, MeasurementError, MeasurementErrorKind, WireGeometry,
    WireNode,
};

pub(super) fn unsupported_reason(
    node: &WireNode,
    context: &Context<'_>,
) -> Result<Option<String>, MeasurementError> {
    let unsupported = |reason: &str| Some(format!("{}: {reason}", context.node_path));
    if node.layout_input != LayoutInput::Box || node.tag_name.as_deref() != Some("div") {
        return Ok(unsupported(
            "element scroll sizes require an actual source DIV box",
        ));
    }
    if node.layout_ready_inline_root.is_some()
        || node.layout_ready_anonymous_grid_text_wrapper.is_some()
        || node.prepared_paragraph.is_some()
        || node.atomic_paragraph_metrics.is_some()
        || node.prepared_occurrence.is_some()
        || node.inline_segments.is_some()
        || node.inline_boundary.is_some()
        || node.line_control_participation.is_some()
        || node.atomic_inline_participation.is_some()
        || node.browser_br_observation.is_some()
        || node.range_rectangles.is_some()
        || node.range_fragments.is_some()
    {
        return Ok(unsupported(
            "element scroll sizes exclude synthetic, prepared and control observations",
        ));
    }
    let style = context.required(node.style.as_ref(), "style")?;
    if let Some(reason) = style.element_scroll_size_unsupported_reason() {
        return Ok(unsupported(reason));
    }
    let children = context.required(node.children.as_ref(), "children")?;
    if node.text_content.is_some() && !children.is_empty() {
        return Ok(unsupported(
            "opaque element text must belong to an actual source leaf",
        ));
    }
    for (index, child) in children.iter().enumerate() {
        if let Some(reason) = unsupported_reason(child, &context.child(index))? {
            return Ok(Some(reason));
        }
    }
    Ok(None)
}

pub(super) fn collect(
    root: &WireNode,
    use_rounding: bool,
    context: &Context<'_>,
) -> Result<Vec<ElementScrollSize>, MeasurementError> {
    fn visit(
        node: &WireNode,
        use_rounding: bool,
        path: String,
        context: &Context<'_>,
        rows: &mut Vec<ElementScrollSize>,
    ) -> Result<(), MeasurementError> {
        let (geometry, field) = if use_rounding {
            (node.smart_rounded_layout.as_ref(), "smartRoundedLayout")
        } else {
            (node.unrounded_layout.as_ref(), "unroundedLayout")
        };
        // Keep all four DOM integers from one selected wire object, before
        // layout geometry is converted or formatted through the scalar lane.
        let geometry = context.required(geometry, field)?;
        rows.push(selected_row(geometry, field, path.clone(), context)?);
        for (index, child) in context
            .required(node.children.as_ref(), "children")?
            .iter()
            .enumerate()
        {
            visit(
                child,
                use_rounding,
                format!("{path}.{index}"),
                &context.child(index),
                rows,
            )?;
        }
        Ok(())
    }
    let mut rows = Vec::new();
    visit(root, use_rounding, "0".into(), context, &mut rows)?;
    Ok(rows)
}

fn dom_size(
    value: Option<f64>,
    context: &Context<'_>,
    field: &str,
) -> Result<u32, MeasurementError> {
    let value = context.required(value, field)?;
    context.ensure(
        value.is_finite() && value >= 0.0 && value.fract() == 0.0,
        field,
        "DOM size must be a finite nonnegative integer",
    )?;
    if value > f64::from(i32::MAX) {
        return Err(context.error(
            field,
            MeasurementErrorKind::OutOfRange,
            "DOM size exceeds the nonnegative CSSOM long domain",
        ));
    }
    // The exact f64 integer and domain have been validated before this cast.
    Ok(value as u32)
}

fn selected_row(
    geometry: &WireGeometry,
    field: &str,
    node_path: String,
    context: &Context<'_>,
) -> Result<ElementScrollSize, MeasurementError> {
    let row = ElementScrollSize {
        node_path,
        client_width: dom_size(
            geometry.client_width,
            context,
            &format!("{field}.clientWidth"),
        )?,
        client_height: dom_size(
            geometry.client_height,
            context,
            &format!("{field}.clientHeight"),
        )?,
        scroll_width: dom_size(
            geometry.scroll_width,
            context,
            &format!("{field}.scrollWidth"),
        )?,
        scroll_height: dom_size(
            geometry.scroll_height,
            context,
            &format!("{field}.scrollHeight"),
        )?,
    };
    context.compatible(
        row.client_width <= row.scroll_width,
        &format!("{field}.scrollWidth"),
        "scrollWidth must be at least clientWidth",
    )?;
    context.compatible(
        row.client_height <= row.scroll_height,
        &format!("{field}.scrollHeight"),
        "scrollHeight must be at least clientHeight",
    )?;
    Ok(row)
}

#[cfg(test)]
#[path = "element_scroll_sizes_tests.rs"]
mod tests;
