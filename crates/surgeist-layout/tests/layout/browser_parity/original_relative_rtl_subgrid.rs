//! Native comparison of the original Body, edged Grid and positioned Subgrid child.

use super::exact_differences::{
    QualificationError, QualifiedComparison, QualifiedVerdict, RawVerdict,
};
use super::root_context::QualifiedRootContext;
use super::{
    Error, Expectation, Golden, Node, NodeKind, NumericComparisonSink, RootContext,
    compare_captured_with_batch, layout,
};

type Result<T> = std::result::Result<T, QualificationError>;

fn changed(component: &str) -> QualificationError {
    QualificationError::IdentityChanged {
        component: component.into(),
    }
}

fn has_style(node: &Node, fields: &[(&str, &str)]) -> bool {
    fields
        .iter()
        .all(|(key, value)| node.style.get(key) == Some(*value))
}

fn has_box_role(node: &Node, display: &str, direction: &str) -> bool {
    has_style(
        node,
        &[
            ("display", display),
            ("direction", direction),
            ("box-sizing", "content-box"),
            ("font-family", "monospace"),
            ("font-size", "12px"),
            ("line-height", "12px"),
        ],
    ) && node.style.get("writing-mode").unwrap_or("horizontal-tb") == "horizontal-tb"
}

fn text_leaf(node: &Node, expected: &str, direction: &str) -> bool {
    node.kind == NodeKind::Text
        && node.text.as_deref() == Some(expected)
        && node.children.is_empty()
        && node.style.get("source-tag") == Some("div")
        && has_box_role(node, "block", direction)
}

fn complete_rectangles(expected: &Expectation) -> bool {
    [expected.x, expected.y, expected.width, expected.height]
        .into_iter()
        .all(|field| field.is_some_and(|value| value.is_finite()))
        && expected.scroll_size.is_none()
        && expected.fragments.is_none()
        && expected.range_rectangles.is_none()
        && expected.browser_control.is_none()
        && expected.browser_br_font_band.is_none()
        && expected.children.iter().all(complete_rectangles)
}

fn validate(golden: &Golden) -> Result<()> {
    if !golden.use_rounding
        || !complete_rectangles(&golden.expectations)
        || !matches!(
            golden.viewport.root_context,
            RootContext::Qualified(QualifiedRootContext::BlockFlowAuto {
                parent_direction: layout::Direction::Ltr,
                ..
            })
        )
        || golden.has_element_scroll_sizes()
        || golden.has_prepared_range_frame()
        || golden.measured_leaf.is_some()
        || golden.has_flex_content_text()
        || golden.flex_composition_image.is_some()
    {
        return Err(changed("original relative RTL Subgrid capture frame"));
    }
    let body = &golden.root;
    let [grid] = body.children.as_slice() else {
        return Err(changed("original Body to Grid mapping"));
    };
    if body.kind != NodeKind::Div
        || body.text.is_some()
        || !has_box_role(body, "block", "ltr")
        || !has_style(body, &[("source-tag", "body"), ("position", "static")])
        || ["width", "height"]
            .iter()
            .any(|key| body.style.get(key).is_some_and(|value| value != "auto"))
        || [
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
        ]
        .iter()
        .any(|key| body.style.get(key).is_some_and(|value| value != "0px"))
    {
        return Err(changed("original static Auto Body input"));
    }
    if grid.kind != NodeKind::Div
        || grid.text.is_some()
        || !has_box_role(grid, "grid", "ltr")
        || grid.style.get("position").is_some()
        || !has_style(
            grid,
            &[
                ("source-tag", "div"),
                ("width", "500px"),
                ("padding-top", "1px"),
                ("padding-right", "2px"),
                ("padding-bottom", "1px"),
                ("padding-left", "2px"),
                ("border-top", "1px"),
                ("border-right", "1px"),
                ("border-bottom", "1px"),
                ("border-left", "1px"),
                ("grid-template-rows", "repeat(4, auto)"),
                ("grid-template-columns", "repeat(10, 30px)"),
                ("grid-auto-rows", "100px"),
                ("grid-auto-columns", "100px"),
                ("align-content", "start"),
                ("justify-content", "start"),
            ],
        )
    {
        return Err(changed("original edged outer Grid input"));
    }
    let Some((subgrid, digits)) = grid.children.split_last() else {
        return Err(changed("original Grid child census"));
    };
    if digits.len() != 10
        || digits.iter().enumerate().any(|(index, digit)| {
            !text_leaf(digit, &(index + 1).to_string(), "ltr")
                || digit.style.get("grid-row-start") != Some("1")
                || digit.style.get("position").is_some()
        })
    {
        return Err(changed("original fixed-column text inputs"));
    }
    let [positioned] = subgrid.children.as_slice() else {
        return Err(changed("original relative Subgrid child census"));
    };
    if subgrid.kind != NodeKind::Div
        || subgrid.text.is_some()
        || !has_box_role(subgrid, "grid", "rtl")
        // The V4 serializer elides the Native in-flow default after the
        // wire proof verifies the explicit authored relative role.
        || subgrid.style.get("position").unwrap_or("relative") != "relative"
        || !has_style(
            subgrid,
            &[
                ("source-tag", "div"),
                ("grid-template-rows", "auto"),
                ("grid-template-columns", "subgrid"),
                ("grid-column-start", "2"),
                ("grid-column-end", "span 8"),
                ("grid-row-start", "2"),
                ("grid-row-end", "span 2"),
                ("padding-top", "4px"),
                ("padding-right", "33px"),
                ("padding-bottom", "1px"),
                ("padding-left", "10px"),
                ("border-top", "3px"),
                ("border-right", "3px"),
                ("border-bottom", "3px"),
                ("border-left", "3px"),
                ("min-width", "0px"),
                ("min-height", "12px"),
            ],
        )
        || !text_leaf(positioned, "x", "rtl")
        || !has_style(
            positioned,
            &[
                ("position", "absolute"),
                ("grid-column-start", "3"),
                ("grid-column-end", "5"),
                ("top", "0px"),
                ("right", "0px"),
                ("bottom", "0px"),
                ("left", "0px"),
            ],
        )
    {
        return Err(changed(
            "original relative RTL Subgrid and positioned input",
        ));
    }
    Ok(())
}

pub(super) fn compare(
    golden: &Golden,
) -> Result<(
    Vec<String>,
    std::result::Result<QualifiedComparison, String>,
)> {
    validate(golden)?;
    let mut numeric = NumericComparisonSink::CollectNumeric(Vec::new());
    compare_captured_with_batch(golden, &mut numeric, |tree, _| {
        if tree.nodes.len() != 14
            || tree
                .nodes
                .iter()
                .any(|node| !node.unrounded_present || !node.final_layout_present)
        {
            return Err(Error::new(
                "incomplete Native original relative RTL Subgrid layout",
            ));
        }
        Ok(())
    })
    .map_err(|error| QualificationError::ComparisonFailure {
        reason: error.to_string(),
    })?;
    let NumericComparisonSink::CollectNumeric(differences) = numeric else {
        unreachable!("numeric comparison owns its collecting sink");
    };
    let fields: Vec<_> = differences
        .iter()
        .map(|field| format!("{field:?}"))
        .collect();
    let verdict = if fields.is_empty() {
        Ok(QualifiedComparison {
            raw: RawVerdict::Matches,
            qualified: QualifiedVerdict::MatchingControl,
        })
    } else {
        Err(QualificationError::DifferenceSetChanged {
            missing: Vec::new(),
            extra: fields.clone(),
            changed: Vec::new(),
        }
        .to_string())
    };
    Ok((fields, verdict))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT: &str = include_str!(
        "captured/original-relative-rtl-subgrid-positioned/xml/subgrid/subgrid_abspos_relative_rtl_column_3_to_5__border_box_ltr.xml"
    );

    #[test]
    fn original_relative_rtl_subgrid_box_geometry_matches_wider_text_advance_control() {
        let golden = Golden::parse(CURRENT).unwrap();
        assert!(compare(&golden).unwrap().1.is_ok());
        let wider = CURRENT.replace("font-family=\"monospace\"", "font-family=\"ahem\"");
        assert_ne!(CURRENT, wider);
        let wider = Golden::parse(&wider).unwrap();
        super::super::assert_surgeist_matches(&wider).unwrap();
    }

    #[test]
    fn original_relative_rtl_subgrid_refuses_changed_source_placement() {
        for (field, replacement) in [
            ("grid-column-start=\"3\"", "grid-column-start=\"4\""),
            ("position=\"absolute\"", "position=\"relative\""),
            ("top=\"0px\"", "top=\"1px\""),
            ("direction=\"rtl\"", "direction=\"ltr\""),
            ("width=\"500px\"", "width=\"501px\""),
        ] {
            assert!(CURRENT.contains(field));
            let altered = Golden::parse(&CURRENT.replacen(field, replacement, 1)).unwrap();
            assert!(
                validate(&altered).is_err(),
                "{field} accepted as {replacement}"
            );
        }
    }
}
