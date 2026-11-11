//! Historical Grid correspondence for captures of the original Flex parent.

use super::exact_differences::{
    QualificationError, QualifiedComparison, QualifiedVerdict, RawVerdict,
};
use super::root_context::QualifiedRootContext;
use super::{
    Error, Expectation, Golden, Node, NumericComparisonSink, RootContext, Scalar,
    compare_captured_with_batch, layout, parse_number,
};

pub(crate) fn verify_historical(current: &Golden, historical: &Golden) -> Result<(), Error> {
    if !matches!(
        &current.viewport.root_context,
        RootContext::Qualified(QualifiedRootContext::AbsoluteRootLocal {
            containing_direction: layout::Direction::Ltr,
            hypothetical_direction: layout::Direction::Ltr,
            hypothetical_display,
            hypothetical_inline_offset: 0.0,
            hypothetical_block_offset: None,
            ..
        }) if hypothetical_display == "flex"
    ) || current.root.style.get("display") != Some("flex")
        || current.root.style.get("position") != Some("absolute")
        || current.name != historical.name
        || !current.use_rounding
        || !historical.use_rounding
        || !matches!(
            &historical.viewport.root_context,
            RootContext::UnqualifiedCapture { measurement_version: Some(version), .. }
                if version == "2"
        )
    {
        return Err(Error::new(
            "original Flex parent historical identity changed",
        ));
    }
    let [grid] = current.root.children.as_slice() else {
        return Err(Error::new("original Flex parent must have one Grid child"));
    };
    let [observed] = current.expectations.children.as_slice() else {
        return Err(Error::new(
            "original Flex parent must observe one Grid child",
        ));
    };
    let (parent_x, parent_y, parent_width, parent_height) = rectangle(&current.expectations)?;
    let (x, y, grid_width, grid_height) = rectangle(observed)?;
    let percentage_parent = historical
        .name
        .split_once("__")
        .is_some_and(|(stem, _)| stem == "chrome_issue_325928327");
    let preferred_size_matches = if percentage_parent {
        current.root.style.get("width") == Some("max-content")
            && parent_width == grid_width
            && parent_height == grid_height
            && original_percentage_grid_shape(grid)
            && original_percentage_grid_shape(&historical.root)
    } else {
        let authored_width = current
            .root
            .style
            .get("width")
            .and_then(|value| value.strip_suffix("px"))
            .ok_or_else(|| Error::new("missing authored Flex parent width"))?;
        parse_number(authored_width)? == parent_width && grid.children.len() == 2
    };
    if !preferred_size_matches
        || current.root.style.get("height") != Some("max-content")
        || current.root.style.get("direction") != grid.style.get("direction")
        || current.root.style.get("box-sizing") != grid.style.get("box-sizing")
        || grid.style.get("display") != Some("grid")
    {
        return Err(Error::new("original Flex parent or Grid sizing changed"));
    }
    let start = match grid.style.get("direction") {
        Some("ltr") => 0.0,
        Some("rtl") => parent_width - grid_width,
        _ => return Err(Error::new("missing original Grid direction")),
    };
    let historical_rectangle = rectangle(&historical.expectations)?;
    if parent_x != 0.0
        || parent_y != 0.0
        || x != start
        || y != 0.0
        || historical_rectangle.0 != 0.0
        || historical_rectangle.1 != 0.0
    {
        return Err(Error::new(
            "original Grid origin disagrees with its containing frame",
        ));
    }
    compare_subtree(
        grid,
        observed,
        &historical.root,
        &historical.expectations,
        true,
        !percentage_parent,
    )
}

fn original_percentage_grid_shape(grid: &Node) -> bool {
    let [item] = grid.children.as_slice() else {
        return false;
    };
    let [ratio_box] = item.children.as_slice() else {
        return false;
    };
    grid.style.get("width") == Some("100%")
        && grid.style.get("height") == Some("40px")
        && grid.style.get("justify-items") == Some("center")
        && item.style.get("height") == Some("100%")
        && ratio_box.style.get("height") == Some("100%")
        && ratio_box.style.get("aspect-ratio") == Some("1")
        && ratio_box.children.is_empty()
}

fn rectangle(node: &Expectation) -> Result<(Scalar, Scalar, Scalar, Scalar), Error> {
    match (node.x, node.y, node.width, node.height) {
        (Some(x), Some(y), Some(width), Some(height))
            if [x, y, width, height].into_iter().all(Scalar::is_finite)
                && width >= 0.0
                && height >= 0.0
                && node.scroll_size.is_none()
                && node.fragments.is_none()
                && node.range_rectangles.is_none()
                && node.browser_control.is_none()
                && node.browser_br_font_band.is_none() =>
        {
            Ok((x, y, width, height))
        }
        _ => Err(Error::new("incomplete original Grid geometry")),
    }
}

fn compare_subtree(
    current: &Node,
    observed: &Expectation,
    historical: &Node,
    expected: &Expectation,
    root: bool,
    has_text_leaves: bool,
) -> Result<(), Error> {
    if current.kind != historical.kind
        || current.text != historical.text
        || current.style.get("display") != historical.style.get("display")
        || current.style.get("direction") != historical.style.get("direction")
        || current.style.get("box-sizing").unwrap_or("border-box")
            != historical.style.get("box-sizing").unwrap_or("border-box")
        || current.children.len() != historical.children.len()
        || observed.children.len() != current.children.len()
        || expected.children.len() != historical.children.len()
    {
        return Err(Error::new("original Grid source subtree changed"));
    }
    if has_text_leaves {
        if current.style.get("font-family") != Some("ahem")
            || current.style.get("font-size") != Some("10px")
            || current.style.get("line-height") != Some("10px")
            || (!root && (!current.children.is_empty() || current.text.is_none()))
        {
            return Err(Error::new("original Grid text or font mapping changed"));
        }
    } else if current.text.is_some() {
        return Err(Error::new(
            "original percentage Grid unexpectedly contains text",
        ));
    }
    let (x, y, width, height) = rectangle(observed)?;
    let origin = if root { (0.0, 0.0) } else { (x, y) };
    if (origin.0, origin.1, width, height) != rectangle(expected)? {
        return Err(Error::new(
            "original Grid geometry differs from its historical observation",
        ));
    }
    for ((current, observed), (historical, expected)) in current
        .children
        .iter()
        .zip(&observed.children)
        .zip(historical.children.iter().zip(&expected.children))
    {
        compare_subtree(
            current,
            observed,
            historical,
            expected,
            false,
            has_text_leaves,
        )?;
    }
    Ok(())
}

/// Report complete rounded box differences and the independently captured
/// unrounded Grid offset within the actual Flex parent.
pub(crate) fn compare(
    golden: &Golden,
) -> Result<(Vec<String>, Result<QualifiedComparison, String>), QualificationError> {
    let failure = |error: Error| QualificationError::ComparisonFailure {
        reason: error.to_string(),
    };
    let [observed] = golden.expectations.children.as_slice() else {
        return Err(failure(Error::new("missing original Grid observation")));
    };
    let (x, y, _, _) = rectangle(observed).map_err(failure)?;
    let mut numeric = NumericComparisonSink::CollectNumeric(Vec::new());
    let mut fields = Vec::new();
    let complete = compare_captured_with_batch(golden, &mut numeric, |tree, _| {
        if tree
            .nodes
            .iter()
            .any(|node| !node.unrounded_present || !node.final_layout_present)
        {
            return Err(Error::new("incomplete Native original Flex parent layout"));
        }
        let [grid] = tree.nodes[0].children.as_slice() else {
            return Err(Error::new("Native Flex parent must have one Grid child"));
        };
        let native = tree.nodes[*grid].unrounded.location;
        for (axis, browser, native) in [("x", x, native.x), ("y", y, native.y)] {
            if browser != native {
                fields.push(format!(
                    "{}/0: unrounded {axis}, browser {browser}, Native {native}",
                    golden.name
                ));
            }
        }
        Ok(())
    })
    .map_err(failure)?;
    if !complete.endpoint_unobservable_fields().is_empty() {
        return Err(failure(Error::new(
            "unobservable original Flex parent endpoint",
        )));
    }
    let NumericComparisonSink::CollectNumeric(mismatches) = numeric else {
        unreachable!("numeric collection owns its collecting sink");
    };
    fields.extend(mismatches.into_iter().map(|field| format!("{field:?}")));
    let verdict = if fields.is_empty() {
        Ok(QualifiedComparison {
            raw: RawVerdict::Matches,
            qualified: QualifiedVerdict::MatchingControl,
        })
    } else {
        Err(format!(
            "original Flex parent has {} raw differences",
            fields.len()
        ))
    };
    Ok((fields, verdict))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDITION: &str = "tests/layout/browser_parity/captured/original-flex-parent-qualification";
    const FIXTURE: &str = "grid/grid_available_space_smaller_than_min_content__border_box_rtl.xml";

    fn pair() -> (Golden, Golden) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(EDITION);
        (
            Golden::parse_file(root.join("xml").join(FIXTURE)).unwrap(),
            Golden::parse_file(root.join("historical").join(FIXTURE)).unwrap(),
        )
    }

    #[test]
    fn historical_correspondence_requires_the_real_parent_and_nested_offsets() {
        let (mut current, mut historical) = pair();
        verify_historical(&current, &historical).unwrap();
        assert_eq!(current.expectations.children[0].x, Some(-20.0));

        historical.expectations.children[0].width = None;
        assert!(verify_historical(&current, &historical).is_err());
        let (_, historical) = pair();
        current.expectations.width = Some(80.0);
        current.expectations.children[0].x = Some(0.0);
        assert!(verify_historical(&current, &historical).is_err());
    }

    #[test]
    fn original_percentage_parent_preserves_historical_grid_geometry() {
        let historical = Golden::parse(include_str!(
            "xml/grid/chrome_issue_325928327__border_box_ltr.xml"
        ))
        .unwrap();
        // Authored test oracle for the original 800×600 frame and 40px boxes.
        let current = Golden::parse(
            r#"<test name="chrome_issue_325928327__border_box_ltr" measurement-version="4" observation-frame="root-border-local" use-rounding="true">
  <viewport root-context="absolute-root-local" containing-width="800px" containing-height="600px" containing-direction="ltr" hypothetical-direction="ltr" hypothetical-display="flex" hypothetical-anchor="inline-start" hypothetical-inline-offset="0px" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
  <input>
    <div grid-alignment-profile="{&quot;profile&quot;:1,&quot;owners&quot;:[]}" source-tag="div" display="flex" direction="ltr" position="absolute" align-items="start" justify-content="start" width="max-content" height="max-content">
      <div source-tag="div" display="grid" direction="ltr" justify-items="center" width="100%" height="40px">
        <div source-tag="div" display="flex" direction="ltr" height="100%">
          <div source-tag="div" display="flex" direction="ltr" height="100%" aspect-ratio="1"/>
        </div>
      </div>
    </div>
  </input>
  <expectations>
    <node x="0" y="0" width="40" height="40">
      <node x="0" y="0" width="40" height="40">
        <node x="0" y="0" width="40" height="40">
          <node x="0" y="0" width="40" height="40"/>
        </node>
      </node>
    </node>
  </expectations>
  <diagnostics>
    <world-rounded-layout node-path="0" x="0" y="0" width="40" height="40" scroll-width="40" scroll-height="40" client-width="40" client-height="40"/>
    <world-rounded-layout node-path="0.0" x="0" y="0" width="40" height="40" scroll-width="40" scroll-height="40" client-width="40" client-height="40"/>
    <world-rounded-layout node-path="0.0.0" x="0" y="0" width="40" height="40" scroll-width="40" scroll-height="40" client-width="40" client-height="40"/>
    <world-rounded-layout node-path="0.0.0.0" x="0" y="0" width="40" height="40" scroll-width="40" scroll-height="40" client-width="40" client-height="40"/>
  </diagnostics>
</test>"#,
        )
        .unwrap();
        let (raw, verdict) = compare(&current).unwrap();
        assert!(raw.is_empty() && verdict.is_ok(), "{raw:?}");
        verify_historical(&current, &historical).unwrap();

        let mut fractional = current;
        let ratio_box = &mut fractional.root.children[0].children[0].children[0];
        ratio_box.style.attrs.insert("width".into(), "40px".into());
        ratio_box
            .style
            .attrs
            .insert("height".into(), "39.75px".into());
        let (raw, verdict) = compare(&fractional).unwrap();
        assert!(verdict.is_err());
        assert_eq!(
            raw,
            [
                "chrome_issue_325928327__border_box_ltr/0/0/0: unrounded height, browser 40, Native 39.75"
            ],
        );
    }

    #[test]
    fn comparison_reports_a_signed_grid_offset_even_when_rounding_can_hide_it() {
        let (mut current, _) = pair();
        let (raw, verdict) = compare(&current).unwrap();
        assert!(raw.is_empty() && verdict.is_ok());
        current.expectations.children[0].x = Some(-19.75);
        let (raw, verdict) = compare(&current).unwrap();
        assert!(verdict.is_err());
        assert!(
            raw.iter()
                .any(|field| field.ends_with("/0: unrounded x, browser -19.75, Native -20"))
        );
    }
}
