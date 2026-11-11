//! Browser root capture is distinct from historical source/expectation parsing.

use super::{Error, Scalar, layout, parse_direction, parse_number};

#[derive(Clone, Debug, PartialEq)]
pub enum RootContext {
    Qualified(QualifiedRootContext),
    /// Exact historical metadata retained without issuing a sizing operation.
    UnqualifiedCapture {
        measurement_version: Option<String>,
        raw_viewport: String,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum QualifiedRootContext {
    AbsoluteRootLocal {
        containing_size: layout::Size<layout::NonNegativeFiniteOf>,
        containing_direction: layout::Direction,
        hypothetical_direction: layout::Direction,
        hypothetical_display: String,
        hypothetical_inline_offset: Scalar,
        hypothetical_block_offset: Option<Scalar>,
        fallback: RootFallback,
    },
    BlockFlowAuto {
        parent_inline_extent: layout::NonNegativeFiniteOf,
        parent_direction: layout::Direction,
        fallback: RootFallback,
    },
    InlineGrid {
        parent_inline_extent: layout::NonNegativeFiniteOf,
        fallback: RootFallback,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct RootFallback {
    initial_containing_block: layout::Size<layout::NonNegativeFiniteOf>,
    // The source-qualified profile has no intervening element scrollport.
    // Do not collapse this absence and the ICB into a "nearest qualifying" frame.
}

fn extent(node: roxmltree::Node<'_, '_>, name: &str) -> Result<layout::NonNegativeFiniteOf, Error> {
    let raw = node
        .attribute(name)
        .ok_or_else(|| Error::new(format!("missing root-context field {name}")))?;
    let value: Scalar = parse_number(
        raw.strip_suffix("px")
            .ok_or_else(|| Error::new(format!("{name} requires a CSS-pixel extent")))?,
    )?;
    layout::NonNegativeFiniteOf::new(value)
        .map_err(|error| Error::new(format!("invalid {name}: {error:?}")))
}

pub(crate) fn parse_root_context(
    node: roxmltree::Node<'_, '_>,
    version: Option<&str>,
) -> Result<RootContext, Error> {
    match version {
        None | Some("1" | "2" | "3") => {
            let raw = &node.document().input_text()[node.range()];
            return Ok(RootContext::UnqualifiedCapture {
                measurement_version: version.map(str::to_owned),
                raw_viewport: raw.to_owned(),
            });
        }
        Some("4") => {}
        Some(other) => {
            return Err(Error::new(format!(
                "unsupported root-capture version {other}"
            )));
        }
    }
    let kind = node
        .attribute("root-context")
        .ok_or_else(|| Error::new("missing root-context authority"))?;
    let specific = match kind {
        "absolute-root-local" => &[
            "containing-width",
            "containing-height",
            "containing-direction",
            "hypothetical-direction",
            "hypothetical-display",
            "hypothetical-anchor",
            "hypothetical-inline-offset",
            "hypothetical-block-offset",
        ][..],
        "block-flow-auto" | "inline-grid" => &["parent-inline-extent", "parent-direction"][..],
        _ => return Err(Error::new(format!("unsupported root context {kind}"))),
    };
    for attribute in node.attributes() {
        if ![
            "root-context",
            "fallback-icb-width",
            "fallback-icb-height",
            "fallback-nearest-scrollport",
            "fallback-containing-width-limit",
            "fallback-containing-height-limit",
        ]
        .contains(&attribute.name())
            && !specific.contains(&attribute.name())
        {
            return Err(Error::new(format!(
                "unexpected root-context field {}",
                attribute.name()
            )));
        }
    }
    if node.attribute("fallback-nearest-scrollport") != Some("none") {
        return Err(Error::new(
            "missing or unsupported nearest-scrollport source authority",
        ));
    }
    if node.attribute("fallback-containing-width-limit") != Some("unbounded")
        || node.attribute("fallback-containing-height-limit") != Some("unbounded")
    {
        return Err(Error::new(
            "missing or unsupported containing-limit source authority",
        ));
    }
    let icb = layout::Size::new(
        extent(node, "fallback-icb-width")?,
        extent(node, "fallback-icb-height")?,
    );
    let fallback = RootFallback {
        initial_containing_block: icb,
    };
    let direction = |field| {
        node.attribute(field)
            .ok_or_else(|| Error::new(format!("missing root-context field {field}")))
            .and_then(parse_direction)
    };
    let qualified = match kind {
        "absolute-root-local" => {
            let containing_size = layout::Size::new(
                extent(node, "containing-width")?,
                extent(node, "containing-height")?,
            );
            if containing_size != icb {
                return Err(Error::new(
                    "positioned root does not name its authenticated ICB",
                ));
            }
            let hypothetical_direction = direction("hypothetical-direction")?;
            let hypothetical_display = node
                .attribute("hypothetical-display")
                .filter(|value| {
                    [
                        "block",
                        "flow-root",
                        "flex",
                        "inline-flex",
                        "grid",
                        "inline-grid",
                        "grid-lanes",
                    ]
                    .contains(value)
                })
                .ok_or_else(|| Error::new("missing or unsupported hypothetical outer display"))?
                .to_owned();
            if node.attribute("hypothetical-anchor") != Some("inline-start") {
                return Err(Error::new("missing or unsupported hypothetical anchor"));
            }
            let raw = node
                .attribute("hypothetical-inline-offset")
                .and_then(|value| value.strip_suffix("px"))
                .ok_or_else(|| {
                    Error::new("hypothetical-inline-offset requires signed CSS pixels")
                })?;
            let hypothetical_inline_offset = parse_number(raw)?;
            let hypothetical_block_offset = node
                .attribute("hypothetical-block-offset")
                .map(|offset| {
                    offset
                        .strip_suffix("px")
                        .ok_or_else(|| {
                            Error::new("hypothetical-block-offset requires signed CSS pixels")
                        })
                        .and_then(parse_number)
                })
                .transpose()?;
            QualifiedRootContext::AbsoluteRootLocal {
                containing_size,
                containing_direction: direction("containing-direction")?,
                hypothetical_direction,
                hypothetical_display,
                hypothetical_inline_offset,
                hypothetical_block_offset,
                fallback,
            }
        }
        "block-flow-auto" => {
            let parent_inline_extent = extent(node, "parent-inline-extent")?;
            if parent_inline_extent != icb.width {
                return Err(Error::new(
                    "Auto zero-edge body inline extent disagrees with ICB",
                ));
            }
            QualifiedRootContext::BlockFlowAuto {
                parent_inline_extent,
                parent_direction: direction("parent-direction")?,
                fallback,
            }
        }
        "inline-grid" => {
            let parent_inline_extent = extent(node, "parent-inline-extent")?;
            if direction("parent-direction")? != layout::Direction::Ltr
                || parent_inline_extent != icb.width
            {
                return Err(Error::new(
                    "inline Grid requires the zero-edge LTR containing frame",
                ));
            }
            QualifiedRootContext::InlineGrid {
                parent_inline_extent,
                fallback,
            }
        }
        _ => return Err(Error::new("invalid qualified root kind")),
    };
    let context = RootContext::Qualified(qualified);
    if matches!(
        &context,
        RootContext::Qualified(QualifiedRootContext::AbsoluteRootLocal { .. })
    ) {
        context.request()?;
    }
    Ok(context)
}

impl RootFallback {
    fn normalized(&self) -> layout::OrthogonalFallbackOf<Scalar> {
        layout::OrthogonalFallbackOf::new(
            self.initial_containing_block.map(Some),
            layout::NearestScrollportOf::Absent,
            layout::Size::splat(layout::OrthogonalFallbackLimitOf::Unbounded),
        )
    }
}
impl RootContext {
    pub(super) fn validate_root(&self, root: &super::Node) -> Result<(), Error> {
        if let Self::Qualified(QualifiedRootContext::AbsoluteRootLocal {
            hypothetical_display,
            ..
        }) = self
        {
            let expected = match hypothetical_display.as_str() {
                "block" => "block",
                "flow-root" => "flow-root",
                "flex" | "inline-flex" => "flex",
                "grid" | "inline-grid" => "grid",
                "grid-lanes" => "grid-lanes",
                _ => return Err(Error::new("unsupported hypothetical outer display")),
            };
            if root.style.get("position") != Some("absolute")
                || root.style.get("display") != Some(expected)
            {
                return Err(Error::new(
                    "absolute proof disagrees with actual position or blockified display",
                ));
            }
        }
        if matches!(
            self,
            Self::Qualified(QualifiedRootContext::InlineGrid { .. })
        ) && (root.style.get("display") != Some("inline-grid")
            || root.style.get("position") != Some("static")
            || root.style.get("direction") != Some("ltr")
            || root.style.get("writing-mode").unwrap_or("horizontal-tb") != "horizontal-tb")
        {
            return Err(Error::new(
                "inline Grid proof disagrees with the actual static root",
            ));
        }
        Ok(())
    }
    pub(super) fn request(&self) -> Result<layout::LayoutRootRequest, Error> {
        let Self::Qualified(context) = self else {
            return Err(Error::new(
                "MissingRootContextAuthority: historical viewport metadata cannot issue a Native sizing operation",
            ));
        };
        Ok(match context {
            QualifiedRootContext::AbsoluteRootLocal {
                containing_size,
                containing_direction,
                hypothetical_inline_offset,
                hypothetical_block_offset,
                fallback,
                ..
            } => {
                let local = layout::RootLocalPositionedContext::horizontal_auto_insets(
                    *containing_size,
                    *containing_direction,
                    *hypothetical_inline_offset,
                    fallback.normalized(),
                )
                .map_err(|error| {
                    Error::new(format!("invalid hypothetical inline offset: {error:?}"))
                })?;
                let local = if let Some(offset) = hypothetical_block_offset {
                    local
                        .with_hypothetical_block_offset(*offset)
                        .map_err(|error| {
                            Error::new(format!("invalid hypothetical block offset: {error:?}"))
                        })?
                } else {
                    local
                };
                layout::LayoutRootRequest::positioned_root_local(local).map_err(|error| {
                    Error::new(format!("invalid root-local available space: {error:?}"))
                })?
            }
            QualifiedRootContext::InlineGrid {
                parent_inline_extent,
                fallback,
            } => layout::LayoutRootRequest::standalone(
                layout::SizingRequest::new(
                    layout::Size::new(
                        layout::AxisSizingRequest::new(
                            layout::AvailableSpace::Definite(*parent_inline_extent),
                            layout::SizingOperation::Normal,
                        ),
                        layout::AxisSizingRequest::new(
                            layout::AvailableSpace::Indefinite,
                            layout::SizingOperation::Normal,
                        ),
                    ),
                    fallback.normalized(),
                ),
                layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr),
                layout::Size::new(Some(*parent_inline_extent), None),
            ),
            QualifiedRootContext::BlockFlowAuto {
                parent_inline_extent,
                parent_direction,
                fallback,
            } => layout::LayoutRootRequest::block_flow(
                layout::Size::splat(layout::SizingOperation::Normal),
                layout::BlockFlowRootContext::horizontal_auto_block(
                    *parent_inline_extent,
                    *parent_direction,
                    fallback.normalized(),
                ),
            ),
        })
    }
}

impl RootContext {
    pub(super) fn validate_native_origin(
        &self,
        location: layout::Point,
        size: layout::Size,
    ) -> Result<(), Error> {
        let Self::Qualified(context) = self else {
            return Err(Error::new("MissingRootContextAuthority"));
        };
        let (inline_extent, direction) = match context {
            QualifiedRootContext::BlockFlowAuto {
                parent_inline_extent,
                parent_direction,
                ..
            } => (parent_inline_extent.get(), *parent_direction),
            QualifiedRootContext::AbsoluteRootLocal { .. }
            | QualifiedRootContext::InlineGrid { .. } => {
                if location != layout::Point::ZERO {
                    return Err(Error::new("Native root-local origin must be (0, 0)"));
                }
                return Ok(());
            }
        };
        let expected_x = if direction == layout::Direction::Ltr {
            0.0
        } else {
            inline_extent - size.width
        };
        // Same native lane/order as the selected zero-edge start equation.
        // No browser root rectangle participates in this assertion or request.
        if location.x != expected_x || location.y != 0.0 {
            return Err(Error::new(format!(
                "Native containing-frame root origin mismatch: expected ({expected_x}, 0), got ({}, {})",
                location.x, location.y,
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(raw: &str, version: Option<&str>) -> Result<RootContext, Error> {
        let document = roxmltree::Document::parse(raw).unwrap();
        parse_root_context(document.root_element(), version)
    }
    #[test]
    fn historical_viewport_is_retained_but_never_issues_a_request() {
        let raw = r#"<viewport width="max-content" height="max-content" root-context="root"/>"#;
        let parsed = context(raw, Some("2")).unwrap();
        assert!(
            matches!(&parsed, RootContext::UnqualifiedCapture { raw_viewport, .. } if raw_viewport == raw)
        );
        assert!(
            parsed
                .request()
                .unwrap_err()
                .to_string()
                .contains("MissingRootContextAuthority")
        );
    }
    #[test]
    fn auto_body_keeps_block_space_indefinite_and_owns_separate_fallback_facts() {
        let raw = r#"<viewport root-context="block-flow-auto" parent-inline-extent="800px" parent-direction="rtl" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>"#;
        let request = context(raw, Some("4")).unwrap().request().unwrap();
        let sizing = request.sizing_request();
        assert_eq!(
            sizing.axes().width.space().definite_size().unwrap().get(),
            800.0
        );
        assert_eq!(
            sizing.axes().height.space(),
            layout::AvailableSpace::Indefinite
        );
        assert_eq!(
            sizing.axes().height.operation(),
            layout::SizingOperation::Normal
        );
        let fallback = sizing.orthogonal_fallback();
        assert_eq!(
            fallback.initial_containing_block().height.unwrap().get(),
            600.0
        );
        assert_eq!(
            fallback.nearest_scrollport(),
            layout::NearestScrollportOf::Absent
        );
        assert_eq!(
            fallback.containing_limits().height,
            layout::OrthogonalFallbackLimitOf::Unbounded
        );
        let layout::LayoutRootContext::BlockFlow(parent) = request.context() else {
            panic!("wrong root context");
        };
        assert_eq!(
            parent.parent_flow_axes(),
            layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Rtl)
        );
    }
    #[test]
    fn incomplete_or_contradictory_new_root_metadata_refuses() {
        let raw = r#"<viewport root-context="block-flow-auto" parent-inline-extent="800px" parent-direction="ltr" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>"#;
        for invalid in [
            raw.replace(r#" parent-direction="ltr""#, ""),
            raw.replace("800px", "-1px"),
            raw.replace(
                r#"parent-inline-extent="800px""#,
                r#"parent-inline-extent="799px""#,
            ),
            raw.replace(
                r#"fallback-nearest-scrollport="none""#,
                r#"fallback-nearest-scrollport="unknown""#,
            ),
            raw.replace(
                r#"fallback-containing-height-limit="unbounded""#,
                r#"fallback-containing-height-limit="unknown""#,
            ),
            raw.replace("/>", r#" width="max-content"/>"#),
        ] {
            assert!(context(&invalid, Some("4")).is_err());
        }
    }
    fn absolute(offset: &str, html: &str, body: &str) -> String {
        format!(
            r#"<viewport root-context="absolute-root-local" containing-width="800px" containing-height="600px" containing-direction="{html}" hypothetical-direction="{body}" hypothetical-display="inline-grid" hypothetical-anchor="inline-start" hypothetical-inline-offset="{offset}px" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>"#
        )
    }
    #[test]
    fn local_absolute_uses_signed_progression_distance_and_independent_directions() {
        for html in ["ltr", "rtl"] {
            for body in ["ltr", "rtl"] {
                for (offset, expected) in
                    [("0", 800.0), ("100", 700.0), ("-100", 900.0), ("900", 0.0)]
                {
                    let parsed = context(&absolute(offset, html, body), Some("4")).unwrap();
                    let request = parsed.request().unwrap();
                    assert_eq!(
                        request
                            .sizing_request()
                            .axes()
                            .width
                            .space()
                            .definite_size()
                            .unwrap()
                            .get(),
                        expected
                    );
                    let layout::LayoutRootContext::RootLocalPositioned(local) = request.context()
                    else {
                        panic!("wrong local capability");
                    };
                    assert_eq!(
                        local.containing_flow_axes().direction(),
                        parse_direction(html).unwrap()
                    );
                    assert_eq!(
                        local.hypothetical_inline_offset(),
                        offset.parse::<Scalar>().unwrap()
                    );
                    parsed
                        .validate_native_origin(layout::Point::ZERO, layout::Size::new(100.0, 20.0))
                        .unwrap();
                    assert!(
                        parsed
                            .validate_native_origin(
                                layout::Point::new(1.0, 0.0),
                                layout::Size::new(100.0, 20.0)
                            )
                            .is_err()
                    );
                }
            }
        }
    }
    #[test]
    fn local_root_metadata_rejects_missing_unknown_nonfinite_and_overflow_facts() {
        let raw = absolute("0", "ltr", "rtl");
        for field in [
            "hypothetical-direction=\"rtl\" ",
            "hypothetical-display=\"inline-grid\" ",
            "hypothetical-anchor=\"inline-start\" ",
            "hypothetical-inline-offset=\"0px\" ",
        ] {
            assert!(
                context(&raw.replace(field, ""), Some("4")).is_err(),
                "{field}"
            );
        }
        for invalid in [
            raw.replace("inline-start", "physical-left"),
            raw.replace("inline-grid", "inline-table"),
            raw.replace("0px\" fallback", "NaNpx\" fallback"),
            raw.replace("0px\" fallback", "1e100px\" fallback"),
            raw.replace("absolute-root-local", "positioned-auto-insets"),
            raw.replace("/>", " extra=\"1\"/>"),
            absolute("-3.4028235e38", "ltr", "ltr").replace("800px", "3.4028235e38px"),
        ] {
            assert!(context(&invalid, Some("4")).is_err(), "{invalid}");
        }
        let old = raw.replace("absolute-root-local", "positioned-auto-insets");
        let parsed = context(&old, Some("3")).unwrap();
        assert!(matches!(&parsed, RootContext::UnqualifiedCapture { .. }));
        assert!(parsed.request().is_err());
    }
}
