//! Background and blend-mode values.
//!
//! # Background position
//!
//! `background-position` instead exposes a
//! nonempty [`CssBackgroundPositionList`] whose layers also admit the background-only
//! three-component form. Its noninherited longhand initial is one `0% 0%` layer.
//! [`CssBackgroundPosition::serialize_specified`](crate::values::background::CssBackgroundPosition::serialize_specified) and the list's matching method
//! emit horizontal-first specified values, preserving an omitted edge offset.
//! The checked [`CssBackgroundLayer::try_new`](crate::values::background::CssBackgroundLayer::try_new) and [`CssBackground::try_new`](crate::values::background::CssBackground::try_new)
//! constructors retain optional authored components, couple size to position, and
//! restrict color to the final layer. Intrinsic `background` expansion fills each
//! of seven per-layer lists from the terminal schema initial and emits one color.
//! It also resets `background-blend-mode` to one `normal` entry. Expansion and
//! normalization account for nine terminal contributions; a CSS-wide keyword
//! applies to all nine. The authored shorthand retains its eight settable members.
//! [`CssBackground::serialize_specified`](crate::values::background::CssBackground::serialize_specified) composes canonical specified children in
//! grammar order, omitting proved simple initials under one cumulative budget.
//! Omitted authored children still consume input and projection visits; an empty
//! effective layer emits `none`. Keywords, calculations, and contextual colors
//! retain their symbolic meaning.
//!
//! # Blend modes and isolation
//!
//! `background-blend-mode`, `isolation`, and `mix-blend-mode` provide noninherited
//! intrinsic initials of one `normal` list entry, `auto`, and `normal`. Their
//! [`CssLonghandValueRef`](crate::property::CssLonghandValueRef) variants borrow the existing typed values. Expansion,
//! pending-substitution reentry, and normalization retain authored blend-list
//! order, duplicates, source occurrences, and declaration importance. Checked
//! admission requires complete original components, while browser recovery stays
//! observable in the parse report. Specified output and normalization use
//! cumulative limits and return no partial public result on failure.
//!
//! # Background longhands
//!
//! `background-color` contributes one noninherited [`CssColor`](crate::color::CssColor) with transparent
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

pub(crate) mod background_layer_serialization;
pub(crate) mod background_serialization;
pub(crate) mod position;
pub(crate) mod syntax;

#[doc(inline)]
pub use position::{CssBackgroundPosition, CssBackgroundPositionList};
#[doc(inline)]
pub use syntax::{
    CssBackground, CssBackgroundAttachment, CssBackgroundAttachmentList, CssBackgroundBox,
    CssBackgroundBoxList, CssBackgroundConstructionError, CssBackgroundLayer,
    CssBackgroundLayerBoxes, CssBackgroundRepeat, CssBackgroundRepeatList,
    CssBackgroundRepeatStyle, CssBackgroundSize, CssBackgroundSizeList, CssBlendMode,
    CssBlendModeList,
};
