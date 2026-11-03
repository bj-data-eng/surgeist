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
    PositionedAutoInsets {
        containing_width: f64,
        containing_height: f64,
        containing_direction: Direction,
        fallback: Fallback,
    },
    BlockFlowAuto {
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
    pub(super) fn validate_alternative(
        &self,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        match &self.root_context {
            WireRootContext::PositionedAutoInsets {
                containing_width,
                containing_height,
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
                    "positioned source profile requires its actual initial containing block",
                )?;
            }
            WireRootContext::BlockFlowAuto {
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
                    "zero-edge Auto body inline extent must match the authenticated ICB",
                )?;
            }
        }
        Ok(())
    }
    pub(super) fn validate(&self, context: &Context<'_>) -> Result<Attributes, MeasurementError> {
        self.validate_alternative(context)?;
        let (mut attrs, fallback) = match &self.root_context {
            WireRootContext::PositionedAutoInsets {
                containing_width,
                containing_height,
                containing_direction,
                fallback,
            } => (
                vec![
                    ("root-context", "positioned-auto-insets".into()),
                    (
                        "containing-width",
                        format!("{}px", number(*containing_width)),
                    ),
                    (
                        "containing-height",
                        format!("{}px", number(*containing_height)),
                    ),
                    ("containing-direction", containing_direction.as_str().into()),
                ],
                fallback,
            ),
            WireRootContext::BlockFlowAuto {
                parent_inline_extent,
                parent_direction,
                fallback,
            } => (
                vec![
                    ("root-context", "block-flow-auto".into()),
                    (
                        "parent-inline-extent",
                        format!("{}px", number(*parent_inline_extent)),
                    ),
                    ("parent-direction", parent_direction.as_str().into()),
                ],
                fallback,
            ),
        };
        attrs.extend(fallback.attrs());
        Ok(attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
