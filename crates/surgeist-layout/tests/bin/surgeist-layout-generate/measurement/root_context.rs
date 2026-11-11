//! Source-qualified browser root frames; no rectangle supplies these operands.

use super::{Attributes, Context, MeasurementError, layout_number, nonnegative, number};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireViewport {
    root_context: WireRootContext,
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum WireRootContext {
    AbsoluteRootLocal {
        containing_width: f64,
        containing_height: f64,
        containing_direction: Direction,
        hypothetical_direction: Direction,
        hypothetical_display: HypotheticalDisplay,
        hypothetical_anchor: HypotheticalAnchor,
        hypothetical_inline_offset: f64,
        #[serde(default, deserialize_with = "super::present")]
        hypothetical_block_offset: Option<f64>,
        fallback: Fallback,
    },
    BlockFlowAuto {
        parent_inline_extent: f64,
        parent_direction: Direction,
        fallback: Fallback,
    },
    InlineGrid {
        parent_inline_extent: f64,
        parent_direction: Direction,
        fallback: Fallback,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Direction {
    Ltr,
    Rtl,
}
impl Direction {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum HypotheticalDisplay {
    Block,
    FlowRoot,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    GridLanes,
}
impl HypotheticalDisplay {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::FlowRoot => "flow-root",
            Self::Flex => "flex",
            Self::InlineFlex => "inline-flex",
            Self::Grid => "grid",
            Self::InlineGrid => "inline-grid",
            Self::GridLanes => "grid-lanes",
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum HypotheticalAnchor {
    InlineStart,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fallback {
    initial_containing_block: PhysicalExtent,
    nearest_scrollport: NearestScrollport,
    containing_limits: ContainingLimits,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PhysicalExtent {
    width: f64,
    height: f64,
}
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum NearestScrollport {
    // The closed source profile proves no intervening element scrollport.
    // The viewport/ICB remains a separate, definite physical fallback owner.
    None,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContainingLimits {
    width: UnboundedLimit,
    height: UnboundedLimit,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum UnboundedLimit {
    Unbounded,
}

impl Fallback {
    fn validate(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        layout_number(
            nonnegative(
                self.initial_containing_block.width,
                context,
                "viewport.rootContext.fallback.initialContainingBlock.width",
            )?,
            context,
            "viewport.rootContext.fallback.initialContainingBlock.width",
        )?;
        layout_number(
            nonnegative(
                self.initial_containing_block.height,
                context,
                "viewport.rootContext.fallback.initialContainingBlock.height",
            )?,
            context,
            "viewport.rootContext.fallback.initialContainingBlock.height",
        )?;
        let NearestScrollport::None = self.nearest_scrollport;
        let _ = (
            &self.containing_limits.width,
            &self.containing_limits.height,
        );
        Ok(())
    }
    fn attrs(&self) -> Attributes {
        vec![
            (
                "fallback-icb-width",
                format!("{}px", number(self.initial_containing_block.width)),
            ),
            (
                "fallback-icb-height",
                format!("{}px", number(self.initial_containing_block.height)),
            ),
            ("fallback-nearest-scrollport", "none".into()),
            ("fallback-containing-width-limit", "unbounded".into()),
            ("fallback-containing-height-limit", "unbounded".into()),
        ]
    }
}

impl WireViewport {
    pub(super) fn root_local_positioned(&self) -> bool {
        matches!(self.root_context, WireRootContext::AbsoluteRootLocal { .. })
    }
    pub(super) fn inline_grid(&self) -> bool {
        matches!(self.root_context, WireRootContext::InlineGrid { .. })
    }

    pub(super) fn hypothetical_block_offset(&self) -> Option<f64> {
        match &self.root_context {
            WireRootContext::AbsoluteRootLocal {
                hypothetical_block_offset,
                ..
            } => *hypothetical_block_offset,
            WireRootContext::BlockFlowAuto { .. } | WireRootContext::InlineGrid { .. } => None,
        }
    }

    pub(super) fn validate_root_style(
        &self,
        style: Option<&super::style::Style>,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        if let WireRootContext::AbsoluteRootLocal {
            hypothetical_display,
            ..
        } = &self.root_context
        {
            let style = context.required(style, "style")?;
            let expected = match hypothetical_display {
                HypotheticalDisplay::Block => "block",
                HypotheticalDisplay::FlowRoot => "flow-root",
                HypotheticalDisplay::Flex | HypotheticalDisplay::InlineFlex => "flex",
                HypotheticalDisplay::Grid | HypotheticalDisplay::InlineGrid => "grid",
                HypotheticalDisplay::GridLanes => "grid-lanes",
            };
            context.compatible(
                style.position.as_deref() == Some("absolute")
                    && style.display.as_deref() == Some(expected),
                "viewport.rootContext.hypotheticalDisplay",
                "absolute proof disagrees with actual position or blockified display",
            )?;
        } else if self.inline_grid() {
            let style = context.required(style, "style")?;
            context.compatible(
                style.position.as_deref() == Some("static")
                    && style.display.as_deref() == Some("inline-grid"),
                "viewport.rootContext",
                "inline Grid proof requires its authored static display",
            )?;
        }
        Ok(())
    }
    pub(super) fn validate_alternative(
        &self,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        match &self.root_context {
            WireRootContext::AbsoluteRootLocal {
                containing_width,
                containing_height,
                hypothetical_inline_offset,
                hypothetical_block_offset,
                fallback,
                ..
            } => {
                fallback.validate(context)?;
                layout_number(
                    nonnegative(
                        *containing_width,
                        context,
                        "viewport.rootContext.containingWidth",
                    )?,
                    context,
                    "viewport.rootContext.containingWidth",
                )?;
                layout_number(
                    nonnegative(
                        *containing_height,
                        context,
                        "viewport.rootContext.containingHeight",
                    )?,
                    context,
                    "viewport.rootContext.containingHeight",
                )?;
                context.compatible(
                    *containing_width == fallback.initial_containing_block.width
                        && *containing_height == fallback.initial_containing_block.height,
                    "viewport.rootContext",
                    "absolute source profile requires its actual initial containing block",
                )?;
                let offset = layout_number(
                    *hypothetical_inline_offset,
                    context,
                    "viewport.rootContext.hypotheticalInlineOffset",
                )? as f32;
                let width = *containing_width as f32;
                layout_number(
                    f64::from(width - offset),
                    context,
                    "viewport.rootContext.hypotheticalInlineOffset",
                )?;
                if let Some(offset) = hypothetical_block_offset {
                    let offset = layout_number(
                        *offset,
                        context,
                        "viewport.rootContext.hypotheticalBlockOffset",
                    )? as f32;
                    layout_number(
                        f64::from(*containing_height as f32 - offset),
                        context,
                        "viewport.rootContext.hypotheticalBlockOffset",
                    )?;
                }
            }
            WireRootContext::BlockFlowAuto {
                parent_inline_extent,
                fallback,
                ..
            }
            | WireRootContext::InlineGrid {
                parent_inline_extent,
                fallback,
                ..
            } => {
                fallback.validate(context)?;
                layout_number(
                    nonnegative(
                        *parent_inline_extent,
                        context,
                        "viewport.rootContext.parentInlineExtent",
                    )?,
                    context,
                    "viewport.rootContext.parentInlineExtent",
                )?;
                context.compatible(
                    *parent_inline_extent == fallback.initial_containing_block.width,
                    "viewport.rootContext",
                    "zero-edge containing inline extent must match the authenticated ICB",
                )?;
            }
        }
        Ok(())
    }
    pub(super) fn validate(&self, context: &Context<'_>) -> Result<Attributes, MeasurementError> {
        self.validate_alternative(context)?;
        let (mut attrs, fallback) = match &self.root_context {
            WireRootContext::AbsoluteRootLocal {
                containing_width,
                containing_height,
                containing_direction,
                hypothetical_direction,
                hypothetical_display,
                hypothetical_anchor: HypotheticalAnchor::InlineStart,
                hypothetical_inline_offset,
                fallback,
                ..
            } => (
                vec![
                    ("root-context", "absolute-root-local".into()),
                    (
                        "containing-width",
                        format!("{}px", number(*containing_width)),
                    ),
                    (
                        "containing-height",
                        format!("{}px", number(*containing_height)),
                    ),
                    ("containing-direction", containing_direction.as_str().into()),
                    (
                        "hypothetical-direction",
                        hypothetical_direction.as_str().into(),
                    ),
                    ("hypothetical-display", hypothetical_display.as_str().into()),
                    ("hypothetical-anchor", "inline-start".into()),
                    (
                        "hypothetical-inline-offset",
                        format!("{}px", number(*hypothetical_inline_offset)),
                    ),
                ],
                fallback,
            ),
            WireRootContext::BlockFlowAuto {
                parent_inline_extent,
                parent_direction,
                fallback,
            }
            | WireRootContext::InlineGrid {
                parent_inline_extent,
                parent_direction,
                fallback,
            } => (
                vec![
                    (
                        "root-context",
                        if self.inline_grid() {
                            "inline-grid"
                        } else {
                            "block-flow-auto"
                        }
                        .into(),
                    ),
                    (
                        "parent-inline-extent",
                        format!("{}px", number(*parent_inline_extent)),
                    ),
                    ("parent-direction", parent_direction.as_str().into()),
                ],
                fallback,
            ),
        };
        if let Some(offset) = self.hypothetical_block_offset() {
            attrs.push(("hypothetical-block-offset", format!("{}px", number(offset))));
        }
        attrs.extend(fallback.attrs());
        Ok(attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_context_distinguishes_explicit_static_top_from_absence() {
        let mut viewport = serde_json::json!({
            "rootContext": {
                "kind": "absolute-root-local",
                "containingWidth": 800,
                "containingHeight": 600,
                "containingDirection": "ltr",
                "hypotheticalDirection": "ltr",
                "hypotheticalDisplay": "block",
                "hypotheticalAnchor": "inline-start",
                "hypotheticalInlineOffset": 0,
                "fallback": {
                    "initialContainingBlock": {"width": 800, "height": 600},
                    "nearestScrollport": "none",
                    "containingLimits": {"width": "unbounded", "height": "unbounded"}
                }
            }
        });
        let context = Context {
            case_id: "root",
            variant: "border-box-ltr",
            node_path: "/".into(),
        };
        let attribute = |input| {
            serde_json::from_value::<WireViewport>(input)
                .unwrap()
                .validate(&context)
                .unwrap()
                .into_iter()
                .find_map(|(key, value)| (key == "hypothetical-block-offset").then_some(value))
        };
        assert_eq!(attribute(viewport.clone()), None);
        viewport["rootContext"]["hypotheticalBlockOffset"] = serde_json::json!(0);
        assert_eq!(attribute(viewport.clone()).as_deref(), Some("0px"));
        viewport["rootContext"]["hypotheticalBlockOffset"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<WireViewport>(viewport).is_err());
    }

    #[test]
    fn root_context_requires_a_complete_closed_payload() {
        let valid = r#"{"rootContext":{"kind":"block-flow-auto","parentInlineExtent":800,"parentDirection":"rtl","fallback":{"initialContainingBlock":{"width":800,"height":600},"nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}}"#;
        let context = Context {
            case_id: "root",
            variant: "rtl",
            node_path: "root".into(),
        };
        let wire: WireViewport = serde_json::from_str(valid).unwrap();
        let attrs = wire.validate(&context).unwrap();
        assert!(attrs.contains(&("parent-direction", "rtl".into())));
        assert!(attrs.contains(&("fallback-nearest-scrollport", "none".into())));
        for invalid in [
            valid.replace("\"parentInlineExtent\":800,", ""),
            valid.replace(
                "\"nearestScrollport\":\"none\"",
                "\"nearestScrollport\":\"unknown\"",
            ),
            valid.replace("\"kind\":\"block-flow-auto\"", "\"kind\":\"root\""),
            valid.replace(
                "\"parentDirection\":\"rtl\"",
                "\"parentDirection\":\"rtl\",\"height\":\"max-content\"",
            ),
        ] {
            assert!(serde_json::from_str::<WireViewport>(&invalid).is_err());
        }
    }

    #[test]
    fn contradictory_extent_and_negative_facts_refuse() {
        let context = Context {
            case_id: "root",
            variant: "ltr",
            node_path: "root".into(),
        };
        for width in [-1, 799] {
            let raw = serde_json::json!({"rootContext":{"kind":"block-flow-auto","parentInlineExtent":width,"parentDirection":"ltr","fallback":{"initialContainingBlock":{"width":800,"height":600},"nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}}).to_string();
            let wire: WireViewport = serde_json::from_str(&raw).unwrap();
            assert!(wire.validate(&context).is_err());
        }
    }
    #[test]
    fn absolute_local_requires_complete_proof_and_checked_native_remaining_space() {
        let context = Context {
            case_id: "local",
            variant: "rtl",
            node_path: "root".into(),
        };
        let valid = serde_json::json!({"rootContext":{"kind":"absolute-root-local","containingWidth":800,"containingHeight":600,"containingDirection":"ltr","hypotheticalDirection":"rtl","hypotheticalDisplay":"inline-grid","hypotheticalAnchor":"inline-start","hypotheticalInlineOffset":0,"fallback":{"initialContainingBlock":{"width":800,"height":600},"nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}});
        let wire: WireViewport = serde_json::from_value(valid.clone()).unwrap();
        assert!(wire.root_local_positioned());
        let attrs = wire.validate(&context).unwrap();
        assert!(attrs.contains(&("hypothetical-inline-offset", "0px".into())));
        assert!(attrs.contains(&("hypothetical-direction", "rtl".into())));
        for field in [
            "hypotheticalDirection",
            "hypotheticalDisplay",
            "hypotheticalAnchor",
            "hypotheticalInlineOffset",
        ] {
            let mut invalid = valid.clone();
            invalid["rootContext"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                serde_json::from_value::<WireViewport>(invalid).is_err(),
                "{field}"
            );
        }
        for (field, value) in [
            ("hypotheticalDisplay", serde_json::json!("inline-table")),
            ("hypotheticalAnchor", serde_json::json!("physical-left")),
            ("hypotheticalInlineOffset", serde_json::Value::Null),
            ("extra", serde_json::json!(1)),
        ] {
            let mut invalid = valid.clone();
            invalid["rootContext"][field] = value;
            assert!(
                serde_json::from_value::<WireViewport>(invalid).is_err(),
                "{field}"
            );
        }
        let mut overflow = valid;
        overflow["rootContext"]["containingWidth"] = serde_json::json!(f64::from(f32::MAX));
        overflow["rootContext"]["fallback"]["initialContainingBlock"]["width"] =
            serde_json::json!(f64::from(f32::MAX));
        overflow["rootContext"]["hypotheticalInlineOffset"] =
            serde_json::json!(-f64::from(f32::MAX));
        assert!(
            serde_json::from_value::<WireViewport>(overflow)
                .unwrap()
                .validate(&context)
                .is_err()
        );
    }
}
