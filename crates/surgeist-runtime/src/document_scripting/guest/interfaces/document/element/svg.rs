//! Interface selection for exact SVG local names.

use super::*;

#[derive(Clone, Copy)]
enum Parent {
    Base,
    Graphics,
    Geometry,
    Gradient,
    TextContent,
    TextPositioning,
    Animation,
    TransferFunction,
}

// The defining SVG2, SVG Animations, CSS Masking and Filter Effects IDL
// clauses supply the names and parent interfaces recorded under #2590.
const ELEMENTS: &[(&str, &str, Parent)] = &[
    ("a", "SVGAElement", Parent::Graphics),
    ("animate", "SVGAnimateElement", Parent::Animation),
    (
        "animateMotion",
        "SVGAnimateMotionElement",
        Parent::Animation,
    ),
    (
        "animateTransform",
        "SVGAnimateTransformElement",
        Parent::Animation,
    ),
    ("circle", "SVGCircleElement", Parent::Geometry),
    ("clipPath", "SVGClipPathElement", Parent::Base),
    ("defs", "SVGDefsElement", Parent::Graphics),
    ("desc", "SVGDescElement", Parent::Base),
    ("ellipse", "SVGEllipseElement", Parent::Geometry),
    ("feBlend", "SVGFEBlendElement", Parent::Base),
    ("feColorMatrix", "SVGFEColorMatrixElement", Parent::Base),
    (
        "feComponentTransfer",
        "SVGFEComponentTransferElement",
        Parent::Base,
    ),
    ("feComposite", "SVGFECompositeElement", Parent::Base),
    (
        "feConvolveMatrix",
        "SVGFEConvolveMatrixElement",
        Parent::Base,
    ),
    (
        "feDiffuseLighting",
        "SVGFEDiffuseLightingElement",
        Parent::Base,
    ),
    (
        "feDisplacementMap",
        "SVGFEDisplacementMapElement",
        Parent::Base,
    ),
    ("feDistantLight", "SVGFEDistantLightElement", Parent::Base),
    ("feDropShadow", "SVGFEDropShadowElement", Parent::Base),
    ("feFlood", "SVGFEFloodElement", Parent::Base),
    ("feFuncA", "SVGFEFuncAElement", Parent::TransferFunction),
    ("feFuncB", "SVGFEFuncBElement", Parent::TransferFunction),
    ("feFuncG", "SVGFEFuncGElement", Parent::TransferFunction),
    ("feFuncR", "SVGFEFuncRElement", Parent::TransferFunction),
    ("feGaussianBlur", "SVGFEGaussianBlurElement", Parent::Base),
    ("feImage", "SVGFEImageElement", Parent::Base),
    ("feMerge", "SVGFEMergeElement", Parent::Base),
    ("feMergeNode", "SVGFEMergeNodeElement", Parent::Base),
    ("feMorphology", "SVGFEMorphologyElement", Parent::Base),
    ("feOffset", "SVGFEOffsetElement", Parent::Base),
    ("fePointLight", "SVGFEPointLightElement", Parent::Base),
    (
        "feSpecularLighting",
        "SVGFESpecularLightingElement",
        Parent::Base,
    ),
    ("feSpotLight", "SVGFESpotLightElement", Parent::Base),
    ("feTile", "SVGFETileElement", Parent::Base),
    ("feTurbulence", "SVGFETurbulenceElement", Parent::Base),
    ("filter", "SVGFilterElement", Parent::Base),
    ("foreignObject", "SVGForeignObjectElement", Parent::Graphics),
    ("g", "SVGGElement", Parent::Graphics),
    ("image", "SVGImageElement", Parent::Graphics),
    ("line", "SVGLineElement", Parent::Geometry),
    (
        "linearGradient",
        "SVGLinearGradientElement",
        Parent::Gradient,
    ),
    ("marker", "SVGMarkerElement", Parent::Base),
    ("mask", "SVGMaskElement", Parent::Base),
    ("metadata", "SVGMetadataElement", Parent::Base),
    ("mpath", "SVGMPathElement", Parent::Base),
    ("path", "SVGPathElement", Parent::Geometry),
    ("pattern", "SVGPatternElement", Parent::Base),
    ("polygon", "SVGPolygonElement", Parent::Geometry),
    ("polyline", "SVGPolylineElement", Parent::Geometry),
    (
        "radialGradient",
        "SVGRadialGradientElement",
        Parent::Gradient,
    ),
    ("rect", "SVGRectElement", Parent::Geometry),
    ("script", "SVGScriptElement", Parent::Base),
    ("set", "SVGSetElement", Parent::Animation),
    ("stop", "SVGStopElement", Parent::Base),
    ("style", "SVGStyleElement", Parent::Base),
    ("svg", "SVGSVGElement", Parent::Graphics),
    ("switch", "SVGSwitchElement", Parent::Graphics),
    ("symbol", "SVGSymbolElement", Parent::Graphics),
    ("text", "SVGTextElement", Parent::TextPositioning),
    ("textPath", "SVGTextPathElement", Parent::TextContent),
    ("title", "SVGTitleElement", Parent::Base),
    ("tspan", "SVGTSpanElement", Parent::TextPositioning),
    ("use", "SVGUseElement", Parent::Graphics),
    ("view", "SVGViewElement", Parent::Base),
];

pub(super) struct Interface(Option<usize>);

pub(super) fn interface(
    local: &DocumentString,
    mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
) -> Result<Interface, DocumentGuestEventError> {
    for (index, &(name, _, _)) in ELEMENTS.iter().enumerate() {
        step()?;
        if local
            .code_units()
            .iter()
            .copied()
            .eq(name.bytes().map(u16::from))
        {
            return Ok(Interface(Some(index)));
        }
    }
    Ok(Interface(None))
}

pub(super) struct SvgBindings {
    base: RetainedCallbackObject,
    specialized: Vec<RetainedCallbackObject>,
    _backing: GuestBacking,
}

impl SvgBindings {
    pub(super) fn install<'scope>(
        scope: &mut Scope<'scope>,
        global: &Value<'scope>,
        element_constructor: &Value<'scope>,
        element: &Value<'scope>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    ) -> Result<Self, DocumentGuestEventError> {
        let base = install_named(scope, global, "SVGElement", element_constructor, element)?;
        let graphics = install_named(scope, global, "SVGGraphicsElement", &base.0, &base.1)?;
        let geometry = install_named(
            scope,
            global,
            "SVGGeometryElement",
            &graphics.0,
            &graphics.1,
        )?;
        let gradient = install_named(scope, global, "SVGGradientElement", &base.0, &base.1)?;
        let text_content = install_named(
            scope,
            global,
            "SVGTextContentElement",
            &graphics.0,
            &graphics.1,
        )?;
        let text_positioning = install_named(
            scope,
            global,
            "SVGTextPositioningElement",
            &text_content.0,
            &text_content.1,
        )?;
        let animation = install_named(scope, global, "SVGAnimationElement", &base.0, &base.1)?;
        let transfer = install_named(
            scope,
            global,
            "SVGComponentTransferFunctionElement",
            &base.0,
            &base.1,
        )?;
        let bytes = ELEMENTS
            .len()
            .checked_mul(size_of::<RetainedCallbackObject>())
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let backing = fund(bytes)?;
        let mut specialized = Vec::new();
        specialized
            .try_reserve_exact(ELEMENTS.len())
            .map_err(|_| DocumentScriptNativeError::StorageLimit)?;
        if specialized.capacity() != ELEMENTS.len() {
            return Err(DocumentScriptNativeError::RepresentationExhausted.into());
        }
        for &(_, name, parent) in ELEMENTS {
            let (constructor, parent) = match parent {
                Parent::Base => &base,
                Parent::Graphics => &graphics,
                Parent::Geometry => &geometry,
                Parent::Gradient => &gradient,
                Parent::TextContent => &text_content,
                Parent::TextPositioning => &text_positioning,
                Parent::Animation => &animation,
                Parent::TransferFunction => &transfer,
            };
            let (_, prototype) = install_named(scope, global, name, constructor, parent)?;
            let observed = scope.callback_object(&prototype)?;
            specialized.push(scope.retain_callback_object(observed, fund)?);
        }
        let observed = scope.callback_object(&base.1)?;
        Ok(Self {
            base: scope.retain_callback_object(observed, fund)?,
            specialized,
            _backing: backing,
        })
    }

    pub(super) fn prototype(
        &self,
        interface: Interface,
    ) -> Result<&RetainedCallbackObject, DocumentGuestEventError> {
        match interface.0 {
            Some(index) => self
                .specialized
                .get(index)
                .ok_or(DocumentScriptNativeError::ProjectionCustody.into()),
            None => Ok(&self.base),
        }
    }
}
