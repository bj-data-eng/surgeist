//! `<position>` values.
//!
//! # Property-specific authored positions
//!
//! Generic CSS positions retain both axes and distinguish free offsets from offsets authored
//! against a named edge. [`CssNumeric<CssLengthPercentage>`](crate::numeric::CssNumeric) accepts only the symbolic
//! length-percentage domain;
//! it does not resolve percentages, calculations, writing modes, or positioning boxes.
//!
//! ```
//! use surgeist_css::component::CssComponentValue;
//! use surgeist_css::numeric::{CssLengthPercentage, CssNumeric};
//!
//! let offset = CssNumeric::<CssLengthPercentage>::try_from_component(
//!     CssComponentValue::try_token("25%").expect("exact authored percentage"),
//! ).expect("position-valid offset");
//! assert_eq!(offset.literal().unwrap().numeric().representation(), "25");
//! assert!(CssNumeric::<CssLengthPercentage>::try_from_component(
//!     CssComponentValue::try_ident("auto").unwrap(),
//! ).is_err());
//! ```
//!
//! [`CssPhysicalPosition::try_new`](crate::values::position::CssPhysicalPosition::try_new) checks the generic position's cross-axis pairing of edge
//! offsets. Its borrowed horizontal and vertical views make omitted centered axes and authored
//! edge origins explicit. `transform-origin` uses this physical grammar.
//! `object-position`, `perspective-origin` and every `mask-position` layer use the Values 5
//! [`CssPosition`], which also admits the flow-relative keywords and keeps them unresolved.
//!
//! `transform-origin` exposes the directed 2D split plus an optional checked
//! [`CssNumeric<CssLength>`](crate::numeric::CssNumeric) length.
//!
//! ```
//! use surgeist_css::values::position::{CssHorizontalPosition, CssPositionRef, CssVerticalPosition};
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents(concat!(
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
//!     background.value().positions()[0].horizontal(),
//!     CssHorizontalPosition::LeftOffset(offset)
//!         if matches!(offset.literal().unwrap().component().view(), surgeist_css::component::CssComponentValueRef::Token(surgeist_css::component::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "10" && unit == "px")
//! ));
//!
//! assert_eq!(
//!     background.value().serialize_specified(Default::default()).unwrap(),
//!     "left 10px top",
//! );
//!
//! let CssKnownPropertyValueRef::MaskPosition(mask) = report.syntax()[1]
//!     .known().expect("known mask position")
//!     .property_value().expect("ordinary mask position")
//! else { panic!("expected mask-position") };
//! let CssPositionRef::Cartesian(mask_position) = mask.value().positions()[0].view()
//! else { panic!("a Cartesian mask position") };
//! assert!(matches!(mask_position.vertical(), CssVerticalPosition::BottomOffset(_)));
//!
//! let CssKnownPropertyValueRef::ObjectPosition(object) = report.syntax()[2]
//!     .known().expect("known object position")
//!     .property_value().expect("ordinary object position")
//! else { panic!("expected object-position") };
//! let CssPositionRef::Cartesian(object_position) = object.value().view()
//! else { panic!("a Cartesian object position") };
//! assert!(matches!(object_position.horizontal(), CssHorizontalPosition::Center));
//!
//! let CssKnownPropertyValueRef::TransformOrigin(transform) = report.syntax()[3]
//!     .known().expect("known transform origin")
//!     .property_value().expect("ordinary transform origin")
//! else { panic!("expected transform-origin") };
//! assert!(matches!(transform.value().z().unwrap().literal().unwrap().component().view(), surgeist_css::component::CssComponentValueRef::Token(surgeist_css::component::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "50" && unit == "px"));
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
//! `object-position`, `perspective-origin`, `mask-position` and the mask shorthand use the full
//! Values 5 [`CssPosition`], as do circle and ellipse; `transform-origin` keeps the physical
//! [`CssPhysicalPosition`].
//! Its checked Cartesian, named-flow and relative-flow constructors retain symbolic axes;
//! [`CssPosition::view`](crate::values::position::CssPosition::view) exposes borrowed payloads. All-center values have one Cartesian form.
//! Specified output orders Cartesian axes horizontally/vertically and flow axes by block/inline.
//! Cascade, substitution, contextual resolution, layout, painting, transforms, and cross-crate
//! lowering remain outside this surface.

pub(crate) mod position_serialization;
pub(crate) mod position_syntax;

#[doc(inline)]
pub use position_syntax::{
    CssBlockPosition, CssCartesianPosition, CssHorizontalPosition, CssHorizontalPositionKeyword,
    CssInlinePosition, CssNamedFlowPosition, CssPhysicalPosition, CssPosition,
    CssPositionConstructionError, CssPositionList, CssPositionRef, CssRelativeAxisPosition,
    CssRelativeFlowPosition, CssVerticalPosition, CssVerticalPositionKeyword,
};
