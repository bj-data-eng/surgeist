#![forbid(unsafe_code)]
#![cfg_attr(
    not(feature = "app-strict"),
    doc = r#"
The application-strict validators are intentionally absent unless the
`app-strict` feature is enabled.

```compile_fail
use surgeist_css::validate_sheet;

let _ = validate_sheet(".x { color: red; }");
```

```compile_fail
use surgeist_css::validate_style_attribute;

let _ = validate_style_attribute("color: red");
```
"#
)]
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
//! [`CssKnownDeclaration`] is parser-owned and has private fields. Its
//! [`CssKnownDeclaration::property`] identity is derived from the active coupled
//! value, so callers cannot create a property/value mismatch.
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
//!     CssImportance, CssKnownDeclaredValueRef, CssKnownPropertyValueRef,
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
//!             assert!(width.i01_subset().is_some());
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
//! Each of the 179 property-schema rows generates one private-field
//! `Css<SchemaVariant>PropertyValue` wrapper. Its `as_css()` method returns the
//! exact authored ordinary value, preserving interior spelling and trivia while
//! excluding parser-owned boundary trivia and the terminal importance annotation.
//! Its `i01_subset()` method is a compatibility view: every I01 input retains its
//! exact `Some` projection, while newly accepted I02 syntax returns `None` when
//! the frozen I01 representation cannot carry it.
//!
//! The generated [`CssOverflowPropertyValue`] is the authored wrapper for the
//! `overflow` row. [`CssOverflowI01PropertyValue`] is its renamed I01 payload and
//! retains the `Single` and `Pair` value shapes.
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
//! let angle = CssAngleCalculation::try_literal(-0.5, CssAngleUnit::Turns)
//!     .expect("finite authored angle");
//! assert_eq!(angle.result_type(), CssCalculationType::Angle);
//! assert!(matches!(
//!     angle.expression(),
//!     CssCalculationExpressionRef::Value(CssCalculationValueRef::Angle(value))
//!         if value.value() == -0.5 && value.unit() == CssAngleUnit::Turns
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
//! against a named edge. [`CssPositionOffset`] accepts only the symbolic length-percentage domain;
//! it does not resolve percentages, calculations, writing modes, or positioning boxes.
//!
//! ```
//! use surgeist_css::{CssLength, CssPositionOffset};
//!
//! let offset = CssPositionOffset::try_new(
//!     CssLength::try_percent(25.0).expect("finite authored percentage"),
//! )
//! .expect("position-valid offset");
//! assert!(matches!(offset.value(), CssLength::Percent(value) if value.value() == 25.0));
//! assert!(CssPositionOffset::try_new(CssLength::Auto).is_none());
//! ```
//!
//! [`CssPositionValue`] is parser-produced with private fields. Its borrowed horizontal and
//! vertical views make omitted centered axes and authored edge origins explicit without allowing
//! callers to forge an invalid cross-axis combination. `object-position` and every
//! `mask-position` layer use this exact generic grammar. `background-position` instead exposes a
//! nonempty [`CssBackgroundPositionList`] whose layers also admit the background-only
//! three-component form. `transform-origin` exposes the directed 2D split plus an optional checked
//! [`CssTransformOriginZ`] length.
//!
//! ```
//! use surgeist_css::{
//!     CssHorizontalPosition, CssKnownPropertyValueRef, CssLength, CssVerticalPosition,
//!     parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute(concat!(
//!     "background-position: left 10px top; ",
//!     "mask-position: right 5% bottom 2px; ",
//!     "object-position: center 25%; ",
//!     "transform-origin: top 50px",
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
//!         if matches!(offset.value(), CssLength::Px(value) if value.value() == 10.0)
//! ));
//!
//! let CssKnownPropertyValueRef::MaskPosition(mask) = report.syntax()[1]
//!     .known().expect("known mask position")
//!     .property_value().expect("ordinary mask position")
//! else { panic!("expected mask-position") };
//! assert!(matches!(
//!     mask.positions().positions()[0].value().vertical(),
//!     CssVerticalPosition::BottomOffset(_)
//! ));
//!
//! let CssKnownPropertyValueRef::ObjectPosition(object) = report.syntax()[2]
//!     .known().expect("known object position")
//!     .property_value().expect("ordinary object position")
//! else { panic!("expected object-position") };
//! assert!(matches!(object.position().value().horizontal(), CssHorizontalPosition::Center));
//!
//! let CssKnownPropertyValueRef::TransformOrigin(transform) = report.syntax()[3]
//!     .known().expect("known transform origin")
//!     .property_value().expect("ordinary transform origin")
//! else { panic!("expected transform-origin") };
//! assert!(matches!(
//!     transform.origin().z().map(|z| z.value()),
//!     Some(CssLength::Px(value)) if value.value() == 50.0
//! ));
//! ```
//!
//! The background, mask, and transform wrappers retain `i01_subset()` as a frozen compatibility
//! view. Newly accepted current syntax returns `None` when it cannot be represented without loss;
//! `object-position` is additive and has no I01 projection. Function-specific position grammars,
//! cascade, substitution, contextual resolution, layout, painting, transforms, and cross-crate
//! lowering remain outside this surface.
//!
//! # Dedicated authored function grammars
//!
//! Current property accessors expose dedicated typed function families while
//! `i01_subset()` remains the frozen compatibility view. Transform wrappers return
//! [`CssTransformValue`], timing-function wrappers expose [`CssEasingValue`] lists,
//! filter and backdrop-filter wrappers return [`CssFilterValue`], box-shadow returns
//! [`CssBoxShadow`], and clip-path returns an optional [`CssClipPathValue`]. A current
//! value can be valid when its I01 projection is `None`; compatibility data is not the
//! current grammar.
//!
//! ```
//! use surgeist_css::{
//!     CssBasicShapeValue, CssClipPathValue, CssFilterFunctionValue, CssFilterValue,
//!     CssKnownPropertyValueRef, CssTransformFunctionValue, CssTransformValue,
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
//!     transform.current(),
//!     CssTransformValue::Functions(functions)
//!         if matches!(functions.functions()[0], CssTransformFunctionValue::Translate3d(_))
//! ));
//!
//! let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[1]
//!     .known().expect("known filter")
//!     .property_value().expect("ordinary filter")
//! else { panic!("expected filter") };
//! assert!(matches!(
//!     filter.current(),
//!     CssFilterValue::Functions(functions)
//!         if matches!(functions.functions()[1], CssFilterFunctionValue::DropShadow(_))
//! ));
//!
//! let CssKnownPropertyValueRef::ClipPath(clip) = report.syntax()[2]
//!     .known().expect("known clip path")
//!     .property_value().expect("ordinary clip path")
//! else { panic!("expected clip-path") };
//! assert!(matches!(
//!     clip.current(),
//!     Some(CssClipPathValue::BasicShape(CssBasicShapeValue::Polygon(polygon)))
//!         if polygon.round().is_some()
//! ));
//! ```
//!
//! The typed transform family covers the selected two-dimensional Transforms 1
//! functions and the preserved I01 three-dimensional subset with exact arity,
//! separators, and dimensions. Easing values distinguish keywords,
//! `cubic-bezier()`, and `steps()`. Box shadows and filter `drop-shadow()` have
//! separate models, filter lists preserve URL/function order, and the selected
//! basic-shape family exposes `inset()`, `circle()`, `ellipse()`, and `polygon()`,
//! including polygon `round <length>`.
//!
//! These are authored syntax values. This crate does not multiply transform matrices,
//! interpolate or evaluate easing, render shadows or filters, resolve URLs, compute
//! shape geometry, perform layout or painting, or lower values into sibling crates.
//! `path()`, `shape()`, `rect()`, `xywh()`, and clip-path reference-box combinations
//! remain outside the selected subset. `transition`, `animation`, `backdrop-filter`,
//! and `clip-path` retain explicit Partial metadata boundaries; support for a typed
//! function does not promote an aggregate or an unselected production.
//!
//! # Authored colors and frozen I01 compatibility
//!
//! The current color model preserves authored Color 4 syntax. It distinguishes
//! named, transparent, current, hexadecimal, current and deprecated system,
//! legacy and modern RGB/HSL, HWB, Lab/LCH, Oklab/Oklch, and predefined
//! `color()` branches. Finite specified components remain authored when they are
//! outside a computed range, and typed calculations remain symbolic. The current
//! opacity model similarly preserves finite numbers and percentages, including
//! signed and out-of-range specified values.
//!
//! Color-bearing property wrappers expose their current value through
//! `current()`, and the opacity wrapper exposes its current [`CssOpacityValue`]
//! through `value()`. Their `i01_subset()` is a separate frozen compatibility
//! view: every frozen I01 input keeps its exact projection, while a newly
//! accepted current value returns `None` when [`CssColor`] or [`CssOpacity`]
//! cannot represent it without loss. A missing compatibility projection does
//! not make the current value invalid.
//!
//! ```
//! use surgeist_css::{
//!     CssAuthoredSystemColor, CssKnownPropertyValueRef, CssOpacityValue,
//!     parse_style_attribute,
//! };
//!
//! let report = parse_style_attribute("color: ActiveBorder; opacity: 150%");
//! assert!(report.is_clean());
//!
//! let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
//!     .known().expect("known color")
//!     .property_value().expect("ordinary color")
//! else { panic!("expected color") };
//! assert_eq!(
//!     color.current().system(),
//!     Some(CssAuthoredSystemColor::ActiveBorder),
//! );
//! assert!(color.i01_subset().is_none());
//!
//! let CssKnownPropertyValueRef::Opacity(opacity) = report.syntax()[1]
//!     .known().expect("known opacity")
//!     .property_value().expect("ordinary opacity")
//! else { panic!("expected opacity") };
//! assert!(matches!(opacity.value(), CssOpacityValue::Percentage(value)
//!     if value.value() == 150.0));
//! assert!(opacity.i01_subset().is_none());
//! ```
//!
//! The preserved Color 5 surface is intentionally narrower. Relative colors
//! cover `rgb`/`rgba`, `hsl`/`hsla`, `hwb`, `lab`, `lch`, `oklab`, `oklch`,
//! and predefined RGB/XYZ `color()` spaces with closed per-family channel
//! environments. `color-mix()` requires an interpolation method and exactly two
//! colors, accepts optional trailing percentages, and permits hue interpolation
//! methods only in polar spaces. `alpha()`, custom color profiles,
//! `light-dark()`, and `device-cmyk()` are not part of this surface.
//!
//! These values remain authored syntax. This crate does not clamp computed
//! color or opacity values, resolve `currentcolor` or system colors, evaluate
//! relative channels or calculations, perform color conversion or gamut mapping,
//! resolve a mix, apply contrast, serialize computed colors, or lower colors
//! into a sibling crate.
//!
//! # Authored Grid repetition and keyframe structure
//!
//! The six Grid repetition consumers expose parser-owned current values through
//! their `current()` accessors while retaining the frozen `i01_subset()`
//! compatibility view. [`CssAuthoredGridTrackList`] distinguishes general track
//! lists from lists containing exactly one [`CssAuthoredGridAutoRepeat`]. Integer
//! repetition is non-recursive; automatic repetition and surrounding tracks use
//! fixed sizes; and `grid-auto-rows` and `grid-auto-columns` expose
//! [`CssAuthoredGridTrackSizeList`] values without `repeat()`.
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
//! subgrid name-repeat and other unselected Grid 2 property grammar.
//! `@keyframes` remains Partial for calculation selectors, string names, and
//! unselected declaration-processing grammar. This crate does not perform Grid
//! layout, cascade declarations, evaluate or interpolate keyframes, run
//! timelines, or lower either syntax family into sibling Surgeist crates.
//!
//! # Fonts 3 typography and font-face
//!
//! The current authored font surface implements the sixteen Fonts 3 property
//! grammars, including family/global boundaries, four-ASCII-character OpenType
//! tags, non-negative feature indices, the explicit and system `font` branches,
//! synthesis, and the five variant longhands. Concrete property wrappers expose
//! parser-owned current values through their semantic accessors and retain
//! `i01_subset()` only as a frozen compatibility projection.
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
//! assert!(font.i01_subset().is_none());
//! ```
//!
//! [`CssFontFaceDescriptors::occurrences`] exposes valid descriptor occurrences
//! in authored order, while typed effective accessors return the last valid
//! occurrence. Invalid and unknown occurrences recover with
//! [`CssRecoveryAction::DropDescriptor`] without erasing valid neighbors. A
//! [`CssFontFaceRule`] is retained only when valid effective `font-family` and
//! `src` descriptors remain.
//!
//! Fonts 3 property, descriptor, source, and OpenType-tag records cite the dated
//! `O-FONTS3` source and are Complete. The selected numeric weight, descriptor
//! range, and modern source-hint deltas cite `I-FONTS4` as separate Partial
//! records; `font-display` is Complete and `@font-feature-values` remains
//! RecognizedUnsupported. This crate does not load or match fonts, resolve
//! fallback or OpenType feature application, shape glyphs, apply cascade or
//! substitution, evaluate computed values, expose CSSOM, serialize, or lower
//! into another Surgeist crate.
//!
//! # Timing domains and I01 compatibility
//!
//! Duration literals are finite and non-negative; delay literals are finite and signed. A range
//! constraint that belongs to a literal is enforced immediately, while a well-typed calculation
//! remains representable for later computed-value processing. Current property accessors expose
//! those distinct domains. [`CssKnownDeclaration::property_value`] still provides
//! `i01_subset()` on each concrete wrapper as the frozen compatibility view.
//!
//! ```
//! use surgeist_css::{
//!     CssDelay, CssDuration, CssDurationLiteral, CssKnownPropertyValueRef,
//!     CssTimeUnit, parse_style_attribute,
//! };
//!
//! assert!(CssDurationLiteral::try_new(-1.0, CssTimeUnit::Seconds).is_none());
//!
//! let report = parse_style_attribute(concat!(
//!     "transition-duration: calc(-1s + 2s); ",
//!     "transition-delay: -250ms",
//! ));
//! assert!(report.is_clean());
//!
//! let CssKnownPropertyValueRef::TransitionDuration(duration) = report.syntax()[0]
//!     .known()
//!     .expect("known duration")
//!     .property_value()
//!     .expect("ordinary duration")
//! else {
//!     panic!("expected transition-duration");
//! };
//! assert!(matches!(
//!     duration.durations().values()[0],
//!     CssDuration::Calculation(_)
//! ));
//! assert!(duration.i01_subset().is_none());
//!
//! let CssKnownPropertyValueRef::TransitionDelay(delay) = report.syntax()[1]
//!     .known()
//!     .expect("known delay")
//!     .property_value()
//!     .expect("ordinary delay")
//! else {
//!     panic!("expected transition-delay");
//! };
//! assert!(matches!(
//!     delay.delays().values()[0],
//!     CssDelay::Literal(value) if value.value() == -250.0
//! ));
//!
//! let i01 = parse_style_attribute("transition-duration: 1s");
//! let CssKnownPropertyValueRef::TransitionDuration(duration) = i01.syntax()[0]
//!     .known()
//!     .expect("known duration")
//!     .property_value()
//!     .expect("ordinary duration")
//! else {
//!     panic!("expected transition-duration");
//! };
//! assert!(duration.i01_subset().is_some());
//! ```
//!
//! New signed-delay and typed-calculation syntax returns no I01 projection when the older payload
//! cannot represent it. This crate owns authored timing syntax only; timeline evaluation and
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
//!     CssNamespaceConstraint, CssPseudoElement, CssRule, CssSelector, parse_sheet,
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
//! let CssSelector::Compound(selector) = style.selector() else {
//!     panic!("expected compound selector");
//! };
//! let qualified = selector.type_selector().expect("qualified type selector");
//! assert!(matches!(
//!     qualified.namespace(),
//!     CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
//! ));
//! assert_eq!(qualified.local_name(), Some("a"));
//! assert_eq!(selector.ids(), ["first", "second"]);
//! assert_eq!(selector.key().map(String::as_str), Some("second"));
//! let [attribute] = selector.attributes() else {
//!     panic!("expected one attribute selector");
//! };
//! assert_eq!(attribute.namespace(), &CssNamespaceConstraint::ExplicitNone);
//! assert!(matches!(
//!     selector
//!         .pseudo_elements()
//!         .expect("pseudo-element sequence")
//!         .pseudo_elements(),
//!     [CssPseudoElement::FirstLine]
//! ));
//! ```
//!
//! The top-level phase machine is `Initial`, `InitialLayers`, `Imports`,
//! `ImportsAfterInitialLayers`, `Namespaces`, and `Body`. Initial layers still admit imports but
//! permanently prohibit namespaces. Only `Initial` and `Imports` admit a namespace; after that,
//! only consecutive namespaces remain valid until a layer or body transition. Invalid or ignored
//! rules do not change the phase or active bindings. Malformed, block-form, nested, or late
//! namespaces recover as one [`CssRecoveryAction::DropAtRule`].
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
//! [`feature_metadata`] exposes its distinct [`CssFeatureKind::PropertyAlias`] record. The
//! shared box-edge and blend-mode productions likewise have independent complete records.
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
//! assert_eq!(alias.status(), CssSupportStatus::Complete);
//! ```
//!
//! # Media, supports, imports, and prelude recovery
//!
//! Media Queries 3 syntax preserves defined-false authored input without confusing it with
//! malformed-member recovery. A balanced unknown feature or value is retained as
//! [`CssMediaConditionKind::DefinedFalse`] with no diagnostic. A reserved or structurally
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
//!     ] if matches!(condition.kind(), CssMediaConditionKind::DefinedFalse(_))
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
//!         if value.authored() == "future-layout(mode)"
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
//! ```compile_fail
//! use surgeist_css::CssCalcOperator;
//!
//! fn describe(value: CssCalcOperator) -> &'static str {
//!     match value {
//!         CssCalcOperator::Add => "add",
//!         CssCalcOperator::Subtract => "subtract",
//!     }
//! }
//! ```
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
//! authored values without performing layout, pagination, or painting. The
//! generic Syntax 3 authored shells and the remaining selected Values 3 records
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
//! let unsupported = feature_metadata("later.rule.font-feature-values")
//!     .expect("recognized unsupported rule metadata");
//! assert_eq!(
//!     unsupported.status(),
//!     CssSupportStatus::RecognizedUnsupported,
//! );
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
//! `ext.value.grid-repeat`, `ext.value.basic-shape`,
//! `ext.descriptor.font-weight-range`, `ext.descriptor.font-style-oblique-range`,
//! `ext.descriptor.font-stretch-range`, `ext.value.font-source-modern-hints`,
//! `ext.property.font-weight-range`, `ext.supports.selector`,
//! `ext.media.range.width`, `ext.media.range.height`,
//! `ext.media.range.resolution`, `ext.media.range.color`, and
//! `ext.media.range.monochrome` records remain Partial with explicit subset and
//! remainder metadata. `@font-feature-values` remains RecognizedUnsupported. C13's
//! 456 public catalog records plus C14's 31 additions reconcile to exactly 487
//! records. That catalog cardinality is distinct from the immutable official
//! inventory of exactly 162 property units, one normative legacy shorthand, and
//! 167 non-property units. All 219 I01 baseline records retain their
//! classifications, and the exclusion registry remains exactly 131 rows.
//!
//! # Support metadata and application policy
//!
//! [`feature_catalog`] describes each declared conformance production as
//! [`CssSupportStatus::Complete`], [`CssSupportStatus::Partial`], or
//! [`CssSupportStatus::RecognizedUnsupported`]. Partial records state both their
//! accepted subset and valid-but-unsupported remainder. A diagnostic-free use of
//! a partial production's accepted subset is still a clean parse.
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
//! The optional `app-strict` feature adds `validate_sheet` and
//! `validate_style_attribute`. Each validator consumes ordinary parsing semantics
//! and its report, accepts exactly a clean report, and otherwise preserves the
//! complete non-empty diagnostic sequence in [`CssValidationFailure`]. The
//! feature does not select a second grammar or change ordinary parsing.
//!
//! # Boundary
//!
//! This crate owns authored CSS syntax, intrinsic grammar validation, recovery
//! boundaries, diagnostic provenance, and support metadata. It does not apply
//! cascade or inheritance; substitute custom properties; validate computed
//! post-substitution values; evaluate queries; match selectors; resolve URLs,
//! resources, units, or colors; perform layout, painting, or animation; expose a
//! mutable CSSOM; or lower CSS into sibling Surgeist types.

mod conformance;
mod error;
mod parser;
mod properties;
mod report;
mod source;
mod syntax;
#[cfg(test)]
mod test_support;
mod validation;

pub use conformance::*;
pub use error::*;
pub use parser::{parse_sheet, parse_style_attribute};
pub use properties::*;
pub use report::*;
pub use source::*;
pub use syntax::*;
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
#[cfg(feature = "app-strict")]
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
#[cfg(feature = "app-strict")]
pub fn validate_style_attribute(input: &str) -> Result<CssDeclarationList, CssValidationFailure> {
    parser::parse_style_attribute(input).into_validation_result()
}

#[cfg(test)]
mod tests;
