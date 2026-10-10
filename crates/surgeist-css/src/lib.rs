#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Browser-recovering CSS ingestion for Surgeist.
//!
//! [`parse_sheet`] and [`parse_style_attribute`] parse UTF-8 input into CSS-owned
//! authored syntax plus every structured recovery diagnostic in source order.
//! Retained nodes are valid by construction. Unsupported or malformed source
//! units are recovered at their grammar boundary so later valid siblings remain
//! eligible; invalid authored nodes are never retained.
//!
//! A [`CssParseReport::is_clean`] result means exactly that the diagnostic slice
//! is empty. It is not a separate syntax-validity predicate, and callers must not
//! infer cleanliness from an empty retained sheet or declaration list.
//!
//! # Stylesheets and recovery
//!
//! ```
//! use surgeist_css::{CssErrorCode, CssRecoveryAction, CssRule, parse_sheet};
//!
//! let report = parse_sheet(
//!     ".before { color: red; } @unknown value; .after { color: blue; }",
//! );
//! assert_eq!(report.syntax().rules().len(), 2);
//! assert!(matches!(report.syntax().rules()[0], CssRule::Style(_)));
//! let diagnostic = &report.diagnostics()[0];
//! assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownAtRule);
//! assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
//! ```
//!
//! # Authored nesting
//!
//! A style rule owns its complete selector list, leading declarations, and ordered child
//! rules. Later declaration runs use [`CssRule::NestedDeclarations`], preserving their place
//! after a nested rule without selecting cascade winners or expanding parent selectors.
//!
//! ```
//! use surgeist_css::{CssRule, parse_sheet};
//!
//! let report = parse_sheet(".card, #featured { color: red; &:hover { opacity: 1; } color: blue; }");
//! assert!(report.is_clean());
//! let [CssRule::Style(parent)] = report.syntax().rules() else { unreachable!() };
//! assert_eq!(parent.selectors().selectors().len(), 2);
//! assert_eq!(parent.declarations().len(), 1);
//! let [CssRule::Style(child), CssRule::NestedDeclarations(after)] = parent.rules() else {
//!     unreachable!()
//! };
//! assert_eq!(child.declarations().len(), 1);
//! assert_eq!(after.declarations().len(), 1);
//! ```
//!
//! # Style attributes and declarations
//!
//! Style attributes share the ordinary declaration grammar used by style-rule
//! blocks. Declarations retain authored order, semantic source positions,
//! property/value coupling, custom-property text, substitution-dependent text,
//! and terminal [`CssImportance`].
//!
//! ```
//! use surgeist_css::{
//!     CssImportance, CssKnownProperty, CssPropertyNameRef, parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute(
//!     "--Theme: RGB(1, 2, var(--fallback)); mystery: 1; width: var(--size, 2px) !important",
//! );
//! assert_eq!(report.syntax().len(), 2);
//! assert_eq!(report.diagnostics().len(), 1);
//! assert_eq!(
//!     report.syntax()[0]
//!         .custom()
//!         .expect("custom declaration")
//!         .value()
//!         .value()
//!         .expect("authored custom value")
//!         .as_css(),
//!     "RGB(1, 2, var(--fallback))",
//! );
//! let width = &report.syntax()[1];
//! assert_eq!(width.importance(), CssImportance::Important);
//! assert!(matches!(width.property_name(), CssPropertyNameRef::Known(_)));
//! let value = width.known().expect("coupled width declaration");
//! assert_eq!(value.property(), CssKnownProperty::Width);
//! assert_eq!(
//!     value
//!         .substitution_dependent()
//!         .expect("symbolic authored value")
//!         .as_css(),
//!     "var(--size, 2px)",
//! );
//! ```
//!
//! # Declaration inspection and API evolution
//!
//! Parsing and [`parse_property_value`] construct [`CssKnownDeclaration`] through the same
//! property grammar. Its fields are private, and [`CssKnownDeclaration::property`] derives
//! identity from the active coupled value, so callers cannot create a property/value mismatch.
//! [`CssKnownDeclaration::declared_value`] returns exactly one of the
//! [`CssKnownDeclaredValueRef::Property`], [`CssKnownDeclaredValueRef::Global`],
//! or [`CssKnownDeclaredValueRef::SubstitutionDependent`] branches. The
//! [`CssKnownDeclaration::property_value`], [`CssKnownDeclaration::global`], and
//! [`CssKnownDeclaration::substitution_dependent`] convenience accessors are
//! mutually exclusive views of those same branches.
//!
//! The property branch borrows a non-exhaustive [`CssKnownPropertyValueRef`].
//! Match its concrete generated wrapper and retain a wildcard for future
//! variants:
//!
//! ```
//! use surgeist_css::{
//!     CssBoxSize, CssImportance, CssKnownDeclaredValueRef, CssKnownPropertyValueRef, CssSizeValue,
//!     parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute("width: calc(100% - 12px) !important");
//! let declaration = &report.syntax()[0];
//! assert_eq!(declaration.importance(), CssImportance::Important);
//! let known = declaration.known().expect("known declaration");
//!
//! match known.declared_value() {
//!     CssKnownDeclaredValueRef::Property(property) => match property {
//!         CssKnownPropertyValueRef::Width(width) => {
//!             assert_eq!(width.as_css(), "calc(100% - 12px)");
//!             let CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length)) = width.value() else {
//!                 panic!("expected a length-percentage");
//!             };
//!             assert!(length.calculation().is_some());
//!         }
//!         _ => panic!("expected width"),
//!     },
//!     CssKnownDeclaredValueRef::Global(_)
//!     | CssKnownDeclaredValueRef::SubstitutionDependent(_) => {
//!         panic!("expected an ordinary property value")
//!     }
//!     _ => panic!("future declared-value branch"),
//! }
//! ```
//!
//! Each ordinary property-schema row generates one private-field
//! `Css<SchemaVariant>PropertyValue` wrapper holding the authored declaration
//! and its grammar-checked semantic value. `as_css()` returns the exact authored
//! ordinary slice, excluding boundary trivia and terminal importance. Checked
//! construction uses token-preserving serialization; original or programmatic
//! provenance remains available through [`CssDeclaration::value_components`].
//! Each wrapper has one semantic accessor: `value()` by default, or a domain name
//! such as `images()`, `factor()`, `ratio()`, `families()`, or `font()`.
//! The `all` property accepts only global or substitution-dependent values and
//! has no ordinary wrapper.
//!
//! The `background-image` and `mask-image` wrappers expose [`CssImageValueList`]
//! through `images()`, retaining URL, `none`, gradient and LightDark branches in order.
//! [`CssLightDarkColor::try_new`] and [`CssLightDarkImage::try_new`] retain two
//! checked branches and enforce the complete composed subtree's 256-level ceiling.
//! Color pairs remain contextual; image pairs may contain two `none` branches.
//! Specified serialization shares the enclosing resource budget and preserves both
//! branches. Downstream style owns used-scheme selection and resource resolution.
//! [`CssContrastColor::try_new`] retains one checked symbolic color input, enforcing
//! the complete composed depth ceiling. Its input uses ordinary Standalone specified
//! serialization under the enclosing budget. Downstream style evaluates contrast
//! and chooses white or black; authored CSS retains the input unchanged.
//! [`CssDeviceCmykColor::try_new`] retains four ordered, unbounded ink channels
//! and optional alpha. Legacy form is number-only with no alpha; modern form
//! also accepts percentages and `none`. [`CssColor::device_cmyk_value`] borrows
//! the device-dependent input. Declared Standalone/Mix output scales direct
//! percentages to numbers; Origin preserves their domains and explicit alpha.
//! Calculations remain symbolic, and every child shares the enclosing depth and
//! serialization limits. The independent concrete-sample helpers
//! [`naively_convert_cmyk_to_srgba`] and [`naively_convert_srgba_to_cmyk`] implement
//! the uncalibrated Color 5 numerical fallback with finite `f64` coordinates.
//! They preserve alpha bits and do not resolve authored colors or device profiles.
//! Contextual conversion and computed output belong later.
//! [`convert_lab_to_lch`] and [`convert_oklab_to_oklch`] provide pure numerical
//! rectangular-to-polar coordinates from Color 4, with space-specific missing-hue
//! thresholds. [`CssPolarColorCoordinates`] checks finite lightness/chroma and
//! normalizes optional hue; [`convert_lch_to_lab`] and [`convert_oklch_to_oklab`]
//! invert those coordinates, setting both axes to zero when hue is missing.
//! These helpers use concrete number units independently of authored colors.
//! [`CssHslColorCoordinates`] and [`CssHwbColorCoordinates`] describe concrete
//! HSL and HWB coordinates with optional degree hue and percentage reference
//! units. [`convert_hsl_to_srgb`], [`convert_srgb_to_hsl`],
//! [`convert_hwb_to_srgb`] and [`convert_srgb_to_hwb`] convert encoded sRGB
//! samples without gamut clipping. Inverse conversion marks powerless hue
//! missing while retaining the other coordinates; forward missing hue uses zero.
//! [`CssRectangularColorCoordinates`] checks concrete RGB, XYZ, Lab and Oklab
//! channels with explicit missingness. Its [`convert_to`](CssRectangularColorCoordinates::convert_to)
//! method converts among [`CssRectangularColorSpace`] values without gamut
//! clipping, preserving intermediate range with binary64 significand precision.
//! Same-space conversion retains all bits; cross-space missing channels become
//! zero locally. Alpha and authored expression evaluation remain separate.
//! [`CssOverflowPropertyValue::value`] exposes [`CssOverflowValue`], retaining
//! the authored one- or two-axis form.
//!
//! Public enums marked `#[non_exhaustive]` require a wildcard arm in downstream
//! matches. Closed enums, including [`CssImportance`] and [`CssSupportStatus`],
//! remain exhaustively matchable. This inspection model does not change parsing,
//! recovery, or diagnostics.
//!
//! # Typed authored calculations
//!
//! Calculation roots preserve authored numeric values and units without resolving layout,
//! timelines, or device context. Literal construction is checked, and every expression is
//! inspected through borrowed views while the owned compound representation remains private.
//!
//! ```
//! use surgeist_css::{
//!     CssAngleCalculation, CssAngleUnit, CssCalculationExpressionRef,
//!     CssCalculationType, CssCalculationValueRef,
//! };
//!
//! let angle = CssAngleCalculation::try_literal("-0.5", CssAngleUnit::Turns)
//!     .expect("checked authored angle");
//! assert_eq!(angle.result_type(), CssCalculationType::Angle);
//! assert!(matches!(
//!     angle.expression(),
//!     CssCalculationExpressionRef::Value(CssCalculationValueRef::Angle(value))
//!         if value.representation() == "-0.5" && value.unit() == Some("turn")
//! ));
//! ```
//!
//! Calculation trees remain authored and symbolic. This crate checks finite literal values and
//! dimensional validity, but it does not resolve relative units, evaluate computed ranges, or
//! run animation timelines.
//!
//! # Property-specific authored positions
//!
//! Generic CSS positions retain both axes and distinguish free offsets from offsets authored
//! against a named edge. [`CssSpecifiedLengthPercentage`] accepts only the symbolic length-percentage domain;
//! it does not resolve percentages, calculations, writing modes, or positioning boxes.
//!
//! ```
//! use surgeist_css::{CssComponentValue, CssSpecifiedLengthPercentage};
//!
//! let offset = CssSpecifiedLengthPercentage::try_from_component(
//!     CssComponentValue::try_token("25%").expect("exact authored percentage"),
//! ).expect("position-valid offset");
//! assert!(matches!(offset.literal_component().unwrap().view(), surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number)) if number.representation() == "25"));
//! assert!(CssSpecifiedLengthPercentage::try_from_component(
//!     CssComponentValue::try_ident("auto").unwrap(),
//! ).is_err());
//! ```
//!
//! [`CssPhysicalPosition::try_new`] checks the generic position's cross-axis pairing of edge
//! offsets. Its borrowed horizontal and vertical views make omitted centered axes and authored
//! edge origins explicit. `object-position` and every
//! `mask-position` layer use this exact generic grammar. `background-position` instead exposes a
//! nonempty [`CssBackgroundPositionList`] whose layers also admit the background-only
//! three-component form. Its noninherited longhand initial is one `0% 0%` layer.
//! [`CssBackgroundPosition::serialize_specified`] and the list's matching method
//! emit horizontal-first specified values, preserving an omitted edge offset.
//! The checked [`CssBackgroundLayer::try_new`] and [`CssBackground::try_new`]
//! constructors retain optional authored components, couple size to position, and
//! restrict color to the final layer. Intrinsic `background` expansion fills each
//! of seven per-layer lists from the terminal schema initial and emits one color.
//! It also resets `background-blend-mode` to one `normal` entry. Expansion and
//! normalization account for nine terminal contributions; a CSS-wide keyword
//! applies to all nine. The authored shorthand retains its eight settable members.
//! [`CssBackground::serialize_specified`] composes canonical specified children in
//! grammar order, omitting proved simple initials under one cumulative budget.
//! Omitted authored children still consume input and projection visits; an empty
//! effective layer emits `none`. Keywords, calculations, and contextual colors
//! retain their symbolic meaning.
//! `transform-origin` exposes the directed 2D split plus an optional checked
//! [`CssSpecifiedLength`] length.
//!
//! ```
//! use surgeist_css::{
//!     CssHorizontalPosition, CssKnownPropertyValueRef, CssVerticalPosition,
//!     parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute(concat!(
//!     "background-position: left 10px top; ",
//!     "mask-position: right 5% bottom 2px; ",
//!     "object-position: center 25%; ",
//!     "transform-origin: left top 50px",
//! ));
//! assert!(report.is_clean());
//!
//! let CssKnownPropertyValueRef::BackgroundPosition(background) = report.syntax()[0]
//!     .known().expect("known background position")
//!     .property_value().expect("ordinary background position")
//! else { panic!("expected background-position") };
//! assert!(matches!(
//!     background.positions().positions()[0].horizontal(),
//!     CssHorizontalPosition::LeftOffset(offset)
//!         if matches!(offset.literal_component().unwrap().view(), surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "10" && unit == "px")
//! ));
//!
//! assert_eq!(background.positions().serialize_specified().unwrap(), "left 10px top");
//!
//! let CssKnownPropertyValueRef::MaskPosition(mask) = report.syntax()[1]
//!     .known().expect("known mask position")
//!     .property_value().expect("ordinary mask position")
//! else { panic!("expected mask-position") };
//! assert!(matches!(
//!     mask.positions().positions()[0].vertical(),
//!     CssVerticalPosition::BottomOffset(_)
//! ));
//!
//! let CssKnownPropertyValueRef::ObjectPosition(object) = report.syntax()[2]
//!     .known().expect("known object position")
//!     .property_value().expect("ordinary object position")
//! else { panic!("expected object-position") };
//! assert!(matches!(object.position().horizontal(), CssHorizontalPosition::Center));
//!
//! let CssKnownPropertyValueRef::TransformOrigin(transform) = report.syntax()[3]
//!     .known().expect("known transform origin")
//!     .property_value().expect("ordinary transform origin")
//! else { panic!("expected transform-origin") };
//! assert!(matches!(transform.origin().z().unwrap().literal_component().unwrap().view(), surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "50" && unit == "px"));
//! ```
//!
//! A vertical-only planar keyword followed by a length (`top 50px` or
//! `bottom calc(1px * 2)`) is rejected. A planar pair may be followed by a
//! checked Z length, as in `left top 50px`. This follows the selected pinned
//! WebKit transform-origin consumer; CSS Values 4 gives a conflicting example
//! for the two-token form, recorded in the reference guide.
//!
//! The position property wrappers expose only their checked semantic models.
//! `background-position` retains its distinct three-component grammar, while
//! `object-position`, `mask-position`, and mask shorthand use physical [`CssPhysicalPosition`].
//! Circle and ellipse instead use the full Values 5 [`CssPosition`] coordinate families.
//! Its checked Cartesian, named-flow and relative-flow constructors retain symbolic axes;
//! [`CssPosition::view`] exposes borrowed payloads. All-center values have one Cartesian form.
//! Specified output orders Cartesian axes horizontally/vertically and flow axes by block/inline.
//! Cascade, substitution, contextual resolution, layout, painting, transforms, and cross-crate
//! lowering remain outside this surface.
//!
//! # Dedicated authored function grammars
//!
//! Property accessors expose dedicated typed function families. Timing-function
//! wrappers expose the sole [`CssEasingList`] through `timing_functions()`.
//! Transform wrappers expose the sole [`CssTransform`] through `value()`; filter and
//! backdrop-filter wrappers return [`CssFilter`], box-shadow returns [`CssBoxShadow`], and
//! clip-path exposes the sole [`CssClipPath`] through `value()`.
//!
//! Authored Shapes 1 adds three independent noninherited longhands:
//! `shape-outside` borrows [`CssShapeOutside`], `shape-image-threshold` borrows
//! [`CssOpacityValue`], and `shape-margin` borrows
//! [`CssSpecifiedNonNegativeLengthPercentage`]. [`CssShapeOutsideShape`] retains
//! an optional [`CssShapeBox`] with only the four visual/margin boxes. Omitted
//! boxes stay distinct from explicit margin-box, and canonical pairs emit the
//! shape first. Threshold values remain unclamped; ordinary margin literals
//! are nonnegative while actual math defers range handling. Geometry, resource
//! loading, percentage bases and computed threshold clamping remain downstream.
//!
//! Motion Path's five offset longhands and [`CssOffset`] compose the shared
//! numeric, URL, position and BasicShape owners. [`CssOffsetPath::view`]
//! distinguishes none, box-only and a path with an optional [`CssCoordBox`].
//! [`CssRay`] retains a strict angle bearing, optional authored size, containment
//! and an optional [`CssPosition`] from the full symbolic Cartesian, named-flow
//! and relative-flow families. [`CssOffsetRotate`] retains modifier/angle
//! absence without applying tangent rotation. Offset's five ordered reset
//! contributions use central normal/none/zero/auto/auto initials; its specified
//! output emits only authored constituents. Geometry, resources and animation
//! execution remain downstream.
//!
//! Clip-path is a noninherited terminal with [`CssClipPath::None`] initial and the
//! shared ordinary, CSS-wide, pending-reentry and normalization lifecycle. It accepts
//! all seven [`CssBoxEdgeKeyword`] alternatives alone or with one shape in either
//! authored order. [`CssClipPathShape`] retains optional reference-box omission;
//! its serializer emits shape before box without resolving their geometry.
//! [`CssEllipseRadii`] contains two independent [`CssEllipseRadius`] components,
//! each an extent or a nonnegative length-percentage. [`CssEllipseShape::radii`]
//! distinguishes an omitted pair. Polygon round is an optional signed pure length,
//! following optional fill-rule; percentages and reversed modifiers are rejected.
//!
//! Clip-path, its composition, basic shapes and all eight function models provide
//! bounded specified serializers preserving optional omissions, shape offset/radius arities, units,
//! exact magnitudes and symbolic math through shared cumulative child providers.
//! Circle and ellipse positions retain the full imported Cartesian, named-flow, and
//! relative-flow grammar. [`CssRectShape`] retains four signed edges or `auto`;
//! [`CssXywhShape`] retains signed offsets and nonnegative width and height. Both
//! retain optional round radii. [`CssPathShape`] composes optional [`CssFillRule`]
//! with checked [`CssPathData`], preserving the decoded SVG spelling and string origin.
//! Its complete SVG scanner admits move-only paths and rejects malformed suffixes
//! without computing geometry. [`CssShapeFunction`] retains a full starting position
//! and a nonempty ordered [`CssShapeCommandList`]. Curves and smooth commands couple
//! absolute or relative endpoints with correspondingly typed controls; arcs retain
//! signed radii and optional sweep, size and strict angles. Specified output retains
//! authored omissions and emits the required first comma. Reference-box/default resolution, clipping and
//! painting belong to downstream owners.
//!
//! [`CssShadow`] requires blur before spread, and [`CssBoxShadowList`] is nonempty.
//! Box-shadow is a noninherited terminal with [`CssBoxShadow::None`] initial;
//! ordinary, CSS-wide, and pending values use the shared expansion and normalization
//! lifecycle. Length components form one contiguous group, while color and inset
//! may precede or follow that group. [`CssDropShadow`] has signed offsets and an
//! optional nonnegative `standard_deviation()`, with optional color on either side.
//! Its third length is a standard deviation; box-shadow retains `blur_radius()`.
//!
//! `serialize_specified()` and `serialize_specified_with_limits()` on all four
//! shadow models retain authored omissions and explicit zero/currentcolor. Box
//! shadows emit color, offsets, blur, spread, then inset; lists use comma-space;
//! drop shadows emit the full `drop-shadow()` function. Numeric and color children
//! use their existing canonical providers with one cumulative resource budget.
//! These operations preserve symbolic math/colors and diagnostic origins.
//!
//! ```
//! use surgeist_css::{
//!     CssBasicShape, CssClipPath, CssFilterFunction, CssFilter,
//!     CssKnownPropertyValueRef, CssTransformFunction, CssTransform,
//!     parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute(concat!(
//!     "transform: translate3d(10%, 2px, 4em) rotate(45deg); ",
//!     "filter: blur(2px) drop-shadow(red 1px 2px 3px); ",
//!     "clip-path: polygon(round 2px, 0 0, 100% 0)",
//! ));
//! assert!(report.is_clean());
//!
//! let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
//!     .known().expect("known transform")
//!     .property_value().expect("ordinary transform")
//! else { panic!("expected transform") };
//! assert!(matches!(
//!     transform.value(),
//!     CssTransform::Functions(functions)
//!         if matches!(functions.functions()[0], CssTransformFunction::Translate3d(_))
//! ));
//!
//! let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[1]
//!     .known().expect("known filter")
//!     .property_value().expect("ordinary filter")
//! else { panic!("expected filter") };
//! assert!(matches!(
//!     filter.value(),
//!     CssFilter::Functions(functions)
//!         if matches!(functions.functions()[1], CssFilterFunction::DropShadow(_))
//! ));
//!
//! let CssKnownPropertyValueRef::ClipPath(clip) = report.syntax()[2]
//!     .known().expect("known clip path")
//!     .property_value().expect("ordinary clip path")
//! else { panic!("expected clip-path") };
//! assert!(matches!(
//!     clip.value(),
//!     CssClipPath::BasicShape(shape)
//!         if matches!(shape.shape(), CssBasicShape::Polygon(polygon) if polygon.round().is_some())
//! ));
//! ```
//!
//! The typed transform family covers the selected two-dimensional Transforms 1
//! functions and the selected three-dimensional subset with exact arity,
//! separators, and dimensions. Matrices, rotation axes, scale numbers, and easing
//! coordinates share `CssSpecifiedNumber`, which retains exact literals and symbolic
//! math. All scale percentage branches use `CssSpecifiedPercentage`.
//! Ordinary cubic-bezier X coordinates are checked exactly against inclusive [0, 1];
//! genuine calculations remain unresolved. The independent `scale` property retains
//! one to three typed number/percentage factors, including symbolic calculations. Transform, filter, gradient, and
//! image-orientation angles share exact `CssAngleLiteral` values and symbolic
//! Angle-root calculations through the strict `CssAngleValue`. Transforms, filters,
//! and gradient directions admit bare zero through `CssAngleOrZero`; image orientation
//! and shape rotation require an angle dimension or calculation. Raw scalars retain provenance while
//! semantic aggregates compare structure independently of scalar origins.
//! Easing values distinguish keywords,
//! `cubic-bezier()`, `steps()`, and authored `linear()` stops. Box shadows and filter
//! `drop-shadow()` have
//! separate models, filter lists preserve URL/function order, and the selected
//! basic-shape family exposes `inset()`, `circle()`, `ellipse()`, `polygon()`,
//! `rect()`, `xywh()`, `path()`, and `shape()`, including polygon `round <length>` and optional
//! rectangular round radii.
//!
//! These are authored syntax values. This crate does not multiply transform matrices,
//! interpolate or evaluate easing, render shadows or filters, resolve URLs, compute
//! shape geometry, perform layout or painting, or lower values into sibling crates.
//! Basic shapes and `clip-path` expose complete authored grammar support; this does
//! not establish contextual geometry or exact calculation projection. The selected
//! transition property family has intrinsic metadata and four ordered shorthand
//! contributions, preserving one list entry per authored item. [`CssLonghandValueRef`]
//! exposes property, duration, timing-function and delay lists, including explicit
//! child origins and programmatic omission defaults. The selected Animation family
//! likewise has intrinsic metadata and eight ordered contributions: duration,
//! timing-function, delay, iteration-count, direction, fill-mode, play-state and
//! name. Its borrowed variants retain the existing typed lists and one entry per
//! authored item, using schema initials for omissions. Both selected authored
//! families have Complete support metadata; contextual execution belongs downstream.
//! Checked whole-property construction and pending reentry reject original implicit
//! closures, while reused parser-produced typed children retain recovery origins.
//! Support for a typed function does not promote an unselected production.
//!
//! [`CssCustomIdent`] owns decoded custom-identifier admission. Timing names refine
//! that checked value through [`CssKeyframesIdent`] (shared by keyframe definitions
//! and animation names) and [`CssTransitionPropertyName`]. [`CssGridLineNames`]
//! contains checked [`CssGridLineName`] values with the Grid exclusions. Parsing
//! and public construction both reject mixed `None` transition lists.
//!
//! Both filter properties are noninherited terminals with an intrinsic `none`
//! initial. [`CssFilterHueRotate`] retains an omitted angle separately from its
//! effective `0deg`; [`CssFilterBlur`] likewise retains omission and effective `0px`.
//! [`CssFilter`], [`CssFilterFunctionList`] and [`CssFilterFunction`] serialize
//! specified values under one cumulative resource budget. They retain authored
//! order, omitted arguments, explicit defaults, exact checked amounts and symbolic
//! children. Ordinary angle literals keep their finite precision and units;
//! calculations reuse the existing shared projection without filter-specific
//! normalization or clamping. The backdrop source selects only the named property
//! in an immutable exploring draft, with its Level 1 dependency pinned to the
//! selected 2018 publication. It does not select the complete Level 2 module.
//! The four authored SVG filter declarations are also complete CSS grammars:
//! `flood-color` and `lighting-color` reuse [`CssColor`], while `flood-opacity`
//! reuses the exact [`CssOpacityValue`] without authored range clamping.
//! [`CssColorInterpolationFilters`] distinguishes `Auto`, `Srgb` and `LinearRgb`
//! and emits lowercase specified keywords. Their intrinsic initials are black,
//! 1, white and linearRGB respectively; only color-interpolation-filters inherits.
//! SVG presentation attributes, applicability, computed opacity clamping and
//! filter/lighting execution remain downstream. The selected Filter1 property
//! is nonanimatable; this enum does not select a CSS filter-function color policy.
//!
//! ```
//! use surgeist_css::{CssFilter, CssFilterFunction, CssFilterFunctionList, CssFilterHueRotate};
//! let functions = CssFilterFunctionList::try_new(vec![
//!     CssFilterFunction::HueRotate(CssFilterHueRotate::omitted()),
//! ]).unwrap();
//! assert_eq!(CssFilter::Functions(functions).serialize_specified().unwrap(), "hue-rotate()");
//! ```
//!
//! # Authored colors
//!
//! [`CssColor`] is the sole checked authored color graph. It retains named,
//! hexadecimal, system, RGB/HSL/HWB, Lab/LCH, predefined `color()`, relative,
//! custom-profile, `alpha()`, and `color-mix()` branches. Ordinary color numbers,
//! percentages, and angles retain exact lexical coefficients and origins;
//! calculations remain symbolic. Color-bearing property wrappers expose this
//! graph through `value()` or an aggregate `color()` accessor.
//! [`CssColor::keyword_srgba8`] exposes the intrinsic encoded sRGB bytes of
//! opaque named colors and transparent black, preserving their keyword identity.
//! It returns `None` for representations outside this fixed keyword projection.
//!
//! ```
//! use surgeist_css::{CssKnownPropertyValueRef, CssSystemColor, parse_style_attribute};
//!
//! let report = parse_style_attribute("color: ActiveBorder; border: solid #fff");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("color") };
//! assert_eq!(color.value().system(), Some(CssSystemColor::ActiveBorder));
//! let CssKnownPropertyValueRef::Border(border) = report.syntax()[1]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("border") };
//! assert_eq!(border.value().color().unwrap().hex_value().unwrap().digits(), "fff");
//! ```
//!
//! Checked constructors compose each color family without resolving
//! `currentcolor`, system colors, profiles, relative channels, or a mix. The
//! specified serializer applies cumulative resource limits and returns an atomic
//! canonical string. Authored out-of-range components are retained until their
//! computed-value phase. Opacity remains a separate authored numeric model.
//!
//! # Authored Grid repetition and keyframe structure
//!
//! The six Grid repetition consumers expose authored values through
//! their `value()` accessors. [`CssGridTrackList`] represents `none`, general
//! track lists, lists containing exactly one [`CssGridAutoRepeat`], and subgrid
//! line-name lists. [`CssGridSubgridComponent`] retains ordered groups and
//! [`CssGridNameRepeat`] values, allowing at most one auto-fill name repeat.
//! Counted repeats use checked [`CssPositiveIntegerValue`] children: bare roots
//! must be positive, while function math retains its graph and deferred range.
//! Integer
//! and automatic repetition are non-recursive. Under the
//! [selected Grid 3 grammar](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat),
//! automatic bodies admit general track sizes, including intrinsic and flexible
//! sizes, while surrounding tracks and integer repeats retain fixed sizes.
//! `grid-auto-rows` and `grid-auto-columns` expose
//! [`CssGridTrackSizeList`] values without `repeat()`.
//! [`CssGridAutoRepeat::content`] therefore borrows
//! [`CssGridTrackRepeatContent`], while surrounding fixed repeats retain
//! [`CssGridFixedRepeatContent`].
//! Track breadths retain exact ordinary `fr`, length, and percentage quantities
//! in checked specified scalars. Flex-result math follows the pinned WebKit
//! behavior where [Grid 2's `fr` rule](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#fr-unit)
//! and [Values 4 math typing](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking)
//! conflict. A flex calculation remains symbolic and cannot enter an inflexible
//! `minmax()` minimum or `fit-content()`. Grid track models provide checked
//! constructors and bounded canonical `serialize_specified()` separately from
//! declaration `as_css()`.
//!
//! [`CssKeyframesRule`] and [`CssKeyframeBlock`] preserve source structure. Empty
//! rules and blocks remain present, while repeated selector blocks, equivalent
//! offsets across blocks, and repeated equivalent selectors within one
//! [`CssKeyframeSelectorList`] remain in authored order. Dropping an invalid
//! declaration leaves its valid empty parents; an invalid selector still drops
//! the smallest invalid block. The parser does not sort, merge, or deduplicate
//! keyframes.
//!
//! [`CssGridTemplate`] also retains area-string rows with optional authored
//! sizes and boundary names through [`CssGridTemplateAreaTrack`], reusing the
//! existing checked area matrix and repeat-free column list. Intrinsic expansion
//! emits three ordered template members or six grid members through shared
//! metadata, strict pending reentry and normalization. Template axes and areas
//! start at `none`; implicit sizes start at `auto`, with the accepted flow initial
//! `normal`. The selected authored repeat and six consuming property productions
//! are Complete. `grid-auto-flow` retains its separate unresolved orientation
//! boundary. Repeat evaluation and used track sizing remain downstream.
//! `@keyframes` supports the complete selected authored production. Quoted names
//! admit empty and whitespace-only text. Literal percentage offsets use checked
//! binary64; percentage-valued calculations retain typed graphs and origins.
//! [`CssKeyframePercent::literal_value`] and [`CssKeyframePercent::calculation`]
//! expose those states, while [`CssKeyframeSelector::offset`] clones the authored
//! domain without evaluating it. Keyframe mode drops the eight prohibited Animation
//! declarations and retains animation-timing-function even at the final endpoint.
//! Computed range clamping, endpoint easing use and name lookup remain downstream.
//! Authored equality preserves quoted versus identifier form and calculation origins.
//! This crate does not perform Grid
//! layout, cascade declarations, evaluate or interpolate keyframes, run
//! timelines, or lower either syntax family into sibling Surgeist crates.
//!
//! # Authored item flow
//!
//! Grid3's provisional Appendix A supplies the selected flow-oriented
//! `item-direction`, `item-wrap`, `item-pack` and `item-flow` grammars.
//! Direction, wrap, packing and signed flow tolerance remain authored values;
//! physical axes and used placement require downstream context. A shorthand
//! preserves complete wrap/pack groups and emits all four constituents.
//!
//! ```rust
//! use surgeist_css::{
//!     CssKnownPropertyValueRef, CssItemWrap, CssItemWrapOrder,
//!     parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute("item-flow: balance dense reverse -2px row");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::ItemFlow(flow) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("item-flow value") };
//! assert_eq!(
//!     flow.value().serialize_specified().unwrap(),
//!     "row reverse dense balance -2px",
//! );
//! let order_only = CssItemWrap::Order(CssItemWrapOrder::Reverse);
//! assert_eq!(order_only.serialize_specified().unwrap(), "reverse");
//! ```
//!
//! # Authored wrapping and whitespace
//!
//! Selected Text 4 wrapping and whitespace shorthands preserve authored fields
//! separately from their intrinsic two/three-member projections. Missing fields
//! reset to longhand initials during expansion. White-space's four special forms
//! keep their spelling, while trim flags form one contiguous constituent group.
//! Word breaking includes manual, auto-phrase and the deprecated CSS break-word.
//! These values do not execute text processing or resolve contextual wrapping.
//!
//! ```rust
//! use surgeist_css::{
//!     CssKnownPropertyValueRef, CssTextWrap, CssTextWrapMode, CssTextWrapStyle,
//!     CssWhiteSpaceKeyword, parse_style_attribute,
//! };
//!
//! let wrapping = CssTextWrap::try_new(
//!     Some(CssTextWrapMode::NoWrap), Some(CssTextWrapStyle::Balance),
//! ).unwrap();
//! assert_eq!(wrapping.serialize_specified().unwrap(), "nowrap balance");
//! assert!(CssTextWrap::try_new(None, None).is_none());
//! let report = parse_style_attribute("white-space: pre-wrap");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::WhiteSpace(value) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("white-space") };
//! assert_eq!(value.value().keyword(), Some(CssWhiteSpaceKeyword::PreWrap));
//! assert_eq!(value.value().serialize_specified().unwrap(), "pre-wrap");
//! ```
//!
//! # Authored text transformations, break preferences and tabs
//!
//! [`CssTextTransform`] distinguishes `none`, `math-auto` and a checked nonempty
//! [`CssTextTransformSet`]. Case, width and kana roles retain independent presence;
//! specified output emits them in that order. [`CssWrapInside`],
//! [`CssWrapBoundary`] and [`CssLineBreak`] retain finite break preferences, while
//! [`CssWordSpaceTransform`] retains `none` or a separator base and optional phrase flag.
//! [`CssTabSize`] keeps checked Number and Length domains distinct: bare zero is
//! a number, explicit `0px` a length, and typed calculation ranges remain deferred.
//!
//! These seven Text 4 longhands, [`CssTextIndent`] from Text 4 and
//! [`CssVerticalAlign`] from CSS2 expose intrinsic initials, inheritance metadata
//! and one typed contribution each. CSS-wide values stay symbolic; pending
//! substitution uses strict, retryable reentry preserving original importance and
//! replacement origins. Language processing, font metrics, indentation,
//! alignment and tab-stop execution belong to downstream owners.
//!
//! ```rust
//! use surgeist_css::{
//!     CssKnownPropertyValueRef, CssTabSize, CssTextTransform, CssTextTransformCase,
//!     CssTextTransformSet, parse_style_attribute,
//! };
//!
//! let set = CssTextTransformSet::try_new(
//!     Some(CssTextTransformCase::Uppercase), true, false,
//! ).unwrap();
//! assert_eq!(set.case(), Some(CssTextTransformCase::Uppercase));
//! assert!(set.full_width());
//! assert!(!set.full_size_kana());
//! assert!(CssTextTransformSet::try_new(None, false, false).is_none());
//! let transform = CssTextTransform::Transforms(set);
//! assert_eq!(transform.serialize_specified().unwrap(), "uppercase full-width");
//!
//! let report = parse_style_attribute("text-transform:full-width uppercase;tab-size:0px");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::TextTransform(value) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("text-transform") };
//! assert_eq!(value.value(), &transform);
//! let CssKnownPropertyValueRef::TabSize(tab) = report.syntax()[1]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("tab-size") };
//! assert!(matches!(tab.value(), CssTabSize::Length(_)));
//! assert_eq!(tab.value().serialize_specified().unwrap(), "0px");
//! ```
//!
//! # Authored hyphenation, justification and spacing
//!
//! Hyphenation preserves exact nonnegative integer tokens, one to three authored
//! character-count slots, and symbolic Integer-root calculations. Effective
//! omitted slots remain separate from authored serialization and computed rounding.
//! Hyphenate-character retains its original string component and decoded content;
//! zone and line-padding reuse signed numeric owners. Text-justify preserves
//! authored distribute and no-compress presence, with computed equivalence downstream.
//!
//! Text-spacing expands exactly spacing-trim then autospace. Its complete
//! constituents remain contiguous, and omitted members receive intrinsic normal
//! initials without extra authored output tokens. Autospace preserves omitted
//! insertion mode; hanging-punctuation keeps an exclusive force/allow end role.
//! These properties share strict checked admission, pending replacement, intrinsic
//! metadata and cumulative specified output. Language, glyph and layout execution
//! belongs to downstream owners. See the
//! [reference](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-css/docs/reference.md#authored-hyphenation-justification-and-spacing).
//!
//! ```rust
//! use surgeist_css::{
//!     CssContributions, CssExpansion, CssKnownProperty, expand_declaration,
//!     parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute(
//!     "hyphenate-limit-chars:auto 2;text-justify:no-compress distribute;\
//!      text-spacing:replace punctuation trim-both",
//! );
//! assert!(report.is_clean());
//! assert_eq!(report.syntax()[0].to_specified_css().unwrap(),
//!     "hyphenate-limit-chars: auto 2;");
//! assert_eq!(report.syntax()[1].to_specified_css().unwrap(),
//!     "text-justify: distribute no-compress;");
//! assert_eq!(report.syntax()[2].to_specified_css().unwrap(),
//!     "text-spacing: trim-both punctuation replace;");
//! let CssExpansion::Contributions(CssContributions::Longhands(values)) =
//!     expand_declaration(&report.syntax()[2]).unwrap()
//! else { panic!("two spacing terminals") };
//! assert_eq!(values.items().iter().map(|v| v.property()).collect::<Vec<_>>(),
//!     [CssKnownProperty::TextSpacingTrim, CssKnownProperty::TextAutospace]);
//! ```
//!
//! Checked constituent construction preserves authored omissions:
//!
//! ```rust
//! use surgeist_css::{
//!     CssAutospace, CssAutospaceMode, CssAutospaceValues, CssSpacingTrim,
//!     CssTextJustify, CssTextJustifyBase, CssTextSpacing, CssTextSpacingValues,
//! };
//!
//! assert!(CssTextJustify::try_new(None, false).is_none());
//! let justify = CssTextJustify::try_new(Some(CssTextJustifyBase::Distribute), true)
//!     .unwrap();
//! assert_eq!(justify.serialize_specified().unwrap(), "distribute no-compress");
//! let flags = CssAutospaceValues::try_new(true, false, true, None).unwrap();
//! assert_eq!(flags.mode(), None);
//! assert_eq!(flags.effective_mode(), CssAutospaceMode::Insert);
//! let fields = CssTextSpacingValues::try_new(
//!     Some(CssSpacingTrim::TrimBoth), Some(CssAutospace::Spacing(flags)),
//! ).unwrap();
//! assert_eq!(CssTextSpacing::Components(fields).serialize_specified().unwrap(),
//!     "trim-both ideograph-alpha punctuation");
//! ```
//!
//! # Typography, font families, and font-face
//!
//! Family lists, the `@font-face` family descriptor, and `local()` names follow
//! the selected September 7, 2026 Fonts 4 grammar. [`CssFontFamilyName`] preserves
//! decoded identifier tokens and distinguishes literal names from all fifteen
//! typed generic families. Family and font wrappers expose their current
//! `families()` and `font()` models without the obsolete I01 projections; use
//! [`CssFontValue`] and [`CssExplicitFont`] for explicit or system fonts.
//! Other typography includes four-ASCII-character OpenType tags, non-negative
//! feature indices, synthesis, and variant longhands. Individual metadata
//! records identify the implemented grammar and its dated source.
//!
//! ```
//! use surgeist_css::{
//!     CssFontValue, CssKnownPropertyValueRef, CssSystemFont,
//!     parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute("font: menu; font-weight: 725");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::Font(font) = report.syntax()[0]
//!     .known().expect("known font")
//!     .property_value().expect("ordinary font")
//! else { panic!("expected font") };
//! assert!(matches!(font.font(), CssFontValue::System(CssSystemFont::Menu)));
//! assert_eq!(font.as_css(), "menu");
//! ```
//!
//! `@font-feature-values` retains ordered families, all seven subsidiary blocks,
//! repeated definitions, and interleaved `font-display` occurrences through
//! [`CssFontFeatureValuesRule`]. Checked constructors share parser constraints;
//! [`CssFontFeatureValueIndex`] preserves exact nonnegative decimal digits without a
//! machine-integer limit. Parsed integer origins address their complete original
//! token, while Rust construction has no invented source coordinates.
//!
//! The pinned Fonts 4 draft (7 September 2026) conflicts between sections 6.9.1
//! and 6.9.2. The selected section 6.9.2 / frozen WebKit policy admits one or two
//! character-variant indexes and nonempty styleset lists without feature-specific
//! upper bounds. The normative contradiction remains unresolved. All index
//! positions retain arbitrarily large exact nonnegative authored integer tokens;
//! downstream font activation may ignore unsupported feature indexes.
//!
//! Ordinary stylesheet/group/scope rule lists retain these global named rules;
//! style-rule ancestry rejects them. Normalization retains one opaque payload and
//! its parent contexts, without emitting property contributions or applying font
//! mapping/cascade. Normalization limits do not count payload declarations or bound
//! payload allocation. Fonts rules have canonical effective specified writers;
//! generic group/sheet wrapper coverage remains with its separate owner.
//!
//! [`CssFontFaceDescriptors::occurrences`] exposes valid descriptor occurrences
//! in authored order, while typed effective accessors return the last valid
//! occurrence. Invalid and unknown occurrences recover with
//! [`CssRecoveryAction::DropDescriptor`] without erasing valid neighbors. Empty
//! [`CssFontFaceRule`] values remain in the authored tree; whether a face can be
//! used belongs to later contextual processing.
//!
//! Family, font, source-list, and font-face records cite `I-FONTS4-20260907`.
//! Family grammar, the font shorthand, and modern source hints are Complete;
//! source list and selected authored font-face rule are Complete, including
//! checked composition and canonical effective-value rule serialization.
//! Named-instance, language and metric descriptors retain typed ordinary values
//! or whole-descriptor environment values. Their payload serializers are bounded
//! and context independent. Values 4 `url()` and `src()` preserve authored
//! function identity and modifiers without loading resources.
//! Older immutable source identities retain their original editions.
//! Family names and lists provide bounded canonical specified serialization.
//! Font loading, matching, fallback, shaping, cascade, substitution, computed
//! values, and live CSSOM behavior belong to their downstream owners.
//!
//! # Authored font-synthesis
//!
//! [`CssFontSynthesis`] represents `none` or a checked set of weight, style,
//! small-caps, and position capabilities. Intrinsic expansion contributes four
//! inherited longhands: selected capabilities become `auto`, omitted ones become
//! `none`. Their initial values are `auto`; only [`CssFontSynthesisStyle`] admits
//! `oblique-only`. Font matching and glyph synthesis remain downstream.
//!
//! ```
//! use surgeist_css::{CssFontSynthesis, CssFontSynthesisValues};
//!
//! let value = CssFontSynthesis::Values(
//!     CssFontSynthesisValues::try_new(true, false, false, true).unwrap(),
//! );
//! assert_eq!(value.serialize_specified().unwrap(), "weight position");
//! ```
//!
//! # Authored font palettes
//!
//! [`CssFontPalette`] retains keywords, symbolic names and checked ordered mixes.
//! [`CssFontPaletteMix`] shares color interpolation and percentage providers,
//! accepts one or more recursive palette components, and checks their complete
//! structural depth. Its specified output preserves symbolic values without
//! selecting a font or evaluating a palette. The property is inherited, initially
//! `normal`, and independent of the `font` shorthand.
//!
//! ```
//! use surgeist_css::{CssFontPalette, CssFontPaletteMix, CssFontPaletteMixComponent};
//!
//! let value = CssFontPalette::Mix(Box::new(CssFontPaletteMix::try_new(None, vec![
//!     CssFontPaletteMixComponent::new(CssFontPalette::Light, None),
//!     CssFontPaletteMixComponent::new(CssFontPalette::Dark, None),
//! ]).unwrap()));
//! assert_eq!(value.serialize_specified().unwrap(), "palette-mix(light, dark)");
//! ```
//!
//! # Ordinary canonical number text
//!
//! Shared ordinary number, percentage and dimension serializers emit shortest base-ten text
//! with at most six fractional places, no exponent, and no negative rounded zero. Exact unit
//! conversion precedes rounding. The selected decimal halfway policy rounds away from zero;
//! CSSOM does not specify that direction. Original checked tokens, admission ranges, equality
//! and diagnostic origins stay exact. Each ordinary scalar keeps one input and one projection
//! visit; byte limits count its actual rounded text and unit within the cumulative writer.
//!
//! # Calculated canonical number text
//!
//! Finite non-color, declared relative-color and ordinary-origin calculation coefficients use
//! the same six-place fixed text policy, rounding the actual binary64 value after projection.
//! Symbols, units, arithmetic, signed-zero and exceptional-value behavior, and traversal costs
//! stay unchanged. Border-image
//! and scroll shorthands compare unrounded canonical components before selecting rounded text.
//! Relative calculations use this policy wherever embedded, including nested origins and mixes.
//! Ordinary origins preserve number/percentage dimensions, canonical angle units and explicit
//! unclamped alpha. Ordinary non-alpha component and hue calculations use this policy when
//! their projected result remains context-dependent, including inside mixes. The existing
//! projection result selects formatting after percentage scaling and angle conversion.
//! Retained calculated alpha and mix weights also use the shared coefficient policy.
//! Contextual alpha and calculated alpha outside ordinary RGB/HSL/HWB stay explicit and
//! unclamped; percentage alpha divides by 100 before formatting, while actual Origin alpha
//! preserves its dimension. Ordinary RGB/HSL/HWB scalar calculated alpha in standalone
//! colors and Mix children independently normalizes NaN to zero and clamps to 0..1,
//! omitting exact unity before rounding. This
//! follows the recorded WebKit interpretation of conflicting Color 4 phase clauses.
//! Calculated weights keep
//! percentages and their calculation wrapper, preventing omitted sibling shares from being
//! filled even when they emit `calc(50%)`. These rules apply to nested mixes and ordinary
//! colors inside a mix used as an origin. Resolved ordinary RGB/HSL/HWB non-alpha slots
//! emit scalars, including resolved siblings of contextual slots. Finite calculated scalars
//! round their original post-scale binary64 bits to six fractional places, nearest with ties
//! toward positive infinity; HSL/HWB conversion formats the final clipped RGB channels.
//! Direct literals keep exact decimal arithmetic. Captures must succeed before finalization
//! and keep their scratch costs even when final text is shorter. Retained alpha, weight and
//! contextual component captures count rounded scratch text within the cumulative budget.
//! Any directly missing ordinary RGB/HSL/HWB component, including alpha, selects a form
//! preserving `none`; omitted or calculated alpha alone does not. RGB uses normalized
//! `color(srgb ...)`, while HSL/HWB retain their named functions. Direct HSL/HWB channels
//! emit percentages and a bare degree hue. RGB clamps to its output domain; HSL saturation
//! has a zero minimum, including resolved negative/NaN calculations under the selected
//! frozen WebKit interpretation. Named lightness and HWB percentages remain unbounded,
//! with NaN becoming zero and dimensional infinities remaining valid calculations.
//! Numeric hue normalizes modulo 360 before conversion or named-form rounding.
//! Ordinary mix children follow the same rule, with cumulative limits on the actual text.
//! Direct Lab/LCH lightness in standalone colors and ordinary mix arguments clamps
//! to 0..100, while Oklab/Oklch lightness
//! clamps to 0..1 after exact percentage conversion. Direct LCH/Oklch chroma has a zero
//! minimum and no upper bound; the signed a/b axes remain unbounded. In-range literals
//! keep exact decimal text. A clipped direct endpoint is selected from borrowed lexical
//! metadata before rational allocation or unnecessary decimal expansion, including
//! finite authored exponents beyond i128. Each selected direct slot adds one logical
//! projection visit. Calculations retain their wrappers and capture costs; origin and
//! relative channels remain unclamped. These direct bounds apply to ordinary mix
//! arguments, including nested mixes and a mix used as an origin color.
//! Unbounded and in-range values still obey typed output and work limits.
//! Integer calculation text follows the finite number policy without
//! computed integer rounding; ordinary integer
//! literals retain exact digits. Arithmetic precision and range remain unfinished.
//!
//! # Authored timing domains
//!
//! Time literals retain exact coefficients and authored units; duration literals are non-negative
//! and delays are signed. Well-typed calculations remain authored for later range processing.
//! Ordinary specified emission converts exactly to seconds before rounding text to at most six
//! fractional places, nearest with ties away from zero under the selected frozen WebKit policy.
//! Authored coefficients, units and origins remain exact. Finite calculation text rounds the
//! existing projected binary64 result with the same policy. Exact calculation arithmetic and
//! range processing remain unfinished.
//!
//! ```
//! use surgeist_css::{
//!     CssDuration, CssTimeLiteral, CssTimeValue, CssKnownPropertyValueRef,
//!     CssTimeUnit, parse_style_attribute,
//! };
//! let negative = CssTimeValue::from_literal(CssTimeLiteral::try_new("-1", CssTimeUnit::Seconds).unwrap());
//! assert!(CssDuration::try_new(negative).is_err());
//! let report = parse_style_attribute("transition-duration: calc(-1s + 2s); transition-delay: -250ms");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::TransitionDuration(duration) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap() else { panic!("duration"); };
//! assert!(duration.durations().values()[0].time().calculation().is_some());
//! let CssKnownPropertyValueRef::TransitionDelay(delay) = report.syntax()[1]
//!     .known().unwrap().property_value().unwrap() else { panic!("delay"); };
//! let literal = delay.delays().values()[0].literal().unwrap();
//! assert_eq!(literal.numeric().representation(), "-250");
//! assert_eq!(literal.unit(), CssTimeUnit::Milliseconds);
//! assert_eq!(literal.serialize_specified().unwrap(), "-0.25s");
//! ```
//!
//! This crate owns authored timing syntax only; timeline evaluation and
//! cross-crate lowering remain downstream responsibilities.
//!
//! # Namespaces and complete Selectors 3 syntax
//!
//! [`CssRule::Namespace`] retains an optional decoded, case-sensitive
//! [`CssNamespacePrefix`], a literal [`CssNamespaceName`], and its parser-produced position.
//! Empty and non-URI names remain valid authored values; this crate does not normalize, resolve,
//! or load them. Selector type, universal, and attribute names expose
//! [`CssNamespaceConstraint`] and [`CssQualifiedSelectorName`]. `Named` requires an earlier active
//! prefix, `ExplicitNone` represents `|`, `Any` represents `*|`, and `Default` represents an
//! unqualified type or universal selector while a default declaration is active. Unqualified
//! attributes are always `ExplicitNone`.
//!
//! ```
//! use surgeist_css::{
//!     CssNamespaceConstraint, CssPseudoElement, CssPseudoElementSegment, CssRule, CssSelector, parse_sheet,
//! };
//!
//! let report = parse_sheet(concat!(
//!     "@namespace svg \"urn:svg\";",
//!     "svg|a#first#second[|lang]::first-line { color: red; }",
//! ));
//! assert!(report.is_clean());
//! let [CssRule::Namespace(namespace), CssRule::Style(style)] = report.syntax().rules() else {
//!     panic!("expected namespace and style rules");
//! };
//! assert_eq!(namespace.prefix().expect("named prefix").as_str(), "svg");
//! assert_eq!(namespace.name().as_str(), "urn:svg");
//!
//! let CssSelector::Compound(selector) = style.selectors().selectors()[0].selector() else {
//!     panic!("expected compound selector");
//! };
//! let qualified = selector.type_selector().expect("qualified type selector");
//! assert!(matches!(
//!     qualified.namespace(),
//!     CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
//! ));
//! assert_eq!(qualified.local_name(), Some("a"));
//! assert_eq!(selector.ids(), ["first", "second"]);
//! let [attribute] = selector.attributes() else {
//!     panic!("expected one attribute selector");
//! };
//! assert_eq!(attribute.namespace(), &CssNamespaceConstraint::ExplicitNone);
//! assert!(matches!(
//!     selector
//!         .pseudo_elements()
//!         .expect("pseudo-element sequence")
//!         .segments(),
//!     [CssPseudoElementSegment::PseudoElement(CssPseudoElement::FirstLine)]
//! ));
//! ```
//!
//! Initial layer statements may precede imports and namespaces, as specified by
//! [Cascade 5](https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#layer-empty).
//! Imports precede namespaces. A layer statement after either import or namespace
//! declarations, or a body rule, closes both prelude sequences. Invalid or ignored
//! rules do not change the phase or active bindings. Malformed, block-form, nested,
//! or late namespaces recover as one [`CssRecoveryAction::DropAtRule`].
//!
//! Complete Selectors 3 syntax includes all attribute matchers and four combinators, ordered
//! repeated IDs and classes, the structural/UI/dynamic pseudo-class families, `:lang()`, and
//! first-line/first-letter pseudo-elements. Legacy single-colon `before`, `after`, `first-line`,
//! and `first-letter` map to the same typed pseudo-elements. Undeclared namespace prefixes follow
//! consumer recovery contract: `:is()` and `:where()` retain an invalid member containing a
//! delimiter `&` as explicit match-nothing syntax and otherwise drop only that member,
//! while unforgiving style, scope, nesting, `:not()`, `:has()`, and nth `of` consumers drop their
//! established containing unit. Matching, specificity, cascade, namespace URI resolution,
//! CSSOM serialization, and cross-crate lowering remain downstream.
//! `:autofill` and required `:-webkit-autofill` share [`CssPseudoClass::Autofill`]
//! and emit the standard spelling. Nonfunctional unknown `::-webkit-*` names
//! retain checked [`CssUnknownWebkitPseudoElement`] identities with ASCII-lowercase
//! semantic names and escaped specified output. They remain symbolic match-nothing
//! pseudos subject to ordinary attachment, suffix and receiving-context restrictions;
//! unknown functional forms remain invalid. User-action suffixes on non-tree-abiding
//! pseudos retain the selected draft's at-risk status. No currently admitted pseudo
//! has internal combinator permission.
//! Pseudo 4 adds [`CssPseudoElement::Prefix`], [`CssPseudoElement::Suffix`],
//! [`CssPseudoElement::SearchText`], [`CssPseudoElement::TargetText`],
//! [`CssPseudoElement::SpellingError`], [`CssPseudoElement::GrammarError`],
//! [`CssPseudoElement::Highlight`], [`CssPseudoElement::Placeholder`],
//! [`CssPseudoElement::FileSelectorButton`] and [`CssPseudoElement::DetailsContent`].
//! Highlight retains one checked, case-preserving [`CssCustomIdent`]. FirstLetter
//! permits Prefix and Suffix children; recognized initial Prefix/Suffix names
//! retain their implied universal origin. Nonfunctional [`CssPseudoClass::Current`]
//! is admitted only in SearchText and element-backed suffix contexts, including
//! inherited logical arguments. Ordinary `:current`, `:current()`, `:past` and
//! `:future` remain unsupported. Part, FileSelectorButton and DetailsContent admit
//! otherwise supported pseudo syntax, with each following segment resetting the
//! receiving permissions. This syntax does not evaluate matching prohibitions,
//! live highlights or optional UA implementation of SearchText.
//! View Transitions 1 retains [`CssViewTransitionName`] as `none` or a checked,
//! case-sensitive [`CssViewTransitionIdent`], excluding `none` and `auto` from
//! the custom-name branch. [`CssPseudoElement::ViewTransition`] and its Group,
//! ImagePair, Old and New functional variants retain wildcard or custom-ident
//! [`CssViewTransitionNameSelector`] arguments. Named descendants add OnlyChild
//! to generic suffix permission, inherited by Not/Is/Where. Exact compounds follow
//! root → group → image-pair → old/new, and Slotted admits all five tree-abiding
//! identities. Participant discovery, matching and live transitions belong downstream.
//!
//! # Counter styles and page rules
//!
//! [`CssRule::CounterStyle`] retains a checked, case-sensitive [`CssCounterStyleName`], every
//! valid descriptor occurrence in authored order, the effective last valid occurrence of each
//! descriptor, and the rule position. The typed descriptor values cover Counter Styles 3
//! `system`, `negative`, `prefix`, `suffix`, `range`, `pad`, `fallback`, `symbols`,
//! `additive-symbols`, and `speak-as`. Invalid individual descriptors recover locally.
//! A grammar-valid collection can still fail to define a counter style; the separate
//! [`CssCounterStyleDefinitionStatus`] records that distinction. Prospective descriptor
//! entries retain real named occurrences or honestly source-free programmatic values.
//!
//! [`CssRule::Page`] retains an empty prelude or a complete named and compound
//! [`CssPageSelectorList`], including comma lists and logical `:recto`/`:verso`.
//! Selectors expose the Page-specific three-component specificity. Page declarations
//! preserve a distinct typed descriptor domain (`size`, `page-orientation`, `marks`,
//! `bleed`) alongside applicable shared properties and custom values. Sixteen canonical
//! margin-box rule names retain ordered children. Shared property grammars and symbolic
//! values are reused, including font-relative lengths; no layout or variable lookup runs.
//! [`parse_page_property_value`], [`parse_page_descriptor_value`], and [`parse_margin_block`]
//! check detached inputs with genuine origins. Checked selected blocks and borrowed
//! [`CssPageRuleView`] support immutable edits without re-expanding authored shorthands.
//! Page and margin CSSOM output follows the adopted Blink whole-rule wrapper under one
//! cumulative atomic budget. Page matching, pagination, cascade, counter registration,
//! inheritance resolution, and generated-marker execution remain downstream.
//!
//! ```
//! use surgeist_css::{CssCounterStyleSystem, CssPagePseudo, CssRule, parse_sheet};
//!
//! let report = parse_sheet(concat!(
//!     "@counter-style digits { system: numeric; symbols: \"0\" \"1\"; suffix: \".\"; } ",
//!     "@page :left { margin-left: -12mm; margin-right: 10%; }",
//! ));
//! assert!(report.is_clean());
//! let [CssRule::CounterStyle(counter), CssRule::Page(page)] = report.syntax().rules() else {
//!     panic!("expected counter-style and page rules");
//! };
//! assert_eq!(counter.name().as_str(), "digits");
//! assert!(matches!(
//!     counter.descriptors().system().map(|value| value.value().view()),
//!     Some(surgeist_css::CssCounterStyleDescriptorValueRef::System(&CssCounterStyleSystem::Numeric))
//! ));
//! assert_eq!(counter.descriptors().occurrences().count(), 3);
//! assert_eq!(page.selectors().selectors()[0].pseudos(), &[CssPagePseudo::Left]);
//! assert_eq!(page.declarations().properties().len(), 2);
//! ```
//!
//! # Residual official properties and legacy orientation
//!
//! The C12 family exposes complete typed authored grammars for thirteen CSS2 residual
//! properties; Writing Modes 3 text combination, orientation, and bidi properties; UI3
//! outline-offset and UI4 caret-color and resize; Containment 2 `contain`; Transforms 1
//! `transform-box`; and Compositing 1 blend and isolation properties. These values retain
//! authored syntax and parser coordinates without applying cascade, layout, pagination,
//! painting, containment semantics, blending, hit testing, or writing-mode resolution.
//!
//! [`CssParserContext::with_svg_glyph_orientation_vertical`] explicitly selects
//! the independent frozen SVG authored definition. Its Auto/Angle/implied-degree
//! Number body retains actual document mode and presentation-attribute admission
//! through originals, pending reentry and one independent terminal contribution.
//! Explicit finite grammar handles keep the Writing Modes meaning. Fixed
//! attribute-value fronts use Normal importance; root owns markup binding.
//! Specified Unitless output becomes degrees using shared numeric precision,
//! without computed quadrant rounding. See the crate's reference for source
//! reconciliation, canonical reparse policy and unresolved normative clauses.
//!
//! Ordinary `glyph-orientation-vertical` is an explicit restricted legacy shorthand that maps to a
//! parser-produced [`CssKnownProperty::TextOrientation`] value. It is not a name-equivalent
//! schema alias: [`CssKnownProperty::aliases`] remains empty for `TextOrientation`, while
//! [`feature_metadata`] exposes its distinct [`CssFeatureKind::PropertyAlias`] record. That
//! record is partial: the admitted literal mapping is supported, while numeric-terminal
//! spelling and math applicability remain unresolved by the selected standards. The shared
//! box-edge and blend-mode productions have independent complete records.
//!
//! `background-blend-mode`, `isolation`, and `mix-blend-mode` provide noninherited
//! intrinsic initials of one `normal` list entry, `auto`, and `normal`. Their
//! [`CssLonghandValueRef`] variants borrow the existing typed values. Expansion,
//! pending-substitution reentry, and normalization retain authored blend-list
//! order, duplicates, source occurrences, and declaration importance. Checked
//! admission requires complete original components, while browser recovery stays
//! observable in the parse report. Specified output and normalization use
//! cumulative limits and return no partial public result on failure.
//!
//! ```
//! use surgeist_css::{
//!     CssBlendMode, CssFeatureKind, CssKnownProperty, CssKnownPropertyValueRef,
//!     CssSupportStatus, feature_metadata, parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute(concat!(
//!     "border-spacing: 2px 3px; ",
//!     "glyph-orientation-vertical: 90; ",
//!     "background-blend-mode: multiply, luminosity",
//! ));
//! assert!(report.is_clean());
//! assert_eq!(
//!     report.syntax()[1].known().expect("legacy shorthand").property(),
//!     CssKnownProperty::TextOrientation,
//! );
//! let CssKnownPropertyValueRef::BackgroundBlendMode(blending) = report.syntax()[2]
//!     .known().expect("known blending property")
//!     .property_value().expect("ordinary value")
//! else { panic!("expected background blend modes") };
//! assert_eq!(
//!     blending.modes().modes(),
//!     &[CssBlendMode::Multiply, CssBlendMode::Luminosity],
//! );
//! let alias = feature_metadata("official.property-alias.glyph-orientation-vertical")
//!     .expect("legacy alias metadata");
//! assert_eq!(alias.kind(), CssFeatureKind::PropertyAlias);
//! assert_eq!(alias.status(), CssSupportStatus::Partial);
//! ```
//!
//! # Authored when and else groups
//!
//! [`CssWhenCondition`] owns the adopted symbolic grammar: homogeneous boolean
//! terms, explicit groups, single media-feature and supports-declaration leaves,
//! and valid opaque enclosures. [`parse_when_condition`] returns checked syntax
//! without evaluation. [`CssWhenRule`] and [`CssElseRule`] retain ordinary children;
//! their scoped counterparts retain the separate scoped grammar. Parsed Else
//! chains require original-token adjacency after complete recovery reconstruction.
//! Checked enclosing assembly instead validates actual supplied sibling order.
//! Normalization retains immutable branch headers and nearest style contexts.
//! Shared specified output composes the graph under cumulative limits; ordinary
//! literal CSSOM returns [`CssRuleCssomKind::When`] or [`CssRuleCssomKind::Else`]
//! as source-undefined. The current Scope literal front remains format-unavailable.
//! See the [adopted authored profile](https://github.com/bj-data-eng/surgeist/issues/647).
//!
//! # Media, supports, imports, and prelude recovery
//!
//! Media syntax preserves unknown conditions separately from malformed-member recovery.
//! A structurally valid unknown feature or value is retained as
//! [`CssMediaConditionKind::UnknownFeature`] with no diagnostic. Arbitrary checked enclosures
//! use [`CssMediaConditionKind::GeneralEnclosed`] when no preceding grammar branch applies.
//! Both preserve unknown truth under negation; query evaluation belongs downstream.
//! A reserved or structurally
//! malformed list member becomes [`CssMediaQuery::Never`] and emits
//! [`CssRecoveryAction::ReplaceMediaQueryWithNever`], allowing later comma siblings to survive.
//!
//! ```
//! use surgeist_css::{
//!     CssMediaConditionKind, CssMediaQuery, CssRecoveryAction, CssRule, parse_sheet,
//! };
//!
//! let report = parse_sheet("@media (future-mode: active), ???, print {}");
//! let [CssRule::Media(media)] = report.syntax().rules() else {
//!     panic!("expected retained media rule");
//! };
//! assert!(matches!(
//!     media.query().queries(),
//!     [
//!         CssMediaQuery::Condition(condition),
//!         CssMediaQuery::Never(_),
//!         CssMediaQuery::Typed(_),
//!     ] if matches!(condition.kind(), CssMediaConditionKind::UnknownFeature(_))
//! ));
//! assert!(matches!(
//!     report.diagnostics(),
//!     [diagnostic]
//!         if diagnostic.action() == CssRecoveryAction::ReplaceMediaQueryWithNever
//! ));
//! ```
//!
//! Supports rules retain declaration tests, boolean grouping, complete Selectors 3 plus the
//! selected existing selector extensions as the typed `selector()` subset, and balanced
//! general-enclosed fallback syntax. Unselected Selectors 4 pseudo-classes and
//! pseudo-elements, and syntax outside the named extension rows remain outside the typed subset.
//! These nodes describe authored tests; the crate never evaluates whether a condition matches.
//! `font-tech()` retains one shared [`CssFontTechHint`], `font-format()` retains
//! the distinct keyword or string [`CssFontFormat`], and `at-rule()` retains a
//! genuine at-keyword argument through [`CssSupportsAtRule`]. Unknown names and
//! `@charset` remain authored predicates. Balanced invalid arguments use the
//! general-enclosed fallback; lexical and resource failures remain terminal.
//!
//! ```
//! use surgeist_css::{CssRule, CssSupportsConditionKind, parse_sheet};
//!
//! let report = parse_sheet(concat!(
//!     "@supports selector(.card > .item:hover) {}",
//!     "@supports future-layout(mode) {}",
//! ));
//! assert!(report.is_clean());
//! let [CssRule::Supports(selector), CssRule::Supports(fallback)] =
//!     report.syntax().rules()
//! else {
//!     panic!("expected supports rules");
//! };
//! assert!(matches!(
//!     selector.condition().kind(),
//!     CssSupportsConditionKind::Selector(_)
//! ));
//! assert!(matches!(
//!     fallback.condition().kind(),
//!     CssSupportsConditionKind::GeneralEnclosed(value)
//!         if value.authored() == Some("future-layout(mode)")
//! ));
//! ```
//!
//! An import prelude is parsed in target, optional `layer`, optional `supports()`, optional media
//! order. A successful initial layer statement may precede imports; a later body rule closes the
//! import phase. Invalid order or clauses drop only the import and leave later siblings eligible.
//! Import targets and conditions remain symbolic: URL resolution, resource loading, condition
//! evaluation, cascade, selector matching, and root/sibling lowering are downstream work.
//!
//! ```
//! use surgeist_css::{CssImportLayer, CssRule, CssSupportsConditionKind, parse_sheet};
//!
//! let report = parse_sheet(concat!(
//!     "@layer reset; ",
//!     "@import url(theme.css) layer(theme) supports(display: grid) print;",
//! ));
//! assert!(report.is_clean());
//! let [CssRule::LayerStatement(_), CssRule::Import(import)] = report.syntax().rules() else {
//!     panic!("expected initial layer and import");
//! };
//! assert!(matches!(import.layer(), Some(CssImportLayer::Named(_))));
//! assert!(matches!(
//!     import.supports().expect("supports clause").condition().kind(),
//!     CssSupportsConditionKind::Declaration(_)
//! ));
//! assert!(import.media().is_some());
//! ```
//!
//! # Diagnostics and coordinates
//!
//! Each [`CssRecoveryDiagnostic`] exposes a typed [`ErrorKind`] and stable
//! [`CssErrorCode`], the first responsible [`CssSourcePosition`], the complete
//! [`CssSourceSpan`] of the recovery unit, and the [`CssRecoveryAction`] taken.
//! Byte offsets index the original UTF-8 input. Lines and columns are zero-based,
//! and columns count UTF-16 code units. Display and debug prose are for people;
//! control flow should match typed variants and include a wildcard for every
//! non-exhaustive enum. Closed enums such as [`CssImportance`] and
//! [`CssSupportStatus`] remain exhaustively matchable.
//!
//! Evolving authored-syntax enums intentionally require a wildcard in external
//! matches. These representative exhaustive matches therefore do not compile:
//!
//! ```compile_fail
//! use surgeist_css::CssMediaQueryModifier;
//!
//! fn describe(value: CssMediaQueryModifier) -> &'static str {
//!     match value {
//!         CssMediaQueryModifier::Not => "not",
//!         CssMediaQueryModifier::Only => "only",
//!     }
//! }
//! ```
//!
//! ```compile_fail
//! use surgeist_css::CssSelectorCombinator;
//!
//! fn describe(value: CssSelectorCombinator) -> &'static str {
//!     match value {
//!         CssSelectorCombinator::Descendant => "descendant",
//!         CssSelectorCombinator::Child => "child",
//!         CssSelectorCombinator::NextSibling => "next",
//!         CssSelectorCombinator::SubsequentSibling => "subsequent",
//!     }
//! }
//! ```
//!
//!
//! ```compile_fail
//! use surgeist_css::CssAnimationDirection;
//!
//! fn describe(value: CssAnimationDirection) -> &'static str {
//!     match value {
//!         CssAnimationDirection::Normal => "normal",
//!         CssAnimationDirection::Reverse => "reverse",
//!         CssAnimationDirection::Alternate => "alternate",
//!         CssAnimationDirection::AlternateReverse => "alternate-reverse",
//!     }
//! }
//! ```
//!
//! ```compile_fail
//! use surgeist_css::CssGridAutoFlowAxis;
//!
//! fn describe(value: CssGridAutoFlowAxis) -> &'static str {
//!     match value {
//!         CssGridAutoFlowAxis::Row => "row",
//!         CssGridAutoFlowAxis::Column => "column",
//!     }
//! }
//! ```
//!
//! ```compile_fail
//! use surgeist_css::{CssPredefinedColorSpace, CssRelativeColorFunction};
//!
//! fn describe(value: CssRelativeColorFunction) -> &'static str {
//!     match value {
//!         CssRelativeColorFunction::Rgb => "rgb",
//!         CssRelativeColorFunction::Hsl => "hsl",
//!         CssRelativeColorFunction::Hwb => "hwb",
//!         CssRelativeColorFunction::Lab => "lab",
//!         CssRelativeColorFunction::Lch => "lch",
//!         CssRelativeColorFunction::Oklab => "oklab",
//!         CssRelativeColorFunction::Oklch => "oklch",
//!         CssRelativeColorFunction::Color(CssPredefinedColorSpace::Srgb) => "srgb",
//!         CssRelativeColorFunction::Color(_) => "other color space",
//!     }
//! }
//! ```
//!
//! # Backgrounds, border images, and gradients
//!
//! Background and image values preserve authored layer, image, gradient, stop,
//! border-image, and object-sizing structure. Selected Images 4 additions admit
//! conic gradients, explicit shared [`CssColorInterpolation`], singleton lists,
//! and two authored positions per stop. [`CssGradientColorStop::positions`]
//! retains linear positions; [`CssAngularColorStopPosition`] retains conic angle,
//! percentage, exact zero, and [`CssAnglePercentageCalculation`] branches.
//! Radial and conic `at` fields use the full symbolic [`CssPosition`] owner.
//! They do not resolve URLs, load or
//! decode images, compute geometry, or paint.
//! `background-color` contributes one noninherited [`CssColor`] with transparent
//! intrinsic initial. Its existing color serializer retains symbolic dependencies;
//! CSS-wide keywords remain symbolic and pending values reenter the color grammar.
//! Background and mask size/repeat longhands expose ordered semantic lists through
//! `sizes()` and `repeats()`; background origin/clip expose all authored boxes
//! through `boxes()`, and background attachment exposes `attachments()`.
//! Each of these five background properties expands to one noninherited longhand,
//! retaining its complete ordered list. Initial layers are `auto`, `repeat`,
//! `padding-box`, `border-box`, and `scroll`, respectively. The checked scalar and
//! list owners expose bounded specified serialization: equal repeat axes and
//! equivalent repeat aliases use their shorter form. Sizes emit the effective
//! `auto` height after a non-auto width; `auto auto` collapses to `auto`. Authored
//! height omissions remain unchanged. Every layer shares one cumulative
//! serialization budget; a generated height adds projection and output bytes,
//! without adding an authored input node.
//!
//! ```
//! use surgeist_css::{
//!     CssGradient, CssImageValue, CssKnownPropertyValueRef, CssSupportStatus,
//!     feature_metadata, parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute(concat!(
//!     "background-image: linear-gradient(to right, red 0%, 40%, blue); ",
//!     "border-image: url(frame.png) 10 fill / 2 / 1 round",
//! ));
//! assert!(report.is_clean(), "{:?}", report.diagnostics());
//!
//! let CssKnownPropertyValueRef::BackgroundImage(images) = report.syntax()[0]
//!     .known().expect("known background image")
//!     .property_value().expect("ordinary background image")
//! else { panic!("expected background-image") };
//! assert!(matches!(
//!     images.images().images(),
//!     [CssImageValue::Gradient(CssGradient::Linear(_))]
//! ));
//!
//! let CssKnownPropertyValueRef::BorderImage(border) = report.syntax()[1]
//!     .known().expect("known border image")
//!     .property_value().expect("ordinary border image")
//! else { panic!("expected border-image") };
//! assert!(border.border_image().slice().expect("slice").fill());
//! assert_eq!(
//!     border.border_image().serialize_specified().unwrap(),
//!     "url(\"frame.png\") 10 fill / 2 / 1 round",
//! );
//!
//! let gradient = feature_metadata("official.value.linear-gradient")
//!     .expect("linear-gradient metadata");
//! assert_eq!(gradient.source().id().as_str(), "O-IMAGES3");
//! assert_eq!(gradient.status(), CssSupportStatus::Complete);
//! ```
//!
//! The public catalog exposes all 27 C13 property/value records as Complete and
//! retains Complete metadata for `background-position`, `object-position`, and
//! `box-shadow`. The 33 previously Partial Backgrounds/Borders property records
//! are Complete as well. These metadata transitions do not add official ledger
//! units.
//!
//! # Flexbox, multicolumn, and official grammar closure
//!
//! Flexbox 1 `flex-flow` and all nine Multicolumn 1 properties expose typed
//! authored values without performing layout, pagination, or painting.
//! The Sizing 4 `column-width` extension uses the shared `CssSizeValue` box-size
//! grammar; `CssColumns` exposes both effective values and exact positive counts
//! retain ordinary digits beyond `i32`. The generic Syntax 3 authored shells
//! and the remaining selected Values 3 records
//! are public Complete atomic metadata.
//!
//! ```
//! use surgeist_css::{
//!     CssColumnCount, CssFlexDirection, CssKnownPropertyValueRef, CssSupportStatus,
//!     feature_metadata, parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute("flex-flow: column wrap; columns: 3 12em");
//! assert!(report.is_clean(), "{:?}", report.diagnostics());
//!
//! let CssKnownPropertyValueRef::FlexFlow(flow) = report.syntax()[0]
//!     .known().expect("known flex-flow")
//!     .property_value().expect("ordinary flex-flow")
//! else { panic!("expected flex-flow") };
//! assert_eq!(flow.flow().direction(), CssFlexDirection::Column);
//!
//! let CssKnownPropertyValueRef::Columns(columns) = report.syntax()[1]
//!     .known().expect("known columns")
//!     .property_value().expect("ordinary columns")
//! else { panic!("expected columns") };
//! assert!(matches!(columns.columns().count(), CssColumnCount::Count(_)));
//!
//! let metadata = feature_metadata("official.property.flex-flow")
//!     .expect("public Flexbox metadata");
//! assert_eq!(metadata.source().id().as_str(), "O-FLEXBOX1");
//! assert_eq!(metadata.status(), CssSupportStatus::Complete);
//!
//! let shared = feature_metadata("official.value.syntax-token-stream")
//!     .expect("public Syntax 3 value metadata");
//! assert_eq!(shared.status(), CssSupportStatus::Complete);
//!
//! let extension = feature_metadata("ext.value.relative-color")
//!     .expect("preserved extension metadata");
//! assert_eq!(extension.status(), CssSupportStatus::Partial);
//! assert!(extension.supported_subset().is_some());
//! assert!(extension.unsupported_remainder().is_some());
//!
//! let feature_values = feature_metadata("later.rule.font-feature-values")
//!     .expect("authored font-feature-values metadata");
//! assert_eq!(feature_values.status(), CssSupportStatus::Complete);
//! assert_eq!(feature_values.source().id().as_str(), "I-FONTS4-20260907");
//! assert!(feature_values.unsupported_remainder().is_none());
//! ```
//!
//! C14 makes all 31 records that entered the cycle as Reserved public Complete
//! catalog entries: ten Flexbox and Multicolumn properties; seven generic rule,
//! declaration, and list shells; and fourteen shared values—`syntax-token-stream`,
//! `component-value`, `simple-block`, `function`, `declaration-value`, `any-value`,
//! `an-plus-b`, `unicode-range`, `css-wide-keyword`, `custom-ident`, `ident`,
//! `string`, `url`, and `url-modifier`. The first two groups are 17 shell/property
//! additions, and the last group is 14 shared-value additions. It separately
//! promotes the seven Values 3 records for `dimension`, `angle`,
//! `angle-percentage`, `time-percentage`, `frequency`, `frequency-percentage`, and
//! `calc()` from Partial to Complete.
//!
//! The preserved `ext.value.relative-color`, `ext.value.color-mix`,
//! `ext.descriptor.font-weight-range`, `ext.descriptor.font-style-oblique-range`,
//! `ext.descriptor.font-stretch-range`, `ext.value.font-source-modern-hints`,
//! `ext.property.font-weight-range`, `ext.supports.selector`,
//! `ext.media.range.width`, `ext.media.range.height`,
//! `ext.media.range.resolution`, `ext.media.range.color`, and
//! `ext.media.range.monochrome` records remain Partial with explicit subset and
//! remainder metadata. `@font-face` and `@font-feature-values` provide complete
//! selected authored rule construction and effective specified serialization.
//! The Fonts 4 index-source contradiction remains explicit in the reference. C13's
//! 456 public catalog records plus C14's 31 additions reached 487 at that point;
//! subsequent selected records extend the catalog. [`feature_catalog`] owns its
//! current cardinality, which is distinct from the immutable official
//! inventory of exactly 162 property units, one normative legacy shorthand, and
//! 167 non-property units. The 219 I01 baseline records retained their
//! classifications at that checkpoint; later authored-family completion refines
//! individual records. The exclusion registry contains exactly 130 rows.
//!
//! # Support metadata and application policy
//!
//! [`feature_catalog`] describes each declared conformance production as
//! [`CssSupportStatus::Complete`], [`CssSupportStatus::Partial`], or
//! [`CssSupportStatus::RecognizedUnsupported`]. Partial records state both their
//! accepted subset and scope not claimed as supported. A remainder may describe
//! known-valid unimplemented syntax or unresolved standard applicability; it does
//! not establish that every other form is valid CSS. A diagnostic-free use of a
//! partial production's accepted subset is still a clean parse.
//!
//! The source registry assigns every selected dated specification or preserved
//! repository baseline a stable [`CssSpecificationSourceId`], module, level, and
//! [`CssSpecificationTier`]. A tier classifies provenance only; it never implies
//! parser support. Each source has exactly one immutable specification URL or
//! repository provenance value. [`specification_source`], [`feature_metadata`],
//! and [`conformance_exclusion`] use exact, case-sensitive IDs without trimming
//! or aliasing.
//!
//! ```
//! use surgeist_css::{
//!     CssExclusionReason, CssSpecificationTier, CssSupportStatus,
//!     conformance_exclusion, feature_metadata, specification_source,
//! };
//!
//! let color = specification_source("O-COLOR4").expect("dated Color 4 source");
//! assert_eq!(color.tier(), CssSpecificationTier::Snapshot2026Official);
//! assert!(specification_source("o-color4").is_none());
//!
//! let importance = feature_metadata("foundation.declaration.importance")
//!     .expect("atomic parser-facing record");
//! assert_eq!(importance.status(), CssSupportStatus::Complete);
//! assert!(importance.baseline_alias_targets().is_empty());
//!
//! let pseudo_elements = feature_metadata("baseline.selector.pseudo-element")
//!     .expect("preserved aggregate alias");
//! assert_eq!(
//!     pseudo_elements.baseline_alias_targets()[0].as_str(),
//!     "official.selector.generated",
//! );
//!
//! let processing = conformance_exclusion("excluded.O-IMAGES3.processing")
//!     .expect("official source exclusion");
//! assert_eq!(
//!     processing.reason(),
//!     CssExclusionReason::OutsideAuthoredSyntaxBoundary,
//! );
//! ```
//!
//! An atomic feature record is parser-facing and has one truthful support
//! status. The four preserved baseline aggregate aliases remain queryable and
//! expose immutable atomic target slices, but they do not own parser dispatch.
//! Private reserved coverage slots describe later grammar boundaries only: they
//! are not feature records, carry no support status, and do not make their
//! spellings recognized. [`conformance_exclusions`] records informative,
//! superseded, and out-of-boundary official source items separately; exclusions
//! carry no support status and never change parser diagnostics. These metadata
//! and inventory boundaries do not change accepted CSS, retained syntax,
//! diagnostics, positions, spans, or recovery actions.
//!
//! [`validate_sheet`] and [`validate_style_attribute`] are always available.
//! Each validator consumes ordinary parsing semantics
//! and its report, accepts exactly a clean report, and otherwise preserves the
//! complete non-empty diagnostic sequence in [`CssValidationFailure`]. The
//! validation step does not select a second grammar or change ordinary parsing.
//!
//! # Immutable stylesheet normalization
//!
//! [`normalize_sheet`] preserves an ordered stream of rule occurrences and
//! grouped declaration contributions. [`normalize_report`] also retains every
//! original recovery diagnostic. Shared rule and selector contexts preserve
//! nesting, empty rules, conditions, imports, layers, scopes, and complete
//! nonstyle payloads without multiplying selectors or selecting cascade winners.
//! The current property-expansion slice remains explicit: valid declarations
//! outside it fail atomically with [`CssNormalizationError`].
//!
//! ```
//! use surgeist_css::{CssNormalizedItem, normalize_report, parse_sheet};
//!
//! let report = parse_sheet(".a, #b { margin-block: 1px; & .child { padding-block: 2px } }");
//! let normalized = normalize_report(&report).expect("supported expansion");
//! assert!(normalized.is_clean());
//! assert_eq!(normalized.syntax().items().len(), 4);
//! let [CssNormalizedItem::Rule(_), CssNormalizedItem::Declaration(parent),
//!      CssNormalizedItem::Rule(_), CssNormalizedItem::Declaration(child)] =
//!     normalized.syntax().items() else { unreachable!() };
//! assert!(child.selector_context().parent().unwrap()
//!     .same_context(parent.selector_context()));
//! ```
//!
//! # Authored Text Decoration
//!
//! The selected effective Decoration 4 grammar covers decoration, underline,
//! skipping, emphasis and text-shadow. All roles participate in intrinsic
//! metadata, shorthand expansion, strict construction and pending reentry.
//! The decoration shorthand sets Line, Thickness, Style and Color only; skip
//! sets Self, Box, Inset, Spaces and Ink; emphasis sets Style and Color only.
//! CSS-wide values and pending values stay symbolic, with original components.
//! [`CssTextEmphasisMark`] preserves fill/shape omissions and origins;
//! [`CssTextEmphasisPosition`] preserves an omitted Right side. Canonical output
//! omits assumed Filled before a shape and explicit Right after a vertical side.
//! Strings remain untruncated and font-dependent shape selection stays downstream.
//! [`CssTextShadowLayer`] checks nonnegative text spread and original closure,
//! reusing the box-shadow provider while preserving its signed-spread policy.
//!
//! ```
//! use surgeist_css::{CssTextEmphasisFill, CssTextEmphasisMark, CssTextEmphasisShape,
//!     CssTextEmphasisPosition, CssTextEmphasisVertical, CssTextSide};
//! let mark = CssTextEmphasisMark::try_new(Some(CssTextEmphasisFill::Filled),
//!     Some(CssTextEmphasisShape::Dot)).unwrap();
//! assert_eq!(mark.serialize_specified().unwrap(), "dot");
//! assert_eq!(mark.fill(), Some(CssTextEmphasisFill::Filled));
//! let position = CssTextEmphasisPosition::new(CssTextEmphasisVertical::Over, None);
//! assert_eq!(position.side(), None);
//! assert_eq!(position.effective_side(), CssTextSide::Right);
//! assert_eq!(position.serialize_specified().unwrap(), "over");
//! ```
//!
//! # Boundary
//!
//! This crate owns authored CSS syntax, intrinsic grammar validation, recovery
//! boundaries, diagnostic provenance, and support metadata. It does not apply
//! cascade or inheritance; substitute custom properties; validate computed
//! post-substitution values; evaluate queries; match selectors; resolve URLs,
//! resources, units, or colors; perform layout, painting, or animation; expose a
//! mutable CSSOM; or lower CSS into sibling Surgeist types.

mod background_layer_serialization;
mod background_serialization;
mod border_color;
mod border_image_serialization;
mod clip_path_serialization;
mod color_adjustment;
mod motion_serialization;
mod shape_outside_serialization;
pub use color_adjustment::{
    CssColorScheme, CssColorSchemeConstructionError, CssColorSchemeKeyword, CssColorSchemeName,
    CssForcedColorAdjust, CssPrintColorAdjust,
};
mod block_fragments;
mod color_profile;
mod color_profile_serialization;
mod common_serialization;
mod component_values;
mod conformance;
mod content_serialization;
mod content_values;
mod counter_changes;
mod counter_changes_serialization;
mod counter_definition;
mod counter_style_serialization;
pub use counter_definition::{
    CssCounterStyleAlgorithm, CssCounterStyleDefinitionIssue, CssCounterStyleDefinitionStatus,
    CssCounterStyleDescriptorCollection, CssCounterStyleDescriptorEntry,
};
mod descriptor_values;
pub use block_fragments::{CssBlockFragment, CssRuleList, CssStyleAncestor};
mod stylesheet_input;
mod syntax_consumption;
pub use descriptor_values::{
    CssCounterStyleDescriptorValue, CssCounterStyleDescriptorValueRef, CssCounterStyleValueError,
    CssCounterStyleValueErrorKind, CssFontFeatureDisplayValue, CssFontFeatureDisplayValueRef,
    CssFontFeatureValue, CssFontFeatureValueError, CssFontFeatureValueErrorKind,
    CssFontFeatureValueRef,
};
mod cursor_serialization;
mod cursor_values;
mod declaration_serialization;
mod easing_serialization;
mod error;
mod expansion;
mod filter_serialization;
mod font_face_declaration_block;
mod font_face_values;
mod font_family_serialization;
mod font_feature_values;
mod font_palette;
mod font_palette_serialization;
mod font_palette_values;
mod font_rule_serialization;
pub use font_face_declaration_block::{
    CssFontFaceDeclarationBlockError, CssSpecifiedFontFaceDeclarationBlock,
};
mod font_source_serialization;
mod font_synthesis;
mod font_variant;
mod font_variant_serialization;
mod image_orientation_serialization;
mod image_serialization;
mod imports;
mod keyword_property_serialization;
mod list_style_serialization;
mod list_styles;
mod misc_property_serialization;
mod outline_serialization;
mod quotes;
mod shape_serialization;
mod specified_declaration_block;
mod specified_inverse;
mod text_value_serialization;
mod timed_property_serialization;
mod transform_value_serialization;
pub use imports::*;
mod custom_media;
pub use custom_media::*;
mod named_supports;
pub use named_supports::*;
mod container;
mod container_features;
mod container_properties;
pub use container_properties::*;
mod container_scroll;
pub use container_scroll::*;
mod container_style;
pub use container_features::*;
pub use container_style::*;
mod when;
pub use when::*;
mod supports;
pub use supports::{CssSupportsAtRule, CssSupportsConstructionError};
mod cssom_rule_serialization;
mod edited_rule;
pub use edited_rule::{CssEditedGroupPreludeRef, CssEditedRuleView, CssRuleGraphInput};
mod keyframe_serialization;
pub use keyframe_serialization::{
    CssKeyframeRuleView, CssKeyframeRuleViewError, CssKeyframesRuleView,
};
mod media;
mod named_supports_serialization;
mod page_keyframe_serialization;
mod page_projection;
mod page_serialization;
pub use page_projection::{
    CssMarginRuleView, CssPageProjectionError, CssPageRuleView, CssSpecifiedPageDeclarationBlock,
    CssSpecifiedPageDeclarationEntry,
};
mod query_rule_serialization;
mod rule_construction;
mod selector_serialization;
pub use selector_serialization::CssScopeSelectorCssomContext;
mod selector_anchors;
mod shadow_serialization;
#[cfg(test)]
mod specified_provider_composition_tests;
mod specified_rule_graph;
mod specified_rule_serialization;
pub use rule_construction::{CssRuleConstructionError, CssRuleConstructionErrorKind};
mod url_serialization;
pub use media::*;
mod media_features;
mod normalization;
pub use media_features::*;
mod alignment;
mod angle;
mod aspect_ratio;
mod border_radius;
mod border_spacing;
mod border_style;
mod border_width;
mod box_spacing;
mod break_controls;
mod calc_size;
mod clip;
mod color_alpha;
mod color_scalar;
mod column_rule;
mod column_sizing;
mod contain_intrinsic_size;
mod display;
mod exact_decimal;
mod frequency;
mod gap;
mod grid_template_areas;
mod hsl_color_conversion;
mod inset;
mod integer_value;
mod item_flow;
mod lab_color_conversion;
mod naive_color_conversion;
mod numeric;
mod numeric_formatting;
mod opacity_scalar;
mod page_line_minimum;
mod pending_serialization;
mod position_serialization;
mod rectangular_color_conversion;
mod resolution;
mod rotate;
mod scroll_snap;
mod serialization_escaping;
mod sizing;
mod sizing_controls;
mod speech;
pub use speech::{
    CssAudioCue, CssCue, CssCuePair, CssDecibelLiteral, CssGenericVoice, CssSemitoneLiteral,
    CssSpeak, CssSpeakAs, CssSpeakAsPunctuation, CssSpeechBreak, CssSpeechBreakPair,
    CssSpeechBreakStrength, CssVoiceAge, CssVoiceBalance, CssVoiceBalanceKeyword, CssVoiceDuration,
    CssVoiceFamily, CssVoiceFamilyEntry, CssVoiceFamilyList, CssVoiceFamilyName,
    CssVoiceFamilyNameRef, CssVoiceGender, CssVoiceLevel, CssVoiceOffset, CssVoicePitchRange,
    CssVoiceRate, CssVoiceRateKeyword, CssVoiceStress, CssVoiceVolume, CssVoiceVolumeLevel,
};
mod image_1d;
mod text_alignment;
mod text_box;
mod text_controls;
mod text_decoration;
mod text_spacing;
mod time;
mod ui;
mod ui_serialization;
pub use alignment::{
    CssAlignContentValue, CssAlignItemsValue, CssAlignSelfValue, CssAlignmentPosition,
    CssAlignmentValue, CssBaselinePosition, CssJustifyContentValue, CssJustifyItemsValue,
    CssJustifySelfValue, CssLegacyAlignment, CssOverflowPosition, CssPlaceContentValue,
    CssPlaceItemsValue, CssPlaceSelfValue,
};
pub use angle::{CssAngleLiteral, CssAngleOrZero, CssAngleValue, CssZeroLiteral};
pub use aspect_ratio::{CssRatioOperand, CssSpecifiedRatio};
pub use border_color::{CssBorderColorPair, CssBorderColorShorthand, CssBorderColors};
pub use border_radius::{CssBorderRadiusShorthand, CssCornerRadiusValue};
pub use border_style::{CssBorderStylePair, CssBorderStyleShorthand};
pub use border_width::{CssBorder, CssBorderWidth, CssBorderWidthPair, CssBorderWidthShorthand};
pub use box_spacing::{
    CssBoxSideKind, CssMarginPair, CssMarginShorthand, CssMarginValue, CssPaddingPair,
    CssPaddingShorthand, CssPaddingValue,
};
pub use calc_size::{CssBoxCalcSize, CssCalcSize, CssCalcSizeBasisRef, CssIntrinsicSizeKeyword};
pub use color_alpha::{CssColorAlphaScalarRef, CssParsedColorAlphaRef};
pub use color_scalar::{
    CssColorNumberLiteral, CssColorPercentageLiteral, CssColorScalarError, CssColorScalarErrorKind,
};
pub use contain_intrinsic_size::{
    CssContainIntrinsicSize, CssContainIntrinsicSizeFallback, CssContainIntrinsicSizeValue,
};
pub use display::{
    CssDisplayBox, CssDisplayInside, CssDisplayInternal, CssDisplayLegacy,
    CssDisplayListItemInside, CssDisplayOutside, CssDisplayValue,
};
pub use frequency::{CssFrequencyLiteral, CssFrequencyValue};
pub use gap::{CssGapShorthand, CssGapValue};
pub use grid_template_areas::{
    CssGridTemplateAreaCell, CssGridTemplateAreaError, CssGridTemplateAreaName,
    CssGridTemplateAreaRow, CssGridTemplateAreaRows, CssGridTemplateAreas,
};
pub use image_1d::*;
pub use resolution::{CssResolutionLiteral, CssResolutionValue};
pub use time::{CssDuration, CssTimeLiteral, CssTimeValue};
pub use ui::*;
mod font_width;
pub use font_width::{CssFontFaceWidth, CssFontWidth, CssFontWidthKeyword};
mod font_controls;
pub use font_controls::{
    CssFontKerning, CssFontLanguageOverride, CssFontLanguageString, CssFontOpticalSizing,
    CssFontSizeAdjust,
};
mod font_settings;
pub use font_settings::{
    CssAuthoredFontFeature, CssAuthoredFontFeatureList, CssAuthoredFontFeatureSettings,
    CssAuthoredFontFeatureValue, CssFontFeatureIndex, CssFontVariation, CssFontVariationList,
    CssFontVariationSettings, CssOpenTypeTag,
};
mod font_size;
pub use font_size::CssFontSize;
mod flex;
mod font_shorthand;
pub use flex::{
    CssFlexBasisRef, CssFlexBasisValue, CssFlexComponents, CssFlexDirection, CssFlexFlow,
    CssFlexValue, CssFlexWrap,
};
mod line_height;
pub use line_height::CssLineHeight;
mod font_style;
pub use font_style::{
    CssFontFaceObliqueRange, CssFontFaceStyle, CssFontObliqueAngle, CssFontStyle,
    CssFontStyleKeyword,
};
mod font_weight;
pub use font_weight::{
    CssAbsoluteFontWeight, CssFontFaceWeight, CssFontWeight, CssFontWeightNumber,
};
pub use inset::{CssInsetPair, CssInsetShorthand, CssInsetValue};
pub use integer_value::CssIntegerLiteral;
pub use item_flow::{
    CssItemDirection, CssItemFlow, CssItemPack, CssItemWrap, CssItemWrapMode, CssItemWrapOrder,
};
pub use overflow::CssOverflowValue;
pub use overflow_controls::{
    CssOverflowAnchor, CssOverflowClipMargin, CssScrollBehavior, CssScrollbarGutter,
};
mod view_transitions;
pub use view_transitions::{
    CssViewTransitionIdent, CssViewTransitionName, CssViewTransitionNameSelector,
};
mod will_change;
pub use scroll_snap::{
    CssScrollMarginPair, CssScrollMarginShorthand, CssScrollPaddingPair, CssScrollPaddingShorthand,
    CssScrollPaddingValue, CssScrollSideKind, CssScrollSnapAlign, CssScrollSnapAlignment,
    CssScrollSnapAxis, CssScrollSnapStop, CssScrollSnapStrictness, CssScrollSnapType,
};
pub use scrollbar::CssScrollbarColor;
pub use sizing::{CssBoxSize, CssMaxSizeValue, CssSizeValue};
pub use sizing_controls::{CssFrameSizing, CssMaxSizePair, CssMinIntrinsicSizing, CssSizePair};
pub use text_alignment::{
    CssCharacterAlignment, CssCharacterAlignmentError, CssCharacterAlignmentErrorKind,
    CssTextAlignAllValue, CssTextAlignLastValue, CssTextAlignPosition, CssTextAlignValue,
};
pub use text_spacing::CssTextSpacingAdjustment;
pub use will_change::{
    CssWillChange, CssWillChangeFeature, CssWillChangeFeatures, CssWillChangePropertyName,
};
mod float_clear;
mod overflow;
mod overflow_controls;
mod scrollbar;
mod specified_numeric;
mod specified_serialization;
mod writing_modes;
pub use opacity_scalar::{CssOpacityScalar, CssOpacityScalarKind};
pub use specified_numeric::{
    CssSpecifiedFlex, CssSpecifiedLength, CssSpecifiedLengthPercentage,
    CssSpecifiedNonNegativeFlex, CssSpecifiedNonNegativeLength,
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedNonNegativeNumber,
    CssSpecifiedNonNegativePercentage, CssSpecifiedNumber, CssSpecifiedPercentage,
};
pub use specified_serialization::{
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits,
};
mod parser;
mod parser_context;
mod svg_glyph;
pub use svg_glyph::{
    CssSvgGlyphOrientationVerticalDeclaration, CssSvgGlyphOrientationVerticalMetadata,
    CssSvgGlyphOrientationVerticalValue,
};
mod properties;
mod property_value;
pub use parser_context::{
    CssParserContext, CssParserMode, parse_css_supports_condition, parse_css_supports_declaration,
};
mod report;
mod source;
mod syntax;
mod tokenization;
pub use numeric::{
    CssAngleCalculation, CssAnglePercentageCalculation, CssCalculationConstantRef,
    CssCalculationExpressionRef, CssCalculationFunctionRef, CssCalculationProductFactorRef,
    CssCalculationProductOperator, CssCalculationProductRef, CssCalculationProfileChannelRef,
    CssCalculationSizeRef, CssCalculationSumOperator, CssCalculationSumRef,
    CssCalculationSumTermRef, CssCalculationTreeCountingRef, CssCalculationType,
    CssCalculationUnaryRef, CssCalculationValueRef, CssCalculationVariableRef, CssFlexCalculation,
    CssFrequencyCalculation, CssFrequencyPercentageCalculation, CssHintedNumberCalculation,
    CssIntegerCalculation, CssLengthCalculation, CssLengthPercentageCalculation, CssMathFunction,
    CssNumberCalculation, CssNumericConstant, CssNumericConstructionError,
    CssNumericConstructionErrorKind, CssNumericDimension, CssNumericLiteralRef, CssNumericType,
    CssNumericUnit, CssPercentageCalculation, CssProfileColorCalculation,
    CssProfileColorExpression, CssProfileColorExpressionRef, CssResolutionCalculation,
    CssRoundingStrategy, CssTimeCalculation, CssTreeCountingFunction,
};
#[cfg(test)]
mod test_support;
mod validation;

pub use color_profile::*;
pub use common_serialization::{
    serialize_css_comma_separated_list, serialize_css_comma_separated_list_with_limits,
    serialize_css_identifier, serialize_css_identifier_with_limits, serialize_css_string,
    serialize_css_string_with_limits, serialize_css_whitespace_separated_list,
    serialize_css_whitespace_separated_list_with_limits,
};
pub use component_values::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueLimits, CssComponentValueRef, CssComponentValues, CssFunctionValue,
    CssHashFlag, CssNumericTokenKind, CssNumericTokenRef, CssSerializedOrigin,
    CssSerializedOriginSegment, CssSerializedValue, CssSimpleBlock, CssValueOrigin,
    CssValueTokenRef, parse_comma_separated_component_values,
    parse_comma_separated_component_values_with_limits, parse_component_value,
    parse_component_value_with_limits, parse_component_values, parse_component_values_with_limits,
};
pub use conformance::*;
pub use content_values::*;
pub use counter_changes::{CssCounterChangeValue, CssCounterChangesValue, CssCounterProperty};
pub use cssom_rule_serialization::{
    CssRuleCssomFormat, CssRuleCssomKind, CssRuleCssomSerializationError,
    CssRuleCssomSerializationErrorKind,
};
pub use cursor_values::*;
pub use error::*;
pub use expansion::{
    CssContributionValueRef, CssContributions, CssCustomPropertyContribution, CssExpansion,
    CssExpansionError, CssExpansionErrorKind, CssFourSideShorthandMetadata, CssInitialValueRef,
    CssLonghandContribution, CssLonghandContributions, CssLonghandInitialValue,
    CssLonghandMetadata, CssLonghandProperty, CssLonghandValue, CssLonghandValueRef,
    CssPendingSubstitution, CssPropertyKindRef, CssPropertyMetadata, CssPropertyMetadataError,
    CssShorthandMetadata, CssSvgGlyphOrientationVerticalContribution, CssUniversalReset,
    CssUniversalResetMetadata, CssUserAgentInitial, expand_declaration,
};
pub use font_face_values::{
    CssFontMetricOverride, CssFontNamedInstance, CssFontNamedInstanceString,
};
pub use font_family_serialization::{
    serialize_font_face_family_list, serialize_font_face_family_list_with_limits,
};
pub use font_feature_values::*;
pub use font_palette::*;
pub use font_palette_values::*;
pub use font_synthesis::*;
pub use font_variant::*;
pub use hsl_color_conversion::{
    CssHslColorCoordinates, CssHslHwbConversionError, CssHwbColorCoordinates, convert_hsl_to_srgb,
    convert_hwb_to_srgb, convert_srgb_to_hsl, convert_srgb_to_hwb,
};
pub use lab_color_conversion::{
    CssPolarColorConversionError, CssPolarColorCoordinates, convert_lab_to_lch, convert_lch_to_lab,
    convert_oklab_to_oklch, convert_oklch_to_oklab,
};
pub use list_styles::{CssListStyleTypeValue, CssListStyleValue, CssMarkerSide};
pub use naive_color_conversion::{
    CssNaiveColorConversionError, naively_convert_cmyk_to_srgba, naively_convert_srgba_to_cmyk,
};
pub use normalization::{
    CssNormalizationError, CssNormalizationErrorKind, CssNormalizationLimits,
    CssNormalizationResource, CssNormalizedDeclaration, CssNormalizedItem, CssNormalizedReport,
    CssNormalizedSelector, CssNormalizedSheet, CssRuleContext, CssRuleContextKindRef,
    CssSelectorBinding, CssSelectorContext, normalize_report, normalize_report_with_limits,
    normalize_sheet, normalize_sheet_with_limits,
};
pub use parser::{
    CssAdmittedRule, CssAdmittedStyleSelectors, CssNamespaceContext, CssParsedKeyframeRule,
    CssParsedKeyframeSelectors, CssParsedStyleSelectors, CssRuleAdmissionContext, CssRuleSyntax,
    CssRuleSyntaxKind, CssStyleSelectorContext, classify_rule_syntax,
    classify_rule_syntax_with_limits, parse_color_profile_block,
    parse_color_profile_descriptor_value, parse_counter_style_block,
    parse_counter_style_descriptor_value, parse_cssom_media_query, parse_declaration,
    parse_declaration_block_contents, parse_declaration_block_contents_with_limits,
    parse_declaration_list_text, parse_font_face_block, parse_font_face_declaration_block_contents,
    parse_font_face_declaration_block_contents_with_limits, parse_font_face_descriptor_value,
    parse_font_feature_display_value, parse_font_feature_value, parse_font_feature_value_block,
    parse_font_feature_values_block, parse_font_feature_values_family_list,
    parse_font_feature_values_family_list_with_limits, parse_font_palette_descriptor_value,
    parse_font_palette_values_block, parse_group_block, parse_keyframe_declaration_block,
    parse_keyframe_rule, parse_keyframe_selector_list, parse_keyframes_block, parse_margin_block,
    parse_media_query, parse_media_query_list, parse_page_block,
    parse_page_declaration_block_contents, parse_page_declaration_block_contents_with_limits,
    parse_page_descriptor_value, parse_page_margin_rule, parse_page_margin_rule_with_limits,
    parse_page_selector_list, parse_property_value_text, parse_property_value_text_for_grammar,
    parse_relative_selector_list, parse_rule, parse_scope_block, parse_scoped_group_block,
    parse_selector, parse_selector_list, parse_sheet, parse_style_attribute, parse_style_block,
    parse_style_selector_list, parse_style_selector_list_with_limits, parse_supports_test_block,
};
pub use properties::*;
pub use property_value::{
    CssPropertyValueErrorKind, CssPropertyValueParseError, parse_margin_property_value,
    parse_page_property_value, parse_property_value, parse_property_value_for_grammar,
};
pub use rectangular_color_conversion::{
    CssRectangularColorConversionError, CssRectangularColorCoordinates, CssRectangularColorSpace,
};
pub use report::*;
pub use source::*;
pub use specified_declaration_block::{
    CssDeclarationBlockError, CssDeclarationBlockErrorKind, CssInvalidKeyframeSourceReason,
    CssSpecifiedDeclarationBlock, CssSpecifiedDeclarationEntry, CssSpecifiedDeclarationValueRef,
};
pub use specified_rule_serialization::{
    CssSpecifiedRuleSerializationError, CssSpecifiedRuleSerializationErrorKind,
};
pub use stylesheet_input::*;
pub use syntax::*;

/// Shared ceiling for authored rules and owned component-value trees.
const STRUCTURAL_NESTING_LIMIT: u32 = 256;
#[cfg(test)]
pub(crate) use test_support::{CssParseReportTestExt, CssProperty};

/// Validates a stylesheet by accepting only a clean ordinary parse report.
///
/// This application-strict wrapper consumes the ordinary [`parse_sheet`] report.
/// A clean report yields its retained authored syntax; a recovered report yields
/// every parser-produced diagnostic in unchanged order. Validation does not
/// select a different grammar or perform cascade, substitution,
/// contextual resolution, selector matching, or resource loading.
///
/// ```
/// use surgeist_css::validate_sheet;
///
/// let sheet = validate_sheet(".x { color: red; }").expect("clean stylesheet");
/// assert_eq!(sheet.rules().len(), 1);
/// ```
pub fn validate_sheet(input: &str) -> Result<CssSheet, CssValidationFailure> {
    parser::parse_sheet(input).into_validation_result()
}

/// Validates a style attribute by accepting only a clean ordinary parse report.
///
/// This application-strict wrapper consumes the ordinary
/// [`parse_style_attribute`] report. A clean report yields its retained authored
/// declarations; a recovered report yields the complete parser-produced
/// diagnostic sequence unchanged. It does not select a different declaration
/// grammar or apply cascade,
/// substitution, contextual resolution, selector matching, or resource loading.
///
/// ```
/// use surgeist_css::validate_style_attribute;
///
/// let declarations = validate_style_attribute("color: red")
///     .expect("clean style attribute");
/// assert_eq!(declarations.len(), 1);
/// ```
pub fn validate_style_attribute(input: &str) -> Result<CssDeclarationList, CssValidationFailure> {
    parser::parse_style_attribute(input).into_validation_result()
}

/// Validates raw declaration-list text by accepting exactly its clean parse report.
///
/// This uses the ordinary property grammar and raw unit synchronization of
/// [`parse_declaration_list_text`]. Recovery returns the complete diagnostic
/// sequence unchanged; no CSSOM projection, cascade or contextual resolution is applied.
pub fn validate_declaration_list_text(
    input: &str,
) -> Result<CssDeclarationList, CssValidationFailure> {
    parser::parse_declaration_list_text(input).into_validation_result()
}

#[cfg(test)]
mod tests;
