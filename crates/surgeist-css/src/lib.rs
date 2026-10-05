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
//! [`CssImportance`] and [`CssSupportStatus`] are exactly the two closed public
//! enums. All other public enums are non-exhaustive and downstream matches must
//! include a wildcard. This inspection model does not change parsing, recovery,
//! or diagnostics.
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
//! math. Three-dimensional scale percentage branches use `CssSpecifiedPercentage`.
//! Ordinary cubic-bezier X coordinates are checked exactly against inclusive [0, 1];
//! genuine calculations remain unresolved. The independent `scale` property retains
//! its selected one-to-three literal-number subset. Transform, filter, gradient, and
//! image-orientation angles share exact `CssAngleLiteral` values and symbolic
//! Angle-root calculations through the strict `CssAngleValue`. Transforms, filters,
//! and gradient directions admit bare zero through `CssAngleOrZero`; image orientation
//! and shape rotation require an angle dimension or calculation. Raw scalars retain provenance while
//! semantic aggregates compare structure independently of scalar origins.
//! Easing values distinguish keywords,
//! `cubic-bezier()`, and `steps()`. Box shadows and filter `drop-shadow()` have
//! separate models, filter lists preserve URL/function order, and the selected
//! basic-shape family exposes `inset()`, `circle()`, `ellipse()`, `polygon()`,
//! `rect()`, `xywh()`, `path()`, and `shape()`, including polygon `round <length>` and optional
//! rectangular round radii.
//!
//! These are authored syntax values. This crate does not multiply transform matrices,
//! interpolate or evaluate easing, render shadows or filters, resolve URLs, compute
//! shape geometry, perform layout or painting, or lower values into sibling crates.
//! Basic shapes and `clip-path` expose complete authored grammar support; this does
//! not establish contextual geometry or exact calculation projection. `transition`
//! and `animation` retain explicit Partial metadata boundaries; support for a typed
//! function does not promote an unselected production.
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
//! their `value()` accessors. [`CssGridTrackList`] distinguishes general track
//! lists from lists containing exactly one [`CssGridAutoRepeat`]. Integer
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
//! Grid repetition and the six consuming Grid properties remain Partial for
//! subgrid name-repeat, remaining `grid`/`grid-template` shorthand alternatives,
//! and implicit-track lifecycle work. Repetition counts and used track
//! sizes require downstream layout context and remain unresolved here.
//! `@keyframes` remains Partial for calculation selectors, string names, and
//! unselected declaration-processing grammar. This crate does not perform Grid
//! layout, cascade declarations, evaluate or interpolate keyframes, run
//! timelines, or lower either syntax family into sibling Surgeist crates.
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
//! the existing consumer recovery contract: `:is()` and `:where()` drop only the invalid member,
//! while unforgiving style, scope, nesting, `:not()`, `:has()`, and nth `of` consumers drop their
//! established containing unit. Matching, specificity, cascade, namespace URI resolution,
//! CSSOM serialization, and cross-crate lowering remain downstream.
//!
//! # Counter styles and page rules
//!
//! [`CssRule::CounterStyle`] retains a checked, case-sensitive [`CssCounterStyleName`], every
//! valid descriptor occurrence in authored order, the effective last valid occurrence of each
//! descriptor, and the rule position. The typed descriptor values cover Counter Styles 3
//! `system`, `negative`, `prefix`, `suffix`, `range`, `pad`, `fallback`, `symbols`,
//! `additive-symbols`, and `speak-as`. Definitions with an invalid effective descriptor
//! combination are dropped as one at-rule; an invalid or unknown individual descriptor is
//! dropped while valid descriptor and rule siblings remain eligible.
//!
//! [`CssRule::Page`] retains the CSS2 default page form or one [`CssPageSelector`] plus valid
//! page-context margin declarations in authored order. The page body accepts only `margin` and
//! its four longhands with the CSS2 length, percentage, `auto`, and negative-value domains.
//! Invalid or unknown declarations are dropped individually. Both rule families are top-level,
//! block-form authored syntax; pagination, page matching, cascade, counter registration,
//! inheritance resolution, generated-marker rendering, and margin-box rules are excluded.
//!
//! ```
//! use surgeist_css::{CssCounterStyleSystem, CssPageSelector, CssRule, parse_sheet};
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
//!     counter.descriptors().system().map(|value| value.value()),
//!     Some(CssCounterStyleSystem::Numeric)
//! ));
//! assert_eq!(counter.descriptors().occurrences().count(), 3);
//! assert_eq!(page.selector(), Some(CssPageSelector::Left));
//! assert_eq!(page.declarations().len(), 2);
//! ```
//!
//! # Residual official properties and legacy orientation
//!
//! The C12 family exposes complete typed authored grammars for thirteen CSS2 residual
//! properties; Writing Modes 3 text combination, orientation, and bidi properties; UI3 caret,
//! outline-offset, and resize properties; Containment 1 `contain`; Transforms 1
//! `transform-box`; and Compositing 1 blend and isolation properties. These values retain
//! authored syntax and parser coordinates without applying cascade, layout, pagination,
//! painting, containment semantics, blending, hit testing, or writing-mode resolution.
//!
//! `glyph-orientation-vertical` is an explicit restricted legacy shorthand that maps to a
//! parser-produced [`CssKnownProperty::TextOrientation`] value. It is not a name-equivalent
//! schema alias: [`CssKnownProperty::aliases`] remains empty for `TextOrientation`, while
//! [`feature_metadata`] exposes its distinct [`CssFeatureKind::PropertyAlias`] record. That
//! record is partial: the admitted literal mapping is supported, while numeric-terminal
//! spelling and math applicability remain unresolved by the selected standards. The shared
//! box-edge and blend-mode productions have independent complete records.
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
//! general-enclosed fallback syntax. `||`, unselected Selectors 4 pseudo-classes and
//! pseudo-elements, and syntax outside the named extension rows remain outside the typed subset.
//! These nodes describe authored tests; the crate never evaluates whether a condition matches.
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
//! non-exhaustive enum. [`CssImportance`] and [`CssSupportStatus`] are the two
//! deliberately closed enums and remain exhaustively matchable.
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
//! border-image, and object-sizing structure. They do not resolve URLs, load or
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
//! `ext.value.grid-repeat`,
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
//! 167 non-property units. All 219 I01 baseline records retain their
//! classifications, and the exclusion registry now contains exactly 130 rows.
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
mod color_profile;
mod color_profile_serialization;
mod common_serialization;
mod component_values;
mod conformance;
mod content_serialization;
mod content_values;
mod counter_changes;
mod counter_changes_serialization;
mod easing_serialization;
mod error;
mod expansion;
mod filter_serialization;
mod font_face_values;
mod font_family_serialization;
mod font_feature_values;
mod font_palette;
mod font_palette_serialization;
mod font_palette_values;
mod font_rule_serialization;
mod font_source_serialization;
mod font_synthesis;
mod font_variant;
mod font_variant_serialization;
mod image_serialization;
mod imports;
mod list_style_serialization;
mod list_styles;
mod quotes;
mod shape_serialization;
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
mod supports;
pub use supports::CssSupportsConstructionError;
mod media;
mod named_supports_serialization;
mod rule_construction;
mod shadow_serialization;
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
mod scroll_snap;
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
mod text_alignment;
mod text_spacing;
mod time;
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
pub use resolution::{CssResolutionLiteral, CssResolutionValue};
pub use time::{CssDuration, CssTimeLiteral, CssTimeValue};
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
pub use overflow::CssOverflowValue;
pub use overflow_controls::{
    CssOverflowAnchor, CssOverflowClipMargin, CssScrollBehavior, CssScrollbarGutter,
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
    CssSpecifiedLength, CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeFlex,
    CssSpecifiedNonNegativeLength, CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedNonNegativeNumber, CssSpecifiedNonNegativePercentage, CssSpecifiedNumber,
    CssSpecifiedPercentage,
};
pub use specified_serialization::{
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits,
};
mod parser;
mod properties;
mod property_value;
mod report;
mod source;
mod syntax;
pub use numeric::{
    CssAngleCalculation, CssCalculationConstantRef, CssCalculationExpressionRef,
    CssCalculationFunctionRef, CssCalculationProductFactorRef, CssCalculationProductOperator,
    CssCalculationProductRef, CssCalculationProfileChannelRef, CssCalculationSizeRef,
    CssCalculationSumOperator, CssCalculationSumRef, CssCalculationSumTermRef,
    CssCalculationTreeCountingRef, CssCalculationType, CssCalculationUnaryRef,
    CssCalculationValueRef, CssCalculationVariableRef, CssFlexCalculation, CssFrequencyCalculation,
    CssFrequencyPercentageCalculation, CssHintedNumberCalculation, CssIntegerCalculation,
    CssLengthCalculation, CssLengthPercentageCalculation, CssMathFunction, CssNumberCalculation,
    CssNumericConstant, CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssNumericDimension, CssNumericLiteralRef, CssNumericType, CssNumericUnit,
    CssPercentageCalculation, CssProfileColorCalculation, CssProfileColorExpression,
    CssProfileColorExpressionRef, CssResolutionCalculation, CssRoundingStrategy,
    CssTimeCalculation, CssTreeCountingFunction,
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
    CssHashFlag, CssNumericTokenKind, CssNumericTokenRef, CssParsedOrigin, CssSerializedOrigin,
    CssSerializedOriginSegment, CssSerializedValue, CssSimpleBlock, CssSourceSnapshot,
    CssValueOrigin, CssValueTokenRef, parse_component_values, parse_component_values_with_limits,
};
pub use conformance::*;
pub use content_values::*;
pub use counter_changes::{CssCounterChangeValue, CssCounterChangesValue, CssCounterProperty};
pub use error::*;
pub use expansion::{
    CssContributionValueRef, CssContributions, CssCustomPropertyContribution, CssExpansion,
    CssExpansionError, CssExpansionErrorKind, CssFourSideShorthandMetadata, CssInitialValueRef,
    CssLonghandContribution, CssLonghandContributions, CssLonghandInitialValue,
    CssLonghandMetadata, CssLonghandProperty, CssLonghandValue, CssLonghandValueRef,
    CssPendingSubstitution, CssPropertyKindRef, CssPropertyMetadata, CssPropertyMetadataError,
    CssShorthandMetadata, CssUniversalReset, CssUniversalResetMetadata, CssUserAgentInitial,
    expand_declaration,
};
pub use font_face_values::{
    CssFontMetricOverride, CssFontNamedInstance, CssFontNamedInstanceString,
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
    CssNamespaceContext, parse_color_profile_descriptor_value, parse_cssom_media_query,
    parse_declaration, parse_font_face_descriptor_value, parse_font_palette_descriptor_value,
    parse_media_query, parse_media_query_list, parse_property_value_text,
    parse_property_value_text_for_grammar, parse_rule, parse_selector, parse_selector_list,
    parse_sheet, parse_style_attribute, parse_style_block,
};
pub use properties::*;
pub use property_value::{
    CssPropertyValueErrorKind, CssPropertyValueParseError, parse_property_value,
    parse_property_value_for_grammar,
};
pub use rectangular_color_conversion::{
    CssRectangularColorConversionError, CssRectangularColorCoordinates, CssRectangularColorSpace,
};
pub use report::*;
pub use source::*;
pub use specified_rule_serialization::{
    CssSpecifiedRuleSerializationError, CssSpecifiedRuleSerializationErrorKind,
};
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

#[cfg(test)]
mod tests;
