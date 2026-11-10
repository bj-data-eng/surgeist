//! Portable qualification of the original static body and its in-flow Grid.

use super::capture_authority::{PathDigest, SpecializedEvidence, sha256};
use super::exact_differences::{
    QualificationError, QualifiedComparison, QualifiedVerdict, RawVerdict,
};
use super::layout::Direction;
use super::root_context::QualifiedRootContext;
use super::{
    Error, Expectation, Golden, Node, NodeKind, NumericComparisonSink, RootContext, Scalar,
    compare_captured_with_batch,
};

use crate::original_subgrid_body_profile as profile;
pub(crate) use profile::{REQUEST, SCRIPT_PATH as PROOF_SCRIPT_PATH, SOURCE, SOURCE_SHA256};
pub(crate) const PROOF_SCRIPT: &[u8] = profile::SCRIPT.as_bytes();

const STEM: &str = "subgrid_gap_percentage_border_bottom";
const GRID_BORDERS: [&str; 4] = ["border-top", "border-right", "border-bottom", "border-left"];

// Preorder of the actual source boxes below the body. Six explicit row
// items precede the nested subgrid and its four two-box items.
const GRID_SHAPE: [(&str, usize); 16] = [
    ("grid", 7),
    ("block", 0),
    ("block", 0),
    ("block", 0),
    ("block", 0),
    ("block", 0),
    ("block", 0),
    ("grid", 4),
    ("block", 1),
    ("block", 0),
    ("block", 1),
    ("block", 0),
    ("block", 1),
    ("block", 0),
    ("block", 1),
    ("block", 0),
];

type Result<T> = std::result::Result<T, QualificationError>;

pub(crate) fn evidence() -> SpecializedEvidence {
    SpecializedEvidence::OriginalSubgridBody {
        proof_script: PathDigest {
            path: PROOF_SCRIPT_PATH.into(),
            sha256: sha256(PROOF_SCRIPT),
        },
    }
}

fn changed(component: impl Into<String>) -> QualificationError {
    QualificationError::IdentityChanged {
        component: component.into(),
    }
}

fn variant_styles(variant: &str) -> Result<(&'static str, &'static str)> {
    match variant {
        "border_box_ltr" => Ok(("ltr", "border-box")),
        "content_box_ltr" => Ok(("ltr", "content-box")),
        "border_box_rtl" => Ok(("rtl", "border-box")),
        "content_box_rtl" => Ok(("rtl", "content-box")),
        _ => Err(changed("original subgrid body variant")),
    }
}

fn rectangle(expected: &Expectation, path: &str) -> Result<[Scalar; 4]> {
    let fields = [expected.x, expected.y, expected.width, expected.height];
    let [Some(x), Some(y), Some(width), Some(height)] = fields else {
        return Err(QualificationError::IncompleteRepresentation {
            key: format!("{path}: expected x, y, width and height"),
        });
    };
    if ![x, y, width, height].into_iter().all(Scalar::is_finite)
        || width < 0.0
        || height < 0.0
        || expected.scroll_size.is_some()
        || expected.fragments.is_some()
        || expected.range_rectangles.is_some()
        || expected.browser_control.is_some()
        || expected.browser_br_font_band.is_some()
    {
        return Err(changed(format!("{path}: invalid original box observation")));
    }
    Ok([x, y, width, height])
}

fn validate_grid(
    grid: &Node,
    expected: &Expectation,
    direction: &str,
    box_sizing: &str,
    path: &str,
) -> Result<()> {
    fn visit(
        node: &Node,
        expected: &Expectation,
        direction: &str,
        box_sizing: &str,
        path: &str,
        shape: &mut std::slice::Iter<'_, (&str, usize)>,
    ) -> Result<()> {
        let Some(&(display, children)) = shape.next() else {
            return Err(changed(format!("{path}: unexpected original Grid box")));
        };
        let style = &node.style;
        if node.kind != NodeKind::Div
            || node.text.is_some()
            || style.get("source-tag") != Some("div")
            || style.get("display") != Some(display)
            || style.get("direction") != Some(direction)
            // The fixture schema elides its Native border-box default.
            || style.get("box-sizing").unwrap_or("border-box") != box_sizing
            || style.get("writing-mode").unwrap_or("horizontal-tb") != "horizontal-tb"
            || style.get("position").is_some()
            || node.children.len() != children
            || expected.children.len() != children
        {
            return Err(changed(format!(
                "{path}: original Grid source mapping or style"
            )));
        }
        rectangle(expected, path)?;
        for (index, (child, expected)) in node.children.iter().zip(&expected.children).enumerate() {
            visit(
                child,
                expected,
                direction,
                box_sizing,
                &format!("{path}/{index}"),
                shape,
            )?;
        }
        Ok(())
    }

    if GRID_BORDERS
        .iter()
        .any(|field| grid.style.get(field) != Some("1px"))
    {
        return Err(changed("original Grid border"));
    }
    if has_definite_preferred_size(grid) {
        return Err(changed("original Auto Grid input"));
    }
    visit(
        grid,
        expected,
        direction,
        box_sizing,
        path,
        &mut GRID_SHAPE.iter(),
    )
}

fn has_definite_preferred_size(node: &Node) -> bool {
    ["width", "height"]
        .iter()
        .any(|field| node.style.get(field).is_some_and(|value| value != "auto"))
}

pub(crate) fn validate(golden: &Golden, variant: &str) -> Result<()> {
    let (direction, box_sizing) = variant_styles(variant)?;
    if golden.name != format!("{STEM}__{variant}")
        || !golden.use_rounding
        || !matches!(
            golden.viewport.root_context,
            RootContext::Qualified(QualifiedRootContext::BlockFlowAuto {
                parent_direction: Direction::Ltr,
                ..
            })
        )
        || golden.has_prepared_range_frame()
        || golden.has_element_scroll_sizes()
        || golden.measured_leaf.is_some()
        || golden.has_flex_content_text()
        || golden.flex_composition_image.is_some()
    {
        return Err(changed("original subgrid body capture frame"));
    }
    let body = &golden.root;
    let style = &body.style;
    if body.kind != NodeKind::Div
        || body.text.is_some()
        || style.get("source-tag") != Some("body")
        || style.get("display") != Some("block")
        || style.get("position") != Some("static")
        || style.get("direction") != Some(direction)
        || style.get("box-sizing") != Some("content-box")
        || style.get("writing-mode").unwrap_or("horizontal-tb") != "horizontal-tb"
        || has_definite_preferred_size(body)
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
        .any(|field| style.get(field).is_some_and(|value| value != "0px"))
    {
        return Err(changed("original static Auto body input"));
    }
    let ([grid], [expected_grid]) = (&body.children[..], &golden.expectations.children[..]) else {
        return Err(changed("original body must have its sole Grid child"));
    };
    rectangle(&golden.expectations, "/")?;
    validate_grid(grid, expected_grid, direction, box_sizing, "/0")
}

pub(crate) fn compare(
    golden: &Golden,
    variant: &str,
) -> Result<(Vec<String>, Result<QualifiedComparison>)> {
    validate(golden, variant)?;
    let mut numeric = NumericComparisonSink::CollectNumeric(Vec::new());
    let mut fields = Vec::new();
    compare_captured_with_batch(golden, &mut numeric, |tree, _| {
        if tree.nodes.len() != GRID_SHAPE.len() + 1
            || tree
                .nodes
                .iter()
                .any(|node| !node.unrounded_present || !node.final_layout_present)
        {
            return Err(Error::new("incomplete Native original subgrid body layout"));
        }
        let [grid] = tree.nodes[0].children.as_slice() else {
            return Err(Error::new("Native original body has no sole Grid child"));
        };
        // The authenticated browser proof checks these exact unrounded child
        // coordinates. Keep them separate from the rounded box comparison.
        let location = tree.nodes[*grid].unrounded.location;
        for (axis, value) in [("x", location.x), ("y", location.y)] {
            if value != 0.0 {
                fields.push(format!(
                    "{}/0: unrounded {axis}, browser 0, Native {value}",
                    golden.name
                ));
            }
        }
        Ok(())
    })
    .map_err(|error| QualificationError::ComparisonFailure {
        reason: error.to_string(),
    })?;
    let NumericComparisonSink::CollectNumeric(mismatches) = numeric else {
        unreachable!("numeric collection owns its collecting sink");
    };
    fields.extend(mismatches.into_iter().map(|field| format!("{field:?}")));
    let qualified = if fields.is_empty() {
        Ok(QualifiedComparison {
            raw: RawVerdict::Matches,
            qualified: QualifiedVerdict::MatchingControl,
        })
    } else {
        Err(QualificationError::DifferenceSetChanged {
            missing: Vec::new(),
            extra: fields.clone(),
            changed: Vec::new(),
        })
    };
    Ok((fields, qualified))
}

pub(crate) fn verify_historical(golden: &Golden, variant: &str, historical: &Golden) -> Result<()> {
    validate(golden, variant)?;
    if historical.name != golden.name
        || !historical.use_rounding
        || !matches!(
            &historical.viewport.root_context,
            RootContext::UnqualifiedCapture { measurement_version: Some(version), .. }
                if version == "2"
        )
    {
        return Err(changed("historical original subgrid frame"));
    }
    validate_grid(
        &historical.root,
        &historical.expectations,
        "ltr",
        "content-box",
        "/",
    )?;

    fn visit(
        current: &Expectation,
        historical: &Expectation,
        rtl_parent_width: Option<Scalar>,
        rtl: bool,
        path: &str,
        changed_fields: &mut Vec<String>,
    ) -> Result<()> {
        let [x, y, width, height] = rectangle(current, path)?;
        let inline_start = rtl_parent_width.map_or(x, |parent| parent - x - width);
        let historical_rect = rectangle(historical, path)?;
        for ((field, current), historical) in ["x", "y", "width", "height"]
            .into_iter()
            .zip([inline_start, y, width, height])
            .zip(historical_rect)
        {
            if current != historical {
                changed_fields.push(format!(
                    "{path} {field}: historical {historical}, fresh {current}"
                ));
            }
        }
        for (index, (child, historical_child)) in current
            .children
            .iter()
            .zip(&historical.children)
            .enumerate()
        {
            visit(
                child,
                historical_child,
                rtl.then_some(width),
                rtl,
                &format!("{path}/{index}"),
                changed_fields,
            )?;
        }
        Ok(())
    }

    // The historical boxes are all LTR/content-box, regardless of filename.
    // The fresh Grid's parent is the actual body and its placement remains in
    // the physical frame. Only descendants use logical inline-start for this
    // independent correspondence check against the historical observations.
    let (direction, _) = variant_styles(variant)?;
    let mut changed_fields = Vec::new();
    visit(
        &golden.expectations.children[0],
        &historical.expectations,
        None,
        direction == "rtl",
        "/0",
        &mut changed_fields,
    )?;
    if !changed_fields.is_empty() {
        return Err(QualificationError::DifferenceSetChanged {
            missing: Vec::new(),
            extra: Vec::new(),
            changed: changed_fields,
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "original_subgrid_body_tests.rs"]
mod tests;
