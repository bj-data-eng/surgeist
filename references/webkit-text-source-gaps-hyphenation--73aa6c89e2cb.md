# WebKit: bounded text source witnesses

This reference retains complete source files from WebKit at immutable revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`, including original file-specific license headers. It supplies bounded evidence for whitespace value support, character-alignment support and committed-line hyphenation state. It is not a CSS specification or a whole-engine conformance claim. These source files are reference material only.

License material: [WebKit LGPL v2](../licenses/webkit/LICENSE-LGPL-2.txt); BSD notices are retained in each applicable file. CSSProperties is not redistributed here; its [existing metadata witness](webkit-cssproperties--73aa6c89e2cb--848d6e24bbd4.md) records its identity.

## RenderStyleConstants.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/rendering/style/RenderStyleConstants.h). Path: `Source/WebCore/rendering/style/RenderStyleConstants.h`. Source bytes: 28541; source lines: 1295; SHA-256: `253181e6b836030ebfef166c96c3839c994dd1d3f427cb48578a9ae97389a17b`.

```cpp
/*
 * Copyright (C) 2000 Lars Knoll (knoll@kde.org)
 *           (C) 2000 Antti Koivisto (koivisto@kde.org)
 *           (C) 2000 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2003-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2006 Graham Dennis (graham.dennis@gmail.com)
 * Copyright (C) 2009 Torch Mobile Inc. All rights reserved. (http://www.torchmobile.com/)
 * Copyright (C) 2026 Samuel Weinig <sam@webkit.org>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

#pragma once

#include <WebCore/BoxSides.h>
#include <initializer_list>
#include <limits>
#include <optional>
#include <type_traits>
#include <wtf/EnumSet.h>
#include <wtf/EnumTraits.h>

namespace WTF {
class String;
class TextStream;
}

namespace WebCore {

enum class DumpStyleValues {
    All,
    NonInitial,
};

enum class PrintColorAdjust : bool {
    Economy,
    Exact
};

// https://drafts.csswg.org/css-values-5/#interpolate-size
enum class InterpolateSize : bool {
    NumericOnly,
    AllowKeywords
};

enum class PseudoElementType : uint8_t {
    // Public:
    FirstLine,
    FirstLetter,
    GrammarError,
    Highlight,
    Marker,
    Before,
    After,
    Selection,
    Backdrop,
    WebKitScrollbar,
    SpellingError,
    TargetText,
    Checkmark,
    PickerIcon,
    ViewTransition,
    ViewTransitionGroup,
    ViewTransitionImagePair,
    ViewTransitionOld,
    ViewTransitionNew,

    // Special: This is only used for getComputedStyle(), and primarily in the event there's no
    // concrete backing element and we have to look up the matching style rules. It is also limited
    // to non-prefixed parts.
    UserAgentPartFallback,

    // Internal:
    WebKitScrollbarThumb,
    WebKitScrollbarButton,
    WebKitScrollbarTrack,
    WebKitScrollbarTrackPiece,
    WebKitScrollbarCorner,
    WebKitResizer,
    InternalWritingSuggestions,

    HighestEnumValue = InternalWritingSuggestions
};

constexpr auto allPublicPseudoElementTypes = EnumSet {
    PseudoElementType::FirstLine,
    PseudoElementType::FirstLetter,
    PseudoElementType::GrammarError,
    PseudoElementType::Highlight,
    PseudoElementType::Marker,
    PseudoElementType::Before,
    PseudoElementType::After,
    PseudoElementType::Selection,
    PseudoElementType::Backdrop,
    PseudoElementType::WebKitScrollbar,
    PseudoElementType::SpellingError,
    PseudoElementType::TargetText,
    PseudoElementType::Checkmark,
    PseudoElementType::PickerIcon,
    PseudoElementType::ViewTransition,
    PseudoElementType::ViewTransitionGroup,
    PseudoElementType::ViewTransitionImagePair,
    PseudoElementType::ViewTransitionOld,
    PseudoElementType::ViewTransitionNew
};

constexpr auto allHighlightPseudoElementTypes = EnumSet {
    PseudoElementType::GrammarError,
    PseudoElementType::Highlight,
    PseudoElementType::Selection,
    PseudoElementType::SpellingError,
    PseudoElementType::TargetText
};

constexpr auto allInternalPseudoElementTypes = EnumSet {
    PseudoElementType::WebKitScrollbarThumb,
    PseudoElementType::WebKitScrollbarButton,
    PseudoElementType::WebKitScrollbarTrack,
    PseudoElementType::WebKitScrollbarTrackPiece,
    PseudoElementType::WebKitScrollbarCorner,
    PseudoElementType::WebKitResizer,
    PseudoElementType::InternalWritingSuggestions,
};

constexpr auto allPseudoElementTypes = allPublicPseudoElementTypes | allInternalPseudoElementTypes;

inline std::optional<PseudoElementType> parentPseudoElement(PseudoElementType pseudoElementType)
{
    switch (pseudoElementType) {
    case PseudoElementType::FirstLetter: return PseudoElementType::FirstLine;
    case PseudoElementType::ViewTransitionGroup: return PseudoElementType::ViewTransition;
    case PseudoElementType::ViewTransitionImagePair: return PseudoElementType::ViewTransitionGroup;
    case PseudoElementType::ViewTransitionNew: return PseudoElementType::ViewTransitionImagePair;
    case PseudoElementType::ViewTransitionOld: return PseudoElementType::ViewTransitionImagePair;
    default: return std::nullopt;
    }
}

enum class ColumnFill : bool {
    Balance,
    Auto
};

enum class ColumnSpan : bool {
    None,
    All
};

enum class BorderCollapse : bool {
    Separate,
    Collapse
};

// These have been defined in the order of their precedence for border-collapsing. Do
// not change this order! This order also must match the order in CSSValueKeywords.in.
enum class BorderStyle : uint8_t {
    None,
    Hidden,
    Inset,
    Groove,
    Outset,
    Ridge,
    Dotted,
    Dashed,
    Solid,
    Double
};

constexpr bool isVisibleBorderStyle(BorderStyle value)
{
    return value > BorderStyle::Hidden;
}

constexpr BorderStyle collapsedBorderStyle(BorderStyle style)
{
    if (style == BorderStyle::Outset)
        return BorderStyle::Groove;
    if (style == BorderStyle::Inset)
        return BorderStyle::Ridge;
    return style;
}

enum class BorderPrecedence : uint8_t {
    Off,
    Table,
    ColumnGroup,
    Column,
    RowGroup,
    Row,
    Cell
};

enum class OutlineStyle : uint8_t {
    Auto,
    None,
    Inset,
    Groove,
    Outset,
    Ridge,
    Dotted,
    Dashed,
    Solid,
    Double
};

enum class PositionType : uint8_t {
    Static = 0,
    Relative = 1,
    Absolute = 2,
    Sticky = 3,
    // This value is required to pack our bits efficiently in RenderObject.
    Fixed = 6
};

enum class Float : uint8_t {
    None,
    Left,
    Right,
    InlineStart,
    InlineEnd,
};

enum class UsedFloat : uint8_t {
    None  = 1 << 0,
    Left  = 1 << 1,
    Right = 1 << 2
};

// Box decoration attributes. Not inherited.

enum class BoxDecorationBreak : bool {
    Slice,
    Clone
};

// Box attributes. Not inherited.

enum class BoxSizing : bool {
    ContentBox,
    BorderBox
};

// Random visual rendering model attributes. Not inherited.

enum class Overflow : uint8_t {
    Visible,
    Hidden,
    Clip,
    Scroll,
    Auto,
    PagedX,
    PagedY
};

constexpr bool isNonVisibleOverflow(Overflow overflow)
{
    return overflow == Overflow::Hidden
        || overflow == Overflow::Scroll
        || overflow == Overflow::Clip;
}

enum class Clear : uint8_t {
    None,
    Left,
    Right,
    InlineStart,
    InlineEnd,
    Both
};

enum class UsedClear : uint8_t {
    None,
    Left,
    Right,
    Both
};

enum class TableLayoutType : bool {
    Auto,
    Fixed
};

enum class SpatialType : bool {
    None,
    Portal
};

enum class PortalActionType : bool {
    None,
    Orbit
};

enum class PositionContextType : bool {
    Container,
    Anchor
};

enum class TextCombine : bool {
    None,
    All
};

enum class FillAttachment : uint8_t {
    ScrollBackground,
    LocalBackground,
    FixedBackground
};

enum class FillBox : uint8_t {
    BorderBox,
    PaddingBox,
    ContentBox,
    BorderArea,
    Text,
    NoClip
};

constexpr unsigned FillBoxBitWidth = 3;

constexpr inline FillBox clipMax(FillBox clipA, FillBox clipB)
{
    if (clipA == FillBox::BorderBox || clipB == FillBox::BorderBox)
        return FillBox::BorderBox;
    if (clipA == FillBox::PaddingBox || clipB == FillBox::PaddingBox)
        return FillBox::PaddingBox;
    if (clipA == FillBox::ContentBox || clipB == FillBox::ContentBox)
        return FillBox::ContentBox;
    return FillBox::NoClip;
}

enum class FillRepeat : uint8_t {
    Repeat,
    NoRepeat,
    Round,
    Space
};

// CSS3 Background Values
enum class FillSizeType : uint8_t {
    Contain,
    Cover,
    Size,
    None
};

// CSS3 <position>
enum class Edge : uint8_t {
    Top,
    Right,
    Bottom,
    Left
};

// CSS3 Marquee Properties

enum class MarqueeBehavior : uint8_t {
    None,
    Scroll,
    Slide,
    Alternate
};

enum class MarqueeDirection : uint8_t {
    Auto,
    Left,
    Right,
    Up,
    Down,
    Forward,
    Backward
};

// Deprecated Flexible Box Properties

enum class BoxPack : uint8_t {
    Start,
    Center,
    End,
    Justify
};

enum class BoxAlignment : uint8_t {
    Stretch,
    Start,
    Center,
    End,
    Baseline
};

enum class BoxOrient : bool {
    Horizontal,
    Vertical
};

enum class BoxLines : bool {
    Single,
    Multiple
};

enum class BoxDirection : bool {
    Normal,
    Reverse
};

// CSS3 Flexbox Properties

enum class FlexDirection : uint8_t {
    Row,
    RowReverse,
    Column,
    ColumnReverse
};

inline AxisDirection toAxisDirection(FlexDirection direction) { return static_cast<AxisDirection>(direction == FlexDirection::RowReverse || direction == FlexDirection::ColumnReverse); }

enum class ItemPosition : uint8_t {
    Legacy,
    Auto,
    Normal,
    Stretch,
    Baseline,
    LastBaseline,
    Center,
    Start,
    End,
    SelfStart,
    SelfEnd,
    FlexStart,
    FlexEnd,
    Left,
    Right,
    AnchorCenter,
};

enum class OverflowAlignment : uint8_t {
    Default,
    Unsafe,
    Safe
};

enum class ItemPositionType : bool {
    NonLegacy,
    Legacy
};

enum class ContentPosition : uint8_t {
    Normal,
    Baseline,
    LastBaseline,
    Center,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Left,
    Right
};

enum class ContentDistribution : uint8_t {
    Default,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch
};


enum class TextSecurity : uint8_t {
    None,
    Disc,
    Circle,
    Square
};

enum class InputSecurity : bool {
    Auto,
    None
};

// CSS3 User Modify Properties

enum class UserModify : uint8_t {
    ReadOnly,
    ReadWrite,
    ReadWritePlaintextOnly
};

// CSS3 User Drag Values

enum class UserDrag : uint8_t {
    Auto,
    None,
    Element
};

// CSS3 User Select Values

enum class UserSelect : uint8_t {
    None,
    Text,
    All,
    Auto
};

// CSS3 Image Values
enum class ObjectFit : uint8_t {
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown
};

enum class AspectRatioType : uint8_t {
    Auto,
    Ratio,
    AutoAndRatio,
    AutoZero
};

enum class WordBreak : uint8_t {
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
    AutoPhrase
};

enum class OverflowWrap : uint8_t {
    Normal,
    BreakWord,
    Anywhere
};

enum class NBSPMode : bool {
    Normal,
    Space
};

enum class LineBreak : uint8_t {
    Auto,
    Loose,
    Normal,
    Strict,
    AfterWhiteSpace,
    Anywhere
};

enum class QuoteType : uint8_t {
    OpenQuote,
    CloseQuote,
    NoOpenQuote,
    NoCloseQuote
};

enum class SynthesizedGlyph : uint8_t {
    PickerUp,
    PickerDown
};

WTF::String fallbackText(SynthesizedGlyph);

enum class AnimationFillMode : uint8_t {
    None,
    Forwards,
    Backwards,
    Both
};

enum class AnimationPlayState : bool {
    Running,
    Paused
};

enum class WhiteSpace : uint8_t {
    Normal,
    Pre,
    PreWrap,
    PreLine,
    NoWrap,
    BreakSpaces
};

enum class WhiteSpaceCollapse : uint8_t {
    Collapse,
    Preserve,
    PreserveBreaks,
    BreakSpaces
};

enum class ReflectionDirection : uint8_t {
    Below,
    Above,
    Left,
    Right
};

enum class TextDecorationStyle : uint8_t {
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy
};

enum class TextJustify : uint8_t {
    Auto,
    None,
    InterWord,
    InterCharacter
};

enum class TextDecorationSkipInk : uint8_t {
    None,
    Auto,
    All
};

enum class TextGroupAlign : uint8_t {
    None,
    Start,
    End,
    Left,
    Right,
    Center
};

enum class TextBoxTrim : uint8_t {
    None,
    TrimStart,
    TrimEnd,
    TrimBoth
};

enum class TextEdgeOver : uint8_t {
    Text,
    Ideographic,
    IdeographicInk,
    Cap,
    Ex
};

enum class TextEdgeUnder : uint8_t {
    Text,
    Ideographic,
    IdeographicInk,
    Alphabetic,
};

enum class TextZoom : bool {
    Normal,
    Reset
};

enum class BreakBetween : uint8_t {
    Auto,
    Avoid,
    AvoidColumn,
    AvoidPage,
    Column,
    Page,
    LeftPage,
    RightPage,
    RectoPage,
    VersoPage
};
bool NODELETE alwaysPageBreak(BreakBetween);
    
enum class BreakInside : uint8_t {
    Auto,
    Avoid,
    AvoidColumn,
    AvoidPage
};

enum class EmptyCell : bool {
    Show,
    Hide
};

enum class CaptionSide : uint8_t {
    Top,
    Bottom
};

enum class ListStylePosition : bool {
    Outside,
    Inside
};

enum class Visibility : uint8_t {
    Visible,
    Hidden,
    Collapse
};

enum class CursorType : uint8_t {
    // The following must match the order in CSSValueKeywords.in.
    Auto,
    Default,
    // None
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    // Copy
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    EResize,
    NResize,
    NEResize,
    NWResize,
    SResize,
    SEResize,
    SWResize,
    WResize,
    EWResize,
    NSResize,
    NESWResize,
    NWSEResize,
    ColumnResize,
    RowResize,
    AllScroll,
    ZoomIn,
    ZoomOut,

    // The following are handled as exceptions so don't need to match.
    Copy,
    None
};

#if ENABLE(CURSOR_VISIBILITY)
enum class CursorVisibility : bool {
    Auto,
    AutoHide,
};
#endif

enum class InsideLink : uint8_t {
    NotInside,
    InsideUnvisited,
    InsideVisited
};
    
enum class PointerEvents : uint8_t {
    None,
    Auto,
    Stroke,
    Fill,
    Painted,
    Visible,
    VisibleStroke,
    VisibleFill,
    VisiblePainted,
    BoundingBox,
    All
};

enum class TransformStyle3D : uint8_t {
    Flat,
    Preserve3D,
#if HAVE(CORE_ANIMATION_SEPARATED_LAYERS)
    Separated
#endif
};

enum class BackfaceVisibility : uint8_t {
    Visible,
    Hidden
};

enum class TransformBox : uint8_t {
    StrokeBox,
    ContentBox,
    BorderBox,
    FillBox,
    ViewBox
};

enum class OverflowContinue : uint8_t {
    Auto,
    Discard,
    WebkitLegacy
};

enum class Hyphens : uint8_t {
    None,
    Manual,
    Auto
};

enum class TextEmphasisFill : bool {
    Filled,
    Open
};

enum class TextEmphasisMark : uint8_t {
    Dot,
    Circle,
    DoubleCircle,
    Triangle,
    Sesame
};

enum class TextWrapMode : bool {
    Wrap,
    NoWrap
};

enum class TextWrapStyle : uint8_t {
    Auto,
    Balance,
    Pretty,
    Stable
};

enum class WrapInside : bool {
    Auto,
    Avoid
};

enum class ImageRendering : uint8_t {
    Auto = 0,
    OptimizeSpeed,
    OptimizeQuality,
    CrispEdges,
    Pixelated
};

enum class Order : bool {
    Logical,
    Visual
};

enum class ColumnAxis : uint8_t {
    Horizontal,
    Vertical,
    Auto
};

enum class ColumnProgression : bool {
    Normal,
    Reverse
};

enum class LineSnap : uint8_t {
    None,
    Baseline,
    Contain
};

enum class LineAlign : bool {
    None,
    Edges
};

enum class RubyPosition : uint8_t {
    Over,
    Under,
    InterCharacter,
    LegacyInterCharacter
};

enum class RubyAlign : uint8_t {
    Start,
    Center,
    SpaceBetween,
    SpaceAround
};

enum class RubyOverhang : bool {
    Auto,
    Spaces
};

enum class ColorScheme : uint8_t {
    Light = 1 << 0,
    Dark = 1 << 1
};

constexpr size_t ColorSchemeBits = 2;

enum class AutoRepeatType : uint8_t {
    None,
    Fill,
    Fit
};

#if USE(FREETYPE)
// The maximum allowed font size is 32767 because `hb_position_t` is `int32_t`,
// where the first 16 bits are used to represent the integer part which effectively makes it `signed short`
constexpr float maximumAllowedFontSize = std::numeric_limits<short>::max();
#else
// Reasonable maximum to prevent insane font sizes from causing crashes on some platforms (such as Windows).
constexpr float maximumAllowedFontSize = 1000000.0f;
#endif

enum class Isolation : bool {
    Auto,
    Isolate
};

// Fill, Stroke, ViewBox are just used for SVG.
enum class CSSBoxType : uint8_t {
    BoxMissing = 0,
    MarginBox,
    BorderBox,
    PaddingBox,
    ContentBox,
    FillBox,
    StrokeBox,
    ViewBox
};

enum class ScrollSnapStrictness : bool {
    Proximity,
    Mandatory
};

enum class ScrollSnapAxis : uint8_t {
    XAxis,
    YAxis,
    Block,
    Inline,
    Both
};

enum class ScrollSnapAxisAlignType : uint8_t {
    None,
    Start,
    Center,
    End
};

enum class ScrollSnapStop : bool {
    Normal,
    Always,
};

enum class FontLoadingBehavior : uint8_t {
    Auto,
    Block,
    Swap,
    Fallback,
    Optional
};

enum class EventListenerRegionType : uint64_t {
    Wheel                      = 1LLU << 0,
    NonPassiveWheel            = 1LLU << 1,
    MouseClick                 = 1LLU << 2,
    TouchStart                 = 1LLU << 3,
    NonPassiveTouchStart       = 1LLU << 4,
    TouchEnd                   = 1LLU << 5,
    NonPassiveTouchEnd         = 1LLU << 6,
    TouchCancel                = 1LLU << 7,
    TouchMove                  = 1LLU << 8,
    NonPassiveTouchMove        = 1LLU << 9,
    TouchForceChange           = 1LLU << 10,
    NonPassiveTouchForceChange = 1LLU << 11,
    PointerDown                = 1LLU << 12,
    NonPassivePointerDown      = 1LLU << 13,
    PointerEnter               = 1LLU << 14,
    NonPassivePointerEnter     = 1LLU << 15,
    PointerLeave               = 1LLU << 16,
    NonPassivePointerLeave     = 1LLU << 17,
    PointerMove                = 1LLU << 18,
    NonPassivePointerMove      = 1LLU << 19,
    PointerOut                 = 1LLU << 20,
    NonPassivePointerOut       = 1LLU << 21,
    PointerOver                = 1LLU << 22,
    NonPassivePointerOver      = 1LLU << 23,
    PointerUp                  = 1LLU << 24,
    NonPassivePointerUp        = 1LLU << 25,
    MouseDown                  = 1LLU << 26,
    NonPassiveMouseDown        = 1LLU << 27,
    MouseUp                    = 1LLU << 28,
    NonPassiveMouseUp          = 1LLU << 29,
    MouseMove                  = 1LLU << 30,
    NonPassiveMouseMove        = 1LLU << 31,
    GestureChange              = 1LLU << 32,
    NonPassiveGestureChange    = 1LLU << 33,
    GestureEnd                 = 1LLU << 34,
    NonPassiveGestureEnd       = 1LLU << 35,
    GestureStart               = 1LLU << 36,
    NonPassiveGestureStart     = 1LLU << 37,
};

enum class MathShift : bool {
    Normal,
    Compact,
};

enum class MathStyle : bool {
    Normal,
    Compact,
};

enum class ContainIntrinsicSizeType : uint8_t {
    None,
    Length,
    AutoAndLength,
    AutoAndNone,
};

enum class ContentVisibility : uint8_t {
    Visible,
    Auto,
    Hidden,
};

enum class BlockStepAlign : uint8_t {
    Auto,
    Center,
    Start,
    End
};

enum class BlockStepInsert : uint8_t {
    MarginBox,
    PaddingBox,
    ContentBox
};

enum class BlockStepRound : uint8_t {
    Up,
    Down,
    Nearest
};

enum class FieldSizing : bool {
    Fixed,
    Content
};

enum class BaselineSource : uint8_t {
    Auto,
    First,
    Last
};

enum class NinePieceImageRule : uint8_t {
    Stretch,
    Round,
    Space,
    Repeat,
};

enum class AnimationDirection : uint8_t {
    Normal,
    Alternate,
    Reverse,
    AlternateReverse
};

enum class TransitionBehavior : bool {
    Normal,
    AllowDiscrete,
};

enum class Scroller : uint8_t {
    Nearest,
    Root,
    Self
};

enum class TextAnchor : uint8_t {
    Start,
    Middle,
    End
};

enum class ColorInterpolation : uint8_t {
    Auto,
    SRGB,
    LinearRGB
};

enum class ShapeRendering : uint8_t {
    Auto,
    OptimizeSpeed,
    CrispEdges,
    GeometricPrecision
};

enum class GlyphOrientation : uint8_t {
    Degrees0,
    Degrees90,
    Degrees180,
    Degrees270,
    Auto
};

enum class AlignmentBaseline : uint8_t {
    Baseline,
    BeforeEdge,
    TextBeforeEdge,
    Middle,
    Central,
    AfterEdge,
    TextAfterEdge,
    Ideographic,
    Alphabetic,
    Hanging,
    Mathematical
};

enum class DominantBaseline : uint8_t {
    Auto,
    Ideographic,
    Alphabetic,
    Hanging,
    Mathematical,
    Central,
    Middle,
    TextAfterEdge,
    TextBeforeEdge
};

enum class VectorEffect : uint8_t {
    None,
    NonScalingStroke
};

enum class BufferedRendering : uint8_t {
    Auto,
    Dynamic,
    Static
};

enum class MaskType : uint8_t {
    Luminance,
    Alpha
};

CSSBoxType NODELETE transformBoxToCSSBoxType(TransformBox);

constexpr float defaultMiterLimit = 4;

enum class UsesSVGZoomRulesForLength : bool { No, Yes };

WTF::TextStream& operator<<(WTF::TextStream&, AnimationDirection);
WTF::TextStream& operator<<(WTF::TextStream&, AnimationFillMode);
WTF::TextStream& operator<<(WTF::TextStream&, AnimationPlayState);
WTF::TextStream& operator<<(WTF::TextStream&, AspectRatioType);
WTF::TextStream& operator<<(WTF::TextStream&, AutoRepeatType);
WTF::TextStream& operator<<(WTF::TextStream&, BackfaceVisibility);
WTF::TextStream& operator<<(WTF::TextStream&, BlockStepAlign);
WTF::TextStream& operator<<(WTF::TextStream&, BlockStepInsert);
WTF::TextStream& operator<<(WTF::TextStream&, BlockStepRound);
WTF::TextStream& operator<<(WTF::TextStream&, BorderCollapse);
WTF::TextStream& operator<<(WTF::TextStream&, BorderStyle);
WTF::TextStream& operator<<(WTF::TextStream&, BoxAlignment);
WTF::TextStream& operator<<(WTF::TextStream&, BoxDecorationBreak);
WTF::TextStream& operator<<(WTF::TextStream&, BoxDirection);
WTF::TextStream& operator<<(WTF::TextStream&, BoxLines);
WTF::TextStream& operator<<(WTF::TextStream&, BoxOrient);
WTF::TextStream& operator<<(WTF::TextStream&, BoxPack);
WTF::TextStream& operator<<(WTF::TextStream&, BoxSizing);
WTF::TextStream& operator<<(WTF::TextStream&, BreakBetween);
WTF::TextStream& operator<<(WTF::TextStream&, BreakInside);
WTF::TextStream& operator<<(WTF::TextStream&, CSSBoxType);
WTF::TextStream& operator<<(WTF::TextStream&, CaptionSide);
WTF::TextStream& operator<<(WTF::TextStream&, Clear);
WTF::TextStream& operator<<(WTF::TextStream&, UsedClear);
#if ENABLE(DARK_MODE_CSS)
WTF::TextStream& operator<<(WTF::TextStream&, ColorScheme);
#endif
WTF::TextStream& operator<<(WTF::TextStream&, ColumnAxis);
WTF::TextStream& operator<<(WTF::TextStream&, ColumnFill);
WTF::TextStream& operator<<(WTF::TextStream&, ColumnProgression);
WTF::TextStream& operator<<(WTF::TextStream&, ColumnSpan);
WTF::TextStream& operator<<(WTF::TextStream&, ContentDistribution);
WTF::TextStream& operator<<(WTF::TextStream&, ContentPosition);
WTF::TextStream& operator<<(WTF::TextStream&, ContentVisibility);
WTF::TextStream& operator<<(WTF::TextStream&, CursorType);
#if ENABLE(CURSOR_VISIBILITY)
WTF::TextStream& operator<<(WTF::TextStream&, CursorVisibility);
#endif
WTF::TextStream& operator<<(WTF::TextStream&, Edge);
WTF::TextStream& operator<<(WTF::TextStream&, EmptyCell);
WTF::TextStream& operator<<(WTF::TextStream&, EventListenerRegionType);
WTF::TextStream& operator<<(WTF::TextStream&, FillAttachment);
WTF::TextStream& operator<<(WTF::TextStream&, FillBox);
WTF::TextStream& operator<<(WTF::TextStream&, FillRepeat);
WTF::TextStream& operator<<(WTF::TextStream&, FillSizeType);
WTF::TextStream& operator<<(WTF::TextStream&, FlexDirection);
WTF::TextStream& operator<<(WTF::TextStream&, Float);
WTF::TextStream& operator<<(WTF::TextStream&, UsedFloat);
WTF::TextStream& operator<<(WTF::TextStream&, Hyphens);
WTF::TextStream& operator<<(WTF::TextStream&, ImageRendering);
WTF::TextStream& operator<<(WTF::TextStream&, InsideLink);
WTF::TextStream& operator<<(WTF::TextStream&, InterpolateSize);
WTF::TextStream& operator<<(WTF::TextStream&, Isolation);
WTF::TextStream& operator<<(WTF::TextStream&, ItemPosition);
WTF::TextStream& operator<<(WTF::TextStream&, ItemPositionType);
WTF::TextStream& operator<<(WTF::TextStream&, LineAlign);
WTF::TextStream& operator<<(WTF::TextStream&, LineBreak);
WTF::TextStream& operator<<(WTF::TextStream&, LineSnap);
WTF::TextStream& operator<<(WTF::TextStream&, ListStylePosition);
WTF::TextStream& operator<<(WTF::TextStream&, MarqueeBehavior);
WTF::TextStream& operator<<(WTF::TextStream&, MarqueeDirection);
WTF::TextStream& operator<<(WTF::TextStream&, NBSPMode);
WTF::TextStream& operator<<(WTF::TextStream&, NinePieceImageRule);
WTF::TextStream& operator<<(WTF::TextStream&, ObjectFit);
WTF::TextStream& operator<<(WTF::TextStream&, Order);
WTF::TextStream& operator<<(WTF::TextStream&, OutlineStyle);
WTF::TextStream& operator<<(WTF::TextStream&, WebCore::Overflow);
WTF::TextStream& operator<<(WTF::TextStream&, OverflowAlignment);
WTF::TextStream& operator<<(WTF::TextStream&, OverflowWrap);
WTF::TextStream& operator<<(WTF::TextStream&, PointerEvents);
WTF::TextStream& operator<<(WTF::TextStream&, PortalActionType);
WTF::TextStream& operator<<(WTF::TextStream&, PositionContextType);
WTF::TextStream& operator<<(WTF::TextStream&, PositionType);
WTF::TextStream& operator<<(WTF::TextStream&, PrintColorAdjust);
WTF::TextStream& operator<<(WTF::TextStream&, PseudoElementType);
WTF::TextStream& operator<<(WTF::TextStream&, QuoteType);
WTF::TextStream& operator<<(WTF::TextStream&, SynthesizedGlyph);
WTF::TextStream& operator<<(WTF::TextStream&, ReflectionDirection);
WTF::TextStream& operator<<(WTF::TextStream&, RubyPosition);
WTF::TextStream& operator<<(WTF::TextStream&, RubyAlign);
WTF::TextStream& operator<<(WTF::TextStream&, RubyOverhang);
WTF::TextStream& operator<<(WTF::TextStream&, ScrollSnapAxis);
WTF::TextStream& operator<<(WTF::TextStream&, ScrollSnapAxisAlignType);
WTF::TextStream& operator<<(WTF::TextStream&, ScrollSnapStop);
WTF::TextStream& operator<<(WTF::TextStream&, ScrollSnapStrictness);
WTF::TextStream& operator<<(WTF::TextStream&, Scroller);
WTF::TextStream& operator<<(WTF::TextStream&, SpatialType);
WTF::TextStream& operator<<(WTF::TextStream&, TableLayoutType);
WTF::TextStream& operator<<(WTF::TextStream&, TextCombine);
WTF::TextStream& operator<<(WTF::TextStream&, TextDecorationSkipInk);
WTF::TextStream& operator<<(WTF::TextStream&, TextDecorationStyle);
WTF::TextStream& operator<<(WTF::TextStream&, TextEmphasisFill);
WTF::TextStream& operator<<(WTF::TextStream&, TextEmphasisMark);
WTF::TextStream& operator<<(WTF::TextStream&, TextGroupAlign);
WTF::TextStream& operator<<(WTF::TextStream&, TextJustify);
WTF::TextStream& operator<<(WTF::TextStream&, TextSecurity);
WTF::TextStream& operator<<(WTF::TextStream&, TextWrapMode);
WTF::TextStream& operator<<(WTF::TextStream&, TextWrapStyle);
WTF::TextStream& operator<<(WTF::TextStream&, WrapInside);
WTF::TextStream& operator<<(WTF::TextStream&, TextBoxTrim);
WTF::TextStream& operator<<(WTF::TextStream&, TextEdgeOver);
WTF::TextStream& operator<<(WTF::TextStream&, TextEdgeUnder);
WTF::TextStream& operator<<(WTF::TextStream&, TextZoom);
WTF::TextStream& operator<<(WTF::TextStream&, TransformBox);
WTF::TextStream& operator<<(WTF::TextStream&, TransformStyle3D);
WTF::TextStream& operator<<(WTF::TextStream&, TransitionBehavior);
WTF::TextStream& operator<<(WTF::TextStream&, UserDrag);
WTF::TextStream& operator<<(WTF::TextStream&, UserModify);
WTF::TextStream& operator<<(WTF::TextStream&, UserSelect);
WTF::TextStream& operator<<(WTF::TextStream&, Visibility);
WTF::TextStream& operator<<(WTF::TextStream&, WhiteSpace);
WTF::TextStream& operator<<(WTF::TextStream&, WhiteSpaceCollapse);
WTF::TextStream& operator<<(WTF::TextStream&, WordBreak);
WTF::TextStream& operator<<(WTF::TextStream&, MathShift);
WTF::TextStream& operator<<(WTF::TextStream&, MathStyle);
WTF::TextStream& operator<<(WTF::TextStream&, ContainIntrinsicSizeType);
WTF::TextStream& operator<<(WTF::TextStream&, FieldSizing);
WTF::TextStream& operator<<(WTF::TextStream&, BaselineSource);
WTF::TextStream& operator<<(WTF::TextStream&, OverflowContinue);

WTF::TextStream& operator<<(WTF::TextStream&, AlignmentBaseline);
WTF::TextStream& operator<<(WTF::TextStream&, BufferedRendering);
WTF::TextStream& operator<<(WTF::TextStream&, ColorInterpolation);
WTF::TextStream& operator<<(WTF::TextStream&, DominantBaseline);
WTF::TextStream& operator<<(WTF::TextStream&, GlyphOrientation);
WTF::TextStream& operator<<(WTF::TextStream&, MaskType);
WTF::TextStream& operator<<(WTF::TextStream&, ShapeRendering);
WTF::TextStream& operator<<(WTF::TextStream&, TextAnchor);
WTF::TextStream& operator<<(WTF::TextStream&, VectorEffect);

} // namespace WebCore
```

## StyleTextAlign.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/style/values/text/StyleTextAlign.h). Path: `Source/WebCore/style/values/text/StyleTextAlign.h`. Source bytes: 2193; source lines: 56; SHA-256: `0894e1978008e70ef3dcbccc881970df776d44168e11ffb6cd898cc0c74b28bf`.

```cpp
/*
 * Copyright (C) 2025 Samuel Weinig <sam@webkit.org>
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <WebCore/StyleValueTypes.h>

namespace WebCore {
namespace Style {

// <'text-align'> = start | end | left | right | center | justify | match-parent | justify-all | -webkit-left | -webkit-right | -webkit-center
// NOTE: `match-parent` is computed to a specific alignment during style building.
// FIXME: Support `justify-all`
// https://drafts.csswg.org/css-text/#propdef-text-align

// The order of this enum must match the order of the text align values in CSSValueKeywords.in.
enum class TextAlign : uint8_t {
    Left,
    Right,
    Center,
    Justify,
    WebKitLeft,
    WebKitRight,
    WebKitCenter,
    Start,
    End,
};

// MARK: - Conversion

// NOTE: Custom conversion is required to resolve `match-parent` and `-internal-th-center`.
template<> struct CSSValueConversion<TextAlign> { auto operator()(BuilderState&, const CSSValue&) -> TextAlign; };

} // namespace Style
} // namespace WebCore
```

## TextBreakingPositionContext.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/text/TextBreakingPositionContext.h). Path: `Source/WebCore/layout/formattingContexts/inline/text/TextBreakingPositionContext.h`. Source bytes: 4196; source lines: 102; SHA-256: `6ce24f8b1e728416f7f8dce5b6aef1832886d4a6fc0e50b22e86635923984a42`.

```cpp
/*
 * Copyright (C) 2024 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <WebCore/StyleComputedStyle.h>
#include <WebCore/StyleWebKitLocale.h>
#include <wtf/HashFunctions.h>
#include <wtf/HashMap.h>
#include <wtf/Hasher.h>

namespace WebCore {
namespace Layout {

enum class WhiteSpaceCollapseBehavior : uint8_t {
    CollapseAll,
    CollapseWhitespaceSequence,
    Preserve
};

static WhiteSpaceCollapseBehavior BehaviorForWhiteSpaceCollapse(WhiteSpaceCollapse collapse)
{
    switch (collapse) {
    case WhiteSpaceCollapse::Collapse:
        return WhiteSpaceCollapseBehavior::CollapseAll;
    case WhiteSpaceCollapse::PreserveBreaks:
        return WhiteSpaceCollapseBehavior::CollapseWhitespaceSequence;
    case WhiteSpaceCollapse::Preserve:
    case WhiteSpaceCollapse::BreakSpaces:
        // TODO: WhiteSpaceCollapse::PreserveSpaces should also be handled this way.
        // WhiteSpaceCollapse::Preserve and WhiteSpaceCollapse::BreakSpaces
        // have the same text breaking positions so we don't need to recalculate
        // breaking points when switching between these behaviors.
        return WhiteSpaceCollapseBehavior::Preserve;
    }
    ASSERT_NOT_REACHED();
    return WhiteSpaceCollapseBehavior::CollapseAll;
}

struct TextBreakingPositionContext {
    WhiteSpaceCollapseBehavior whitespaceCollapseBehavior { WhiteSpaceCollapseBehavior::CollapseAll };
    OverflowWrap overflowWrap { OverflowWrap::Normal };
    LineBreak lineBreak { LineBreak::Normal };
    WordBreak wordBreak { WordBreak::Normal };
    NBSPMode nbspMode { NBSPMode::Normal };
    AtomString locale;

    bool isHashTableDeletedValue { false };

    TextBreakingPositionContext(const Style::ComputedStyle&);
    TextBreakingPositionContext() = default;

    friend bool operator==(const TextBreakingPositionContext&, const TextBreakingPositionContext&) = default;
};

inline TextBreakingPositionContext::TextBreakingPositionContext(const Style::ComputedStyle& style)
    : whitespaceCollapseBehavior(BehaviorForWhiteSpaceCollapse(style.whiteSpaceCollapse()))
    , overflowWrap(style.overflowWrap())
    , lineBreak(style.lineBreak())
    , wordBreak(style.wordBreak())
    , nbspMode(style.nbspMode())
    , locale(Style::toPlatform(style.usedLocale()))
{
}

void add(Hasher&, const TextBreakingPositionContext&);

} // namespace Layout
} // namespace WebCore

namespace WTF {

template<>
struct HashTraits<WebCore::Layout::TextBreakingPositionContext> : GenericHashTraits<WebCore::Layout::TextBreakingPositionContext> {
    static void constructDeletedValue(WebCore::Layout::TextBreakingPositionContext& slot) { slot.isHashTableDeletedValue = true; }
    static bool isDeletedValue(const WebCore::Layout::TextBreakingPositionContext& value) { return value.isHashTableDeletedValue; }
    static WebCore::Layout::TextBreakingPositionContext emptyValue() { return { }; }
};

} // namespace WTF
```

## InlineLayoutState.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineLayoutState.h). Path: `Source/WebCore/layout/formattingContexts/inline/InlineLayoutState.h`. Source bytes: 6900; source lines: 122; SHA-256: `4ea1e1b7a071c31294a473ec383ae0425d68dd6f573f3d366206c4312242525e`.

```cpp
/*
 * Copyright (C) 2023 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <WebCore/AvailableLineWidthOverride.h>
#include <WebCore/BlockLayoutState.h>

namespace WebCore {
namespace Layout {

class InlineLayoutState {
public:
    InlineLayoutState(BlockLayoutState&);

    void setClearGapAfterLastLine(InlineLayoutUnit verticalGap);
    InlineLayoutUnit clearGapAfterLastLine() const { return m_clearGapAfterLastLine; }

    void setClearGapBeforeFirstLine(InlineLayoutUnit verticalGap) { m_clearGapBeforeFirstLine = verticalGap; }
    InlineLayoutUnit clearGapBeforeFirstLine() const { return m_clearGapBeforeFirstLine; }

    const BlockLayoutState& parentBlockLayoutState() const LIFETIME_BOUND { return m_parentBlockLayoutState; }
    BlockLayoutState& parentBlockLayoutState() LIFETIME_BOUND { return m_parentBlockLayoutState; }

    const PlacedFloats& placedFloats() const LIFETIME_BOUND { return m_parentBlockLayoutState.placedFloats(); }
    PlacedFloats& placedFloats() LIFETIME_BOUND { return m_parentBlockLayoutState.placedFloats(); }

    void setAvailableLineWidthOverride(AvailableLineWidthOverride availableLineWidthOverride) { m_availableLineWidthOverride = availableLineWidthOverride; }
    const AvailableLineWidthOverride& availableLineWidthOverride() const LIFETIME_BOUND { return m_availableLineWidthOverride; }

    void setLegacyClampedLineIndex(size_t lineIndex) { m_legacyClampedLineIndex = lineIndex; }
    std::optional<size_t> legacyClampedLineIndex() const { return m_legacyClampedLineIndex; }

    void setLineCount(size_t lineCount) { m_lineCount = lineCount; }
    void incrementLineCount() { ++m_lineCount; }
    size_t lineCount() const { return m_lineCount; }

    void setLineCountWithInlineContentIncludingNestedBlocks(size_t lineCount) { m_lineCountWithInlineContentIncludingNestedBlocks = lineCount; }
    size_t lineCountWithInlineContentIncludingNestedBlocks() const { return m_lineCountWithInlineContentIncludingNestedBlocks; }

    void setHyphenationLimitLines(size_t hyphenateLimitLines) { m_hyphenateLimitLines = hyphenateLimitLines; }
    void incrementSuccessiveHyphenatedLineCount() { ++m_successiveHyphenatedLineCount; }
    void resetSuccessiveHyphenatedLineCount() { m_successiveHyphenatedLineCount = 0; }
    bool isHyphenationDisabled() const { return m_hyphenateLimitLines && *m_hyphenateLimitLines <= m_successiveHyphenatedLineCount; }

    void setFirstLineStartTrimForInitialLetter(InlineLayoutUnit trimmedThisMuch) { m_firstLineStartTrimForInitialLetter = trimmedThisMuch; }
    InlineLayoutUnit firstLineStartTrimForInitialLetter() const { return m_firstLineStartTrimForInitialLetter; }

    void setInStandardsMode() { m_inStandardsMode = true; }
    bool inStandardsMode() const { return m_inStandardsMode; }

    void setShouldShapeTextAcrossInlineBoxes() { m_shouldShapeTextAcrossInlineBoxes = true; }
    bool shouldShapeTextAcrossInlineBoxes() const { return m_shouldShapeTextAcrossInlineBoxes; }

    // Excluded list markers belonging to ancestor list items, which are aligned with this formatting context's first formatted line but are no part of its content.
    using AscentAndDescent = std::pair<InlineLayoutUnit, InlineLayoutUnit>;
    void setExcludedMarkerLayoutBounds(Vector<AscentAndDescent>&& layoutBounds) { m_excludedMarkerLayoutBounds = WTF::move(layoutBounds); }
    const Vector<AscentAndDescent>& excludedMarkerLayoutBounds() const LIFETIME_BOUND { return m_excludedMarkerLayoutBounds; }

    void setNestedListMarkerOffsets(HashMap<CheckedRef<const ElementBox>, LayoutUnit>&& nestedListMarkerOffsets) { m_nestedListMarkerOffsets = WTF::move(nestedListMarkerOffsets); }
    LayoutUnit nestedListMarkerOffset(const ElementBox& listMarkerBox) const { return m_nestedListMarkerOffsets.get(listMarkerBox); }
    void setShouldNotSynthesizeInlineBlockBaseline() { m_shouldNotSynthesizeInlineBlockBaseline = true; }
    bool shouldNotSynthesizeInlineBlockBaseline() const { return m_shouldNotSynthesizeInlineBlockBaseline; }

    void setContentMayHaveInkOverflow(bool mayHaveInkOverflow) { m_contentMayHaveInkOverflow |= mayHaveInkOverflow; }
    bool contentMayHaveInkOverflow() const { return m_contentMayHaveInkOverflow; }

private:
    BlockLayoutState& m_parentBlockLayoutState;
    InlineLayoutUnit m_clearGapBeforeFirstLine { 0.f };
    InlineLayoutUnit m_clearGapAfterLastLine { 0.f };
    InlineLayoutUnit m_firstLineStartTrimForInitialLetter { 0.f };
    std::optional<size_t> m_legacyClampedLineIndex { };
    std::optional<size_t> m_hyphenateLimitLines { };
    size_t m_successiveHyphenatedLineCount { 0 };
    size_t m_lineCount { 0 }; // Note that this does not include lines from nested blocks and it does not include lines with no content either.
    size_t m_lineCountWithInlineContentIncludingNestedBlocks { 0 };
    // FIXME: This is required by the integaration codepath.
    HashMap<CheckedRef<const ElementBox>, LayoutUnit> m_nestedListMarkerOffsets;
    Vector<AscentAndDescent> m_excludedMarkerLayoutBounds;
    AvailableLineWidthOverride m_availableLineWidthOverride;
    bool m_contentMayHaveInkOverflow { false };
    bool m_shouldNotSynthesizeInlineBlockBaseline { false };
    bool m_inStandardsMode { false };
    bool m_shouldShapeTextAcrossInlineBoxes { false };
};

inline InlineLayoutState::InlineLayoutState(BlockLayoutState& parentBlockLayoutState)
    : m_parentBlockLayoutState(parentBlockLayoutState)
{
}

inline void InlineLayoutState::setClearGapAfterLastLine(InlineLayoutUnit verticalGap)
{
    ASSERT(verticalGap >= 0);
    m_clearGapAfterLastLine = verticalGap;
}

}
}
```

## InlineFormattingContext.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineFormattingContext.cpp). Path: `Source/WebCore/layout/formattingContexts/inline/InlineFormattingContext.cpp`. Source bytes: 35452; source lines: 654; SHA-256: `8b37db271540bc16efd804ce3e01407befa2c9e29491bfa2c4d6b8e3e3e42ed0`.

```cpp
/*
 * Copyright (C) 2018-2024 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "InlineFormattingContext.h"

#include "AvailableLineWidthOverride.h"
#include "FloatingContext.h"
#include "FontCascade.h"
#include "InlineContentCache.h"
#include "InlineContentConstrainer.h"
#include "InlineDamage.h"
#include "InlineDisplayBox.h"
#include "InlineDisplayContentBuilder.h"
#include "InlineDisplayLineBuilder.h"
#include "InlineInvalidation.h"
#include "InlineItemsBuilder.h"
#include "InlineLayoutState.h"
#include "InlineLineBox.h"
#include "InlineLineBoxBuilder.h"
#include "InlineLineTypes.h"
#include "InlineTextItem.h"
#include "IntrinsicWidthHandler.h"
#include "LayoutBox.h"
#include "LayoutContext.h"
#include "LayoutDescendantIterator.h"
#include "LayoutElementBox.h"
#include "LayoutInitialContainingBlock.h"
#include "LayoutInlineTextBox.h"
#include "LayoutIntegrationUtils.h"
#include "LayoutState.h"
#include "Logging.h"
#include "RangeBasedLineBuilder.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "TextOnlySimpleLineBuilder.h"
#include "TextUtil.h"
#include <wtf/TZoneMallocInlines.h>
#include <wtf/text/TextStream.h>

namespace WebCore {
namespace Layout {

WTF_MAKE_TZONE_ALLOCATED_IMPL(InlineContentCache);
WTF_MAKE_TZONE_ALLOCATED_IMPL(InlineFormattingContext);
WTF_MAKE_TZONE_ALLOCATED_IMPL(InlineLayoutResult);

static std::optional<InlineItemRange> NODELETE partialRangeForDamage(const InlineItemList& inlineItemList, const InlineDamage& lineDamage)
{
    auto layoutStartPosition = lineDamage.layoutStartPosition()->inlineItemPosition;
    if (layoutStartPosition.index >= inlineItemList.size()) {
        ASSERT_NOT_REACHED();
        return { };
    }
    auto* damagedInlineTextItem = dynamicDowncast<InlineTextItem>(inlineItemList[layoutStartPosition.index]);
    if (layoutStartPosition.offset && (!damagedInlineTextItem || layoutStartPosition.offset >= damagedInlineTextItem->length())) {
        ASSERT_NOT_REACHED();
        return { };
    }
    return InlineItemRange { layoutStartPosition, { inlineItemList.size(), 0 } };
}

static bool NODELETE isEmptyInlineContent(const InlineItemList& inlineItemList)
{
    if (inlineItemList.isEmpty())
        return true;

    // Very common, pseudo before/after empty content.
    if (inlineItemList.size() != 1)
        return false;

    auto* inlineTextItem = dynamicDowncast<InlineTextItem>(inlineItemList[0]);
    return inlineTextItem && !inlineTextItem->length();
}

InlineFormattingContext::InlineFormattingContext(const ElementBox& rootBlockContainer, LayoutState& globalLayoutState, BlockLayoutState& parentBlockLayoutState)
    : m_rootBlockContainer(rootBlockContainer)
    , m_globalLayoutState(globalLayoutState)
    , m_floatingContext(rootBlockContainer, globalLayoutState, parentBlockLayoutState.placedFloats())
    , m_inlineFormattingUtils(*this)
    , m_inlineQuirks(*this)
    , m_integrationUtils(globalLayoutState)
    , m_inlineContentCache(globalLayoutState.inlineContentCache(rootBlockContainer))
    , m_inlineLayoutState(parentBlockLayoutState)
{
    initializeInlineLayoutState(globalLayoutState);
}

std::unique_ptr<InlineLayoutResult> InlineFormattingContext::layout(const ConstraintsForInlineContent& constraints, InlineDamage* lineDamage)
{
    rebuildInlineItemListIfNeeded(lineDamage);

    if (formattingUtils().shouldDiscardRemainingContentInBlockDirection()) {
        // This inline content may be completely collapsed (i.e. after clamped block container)
        resetBoxGeometriesForDiscardedContent({ { }, { inlineContentCache().inlineItems().content().size(), { } } }, { });
        return { };
    }

    auto hasExcludedMarker = !layoutState().excludedMarkerLayoutBounds().isEmpty();
    if (!root().hasInFlowChild() && !root().hasOutOfFlowChild() && !hasExcludedMarker) {
        // Float only content does not support partial layout.
        ASSERT(!InlineInvalidation::mayOnlyNeedPartialLayout(lineDamage));
        layoutFloatContentOnly(constraints);
        return { };
    }

    auto& inlineItems = inlineContentCache().inlineItems();
    auto& inlineItemList = inlineItems.content();
    if (inlineItemList.isEmpty() && hasExcludedMarker) {
        auto layoutResult = makeUniqueRef<InlineLayoutResult>();
        createDisplayContentForEmptyInlineContent(constraints, inlineItemList, layoutResult.get());
        layoutResult->range = InlineLayoutResult::Range::Full;
        return layoutResult.moveToUniquePtr();
    }

    auto needsLayoutRange = [&]() -> InlineItemRange {
        if (!InlineInvalidation::mayOnlyNeedPartialLayout(lineDamage))
            return { { }, { inlineItemList.size(), { } } };
        if (auto partialRange = partialRangeForDamage(inlineItemList, *lineDamage))
            return *partialRange;
        // We should be able to produce partial range for partial layout.
        ASSERT_NOT_REACHED();
        // Let's turn this unexpected state to full layout.
        lineDamage = nullptr;
        return { { }, { inlineItemList.size(), { } } };
    }();

    if (needsLayoutRange.isEmpty()) {
        ASSERT_NOT_REACHED();
        return { };
    }

    auto previousLine = [&]() -> std::optional<PreviousLine> {
        if (!needsLayoutRange.start)
            return { };
        if (!lineDamage || !lineDamage->layoutStartPosition()) {
            ASSERT_NOT_REACHED();
            return { };
        }
        if (!lineDamage->layoutStartPosition()->lineIndex)
            return { };
        auto lastLineIndex = lineDamage->layoutStartPosition()->lineIndex - 1;
        // FIXME: We should be able to extract the last line information and provide it to layout as "previous line" (ends in line break and inline direction).
        return PreviousLine { lastLineIndex, { }, { }, true, { }, { } };
    }();

    auto& inlineLayoutState = layoutState();
    inlineLayoutState.setLineCount(previousLine ? previousLine->lineIndex + 1lu : 0lu);
    // FIXME: This needs partial support when line-clamped content has nested blocks.
    inlineLayoutState.setLineCountWithInlineContentIncludingNestedBlocks(inlineLayoutState.lineCount());

    auto textWrapStyle = root().style().textWrapStyle();
    if (root().style().textWrapMode() == TextWrapMode::Wrap && (textWrapStyle == TextWrapStyle::Balance || textWrapStyle == TextWrapStyle::Pretty)) {
        auto constrainer = InlineContentConstrainer { *this, inlineItemList, constraints.horizontal() };
        auto constrainedLineWidths = constrainer.computeParagraphLevelConstraints(textWrapStyle);
        if (constrainedLineWidths)
            inlineLayoutState.setAvailableLineWidthOverride({ *constrainedLineWidths });
    }

    if (TextOnlySimpleLineBuilder::isEligibleForSimplifiedTextOnlyInlineLayoutByContent(inlineItems, inlineLayoutState.placedFloats()) && TextOnlySimpleLineBuilder::isEligibleForSimplifiedInlineLayoutByStyle(root())) {
        auto simplifiedLineBuilder = makeUniqueRef<TextOnlySimpleLineBuilder>(*this, root(), constraints.horizontal(), inlineItemList);
        auto shouldUseSimplifiedDisplayContentBuild = [&] {
            if (lineDamage || !m_globalLayoutState.inStandardsMode() || inlineLayoutState.parentBlockLayoutState().lineClamp())
                return false;
            return TextOnlySimpleLineBuilder::isEligibleForSimplifiedDisplayBuild(root());
        };
        return lineLayout(simplifiedLineBuilder, inlineItemList, needsLayoutRange, previousLine, constraints, lineDamage, shouldUseSimplifiedDisplayContentBuild()).moveToUniquePtr();
    }
    if (RangeBasedLineBuilder::isEligibleForRangeInlineLayout(*this, needsLayoutRange, inlineItems, inlineLayoutState.placedFloats())) {
        auto rangeBasedLineBuilder = makeUniqueRef<RangeBasedLineBuilder>(*this, constraints.horizontal(), inlineItems);
        return lineLayout(rangeBasedLineBuilder, inlineItemList, needsLayoutRange, previousLine, constraints, lineDamage).moveToUniquePtr();
    }
    auto lineBuilder = makeUniqueRef<LineBuilder>(*this, constraints.horizontal(), inlineItemList, inlineContentCache().textSpacingContext());
    return lineLayout(lineBuilder, inlineItemList, needsLayoutRange, previousLine, constraints, lineDamage).moveToUniquePtr();
}

std::pair<LayoutUnit, LayoutUnit> InlineFormattingContext::minimumMaximumContentSize(InlineDamage* lineDamage)
{
    auto& inlineContentCache = this->inlineContentCache();
    auto minimumContentSize = inlineContentCache.minimumContentSize();
    auto maximumContentSize = inlineContentCache.maximumContentSize();

    if (minimumContentSize && maximumContentSize)
        return { ceiledLayoutUnit(*minimumContentSize), ceiledLayoutUnit(*maximumContentSize) };

    rebuildInlineItemListIfNeeded(lineDamage);
    auto& inlineItems = inlineContentCache.inlineItems();

    if (!isEmptyInlineContent(inlineItems.content())) {
        auto intrinsicWidthHandler = IntrinsicWidthHandler { *this, inlineItems };

        if (!minimumContentSize)
            minimumContentSize = intrinsicWidthHandler.minimumContentSize();
        if (!maximumContentSize) {
            maximumContentSize = intrinsicWidthHandler.maximumContentSize();
            if (intrinsicWidthHandler.maximumIntrinsicWidthLineContent())
                inlineContentCache.setMaximumIntrinsicWidthLineContent(WTF::move(*intrinsicWidthHandler.maximumIntrinsicWidthLineContent()));
        }
    } else {
        minimumContentSize = minimumContentSize.value_or(0.f);
        maximumContentSize = maximumContentSize.value_or(0.f);
    }
#ifndef NDEBUG
    // FIXME: "Nominally, the smallest size a box could take that doesn’t lead to overflow that could be avoided by choosing a larger size.
    // Formally, the size of the box when sized under a min-content constraint"
    // 'nominally' seems to overrule 'formally' when inline content has negative text indent.
    // This also undermines the idea of computing min/max values independently.
    if (*minimumContentSize > *maximumContentSize) {
        auto hasNegativeImplicitMargin = [](auto& style) {
            auto textIndentFixedLength = style.textIndent().amount.tryFixed();
            return (textIndentFixedLength && textIndentFixedLength->isNegative()) || style.usedWordSpacing() < 0 || style.usedLetterSpacing() < 0;
        };
        auto contentHasNegativeImplicitMargin = hasNegativeImplicitMargin(root().style());
        if (!contentHasNegativeImplicitMargin) {
            for (auto& layoutBox : descendantsOfType<Box>(root())) {
                contentHasNegativeImplicitMargin = hasNegativeImplicitMargin(layoutBox.style());
                if (contentHasNegativeImplicitMargin)
                    break;
            }
        }
        ASSERT(contentHasNegativeImplicitMargin);
    }
#endif
    minimumContentSize = std::min(*minimumContentSize, *maximumContentSize);

    inlineContentCache.setMinimumContentSize(*minimumContentSize);
    inlineContentCache.setMaximumContentSize(*maximumContentSize);
    return { ceiledLayoutUnit(*minimumContentSize), ceiledLayoutUnit(*maximumContentSize) };
}

LayoutUnit InlineFormattingContext::minimumContentSize(InlineDamage* lineDamage)
{
    auto& inlineContentCache = this->inlineContentCache();
    if (inlineContentCache.minimumContentSize())
        return ceiledLayoutUnit(*inlineContentCache.minimumContentSize());

    rebuildInlineItemListIfNeeded(lineDamage);
    auto& inlineItems = inlineContentCache.inlineItems();
    auto minimumContentSize = InlineLayoutUnit { };
    if (!isEmptyInlineContent(inlineItems.content()))
        minimumContentSize = IntrinsicWidthHandler { *this, inlineItems }.minimumContentSize();
    inlineContentCache.setMinimumContentSize(minimumContentSize);
    return ceiledLayoutUnit(minimumContentSize);
}

LayoutUnit InlineFormattingContext::maximumContentSize(InlineDamage* lineDamage)
{
    auto& inlineContentCache = this->inlineContentCache();
    if (inlineContentCache.maximumContentSize())
        return ceiledLayoutUnit(*inlineContentCache.maximumContentSize());

    rebuildInlineItemListIfNeeded(lineDamage);
    auto& inlineItems = inlineContentCache.inlineItems();
    auto maximumContentSize = InlineLayoutUnit { };
    if (!isEmptyInlineContent(inlineItems.content())) {
        auto intrinsicWidthHandler = IntrinsicWidthHandler { *this, inlineItems };

        maximumContentSize = intrinsicWidthHandler.maximumContentSize();
        if (intrinsicWidthHandler.maximumIntrinsicWidthLineContent())
            inlineContentCache.setMaximumIntrinsicWidthLineContent(WTF::move(*intrinsicWidthHandler.maximumIntrinsicWidthLineContent()));
    }
    inlineContentCache.setMaximumContentSize(maximumContentSize);
    return ceiledLayoutUnit(maximumContentSize);
}

static bool mayExitFromPartialLayout(const InlineDamage& lineDamage, size_t lineIndex, const InlineDisplay::Boxes& newContent)
{
    if (lineDamage.layoutStartPosition()->lineIndex == lineIndex) {
        // Never stop at the damaged line. Adding trailing overflowing content could easily produce the
        // same set of display boxes for the first damaged line.
        return false;
    }
    auto trailingContentFromPreviousLayout = lineDamage.trailingContentForLine(lineIndex);
    return trailingContentFromPreviousLayout ? (!newContent.isEmpty() && *trailingContentFromPreviousLayout == newContent.last()) : false;
}

static inline void handleAfterSideMargin(BlockLayoutState::MarginState& marginState, InlineDisplay::Content& displayContent)
{
    if (!InlineDisplayLineBuilder::hasTrailingLineWithBlockContent(displayContent.lines))
        marginState.canCollapseMarginAfterWithChildren = false;
}

UniqueRef<InlineLayoutResult> InlineFormattingContext::lineLayout(AbstractLineBuilder& lineBuilder, const InlineItemList& inlineItemList, InlineItemRange needsLayoutRange, std::optional<PreviousLine> previousLine, const ConstraintsForInlineContent& constraints, const InlineDamage* lineDamage, bool mayUseSimplifiedDisplayContentBuild)
{
    ASSERT(!needsLayoutRange.isEmpty());
    auto layoutResult = makeUniqueRef<InlineLayoutResult>();
    auto& inlineLayoutState = layoutState();
    auto& marginState = inlineLayoutState.parentBlockLayoutState().marginState();

    auto isPartialLayout = InlineInvalidation::mayOnlyNeedPartialLayout(lineDamage);
    ASSERT(isPartialLayout || !previousLine);

    if (!isPartialLayout && (createDisplayContentForLineFromCachedContent(constraints, layoutResult.get(), mayUseSimplifiedDisplayContentBuild) || createDisplayContentForEmptyInlineContent(constraints, inlineItemList, layoutResult.get()))) {
        layoutResult->range = InlineLayoutResult::Range::Full;
        handleAfterSideMargin(marginState, layoutResult->displayContent);
        return layoutResult;
    }

    auto floatingContext = this->floatingContext();
    auto lineLogicalTop = InlineLayoutUnit { constraints.logicalTop() };
    auto previousLineEnd = std::optional<InlineItemPosition> { };
    auto leadingInlineItemPosition = needsLayoutRange.start;
    auto isFirstFormattedLineCandidate = !previousLine;

    while (true) {

        auto lineInitialRect = InlineRect { lineLogicalTop, constraints.horizontal().logicalLeft, constraints.horizontal().logicalWidth, formattingUtils().initialLineHeight(!previousLine.has_value()) };
        auto lineInput = LineInput { { leadingInlineItemPosition, needsLayoutRange.end }, lineInitialRect };
        auto lineIndex = previousLine ? (previousLine->lineIndex + 1lu) : 0lu;

        auto lineLayoutResult = lineBuilder.layoutInlineContent(lineInput, previousLine, isFirstFormattedLineCandidate);
        auto hasContentfulInFlowContent = lineLayoutResult.hasContentfulInFlowContent();

        auto canUseSimplifiedDisplayContentBuild = mayUseSimplifiedDisplayContentBuild && hasContentfulInFlowContent;
        auto lineBox = canUseSimplifiedDisplayContentBuild ? LineBoxBuilder { *this, lineLayoutResult }.buildForRootInlineBoxOnly(lineIndex) : LineBoxBuilder { *this, lineLayoutResult }.build(lineIndex);
        auto lineLogicalRect = createDisplayContentForInlineContent(lineBox, lineLayoutResult, constraints, layoutResult->displayContent, canUseSimplifiedDisplayContentBuild);
        if (!canUseSimplifiedDisplayContentBuild) {
            updateBoxGeometryForPlacedFloats(lineLayoutResult.floatContent.placedFloats);
            updateLayoutStateWithLineLayoutResult(lineLayoutResult, lineLogicalRect, floatingContext);
        }
        if (hasContentfulInFlowContent) {
            isFirstFormattedLineCandidate = false;
            inlineLayoutState.incrementLineCount();
        }

        auto lineContentEnd = lineLayoutResult.inlineItemRange.end;
        leadingInlineItemPosition = InlineFormattingUtils::leadingInlineItemPositionForNextLine(lineContentEnd, previousLineEnd, !lineLayoutResult.floatContent.hasIntrusiveFloat.isEmpty() || !lineLayoutResult.floatContent.placedFloats.isEmpty(), needsLayoutRange.end);

        auto isEndOfContent = leadingInlineItemPosition == needsLayoutRange.end && lineLayoutResult.floatContent.suspendedFloats.isEmpty();
        if (isEndOfContent) {
            layoutResult->range = !isPartialLayout ? InlineLayoutResult::Range::Full : InlineLayoutResult::Range::FullFromDamage;
            break;
        }

        if (isPartialLayout && mayExitFromPartialLayout(*lineDamage, lineIndex, layoutResult->displayContent.boxes)) {
            layoutResult->range = InlineLayoutResult::Range::PartialFromDamage;
            break;
        }

        if (formattingUtils().shouldDiscardRemainingContentInBlockDirection()) {
            resetBoxGeometriesForDiscardedContent({ leadingInlineItemPosition, needsLayoutRange.end }, lineLayoutResult.floatContent.suspendedFloats);
            layoutResult->range = !isPartialLayout ? InlineLayoutResult::Range::Full : InlineLayoutResult::Range::FullFromDamage;
            layoutResult->didDiscardContent = true;
            break;
        }

        previousLine = PreviousLine { lineIndex, lineLayoutResult.contentGeometry.trailingOverflowingContentWidth, lineLayoutResult.endsWithLineBreak() || lineLayoutResult.isBlockContent(), lineLayoutResult.hasContentfulInFlowContent(), lineLayoutResult.directionality.inlineBaseDirection, WTF::move(lineLayoutResult.floatContent.suspendedFloats) };
        previousLineEnd = lineContentEnd;
        lineLogicalTop = formattingUtils().logicalTopForNextLine(lineLayoutResult, lineLogicalRect, floatingContext);
    }
    InlineDisplayLineBuilder::addLegacyLineClampTrailingLinkBoxIfApplicable(*this, inlineLayoutState, layoutResult->displayContent);
    handleAfterSideMargin(marginState, layoutResult->displayContent);

    return layoutResult;
}

void InlineFormattingContext::layoutFloatContentOnly(const ConstraintsForInlineContent& constraints)
{
    ASSERT(!root().hasInFlowChild());

    auto& inlineContentCache = this->inlineContentCache();
    auto floatingContext = this->floatingContext();
    auto& placedFloats = layoutState().placedFloats();

    InlineItemsBuilder { inlineContentCache, root(), m_globalLayoutState.securityOrigin() }.build({ });

    for (auto& inlineItem : inlineContentCache.inlineItems().content()) {
        if (inlineItem.isFloat()) {
            CheckedRef floatBox = inlineItem.layoutBox();

            integrationUtils().layoutWithFormattingContextForBox(downcast<ElementBox>(floatBox));

            auto& floatBoxGeometry = geometryForBox(floatBox);
            auto staticPosition = LayoutPoint { constraints.horizontal().logicalLeft, constraints.logicalTop() };
            staticPosition.move(floatBoxGeometry.marginStart(), floatBoxGeometry.marginBefore());
            floatBoxGeometry.setTopLeft(staticPosition);

            auto floatBoxTopLeft = floatingContext.positionForFloat(floatBox, floatBoxGeometry, constraints.horizontal());
            floatBoxGeometry.setTopLeft(floatBoxTopLeft);
            placedFloats.add(floatingContext.makeFloatItem(floatBox, floatBoxGeometry));
            continue;
        }
        ASSERT_NOT_REACHED();
    }
}

void InlineFormattingContext::updateLayoutStateWithLineLayoutResult(const LineLayoutResult& lineLayoutResult, const InlineRect& lineLogicalRect, const FloatingContext& floatingContext)
{
    auto& layoutState = this->layoutState();
    if (auto firstLineGap = lineLayoutResult.lineGeometry.initialLetterClearGap) {
        ASSERT(!layoutState.clearGapBeforeFirstLine());
        layoutState.setClearGapBeforeFirstLine(*firstLineGap);
    }

    if (lineLayoutResult.isFirstLast.isLastLineWithInlineContent) {
        auto logicalTopCandidate = formattingUtils().logicalTopForNextLine(lineLayoutResult, lineLogicalRect, floatingContext);
        layoutState.setClearGapAfterLastLine(std::max(0.f, logicalTopCandidate - lineLogicalRect.bottom()));
    }

    lineLayoutResult.endsWithHyphen() ? layoutState.incrementSuccessiveHyphenatedLineCount() : layoutState.resetSuccessiveHyphenatedLineCount();
    layoutState.setFirstLineStartTrimForInitialLetter(lineLayoutResult.firstLineStartTrim);
}

void InlineFormattingContext::updateBoxGeometryForPlacedFloats(const LineLayoutResult::PlacedFloatList& placedFloats)
{
    for (auto& floatItem : placedFloats) {
        if (!floatItem.layoutBox()) {
            ASSERT_NOT_REACHED();
            // We should not be placing intrusive floats coming from parent BFC.
            continue;
        }
        auto& boxGeometry = geometryForBox(*floatItem.layoutBox());
        auto usedGeometry = floatItem.boxGeometry();
        boxGeometry.setTopLeft(BoxGeometry::borderBoxTopLeft(usedGeometry));
        // Adopt trimmed inline direction margin.
        boxGeometry.setHorizontalMargin(usedGeometry.horizontalMargin());
    }
}

InlineRect InlineFormattingContext::createDisplayContentForInlineContent(const LineBox& lineBox, const LineLayoutResult& lineLayoutResult, const ConstraintsForInlineContent& constraints, InlineDisplay::Content& displayContent, bool canUseSimplifiedDisplayContentBuild)
{
    if (canUseSimplifiedDisplayContentBuild) {
        auto lineBoxRect = lineBox.logicalRect();
        auto rootInlineBoxRect = lineBox.logicalRectForRootInlineBox();
        auto displayLine = InlineDisplay::Line { lineBoxRect, { lineBoxRect.top() + rootInlineBoxRect.top(), lineBoxRect.top() + rootInlineBoxRect.bottom() }, lineBox.alignmentBaseline(), rootInlineBoxRect.left(), rootInlineBoxRect.width() };

        auto numberOfDisplayBoxesFromPreviousLines = displayContent.boxes.size();
        displayContent.boxes.appendVector(InlineDisplayContentBuilder { *this, constraints, lineBox, displayLine }.buildTextOnlyContent(lineLayoutResult));
        ASSERT(displayContent.boxes.size() > numberOfDisplayBoxesFromPreviousLines);
        displayLine.setBoxCount(displayContent.boxes.size() - numberOfDisplayBoxesFromPreviousLines);
        displayContent.lines.append(displayLine);
        return lineBoxRect;
    }

    auto& inlineLayoutState = layoutState();
    auto lineClamp = inlineLayoutState.parentBlockLayoutState().lineClamp();
    // Eligible lines from nested block containers are already included (see layoutWithFormattingContextForBlockInInline).
    auto numberOfLinesWithInlineContent = inlineLayoutState.lineCountWithInlineContentIncludingNestedBlocks() + (lineLayoutResult.hasContentfulInlineContent() ? 1 : 0);
    auto numberOfVisibleLinesAllowed = lineClamp ? std::make_optional(lineClamp->maximumLines) : std::nullopt;

    auto lineIsFullyTruncatedInBlockDirection = numberOfVisibleLinesAllowed ? numberOfLinesWithInlineContent > *numberOfVisibleLinesAllowed : false;
    auto displayLine = InlineDisplayLineBuilder { *this, constraints }.build(lineLayoutResult, lineBox, lineIsFullyTruncatedInBlockDirection);
    auto boxes = InlineDisplayContentBuilder { *this, constraints, lineBox, displayLine }.build(lineLayoutResult);
    displayLine.setBoxCount(boxes.size());

    auto ellipsis = std::optional<InlineDisplay::Line::Ellipsis> { };
    // When a block line is clamped, its content gets clamped and not this line itself.
    if (!lineLayoutResult.isBlockContent()) {
        auto isLegacyLineClamp = lineClamp && lineClamp->isLegacy;
        CheckedRef styleForTruncation = root().isAnonymous() ? IntegrationUtils::firstNonAnonymousAncestorStyle(root()) : root().style();
        auto truncationPolicy = InlineFormattingUtils::lineEndingTruncationPolicy(styleForTruncation, numberOfLinesWithInlineContent, numberOfVisibleLinesAllowed, lineLayoutResult.hasContentfulInFlowContent());
        ellipsis = InlineDisplayLineBuilder::applyEllipsisIfNeeded(truncationPolicy, displayLine, boxes.mutableSpan(), isLegacyLineClamp);
        if (ellipsis) {
            displayLine.setHasEllipsis();
            auto lineHasLegacyLineClamp = isLegacyLineClamp && truncationPolicy == LineEndingTruncationPolicy::WhenContentOverflowsInBlockDirection;
            if (lineHasLegacyLineClamp)
                inlineLayoutState.setLegacyClampedLineIndex(lineBox.lineIndex());
        }
    }

    displayContent.boxes.appendVector(WTF::move(boxes));
    displayContent.lines.append(displayLine);
    if (ellipsis)
        displayContent.setEllipsisOnTrailingLine(WTF::move(*ellipsis));
    inlineLayoutState.setLineCountWithInlineContentIncludingNestedBlocks(numberOfLinesWithInlineContent);
    return InlineFormattingUtils::flipVisualRectToLogicalForWritingMode(displayContent.lines.last().lineBoxRect(), root().writingMode());
}

void InlineFormattingContext::resetBoxGeometriesForDiscardedContent(const InlineItemRange& discardedRange, const LineLayoutResult::SuspendedFloatList& suspendedFloats)
{
    if (discardedRange.isEmpty() && suspendedFloats.isEmpty())
        return;

    auto& inlineItemList = inlineContentCache().inlineItems().content();
    for (auto index = discardedRange.startIndex(); index < discardedRange.endIndex(); ++index) {
        auto& inlineItem = inlineItemList[index];
        auto hasBoxGeometry = inlineItem.isAtomicInlineBox() || inlineItem.isFloat() || inlineItem.isHardLineBreak() || inlineItem.isInlineBoxStart() || inlineItem.isOutOfFlow();
        if (!hasBoxGeometry)
            continue;
        geometryForBox(inlineItem.layoutBox()).reset();
    }

    for (CheckedPtr floatBox : suspendedFloats)
        geometryForBox(*floatBox).reset();
}

bool InlineFormattingContext::createDisplayContentForLineFromCachedContent(const ConstraintsForInlineContent& constraints, InlineLayoutResult& layoutResult, bool mayUseSimplifiedDisplayContentBuild)
{
    auto& inlineContentCache = this->inlineContentCache();

    if (!inlineContentCache.maximumIntrinsicWidthLineContent() || !inlineContentCache.maximumContentSize())
        return false;

    auto horizontalAvailableSpace = constraints.horizontal().logicalWidth;
    if (*inlineContentCache.maximumContentSize() > horizontalAvailableSpace) {
        inlineContentCache.clearMaximumIntrinsicWidthLineContent();
        return false;
    }
    if (!layoutState().placedFloats().isEmpty()) {
        inlineContentCache.clearMaximumIntrinsicWidthLineContent();
        return false;
    }

    auto& lineContent = *inlineContentCache.maximumIntrinsicWidthLineContent();
    auto restoreTrimmedTrailingWhitespaceIfApplicable = [&]() -> std::optional<bool> {
        // Special 'line-break: after-white-space' behavior where min/max width trims trailing whitespace, while
        // layout should preserve _overflowing_ trailing whitespace.
        if (root().style().lineBreak() != LineBreak::AfterWhiteSpace || !lineContent.trimmedTrailingWhitespaceWidth)
            return { };
        if (ceiledLayoutUnit(lineContent.contentGeometry.logicalWidth) + LayoutUnit::epsilon() <= horizontalAvailableSpace)
            return { };
        if (!Line::restoreTrimmedTrailingWhitespace(lineContent.trimmedTrailingWhitespaceWidth, lineContent.runs, lineContent.inlineItemRange, inlineContentCache.inlineItems().content())) {
            ASSERT_NOT_REACHED();
            return false;
        }
        lineContent.contentGeometry.logicalWidth += lineContent.trimmedTrailingWhitespaceWidth;
        lineContent.contentGeometry.logicalRightIncludingNegativeMargin += lineContent.trimmedTrailingWhitespaceWidth;
        lineContent.trimmedTrailingWhitespaceWidth = { };
        return true;
    };
    auto successfullyTrimmed = restoreTrimmedTrailingWhitespaceIfApplicable();
    if (successfullyTrimmed && !*successfullyTrimmed) {
        inlineContentCache.clearMaximumIntrinsicWidthLineContent();
        return false;
    }

    auto logicalTopLeft = InlineLayoutPoint { constraints.horizontal().logicalLeft, constraints.logicalTop() };
    lineContent.lineGeometry.logicalTopLeft = logicalTopLeft;
    lineContent.lineGeometry.logicalWidth = constraints.horizontal().logicalWidth;
    lineContent.lineGeometry.initialLogicalTopLeft = logicalTopLeft;
    lineContent.contentGeometry.logicalLeft = InlineFormattingUtils::horizontalAlignmentOffset(root().style(), lineContent.contentGeometry.logicalWidth, lineContent.lineGeometry.logicalWidth, lineContent.hangingContent.logicalWidth, true);

    auto canUseSimplifiedDisplayContentBuild = mayUseSimplifiedDisplayContentBuild && lineContent.hasContentfulInFlowContent();
    auto lineBox = canUseSimplifiedDisplayContentBuild ? LineBoxBuilder { *this, lineContent }.buildForRootInlineBoxOnly({ }) : LineBoxBuilder { *this, lineContent }.build({ });
    createDisplayContentForInlineContent(lineBox, lineContent, constraints, layoutResult.displayContent, canUseSimplifiedDisplayContentBuild);
    return true;
}

bool InlineFormattingContext::createDisplayContentForEmptyInlineContent(const ConstraintsForInlineContent& constraints, const InlineItemList& inlineItemList, InlineLayoutResult& layoutResult)
{
    if (!isEmptyInlineContent(inlineItemList))
        return false;

    auto emptyLineBreakingResult =  LineLayoutResult { };
    auto logicalTopLeft = InlineLayoutPoint { constraints.horizontal().logicalLeft, constraints.logicalTop() };
    emptyLineBreakingResult.lineGeometry = { logicalTopLeft, { constraints.horizontal().logicalWidth }, logicalTopLeft };
    auto lineBox = LineBoxBuilder { *this, emptyLineBreakingResult }.build({ });
    createDisplayContentForInlineContent(lineBox, emptyLineBreakingResult, constraints, layoutResult.displayContent);
    return true;
}

void InlineFormattingContext::initializeInlineLayoutState(const LayoutState& globalLayoutState)
{
    auto& inlineLayoutState = layoutState();

    if (auto limitLinesValue = root().style().hyphenateLimitLines().tryValue())
        inlineLayoutState.setHyphenationLimitLines(limitLinesValue->value);
    // FIXME: Remove when IFC takes care of running layout on inline-blocks.
    inlineLayoutState.setShouldNotSynthesizeInlineBlockBaseline();
    if (globalLayoutState.inStandardsMode())
        inlineLayoutState.setInStandardsMode();
    if (globalLayoutState.isTextShapingAcrossInlineBoxesEnabled())
        inlineLayoutState.setShouldShapeTextAcrossInlineBoxes();
}

#if ASSERT_ENABLED
static inline bool isOkToAccessBoxGeometry(const Box& layoutBox, const ElementBox& rootBlockContainer, std::optional<InlineFormattingContext::EscapeReason> escapeReason)
{
    if (escapeReason == InlineFormattingContext::EscapeReason::InkOverflowNeedsInitialContiningBlockForStrokeWidth && is<InitialContainingBlock>(layoutBox))
        return true;
    // This is the non-escape case of accessing a box's geometry information within the same formatting context when computing static position for out-of-flow boxes.
    if (layoutBox.isOutOfFlowPositioned())
        return true;
    auto containingBlock = [&]() -> const Box* {
        for (auto* ancestor = &layoutBox.parent(); !is<InitialContainingBlock>(*ancestor); ancestor = &ancestor->parent()) {
            if (ancestor->isContainingBlockForInFlow())
                return ancestor;
        }
        return nullptr;
    };
    // This is the non-escape case of accessing a box's geometry information within the same formatting context.
    return containingBlock() == &rootBlockContainer;
};
#endif

const BoxGeometry& InlineFormattingContext::geometryForBox(const Box& layoutBox, std::optional<EscapeReason> escapeReason) const
{
    ASSERT_UNUSED(escapeReason, isOkToAccessBoxGeometry(layoutBox, root(), escapeReason));
    return m_globalLayoutState.geometryForBox(layoutBox);
}

BoxGeometry& InlineFormattingContext::geometryForBox(const Box& layoutBox)
{
    ASSERT(isOkToAccessBoxGeometry(layoutBox, root(), { }));
    return m_globalLayoutState.ensureGeometryForBox(layoutBox);
}

void InlineFormattingContext::rebuildInlineItemListIfNeeded(InlineDamage* lineDamage)
{
    auto& inlineContentCache = this->inlineContentCache();
    auto inlineItemListNeedsUpdate = inlineContentCache.inlineItems().isEmpty() || (lineDamage && lineDamage->isInlineItemListDirty());
    if (!inlineItemListNeedsUpdate)
        return;

    auto startPositionForInlineItemsBuilding = [&]() -> InlineItemPosition {
        if (!lineDamage) {
            ASSERT(inlineContentCache.inlineItems().isEmpty());
            return { };
        }
        if (auto startPosition = lineDamage->layoutStartPosition()) {
            if (lineDamage->reasons().contains(InlineDamage::Reason::Pagination)) {
                // FIXME: We don't support partial rebuild with certain types of content. Let's just re-collect inline items.
                return { };
            }
            if (inlineContentCache.inlineItems().hasWhiteSpaceTrim()) {
                // white-space-trim discards collapsible white space based on cross-box adjacency that a
                // partial rebuild starting mid-content cannot resolve. Re-collect all inline items.
                return { };
            }
            return startPosition->inlineItemPosition;
        }
        // Unsupported damage. Need to run full build/layout.
        return { };
    };
    InlineItemsBuilder { inlineContentCache, root(), m_globalLayoutState.securityOrigin() }.build(startPositionForInlineItemsBuilding());
    if (lineDamage)
        lineDamage->setInlineItemListClean();
    inlineContentCache.clearMaximumIntrinsicWidthLineContent();
}

}
}

```

## InlineLineBuilder.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineLineBuilder.cpp). Path: `Source/WebCore/layout/formattingContexts/inline/InlineLineBuilder.cpp`. Source bytes: 111440; source lines: 1979; SHA-256: `cd0801fe1454de6fefc0d59ead751dfaa0918d95295e3967eb1e09ab27eed7e4`.

```cpp
/*
 * Copyright (C) 2019-2023 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "InlineLineBuilder.h"

#include "ComplexTextController.h"
#include "InlineContentAligner.h"
#include "InlineFormattingContext.h"
#include "InlineFormattingUtils.h"
#include "InlineQuirks.h"
#include "LayoutBox.h"
#include "LayoutBoxGeometry.h"
#include "LayoutShape.h"
#include "RubyFormattingContext.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "StyleComputedStyle+InitialInlines.h"
#include "StyleWebKitLineBoxContain.h"
#include "TextUtil.h"
#include "UnicodeBidi.h"
#include <ranges>
#include <wtf/unicode/CharacterNames.h>

namespace WebCore {
namespace Layout {

struct LineContent {
    WTF_DEPRECATED_MAKE_STRUCT_FAST_ALLOCATED(LineContent);

    InlineItemRange range;
    size_t partialTrailingContentLength { 0 };
    std::optional<InlineLayoutUnit> overflowLogicalWidth { };
    HashMap<const Box*, InlineLayoutUnit> rubyBaseAlignmentOffsetList { };
    InlineLayoutUnit rubyAnnotationOffset { 0.f };
    enum class LineBreakReason : uint8_t {
        ForcedLineBreakByBlockContent,
        Other
    };
    LineBreakReason lineBreakReason { LineBreakReason::Other };
};

static bool isContentfulOrHasDecoration(const InlineItem& inlineItem, const InlineFormattingContext& formattingContext)
{
    if (inlineItem.isFloat() || inlineItem.isOutOfFlow())
        return false;
    if (auto* inlineTextItem = dynamicDowncast<InlineTextItem>(inlineItem)) {
        auto wouldProduceEmptyRun = inlineTextItem->isFullyTrimmable() || inlineTextItem->isEmpty() || inlineTextItem->isWordSeparator() || inlineTextItem->isZeroWidthSpaceSeparator() || inlineTextItem->isQuirkNonBreakingSpace();
        return !wouldProduceEmptyRun;
    }

    if (inlineItem.isInlineBoxStart())
        return !!formattingContext.geometryForBox(inlineItem.layoutBox()).marginBorderAndPaddingStart();
    if (inlineItem.isInlineBoxEnd())
        return !!formattingContext.geometryForBox(inlineItem.layoutBox()).marginBorderAndPaddingEnd();
    return inlineItem.isAtomicInlineBox() || inlineItem.isLineBreak();
}

static inline StringBuilder toString(const Line::RunList& runs)
{
    // FIXME: We could try to reuse the content builder in InlineItemsBuilder if this turns out to be a perf bottleneck.
    StringBuilder lineContentBuilder;
    for (auto& run : runs) {
        if (!run.isText())
            continue;
        auto& textContent = run.textContent();
        lineContentBuilder.append(StringView(downcast<InlineTextBox>(run.layoutBox()).content()).substring(textContent.start, textContent.length));
    }
    return lineContentBuilder;
}

static inline Vector<int32_t> computedVisualOrder(const Line::RunList& lineRuns, Vector<int32_t>& visualOrderList)
{
    Vector<UBiDiLevel> runLevels;
    runLevels.reserveInitialCapacity(lineRuns.size());

    Vector<size_t> runIndexOffsetMap;
    runIndexOffsetMap.reserveInitialCapacity(lineRuns.size());
    size_t numberOfOpaqueRuns = 0;
    for (size_t i = 0, accumulatedOffset = 0; i < lineRuns.size(); ++i) {
        if (lineRuns[i].bidiLevel() == InlineItem::opaqueBidiLevel) {
            ++accumulatedOffset;
            ++numberOfOpaqueRuns;
            continue;
        }

        // bidiLevels are required to be less than the MAX + 1, otherwise
        // ubidi_reorderVisual will silently fail.
        if (lineRuns[i].bidiLevel() > UBIDI_MAX_EXPLICIT_LEVEL + 1) {
            ASSERT(lineRuns[i].bidiLevel() == UBIDI_DEFAULT_LTR);
            continue;
        }

        runLevels.append(lineRuns[i].bidiLevel());
        runIndexOffsetMap.append(accumulatedOffset);
    }

    auto forceBiDiOnOpaqueLine = [&] {
        if (lineRuns.isEmpty() || numberOfOpaqueRuns != lineRuns.size())
            return;
        // When an RTL line has only opaque items (e.g. [spanning inline box start][inline box end] on <span><div></div></span>)
        // we need to set the bidi level on the spanning inline box as if it was contentful to initiate bidi processing (mainly just RTL direction align).
        if (!lineRuns.first().isLineSpanningInlineBoxStart())
            return;
        runLevels.append(lineRuns.first().layoutBox().parent().writingMode().isBidiLTR() ? UBIDI_LTR : UBIDI_RTL);
        runIndexOffsetMap.append(0);
    };
    forceBiDiOnOpaqueLine();

    visualOrderList.resizeToFit(runLevels.size());
    ubidi_reorderVisual(runLevels.span().data(), runLevels.size(), visualOrderList.mutableSpan().data());
    if (numberOfOpaqueRuns) {
        ASSERT(visualOrderList.size() == runIndexOffsetMap.size());
        for (size_t i = 0; i < runIndexOffsetMap.size(); ++i)
            visualOrderList[i] += runIndexOffsetMap[visualOrderList[i]];
    }
    return visualOrderList;
}

static bool NODELETE hasTrailingSoftWrapOpportunity(size_t softWrapOpportunityIndex, size_t layoutRangeEnd, std::span<const InlineItem> inlineItemList)
{
    if (!softWrapOpportunityIndex || softWrapOpportunityIndex == layoutRangeEnd) {
        // This candidate inline content ends because the entire content ends and not because there's a soft wrap opportunity.
        return false;
    }
    // See https://www.w3.org/TR/css-text-3/#line-break-details
    auto& trailingInlineItem = inlineItemList[softWrapOpportunityIndex - 1];
    if (trailingInlineItem.isFloat()) {
        // While we stop at floats, they are not considered real soft wrap opportunities.
        return false;
    }
    if (trailingInlineItem.isAtomicInlineBox() || trailingInlineItem.isLineBreak() || trailingInlineItem.isWordBreakOpportunity() || trailingInlineItem.isInlineBoxEnd()) {
        // For Web-compatibility there is a soft wrap opportunity before and after each replaced element or other atomic inline.
        return true;
    }
    if (auto* inlineTextItem = dynamicDowncast<InlineTextItem>(trailingInlineItem)) {
        if (inlineTextItem->isWhitespace())
            return true;
        // Now in case of non-whitespace trailing content, we need to check if the actual soft wrap opportunity belongs to the next set.
        // e.g. "this_is_the_trailing_run<span> <-but_this_space_here_is_the_soft_wrap_opportunity"
        // When there's an inline box start(<span>)/end(</span>) between the trailing and the (next)leading run, while we break before the inline box start (<span>)
        // the actual soft wrap position is after the inline box start (<span>) but in terms of line breaking continuity the inline box start (<span>) and the whitespace run belong together.
        RELEASE_ASSERT(layoutRangeEnd <= inlineItemList.size());
        for (auto index = softWrapOpportunityIndex; index < layoutRangeEnd; ++index) {
            if (inlineItemList[index].isInlineBoxStart() || inlineItemList[index].isInlineBoxEnd() || inlineItemList[index].isOutOfFlow())
                continue;
            // FIXME: Check if [non-whitespace][inline-box][no-whitespace] content has rules about it.
            // For now let's say the soft wrap position belongs to the next set of runs when [non-whitespace][inline-box][whitespace], [non-whitespace][inline-box][box] etc.
            auto inlineItemListTextItem = dynamicDowncast<InlineTextItem>(inlineItemList[index]);
            return inlineItemListTextItem && !inlineItemListTextItem->isWhitespace();
        }
        return true;
    }
    if (trailingInlineItem.isInlineBoxStart()) {
        // This is a special case when the inline box's first child is a float box.
        return false;
    }
    if (trailingInlineItem.isOutOfFlow()) {
        for (auto index = softWrapOpportunityIndex; index--;) {
            if (!inlineItemList[index].isOutOfFlow())
                return hasTrailingSoftWrapOpportunity(index + 1, layoutRangeEnd, inlineItemList);
        }
        ASSERT(inlineItemList[softWrapOpportunityIndex].isFloat() || inlineItemList[softWrapOpportunityIndex].isBlock());
        return false;
    }
    if (trailingInlineItem.isBlock())
        return true;

    ASSERT_NOT_REACHED();
    return true;
};

static TextDirection inlineBaseDirectionForLineContent(const Line::RunList& runs, const Style::ComputedStyle& rootStyle, std::optional<PreviousLine> previousLine)
{
    ASSERT(!runs.isEmpty());
    auto shouldUseBlockDirection = rootStyle.unicodeBidi() != UnicodeBidi::Plaintext;
    if (shouldUseBlockDirection)
        return rootStyle.writingMode().bidiDirection();
    // A previous line that ended a paragraph, with a line break (<br> or preserved \n) or with a block level box on it,
    // introduces a new unicode paragraph, so this line takes its direction from its own content.
    if (previousLine && !previousLine->endsParagraph)
        return previousLine->inlineBaseDirection;
    return TextUtil::directionForTextContent(toString(runs));
}

struct LineCandidate {
    WTF_DEPRECATED_MAKE_STRUCT_FAST_ALLOCATED(LineCandidate);

    void reset();

    struct InlineContent {
        const InlineContentBreaker::ContinuousContent& NODELETE continuousContent() const { return m_continuousContent; }
        InlineContentBreaker::ContinuousContent& NODELETE continuousContent() { return m_continuousContent; }
        const InlineItem* NODELETE trailingLineBreak() const { return m_trailingLineBreak; }
        const InlineItem* NODELETE trailingWordBreakOpportunity() const { return m_trailingWordBreakOpportunity; }

        void appendInlineItem(const InlineItem&, const Style::ComputedStyle&, InlineLayoutUnit logicalWidth, InlineLayoutUnit textSpacingAdjustment = 0);
        void reset();
        bool NODELETE isEmpty() const { return m_continuousContent.runs().isEmpty() && !trailingWordBreakOpportunity() && !trailingLineBreak(); }

        void NODELETE setHasTrailingSoftWrapOpportunity(bool hasTrailingSoftWrapOpportunity) { m_hasTrailingSoftWrapOpportunity = hasTrailingSoftWrapOpportunity; }
        bool NODELETE hasTrailingSoftWrapOpportunity() const { return m_hasTrailingSoftWrapOpportunity; }

        void NODELETE setTrailingSoftHyphenWidth(InlineLayoutUnit hyphenWidth) { m_continuousContent.setTrailingSoftHyphenWidth(hyphenWidth); }

        void NODELETE setHangingContentWidth(InlineLayoutUnit logicalWidth) { m_continuousContent.setHangingContentWidth(logicalWidth); }

        void NODELETE setHasTrailingClonedDecoration(bool hasClonedDecoration) { m_hasTrailingClonedDecoration = hasClonedDecoration; }
        bool NODELETE hasTrailingClonedDecoration() const { return m_hasTrailingClonedDecoration; }

        void NODELETE setMinimumRequiredWidth(InlineLayoutUnit minimumRequiredWidth) { m_continuousContent.setMinimumRequiredWidth(minimumRequiredWidth); }

        std::optional<size_t> NODELETE firstTextRunIndex() const { return m_firstTextRunIndex; }
        std::optional<size_t> NODELETE lastTextRunIndex() const { return m_lastTextRunIndex; }

        bool NODELETE isShapingCandidateByContent() const { return m_hasTextContentSpanningBoxes; }

    private:
        InlineContentBreaker::ContinuousContent m_continuousContent;
        const InlineItem* m_trailingLineBreak { nullptr };
        const InlineItem* m_trailingWordBreakOpportunity { nullptr };
        bool m_hasTrailingClonedDecoration { false };
        bool m_hasTrailingSoftWrapOpportunity { false };
        std::optional<size_t> m_firstTextRunIndex { };
        std::optional<size_t> m_lastTextRunIndex { };
        std::optional<size_t> m_lastInlineBoxIndex { };
        bool m_hasTextContentSpanningBoxes { false };
    };

    // Candidate content is a collection of inline content or a float box.
    InlineContent inlineContent;
    const InlineItem* floatItem { nullptr };
    const InlineItem* blockItem { nullptr };
};

inline void LineCandidate::InlineContent::appendInlineItem(const InlineItem& inlineItem, const Style::ComputedStyle& style, InlineLayoutUnit logicalWidth, InlineLayoutUnit textSpacingAdjustment)
{
    if (inlineItem.isAtomicInlineBox() || inlineItem.isOutOfFlow())
        return m_continuousContent.append(inlineItem, style, logicalWidth, textSpacingAdjustment);

    if (inlineItem.isInlineBoxStartOrEnd()) {
        auto numberOfRuns = m_continuousContent.runs().size();
        m_hasTextContentSpanningBoxes = m_hasTextContentSpanningBoxes || (m_lastTextRunIndex && m_lastTextRunIndex == numberOfRuns - 1);
        m_lastInlineBoxIndex = numberOfRuns;
        m_continuousContent.append(inlineItem, style, logicalWidth, textSpacingAdjustment);
        return;
    }

    if (auto* inlineTextItem = dynamicDowncast<InlineTextItem>(inlineItem)) {
        auto numberOfRuns = m_continuousContent.runs().size();
        m_firstTextRunIndex = m_firstTextRunIndex.value_or(numberOfRuns);
        m_lastTextRunIndex = numberOfRuns;
        m_hasTextContentSpanningBoxes = m_hasTextContentSpanningBoxes || (m_lastInlineBoxIndex && m_lastInlineBoxIndex == numberOfRuns - 1);
        return m_continuousContent.appendTextContent(*inlineTextItem, style, logicalWidth);
    }

    if (inlineItem.isLineBreak()) {
        m_trailingLineBreak = &inlineItem;
        return;
    }

    if (inlineItem.isWordBreakOpportunity()) {
        m_trailingWordBreakOpportunity = &inlineItem;
        return;
    }

    ASSERT_NOT_REACHED();
}

inline void LineCandidate::InlineContent::reset()
{
    m_continuousContent.reset();
    m_trailingLineBreak = { };
    m_trailingWordBreakOpportunity = { };
    m_hasTrailingClonedDecoration = { };
    m_hasTrailingSoftWrapOpportunity = { };
    m_firstTextRunIndex = { };
    m_lastTextRunIndex = { };
    m_lastInlineBoxIndex = { };
    m_hasTextContentSpanningBoxes = { };
}

inline void LineCandidate::reset()
{
    floatItem = { };
    blockItem = { };
    inlineContent.reset();
}

LineBuilder::LineBuilder(InlineFormattingContext& inlineFormattingContext, HorizontalConstraints rootHorizontalConstraints, const InlineItemList& inlineItemList, TextSpacingContext textSpacingContext)
    : AbstractLineBuilder(inlineFormattingContext, inlineFormattingContext.root(), rootHorizontalConstraints, inlineItemList)
    , m_floatingContext(inlineFormattingContext.floatingContext())
    , m_textSpacingContext(WTF::move(textSpacingContext))
{
}

LineLayoutResult LineBuilder::layoutInlineContent(const LineInput& lineInput, const std::optional<PreviousLine>& previousLine, bool isFirstFormattedLineCandidate)
{
    initialize(lineInput.initialLogicalRect, lineInput.needsLayoutRange, previousLine, isFirstFormattedLineCandidate);
    auto lineContent = placeInlineAndFloatContent(lineInput.needsLayoutRange);
    auto result = m_line.close();
    auto inlineContentEnding = result.isContentful ? InlineFormattingUtils::inlineContentEnding(result) : std::nullopt;

    auto updateMarginStateIfNeeded = [&] {
        // When a line places contentful inline content, the block margin from previous content stays
        // before this line; nothing is left for the next line, so reset marginState. Lines without
        // contentful inline content leave marginState alone for the next line to apply.
        if (!inlineContentEnding)
            return;
        auto& marginState = blockLayoutState().marginState();
        marginState.resetMarginValues();
        if (marginState.atBeforeSideOfBlock)
            marginState.resetBeforeSideOfBlock();
    };
    updateMarginStateIfNeeded();

    if (isInIntrinsicWidthMode()) {
        return { lineContent->range
            , WTF::move(result.runs)
            , { WTF::move(m_placedFloats), WTF::move(m_suspendedFloats), { } }
            , { { }, result.contentLogicalWidth, { }, lineContent->overflowLogicalWidth }
            , { m_lineLogicalRect.topLeft(), m_lineLogicalRect.width(), m_lineInitialLogicalRect.topLeft() }
            , { }
            , { }
            , { isFirstFormattedLineCandidate && inlineContentEnding.has_value() ? IsFirstFormattedLine::Yes : IsFirstFormattedLine::No, { } }
            , { }
            , inlineContentEnding
            , { }
            , { }
            , { }
            , { }
        };
    }

    auto isLastInlineContent = isLastLineWithInlineContent(lineContent, lineInput.needsLayoutRange.endIndex(), result.runs);
    // Lines with nothing but content trailing out-of-flow boxes should also be considered last line for alignment
    // e.g. <div style="text-align-last: center">last line<br><div style="display: inline; position: absolute"></div></div>
    // Both the inline content ('last line') and the trailing out-of-flow box are supposed to be center aligned.
    auto shouldTreatAsLastLine = isLastInlineContent || lineContent->range.endIndex() == lineInput.needsLayoutRange.endIndex();
    auto inlineBaseDirection = !result.runs.isEmpty() ? inlineBaseDirectionForLineContent(result.runs, rootStyle(), m_previousLine) : TextDirection::LTR;
    auto lineEndsWithForcedLineBreak = lineContent->lineBreakReason == LineContent::LineBreakReason::ForcedLineBreakByBlockContent || Line::hasTrailingForcedLineBreak(result.runs);
    auto isLastLineOrLineEndsWithForcedLineBreak = shouldTreatAsLastLine || lineEndsWithForcedLineBreak;
    auto contentLogicalLeft = !result.runs.isEmpty() ? InlineFormattingUtils::horizontalAlignmentOffset(rootStyle(), result.contentLogicalRight, m_lineLogicalRect.width(), result.hangingTrailingContentWidth, isLastLineOrLineEndsWithForcedLineBreak, inlineBaseDirection) : 0.f;
    Vector<int32_t> visualOrderList;
    if (result.contentNeedsBidiReordering)
        computedVisualOrder(result.runs, visualOrderList);

    return { lineContent->range
        , WTF::move(result.runs)
        , { WTF::move(m_placedFloats), WTF::move(m_suspendedFloats), m_lineIsConstrainedByFloat }
        , { contentLogicalLeft, result.contentLogicalWidth, contentLogicalLeft + result.contentLogicalRight, lineContent->overflowLogicalWidth }
        , { m_lineLogicalRect.topLeft(), m_lineLogicalRect.width(), m_lineInitialLogicalRect.topLeft(), m_initialIntrusiveFloatsWidth, m_initialLetterClearGap }
        , { !result.isHangingTrailingContentWhitespace, result.hangingTrailingContentWidth, result.hangablePunctuationStartWidth }
        , { WTF::move(visualOrderList), inlineBaseDirection }
        , { isFirstFormattedLineCandidate && inlineContentEnding.has_value() ? IsFirstFormattedLine::Yes : IsFirstFormattedLine::No, isLastInlineContent }
        , { WTF::move(lineContent->rubyBaseAlignmentOffsetList), lineContent->rubyAnnotationOffset }
        , inlineContentEnding
        , result.nonSpanningInlineLevelBoxCount
        , { }
        , { }
        , lineContent->range.isEmpty() ? std::make_optional(m_lineLogicalRect.top() + m_candidateContentMaximumHeight) : std::nullopt
    };
}

void LineBuilder::createLineSpanningInlineBoxes(const InlineItemRange& needsLayoutRange)
{
    auto isRootLayoutBox = [&](auto& elementBox) {
        return &elementBox == &root();
    };
    if (needsLayoutRange.isEmpty())
        return;
    // An inline box may not necessarily start on the current line:
    // <span>first line<br>second line<span>with some more embedding<br> forth line</span></span>
    // We need to make sure that there's an [InlineBoxStart] for every inline box that's present on the current line.
    // We only have to do it on the first run as any subsequent inline content is either at the same/higher nesting level.
    auto& firstInlineItem = m_inlineItemList[needsLayoutRange.startIndex()];
    // If the parent is the formatting root, we can stop here. This is root inline box content, there's no nesting inline box from the previous line(s)
    // unless the inline box closing is forced over to the current line.
    // e.g.
    // <span>normally the inline box closing forms a continuous content</span>
    // <span>unless it's forced to the next line<br></span>
    auto& firstLayoutBox = firstInlineItem.layoutBox();
    auto hasLeadingInlineBoxEnd = firstInlineItem.isInlineBoxEnd();

    if (!hasLeadingInlineBoxEnd) {
        if (isRootLayoutBox(firstLayoutBox.parent()))
            return;

        if (isRootLayoutBox(firstLayoutBox.parent().parent())) {
            // In many cases the entire content is wrapped inside a single inline box.
            // e.g. <div><span>wall of text with<br>single, line spanning inline box...</span></div>
            ASSERT(firstLayoutBox.parent().isInlineBox());
            m_lineSpanningInlineBoxes.append({ firstLayoutBox.parent(), InlineItem::Type::InlineBoxStart, InlineItem::opaqueBidiLevel });
            return;
        }
    }

    Vector<const Box*, 2> spanningLayoutBoxList;
    if (hasLeadingInlineBoxEnd)
        spanningLayoutBoxList.append(&firstLayoutBox);

    auto* ancestor = &firstInlineItem.layoutBox().parent();
    while (!isRootLayoutBox(*ancestor)) {
        spanningLayoutBoxList.append(ancestor);
        ancestor = &ancestor->parent();
    }
    // Let's treat these spanning inline items as opaque bidi content. They should not change the bidi levels on adjacent content.
    for (auto* spanningInlineBox : spanningLayoutBoxList | std::views::reverse)
        m_lineSpanningInlineBoxes.append({ *spanningInlineBox, InlineItem::Type::InlineBoxStart, InlineItem::opaqueBidiLevel });
}

void LineBuilder::initialize(const InlineRect& initialLineLogicalRect, const InlineItemRange& needsLayoutRange, const std::optional<PreviousLine>& previousLine, bool isFirstFormattedLineCandidate)
{
    ASSERT(!needsLayoutRange.isEmpty() || (previousLine && !previousLine->suspendedFloats.isEmpty()));
    reset();

    m_previousLine = previousLine;
    m_isFirstFormattedLineCandidate = isFirstFormattedLineCandidate;
    m_placedFloats.clear();
    m_suspendedFloats.clear();
    m_lineSpanningInlineBoxes.clear();
    m_overflowingLogicalWidth = { };
    m_partialLeadingTextItem = { };
    m_initialLetterClearGap = { };
    m_candidateContentMaximumHeight = { };
    inlineContentBreaker().setHyphenationDisabled(layoutState().isHyphenationDisabled());

    createLineSpanningInlineBoxes(needsLayoutRange);
    m_line.initialize(m_lineSpanningInlineBoxes, isFirstFormattedLineCandidate);

    m_lineInitialLogicalRect = initialLineLogicalRect;
    auto previousLineEndsParagraph = previousLine ? std::make_optional(previousLine->endsParagraph ? InlineFormattingUtils::PreviousLineEndsParagraph::Yes : InlineFormattingUtils::PreviousLineEndsParagraph::No) : std::nullopt;
    m_lineMarginStart = formattingContext().formattingUtils().computedTextIndent(isInIntrinsicWidthMode() ? InlineFormattingUtils::IsIntrinsicWidthMode::Yes : InlineFormattingUtils::IsIntrinsicWidthMode::No, isFirstFormattedLineCandidate ? IsFirstFormattedLine::Yes : IsFirstFormattedLine::No, previousLineEndsParagraph, initialLineLogicalRect.width());

    auto computeLineLogicalRect = [&] {
        // Apply the block margin coming from previous content (e.g. margin-bottom of a preceding block-in-inline
        // child) before narrowing for floats so floatAvoidingRect runs at the line's actual y position.
        // e.g.
        // <span><div style="margin-bottom: 100px;"></div>text</span>
        // where "text" sits at the block's content bottom plus 100px.
        auto lineLogicalRect = initialLineLogicalRect;
        auto& marginState = blockLayoutState().marginState();
        // If the previous line had no contentful in-flow content (float-only or empty), the same pending margin
        // is already baked into its top via this advance. Re-applying would double-count.
        // A margin still at the before side of the block is only skipped when it can actually collapse out
        // through that edge. When the root has a border or padding it cannot, so a self collapsing block on a
        // previous line has already committed the margin and the line has to move down by it.
        auto marginCollapsesThroughBeforeSide = marginState.atBeforeSideOfBlock && marginState.canCollapseMarginBeforeWithChildren;
        if (!marginCollapsesThroughBeforeSide && previousLine && previousLine->hasContentfulInFlowContent)
            lineLogicalRect.moveVertically(marginState.margin());
        auto constraints = floatAvoidingRect(lineLogicalRect, { });
        m_lineIsConstrainedByFloat = constraints.constrainedSideSet;
        return constraints.logicalRect;
    };
    m_lineLogicalRect = computeLineLogicalRect();
    // This is by how much intrusive floats (coming from parent/sibling FCs) initially offset the line.
    m_initialIntrusiveFloatsWidth = m_lineLogicalRect.left() - initialLineLogicalRect.left();
    m_lineLogicalRect.moveHorizontally(m_lineMarginStart);
    // While negative margins normally don't expand the available space, intrinsic width contribution computation gets confused by negative text-indent
    // (shrink the space needed for the content) which we have to balance it here.
    m_lineLogicalRect.expandHorizontally(-m_lineMarginStart);
    m_lineContentEdgeOffset = m_lineLogicalRect.left() - initialLineLogicalRect.left();

    auto initializeLeadingContentFromOverflow = [&] {
        if (!previousLine || !needsLayoutRange.start.offset)
            return;
        auto overflowingInlineItemPosition = needsLayoutRange.start;
        if (auto* overflowingInlineTextItem = dynamicDowncast<InlineTextItem>(m_inlineItemList[overflowingInlineItemPosition.index])) {
            ASSERT(overflowingInlineItemPosition.offset < overflowingInlineTextItem->length());
            auto overflowingLength = overflowingInlineTextItem->length() - overflowingInlineItemPosition.offset;
            if (overflowingLength) {
                // Turn previous line's overflow content into the next line's leading content.
                // "sp[<-line break->]lit_content" -> break position: 2 -> leading partial content length: 11.
                m_partialLeadingTextItem = overflowingInlineTextItem->right(overflowingLength, previousLine->trailingOverflowingContentWidth);
                return;
            }
        }
        m_overflowingLogicalWidth = previousLine->trailingOverflowingContentWidth;
    };
    initializeLeadingContentFromOverflow();
}

UniqueRef<LineContent> LineBuilder::placeInlineAndFloatContent(const InlineItemRange& needsLayoutRange)
{
    size_t resumedFloatCount = 0;
    auto layoutPreviouslySuspendedFloats = [&] {
        if (!m_previousLine)
            return true;
        // FIXME: Note that placedInlineItemCount is not incremented here as these floats are already accounted for (at previous line)
        // as LineContent only takes one range -meaning that inline layout may continue while float layout is being suspended
        // and the placed InlineItem range ends at the last inline item placed on the current line.
        for (size_t index = 0; index < m_previousLine->suspendedFloats.size(); ++index) {
            auto& suspendedFloat = *m_previousLine->suspendedFloats[index];
            auto isPlaced = tryPlacingFloatBox(suspendedFloat, !index ? MayOverConstrainLine::OnlyWhenFirstFloatOnLine : MayOverConstrainLine::No);
            if (!isPlaced) {
                // Can't place more floats here. We'll try to place these floats on subsequent lines.
                for (; index < m_previousLine->suspendedFloats.size(); ++index)
                    m_suspendedFloats.append(m_previousLine->suspendedFloats[index]);
                return false;
            }
            ++resumedFloatCount;
        }
        m_previousLine->suspendedFloats.clear();
        return true;
    };

    auto lineContent = makeUniqueRef<LineContent>();

    if (!layoutPreviouslySuspendedFloats()) {
        // Couldn't even manage to place all suspended floats from previous line(s). -which also means we can't fit any inline content at this vertical position.
        lineContent->range = { needsLayoutRange.start, needsLayoutRange.start };
        m_candidateContentMaximumHeight = m_lineLogicalRect.height();
        return lineContent;
    }

    size_t placedInlineItemCount = 0;

    auto layoutInlineAndFloatContent = [&] {
        auto lineCandidate = makeUniqueRef<LineCandidate>();

        auto currentItemIndex = needsLayoutRange.startIndex();
        while (currentItemIndex < needsLayoutRange.endIndex()) {
            // 1. Collect the set of runs that we can commit to the line as one entity e.g. <span>text_and_span_start_span_end</span>.
            // 2. Apply floats and shrink the available horizontal space e.g. <span>intru_<div style="float: left"></div>sive_float</span>.
            // 3. Check if the content fits the line and commit the content accordingly (full, partial or not commit at all).
            // 4. Return if we are at the end of the line either by not being able to fit more content or because of an explicit line break.
            auto candidateStartEndIndex = std::pair<size_t, size_t> { currentItemIndex, formattingContext().formattingUtils().nextWrapOpportunity(currentItemIndex, needsLayoutRange, m_inlineItemList) };
            candidateContentForLine(lineCandidate, candidateStartEndIndex, needsLayoutRange, m_line.contentLogicalRight());
            // Now check if we can put this content on the current line.
            if (auto* floatItem = lineCandidate->floatItem) {
                ASSERT(lineCandidate->inlineContent.isEmpty());
                if (!tryPlacingFloatBox(floatItem->layoutBox(), m_line.runs().isEmpty() ? MayOverConstrainLine::Yes : MayOverConstrainLine::No)) {
                    // This float overconstrains the line (it simply means shrinking the line box by the float would cause inline content overflow.)
                    // At this point we suspend float layout but continue with inline layout.
                    // Such suspended float will be placed at the next available vertical positon when this line "closes".
                    m_suspendedFloats.append(&floatItem->layoutBox());
                }
                ++placedInlineItemCount;
            } else if (auto* blockItem = lineCandidate->blockItem) {
                // We need to break whenever we come across a block level block to ensure it's the only item on the line.
                // This is unlike hard line break as in case of 'text<br>', hard line break stays on the current line.
                if (placedInlineItemCount) {
                    lineContent->lineBreakReason = LineContent::LineBreakReason::ForcedLineBreakByBlockContent;
                    return;
                }

                ASSERT(lineCandidate->inlineContent.isEmpty());
                handleBlockContent(*blockItem);
                ++placedInlineItemCount;
                // It's always end of line before/after a block level box.
                return;
            } else {
                auto result = handleInlineContent(needsLayoutRange, lineCandidate);
                auto isEndOfLine = result.isEndOfLine == InlineContentBreaker::IsEndOfLine::Yes;
                if (!result.committedCount.isRevert) {
                    placedInlineItemCount += result.committedCount.value;
                    auto& inlineContent = lineCandidate->inlineContent;
                    auto inlineContentIsFullyPlaced = inlineContent.continuousContent().runs().size() == result.committedCount.value && !result.partialTrailingContentLength;
                    if (inlineContentIsFullyPlaced) {
                        if (auto* wordBreakOpportunity = inlineContent.trailingWordBreakOpportunity()) {
                            // <wbr> needs to be on the line as an empty run so that we can construct an inline box and compute basic geometry.
                            ++placedInlineItemCount;
                            m_line.appendWordBreakOpportunity(*wordBreakOpportunity, wordBreakOpportunity->style());
                        }
                        if (inlineContent.trailingLineBreak()) {
                            // Fully placed (or empty) content followed by a line break means "end of line".
                            // FIXME: This will put the line break box at the end of the line while in case of some inline boxes, the line break
                            // could very well be at an earlier position. This has no visual implications at this point though (only geometry correctness on the line break box).
                            // e.g. <span style="border-right: 10px solid green">text<br></span> where the <br>'s horizontal position is before the right border and not after.
                            auto& trailingLineBreak = *inlineContent.trailingLineBreak();
                            m_line.appendLineBreak(trailingLineBreak, trailingLineBreak.style());
                            if (trailingLineBreak.bidiLevel() != UBIDI_DEFAULT_LTR)
                                m_line.setContentNeedsBidiReordering();
                            ++placedInlineItemCount;
                            isEndOfLine = true;
                        }
                    }
                } else {
                    auto dropSuspendedFloatsFromRevertedContent = [&] {
                        for (auto index = needsLayoutRange.startIndex() + result.committedCount.value; index < needsLayoutRange.startIndex() + placedInlineItemCount; ++index) {
                            auto& inlineItem = m_inlineItemList[index];
                            if (inlineItem.isFloat())
                                m_suspendedFloats.removeFirst(&inlineItem.layoutBox());
                        }
                    };
                    dropSuspendedFloatsFromRevertedContent();
                    placedInlineItemCount = result.committedCount.value;
                }

                if (isEndOfLine) {
                    lineContent->partialTrailingContentLength = result.partialTrailingContentLength;
                    lineContent->overflowLogicalWidth = result.overflowLogicalWidth;
                    return;
                }
            }
            currentItemIndex = needsLayoutRange.startIndex() + placedInlineItemCount;
        }
        // Looks like we've run out of content.
        ASSERT(placedInlineItemCount || resumedFloatCount);
    };
    layoutInlineAndFloatContent();

    auto computePlacedInlineItemRange = [&] {
        lineContent->range = { needsLayoutRange.start, needsLayoutRange.start };

        if (!placedInlineItemCount)
            return;

        // Layout range already includes "suspended" floats from previous line(s). See layoutPreviouslySuspendedFloats above for details.
        ASSERT(m_placedFloats.size() >= resumedFloatCount);
        auto onlyFloatContentPlaced = placedInlineItemCount == m_placedFloats.size() - resumedFloatCount;
        if (onlyFloatContentPlaced || !lineContent->partialTrailingContentLength) {
            lineContent->range.end = { needsLayoutRange.startIndex() + placedInlineItemCount, { } };
            return;
        }

        auto trailingInlineItemIndex = needsLayoutRange.startIndex() + placedInlineItemCount - 1;
        auto overflowingInlineTextItemLength = downcast<InlineTextItem>(m_inlineItemList[trailingInlineItemIndex]).length();
        ASSERT(lineContent->partialTrailingContentLength && lineContent->partialTrailingContentLength < overflowingInlineTextItemLength);
        lineContent->range.end = { trailingInlineItemIndex, overflowingInlineTextItemLength - lineContent->partialTrailingContentLength };
    };
    computePlacedInlineItemRange();

    ASSERT(lineContent->range.endIndex() <= needsLayoutRange.endIndex());

    auto handleLineEnding = [&] {
        auto isLastInlineContent = isLastLineWithInlineContent(lineContent, needsLayoutRange.endIndex(), m_line.runs());
        auto horizontalAvailableSpace = m_lineLogicalRect.width();
        auto& rootStyle = this->rootStyle();

        auto handleTrailingContent = [&] {
            auto& quirks = formattingContext().quirks();
            auto lineHasOverflow = [&] {
                return horizontalAvailableSpace < m_line.contentLogicalWidth() && m_line.hasContent();
            };
            auto isLineBreakAfterWhitespace = [&] {
                return rootStyle.lineBreak() == LineBreak::AfterWhiteSpace && intrinsicWidthMode() != IntrinsicWidthMode::Minimum && (!isLastInlineContent || lineHasOverflow());
            };
            m_line.handleTrailingTrimmableContent(isLineBreakAfterWhitespace() ? Line::TrailingContentAction::Preserve : Line::TrailingContentAction::Remove);
            if (quirks.trailingNonBreakingSpaceNeedsAdjustment(isInIntrinsicWidthMode(), lineHasOverflow()))
                m_line.handleOverflowingNonBreakingSpace(isLineBreakAfterWhitespace() ? Line::TrailingContentAction::Preserve : Line::TrailingContentAction::Remove, m_line.contentLogicalWidth() - horizontalAvailableSpace);

            m_line.handleTrailingHangingContent(intrinsicWidthMode(), horizontalAvailableSpace, isLastInlineContent);

            auto mayNeedOutOfFlowOverflowTrimming = !isInIntrinsicWidthMode() && lineHasOverflow() && !lineContent->partialTrailingContentLength && TextUtil::isWrappingAllowed(rootStyle);
            if (mayNeedOutOfFlowOverflowTrimming) {
                // Overflowing out-of-flow boxes should wrap the to subsequent lines just like any other in-flow content.
                // However since we take a shortcut by not considering out-of-flow content as inflow but instead treating it as an opaque box with zero width and no
                // soft wrap opportunity, any overflowing out-of-flow content would pile up as trailing content.
                // Alternatively we could initiate a two pass layout first with out-of-flow content treated as true inflow and a second without them.
                ASSERT(!lineContent->range.end.offset);
                if (auto* lastRemovedTrailingBox = m_line.removeOverflowingOutOfFlowContent()) {
                    auto lineEndIndex = [&] {
                        for (auto index = lineContent->range.start.index; index < lineContent->range.end.index; ++index) {
                            if (&m_inlineItemList[index].layoutBox() == lastRemovedTrailingBox)
                                return index;
                        }
                        ASSERT_NOT_REACHED();
                        return lineContent->range.end.index;
                    };
                    lineContent->range.end.index = lineEndIndex();
                }
            }
        };
        handleTrailingContent();

        // On each line, reset the embedding level of any sequence of whitespace characters at the end of the line
        // to the paragraph embedding level
        m_line.resetBidiLevelForTrailingWhitespace(rootStyle.writingMode().isBidiLTR() ? UBIDI_LTR : UBIDI_RTL);

        if (m_line.hasContent()) {
            auto applyRunBasedAlignmentIfApplicable = [&] {
                if (isInIntrinsicWidthMode())
                    return;

                auto spaceToDistribute = horizontalAvailableSpace - m_line.contentLogicalWidth() + (m_line.isHangingTrailingContentWhitespace() ? m_line.hangingTrailingContentWidth() : 0.f);
                if (root().isRubyAnnotationBox() && rootStyle.textAlign() == Style::ComputedStyle::initialTextAlign()) {
                    lineContent->rubyAnnotationOffset = RubyFormattingContext::applyRubyAlignOnAnnotationBox(m_line, spaceToDistribute, formattingContext());
                    m_line.inflateContentLogicalWidth(spaceToDistribute);
                    m_line.adjustContentRightWithRubyAlign(2 * lineContent->rubyAnnotationOffset);
                    return;
                }
                // Text is justified according to the method specified by the text-justify property,
                // in order to exactly fill the line box. Unless otherwise specified by text-align-last,
                // the last line before a forced break or the end of the block is start-aligned.
                // A block level box inside an inline box starts a block, which is a forced line break too.
                auto lineEndsWithForcedLineBreak = lineContent->lineBreakReason == LineContent::LineBreakReason::ForcedLineBreakByBlockContent || Line::hasTrailingForcedLineBreak(m_line.runs());
                auto hasTextAlignJustify = (isLastInlineContent || lineEndsWithForcedLineBreak) ? rootStyle.textAlignLast() == Style::TextAlignLast::Justify : rootStyle.textAlign() == Style::TextAlign::Justify;
                if (hasTextAlignJustify) {
                    // Detach trailing hanging whitespace into its own run so the text
                    // shaper applies expansion only to inter-word spaces in the content run.
                    m_line.detachHangingTrailingWhitespaceIfApplicable();
                    auto additionalSpaceForAlignedContent = InlineContentAligner::applyTextAlignJustify(m_line.runs(), spaceToDistribute, m_line.hangingTrailingWhitespaceLength());
                    m_line.inflateContentLogicalWidth(additionalSpaceForAlignedContent);
                }
                if (m_line.hasRubyContent())
                    lineContent->rubyBaseAlignmentOffsetList = RubyFormattingContext::applyRubyAlign(m_line, formattingContext());
            };
            applyRunBasedAlignmentIfApplicable();
        }
    };
    handleLineEnding();

    return lineContent;
}

InlineLayoutUnit LineBuilder::leadingPunctuationWidthForLineCandidate(const LineCandidate& lineCandidate) const
{
    auto& inlineContent = lineCandidate.inlineContent;
    auto firstTextRunIndex = inlineContent.firstTextRunIndex();
    if (!firstTextRunIndex)
        return { };

    auto isFirstLineFirstContent = isFirstFormattedLineCandidate() && !m_line.hasContent();
    if (!isFirstLineFirstContent)
        return { };

    auto& runs = inlineContent.continuousContent().runs();
    auto* inlineTextItem = dynamicDowncast<InlineTextItem>(runs[*firstTextRunIndex].inlineItem);
    if (!inlineTextItem) {
        ASSERT_NOT_REACHED();
        return { };
    }
    auto& style = isFirstFormattedLineCandidate() ? inlineTextItem->firstLineStyle() : inlineTextItem->style();
    if (!TextUtil::hasHangablePunctuationStart(*inlineTextItem, style))
        return { };

    if (*firstTextRunIndex) {
        // The text content is not the first in the candidate list. However it may be the first contentful one.
        for (size_t index = *firstTextRunIndex; index--;) {
            if (isContentfulOrHasDecoration(runs[index].inlineItem, formattingContext()))
                return { };
        }
    }
    // This candidate leading content may have hanging punctuation start.
    return TextUtil::hangablePunctuationStartWidth(*inlineTextItem, style);
}

InlineLayoutUnit LineBuilder::trailingPunctuationOrStopOrCommaWidthForLineCandidate(const LineCandidate& lineCandidate, size_t startIndexAfterCandidateContent,  size_t layoutRangeEnd) const
{
    auto& inlineContent = lineCandidate.inlineContent;
    auto lastTextRunIndex = inlineContent.lastTextRunIndex();
    if (!lastTextRunIndex)
        return { };

    auto& runs = inlineContent.continuousContent().runs();
    auto* inlineTextItem = dynamicDowncast<InlineTextItem>(runs[*lastTextRunIndex].inlineItem);
    if (!inlineTextItem) {
        ASSERT_NOT_REACHED();
        return { };
    }

    auto& style = isFirstFormattedLineCandidate() ? inlineTextItem->firstLineStyle() : inlineTextItem->style();

    if (TextUtil::hasHangableStopOrCommaEnd(*inlineTextItem, style)) {
        // Stop or comma does apply to all lines not just the last formatted one.
        return TextUtil::hangableStopOrCommaEndWidth(*inlineTextItem, style);
    }

    if (TextUtil::hasHangablePunctuationEnd(*inlineTextItem, style)) {
        // FIXME: If this turns out to be problematic (finding out if this is the last formatted line that is), we
        // may have to fallback to a post-process setup, where after finishing laying out the content, we go back and re-layout
        // the last (2?) line(s) when there's trailing hanging punctuation.
        // For now let's probe the content all the way to layoutRangeEnd.
        for (auto index = startIndexAfterCandidateContent; index < layoutRangeEnd; ++index) {
            if (isContentfulOrHasDecoration(m_inlineItemList[index], formattingContext()))
                return { };
        }
        return TextUtil::hangablePunctuationEndWidth(*inlineTextItem, style);
    }

    return { };
}

Vector<std::pair<size_t, size_t>> LineBuilder::collectShapeRanges(const LineCandidate& lineCandidate) const
{
    // Normally candidate content is inline items between 2 soft wraping opportunities e.g.
    // <div>some text<span>more text</span></div>
    // where candidate contents are as follows: [some] [ ] [text<span>more] [ ] [text</span>]
    // However when white space is preserved and/or no wrapping is allowed the entire content is
    // one candidate content with all sorts of inline level content.

    // Let's find shaping ranges by filtering out content that are not relevant to shaping,
    // followed by processing this compressed list of [content , break, joint ] where
    // 'content' means shapable content (text)
    // 'break' means shape breaking gap (e.g. whitespace between 2 words)
    // 'keep' means box that keeps adjacent inline items in the same shaping context ("text<span>more" <- inline box start)
    auto& runs = lineCandidate.inlineContent.continuousContent().runs();

    auto isFirstFormattedLineCandidate = this->isFirstFormattedLineCandidate();
    enum class ShapingType : uint8_t { Content, Break, Keep };
    struct Content {
        ShapingType type { ShapingType::Break };
        size_t index { 0 };
    };
    Vector<Content> contentList;
    for (size_t index = 0; index < runs.size(); ++index) {
        auto& inlineItem = runs[index].inlineItem;

        auto type = std::optional<ShapingType> { };
        switch (inlineItem.type()) {
        case InlineItem::Type::Text:
            type = downcast<InlineTextItem>(inlineItem).isWhitespace() ? ShapingType::Break : ShapingType::Content;
            break;
        case InlineItem::Type::AtomicInlineBox:
            type = ShapingType::Break;
            break;
        case InlineItem::Type::InlineBoxStart: {
            [[fallthrough]];
        case InlineItem::Type::InlineBoxEnd:
            auto& boxGeometry = formattingContext().geometryForBox(inlineItem.layoutBox());
            auto& style = isFirstFormattedLineCandidate ? inlineItem.firstLineStyle() : inlineItem.style();
            auto hasDecoration = [&] {
                // Note that this depends on the content being RTL (inline-box-end vs. start decoration matching visual order -visual matching).
                auto shouldCheckLogicalStart = style.writingMode().bidiDirection() == TextDirection::LTR ? inlineItem.type() == InlineItem::Type::InlineBoxEnd : inlineItem.type() == InlineItem::Type::InlineBoxStart;
                return shouldCheckLogicalStart ? boxGeometry.marginStart() || boxGeometry.borderStart() || boxGeometry.paddingStart() : boxGeometry.marginEnd() || boxGeometry.borderEnd() || boxGeometry.paddingEnd();
            };
            auto hasBidiIsolation = isIsolated(style.unicodeBidi());
            type = hasDecoration() || hasBidiIsolation ? ShapingType::Break : ShapingType::Keep;
            break;
        }
        case InlineItem::Type::HardLineBreak:
        case InlineItem::Type::SoftLineBreak:
        case InlineItem::Type::WordBreakOpportunity:
        case InlineItem::Type::Float:
        case InlineItem::Type::OutOfFlow:
        case InlineItem::Type::Block:
            break;
        default:
            ASSERT_NOT_REACHED();
        }

        auto shouldIgnore = [&] {
            if (!type)
                return true;
            if (*type == ShapingType::Content)
                return false;
            return contentList.isEmpty() || *type == contentList.last().type;
        };
        if (!shouldIgnore())
            contentList.append(Content { *type, index });
    }

    // Trailing non-content entries should just be ignored.
    while (!contentList.isEmpty()) {
        if (contentList.last().type == ShapingType::Content)
            break;
        contentList.removeLast();
    }

    if (contentList.isEmpty())
        return { };

    ASSERT(contentList.first().type == ShapingType::Content && contentList.last().type == ShapingType::Content);
    Vector<std::pair<size_t, size_t>> ranges;

    CheckedPtr lastFontCascade = &rootStyle().fontCascade();
    auto leadingContentRunIndex = std::optional<size_t> { };
    auto trailingContentRunIndex = std::optional<size_t> { };
    auto hasBoundaryBetween = false;

    auto resetCandidateRange = [&] {
        leadingContentRunIndex = { };
        trailingContentRunIndex = { };
        hasBoundaryBetween = false;
    };
    auto commitIfHasContentAndReset = [&] {
        if (leadingContentRunIndex && trailingContentRunIndex && hasBoundaryBetween)
            ranges.append({ *leadingContentRunIndex, *trailingContentRunIndex });
        resetCandidateRange();
    };

    for (auto entry : contentList) {
        switch (entry.type) {
        case ShapingType::Break:
            commitIfHasContentAndReset();
            break;
        case ShapingType::Keep:
            if (hasBoundaryBetween) {
                // Nested inline boxes e.g. <span>content<span>more<span>and some more
                ASSERT(leadingContentRunIndex);
                break;
            }
            if (leadingContentRunIndex)
                hasBoundaryBetween = true;
            break;
        case ShapingType::Content: {
            auto& inlineTextItem = downcast<InlineTextItem>(runs[entry.index].inlineItem);
            auto& styleToUse = isFirstFormattedLineCandidate ? inlineTextItem.firstLineStyle() : inlineTextItem.style();
            auto& inlineTextBox = inlineTextItem.inlineTextBox();
            auto isEligibleText = !inlineTextBox.canUseSimpleFontCodePath() && !inlineTextBox.isCombined() && inlineTextItem.direction() == TextDirection::RTL;

            if (!leadingContentRunIndex) {
                if (isEligibleText)
                    leadingContentRunIndex = entry.index;
                lastFontCascade = &styleToUse.fontCascade();
            } else if (hasBoundaryBetween) {
                auto hasMatchingFontCascade = *lastFontCascade.get() == styleToUse.fontCascade();
                if (isEligibleText && hasMatchingFontCascade)
                    trailingContentRunIndex = entry.index;
                else {
                    commitIfHasContentAndReset();
                    if (isEligibleText)
                        leadingContentRunIndex = entry.index;
                    lastFontCascade = &styleToUse.fontCascade();
                }
            } else if (!isEligibleText)
                resetCandidateRange();
            break;
        }
        default:
            ASSERT_NOT_REACHED();
        }
    }
    commitIfHasContentAndReset();
    return ranges;
}

void LineBuilder::applyShapingOnRunRange(LineCandidate& lineCandidate, std::pair<size_t, size_t> range) const
{
    auto& inlineContent = lineCandidate.inlineContent;
    auto& runs = inlineContent.continuousContent().runs();
    if (range.first >= range.second || range.first >= runs.size() || range.second >= runs.size()) {
        ASSERT_NOT_REACHED();
        return;
    }
    runs[range.first].shapingBoundary = InlineContentBreaker::ContinuousContent::Run::ShapingBoundary::Start;
    runs[range.second].shapingBoundary = InlineContentBreaker::ContinuousContent::Run::ShapingBoundary::End;

    StringBuilder textContent;
    for (auto index = range.first; index <= range.second; ++index) {
        if (auto* inlineTextItem = dynamicDowncast<InlineTextItem>(runs[index].inlineItem))
            textContent.append(inlineTextItem->content());
    }

    ASSERT(!textContent.isEmpty());
    auto characterScanForCodePath = true;
    auto& style = isFirstFormattedLineCandidate() ? runs[range.first].inlineItem.firstLineStyle() : runs[range.first].inlineItem.style();
    auto textRun = TextRun { textContent, m_lineLogicalRect.left(), { }, ExpansionBehavior::defaultBehavior(), TextDirection::RTL, style.rtlOrdering() == Order::Visual, characterScanForCodePath };
    auto glyphAdvances = ComplexTextController::glyphAdvancesForTextRun(style.fontCascade(), textRun);

    if (glyphAdvances.size() != textRun.length()) {
        ASSERT_NOT_REACHED();
        return;
    }

    size_t glyphIndex = 0;
    auto shapedContentWidth = InlineLayoutUnit { };
    for (auto index = range.first; index <= range.second; ++index) {
        auto& run = runs[index];
        auto* inlineTextItem = dynamicDowncast<InlineTextItem>(run.inlineItem);
        if (!inlineTextItem) {
            ASSERT(run.inlineItem.isInlineBoxStartOrEnd());
            continue;
        }
        auto runWidth = InlineLayoutUnit { };
        for (size_t i = 0; i < inlineTextItem->length(); ++i) {
            runWidth += std::max(0.f, glyphAdvances[glyphIndex]);
            ++glyphIndex;
        }
        run.adjustContentWidth(runWidth);
        shapedContentWidth += runWidth;
    }
    inlineContent.continuousContent().adjustLogicalWidth(shapedContentWidth);
    inlineContent.continuousContent().setHasShapedContent();
}

void LineBuilder::applyShapingIfNeeded(LineCandidate& lineCandidate)
{
    if (!layoutState().shouldShapeTextAcrossInlineBoxes())
        return;

    if (!lineCandidate.inlineContent.isShapingCandidateByContent())
        return;

    for (auto range : collectShapeRanges(lineCandidate))
        applyShapingOnRunRange(lineCandidate, range);
}

void LineBuilder::shapePartialLineCandidate(LineCandidate& lineCandidate, size_t trailingRunIndex) const
{
    auto& inlineContent = lineCandidate.inlineContent;
    auto& runs = inlineContent.continuousContent().runs();

    if (trailingRunIndex >= runs.size()) {
        ASSERT_NOT_REACHED();
        return;
    }

    // Find the shaping boundary end to see if we need to reshape the candidate text.
    for (auto index = trailingRunIndex + 1; index < runs.size(); ++index) {
        auto shapingBoundary = runs[index].shapingBoundary;
        if (!shapingBoundary)
            continue;
        if (*shapingBoundary == InlineContentBreaker::ContinuousContent::Run::ShapingBoundary::Start) {
            // Trailing content is a new shaping boundary, no need to reshape leading content.
            return;
        }
        ASSERT(*shapingBoundary == InlineContentBreaker::ContinuousContent::Run::ShapingBoundary::End);
        auto endPosition = std::optional<size_t> { };
        for (auto i = trailingRunIndex + 1; i--;) {
            auto& run = runs[i];
            if (!endPosition && run.inlineItem.isText())
                endPosition = i;

            auto shapingBoundary = run.shapingBoundary;
            if (shapingBoundary && *shapingBoundary == InlineContentBreaker::ContinuousContent::Run::ShapingBoundary::Start) {
                if (!endPosition) {
                    ASSERT_NOT_REACHED();
                    return;
                }
                if (*endPosition == i) {
                    // No shaping is needed when content does not cross multiple boxes.
                    run.shapingBoundary = { };
                    if (i < trailingRunIndex)
                        run.adjustContentWidth(formattingContext().formattingUtils().inlineItemWidth(run.inlineItem, { }, isFirstFormattedLineCandidate()));
                    return;
                }

                applyShapingOnRunRange(lineCandidate, { i, *endPosition });
                return;
            }
        }
        // We should always find a start when there's an end.
        ASSERT_NOT_REACHED();
    }
}

void LineBuilder::candidateContentForLine(LineCandidate& lineCandidate, std::pair<size_t, size_t> startEndIndex, const InlineItemRange& layoutRange, InlineLayoutUnit currentLogicalRight, SkipFloats skipFloats)
{
    ASSERT(startEndIndex.first < layoutRange.endIndex());
    ASSERT(startEndIndex.second <= layoutRange.endIndex());

    auto isFirstFormattedLineCandidate = this->isFirstFormattedLineCandidate();
    lineCandidate.reset();

    auto isLeadingPartiaContent = startEndIndex.first == layoutRange.startIndex() && m_partialLeadingTextItem;
    if (isLeadingPartiaContent) {
        ASSERT(!m_overflowingLogicalWidth);
        // Handle leading partial content first (overflowing text from the previous line).
        auto itemWidth = formattingContext().formattingUtils().inlineItemWidth(*m_partialLeadingTextItem, m_lineContentEdgeOffset + currentLogicalRight, isFirstFormattedLineCandidate);
        lineCandidate.inlineContent.appendInlineItem(*m_partialLeadingTextItem, m_partialLeadingTextItem->style(), itemWidth);
        currentLogicalRight += itemWidth;
        ++startEndIndex.first;
    }

    auto trailingSoftHyphenInlineTextItemIndex = std::optional<size_t> { };
    auto textSpacingAdjustment = InlineLayoutUnit { };
    auto contentHasInlineItemsWithDecorationClone = !m_line.inlineBoxListWithClonedDecorationEnd().isEmpty();

    for (auto index = startEndIndex.first; index < startEndIndex.second; ++index) {
        auto& inlineItem = m_inlineItemList[index];
        auto& style = isFirstFormattedLineCandidate ? inlineItem.firstLineStyle() : inlineItem.style();
        if (inlineItem.isInlineBoxStart()) {
            if (auto inlineBoxBoundaryTextSpacing = m_textSpacingContext.inlineBoxBoundaryTextSpacings.find(index); inlineBoxBoundaryTextSpacing != m_textSpacingContext.inlineBoxBoundaryTextSpacings.end())
                textSpacingAdjustment = inlineBoxBoundaryTextSpacing->value;
        }

        auto needsLayout = inlineItem.isFloat() || inlineItem.isAtomicInlineBox() || (inlineItem.isOutOfFlow() && inlineItem.layoutBox().isRubyAnnotationBox());
        if (needsLayout) {
            // FIXME: Intrinsic width mode should call into the intrinsic width codepath. Currently we only get here when box has fixed width (meaning no need to run intrinsic width on the box).
            if (!isInIntrinsicWidthMode())
                formattingContext().integrationUtils().layoutWithFormattingContextForBox(downcast<ElementBox>(inlineItem.layoutBox()));
        }

        if (inlineItem.isFloat()) {
            if (skipFloats == SkipFloats::Yes)
                continue;
            lineCandidate.floatItem = &inlineItem;
            // This is a soft wrap opportunity, must be the only item in the list.
            ASSERT(startEndIndex.first + 1 == startEndIndex.second);
            continue;
        }
        if (auto* inlineTextItem = dynamicDowncast<InlineTextItem>(inlineItem)) {
            auto logicalWidth = m_overflowingLogicalWidth ? *std::exchange(m_overflowingLogicalWidth, std::nullopt) : formattingContext().formattingUtils().inlineItemWidth(*inlineTextItem, m_lineContentEdgeOffset + currentLogicalRight, isFirstFormattedLineCandidate);
            if (!currentLogicalRight) {
                if (auto trimmableSpacing = m_textSpacingContext.trimmableTextSpacings.find(index); trimmableSpacing != m_textSpacingContext.trimmableTextSpacings.end())
                    logicalWidth -= trimmableSpacing->value;
            }
            lineCandidate.inlineContent.appendInlineItem(*inlineTextItem, style, logicalWidth);
            // Word spacing does not make the run longer, but it produces an offset instead. See Line::appendTextContent as well.
            currentLogicalRight += logicalWidth + (inlineTextItem->isWordSeparator() ? style.fontCascade().wordSpacing() : 0.f);
            trailingSoftHyphenInlineTextItemIndex = inlineTextItem->hasTrailingSoftHyphen() ? std::make_optional(index) : std::nullopt;
            continue;
        }
        if (inlineItem.isInlineBoxStartOrEnd()) {
            auto& layoutBox = inlineItem.layoutBox();
            auto logicalWidth = formattingContext().formattingUtils().inlineItemWidth(inlineItem, currentLogicalRight, isFirstFormattedLineCandidate);
            if (layoutBox.isRubyBase()) {
                if (inlineItem.isInlineBoxStart()) {
                    // There should only be one ruby base per/annotation candidate content as we allow line breaking between bases unless some special characters between ruby bases prevent us from doing so (see RubyFormattingContext::canBreakAtCharacter)
                    if (auto marginBoxWidth = RubyFormattingContext::annotationBoxLogicalWidth(layoutBox, formattingContext()); marginBoxWidth > 0) {
                        auto& inlineContent = lineCandidate.inlineContent;
                        inlineContent.setMinimumRequiredWidth(inlineContent.continuousContent().minimumRequiredWidth().value_or(InlineLayoutUnit { }) + marginBoxWidth);
                    }
                } else
                    logicalWidth += RubyFormattingContext::baseEndAdditionalLogicalWidth(layoutBox, m_line.runs(), lineCandidate.inlineContent.continuousContent().runs(), formattingContext());
            }

            contentHasInlineItemsWithDecorationClone |= inlineItem.isInlineBoxStart() && style.boxDecorationBreak() == BoxDecorationBreak::Clone;
            lineCandidate.inlineContent.appendInlineItem(inlineItem, style, logicalWidth, textSpacingAdjustment);
            currentLogicalRight += logicalWidth;
            continue;
        }
        if (inlineItem.isAtomicInlineBox()) {
            auto logicalWidth = formattingContext().formattingUtils().inlineItemWidth(inlineItem, currentLogicalRight, isFirstFormattedLineCandidate);
            // FIXME: While the line breaking related properties for atomic level boxes do not depend on the line index (first line style) it'd be great to figure out the correct style to pass in.
            lineCandidate.inlineContent.appendInlineItem(inlineItem, inlineItem.layoutBox().parent().style(), logicalWidth);
            currentLogicalRight += logicalWidth;
            continue;
        }
        if (inlineItem.isLineBreak() || inlineItem.isWordBreakOpportunity()) {
#if ASSERT_ENABLED
            // Since both <br> and <wbr> are explicit word break opportunities they have to be trailing items in this candidate run list unless they are embedded in inline boxes.
            // e.g. <span><wbr></span>
            for (auto i = index + 1; i < startEndIndex.second; ++i)
                ASSERT(m_inlineItemList[i].isInlineBoxEnd() || m_inlineItemList[i].isOutOfFlow());
#endif
            lineCandidate.inlineContent.appendInlineItem(inlineItem, style, { });
            continue;
        }
        if (inlineItem.isOutOfFlow()) {
            lineCandidate.inlineContent.appendInlineItem(inlineItem, style, { });
            continue;
        }
        if (inlineItem.isBlock()) {
            // Blocks must be the only items in the list.
            lineCandidate.blockItem = &inlineItem;
            ASSERT(startEndIndex.first + 1 == startEndIndex.second);
            continue;
        }
        ASSERT_NOT_REACHED();
    }

    if (lineCandidate.floatItem || lineCandidate.blockItem)
        return;

    auto setupTrailingContent = [&] {
        lineCandidate.inlineContent.setHasTrailingClonedDecoration(contentHasInlineItemsWithDecorationClone);

        auto setLeadingAndTrailingHangingPunctuation = [&] {
            auto& inlineContent = lineCandidate.inlineContent;
            auto hangingContentWidth = inlineContent.continuousContent().hangingContentWidth();
            // Do not even try to check for trailing punctuation when the candidate content already has whitespace type of hanging content.
            if (!hangingContentWidth)
                hangingContentWidth += trailingPunctuationOrStopOrCommaWidthForLineCandidate(lineCandidate, startEndIndex.second, layoutRange.endIndex());
            hangingContentWidth += leadingPunctuationWidthForLineCandidate(lineCandidate);
            if (hangingContentWidth)
                lineCandidate.inlineContent.setHangingContentWidth(hangingContentWidth);
        };
        setLeadingAndTrailingHangingPunctuation();

        auto setTrailingSoftHyphenWidth = [&] {
            if (!trailingSoftHyphenInlineTextItemIndex)
                return;
            for (auto index = *trailingSoftHyphenInlineTextItemIndex; index < startEndIndex.second; ++index) {
                if (!is<InlineTextItem>(m_inlineItemList[index]))
                    return;
            }
            auto& trailingInlineTextItem = m_inlineItemList[*trailingSoftHyphenInlineTextItemIndex];
            auto& style = isFirstFormattedLineCandidate ? trailingInlineTextItem.firstLineStyle() : trailingInlineTextItem.style();
            lineCandidate.inlineContent.setTrailingSoftHyphenWidth(TextUtil::hyphenWidth(style));
        };
        setTrailingSoftHyphenWidth();
        lineCandidate.inlineContent.setHasTrailingSoftWrapOpportunity(hasTrailingSoftWrapOpportunity(startEndIndex.second, layoutRange.endIndex(), m_inlineItemList));
    };
    setupTrailingContent();
    applyShapingIfNeeded(lineCandidate);
}

static inline InlineLayoutUnit NODELETE availableWidth(const Line& line, InlineLayoutUnit lineWidth, std::optional<IntrinsicWidthMode> intrinsicWidthMode)
{
#if USE_FLOAT_AS_INLINE_LAYOUT_UNIT
    // 1. Intrinsic width contribution computation sums up floats while line breaker subtracts them.
    // 2. Available space is inherently a LayoutUnit based value (coming from block/flex etc layout) and it is the result of a floored float.
    // These can all lead to epsilon-scale differences.
    if (!intrinsicWidthMode || *intrinsicWidthMode == IntrinsicWidthMode::Maximum)
        lineWidth += LayoutUnit::epsilon();
#endif
    auto availableWidth = lineWidth - line.contentLogicalRight();
    return std::isnan(availableWidth) ? maxInlineLayoutUnit() : availableWidth;
}

LineBuilder::RectAndFloatConstraints LineBuilder::floatAvoidingRect(const InlineRect& logicalRect, InlineLayoutUnit lineMarginStart) const
{
    auto constraints = [&]() -> LineBuilder::RectAndFloatConstraints {
        if (isInIntrinsicWidthMode() || floatingContext().isEmpty())
            return { logicalRect, { } };

        auto constraints = formattingContext().formattingUtils().floatConstraintsForLine(logicalRect.top(), logicalRect.height(), floatingContext());
        if (!constraints.start && !constraints.end)
            return { logicalRect, { } };

        auto constrainedSideSet = OptionSet<UsedFloat> { };
        // text-indent acts as (start)margin on the line. When looking for intrusive floats we need to check against the line's _margin_ box.
        auto marginBoxRect = InlineRect { logicalRect.top(), logicalRect.left() - lineMarginStart, logicalRect.width() + lineMarginStart, logicalRect.height() };

        if (constraints.start && constraints.start->x > marginBoxRect.left()) {
            marginBoxRect.shiftLeftTo(constraints.start->x);
            constrainedSideSet.add(UsedFloat::Left);
        }
        if (constraints.end && constraints.end->x < marginBoxRect.right()) {
            marginBoxRect.setRight(std::max<InlineLayoutUnit>(marginBoxRect.left(), constraints.end->x));
            constrainedSideSet.add(UsedFloat::Right);
        }

        auto lineLogicalRect = InlineRect { marginBoxRect.top(), marginBoxRect.left() + lineMarginStart, marginBoxRect.width() - lineMarginStart, marginBoxRect.height() };
        return { lineLogicalRect, constrainedSideSet };
    }();

    if (auto adjustedRect = formattingContext().quirks().adjustedRectForLineGridLineAlign(constraints.logicalRect))
        constraints.logicalRect = *adjustedRect;

    return constraints;
}

LineBuilder::RectAndFloatConstraints LineBuilder::adjustedLineRectWithCandidateInlineContent(const LineCandidate& lineCandidate) const
{
    // Check if the candidate content would stretch the line and whether additional floats are getting in the way.
    auto& inlineContent = lineCandidate.inlineContent;
    if (isInIntrinsicWidthMode())
        return { m_lineLogicalRect };
    // FIXME: Use InlineFormattingUtils::inlineLevelBoxAffectsLineBox instead.
    auto candidateContentHeight = InlineLayoutUnit { };
    auto lineBoxContain = rootStyle().lineBoxContain();
    for (auto& run : inlineContent.continuousContent().runs()) {
        auto& inlineItem = run.inlineItem;
        if (inlineItem.isText()) {
            auto& styleToUse = isFirstFormattedLineCandidate() ? inlineItem.firstLineStyle() : inlineItem.style();
            candidateContentHeight = std::max<InlineLayoutUnit>(candidateContentHeight, styleToUse.usedLineHeight());
        } else if (inlineItem.isAtomicInlineBox() && lineBoxContain.contains(Style::WebkitLineBoxContainValue::Replaced))
            candidateContentHeight = std::max(candidateContentHeight, InlineLayoutUnit { formattingContext().geometryForBox(inlineItem.layoutBox()).marginBoxHeight() });
    }
    if (candidateContentHeight <= m_lineLogicalRect.height())
        return { m_lineLogicalRect };

    return floatAvoidingRect({ m_lineLogicalRect.topLeft(), m_lineLogicalRect.width(), candidateContentHeight }, m_lineMarginStart);
}

std::optional<LineBuilder::InitialLetterOffsets> LineBuilder::adjustLineRectForInitialLetterIfApplicable(const Box& floatBox)
{
    auto drop = floatBox.style().initialLetter().drop();
    auto isInitialLetter = floatBox.isFloatingPositioned() && floatBox.style().pseudoElementType() == PseudoElementType::FirstLetter && drop;
    if (!isInitialLetter)
        return { };

    // Here we try to set the vertical start position for the float in flush with the adjoining text content's cap height.
    // It's a super premature as at this point we don't normally deal with vertical geometry -other than the incoming vertical constraint.
    auto initialLetterCapHeightOffset = formattingContext().quirks().initialLetterAlignmentOffset(floatBox, rootStyle());
    // While initial-letter based floats do not set their clear property, intrusive floats from sibling IFCs are supposed to be cleared.
    auto intrusiveBottom = blockLayoutState().intrusiveInitialLetterLogicalBottom();
    if (!initialLetterCapHeightOffset && !intrusiveBottom)
        return { };

    auto clearGapBeforeFirstLine = InlineLayoutUnit { };
    if (intrusiveBottom) {
        // When intrusive initial letter is cleared, we introduce a clear gap. This is (with proper floats) normally computed before starting
        // line layout but intrusive initial letters are cleared only when another initial letter shows up. Regular inline content
        // does not need clearance.
        auto intrusiveInitialLetterWidth = std::max(0.f, m_lineLogicalRect.left() - m_lineInitialLogicalRect.left());
        m_lineLogicalRect.setLeft(m_lineInitialLogicalRect.left());
        m_lineLogicalRect.expandHorizontally(intrusiveInitialLetterWidth);
        clearGapBeforeFirstLine = *intrusiveBottom;
    }

    auto sunkenBelowFirstLineOffset = LayoutUnit { };
    auto letterHeight = floatBox.style().initialLetter().height();
    if (drop < letterHeight) {
        // Sunken/raised initial letter pushes contents of the first line down.
        auto numberOfSunkenLines = letterHeight - drop;
        auto verticalGapForInlineContent = numberOfSunkenLines * rootStyle().usedLineHeight();
        clearGapBeforeFirstLine += verticalGapForInlineContent;
        // And we pull the initial letter up.
        initialLetterCapHeightOffset = -verticalGapForInlineContent + initialLetterCapHeightOffset.value_or(0_lu);
    } else if (drop > letterHeight) {
        // Initial letter is sunken below the first line.
        auto numberOfLinesAboveInitialLetter = drop - letterHeight;
        sunkenBelowFirstLineOffset = numberOfLinesAboveInitialLetter * rootStyle().usedLineHeight();
    }

    m_lineLogicalRect.moveVertically(clearGapBeforeFirstLine);
    // There should never be multiple initial letters.
    ASSERT(!m_initialLetterClearGap);
    m_initialLetterClearGap = clearGapBeforeFirstLine;
    return InitialLetterOffsets { initialLetterCapHeightOffset.value_or(0_lu), sunkenBelowFirstLineOffset };
}

bool LineBuilder::shouldTryToPlaceFloatBox(const Box& floatBox, LayoutUnit floatBoxMarginBoxWidth, MayOverConstrainLine mayOverConstrainLine) const
{
    switch (mayOverConstrainLine) {
    case MayOverConstrainLine::Yes:
        return true;
    case MayOverConstrainLine::OnlyWhenFirstFloatOnLine:
        // This is a resumed float from a previous line. Now we need to find a place for it.
        // (which also means that the current line can't have any floats that we couldn't place yet)
        ASSERT(m_suspendedFloats.isEmpty());
        if (!isLineConstrainedByFloat())
            return true;
        [[fallthrough]];
    case MayOverConstrainLine::No: {
        auto lineIsConsideredEmpty = !m_line.hasContent() && !isLineConstrainedByFloat();
        if (lineIsConsideredEmpty)
            return true;
        // Non-clear type of floats stack up (horizontally). It's easy to check if there's space for this float at all,
        // while floats with clear needs post-processing to see if they overlap existing line content (and here we just check if they may fit at all).
        auto lineLogicalWidth = floatBox.hasFloatClear() ? m_lineInitialLogicalRect.width() : m_lineLogicalRect.width();
        auto availableWidthForFloat = lineLogicalWidth - m_line.contentLogicalRight() + m_line.trimmableTrailingWidth();
        return availableWidthForFloat >= InlineLayoutUnit { floatBoxMarginBoxWidth };
    }
    default:
        ASSERT_NOT_REACHED();
        return true;
    }
}

static bool haveEnoughSpaceForFloatWithClear(const LayoutRect& floatBoxMarginBox, bool isLeftPositioned, const InlineRect& lineLogicalRect, InlineLayoutUnit contentLogicalWidth)
{
    auto adjustedLineLogicalLeft = lineLogicalRect.left();
    auto adjustedLineLogicalRight = lineLogicalRect.right();
    if (isLeftPositioned)
        adjustedLineLogicalLeft = std::max<InlineLayoutUnit>(floatBoxMarginBox.maxX(), adjustedLineLogicalLeft);
    else
        adjustedLineLogicalRight = std::min<InlineLayoutUnit>(floatBoxMarginBox.x(), adjustedLineLogicalRight);
    auto availableSpaceForContentWithPlacedFloat = adjustedLineLogicalRight - adjustedLineLogicalLeft;
    return contentLogicalWidth <= availableSpaceForContentWithPlacedFloat;
}

bool LineBuilder::tryPlacingFloatBox(const Box& floatBox, MayOverConstrainLine mayOverConstrainLine)
{
    if (isFloatLayoutSuspended())
        return false;

    auto& floatingContext = this->floatingContext();
    auto& boxGeometry = formattingContext().geometryForBox(floatBox);
    if (!shouldTryToPlaceFloatBox(floatBox, boxGeometry.marginBoxWidth(), mayOverConstrainLine))
        return false;

    auto lineMarginBoxLeft = std::max(0.f, m_lineLogicalRect.left() - m_lineMarginStart);
    auto computeFloatBoxPosition = [&] {
        // Set static position first.
        auto staticPosition = LayoutPoint { lineMarginBoxLeft, m_lineLogicalRect.top() };
        if (auto additionalOffsets = adjustLineRectForInitialLetterIfApplicable(floatBox)) {
            staticPosition.setY(m_lineLogicalRect.top() + additionalOffsets->capHeightOffset);
            boxGeometry.setVerticalMargin({ boxGeometry.marginBefore() + additionalOffsets->sunkenBelowFirstLineOffset, boxGeometry.marginAfter() });
        }
        staticPosition.move(boxGeometry.marginStart(), boxGeometry.marginBefore());
        boxGeometry.setTopLeft(staticPosition);
        // Compute float position by running float layout.
        auto floatingPosition = floatingContext.positionForFloat(floatBox, boxGeometry, rootHorizontalConstraints());
        boxGeometry.setTopLeft(floatingPosition);
    };
    computeFloatBoxPosition();

    auto willFloatBoxShrinkLine = [&] {
        // Float boxes don't get positioned higher than the line.
        auto floatBoxMarginBox = BoxGeometry::marginBoxRect(boxGeometry);
        if (floatBoxMarginBox.isEmpty())
            return false;
        if (floatBoxMarginBox.right() <= lineMarginBoxLeft) {
            // Previous floats already constrain the line horizontally more than this one.
            return false;
        }
        // Empty rect case: "line-height: 0px;" line still intersects with intrusive floats.
        return floatBoxMarginBox.top() == m_lineLogicalRect.top() || floatBoxMarginBox.top() < m_lineLogicalRect.bottom();
    }();

    auto willFloatBoxWithClearFit = [&] {
        if (!willFloatBoxShrinkLine)
            return true;
        auto lineIsConsideredEmpty = !m_line.hasContent() && !isLineConstrainedByFloat();
        if (lineIsConsideredEmpty)
            return true;
        // When floats with clear are placed under existing floats, we may find ourselves in an over-constrained state and
        // can't place this float here.
        auto contentLogicalWidth = m_line.contentLogicalWidth() - m_line.trimmableTrailingWidth();
        return haveEnoughSpaceForFloatWithClear(BoxGeometry::marginBoxRect(boxGeometry), floatingContext.isStartPositioned(floatBox), m_lineLogicalRect, contentLogicalWidth);
    };
    if (floatBox.hasFloatClear() && !willFloatBoxWithClearFit())
        return false;

    auto placeFloatBox = [&] {
        auto lineIndex = m_previousLine ? (m_previousLine->lineIndex + 1) : 0lu;
        auto floatItem = floatingContext.makeFloatItem(floatBox, boxGeometry, lineIndex);
        layoutState().placedFloats().add(floatItem);
        m_placedFloats.append(floatItem);
    };
    placeFloatBox();

    auto adjustLineRectIfNeeded = [&] {
        if (!willFloatBoxShrinkLine) {
            // This float is placed outside the line box. No need to shrink the current line.
            return;
        }
        auto constraints = floatAvoidingRect(m_lineLogicalRect, m_lineMarginStart);
        m_lineLogicalRect = constraints.logicalRect;
        m_lineIsConstrainedByFloat.add(constraints.constrainedSideSet);
    };
    adjustLineRectIfNeeded();

    return true;
}

void LineBuilder::handleBlockContent(const InlineItem& blockItem)
{
    ASSERT(blockItem.isBlock());
    // Blocks are always the only content on the line.
    ASSERT(!m_line.hasContent());
    if (isInIntrinsicWidthMode()) {
        CheckedRef blockBox = downcast<ElementBox>(blockItem.layoutBox());
        auto& boxGeometry = formattingContext().geometryForBox(blockBox.get());
        auto& integrationUtils = formattingContext().integrationUtils();
        auto contribution = *intrinsicWidthMode() == IntrinsicWidthMode::Minimum ? integrationUtils.minContentLogicalWidthContribution(blockBox.get()) : integrationUtils.maxContentLogicalWidthContribution(blockBox.get());
        auto marginBoxWidth = contribution + boxGeometry.marginStart() + boxGeometry.marginEnd();
        return m_line.appendBlock(blockItem, marginBoxWidth);
    }

    if (rootStyle().writingMode().isBidiRTL())
        m_line.setContentNeedsBidiReordering();

    // Undo the eager advance from initialize before handing off to legacy block layout. Legacy reads
    // marginState and re-adds the margin via its own collapsing logic; passing the already-advanced
    // top would double-count the margin. Roll back m_lineLogicalRect.top too so the line's reported
    // bottom doesn't inherit the eager advance (which would shift the next line down).
    auto& marginState = blockLayoutState().marginState();
    // Unless the margin collapses out through the root's before edge, the line's top already includes it: either
    // initialize advanced the line by it, or the previous line's bottom absorbed it as a self collapsing block box's
    // offset. Block layout applies the margin from the container's own position, so take it back off before handing over.
    auto marginCollapsesThroughBeforeSide = marginState.atBeforeSideOfBlock && marginState.canCollapseMarginBeforeWithChildren;
    auto lineTopIncludesPendingMargin = !marginCollapsesThroughBeforeSide;
    if (lineTopIncludesPendingMargin) {
        if (auto blockMargin = marginState.margin())
            m_lineLogicalRect = { m_lineLogicalRect.top() - blockMargin, m_lineInitialLogicalRect.left(), m_lineInitialLogicalRect.width(), m_lineInitialLogicalRect.height() };
    }
    // Block layout places this margin from its own position, where the clearance is accounted for already.
    marginState.marginBeforeWithClearance = { };

    formattingContext().integrationUtils().layoutWithFormattingContextForBlockInInline(downcast<ElementBox>(blockItem.layoutBox()), LayoutPoint { m_lineLogicalRect.topLeft() }, layoutState());
    auto contentWidth = InlineLayoutUnit { };
    if (formattingContext().geometryForBox(blockItem.layoutBox()).borderBoxHeight())
        contentWidth = formattingContext().formattingUtils().inlineItemWidth(blockItem, { }, false);
    m_line.appendBlock(blockItem, contentWidth);
}

LineBuilder::Result LineBuilder::handleInlineContent(const InlineItemRange& layoutRange, LineCandidate& lineCandidate)
{
    auto result = LineBuilder::Result { };
    auto& inlineContent = lineCandidate.inlineContent;

    auto& continuousInlineContent = inlineContent.continuousContent();
    if (continuousInlineContent.runs().isEmpty()) {
        ASSERT(inlineContent.trailingLineBreak() || inlineContent.trailingWordBreakOpportunity());
        result = { inlineContent.trailingLineBreak() ? InlineContentBreaker::IsEndOfLine::Yes : InlineContentBreaker::IsEndOfLine::No };
        return result;
    }

    auto constraints = adjustedLineRectWithCandidateInlineContent(lineCandidate);
    auto availableWidthForCandidateContent = [&] {
        auto lineIndex = m_previousLine ? (m_previousLine->lineIndex + 1) : 0lu;
        // If width constraint overrides exist (e.g. text-wrap: balance), modify the available width accordingly.
        const auto& availableLineWidthOverride = layoutState().availableLineWidthOverride();
        auto widthOverride = availableLineWidthOverride.availableLineWidthOverrideForLine(lineIndex);
        auto availableTotalWidthForContent = widthOverride ? InlineLayoutUnit { widthOverride.value() } - m_lineMarginStart : constraints.logicalRect.width();
        return availableWidth(m_line, availableTotalWidthForContent, intrinsicWidthMode());
    }();

    auto lineHasContent = m_line.hasContent();
    auto verticalPositionHasFloatOrInlineContent = lineHasContent || isLineConstrainedByFloat() || !constraints.constrainedSideSet.isEmpty();
    auto lineBreakingResult = InlineContentBreaker::Result { InlineContentBreaker::Result::Action::Keep, InlineContentBreaker::IsEndOfLine::No, { }, { } };

    if (auto minimumRequiredWidth = continuousInlineContent.minimumRequiredWidth(); minimumRequiredWidth && *minimumRequiredWidth > availableWidthForCandidateContent) {
        if (verticalPositionHasFloatOrInlineContent)
            lineBreakingResult = InlineContentBreaker::Result { InlineContentBreaker::Result::Action::Wrap, InlineContentBreaker::IsEndOfLine::Yes, { }, { } };
    } else {
        auto lineStatus = InlineContentBreaker::LineStatus { m_line.contentLogicalRight(), availableWidthForCandidateContent, m_line.trimmableTrailingWidth(), m_line.trailingSoftHyphenWidth(), m_line.isTrailingRunFullyTrimmable(), verticalPositionHasFloatOrInlineContent, !m_wrapOpportunityList.isEmpty() };
        auto needsClonedDecorationHandling = inlineContent.hasTrailingClonedDecoration() || !m_line.inlineBoxListWithClonedDecorationEnd().isEmpty();
        if (needsClonedDecorationHandling)
            lineBreakingResult = handleInlineContentWithClonedDecoration(lineCandidate, lineStatus);
        else if (continuousInlineContent.logicalWidth() > availableWidthForCandidateContent)
            lineBreakingResult = inlineContentBreaker().processInlineContent(continuousInlineContent, lineStatus);
    }
    result = processLineBreakingResult(lineCandidate, layoutRange, lineBreakingResult);

    auto lineGainsNewContent = lineBreakingResult.action == InlineContentBreaker::Result::Action::Keep || lineBreakingResult.action == InlineContentBreaker::Result::Action::Break;
    if (lineGainsNewContent || !lineHasContent) {
        // In some cases in order to put this content on the line, we have to avoid float boxes that didn't constrain the line initially.
        // (e.g. when this new content is taller than any previous content and there are vertically stacked floats)
        // In some other cases we can't put any content on the line due to such newly discovered floats (e.g. shape-outside floats with gaps in-between them in vertical axis)
        m_lineLogicalRect = constraints.logicalRect;
        m_lineIsConstrainedByFloat.add(constraints.constrainedSideSet);
    }
    m_candidateContentMaximumHeight = constraints.logicalRect.height();
    return result;
}

static inline InlineLayoutUnit NODELETE lineBreakingResultContentWidth(const InlineContentBreaker::ContinuousContent::RunList& runs, const InlineContentBreaker::Result::PartialTrailingContent& trailingContent)
{
    if (trailingContent.trailingRunIndex >= runs.size()) {
        ASSERT_NOT_REACHED();
        return { };
    }

    auto contentWidth = InlineLayoutUnit { };
    for (size_t index = 0; index < trailingContent.trailingRunIndex; ++index)
        contentWidth += runs[index].contentWidth();

    if (auto partialTrailingRun = trailingContent.partialRun)
        return contentWidth + partialTrailingRun->logicalWidth + partialTrailingRun->hyphenWidth.value_or(0.f);

    auto& trailingRun = runs[trailingContent.trailingRunIndex];
    return contentWidth + trailingRun.contentWidth() + trailingContent.hyphenWidth.value_or(0.f);
}

InlineLayoutUnit LineBuilder::placedClonedDecorationWidth(const InlineContentBreaker::ContinuousContent::RunList& runs) const
{
    // Collect already placed, not yet closed inline boxes on the line (minus what we are about to close with the candidate runs)
    // e.g. <div><span>1 <span>2 3 4</span></span></div>
    // At [3] we've got 2 inline boxes placed on the line and they may have space taking (cloned) decoration ends.
    // At [4</span></span>] all inline boxes are closed.
    auto& formattingContext = this->formattingContext();

    HashSet<const Box*> clonedInlineBoxes;
    auto clonedDecorationEndWidth = InlineLayoutUnit { };
    for (auto* box : m_line.inlineBoxListWithClonedDecorationEnd()) {
        clonedDecorationEndWidth += formattingContext.geometryForBox(*box).borderAndPaddingEnd();
        clonedInlineBoxes.add(box);
    }

    for (size_t index = 0; index < runs.size(); ++index) {
        auto& inlineItem = runs[index].inlineItem;
        if (inlineItem.isInlineBoxEnd() && clonedInlineBoxes.contains(&inlineItem.layoutBox()))
            clonedDecorationEndWidth -= formattingContext.geometryForBox(inlineItem.layoutBox()).borderAndPaddingEnd();
    }

    return clonedDecorationEndWidth;
}

InlineLayoutUnit LineBuilder::clonedDecorationAtBreakingPosition(const InlineContentBreaker::ContinuousContent::RunList& runs, const InlineContentBreaker::Result::PartialTrailingContent& trailingContent) const
{
    // Compute how much decoration end we have to put as trailing content if we were to break the line at this position.
    // Collect already committed, but not yet closed inline boxes in addition to these new ones, coming with the candidate content.
    // e.g. <div><span>1 <span>2 3 4</span></span></div>
    // At [<span>2], we have to account for the leading inline box (provided it has cloned decoration) and the inline box (again, if it has cloned decoration) in the candidate content.
    if (trailingContent.trailingRunIndex >= runs.size()) {
        ASSERT_NOT_REACHED();
        return { };
    }

    auto& formattingContext = this->formattingContext();
    auto clonedDecorationWidth = InlineLayoutUnit { };

    for (auto* box : m_line.inlineBoxListWithClonedDecorationEnd())
        clonedDecorationWidth += formattingContext.geometryForBox(*box).borderAndPaddingEnd();

    for (size_t index = 0; index <= trailingContent.trailingRunIndex; ++index) {
        auto& inlineItem = runs[index].inlineItem;
        if (!inlineItem.isInlineBoxStartOrEnd() || inlineItem.style().boxDecorationBreak() != BoxDecorationBreak::Clone)
            continue;

        auto& inlineBoxGeometry = formattingContext.geometryForBox(inlineItem.layoutBox());
        if (inlineItem.isInlineBoxStart()) {
            clonedDecorationWidth += inlineBoxGeometry.borderAndPaddingEnd();
            continue;
        }
        if (inlineItem.isInlineBoxEnd()) {
            clonedDecorationWidth -= inlineBoxGeometry.borderAndPaddingEnd();
            continue;
        }
    }
    ASSERT(clonedDecorationWidth >= 0);
    return std::max(0.f, clonedDecorationWidth);
}

InlineContentBreaker::Result LineBuilder::handleInlineContentWithClonedDecoration(const LineCandidate& lineCandidate, InlineContentBreaker::LineStatus lineStatus)
{
    // 1. call content breaker to see whether the candidate content fits or not
    // 2. when content breaker tells us that this continuous content needs to be broken up, we have to check whether the partial content we are planning to put on the line has cloned decoration and whether it also fits
    // 3. traverse the candidate content up to the breaking position and compute the width of the cloned decoration(s)
    // 4. check if there's enough space for both content and its cloned decoration(s)
    // 5. if not, let's try again (go to #1) with reduced available space
    // At some point we either manage to fit the content + its cloned decoration(s) or we run out of available space
    // e.g.
    // <div style="width: 30px; word-break: break-all">ab<span style="-webkit-box-decoration-break: clone; padding-right: 20px">cd</span>ef</div>
    // (where each character is 10px wide)
    // [ab<span>cd</span>ef] is the continous content (there's no soft wrap opportunity in-between)
    // The breaking position is between [c] and [d]. We are going to put [abc] on the line which means we have to have space
    // for the enclosing inline box's (cloned) decoration end (20px) too, 50px altogether. -but we only have 30px space here.
    // And now we are at step (5); let's probe line breaking with reduced available space, go to step (1) until we find a valid breaking position (which is after [b]).
    ASSERT(lineCandidate.inlineContent.hasTrailingClonedDecoration() || !m_line.inlineBoxListWithClonedDecorationEnd().isEmpty());

    auto& inlineContent = lineCandidate.inlineContent;
    auto& continuousInlineContent = inlineContent.continuousContent();
    auto& runs = continuousInlineContent.runs();
    auto initialAvailableWidth = lineStatus.availableWidth;

    lineStatus.availableWidth -= placedClonedDecorationWidth(runs);

    if (continuousInlineContent.logicalWidth() <= lineStatus.availableWidth)
        return { InlineContentBreaker::Result::Action::Keep, InlineContentBreaker::IsEndOfLine::No, { }, { } };

    while (lineStatus.availableWidth) {
        auto lineBreakingResult = inlineContentBreaker().processInlineContent(continuousInlineContent, lineStatus);
        if (lineBreakingResult.action != InlineContentBreaker::Result::Action::Break)
            return lineBreakingResult;

        if (!lineBreakingResult.partialTrailingContent) {
            ASSERT_NOT_REACHED();
            return lineBreakingResult;
        }

        auto contentWidth = lineBreakingResultContentWidth(runs, *lineBreakingResult.partialTrailingContent);
        auto clonedDecorationWidth = clonedDecorationAtBreakingPosition(runs, *lineBreakingResult.partialTrailingContent);

        if (contentWidth + clonedDecorationWidth <= initialAvailableWidth)
            return lineBreakingResult;
        lineStatus.availableWidth = std::max(0.f, std::min(lineStatus.availableWidth, contentWidth) - 1.f);
    }

    // In case of this unlikely scenario where we couldn't find a fitting setup, let's just go with the last result -this will most likely produce decoration overflow which may be correct in some cases (e.g. 0px available space)
    return inlineContentBreaker().processInlineContent(continuousInlineContent, lineStatus);
}

void LineBuilder::commitCandidateContent(LineCandidate& lineCandidate, std::optional<InlineContentBreaker::Result::PartialTrailingContent> partialTrailingContent)
{
    auto& inlineContent = lineCandidate.inlineContent;
    auto& runs = inlineContent.continuousContent().runs();
    if (runs.isEmpty()) {
        ASSERT(!partialTrailingContent);
        return;
    }

    auto shapingBoundaryStart = std::optional<size_t> { };
    auto appendRun = [&](auto& index) {
        auto& run = runs[index];
        auto& inlineItem = run.inlineItem;

        if (inlineItem.bidiLevel() != UBIDI_DEFAULT_LTR)
            m_line.setContentNeedsBidiReordering();

        if (auto* inlineTextItem = dynamicDowncast<InlineTextItem>(inlineItem)) {
            auto shapingBoundary = [&]() -> std::optional<Line::ShapingBoundary> {
                if (!layoutState().shouldShapeTextAcrossInlineBoxes())
                    return { };

                // Special case trailing partial run as shaping end.
                if (shapingBoundaryStart && partialTrailingContent && partialTrailingContent->trailingRunIndex == index)
                    return { Line::ShapingBoundary::End };

                if (run.shapingBoundary == InlineContentBreaker::ContinuousContent::Run::ShapingBoundary::Start) {
                    ASSERT(!shapingBoundaryStart);
                    shapingBoundaryStart = index;
                    return { Line::ShapingBoundary::Start };
                }

                if (run.shapingBoundary == InlineContentBreaker::ContinuousContent::Run::ShapingBoundary::End) {
                    ASSERT(shapingBoundaryStart);
                    shapingBoundaryStart = { };
                    return { Line::ShapingBoundary::End };
                }

                if (shapingBoundaryStart)
                    return { Line::ShapingBoundary::Inside };

                return { };
            };
            m_line.appendText(*inlineTextItem, run.style, run.contentWidth(), shapingBoundary());
            return;
        }

        if (inlineItem.isLineBreak()) {
            m_line.appendLineBreak(inlineItem, run.style);
            return;
        }

        if (inlineItem.isWordBreakOpportunity()) {
            m_line.appendWordBreakOpportunity(inlineItem, run.style);
            return;
        }

        if (inlineItem.isInlineBoxStart()) {
            m_line.appendInlineBoxStart(inlineItem, run.style, run.contentWidth(), run.textSpacingAdjustment);
            return;
        }

        if (inlineItem.isInlineBoxEnd()) {
            m_line.appendInlineBoxEnd(inlineItem, run.style, run.contentWidth());
            return;
        }

        if (inlineItem.isAtomicInlineBox()) {
            m_line.appendAtomicInlineBox(inlineItem, run.style, run.contentWidth());
            return;
        }

        if (inlineItem.isOutOfFlow()) {
            ASSERT(!run.contentWidth());
            m_line.appendOutOfFlow(inlineItem, run.style);
            return;
        }

        ASSERT_NOT_REACHED();
    };

    if (partialTrailingContent && inlineContent.continuousContent().hasShapedContent())
        shapePartialLineCandidate(lineCandidate, partialTrailingContent->trailingRunIndex);

    ASSERT(!partialTrailingContent || partialTrailingContent->trailingRunIndex <= runs.size());
    auto endOfNonPartialContent = (partialTrailingContent ? std::min(partialTrailingContent->trailingRunIndex, runs.size()) : runs.size());
    for (size_t index = 0; index < endOfNonPartialContent; ++index)
        appendRun(index);

    if (partialTrailingContent) {
        auto trailingRunIndex = partialTrailingContent->trailingRunIndex;
        if (trailingRunIndex >= runs.size()) {
            ASSERT_NOT_REACHED();
            return;
        }

        if (auto partialRun = partialTrailingContent->partialRun) {
            // Create and commit partial trailing item.
            if (auto* trailingInlineTextItem = dynamicDowncast<InlineTextItem>(runs[trailingRunIndex].inlineItem)) {
                auto partialTrailingTextItem = trailingInlineTextItem->left(partialRun->length);
                m_line.appendText(partialTrailingTextItem, trailingInlineTextItem->style(), partialRun->logicalWidth, shapingBoundaryStart ? std::make_optional(Line::ShapingBoundary::End) : std::nullopt);
                if (trailingInlineTextItem->bidiLevel() != UBIDI_DEFAULT_LTR)
                    m_line.setContentNeedsBidiReordering();
            } else
                ASSERT_NOT_REACHED();

            if (auto hyphenWidth = partialRun->hyphenWidth)
                m_line.addTrailingHyphen(*hyphenWidth);
        } else {
            appendRun(trailingRunIndex);
            if (auto hyphenWidth = partialTrailingContent->hyphenWidth)
                m_line.addTrailingHyphen(*hyphenWidth);
        }
    }
}

static const InlineItem& NODELETE wrapOpportunityToRevertTo(const WrapOpportunityList& wrapOpportunityList)
{
    ASSERT(!wrapOpportunityList.isEmpty());
    auto enclosingBoxForWrapOpportunity = [](auto& wrapOpportunity) -> const Box& {
        // Inline boxes set the wrapping rules for their content and not for themselves, so the box that encloses a wrap
        // opportunity is the inline box its trailing content belongs to.
        auto& layoutBox = wrapOpportunity.layoutBox();
        return layoutBox.isInlineBox() ? layoutBox : layoutBox.parent();
    };

    // https://drafts.csswg.org/css-text-4/#wrap-inside
    // Prefer the last wrap opportunity that is outside any wrap-inside: avoid box.
    for (auto* wrapOpportunity : wrapOpportunityList | std::views::reverse) {
        if (!enclosingBoxForWrapOpportunity(*wrapOpportunity).style().effectiveWrapInsideAvoid())
            return *wrapOpportunity;
    }
    // Every opportunity is inside an avoid box, so we must break within one. "A break in an outer box must be used
    // before a break within an inner box", so revert to the last opportunity sitting directly in an outermost avoid box
    // - one whose enclosing box is not itself inside another avoid box.
    for (auto* wrapOpportunity : wrapOpportunityList | std::views::reverse) {
        if (!enclosingBoxForWrapOpportunity(*wrapOpportunity).parent().style().effectiveWrapInsideAvoid())
            return *wrapOpportunity;
    }
    // The outermost avoid box is the only content on the line; break inside it as a last resort.
    return *wrapOpportunityList.last();
}

LineBuilder::Result LineBuilder::processLineBreakingResult(LineCandidate& lineCandidate, const InlineItemRange& layoutRange, const InlineContentBreaker::Result& lineBreakingResult)
{
    auto& candidateRuns = lineCandidate.inlineContent.continuousContent().runs();

    switch (lineBreakingResult.action) {
    case InlineContentBreaker::Result::Action::Keep: {
        // This continuous content can be fully placed on the current line.
        commitCandidateContent(lineCandidate, lineBreakingResult.partialTrailingContent);
        // We are keeping this content on the line but we need to check if we could have wrapped here
        // in order to be able to revert back to this position if needed.
        // Let's just ignore cases like collapsed leading whitespace for now.
        if (lineCandidate.inlineContent.hasTrailingSoftWrapOpportunity() && m_line.hasContent()) {
            auto& trailingRun = candidateRuns.last();
            auto& trailingInlineItem = trailingRun.inlineItem;

            // Note that wrapping here could be driven both by the style of the parent and the inline item itself.
            // e.g inline boxes set the wrapping rules for their content and not for themselves.
            auto& layoutBoxParent = trailingInlineItem.layoutBox().parent();

            // Need to ensure we use the correct style here, so the content breaker and line builder remain in sync.
            auto& parentStyle = isFirstFormattedLineCandidate() ? layoutBoxParent.firstLineStyle() : layoutBoxParent.style();

            auto isWrapOpportunity = TextUtil::isWrappingAllowed(parentStyle);
            if (!isWrapOpportunity && trailingInlineItem.isInlineBoxStartOrEnd())
                isWrapOpportunity = TextUtil::isWrappingAllowed(trailingRun.style);
            if (isWrapOpportunity)
                m_wrapOpportunityList.append(&trailingInlineItem);
        }
        return { lineBreakingResult.isEndOfLine, { candidateRuns.size(), false } };
    }
    case InlineContentBreaker::Result::Action::Wrap: {
        ASSERT(lineBreakingResult.isEndOfLine == InlineContentBreaker::IsEndOfLine::Yes);
        // This continuous content can't be placed on the current line. Nothing to commit at this time.
        // However there are cases when, due to whitespace collapsing, this overflowing content should not be separated from
        // the content on the line.
        // <div>X <span> X</span></div>
        // If the second 'X' overflows the line, the trailing whitespace gets trimmed which introduces a stray inline box
        // on the first line ('X <span>' and 'X</span>' first and second line respectively).
        // In such cases we need to revert the content on the line to a previous wrapping opportunity to keep such content together.
        auto needsRevert = m_line.trimmableTrailingWidth() && !m_line.runs().isEmpty() && m_line.runs().last().isInlineBoxStart();
        if (needsRevert && m_wrapOpportunityList.size() > 1) {
            m_wrapOpportunityList.removeLast();
            return { InlineContentBreaker::IsEndOfLine::Yes, { rebuildLineWithInlineContent(layoutRange, *m_wrapOpportunityList.last()), true } };
        }
        return { InlineContentBreaker::IsEndOfLine::Yes, { }, { }, overflowWidthAsLeadingForNextLine(candidateRuns, lineBreakingResult) };
    }
    case InlineContentBreaker::Result::Action::WrapWithHyphen:
        ASSERT(lineBreakingResult.isEndOfLine == InlineContentBreaker::IsEndOfLine::Yes);
        // This continuous content can't be placed on the current line, nothing to commit.
        // However we need to make sure that the current line gains a trailing hyphen.
        ASSERT(m_line.trailingSoftHyphenWidth());
        m_line.addTrailingHyphen(*m_line.trailingSoftHyphenWidth());
        return { InlineContentBreaker::IsEndOfLine::Yes };
    case InlineContentBreaker::Result::Action::RevertToLastWrapOpportunity:
        ASSERT(lineBreakingResult.isEndOfLine == InlineContentBreaker::IsEndOfLine::Yes);
        // Not only this content can't be placed on the current line, but we even need to revert the line back to an earlier position.
        ASSERT(!m_wrapOpportunityList.isEmpty());
        return { InlineContentBreaker::IsEndOfLine::Yes, { rebuildLineWithInlineContent(layoutRange, wrapOpportunityToRevertTo(m_wrapOpportunityList)), true } };
    case InlineContentBreaker::Result::Action::RevertToLastNonOverflowingWrapOpportunity:
        ASSERT(lineBreakingResult.isEndOfLine == InlineContentBreaker::IsEndOfLine::Yes);
        ASSERT(!m_wrapOpportunityList.isEmpty());
        if (auto committedCount = rebuildLineForTrailingSoftHyphen(layoutRange))
            return { InlineContentBreaker::IsEndOfLine::Yes, { committedCount, true } };
        return { InlineContentBreaker::IsEndOfLine::Yes };
    case InlineContentBreaker::Result::Action::Break: {
        ASSERT(lineBreakingResult.isEndOfLine == InlineContentBreaker::IsEndOfLine::Yes);
        // Commit the combination of full and partial content on the current line.
        ASSERT(lineBreakingResult.partialTrailingContent);
        commitCandidateContent(lineCandidate, lineBreakingResult.partialTrailingContent);
        // When breaking multiple runs <span style="word-break: break-all">text</span><span>content</span>, we might end up breaking them at run boundary.
        // It simply means we don't really have a partial run. Partial content yes, but not partial run.
        auto trailingRunIndex = lineBreakingResult.partialTrailingContent->trailingRunIndex;
        auto committedInlineItemCount = trailingRunIndex + 1;
        if (!lineBreakingResult.partialTrailingContent->partialRun)
            return { InlineContentBreaker::IsEndOfLine::Yes, { committedInlineItemCount, false } };

        auto partialRun = *lineBreakingResult.partialTrailingContent->partialRun;
        auto& trailingInlineTextItem = downcast<InlineTextItem>(candidateRuns[trailingRunIndex].inlineItem);
        ASSERT(partialRun.length < trailingInlineTextItem.length());
        auto overflowLength = trailingInlineTextItem.length() - partialRun.length;
        return { InlineContentBreaker::IsEndOfLine::Yes, { committedInlineItemCount, false }, overflowLength, overflowWidthAsLeadingForNextLine(candidateRuns, lineBreakingResult) };
    }
    }
    ASSERT_NOT_REACHED();
    return { InlineContentBreaker::IsEndOfLine::No };
}

size_t LineBuilder::rebuildLineWithInlineContent(const InlineItemRange& layoutRange, const InlineItem& lastInlineItemToAdd)
{
    ASSERT(!m_wrapOpportunityList.isEmpty());
    m_line.initialize(m_lineSpanningInlineBoxes, isFirstFormattedLineCandidate());

    if (m_partialLeadingTextItem && &*m_partialLeadingTextItem == &lastInlineItemToAdd) {
        LineCandidate lineCandidate;
        lineCandidate.inlineContent.appendInlineItem(*m_partialLeadingTextItem, m_partialLeadingTextItem->style(), formattingContext().formattingUtils().inlineItemWidth(*m_partialLeadingTextItem, { }, false));
        commitCandidateContent(lineCandidate, { });
        return 1;
    }

    size_t numberOfFloatsInRange = 0;
    auto endOfCandidateContent = layoutRange.startIndex();
    for (; endOfCandidateContent < layoutRange.endIndex(); ++endOfCandidateContent) {
        if (m_inlineItemList[endOfCandidateContent].isFloat())
            ++numberOfFloatsInRange;
        if (&m_inlineItemList[endOfCandidateContent] == &lastInlineItemToAdd) {
            ++endOfCandidateContent;
            break;
        }
    }
    ASSERT(endOfCandidateContent < layoutRange.endIndex());

    LineCandidate lineCandidate;
    auto candidateStartEndIndex = std::pair<size_t, size_t> { layoutRange.startIndex(), endOfCandidateContent };
    // We might already have added floats. They shrink the available horizontal space for the line.
    // Let's just reuse what the line has at this point.
    candidateContentForLine(lineCandidate, candidateStartEndIndex, layoutRange, m_line.contentLogicalRight(), SkipFloats::Yes);
    auto result = processLineBreakingResult(lineCandidate, layoutRange, { InlineContentBreaker::Result::Action::Keep, InlineContentBreaker::IsEndOfLine::Yes, { }, { } });

    // Remove floats that are outside of this "rebuild" range to ensure we don't add them twice.
    auto unplaceFloatBox = [&](const Box& floatBox) -> bool {
        m_placedFloats.removeFirstMatching([&floatBox](auto& placedFloatItem) {
            return placedFloatItem.layoutBox() == &floatBox;
        });
        return layoutState().placedFloats().remove(floatBox);
    };
    for (auto index = endOfCandidateContent; index < layoutRange.endIndex(); ++index) {
        auto& inlineItem = m_inlineItemList[index];
        if (inlineItem.isFloat() && unplaceFloatBox(inlineItem.layoutBox()))
            break;
    }

    return result.committedCount.value + numberOfFloatsInRange;
}

size_t LineBuilder::rebuildLineForTrailingSoftHyphen(const InlineItemRange& layoutRange)
{
    if (m_wrapOpportunityList.isEmpty()) {
        // We are supposed to have a wrapping opportunity on the current line at this point.
        ASSERT_NOT_REACHED();
        return { };
    }
    // Revert all the way back to a wrap opportunity when either a soft hyphen fits or no hyphen is required.
    for (auto i = m_wrapOpportunityList.size(); i-- > 1;) {
        auto& softWrapOpportunityItem = *m_wrapOpportunityList[i];
        // FIXME: If this turns out to be a perf issue, we could also traverse the wrap list and keep adding the items
        // while watching the available width very closely.
        auto committedCount = rebuildLineWithInlineContent(layoutRange, softWrapOpportunityItem);
        auto availableWidth = m_lineLogicalRect.width() - m_line.contentLogicalRight();
        auto trailingSoftHyphenWidth = m_line.trailingSoftHyphenWidth();
        // Check if the trailing hyphen now fits the line (or we don't need hyphen anymore).
        if (!trailingSoftHyphenWidth || trailingSoftHyphenWidth <= availableWidth) {
            if (trailingSoftHyphenWidth)
                m_line.addTrailingHyphen(*trailingSoftHyphenWidth);
            return committedCount;
        }
    }
    // Have at least some content on the line.
    auto committedCount = rebuildLineWithInlineContent(layoutRange, *m_wrapOpportunityList.first());
    if (auto trailingSoftHyphenWidth = m_line.trailingSoftHyphenWidth())
        m_line.addTrailingHyphen(*trailingSoftHyphenWidth);
    return committedCount;
}

bool LineBuilder::isLastLineWithInlineContent(const LineContent& lineContent, size_t needsLayoutEnd, const Line::RunList& lineRuns) const
{
    if (lineContent.partialTrailingContentLength)
        return false;
    // FIXME: This needs work with partial layout.
    auto& formattingContext = this->formattingContext();
    if (lineContent.range.endIndex() == needsLayoutEnd) {
        if (!lineContent.range.start) {
            // This is both the first and the last line.
            return true;
        }
        for (auto& lineRun : lineRuns | std::views::reverse) {
            if (Line::Run::isContentfulOrHasDecoration(lineRun, formattingContext))
                return true;
        }
        return false;
    }
    // Look ahead to see if there's more inline type of inline items.
    for (auto i = lineContent.range.endIndex(); i < needsLayoutEnd && i < m_inlineItemList.size(); ++i) {
        // A block level box (block-in-inline) closes the inline content: whatever follows it starts after the block and can't extend this line, so this is the line's last inline content.
        if (m_inlineItemList[i].isBlock())
            return true;
        if (isContentfulOrHasDecoration(m_inlineItemList[i], formattingContext))
            return false;
    }
    return true;
}

}
}

```

## LineLayoutResult.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/LineLayoutResult.h). Path: `Source/WebCore/layout/formattingContexts/inline/LineLayoutResult.h`. Source bytes: 5444; source lines: 119; SHA-256: `b0b6bf43596abe0343daf4bba12feaa0d6249d72efdc275d53a4de32fb4eb10e`.

```cpp
/*
 * Copyright (C) 2023 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <WebCore/InlineLine.h>
#include <WebCore/InlineLineTypes.h>
#include <WebCore/LayoutUnits.h>
#include <WebCore/PlacedFloats.h>

namespace WebCore {
namespace Layout {

struct LineLayoutResult {
    using PlacedFloatList = PlacedFloats::List;
    using SuspendedFloatList = Vector<const Box*>;

    InlineItemRange inlineItemRange;
    Line::RunList runs;

    struct FloatContent {
        PlacedFloatList placedFloats;
        SuspendedFloatList suspendedFloats;
        OptionSet<UsedFloat> hasIntrusiveFloat { };
    };
    FloatContent floatContent { };

    struct ContentGeometry {
        InlineLayoutUnit logicalLeft { 0.f };
        InlineLayoutUnit logicalWidth { 0.f };
        InlineLayoutUnit logicalRightIncludingNegativeMargin { 0.f }; // Note that with negative horizontal margin value, contentLogicalLeft + contentLogicalWidth is not necessarily contentLogicalRight.
        std::optional<InlineLayoutUnit> trailingOverflowingContentWidth { };
    };
    ContentGeometry contentGeometry { };

    struct LineGeometry {
        InlineLayoutPoint logicalTopLeft;
        InlineLayoutUnit logicalWidth { 0.f };
        InlineLayoutPoint initialLogicalTopLeft;
        InlineLayoutUnit intrusiveFloatsOffset { 0.f }; // Inherited floats from parent formatting context offseting line box.
        std::optional<InlineLayoutUnit> initialLetterClearGap { };
    };
    LineGeometry lineGeometry { };

    struct HangingContent {
        bool shouldContributeToScrollableOverflow { false };
        InlineLayoutUnit logicalWidth { 0.f };
        InlineLayoutUnit hangablePunctuationStartWidth { 0.f };
    };
    HangingContent hangingContent { };

    struct Directionality {
        Vector<int32_t> visualOrderList;
        TextDirection inlineBaseDirection { TextDirection::LTR };
    };
    Directionality directionality { };

    struct IsFirstLast {
        IsFirstFormattedLine isFirstFormattedLine { IsFirstFormattedLine::Yes };
        bool isLastLineWithInlineContent { true };
    };
    IsFirstLast isFirstLast { };

    struct Ruby {
        HashMap<const Box*, InlineLayoutUnit> baseAlignmentOffsetList { };
        InlineLayoutUnit annotationAlignmentOffset { 0.f };
    };
    Ruby ruby { };

    // Misc
    enum InlineContentEnding : uint8_t { Generic, Hyphen, LineBreak };
    std::optional<InlineContentEnding> contentfulInlineContentEnding { }; // No value means line either does not have any inline content (float, out-of-flow or block inside inline) or it's non-contentful inline e.g <span></span>

    enum class InflowContentType : uint8_t { Inline, Block };
    std::optional<InflowContentType> inflowContentType() const
    {
        if (contentfulInlineContentEnding.has_value())
            return InflowContentType::Inline;
        if (!runs.isEmpty() && runs.last().isBlock())
            return InflowContentType::Block;
        return { };
    }
    bool hasContentfulInFlowContent() const { return inflowContentType().has_value(); }
    bool hasContentfulInlineContent() const { return inflowContentType() == InflowContentType::Inline; }
    bool isBlockContent() const { return inflowContentType() == InflowContentType::Block; }

    bool endsWithHyphen() const { return contentfulInlineContentEnding && *contentfulInlineContentEnding == InlineContentEnding::Hyphen; }
    bool endsWithLineBreak() const { return contentfulInlineContentEnding && *contentfulInlineContentEnding == InlineContentEnding::LineBreak; }

    size_t nonSpanningInlineLevelBoxCount { 0 };
    InlineLayoutUnit trimmedTrailingWhitespaceWidth { 0.f }; // only used for line-break: after-white-space currently
    InlineLayoutUnit firstLineStartTrim { 0.f }; // This is how much text-box-trim: start adjusts the first line box. We only need it to adjust the initial letter float position (which will not be needed once we drop the float behavior)
    std::optional<InlineLayoutUnit> hintForNextLineTopToAvoidIntrusiveFloat { }; // This is only used for cases when intrusive floats prevent any content placement at current vertical position.
};

}
}
```

## InlineFormattingUtils.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineFormattingUtils.cpp). Path: `Source/WebCore/layout/formattingContexts/inline/InlineFormattingUtils.cpp`. Source bytes: 36835; source lines: 662; SHA-256: `b5bbeaa10f6b38b6e9757afa7d7e2d93f9c555245561323a7e89685745ccd527`.

```cpp
/*
 * Copyright (C) 2018-2026 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "InlineFormattingUtils.h"

#include "FloatingContext.h"
#include "FontCascade.h"
#include "FormattingContext.h"
#include "InlineDisplayContent.h"
#include "InlineLevelBoxInlines.h"
#include "InlineLineBoxVerticalAligner.h"
#include "InlineLineTypes.h"
#include "InlineQuirks.h"
#include "LayoutBoxInlines.h"
#include "LayoutElementBox.h"
#include "RenderObjectDocument.h"
#include "RubyFormattingContext.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "StylePrimitiveNumericTypes+Evaluation.h"
#include <ranges>

namespace WebCore {
namespace Layout {

InlineFormattingUtils::InlineFormattingUtils(const InlineFormattingContext& inlineFormattingContext)
    : m_inlineFormattingContext(inlineFormattingContext)
{
}

InlineLayoutUnit InlineFormattingUtils::logicalTopForNextLine(const LineLayoutResult& lineLayoutResult, const InlineRect& lineLogicalRect, const FloatingContext& floatingContext) const
{
    auto didManageToPlaceInlineContentOrFloat = !lineLayoutResult.inlineItemRange.isEmpty();
    if (didManageToPlaceInlineContentOrFloat) {
        auto logicalTopCandidateByContent = [&] {
            // Normally the next line's logical top is the previous line's logical bottom, but when the line ends
            // with the clear property set, the next line needs to clear the existing floats.
            if (!lineLayoutResult.hasContentfulInlineContent())
                return lineLogicalRect.bottom();
            CheckedRef lastRunLayoutBox = lineLayoutResult.runs.last().layoutBox();
            if (!lastRunLayoutBox->hasFloatClear() || lastRunLayoutBox->isOutOfFlowPositioned())
                return lineLogicalRect.bottom();
            auto blockAxisPositionWithClearance = floatingContext.blockAxisPositionWithClearance(lastRunLayoutBox.get(), formattingContext().geometryForBox(lastRunLayoutBox.get()));
            if (!blockAxisPositionWithClearance)
                return lineLogicalRect.bottom();
            return std::max(lineLogicalRect.bottom(), InlineLayoutUnit(blockAxisPositionWithClearance->position));
        };
        return logicalTopCandidateByContent();
    }

    auto intrusiveFloatBottom = [&]() -> std::optional<InlineLayoutUnit> {
        // Floats must have prevented us placing any content on the line.
        // Move next line below the intrusive float(s).
        ASSERT(!lineLayoutResult.hasContentfulInlineContent() || lineLayoutResult.runs[0].isLineSpanningInlineBoxStart());
        auto nextLineLogicalTop = [&]() -> LayoutUnit {
            if (auto nextLineLogicalTopCandidate = lineLayoutResult.hintForNextLineTopToAvoidIntrusiveFloat)
                return LayoutUnit { *nextLineLogicalTopCandidate };
            // We have to have a hint when intrusive floats prevented any inline content placement.
            ASSERT_NOT_REACHED();
            return LayoutUnit { lineLogicalRect.top() + formattingContext().root().style().usedLineHeight() };
        };
        auto floatConstraints = floatingContext.constraints(toLayoutUnit(lineLogicalRect.top()), nextLineLogicalTop(), FloatingContext::MayBeAboveLastFloat::Yes);
        if (floatConstraints.start && floatConstraints.end) {
            // In case of left and right constraints, we need to pick the one that's closer to the current line.
            return std::min(floatConstraints.start->y, floatConstraints.end->y);
        }
        if (floatConstraints.start)
            return floatConstraints.start->y;
        if (floatConstraints.end)
            return floatConstraints.end->y;
        // If we didn't manage to place a content on this vertical position due to intrusive floats, we have to have
        // at least one float here.
        ASSERT_NOT_REACHED();
        return { };
    };
    if (auto firstAvailableVerticalPosition = intrusiveFloatBottom(); firstAvailableVerticalPosition && *firstAvailableVerticalPosition > lineLogicalRect.top())
        return *firstAvailableVerticalPosition;
    // Do not get stuck on the same vertical position even when we find ourselves in this unexpected state.
    return ceil(nextafter(lineLogicalRect.bottom(), std::numeric_limits<float>::max()));
}

bool InlineFormattingUtils::inlineLevelBoxAffectsLineBox(const InlineLevelBox& inlineLevelBox, const LineBox& lineBox) const
{
    if (!inlineLevelBox.mayStretchLineBox())
        return false;

    if (inlineLevelBox.isLineBreakBox()) {
        // A line break box affects the line box when it has a non-default
        // line-height (e.g. br { line-height: 200px }).
        if (inlineLevelBox.isPreferredLineHeightFontMetricsBased())
            return false;
        return formattingContext().layoutState().inStandardsMode() ? true : InlineQuirks::lineBreakBoxIsOnlyContentOnLine(lineBox);
    }
    if (inlineLevelBox.isListMarker()) {
        // This does not match other browser engines. see webkit.org/b/256390.
        return true;
    }
    if (inlineLevelBox.isInlineBox())
        return formattingContext().layoutState().inStandardsMode() ? true : formattingContext().quirks().inlineBoxAffectsLineBox(inlineLevelBox);
    if (inlineLevelBox.isAtomicInlineBox())
        return !inlineLevelBox.layoutBox().isRubyAnnotationBox();
    return false;
}

InlineRect InlineFormattingUtils::flipVisualRectToLogicalForWritingMode(const InlineRect& visualRect, WritingMode writingMode)
{
    switch (writingMode.blockDirection()) {
    case FlowDirection::TopToBottom:
    case FlowDirection::BottomToTop:
        return visualRect;
    case FlowDirection::LeftToRight:
    case FlowDirection::RightToLeft:
        // FIXME: While vertical-lr and vertical-rl modes do differ in the ordering direction of line boxes
        // in a block container (see: https://drafts.csswg.org/css-writing-modes/#block-flow)
        // we ignore it for now as RenderBlock takes care of it for us.
        return InlineRect { visualRect.left(), visualRect.top(), visualRect.height(), visualRect.width() };
    }
    ASSERT_NOT_REACHED();
    return visualRect;
}

InlineLayoutUnit InlineFormattingUtils::computedTextIndentForFirstLine(const ElementBox& root, InlineLayoutUnit availableWidth)
{
    auto shouldIndent = root.style().textIndent().eachLine.has_value() || (root.isAnonymousTextIndentCandidateForIntegration() || !root.isAnonymous() || (!root.isInlineIntegrationRoot() && root.parent().firstInFlowChild() == &root));
    // Specifying 'hanging' inverts whether the line should be indented or not.
    if (shouldIndent == root.style().textIndent().hanging.has_value())
        return { };
    auto& textIndentAmount = root.style().textIndent().amount;
    return textIndentAmount == 0_css_px ? 0.f : Style::evaluate<InlineLayoutUnit>(textIndentAmount, availableWidth, root.style().usedZoomForLength());
}

InlineLayoutUnit InlineFormattingUtils::computedTextIndent(IsIntrinsicWidthMode isIntrinsicWidthMode, IsFirstFormattedLine isFirstFormattedLine, std::optional<PreviousLineEndsParagraph> previousLineEndsParagraph, InlineLayoutUnit availableWidth) const
{
    CheckedRef root = formattingContext().root();

    // text-indent property specifies the indentation applied to lines of inline content in a block.
    // The indent is treated as a margin applied to the start edge of the line box.
    // The first formatted line of an element is always indented. For example, the first line of an anonymous block box
    // is only affected if it is the first child of its parent element.
    // If 'each-line' is specified, indentation also applies to all lines where the previous line ends with a hard break.
    // [Integration] root()->parent() would normally produce a valid layout box.
    auto shouldIndent = false;
    if (root->style().textIndent().eachLine.has_value())
        shouldIndent = isFirstFormattedLine == IsFirstFormattedLine::Yes || (previousLineEndsParagraph && *previousLineEndsParagraph == PreviousLineEndsParagraph::Yes);
    else if (root->isAnonymousTextIndentCandidateForIntegration()
        || !root->isAnonymous()
        || (!root->isInlineIntegrationRoot() && root->parent().firstInFlowChild() == root.ptr()))
            shouldIndent = isFirstFormattedLine == IsFirstFormattedLine::Yes;

    // Specifying 'hanging' inverts whether the line should be indented or not.
    if (root->style().textIndent().hanging.has_value())
        shouldIndent = !shouldIndent;

    if (!shouldIndent)
        return { };

    auto& textIndentAmount = root->style().textIndent().amount;
    if (textIndentAmount == 0_css_px)
        return { };
    if (isIntrinsicWidthMode == IsIntrinsicWidthMode::Yes && textIndentAmount.isPercentOrCalculated()) {
        // Percentages and calc() expressions containing percentages must be treated as 0
        // for the purpose of calculating intrinsic size contributions, with a zero percentage
        // basis so fixed-length components in calc() are still preserved.
        // https://drafts.csswg.org/css-text/#text-indent-property
        return Style::evaluate<InlineLayoutUnit>(textIndentAmount, 0, root->style().usedZoomForLength());
    }
    return Style::evaluate<InlineLayoutUnit>(textIndentAmount, availableWidth, root->style().usedZoomForLength());
}

InlineLayoutUnit InlineFormattingUtils::initialLineHeight(bool isFirstLine) const
{
    if (formattingContext().layoutState().inStandardsMode())
        return isFirstLine ? formattingContext().root().firstLineStyle().usedLineHeight() : formattingContext().root().style().usedLineHeight();
    return formattingContext().quirks().initialLineHeight();
}

FloatingContext::Constraints InlineFormattingUtils::floatConstraintsForLine(InlineLayoutUnit lineLogicalTop, InlineLayoutUnit contentLogicalHeight, const FloatingContext& floatingContext) const
{
    auto logicalTopCandidate = LayoutUnit { lineLogicalTop };
    auto logicalBottomCandidate = LayoutUnit { lineLogicalTop + contentLogicalHeight };
    if (logicalTopCandidate.mightBeSaturated() || logicalBottomCandidate.mightBeSaturated())
        return { };
    // Check for intruding floats and adjust logical left/available width for this line accordingly.
    return floatingContext.constraints(logicalTopCandidate, logicalBottomCandidate, FloatingContext::MayBeAboveLastFloat::Yes);
}

InlineLayoutUnit InlineFormattingUtils::horizontalAlignmentOffset(const Style::ComputedStyle& rootStyle, InlineLayoutUnit contentLogicalRight, InlineLayoutUnit lineLogicalWidth, InlineLayoutUnit hangingTrailingWidth, bool isLastLineOrLineEndsWithForcedLineBreak, std::optional<TextDirection> inlineBaseDirectionOverride)
{
    // Depending on the line's alignment/justification, the hanging glyph can be placed outside the line box.
    if (hangingTrailingWidth) {
        // If white-space is set to pre-wrap, the UA must (unconditionally) hang this sequence, unless the sequence is followed
        // by a forced line break, in which case it must conditionally hang the sequence is instead.
        // Note that end of last line in a paragraph is considered a forced break.
        auto isConditionalHanging = isLastLineOrLineEndsWithForcedLineBreak;
        // In some cases, a glyph at the end of a line can conditionally hang: it hangs only if it does not otherwise fit in the line prior to justification.
        if (isConditionalHanging) {
            // FIXME: Conditional hanging needs partial overflow trimming at glyph boundary, one by one until they fit.
            contentLogicalRight = std::min(contentLogicalRight, lineLogicalWidth);
        } else
            contentLogicalRight -= hangingTrailingWidth;
    }

    auto horizontalAvailableSpace = lineLogicalWidth - contentLogicalRight;

    if (horizontalAvailableSpace <= 0)
        return { };

    auto isLeftToRightDirection = inlineBaseDirectionOverride.value_or(rootStyle.writingMode().bidiDirection()) == TextDirection::LTR;

    auto computedHorizontalAlignment = [&] {
        auto textAlign = rootStyle.textAlign();
        if (!isLastLineOrLineEndsWithForcedLineBreak)
            return textAlign;
        // The last line before a forced break or the end of the block is aligned according to text-align-last.
        switch (rootStyle.textAlignLast()) {
        case Style::TextAlignLast::Auto:
            if (textAlign == Style::TextAlign::Justify)
                return Style::TextAlign::Start;
            return textAlign;
        case Style::TextAlignLast::Start:
            return Style::TextAlign::Start;
        case Style::TextAlignLast::End:
            return Style::TextAlign::End;
        case Style::TextAlignLast::Left:
            return Style::TextAlign::Left;
        case Style::TextAlignLast::Right:
            return Style::TextAlign::Right;
        case Style::TextAlignLast::Center:
            return Style::TextAlign::Center;
        case Style::TextAlignLast::Justify:
            return Style::TextAlign::Justify;
        default:
            ASSERT_NOT_REACHED();
            return Style::TextAlign::Start;
        }
    };

    switch (computedHorizontalAlignment()) {
    case Style::TextAlign::Left:
    case Style::TextAlign::WebKitLeft:
        if (!isLeftToRightDirection)
            return horizontalAvailableSpace;
        [[fallthrough]];
    case Style::TextAlign::Start:
        return { };
    case Style::TextAlign::Right:
    case Style::TextAlign::WebKitRight:
        if (!isLeftToRightDirection)
            return { };
        [[fallthrough]];
    case Style::TextAlign::End:
        return horizontalAvailableSpace;
    case Style::TextAlign::Center:
    case Style::TextAlign::WebKitCenter:
        return horizontalAvailableSpace / 2;
    case Style::TextAlign::Justify:
        // Style::TextAlign::Justify is a run alignment (and we only do inline box alignment here)
        return { };
    default:
        ASSERT_NOT_IMPLEMENTED_YET();
        return { };
    }
    ASSERT_NOT_REACHED();
    return { };
}

InlineItemPosition InlineFormattingUtils::leadingInlineItemPositionForNextLine(InlineItemPosition lineContentEnd, std::optional<InlineItemPosition> previousLineContentEnd, bool lineHasIntrusiveOrNewlyPlacedFloat, InlineItemPosition layoutRangeEnd)
{
    if (!previousLineContentEnd)
        return lineContentEnd;
    if (previousLineContentEnd->index < lineContentEnd.index || (previousLineContentEnd->index == lineContentEnd.index && previousLineContentEnd->offset < lineContentEnd.offset)) {
        // Either full or partial advancing.
        return lineContentEnd;
    }
    if (lineContentEnd == *previousLineContentEnd && lineHasIntrusiveOrNewlyPlacedFloat) {
        // Couldn't manage to put any content on line due to floats.
        return lineContentEnd;
    }
    if (lineContentEnd == layoutRangeEnd) {
        // End of content.
        return layoutRangeEnd;
    }
    // This looks like a partial content and we are stuck. Let's force-move over to the next inline item.
    // We certainly lose some content, but we would be busy looping otherwise.
    ASSERT_NOT_REACHED();
    return { std::min(lineContentEnd.index + 1, layoutRangeEnd.index), { } };
}

InlineLayoutUnit InlineFormattingUtils::inlineItemWidth(const InlineItem& inlineItem, InlineLayoutUnit contentLogicalLeft, bool useFirstLineStyle) const
{
    if (auto* inlineTextItem = dynamicDowncast<InlineTextItem>(inlineItem)) {
        if (auto contentWidth = inlineTextItem->width())
            return *contentWidth;
        CheckedRef fontCascade = useFirstLineStyle ? inlineTextItem->firstLineStyle().fontCascade() : inlineTextItem->style().fontCascade();
        if (!inlineTextItem->isWhitespace() || InlineTextItem::shouldPreserveSpacesAndTabs(*inlineTextItem))
            return TextUtil::width(*inlineTextItem, fontCascade, contentLogicalLeft);
        return TextUtil::width(*inlineTextItem, fontCascade, inlineTextItem->start(), inlineTextItem->start() + 1, contentLogicalLeft);
    }

    if (inlineItem.isLineBreak() || inlineItem.isWordBreakOpportunity())
        return { };

    CheckedRef layoutBox = inlineItem.layoutBox();
    auto& boxGeometry = formattingContext().geometryForBox(layoutBox.get());

    if (layoutBox->isReplacedBox())
        return boxGeometry.marginBoxWidth();

    if (inlineItem.isInlineBoxStart())
        return boxGeometry.marginStart() + boxGeometry.borderStart() + boxGeometry.paddingStart();

    if (inlineItem.isInlineBoxEnd())
        return boxGeometry.marginEnd() + boxGeometry.borderEnd() + boxGeometry.paddingEnd();

    if (inlineItem.isOutOfFlow())
        return { };

    if (inlineItem.isBlock())
        return boxGeometry.marginBoxWidth();

    // Non-replaced inline box (e.g. inline-block)
    return boxGeometry.marginBoxWidth();
}

static inline bool endsWithSoftWrapOpportunity(const InlineTextItem& previousInlineTextItem, const InlineTextItem& nextInlineTextItem)
{
    ASSERT(!nextInlineTextItem.isWhitespace());
    // We are at the position after a whitespace.
    if (previousInlineTextItem.isWhitespace())
        return true;
    // When both these non-whitespace runs belong to the same layout box with the same bidi level, it's guaranteed that
    // they are split at a soft breaking opportunity. See InlineItemsBuilder::moveToNextBreakablePosition.
    if (&previousInlineTextItem.inlineTextBox() == &nextInlineTextItem.inlineTextBox()) {
        if (previousInlineTextItem.bidiLevel() == nextInlineTextItem.bidiLevel())
            return true;
        // The bidi boundary may or may not be the reason for splitting the inline text box content.
        // FIXME: We could add a "reason flag" to InlineTextItem to tell why the split happened.
        CheckedRef style = previousInlineTextItem.style();
        auto lineBreakIteratorFactory = CachedLineBreakIteratorFactory { previousInlineTextItem.inlineTextBox().content(), Style::toPlatform(style->usedLocale()), TextUtil::lineBreakIteratorMode(style->lineBreak()), TextUtil::contentAnalysis(style->wordBreak()) };
        auto softWrapOpportunityCandidate = nextInlineTextItem.start();
        return TextUtil::findNextBreakablePosition(lineBreakIteratorFactory, softWrapOpportunityCandidate, style.get()) == softWrapOpportunityCandidate;
    }
    return TextUtil::mayBreakInBetween(previousInlineTextItem, nextInlineTextItem);
}

static inline const ElementBox& nearestCommonAncestor(const Box& first, const Box& second, const ElementBox& rootBox)
{
    SUPPRESS_UNCHECKED_LOCAL auto& firstParent = first.parent();
    SUPPRESS_UNCHECKED_LOCAL auto& secondParent = second.parent();
    // Cover a few common cases first.
    // 'some content'
    if (&firstParent == &secondParent)
        return firstParent;
    // some<span>content</span>
    if (&secondParent != &rootBox && &secondParent.parent() == &firstParent)
        return firstParent;
    // <span>some</span>content
    if (&firstParent != &rootBox && &firstParent.parent() == &secondParent)
        return secondParent;
    // <span>some</span><span>content</span>
    if (&firstParent != &rootBox && &secondParent != &rootBox && &firstParent.parent() == &secondParent.parent())
        return firstParent.parent();

    HashSet<CheckedRef<const ElementBox>> descendantsSet;
    for (SUPPRESS_UNCHECKED_LOCAL auto* descendant = &firstParent; descendant != &rootBox; descendant = &descendant->parent())
        descendantsSet.add(*descendant);
    for (SUPPRESS_UNCHECKED_LOCAL auto* descendant = &secondParent; descendant != &rootBox; descendant = &descendant->parent()) {
        if (!descendantsSet.add(*descendant).isNewEntry)
            return *descendant;
    }
    return rootBox;
}

bool InlineFormattingUtils::isAtSoftWrapOpportunity(const InlineItem& previous, const InlineItem& next) const
{
    // FIXME: Transition no-wrapping logic from InlineContentBreaker to here where we compute the soft wrap opportunity indexes.
    // "is at" simple means that there's a soft wrap opportunity right after the [previous].
    // [text][ ][text][inline box start]... (<div>text content<span>..</div>)
    // soft wrap indexes: 0 and 1 definitely, 2 depends on the content after the [inline box start].

    // https://www.w3.org/TR/css-text-4/#line-break-details
    // Figure out if the new incoming content puts the uncommitted content on a soft wrap opportunity.
    // e.g. [inline box start][prior_continuous_content][inline box end] (<span>prior_continuous_content</span>)
    // An incoming <img> box would enable us to commit the "<span>prior_continuous_content</span>" content
    // but an incoming text content would not necessarily.
    ASSERT(previous.isText() || previous.isAtomicInlineBox() || previous.layoutBox().isRubyInlineBox());
    ASSERT(next.isText() || next.isAtomicInlineBox() || next.layoutBox().isRubyInlineBox());

    auto mayWrapPrevious = TextUtil::isWrappingAllowed(previous.layoutBox().parent().style());
    auto mayWrapNext = TextUtil::isWrappingAllowed(next.layoutBox().parent().style());
    if (&previous.layoutBox().parent() == &next.layoutBox().parent() && !mayWrapPrevious && !mayWrapNext)
        return false;

    auto isMarkerContent = [](auto& inlineItem) {
        return inlineItem.layoutBox().parent().style().isListMarkerStyle();
    };
    if (isMarkerContent(previous) && !isMarkerContent(next))
        return TextUtil::isWrappingAllowed(nearestCommonAncestor(previous.layoutBox(), next.layoutBox(), formattingContext().root()).style());

    if (is<InlineTextItem>(previous) && is<InlineTextItem>(next)) {
        auto& previousInlineTextItem = uncheckedDowncast<InlineTextItem>(previous);
        auto& nextInlineTextItem = uncheckedDowncast<InlineTextItem>(next);
        if (previousInlineTextItem.isWhitespace() || nextInlineTextItem.isWhitespace()) {
            // For soft wrap opportunities created by characters that disappear at the line break (e.g. U+0020 SPACE), properties on the box directly
            // containing that character control the line breaking at that opportunity.
            // "<nowrap> </nowrap>after"
            if (previousInlineTextItem.isWhitespace())
                return mayWrapPrevious;

            // "<span>before</span><nowrap> </nowrap>"
            if (!mayWrapNext)
                return false;
            // 'white-space: break-spaces' and '-webkit-line-break: after-white-space': line breaking opportunity exists after every preserved white space character, but not before.
            auto& style = nextInlineTextItem.style();
            return style.whiteSpaceCollapse() != WhiteSpaceCollapse::BreakSpaces && style.lineBreak() != LineBreak::AfterWhiteSpace;
        }
        if (previous.style().lineBreak() == LineBreak::Anywhere || next.style().lineBreak() == LineBreak::Anywhere) {
            // which elements’ line-break, word-break, and overflow-wrap properties control the determination of soft wrap opportunities at such boundaries is undefined in this level.
            // There is a soft wrap opportunity around every typographic character unit, including around any punctuation character or preserved white spaces, or in the middle of words.
            return true;
        }
        // Both previous and next items are non-whitespace text.
        // [text][text] : is a continuous content.
        // [text-][text] : after [hyphen] position is a soft wrap opportunity.
        auto previousAndNextHaveSameParent = &previousInlineTextItem.layoutBox().parent() == &nextInlineTextItem.layoutBox().parent();
        if (previousAndNextHaveSameParent && !TextUtil::isWrappingAllowed(previousInlineTextItem.style()))
            return false;
        // For soft wrap opportunities defined by the boundary between two characters, the white-space property on the nearest common ancestor of the two characters controls breaking.
        if (!endsWithSoftWrapOpportunity(previousInlineTextItem, nextInlineTextItem))
            return false;
        return TextUtil::isWrappingAllowed(nearestCommonAncestor(previousInlineTextItem.layoutBox(), nextInlineTextItem.layoutBox(), formattingContext().root()).style());
    }
    if (previous.layoutBox().isListMarkerBox()) {
        // An outside marker is not on the line's content flow, so it offers nothing to break after.
        return false;
    }
    if (next.layoutBox().isListMarkerBox()) {
        // FIXME: SHould this ever be the case?
        return true;
    }
    if (previous.isAtomicInlineBox() || next.isAtomicInlineBox()) {
        // [text][inline box start][inline box end][inline box] (text<span></span><img>) : there's a soft wrap opportunity between the [text] and [img].
        // The line breaking behavior of a replaced element or other atomic inline is equivalent to an ideographic character.
        return true;
    }

    ASSERT_NOT_REACHED();
    return true;
}

size_t InlineFormattingUtils::nextWrapOpportunity(size_t startIndex, const InlineItemRange& layoutRange, std::span<const InlineItem> inlineItemList) const
{
    // 1. Find the start candidate by skipping leading non-content items e.g "<span><span>start". Opportunity is after "<span><span>".
    // 2. Find the end candidate by skipping non-content items inbetween e.g. "<span><span>start</span>end". Opportunity is after "</span>".
    // 3. Check if there's a soft wrap opportunity between the 2 candidate inline items and repeat.
    // 4. Any force line break/explicit wrap content inbetween is considered as wrap opportunity.

    // [ex-][inline box start][inline box end][float][ample] (ex-<span></span><div style="float:left"></div>ample). Wrap index is at [ex-].
    // [ex][inline box start][amp-][inline box start][le] (ex<span>amp-<span>ample). Wrap index is at [amp-].
    // [ex-][inline box start][line break][ample] (ex-<span><br>ample). Wrap index is after [br].
    auto previousInlineItemIndex = std::optional<size_t> { };
    for (auto index = startIndex; index < layoutRange.endIndex(); ++index) {
        auto& currentItem = inlineItemList[index];
        if (currentItem.isLineBreak() || currentItem.isWordBreakOpportunity()) {
            // We always stop at explicit wrapping opportunities e.g. <br>. However the wrap position may be at later position.
            // e.g. <span><span><br></span></span> <- wrap position is after the second </span>
            // but in case of <span><br><span></span></span> <- wrap position is right after <br>.
            for (++index; index < layoutRange.endIndex() && inlineItemList[index].isInlineBoxEnd(); ++index) { }
            return index;
        }
        auto isInlineBox = currentItem.isInlineBoxStart() || currentItem.isInlineBoxEnd();
        if (isInlineBox) {
            // Need to see what comes next to decide.
            continue;
        }
        if (currentItem.isOutOfFlow()) {
            // This item is invisible to line breaking. Need to pretend it's not here.
            continue;
        }
        if (currentItem.isBlock()) {
            // Let's break before and after a block level box.
            auto wrappingPosition = index == startIndex ? std::min(index + 1, layoutRange.endIndex()) : index;
            return wrappingPosition;
        }
        ASSERT(currentItem.isText() || currentItem.isAtomicInlineBox() || currentItem.isFloat() || currentItem.layoutBox().isRubyInlineBox());
        if (currentItem.isFloat()) {
            // While floats are not part of the inline content and they are not supposed to introduce soft wrap opportunities,
            // e.g. [text][float box][float box][text][float box][text] is essentially just [text][text][text]
            // figuring out whether a float (or set of floats) should stay on the line or not (and handle potentially out of order inline items)
            // brings in unnecessary complexity.
            // For now let's always treat a float as a soft wrap opportunity.
            auto wrappingPosition = index == startIndex ? std::min(index + 1, layoutRange.endIndex()) : index;
            return wrappingPosition;
        }
        if (!previousInlineItemIndex) {
            previousInlineItemIndex = index;
            continue;
        }
        // At this point previous and current items are not necessarily adjacent items e.g "previous<span>current</span>"
        auto& previousItem = inlineItemList[*previousInlineItemIndex];
        if (isAtSoftWrapOpportunity(previousItem, currentItem)) {
            if (*previousInlineItemIndex + 1 == index && (!previousItem.isText() || !currentItem.isText())) {
                // We only know the exact soft wrap opportunity index when the previous and current items are next to each other.
                return index;
            }
            // There's a soft wrap opportunity between 'previousInlineItemIndex' and 'index'.
            // Now forward-find from the start position to see where we can actually wrap.
            // [ex-][ample] vs. [ex-][inline box start][inline box end][ample]
            // where [ex-] is previousInlineItemIndex and [ample] is index.

            // inline content and their inline boxes form unbreakable content.
            // ex-<span></span>ample               : wrap opportunity is after "ex-<span></span>".
            // ex-<span>ample                      : wrap opportunity is after "ex-".
            // ex-<span><span></span></span>ample  : wrap opportunity is after "ex-<span><span></span></span>".
            // ex-</span></span>ample              : wrap opportunity is after "ex-</span></span>".
            // ex-</span><span>ample               : wrap opportunity is after "ex-</span>".
            // ex-<span><span>ample                : wrap opportunity is after "ex-".
            struct InlineBoxPosition {
                const Box* inlineBox { nullptr };
                size_t index { 0 };
            };
            Vector<InlineBoxPosition> inlineBoxStack;
            auto start = *previousInlineItemIndex;
            auto end = index;
            // Soft wrap opportunity is at the first inline box that encloses the trailing content.
            for (auto candidateIndex = start + 1; candidateIndex < end; ++candidateIndex) {
                auto& inlineItem = inlineItemList[candidateIndex];
                ASSERT(inlineItem.isInlineBoxStartOrEnd() || inlineItem.isOutOfFlow());
                if (inlineItem.isInlineBoxStart())
                    inlineBoxStack.append({ &inlineItem.layoutBox(), candidateIndex });
                else if (inlineItem.isInlineBoxEnd() && !inlineBoxStack.isEmpty())
                    inlineBoxStack.removeLast();
            }
            return inlineBoxStack.isEmpty() ? index : inlineBoxStack.first().index;
        }
        previousInlineItemIndex = index;
    }
    return layoutRange.endIndex();
}

std::pair<InlineLayoutUnit, InlineLayoutUnit> InlineFormattingUtils::textEmphasisForInlineBox(const Box& layoutBox, const ElementBox& rootBox)
{
    // Generic, non-inline box inline-level content (e.g. replaced elements) can't have text-emphasis annotations.
    ASSERT(layoutBox.isInlineBox() || &layoutBox == &rootBox);

    CheckedRef style = layoutBox.style();
    auto hasTextEmphasis =  !style->textEmphasisStyle().isNone();
    if (!hasTextEmphasis)
        return { };
    auto emphasisPosition = style->textEmphasisPosition();
    // Normally we resolve visual -> logical values at pre-layout time, but emphasis values are not part of the general box geometry.
    auto hasAboveTextEmphasis = false;
    auto hasUnderTextEmphasis = false;
    if (style->writingMode().isVerticalTypographic()) {
        hasAboveTextEmphasis = !emphasisPosition.contains(Style::TextEmphasisPositionValue::Left);
        hasUnderTextEmphasis = !hasAboveTextEmphasis;
    } else {
        hasAboveTextEmphasis = !emphasisPosition.contains(Style::TextEmphasisPositionValue::Under);
        hasUnderTextEmphasis = !hasAboveTextEmphasis;
    }

    if (!hasAboveTextEmphasis && !hasUnderTextEmphasis)
        return { };

    auto enclosingRubyBase = [&]() -> const ElementBox* {
        if (&layoutBox == &rootBox)
            return nullptr;
        for (auto* ancestor = &layoutBox.parent(); ancestor != &rootBox; ancestor = &ancestor->parent()) {
            if (ancestor->isRubyBase())
                return ancestor;
        }
        return nullptr;
    };
    if (CheckedPtr rubyBase = enclosingRubyBase(); rubyBase && RubyFormattingContext::hasInterlinearAnnotation(*rubyBase)) {
        auto annotationPosition = rubyBase->style().rubyPosition();
        if ((hasAboveTextEmphasis && annotationPosition == RubyPosition::Over) || (hasUnderTextEmphasis && annotationPosition == RubyPosition::Under)) {
            // FIXME: Check if annotation box has content.
            return { };
        }
    }
    auto annotationSize = style->fontCascade().floatEmphasisMarkHeight(style->textEmphasisStyle().markString());
    return { hasAboveTextEmphasis ? annotationSize : 0.f, hasAboveTextEmphasis ? 0.f : annotationSize };
}

LineEndingTruncationPolicy InlineFormattingUtils::lineEndingTruncationPolicy(const Style::ComputedStyle& rootStyle, size_t numberOfContentfulLines, std::optional<size_t> numberOfVisibleLinesAllowed, bool currentLineIsContentful)
{
    if (numberOfVisibleLinesAllowed) {
        if (!currentLineIsContentful) {
            // Content with no inline should never ever receive ellipsis.
            return LineEndingTruncationPolicy::NoTruncation;
        }
        if (numberOfContentfulLines >= *numberOfVisibleLinesAllowed) {
            // The clamped line's block ellipsis replaces the text-overflow ellipsis, while the lines below it are not visible at all.
            return numberOfContentfulLines == *numberOfVisibleLinesAllowed ? LineEndingTruncationPolicy::WhenContentOverflowsInBlockDirection : LineEndingTruncationPolicy::NoTruncation;
        }
        // Lines above the clamp point are not affected by clamping, so text-overflow is what may truncate them.
    }

    // Truncation is in effect when the block container has overflow other than visible.
    if (rootStyle.overflowX() != Overflow::Visible && !rootStyle.textOverflow().isClip())
        return LineEndingTruncationPolicy::WhenContentOverflowsInInlineDirection;
    return LineEndingTruncationPolicy::NoTruncation;
}

std::optional<LineLayoutResult::InlineContentEnding> InlineFormattingUtils::inlineContentEnding(const Line::Result& lineContent)
{
    if (!lineContent.runs.isEmpty() && lineContent.runs.last().isBlock()) {
#if ASSERT_ENABLED
    for (auto& run : lineContent.runs)
        ASSERT(run.isLineSpanningInlineBoxStart() || run.isBlock());
#endif
        return { };
    }

    for (auto& run : lineContent.runs | std::views::reverse) {
        ASSERT(!run.isBlock());
        if (run.isOutOfFlow())
            continue;
        if (run.isLineBreak())
            return { LineLayoutResult::InlineContentEnding::LineBreak };
        if (run.isText() && run.textContent().needsHyphen)
            return { LineLayoutResult::InlineContentEnding::Hyphen };
        return { LineLayoutResult::InlineContentEnding::Generic };
    }
    return { };
}

bool InlineFormattingUtils::shouldDiscardRemainingContentInBlockDirection() const
{
    auto& inlineLayoutState = formattingContext().layoutState();
    auto lineClamp = inlineLayoutState.parentBlockLayoutState().lineClamp();
    if (!lineClamp || !lineClamp->shouldDiscardOverflow)
        return false;
    ASSERT(!lineClamp->isLegacy);
    return lineClamp->maximumLines == inlineLayoutState.lineCountWithInlineContentIncludingNestedBlocks();
}

}
}

```

## InlineContentBreaker.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineContentBreaker.cpp). Path: `Source/WebCore/layout/formattingContexts/inline/InlineContentBreaker.cpp`. Source bytes: 55485; source lines: 991; SHA-256: `a44992fba45e29ecce10b77c0d541cba59d9a30ac9026c60d14f7e57a78589b4`.

```cpp
/*
 * Copyright (C) 2018-2023 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "InlineContentBreaker.h"

#include "FontCascade.h"
#include "Hyphenation.h"
#include "InlineItem.h"
#include "InlineTextItem.h"
#include "LayoutElementBox.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "TextUtil.h"
#include <wtf/unicode/CharacterNames.h>

namespace WebCore {
namespace Layout {

static inline bool NODELETE hasLeadingTextContent(const InlineContentBreaker::ContinuousContent& continuousContent)
{
    for (auto& run : continuousContent.runs()) {
        auto& inlineItem = run.inlineItem;
        if (inlineItem.isInlineBoxStartOrEnd() || inlineItem.isOutOfFlow())
            continue;
        return inlineItem.isText();
    }
    return false;
}

static inline std::optional<size_t> NODELETE nextTextRunIndex(const InlineContentBreaker::ContinuousContent::RunList& runs, size_t startIndex)
{
    for (auto index = startIndex + 1; index < runs.size(); ++index) {
        if (runs[index].inlineItem.isText())
            return index;
    }
    return { };
}

static inline bool isWhitespaceOnlyContent(const InlineContentBreaker::ContinuousContent& continuousContent)
{
    // [<span></span> ] [<span> </span>] [ <span style="padding: 0px;"></span>] are all considered visually empty whitespace content.
    // [<span style="border: 1px solid red"></span> ] while this is whitespace content only, it is not considered visually empty.
    ASSERT(!continuousContent.runs().isEmpty());
    auto hasWhitespace = false;
    for (auto& run : continuousContent.runs()) {
        auto& inlineItem = run.inlineItem;
        if (inlineItem.isInlineBoxStartOrEnd() || inlineItem.isOutOfFlow())
            continue;
        auto isWhitespace = [&] {
            auto* textItem = dynamicDowncast<InlineTextItem>(inlineItem);
            return textItem && textItem->isWhitespace();
        }();
        if (!isWhitespace)
            return false;
        hasWhitespace = true;
    }
    return hasWhitespace;
}

static inline bool NODELETE isNonContentRunsOnly(const InlineContentBreaker::ContinuousContent& continuousContent)
{
    // <span></span> <- non content runs.
    for (auto& run : continuousContent.runs()) {
        auto& inlineItem = run.inlineItem;
        if (inlineItem.isInlineBoxStartOrEnd() || inlineItem.isOutOfFlow())
            continue;
        if (auto* inlineTextItem = dynamicDowncast<InlineTextItem>(inlineItem); inlineTextItem && inlineTextItem->isEmpty())
            continue;
        return false;
    }
    return true;
}

static inline std::optional<size_t> NODELETE firstTextRunIndex(const InlineContentBreaker::ContinuousContent::RunList& continuousContentRuns)
{
    for (size_t index = 0; index < continuousContentRuns.size(); ++index) {
        if (continuousContentRuns[index].inlineItem.isText())
            return index;
    }
    return { };
}

InlineContentBreaker::Result InlineContentBreaker::processInlineContent(const ContinuousContent& candidateContent, const LineStatus& lineStatus)
{
    ASSERT(!std::isnan(lineStatus.availableWidth));
    ASSERT(isMinimumInIntrinsicWidthMode() || candidateContent.logicalWidth() > lineStatus.availableWidth);

    if (auto result = simplifiedMinimumIntrinsicWidthBreak(candidateContent, lineStatus))
        return *result;

    auto result = processOverflowingContent(candidateContent, lineStatus);
    if (result.action == Result::Action::Wrap && lineStatus.trailingSoftHyphenWidth && hasLeadingTextContent(candidateContent)) {
        // A trailing soft hyphen with a wrapped text content turns into a visible hyphen.
        // Let's check if there's enough space for the hyphen character.
        auto hyphenOverflows = *lineStatus.trailingSoftHyphenWidth > lineStatus.availableWidth;
        auto action = hyphenOverflows ? Result::Action::RevertToLastNonOverflowingWrapOpportunity : Result::Action::WrapWithHyphen;
        result = { action, IsEndOfLine::Yes };
    }
    return result;
}

static inline bool canBreakBefore(char32_t character, LineBreak lineBreak)
{
    // FIXME: This should include all the cases from https://unicode.org/reports/tr14
    // Use a breaking matrix similar to lineBreakTable in BreakablePositions.cpp
    // Also see kBreakAllLineBreakClassTable in third_party/blink/renderer/platform/text/text_break_iterator.cc
    if (lineBreak != LineBreak::Loose) {
        if (character == hyphen || character == enDash)
            return false;
    }
    if (character == noBreakSpace)
        return false;
    auto isPunctuation = U_GET_GC_MASK(character) & (U_GC_PS_MASK | U_GC_PE_MASK | U_GC_PI_MASK | U_GC_PF_MASK | U_GC_PO_MASK);
    return character == reverseSolidus || !isPunctuation;
}

static inline InlineContentBreaker::PartialRun firstCharacterBreakRespectingLineStartProhibitions(const InlineTextItem& inlineTextItem, const InlineContentBreaker::ContinuousContent::Run& textRun, InlineLayoutUnit contentLogicalRight)
{
    auto firstCharacterLength = TextUtil::firstUserPerceivedCharacterLength(inlineTextItem);
    auto firstCharacterWidth = TextUtil::width(inlineTextItem, textRun.style.fontCascade(), inlineTextItem.start(), inlineTextItem.start() + firstCharacterLength, contentLogicalRight);
    if (inlineTextItem.inlineTextBox().content().is8Bit())
        return { firstCharacterLength, firstCharacterWidth };

    auto breakPosition = firstCharacterLength;
    auto breakWidth = firstCharacterWidth;
    auto text = inlineTextItem.inlineTextBox().content();
    while (inlineTextItem.start() + breakPosition < inlineTextItem.end()) {
        if (canBreakBefore(text[inlineTextItem.start() + breakPosition], textRun.style.lineBreak()))
            break;
        auto nextPosition = breakPosition;
        U16_FWD_1(text, nextPosition, inlineTextItem.end() - inlineTextItem.start());
        breakWidth = TextUtil::width(inlineTextItem, textRun.style.fontCascade(), inlineTextItem.start(), inlineTextItem.start() + nextPosition, contentLogicalRight);
        breakPosition = nextPosition;
    }
    return { breakPosition, breakWidth };
}

static bool shouldRevertToEarlierWrapOpportunity(const InlineContentBreaker::LineStatus& lineStatus, const InlineContentBreaker::ContinuousContent::RunList& runs, size_t overflowingRunIndex)
{
    // https://drafts.csswg.org/css-text-4/#wrap-inside
    // 'wrap-inside: avoid' suppresses the soft wrap opportunities inside the box, including those introduced by
    // word-break: break-all and line-break: anywhere. Breaking inside the box is only a last resort, so when there
    // is an earlier wrap opportunity to fall back to, revert to it rather than ending the line inside the box.
    if (!lineStatus.hasWrapOpportunityAtPreviousPosition)
        return false;
    auto& overflowingBox = runs[overflowingRunIndex].inlineItem.layoutBox();
    auto& styleToUse = overflowingBox.isInlineBox() ? overflowingBox.style() : overflowingBox.parent().style();
    return styleToUse.effectiveWrapInsideAvoid();
}

InlineContentBreaker::Result InlineContentBreaker::processOverflowingContent(const ContinuousContent& continuousContent, const LineStatus& lineStatus) const
{
    ASSERT(!continuousContent.runs().isEmpty());

    ASSERT(continuousContent.logicalWidth() > lineStatus.availableWidth);
    auto checkForTrailingContentFit = [&]() -> std::optional<InlineContentBreaker::Result> {
        if (continuousContent.isFullyTrimmable()) {
            // fully trimmable content stays on the current line (and gets fully trimmed).
            return InlineContentBreaker::Result { Result::Action::Keep };
        }
        if (continuousContent.hasTrimmableSpace()) {
            // Check if the content fits if we trimmed it.
            if (isWhitespaceOnlyContent(continuousContent))
                return InlineContentBreaker::Result { Result::Action::Keep };
            auto spaceRequired = continuousContent.logicalWidth() - continuousContent.trailingTrimmableWidth();
            if (lineStatus.hasFullyTrimmableTrailingContent)
                spaceRequired -= continuousContent.leadingTrimmableWidth();
            if (spaceRequired <= lineStatus.availableWidth)
                return InlineContentBreaker::Result { Result::Action::Keep };
        }

        if (continuousContent.isHangingContent())
            return InlineContentBreaker::Result { Result::Action::Keep };
        if (continuousContent.hasHangingSpace()) {
            auto spaceRequired = continuousContent.logicalWidth() - continuousContent.hangingContentWidth();
            if (spaceRequired <= lineStatus.availableWidth)
                return InlineContentBreaker::Result { Result::Action::Keep };
        }

        auto canIgnoreNonContentTrailingRuns = lineStatus.trimmableOrHangingWidth && isNonContentRunsOnly(continuousContent);
        if (canIgnoreNonContentTrailingRuns) {
            // Let's see if the non-content runs fit when the line has trailing trimmable/hanging content.
            // "text content <span style="padding: 1px"></span>" <- the <span></span> runs could fit after trimming the trailing whitespace.
            if (continuousContent.logicalWidth() <= lineStatus.availableWidth + lineStatus.trimmableOrHangingWidth)
                return InlineContentBreaker::Result { Result::Action::Keep };
        }

        return { };
    };
    if (auto result = checkForTrailingContentFit())
        return *result;

    size_t overflowingRunIndex = 0;
    if (continuousContent.hasTextContent()) {
        auto tryBreakingContentWithText = [&]() -> std::optional<Result> {
            // 1. This text content is not breakable.
            // 2. This breakable text content does not fit at all. Not even the first glyph. This is a very special case.
            // 3. We can break the content but it still overflows.
            // 4. Managed to break the content before the overflow point.
            auto overflowingContent = processOverflowingContentWithText(continuousContent, lineStatus);
            overflowingRunIndex = overflowingContent.runIndex;
            if (!overflowingContent.breakingPosition)
                return { };
            if (shouldRevertToEarlierWrapOpportunity(lineStatus, continuousContent.runs(), overflowingRunIndex))
                return Result { Result::Action::RevertToLastWrapOpportunity, IsEndOfLine::Yes };
            auto trailingContent = overflowingContent.breakingPosition->trailingContent;
            if (!trailingContent) {
                // We tried to break the content but the available space can't even accommodate the first glyph.
                // 1. Wrap the content over to the next line when we've got content on the line already.
                // 2. Keep the first glyph on the empty line (or keep the whole run if it has only one glyph/completely empty)
                // including closing inline boxes e.g. <span><span>X</span></span> where "X" is the overflowing glyph).
                if (lineStatus.hasContent)
                    return Result { Result::Action::Wrap, IsEndOfLine::Yes };

                auto leadingTextRunIndex = *firstTextRunIndex(continuousContent.runs());
                auto& leadingTextRun = continuousContent.runs()[leadingTextRunIndex];
                auto& inlineTextItem = downcast<InlineTextItem>(leadingTextRun.inlineItem);
                auto firstCharacterLength = TextUtil::firstUserPerceivedCharacterLength(inlineTextItem);
                ASSERT(firstCharacterLength > 0);

                if (inlineTextItem.length() > firstCharacterLength) {
                    auto partialRun = firstCharacterBreakRespectingLineStartProhibitions(inlineTextItem, leadingTextRun, lineStatus.contentLogicalRight);
                    if (partialRun.length < inlineTextItem.length())
                        return Result { Result::Action::Break, IsEndOfLine::Yes, Result::PartialTrailingContent { leadingTextRunIndex, partialRun, { } } };
                }
                // Single character or line-start prohibitions consumed the entire item — keep it as one indivisible unit.
                auto trailingRunIndex = [&]() -> std::optional<size_t> {
                    // Keep the overflowing text content and the closing inline box runs together.
                    // e.g. X</span><span>Y</span> where "X" overflows, the trailing run index is 1.
                    auto& runs = continuousContent.runs();
                    if (leadingTextRunIndex == runs.size() - 1)
                        return { };
                    for (auto runIndex = leadingTextRunIndex + 1; runIndex < runs.size(); ++runIndex) {
                        auto& inlineItem = runs[runIndex].inlineItem;
                        if (inlineItem.isOutOfFlow())
                            continue;
                        if (!inlineItem.isInlineBoxEnd())
                            return runIndex - 1;
                    }
                    return { };
                };
                if (auto runToBreakAfter = trailingRunIndex())
                    return Result { Result::Action::Break, IsEndOfLine::Yes, Result::PartialTrailingContent { *runToBreakAfter, { }, { } } };
                return Result { Result::Action::Keep, IsEndOfLine::Yes };
            }
            if (trailingContent->overflows && lineStatus.hasContent) {
                // We managed to break a run with overflow but the line already has content. Let's wrap it to the next line.
                return Result { Result::Action::Wrap, IsEndOfLine::Yes };
            }
            // Either we managed to break with no overflow or the line is empty.
            auto trailingPartialContent = Result::PartialTrailingContent { overflowingContent.breakingPosition->runIndex, trailingContent->partialRun, trailingContent->hyphenWidth };
            return Result { Result::Action::Break, IsEndOfLine::Yes, trailingPartialContent };
        };
        if (auto result = tryBreakingContentWithText())
            return *result;
    } else if (continuousContent.runs().size() > 1) {
        // FIXME: Add support for various content.
        auto& runs = continuousContent.runs();
        for (size_t i = 0; i < runs.size(); ++i) {
            if (runs[i].inlineItem.isAtomicInlineBox()) {
                overflowingRunIndex = i;
                break;
            }
        }
    }

    // If we are not allowed to break this overflowing content, we still need to decide whether keep it or wrap it to the next line.
    if (!lineStatus.hasContent)
        return { Result::Action::Keep, IsEndOfLine::No };

    if (shouldRevertToEarlierWrapOpportunity(lineStatus, continuousContent.runs(), overflowingRunIndex))
        return { Result::Action::RevertToLastWrapOpportunity, IsEndOfLine::Yes };

    // Now either wrap this content over to the next line or revert back to an earlier wrapping opportunity, or not wrap at all.
    auto shouldWrapUnbreakableContentToNextLine = [&] {
        // The individual runs in this continuous content don't break, let's check if we are allowed to wrap this content to next line (e.g. pre would prevent us from wrapping).
        // Parent style drives the wrapping behavior here unless the overflowing run is an inline box.
        // In such cases decoration overflow is considered as "content" and we need to check the style accordingly.
        // e.g. <div style="white-space: nowrap">no wrap<div style="display: inline-block; white-space: normal">yes wrap</div></div>.
        // While the inline-block has pre-wrap which allows wrapping (for its own content), the content lives in a nowrap context.
        auto& runs = continuousContent.runs();
        auto& overflowingBox = runs[overflowingRunIndex].inlineItem.layoutBox();
        auto& styleToUse = overflowingBox.isInlineBox() ? overflowingBox.style() : overflowingBox.parent().style();
        auto isWrappingAllowed = TextUtil::isWrappingAllowed(styleToUse);
        for (auto index = overflowingRunIndex; !isWrappingAllowed && index--;) {
            auto& styleToUse = runs[index].inlineItem.layoutBox().parent().style();
            isWrappingAllowed = TextUtil::isWrappingAllowed(styleToUse);
        }
        return isWrappingAllowed;
    };
    if (shouldWrapUnbreakableContentToNextLine())
        return { Result::Action::Wrap, IsEndOfLine::Yes };
    if (lineStatus.hasWrapOpportunityAtPreviousPosition)
        return { Result::Action::RevertToLastWrapOpportunity, IsEndOfLine::Yes };
    return { Result::Action::Keep, IsEndOfLine::No };
}

std::optional<InlineContentBreaker::Result> InlineContentBreaker::simplifiedMinimumIntrinsicWidthBreak(const ContinuousContent& candidateContent, const LineStatus& lineStatus) const
{
    if (!isMinimumInIntrinsicWidthMode() || !candidateContent.isTextOnlyContent())
        return { };

    auto& leadingInlineTextItem = downcast<InlineTextItem>(candidateContent.runs().first().inlineItem);
    CheckedRef style = leadingInlineTextItem.style();
    if (!TextUtil::isWrappingAllowed(style))
        return Result { Result::Action::Keep, IsEndOfLine::No };

    if (!lineStatus.hasContent) {
        if (leadingInlineTextItem.isEmpty())
            return Result { Result::Action::Keep, IsEndOfLine::No };
        auto breakBehavior = wordBreakBehavior(style, { });
        if (breakBehavior.isEmpty())
            return Result { Result::Action::Keep, IsEndOfLine::No };

        if (breakBehavior.containsAny({ WordBreakRule::AtArbitraryPositionWithinWords, WordBreakRule::AtArbitraryPosition })) {
            auto firstCharacterLength = TextUtil::firstUserPerceivedCharacterLength(leadingInlineTextItem);
            if (leadingInlineTextItem.length() <= firstCharacterLength)
                return Result { Result::Action::Keep, IsEndOfLine::Yes };
            auto firstCharacterWidth = TextUtil::width(leadingInlineTextItem, style->fontCascade(), leadingInlineTextItem.start(), leadingInlineTextItem.start() + firstCharacterLength, { }, TextUtil::UseTrailingWhitespaceMeasuringOptimization::No);
            return Result { Result::Action::Break, IsEndOfLine::Yes, Result::PartialTrailingContent { { }, PartialRun { firstCharacterLength, firstCharacterWidth }, { } } };
        }
        return { };
    }
    return Result { !lineStatus.trailingSoftHyphenWidth ? Result::Action::Wrap : Result::Action::RevertToLastNonOverflowingWrapOpportunity, IsEndOfLine::Yes };
}

static std::optional<size_t> NODELETE findTrailingRunIndexBeforeBreakableRun(const InlineContentBreaker::ContinuousContent::RunList& runs, size_t breakableRunIndex)
{
    // When the breaking position is at the beginning of the run, the trailing run is the previous one.
    if (!breakableRunIndex)
        return { };
    // Try not break content at inline box boundary
    // e.g. <span>fits</span><span>overflows</span>
    // when the text "overflows" completely overflows, let's break the content right before the '<span>'.
    auto lastOutOfFlowItemIndex = std::optional<size_t> { };
    for (auto trailingCandidateIndex = breakableRunIndex; trailingCandidateIndex--;) {
        auto& inlineItem = runs[trailingCandidateIndex].inlineItem;
        if (inlineItem.isOutOfFlow()) {
            lastOutOfFlowItemIndex = trailingCandidateIndex;
            continue;
        }
        auto isAtInlineBox = inlineItem.isInlineBoxStart();
        if (!isAtInlineBox)
            return lastOutOfFlowItemIndex.value_or(trailingCandidateIndex);
        lastOutOfFlowItemIndex = { };
    }
    return { };
}

static bool NODELETE isBreakableRun(const InlineContentBreaker::ContinuousContent::Run& run)
{
    if (!run.inlineItem.isText()) {
        // Can't break horizontal spacing -> e.g. <span style="padding-right: 100px;">textcontent</span>, if the [inline box end] is the overflown inline item
        // we need to check if there's another inline item beyond the [inline box end] to split.
        return false;
    }
    // Check if this text run needs to stay on the current line.
    return TextUtil::isWrappingAllowed(run.style);
}

static inline std::optional<size_t> lastValidBreakingPosition(const InlineContentBreaker::ContinuousContent::RunList& runs, size_t textRunIndex)
{
    auto& textRun = runs[textRunIndex];
    auto& inlineTextItem = downcast<InlineTextItem>(textRun.inlineItem);
    ASSERT(inlineTextItem.length());
    auto lineBreak = textRun.style.lineBreak();

    auto lastValidBreakingPositionInsideTextRun = [&]() -> std::optional<size_t> {
        // Find out if the candidate position for arbitrary breaking is valid. We can't always break between any characters.
        auto text = inlineTextItem.inlineTextBox().content();
        auto left = inlineTextItem.start();
        for (auto index = inlineTextItem.end() - 1; index > left; --index) {
            U16_SET_CP_START(text, left, index);
            // We should never find surrogates/segments across inline items.
            ASSERT(index >= inlineTextItem.start());
            if (canBreakBefore(text[index], lineBreak))
                return index == inlineTextItem.start() ? std::nullopt : std::make_optional(index);
        }
        return { };
    };

    if (auto nextTextRunCandidateIndex = nextTextRunIndex(runs, textRunIndex)) {
        auto& nextInlineTextItem = downcast<InlineTextItem>(runs[*nextTextRunCandidateIndex].inlineItem);
        auto canBreakAtRunBoundary = nextInlineTextItem.isWhitespace() ? nextInlineTextItem.style().whiteSpaceCollapse() != WhiteSpaceCollapse::BreakSpaces :
            canBreakBefore(nextInlineTextItem.inlineTextBox().content()[nextInlineTextItem.start()], lineBreak);
        if (canBreakAtRunBoundary)
            return inlineTextItem.end();
        return lastValidBreakingPositionInsideTextRun();
    }

    if (textRunIndex == runs.size() - 1)
        return inlineTextItem.end();

    if (!runs[textRunIndex + 1].inlineItem.isInlineBoxStartOrEnd()) {
        ASSERT_NOT_REACHED();
        return inlineTextItem.end();
    }

    return lastValidBreakingPositionInsideTextRun();
}

static std::optional<TextUtil::WordBreakLeft> midWordBreak(const InlineContentBreaker::ContinuousContent::Run& textRun, InlineLayoutUnit runLogicalLeft, InlineLayoutUnit availableWidth)
{
    ASSERT(textRun.style.wordBreak() == WordBreak::BreakAll);
    auto& inlineTextItem = downcast<InlineTextItem>(textRun.inlineItem);

    auto wordBreak = TextUtil::breakWord(inlineTextItem, textRun.style.fontCascade(), textRun.spaceRequired(), availableWidth, runLogicalLeft);
    if (!wordBreak.length || wordBreak.length == inlineTextItem.length())
        return { };

    // Find out if the candidate position for arbitrary breaking is valid. We can't always break between any characters.
    auto lineBreak = textRun.style.lineBreak();
    auto text = inlineTextItem.inlineTextBox().content();
    if (canBreakBefore(text[inlineTextItem.start() + wordBreak.length], lineBreak))
        return wordBreak;

    const auto left = inlineTextItem.start();
    auto right = left + wordBreak.length;
    for (; right > left; --right) {
        U16_SET_CP_START(text, left, right);
        if (canBreakBefore(text[right], lineBreak))
            break;
    }
    if (left == right)
        return { };
    return TextUtil::WordBreakLeft { right - left, TextUtil::width(inlineTextItem, textRun.style.fontCascade(), left, right, runLogicalLeft) };
}

static size_t NODELETE limitBeforeValue(const Style::ComputedStyle& style)
{
    return style.hyphenateLimitBefore().tryValue().value_or(0).value;
}

static size_t NODELETE limitAfterValue(const Style::ComputedStyle& style)
{
    return style.hyphenateLimitAfter().tryValue().value_or(0).value;
}

static size_t NODELETE limitWordValue(const Style::ComputedStyle& style)
{
    return style.internalHyphenateLimitCharsWord().tryValue().value_or(0).value;
}

static inline bool NODELETE hasEnoughContentForHyphenation(size_t contentLength, const Style::ComputedStyle& style)
{
    return limitBeforeValue(style) + limitAfterValue(style) <= contentLength && limitWordValue(style) <= contentLength;
}

static std::optional<size_t> firstHyphenPosition(StringView content, const Style::ComputedStyle& style)
{
    // FIXME: We may produce slightly incorrect (less fine-grained) hyphenation here as the incoming content may just be a partial word.
    // (same applies to hyphenPosition below)
    size_t contentLength = content.length();
    if (!hasEnoughContentForHyphenation(contentLength, style))
        return { };

    auto limitBefore = limitBeforeValue(style);
    auto candidatePosition = std::min(contentLength, contentLength - limitAfterValue(style) + 1);
    auto firstHyphenLocation = std::optional<size_t> { };
    while (true) {
        auto hyphenIndex = lastHyphenLocation(content, candidatePosition, Style::toPlatform(style.usedLocale()));
        if (!hyphenIndex || hyphenIndex < limitBefore)
            return firstHyphenLocation;
        if (hyphenIndex >= candidatePosition) {
            ASSERT_NOT_REACHED();
            return { };
        }
        firstHyphenLocation = hyphenIndex;
        candidatePosition = hyphenIndex;
    }
    return { };
}

static std::optional<size_t> lastHyphenPosition(StringView content, const Style::ComputedStyle& style)
{
    size_t contentLength = content.length();
    if (!hasEnoughContentForHyphenation(contentLength, style))
        return { };

    if (auto hyphenIndex = lastHyphenLocation(content, std::min(contentLength, contentLength - limitAfterValue(style) + 1), Style::toPlatform(style.usedLocale())))
        return hyphenIndex >= limitBeforeValue(style) ? std::make_optional(hyphenIndex) : std::nullopt;
    return { };
}

static std::optional<size_t> hyphenPositionBefore(StringView content, const Style::ComputedStyle& style, size_t beforePosition)
{
    // Find the hyphen position as follows:
    // 1. Split the text by taking the hyphen width into account
    // 2. Find the last hyphen position before the split position
    auto contentLength = content.length();
    if (beforePosition < limitBeforeValue(style) || !hasEnoughContentForHyphenation(contentLength, style))
        return { };

    if (auto hyphenIndex = lastHyphenLocation(content, std::min(beforePosition, contentLength - limitAfterValue(style)) + 1, Style::toPlatform(style.usedLocale())))
        return hyphenIndex >= limitBeforeValue(style) ? std::make_optional(hyphenIndex) : std::nullopt;
    return { };
}

struct CandidateTextRunForBreaking {
    size_t index { 0 };
    bool isOverflowingRun { true };
    InlineLayoutUnit logicalLeft { 0 };
};
std::optional<InlineContentBreaker::PartialRun> InlineContentBreaker::tryBreakingTextRun(const ContinuousContent::RunList& runs, const CandidateTextRunForBreaking& candidateTextRun, InlineLayoutUnit availableWidth, const LineStatus& lineStatus) const
{
    auto& candidateRun = runs[candidateTextRun.index];
    ASSERT(candidateRun.inlineItem.isText());
    auto& inlineTextItem = downcast<InlineTextItem>(candidateRun.inlineItem);
    CheckedRef style = candidateRun.style;
    auto lineHasRoomForContent = availableWidth > 0;

    auto breakRules = wordBreakBehavior(style, lineStatus.hasWrapOpportunityAtPreviousPosition);
    if (breakRules.isEmpty())
        return { };

    CheckedRef fontCascade = style->fontCascade();
    if (breakRules.contains(WordBreakRule::AtArbitraryPositionWithinWords)) {
        auto tryBreakingAtArbitraryPositionWithinWords = [&]() -> std::optional<PartialRun> {
            // Breaking is allowed within “words”: specifically, in addition to soft wrap opportunities allowed for normal, any typographic letter units
            // It does not affect rules governing the soft wrap opportunities created by white space. Hyphenation is not applied.
            ASSERT(!breakRules.contains(WordBreakRule::AtHyphenationOpportunities));
            if (inlineTextItem.isWhitespace()) {
                // AtArbitraryPositionWithinWords does not affect the breaking opportunities around whitespace.
                return { };
            }

            if (!inlineTextItem.length()) {
                // Empty/single character text runs may be breakable based on style, but in practice we can't really split them any further.
                return { };
            }

            if (candidateTextRun.isOverflowingRun) {
                if (lineHasRoomForContent) {
                    // Try to break the overflowing run mid-word.
                    if (auto wordBreak = midWordBreak(candidateRun, candidateTextRun.logicalLeft, availableWidth))
                        return PartialRun { wordBreak->length, wordBreak->logicalWidth };
                }
                if (canBreakBefore(inlineTextItem.inlineTextBox().content()[inlineTextItem.start()], style->lineBreak()))
                    return PartialRun { };
                else {
                    // Since this is an overflowing content and we are allowed to break at arbitrary position, we really ought to find a breaking position.
                    // Unless of course it's really an unbreakable content with nothing but e.g. punctuation characters.
                    // FIXME: This should be merged with the "let's keep the first character on the line" logic (see in InlineContentBreaker::processOverflowingContent)
                    auto firstBreakablePosition = [&] () -> std::optional<TextUtil::WordBreakLeft> {
                        if (lineStatus.hasContent)
                            return { };
                        auto text = inlineTextItem.inlineTextBox().content();
                        const auto left = inlineTextItem.start();
                        auto right = left;
                        U16_SET_CP_START(text, left, right);
                        while (right < inlineTextItem.end()) {
                            U16_FWD_1(text, right, inlineTextItem.length());
                            if (canBreakBefore(text[right], style->lineBreak())) {
                                if (right == inlineTextItem.end())
                                    return { };
                                return TextUtil::WordBreakLeft { right - left, TextUtil::width(inlineTextItem, style->fontCascade(), left, right, candidateTextRun.logicalLeft) };
                            }
                        }
                        return { };
                    };
                    if (auto wordBreak = firstBreakablePosition())
                        return PartialRun { wordBreak->length, wordBreak->logicalWidth };
                }
                return { };
            }

            // This is a non-overflowing content.
            ASSERT(lineHasRoomForContent || !candidateRun.spaceRequired());
            if (auto breakingPosition = lastValidBreakingPosition(runs, candidateTextRun.index)) {
                ASSERT(*breakingPosition <= inlineTextItem.end());
                auto trailingLength = *breakingPosition - inlineTextItem.start();
                auto startPosition = inlineTextItem.start();
                auto endPosition = startPosition + trailingLength;
                return PartialRun { trailingLength, TextUtil::width(inlineTextItem, fontCascade, startPosition, endPosition, candidateTextRun.logicalLeft) };
            }
            return { };
        };
        return tryBreakingAtArbitraryPositionWithinWords();
    }

    if (breakRules.contains(WordBreakRule::AtHyphenationOpportunities)) {
        auto tryBreakingAtHyphenationOpportunity = [&]() -> std::optional<PartialRun> {
            auto hyphenWidth = TextUtil::hyphenWidth(style);
            auto hyphenLocation = [&] {
                if (!candidateTextRun.isOverflowingRun)
                    return lastHyphenPosition(inlineTextItem.content(), style);

                auto availableWidthExcludingHyphen = availableWidth - hyphenWidth;
                auto hasSomeRoomForContent = availableWidthExcludingHyphen > 0 && enoughWidthForHyphenation(availableWidthExcludingHyphen, fontCascade->size());
                if (hasSomeRoomForContent && candidateRun.spaceRequired()) {
                    auto leftSideLength = TextUtil::breakWord(inlineTextItem, fontCascade, candidateRun.spaceRequired(), availableWidthExcludingHyphen, candidateTextRun.logicalLeft).length;
                    if (auto position = hyphenPositionBefore(inlineTextItem.content(), style, leftSideLength))
                        return position;
                }
                return !lineStatus.hasContent && *firstTextRunIndex(runs) == candidateTextRun.index ? firstHyphenPosition(inlineTextItem.content(), style) : std::nullopt;
            };

            if (auto position = hyphenLocation()) {
                auto trailingPartialRunWidthWithHyphen = TextUtil::width(inlineTextItem, fontCascade, inlineTextItem.start(), inlineTextItem.start() + *position, candidateTextRun.logicalLeft);
                return PartialRun { *position, trailingPartialRunWidthWithHyphen, hyphenWidth };
            }
            return { };
        };
        if (auto partialRun = tryBreakingAtHyphenationOpportunity())
            return partialRun;
    }

    if (breakRules.contains(WordBreakRule::AtArbitraryPosition)) {
        auto tryBreakingAtArbitraryPosition = [&]() -> std::optional<PartialRun> {
            if (!inlineTextItem.length()) {
                // Empty text runs may be breakable based on style, but in practice we can't really split them any further.
                return { };
            }
            if (!candidateTextRun.isOverflowingRun) {
                // When the run can be split at arbitrary position let's just return the entire run when it is intended to fit on the line.
                // However the breaking properties only set rules for text content, so let's check if this run is adjacent to another text run.
                ASSERT(inlineTextItem.length());
                // FIXME: We may need to check if the "next" text run is visually adjacent to this non-overflowing run too (e.g. A<span style="border: 100px solid green;"></span>B)
                if (nextTextRunIndex(runs, candidateTextRun.index)) {
                    // We are in-between text runs. It's okay to return the entire run triggering split at the very right edge.
                    auto trailingPartialRunWidth = TextUtil::width(inlineTextItem, fontCascade, candidateTextRun.logicalLeft);
                    return PartialRun { inlineTextItem.length(), trailingPartialRunWidth };
                }
                if (inlineTextItem.length() > 1) {
                    auto startPosition = inlineTextItem.start();
                    auto endPosition = inlineTextItem.end() - 1;
                    return PartialRun { inlineTextItem.length() - 1, TextUtil::width(inlineTextItem, fontCascade, startPosition, endPosition, candidateTextRun.logicalLeft) };
                }
                return { };
            }
            if (!lineHasRoomForContent) {
                // Fast path for cases when there's no room at all. The content is breakable but we don't have space for it.
                return PartialRun { };
            }
            auto wordBreak = TextUtil::breakWord(inlineTextItem, fontCascade, candidateRun.spaceRequired(), availableWidth, candidateTextRun.logicalLeft);
            return PartialRun { wordBreak.length, wordBreak.logicalWidth };
        };
        // With arbitrary breaking there's always a valid breaking position (even if it is before the first position).
        return tryBreakingAtArbitraryPosition();
    }

    return { };
}

std::optional<InlineContentBreaker::OverflowingTextContent::BreakingPosition> InlineContentBreaker::tryBreakingOverflowingRun(const LineStatus& lineStatus, const ContinuousContent::RunList& runs, size_t overflowingRunIndex, InlineLayoutUnit nonOverflowingContentWidth) const
{
    auto overflowingRun = runs[overflowingRunIndex];
    ASSERT(!overflowingRun.inlineItem.isOutOfFlow());
    if (!isBreakableRun(overflowingRun))
        return { };

    auto availableWidth = std::max(0.f, lineStatus.availableWidth - nonOverflowingContentWidth);
    auto partialOverflowingRun = tryBreakingTextRun(runs, { overflowingRunIndex, true, lineStatus.contentLogicalRight + nonOverflowingContentWidth }, availableWidth, lineStatus);
    if (!partialOverflowingRun)
        return { };
    if (partialOverflowingRun->length)
        return OverflowingTextContent::BreakingPosition { overflowingRunIndex, OverflowingTextContent::BreakingPosition::TrailingContent { false, partialOverflowingRun } };
    // When the breaking position is at the beginning of the run, the trailing run is the previous one.
    if (auto trailingRunIndex = findTrailingRunIndexBeforeBreakableRun(runs, overflowingRunIndex))
        return OverflowingTextContent::BreakingPosition { *trailingRunIndex, OverflowingTextContent::BreakingPosition::TrailingContent { } };
    // Sometimes we can't accommodate even the very first character.
    // Note that this is different from when there's no breakable run in this set.
    return OverflowingTextContent::BreakingPosition { };
}

std::optional<InlineContentBreaker::OverflowingTextContent::BreakingPosition> InlineContentBreaker::tryBreakingPreviousNonOverflowingRuns(const LineStatus& lineStatus, const ContinuousContent::RunList& runs, size_t overflowingRunIndex, InlineLayoutUnit nonOverflowingContentWidth) const
{
    auto previousContentWidth = nonOverflowingContentWidth;
    for (auto index = overflowingRunIndex; index--;) {
        auto& run = runs[index];
        previousContentWidth -= run.spaceRequired();
        if (!isBreakableRun(run))
            continue;
        ASSERT(run.inlineItem.isText());
        auto availableWidth = std::max(0.f, lineStatus.availableWidth - previousContentWidth);
        if (auto partialRun = tryBreakingTextRun(runs, { index, false, lineStatus.contentLogicalRight + previousContentWidth }, availableWidth, lineStatus)) {
            // We know this run fits, so if breaking is allowed on the run, it should return a non-empty left-side
            // since it's either at hyphen position or the entire run is returned.
            ASSERT(partialRun->length);
            auto runIsFullyAccommodated = partialRun->length == downcast<InlineTextItem>(run.inlineItem).length();
            if (runIsFullyAccommodated) {
                auto trailingRunIndex = [&] {
                    // Try not break content at inline box boundary.
                    // e.g. <span style="word-wrap: break-word">fits_and_we_break_at_the_right_edge</span><span>overflows</span>
                    // we should forward the breaking index to the closing inline box.
                    // FIXME: We may wanna skip over the visually empty inline boxes only e.g. <span style="word-wrap: break-word">fits_and_we_break_at_the_right_edge</span><span></span><span>overflows</span>
                    auto trailingInlineBoxEndIndex = std::optional<size_t> { };
                    for (auto candidateIndex = index + 1; candidateIndex <= overflowingRunIndex; ++candidateIndex) {
                        auto& trailingInlineItem = runs[candidateIndex].inlineItem;
                        if (trailingInlineItem.isInlineBoxEnd())
                            trailingInlineBoxEndIndex = candidateIndex;
                        if (!trailingInlineItem.isInlineBoxStartOrEnd())
                            break;
                    }
                    ASSERT(!trailingInlineBoxEndIndex || *trailingInlineBoxEndIndex <= overflowingRunIndex);
                    return trailingInlineBoxEndIndex.value_or(index);
                };
                return OverflowingTextContent::BreakingPosition { trailingRunIndex(), OverflowingTextContent::BreakingPosition::TrailingContent { false } };
            }
            return OverflowingTextContent::BreakingPosition { index, OverflowingTextContent::BreakingPosition::TrailingContent { false, partialRun } };
        }
    }
    return { };
}

std::optional<InlineContentBreaker::OverflowingTextContent::BreakingPosition> InlineContentBreaker::tryBreakingNextOverflowingRuns(const LineStatus& lineStatus, const ContinuousContent::RunList& runs, size_t overflowingRunIndex, InlineLayoutUnit nonOverflowingContentWidth) const
{
    auto nextContentWidth = nonOverflowingContentWidth + runs[overflowingRunIndex].spaceRequired();
    for (auto index = overflowingRunIndex + 1; index < runs.size(); ++index) {
        auto& run = runs[index];
        if (!isBreakableRun(run)) {
            nextContentWidth += run.spaceRequired();
            continue;
        }
        ASSERT(run.inlineItem.isText());
        // At this point the available space is zero. Let's try the break these overflowing set of runs at the earliest possible.
        if (auto partialRun = tryBreakingTextRun(runs, { index, true, lineStatus.contentLogicalRight + nextContentWidth }, 0, lineStatus)) {
            // <span>unbreakable_and_overflows<span style="word-break: break-all">breakable</span>
            // The partial run length could very well be 0 meaning the trailing run is actually the overflowing run (see above in the example).
            if (partialRun->length) {
                // We managed to break this text run mid content. It has to be either an arbitrary mid-word or a hyphen break.
                return OverflowingTextContent::BreakingPosition { index, OverflowingTextContent::BreakingPosition::TrailingContent { true, partialRun } };
            }
            if (auto trailingRunIndex = findTrailingRunIndexBeforeBreakableRun(runs, index)) {
                // We may end up _before_ the overflowing run e.g.
                // <span></span><span style="border: 10px solid">breakable</span>
                // with 0px constraint, where while the second (non-empty) [inline box start] overflows, the trailing
                // run ends up being the first [inline box end] inline item.
                return OverflowingTextContent::BreakingPosition { *trailingRunIndex, OverflowingTextContent::BreakingPosition::TrailingContent { true } };
            }
            // This happens when the overflowing run is also the first run in this set, no trailing run.
            return OverflowingTextContent::BreakingPosition { overflowingRunIndex, { } };
        }
        nextContentWidth += run.spaceRequired();
    }
    return { };
}

std::optional<InlineContentBreaker::OverflowingTextContent::BreakingPosition> InlineContentBreaker::tryHyphenationAcrossOverflowingInlineTextItems(const LineStatus& lineStatus, const ContinuousContent::RunList& runs, size_t overflowingRunIndex) const
{
    if (runs.size() == 1)
        return { };

    CheckedRef style = runs.first().inlineItem.style();
    if (!wordBreakBehavior(style, lineStatus.hasWrapOpportunityAtPreviousPosition).contains(WordBreakRule::AtHyphenationOpportunities))
        return { };

    // 1. concatenate adjacent text content
    // 2. find the last hyphen location before the overflowing position
    // 3. find the inline text item where the hyphen location is and compute the partial run width
    auto content = StringBuilder { };
    size_t overflowingRunStartPosition = 0;
    for (size_t index = 0; index < runs.size(); ++index) {
        auto& inlineItem = runs[index].inlineItem;
        // FIXME: Maybe content across inline boxes should be hyphenated as well.
        if (inlineItem.isOutOfFlow())
            continue;
        if (!inlineItem.style().fontCascadeEqual(style.get()))
            return { };

        auto* inlineTextItem = dynamicDowncast<InlineTextItem>(inlineItem);
        if (!inlineTextItem)
            return { };
        if (inlineTextItem->isWhitespace())
            return { };
        content.append(inlineTextItem->content());
        overflowingRunStartPosition += index < overflowingRunIndex ? inlineTextItem->length() : 0;
    }
    // Only non-whitespace text runs with same style.
    CheckedRef fontCascade = style->fontCascade();
    auto hyphenWidth = TextUtil::hyphenWidth(style.get());
    auto availableWidthExcludingHyphen = lineStatus.availableWidth - hyphenWidth;
    if (availableWidthExcludingHyphen <= 0 || !enoughWidthForHyphenation(availableWidthExcludingHyphen, fontCascade->size()))
        return { };

    auto& overflowingRun = runs[overflowingRunIndex];
    auto* textItem = dynamicDowncast<InlineTextItem>(overflowingRun.inlineItem);
    ASSERT(textItem);
    if (!textItem)
        return { };
    // Make sure we always hyphenate before the overflow.
    auto overflowPositionWithHyphen = TextUtil::breakWord(*textItem, fontCascade, overflowingRun.spaceRequired(), availableWidthExcludingHyphen, lineStatus.contentLogicalRight).length;
    auto hyphenLocation = hyphenPositionBefore(content, style, overflowingRunStartPosition + overflowPositionWithHyphen);
    if (!hyphenLocation)
        return { };

    // hyphenLocation must be in or before the overflowing run.
    ASSERT(*hyphenLocation <= overflowingRunStartPosition + overflowPositionWithHyphen);
    auto hyphenLocationWithinInlineTextItem = *hyphenLocation;
    size_t hyphenatedRunIndex = 0;
    for (; hyphenatedRunIndex <= overflowingRunIndex; ++hyphenatedRunIndex) {
        auto& inlineItem = runs[hyphenatedRunIndex].inlineItem;
        if (inlineItem.isOutOfFlow())
            continue;
        auto& inlineTextItem = downcast<InlineTextItem>(inlineItem);
        if (inlineTextItem.length() >= hyphenLocationWithinInlineTextItem)
            break;
        hyphenLocationWithinInlineTextItem -= inlineTextItem.length();
    }
    auto& hyphenatedlineTextItem = downcast<InlineTextItem>(runs[hyphenatedRunIndex].inlineItem);
    if (hyphenLocationWithinInlineTextItem > hyphenatedlineTextItem.length()) {
        ASSERT_NOT_REACHED();
        return { };
    }

    // Hyphen may be right at the run (end) boundary.
    auto partialRun = std::optional<InlineContentBreaker::PartialRun> { };
    if (hyphenLocationWithinInlineTextItem < hyphenatedlineTextItem.length()) {
        auto trailingPartialRunWidthWithHyphen = TextUtil::width(hyphenatedlineTextItem, fontCascade, hyphenatedlineTextItem.start(), hyphenatedlineTextItem.start() + hyphenLocationWithinInlineTextItem, lineStatus.contentLogicalRight);
        partialRun = { hyphenLocationWithinInlineTextItem, trailingPartialRunWidthWithHyphen, hyphenWidth };
    }
    return OverflowingTextContent::BreakingPosition { hyphenatedRunIndex, OverflowingTextContent::BreakingPosition::TrailingContent { false, partialRun, hyphenWidth } };
}

InlineContentBreaker::OverflowingTextContent InlineContentBreaker::processOverflowingContentWithText(const ContinuousContent& continuousContent, const LineStatus& lineStatus) const
{
    auto& runs = continuousContent.runs();
    ASSERT(!runs.isEmpty());

    // Check where the overflow occurs and use the corresponding style to figure out the breaking behavior.
    // <span style="word-break: normal">first</span><span style="word-break: break-all">second</span><span style="word-break: normal">third</span>

    // First find the overflowing run.
    auto nonOverflowingContentWidth = InlineLayoutUnit { };
    auto overflowingRunIndex = runs.size(); 
    for (size_t index = 0; index < runs.size(); ++index) {
        auto& run = runs[index];
        if (run.inlineItem.isOutOfFlow())
            continue;
        auto runLogicalWidth = run.spaceRequired();
        if (nonOverflowingContentWidth + runLogicalWidth > lineStatus.availableWidth) {
            overflowingRunIndex = index;
            break;
        }
        nonOverflowingContentWidth += runLogicalWidth;
    }
    if (overflowingRunIndex == runs.size()) {
        // We have to have either an overflowing run or a soft hyphen.
        ASSERT(continuousContent.hasTrailingSoftHyphen() && runs.size());
        return { runs.size() ? runs.size() - 1 : 0 };
    }

    // Check first if we can actually break the overflowing run.
    if (auto breakingPosition = tryBreakingOverflowingRun(lineStatus, runs, overflowingRunIndex, nonOverflowingContentWidth))
        return { overflowingRunIndex, breakingPosition };

    auto& overflowingInlineItem = runs[overflowingRunIndex].inlineItem;
    // In some cases we just can't break before certain overflowing runs due to content specific CSS rules, e.g. line-break: after-white-space.
    // This is in addition to having soft wrap opportunties only after the whitespace. This is about not breaking at all
    // before the whitespace content e.g.
    // <div style="line-break: after-white-space; word-wrap: break-word">before<span style="white-space: pre">   </span>after</div>
    // "before" content is not breakable sine it is _before_ the overflowing whitespace content.
    auto isBreakingAllowedBeforeOverflowingRun = [&] {
        auto* textItem = dynamicDowncast<InlineTextItem>(overflowingInlineItem);
        return !textItem
            || !textItem->isWhitespace()
            || textItem->style().lineBreak() != LineBreak::AfterWhiteSpace;
    }();
    if (isBreakingAllowedBeforeOverflowingRun) {
        // We did not manage to break the run that overflows the line.
        // Let's try to find a previous breaking position starting from the overflowing run. It surely fits.
        if (auto breakingPosition = tryBreakingPreviousNonOverflowingRuns(lineStatus, runs, overflowingRunIndex, nonOverflowingContentWidth))
            return { overflowingRunIndex, breakingPosition };
    }

    if (auto breakingPosition = tryHyphenationAcrossOverflowingInlineTextItems(lineStatus, runs, overflowingRunIndex))
        return { overflowingRunIndex, breakingPosition };

    // At this point we know that there's no breakable run all the way to the overflowing run.
    // Now we need to check if any run after the overflowing content can break.
    // e.g. <span>this_content_overflows_but_not_breakable<span><span style="word-break: break-all">but_this_is_breakable</span>
    if (auto breakingPosition = tryBreakingNextOverflowingRuns(lineStatus, runs, overflowingRunIndex, nonOverflowingContentWidth))
        return { overflowingRunIndex, breakingPosition };

    // Give up, there's no breakable run in here.
    return { overflowingRunIndex };
}

void InlineContentBreaker::ContinuousContent::setTrailingSoftHyphenWidth(InlineLayoutUnit hyphenWidth)
{
    m_logicalWidth += hyphenWidth;
    m_hasTrailingSoftHyphen = true;
}

void InlineContentBreaker::ContinuousContent::appendToRunList(const InlineItem& inlineItem, const Style::ComputedStyle& style, InlineLayoutUnit offset, InlineLayoutUnit contentWidth, InlineLayoutUnit textSpacingAdjustment)
{
    m_runs.append({ inlineItem, style, offset, contentWidth, textSpacingAdjustment });
    m_logicalWidth = clampTo<InlineLayoutUnit>(m_logicalWidth + offset + contentWidth);
}

void InlineContentBreaker::ContinuousContent::resetTrailingTrimmableContent()
{
    if (!m_leadingTrimmableWidth)
        m_leadingTrimmableWidth = m_trailingTrimmableWidth;
    m_trailingTrimmableWidth = { };
    m_isFullyTrimmable = false;
}

void InlineContentBreaker::ContinuousContent::append(const InlineItem& inlineItem, const Style::ComputedStyle& style, InlineLayoutUnit logicalWidth, InlineLayoutUnit textSpacingAdjustment)
{
    ASSERT(inlineItem.isAtomicInlineBox() || inlineItem.isInlineBoxStartOrEnd() || inlineItem.isOutOfFlow() || inlineItem.isBlock());
    m_isTextOnlyContent = false;
    m_hasTrailingWordSeparator = m_hasTrailingWordSeparator && !inlineItem.isAtomicInlineBox();
    appendToRunList(inlineItem, style, { }, logicalWidth, textSpacingAdjustment);
    if (inlineItem.isAtomicInlineBox()) {
        // Inline boxes (whitespace-> <span></span>) do not prevent the trailing content from getting trimmed/hung
        // but atomic inline level boxes do.
        resetTrailingTrimmableContent();
    }
}

void InlineContentBreaker::ContinuousContent::appendTextContent(const InlineTextItem& inlineTextItem, const Style::ComputedStyle& style, InlineLayoutUnit logicalWidth)
{
    m_hasTextContent = true;
    auto isAfterWordSeparator = m_hasTrailingWordSeparator;
    m_hasTrailingWordSeparator = inlineTextItem.isWordSeparator();
    // https://www.w3.org/TR/css-text-4/#white-space-phase-2
    auto isTrailingHangingContent = inlineTextItem.isWhitespace() && TextUtil::shouldTrailingWhitespaceHang(style);
    if (isTrailingHangingContent)
        setHangingContentWidth(logicalWidth);

    auto trimmableWidth = [&]() -> std::optional<InlineLayoutUnit> {
        if (isTrailingHangingContent)
            return { };
        if (inlineTextItem.isFullyTrimmable() || inlineTextItem.isQuirkNonBreakingSpace())
            return logicalWidth;
        return { };
    }();
    if (!trimmableWidth) {
        auto contentOffset = isAfterWordSeparator ? style.usedWordSpacing() : 0.f;
        appendToRunList(inlineTextItem, style, contentOffset, logicalWidth);
        if (contentOffset && isFullyTrimmable()) {
            // word-spacing offset gets trimmed together with the leading trimmable content.
            m_leadingTrimmableWidth += contentOffset;
        }
        resetTrailingTrimmableContent();
        return;
    }

    m_isFullyTrimmable = m_isFullyTrimmable || m_runs.isEmpty();
    ASSERT(*trimmableWidth <= logicalWidth);
    auto isLeadingTrimmable = trimmableWidth && (!this->logicalWidth() || isFullyTrimmable());
    appendToRunList(inlineTextItem, style, isAfterWordSeparator ? style.usedWordSpacing() : 0.f, logicalWidth);
    if (isLeadingTrimmable) {
        ASSERT(!m_trailingTrimmableWidth);
        m_leadingTrimmableWidth += *trimmableWidth;
        return;
    }
    m_trailingTrimmableWidth = *trimmableWidth == logicalWidth ? m_trailingTrimmableWidth + logicalWidth : *trimmableWidth;
}

void InlineContentBreaker::ContinuousContent::reset()
{
    m_logicalWidth = { };
    m_leadingTrimmableWidth = { };
    m_trailingTrimmableWidth = { };
    m_hangingContentWidth = { };
    m_minimumRequiredWidth = { };
    m_runs.shrink(0);
    m_hasTextContent = false;
    m_isTextOnlyContent = true;
    m_isFullyTrimmable = false;
    m_hasTrailingWordSeparator = false;
    m_hasTrailingSoftHyphen = false;
    m_hasShapedContent = false;
}

}
}
```

## InlineContentBreaker.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineContentBreaker.h). Path: `Source/WebCore/layout/formattingContexts/inline/InlineContentBreaker.h`. Source bytes: 12318; source lines: 224; SHA-256: `ce7fe32f95ce2e354c24610151285faac2efc352dfd9f348b03bd560aef0a88b`.

```cpp
/*
 * Copyright (C) 2018 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <WebCore/FormattingConstraints.h>
#include <WebCore/LayoutUnits.h>
#include <WebCore/TextUtil.h>

namespace WebCore {

namespace Style {
class ComputedStyle;
}

namespace Layout {

class InlineItem;
class InlineTextItem;
struct CandidateTextRunForBreaking;

class InlineContentBreaker {
public:
    struct PartialRun {
        size_t length { 0 };
        InlineLayoutUnit logicalWidth { 0 };
        // FIXME: Remove this and collapse the rest of PartialRun over to PartialTrailingContent.
        std::optional<InlineLayoutUnit> hyphenWidth { };
    };
    enum class IsEndOfLine : bool { No, Yes };
    struct Result {
        enum class Action {
            Keep, // Keep content on the current line.
            Break, // Partial content is on the current line.
            Wrap, // Content is wrapped to the next line.
            WrapWithHyphen, // Content is wrapped to the next line and the current line ends with a visible hyphen.
            // The current content overflows and can't get broken up into smaller bits.
            RevertToLastWrapOpportunity, // The content needs to be reverted back to the last wrap opportunity.
            RevertToLastNonOverflowingWrapOpportunity // The content needs to be reverted back to a wrap opportunity that still fits the line.
        };
        struct PartialTrailingContent {
            size_t trailingRunIndex { 0 };
            std::optional<PartialRun> partialRun; // nullopt partial run means the trailing run is a complete run.
            std::optional<InlineLayoutUnit> hyphenWidth { }; // Hyphen may be at the end of a full run in the middle of the continuous content (e.g. with adjacent InlineTextItems).
        };
        Action action { Action::Keep };
        IsEndOfLine isEndOfLine { IsEndOfLine::No };
        std::optional<PartialTrailingContent> partialTrailingContent { };
        const InlineItem* lastWrapOpportunityItem { nullptr };
    };

    // This struct represents the amount of continuous content committed to content breaking at a time (no in-between wrap opportunities).
    // e.g.
    // <div>text content <span>span1</span>between<span>span2</span></div>
    // [text][ ][content][ ][inline box start][span1][inline box end][between][inline box start][span2][inline box end]
    // continuous candidate content at a time:
    // 1. [text]
    // 2. [ ]
    // 3. [content]
    // 4. [ ]
    // 5. [inline box start][span1][inline box end][between][inline box start][span2][inline box end]
    // see https://drafts.csswg.org/css-text-3/#line-break-details
    struct ContinuousContent {
        InlineLayoutUnit logicalWidth() const { return m_logicalWidth; }
        void adjustLogicalWidth(InlineLayoutUnit logicalWidth) { m_logicalWidth = logicalWidth; }
        std::optional<InlineLayoutUnit> minimumRequiredWidth() const { return m_minimumRequiredWidth; }
        InlineLayoutUnit leadingTrimmableWidth() const { return m_leadingTrimmableWidth; }
        InlineLayoutUnit trailingTrimmableWidth() const { return m_trailingTrimmableWidth; }
        InlineLayoutUnit hangingContentWidth() const { return m_hangingContentWidth.value_or(0.f); }
        bool hasTrimmableSpace() const { return trailingTrimmableWidth() || leadingTrimmableWidth(); }
        bool hasHangingSpace() const { return hangingContentWidth(); }
        bool hasTrailingSoftHyphen() const { return m_hasTrailingSoftHyphen; }
        bool hasTextContent() const { return m_hasTextContent; }
        bool isTextOnlyContent() const { return m_isTextOnlyContent; }
        bool isFullyTrimmable() const { return m_isFullyTrimmable; }
        bool isHangingContent() const { return m_hangingContentWidth && *m_hangingContentWidth == logicalWidth(); }

        void append(const InlineItem&, const Style::ComputedStyle&, InlineLayoutUnit logicalWidth, InlineLayoutUnit textSpacingAdjustment = 0.f);
        void appendTextContent(const InlineTextItem&, const Style::ComputedStyle&, InlineLayoutUnit logicalWidth);
        void setHangingContentWidth(InlineLayoutUnit logicalWidth) { m_hangingContentWidth = logicalWidth; }
        void NODELETE setTrailingSoftHyphenWidth(InlineLayoutUnit);
        void setMinimumRequiredWidth(InlineLayoutUnit minimumRequiredWidth) { m_minimumRequiredWidth = minimumRequiredWidth; }
        void setHasShapedContent() { m_hasShapedContent = true; }
        bool hasShapedContent() const { return m_hasShapedContent; }
        void reset();

        struct Run {
            Run(const InlineItem&, const Style::ComputedStyle&, InlineLayoutUnit offset, InlineLayoutUnit contentWidth, InlineLayoutUnit textSpacingAdjustment = 0.f);
            Run(const Run&);
            Run& operator=(const Run&) = delete;

            InlineLayoutUnit spaceRequired() const { return offset + contentWidth(); }
            void adjustContentWidth(InlineLayoutUnit contentWidth) { m_contentWidth = contentWidth; }
            InlineLayoutUnit contentWidth() const { return m_contentWidth; }

            const InlineItem& inlineItem;
            const Style::ComputedStyle& style;
            InlineLayoutUnit offset { 0 };
            InlineLayoutUnit textSpacingAdjustment { 0 };
            enum class ShapingBoundary : bool { Start, End };
            std::optional<ShapingBoundary> shapingBoundary { };

        private:
            InlineLayoutUnit m_contentWidth { 0 };
        };
        using RunList = Vector<Run, 3>;
        const RunList& runs() const LIFETIME_BOUND { return m_runs; }
        RunList& runs() LIFETIME_BOUND { return m_runs; }

    private:
        void appendToRunList(const InlineItem&, const Style::ComputedStyle&, InlineLayoutUnit offset, InlineLayoutUnit contentWidth, InlineLayoutUnit textSpacingAdjustment = 0.f);
        void NODELETE resetTrailingTrimmableContent();

        RunList m_runs;
        InlineLayoutUnit m_logicalWidth { 0.f };
        InlineLayoutUnit m_leadingTrimmableWidth { 0.f };
        InlineLayoutUnit m_trailingTrimmableWidth { 0.f };
        std::optional<InlineLayoutUnit> m_hangingContentWidth { };
        std::optional<InlineLayoutUnit> m_minimumRequiredWidth { };
        bool m_hasTextContent { false };
        bool m_isTextOnlyContent { true };
        bool m_isFullyTrimmable { false };
        bool m_hasTrailingWordSeparator { false };
        bool m_hasTrailingSoftHyphen { false };
        bool m_hasShapedContent { false };
    };

    struct LineStatus {
        InlineLayoutUnit contentLogicalRight { 0 };
        InlineLayoutUnit availableWidth { 0 };
        // Both of these types of trailing content may be ignored when checking for content fit.
        InlineLayoutUnit trimmableOrHangingWidth { 0 };
        std::optional<InlineLayoutUnit> trailingSoftHyphenWidth;
        bool hasFullyTrimmableTrailingContent { false };
        bool hasContent { false };
        bool hasWrapOpportunityAtPreviousPosition { false };
    };
    Result processInlineContent(const ContinuousContent&, const LineStatus&);
    void setHyphenationDisabled(bool hyphenationIsDisabled) { m_hyphenationIsDisabled = hyphenationIsDisabled; }
    void setIsMinimumInIntrinsicWidthMode(bool isMinimumInIntrinsicWidthMode) { m_isMinimumInIntrinsicWidthMode = isMinimumInIntrinsicWidthMode; }

private:
    Result processOverflowingContent(const ContinuousContent&, const LineStatus&) const;

    struct OverflowingTextContent {
        size_t runIndex { 0 }; // Overflowing run index. There's always an overflowing run.
        struct BreakingPosition {
            size_t runIndex { 0 };
            struct TrailingContent {
                // Trailing content is either the run's left side (when we break the run somewhere in the middle) or the previous run.
                // Sometimes the breaking position is at the very beginning of the first run, so there's no trailing run at all.
                bool overflows { false };
                std::optional<InlineContentBreaker::PartialRun> partialRun { };
                std::optional<InlineLayoutUnit> hyphenWidth { };
            };
            std::optional<TrailingContent> trailingContent { };
        };
        std::optional<BreakingPosition> breakingPosition { }; // Where we actually break this overflowing content.
    };
    OverflowingTextContent processOverflowingContentWithText(const ContinuousContent&, const LineStatus&) const;
    std::optional<Result> simplifiedMinimumIntrinsicWidthBreak(const ContinuousContent&, const LineStatus&) const;
    std::optional<PartialRun> tryBreakingTextRun(const ContinuousContent::RunList& runs, const CandidateTextRunForBreaking&, InlineLayoutUnit availableWidth, const LineStatus&) const;
    std::optional<OverflowingTextContent::BreakingPosition> tryBreakingOverflowingRun(const LineStatus&, const ContinuousContent::RunList&, size_t overflowingRunIndex, InlineLayoutUnit nonOverflowingContentWidth) const;
    std::optional<OverflowingTextContent::BreakingPosition> tryBreakingPreviousNonOverflowingRuns(const LineStatus&, const ContinuousContent::RunList&, size_t overflowingRunIndex, InlineLayoutUnit nonOverflowingContentWidth) const;
    std::optional<OverflowingTextContent::BreakingPosition> tryBreakingNextOverflowingRuns(const LineStatus&, const ContinuousContent::RunList&, size_t overflowingRunIndex, InlineLayoutUnit nonOverflowingContentWidth) const;
    std::optional<OverflowingTextContent::BreakingPosition> tryHyphenationAcrossOverflowingInlineTextItems(const LineStatus&, const ContinuousContent::RunList&, size_t overflowingRunIndex) const;

    using WordBreakRule = TextUtil::WordBreakRule;
    EnumSet<WordBreakRule> wordBreakBehavior(const Style::ComputedStyle& style, bool hasWrapOpportunityAtPreviousPosition) const
    {
        return TextUtil::wordBreakBehavior(style, hasWrapOpportunityAtPreviousPosition,
            m_isMinimumInIntrinsicWidthMode ? TextUtil::IsMinimumInIntrinsicWidthMode::Yes : TextUtil::IsMinimumInIntrinsicWidthMode::No,
            m_hyphenationIsDisabled ? TextUtil::HyphenationIsDisabled::Yes : TextUtil::HyphenationIsDisabled::No);
    }
    bool isMinimumInIntrinsicWidthMode() const { return m_isMinimumInIntrinsicWidthMode; }

private:
    bool m_isMinimumInIntrinsicWidthMode { false };
    bool m_hyphenationIsDisabled { false };
};

inline InlineContentBreaker::ContinuousContent::Run::Run(const InlineItem& inlineItem, const Style::ComputedStyle& style, InlineLayoutUnit offset, InlineLayoutUnit contentWidth, InlineLayoutUnit textSpacingAdjustment)
    : inlineItem(inlineItem)
    , style(style)
    , offset(offset)
    , textSpacingAdjustment(textSpacingAdjustment)
    , m_contentWidth(contentWidth)
{
}

inline InlineContentBreaker::ContinuousContent::Run::Run(const Run& other)
    : inlineItem(other.inlineItem)
    , style(other.style)
    , offset(other.offset)
    , textSpacingAdjustment(other.textSpacingAdjustment)
    , shapingBoundary(other.shapingBoundary)
    , m_contentWidth(other.contentWidth())
{
}

}
}
```

## TextUtil.cpp

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/text/TextUtil.cpp). Path: `Source/WebCore/layout/formattingContexts/inline/text/TextUtil.cpp`. Source bytes: 42235; source lines: 814; SHA-256: `ef914f2a1941915eba05dcb311a6cf6f455184a290d88a46d63198d532270232`.

```cpp
/*
 * Copyright (C) 2018-2024 Apple Inc. All rights reserved.
 * Copyright (C) 2014 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "config.h"
#include "TextUtil.h"

#include "BreakablePositions.h"
#include "ComplexTextController.h"
#include "FontCascadeFonts.h"
#include "FontCascadeInlines.h"
#include "FontInlines.h"
#include "Hyphenation.h"
#include "InlineLineTypes.h"
#include "InlineTextItem.h"
#include "Latin1TextIterator.h"
#include "LayoutInlineTextBox.h"
#include "RenderBox.h"
#include "RenderGlyph.h"
#include "StyleComputedStyle+GettersInlines.h"
#include "SurrogatePairAwareTextIterator.h"
#include "TextRun.h"
#include "TextSpacing.h"
#include "UnicodeHelpers.h"
#include "WidthIterator.h"
#include <numeric>
#include <wtf/text/CharacterProperties.h>
#include <wtf/text/ParsingUtilities.h>
#include <wtf/text/TextBreakIterator.h>

namespace WebCore {
namespace Layout {

InlineLayoutUnit TextUtil::singleSpaceWidth(const FontCascade& fontCascade, bool canUseSimplifiedContentMeasuring)
{
    auto width = canUseSimplifiedContentMeasuring ? fontCascade.primaryFont().spaceWidth() : fontCascade.widthOfSpaceString();
    if (std::isnan(width) || std::isinf(width)) [[unlikely]]
        return std::isnan(width) ? 0.0f : maxInlineLayoutUnit();
    return width;
}

InlineLayoutUnit TextUtil::width(const InlineTextBox& inlineTextBox, const FontCascade& fontCascade, unsigned from, unsigned to, InlineLayoutUnit contentLogicalLeft, UseTrailingWhitespaceMeasuringOptimization useTrailingWhitespaceMeasuringOptimization, TextSpacing::SpacingState spacingState, GlyphOverflow* glyphOverflow)
{
    if (from == to)
        return 0;

    if (inlineTextBox.hasSynthesizedGlyph()) {
        if (CheckedPtr glyphRenderer = dynamicDowncast<RenderGlyph>(inlineTextBox.rendererForIntegration()))
            return glyphRenderer->advanceRatio() * fontCascade.size();
        return (fontCascade.metricsOfPrimaryFont().intAscent() * 2 / 3 + 1) / 2 + 2; // List bullet.
    }

    if (inlineTextBox.isCombined())
        return fontCascade.size();

    auto& text = inlineTextBox.content();
    ASSERT(to <= text.length());
    auto hasKerningOrLigatures = fontCascade.enableKerning() || fontCascade.requiresShaping();
    // The "non-whitespace" + "whitespace" pattern is very common for inline content and since most of the "non-whitespace" runs end up with
    // their "whitespace" pair on the line (notable exception is when trailing whitespace is trimmed).
    // Including the trailing whitespace here enables us to cut the number of text measures when placing content on the line.
    auto extendedMeasuring = useTrailingWhitespaceMeasuringOptimization == UseTrailingWhitespaceMeasuringOptimization::Yes && hasKerningOrLigatures && to < text.length() && text[to] == space;
    if (extendedMeasuring)
        ++to;
    auto width = 0.f;
    auto useSimplifiedContentMeasuring = inlineTextBox.canUseSimplifiedContentMeasuring();
    if (useSimplifiedContentMeasuring) {
        auto view = StringView(text).substring(from, to - from);
        if (fontCascade.canTakeFixedPitchFastContentMeasuring())
            width = fontCascade.widthForSimpleTextWithFixedPitch(view, inlineTextBox.style().collapseWhiteSpace());
        else
            width = fontCascade.widthForTextUsingSimplifiedMeasuring(view);
    } else {
        CheckedRef style = inlineTextBox.style();
        auto directionalOverride = isOverride(style->unicodeBidi());
        auto run = WebCore::TextRun { StringView(text).substring(from, to - from), contentLogicalLeft, { }, ExpansionBehavior::defaultBehavior(), directionalOverride ? style->writingMode().bidiDirection() : TextDirection::LTR, directionalOverride };
        if (!style->collapseWhiteSpace() && !style->tabSize().isZero())
            run.setTabSize(true, Style::toPlatform(style->tabSize(), style->usedZoomForLength()));
        // FIXME: consider moving this to TextRun ctor
        run.setTextSpacingState(spacingState);
        width = fontCascade.width(run, { }, glyphOverflow);
    }

    if (extendedMeasuring)
        width -= (singleSpaceWidth(fontCascade, useSimplifiedContentMeasuring) + fontCascade.wordSpacing());

    if (std::isnan(width) || std::isinf(width)) [[unlikely]]
        return std::isnan(width) ? 0.0f : maxInlineLayoutUnit();
    return std::max(0.f, width);
}

InlineLayoutUnit TextUtil::width(const InlineTextItem& inlineTextItem, const FontCascade& fontCascade, InlineLayoutUnit contentLogicalLeft)
{
    return TextUtil::width(inlineTextItem, fontCascade, inlineTextItem.start(), inlineTextItem.end(), contentLogicalLeft);
}

InlineLayoutUnit TextUtil::width(const InlineTextItem& inlineTextItem, const FontCascade& fontCascade, unsigned from, unsigned to, InlineLayoutUnit contentLogicalLeft, UseTrailingWhitespaceMeasuringOptimization useTrailingWhitespaceMeasuringOptimization, TextSpacing::SpacingState spacingState, GlyphOverflow* glyphOverflow)
{
    RELEASE_ASSERT(from >= inlineTextItem.start());
    RELEASE_ASSERT(to <= inlineTextItem.end());

    if (inlineTextItem.isWhitespace()) {
        CheckedRef inlineTextBox = inlineTextItem.inlineTextBox();
        if (!TextUtil::shouldPreserveSpacesAndTabs(inlineTextBox) || (to - from == 1 && inlineTextBox->content()[from] == space))
            return std::max(0.f, singleSpaceWidth(fontCascade, inlineTextBox->canUseSimplifiedContentMeasuring()));
    }
    return width(protect(inlineTextItem.inlineTextBox()), fontCascade, from, to, contentLogicalLeft, useTrailingWhitespaceMeasuringOptimization, spacingState, glyphOverflow);
}

InlineLayoutUnit TextUtil::trailingWhitespaceWidth(const InlineTextBox& inlineTextBox, const FontCascade& fontCascade, size_t startPosition, size_t endPosition)
{
    ASSERT(endPosition > startPosition + 1);
    ASSERT(inlineTextBox.content()[endPosition - 1] == space);
    return width(inlineTextBox, fontCascade, startPosition, endPosition, { }, UseTrailingWhitespaceMeasuringOptimization::Yes) - 
        width(inlineTextBox, fontCascade, startPosition, endPosition - 1, { }, UseTrailingWhitespaceMeasuringOptimization::No);
}

template <typename TextIterator>
static void fallbackFontsForRunWithIterator(SingleThreadWeakHashSet<const Font>& fallbackFonts, const FontCascade& fontCascade, const TextRun& run, TextIterator& textIterator)
{
    auto isRTL = run.rtl();
    auto isSmallCaps = fontCascade.isSmallCaps();
    Ref primaryFont = fontCascade.primaryFont();

    char32_t currentCharacter = 0;
    unsigned clusterLength = 0;
    while (textIterator.consume(currentCharacter, clusterLength)) {

        auto addFallbackFontForCharacterIfApplicable = [&](auto character) {
            if (isSmallCaps)
                character = u_toupper(character);

            auto glyphData = fontCascade.glyphDataForCharacter(character, isRTL);
            if (glyphData.glyph && glyphData.font && glyphData.font != primaryFont.ptr()) {
                auto isNonSpacingMark = U_MASK(u_charType(character)) & U_GC_MN_MASK;

                // https://drafts.csswg.org/css-text-3/#white-space-processing
                // "Unsupported Default_ignorable characters must be ignored for text rendering."
                auto isIgnored = isDefaultIgnorableCodePoint(character);

                if (isNonSpacingMark || glyphData.font->widthForGlyph(glyphData.glyph)) {
                    if (!isIgnored)
                        fallbackFonts.add(*glyphData.font);
                }
            }
        };
        addFallbackFontForCharacterIfApplicable(currentCharacter);
        textIterator.advance(clusterLength);
    }
}

TextUtil::FallbackFontList TextUtil::fallbackFontsForText(StringView textContent, const Style::ComputedStyle& style, IncludeHyphen includeHyphen)
{
    TextUtil::FallbackFontList fallbackFonts;

    auto collectFallbackFonts = [&](auto&& textRun) {
        if (textRun.text().isEmpty())
            return;

        if (textRun.is8Bit()) {
            Latin1TextIterator textIterator { textRun.span8(), 0, textRun.length() };
            fallbackFontsForRunWithIterator(fallbackFonts, style.fontCascade(), textRun, textIterator);
            return;
        }
        SurrogatePairAwareTextIterator textIterator { textRun.span16(), 0, textRun.length() };
        fallbackFontsForRunWithIterator(fallbackFonts, style.fontCascade(), textRun, textIterator);
    };

    if (includeHyphen == IncludeHyphen::Yes)
        collectFallbackFonts(TextRun { StringView(style.hyphenString()), { }, { }, ExpansionBehavior::defaultBehavior(), style.writingMode().bidiDirection() });
    collectFallbackFonts(TextRun { textContent, { }, { }, ExpansionBehavior::defaultBehavior(), style.writingMode().bidiDirection() });
    return fallbackFonts;
}

template <typename TextIterator>
static TextUtil::EnclosingAscentDescent enclosingGlyphBoundsForRunWithIterator(const FontCascade& fontCascade, bool isRTL, TextIterator& textIterator)
{
    auto enclosingAscent = std::optional<InlineLayoutUnit> { };
    auto enclosingDescent = std::optional<InlineLayoutUnit> { };
    auto isSmallCaps = fontCascade.isSmallCaps();
    Ref primaryFont = fontCascade.primaryFont();

    char32_t currentCharacter = 0;
    unsigned clusterLength = 0;
    while (textIterator.consume(currentCharacter, clusterLength)) {

        auto computeTopAndBottomForCharacter = [&](auto character) {
            if (isSmallCaps)
                character = u_toupper(character);

            auto glyphData = fontCascade.glyphDataForCharacter(character, isRTL);
            Ref font = glyphData.font ? Ref { *glyphData.font } : primaryFont;
            auto bounds = font->boundsForGlyph(glyphData.glyph);

            enclosingAscent = std::min(enclosingAscent.value_or(bounds.y()), bounds.y());
            enclosingDescent = std::max(enclosingDescent.value_or(bounds.maxY()), bounds.maxY());
        };
        computeTopAndBottomForCharacter(currentCharacter);
        textIterator.advance(clusterLength);
    }
    return { enclosingAscent.value_or(0.f), enclosingDescent.value_or(0.f) };
}

TextUtil::EnclosingAscentDescent TextUtil::enclosingGlyphBoundsForText(StringView textContent, const Style::ComputedStyle& style, ShouldUseSimpleGlyphOverflowCodePath shouldUseSimpleGlyphOverflowCodePath)
{
    if (textContent.isEmpty())
        return { };

    if (shouldUseSimpleGlyphOverflowCodePath == ShouldUseSimpleGlyphOverflowCodePath::No) {
        auto overflow = ComplexTextController::enclosingGlyphBoundsForTextRun(protect(style.fontCascade()), TextRun { textContent });
        return { overflow.first, overflow.second };
    }

    if (textContent.is8Bit()) {
        Latin1TextIterator textIterator { textContent.span8(), 0, textContent.length() };
        return enclosingGlyphBoundsForRunWithIterator(protect(style.fontCascade()), style.writingMode().isBidiRTL(), textIterator);
    }

    SurrogatePairAwareTextIterator textIterator { textContent.span16(), 0, textContent.length() };
    return enclosingGlyphBoundsForRunWithIterator(protect(style.fontCascade()), style.writingMode().isBidiRTL(), textIterator);
}

TextUtil::WordBreakLeft TextUtil::breakWord(const InlineTextItem& inlineTextItem, const FontCascade& fontCascade, InlineLayoutUnit textWidth, InlineLayoutUnit availableWidth, InlineLayoutUnit contentLogicalLeft)
{
    return breakWord(protect(inlineTextItem.inlineTextBox()), inlineTextItem.start(), inlineTextItem.length(), textWidth, availableWidth, contentLogicalLeft, fontCascade);
}

TextUtil::WordBreakLeft TextUtil::breakWord(const InlineTextBox& inlineTextBox, size_t startPosition, size_t length, InlineLayoutUnit textWidth, InlineLayoutUnit availableWidth, InlineLayoutUnit contentLogicalLeft, const FontCascade& fontCascade)
{
    ASSERT(availableWidth >= 0);
    ASSERT(length);
    auto& text = inlineTextBox.content();

    if (!textWidth) [[unlikely]] {
        ASSERT_NOT_REACHED();
        return { };
    }

    if (inlineTextBox.canUseSimpleFontCodePath()) {

        auto findBreakingPositionInSimpleText = [&] {
            auto userPerceivedCharacterBoundaryAlignedIndex = [&] (auto index) -> size_t {
                if (text.is8Bit())
                    return index;
                auto alignedStartIndex = index;
                U16_SET_CP_START(text, startPosition, alignedStartIndex);
                ASSERT(alignedStartIndex >= startPosition);
                return alignedStartIndex;
            };

            auto trySimplifiedBreakingPosition = [&] (auto start) -> std::optional<WordBreakLeft> {
                auto mayUseSimplifiedBreakingPositionForFixedPitch = fontCascade.isFixedPitch() && inlineTextBox.canUseSimplifiedContentMeasuring();
                if (!mayUseSimplifiedBreakingPositionForFixedPitch)
                    return { };
                // FIXME: Check if we could bring webkit.org/b/221581 back for system monospace fonts.
                auto monospaceCharacterWidth = fontCascade.widthOfSpaceString();
                size_t estimatedCharacterCount = floorf(availableWidth / monospaceCharacterWidth);
                auto end = userPerceivedCharacterBoundaryAlignedIndex(std::min(start + estimatedCharacterCount, start + length - 1));
                auto underflowWidth = TextUtil::width(inlineTextBox, fontCascade, start, end, contentLogicalLeft);
                if (underflowWidth > availableWidth || underflowWidth + monospaceCharacterWidth < availableWidth) {
                    // This does not look like a real fixed pitch font. Let's just fall back to regular bisect.
                    // In some edge cases (float precision) using monospaceCharacterWidth here may produce an incorrect off-by-one visual overflow.
                    return { };
                }
                return { WordBreakLeft { end - start, underflowWidth } };
            };
            if (auto leftSide = trySimplifiedBreakingPosition(startPosition))
                return *leftSide;

            auto tryEstimateBreakingPosition = [&] (auto start) -> std::optional<WordBreakLeft> {
                // Determine if the breaking position can be found within the range of [-1, 1] by using the average width of the characters.
                auto averageCharacterWidth = InlineLayoutUnit { textWidth / length };
                size_t candidateLength = availableWidth / averageCharacterWidth;
                auto candidateEnd = userPerceivedCharacterBoundaryAlignedIndex(start + candidateLength);
                if (candidateEnd <= start || candidateEnd >= start + length)
                    return { };
                auto contentWidthAtCandidatePosition = TextUtil::width(inlineTextBox, fontCascade, start, candidateEnd, contentLogicalLeft);
                if (contentWidthAtCandidatePosition == availableWidth)
                    return { WordBreakLeft { candidateEnd - start, contentWidthAtCandidatePosition } };

                if (contentWidthAtCandidatePosition > availableWidth) {
                    // Overshot.
                    if (auto adjustedCandidateEnd = userPerceivedCharacterBoundaryAlignedIndex(candidateEnd - 1); adjustedCandidateEnd > start) {
                        auto adjustedContentWidth = TextUtil::width(inlineTextBox, fontCascade, start, adjustedCandidateEnd, contentLogicalLeft);
                        if (adjustedContentWidth <= availableWidth)
                            return { WordBreakLeft { adjustedCandidateEnd - start, adjustedContentWidth } };
                    }
                } else if (auto adjustedCandidateEnd = userPerceivedCharacterBoundaryAlignedIndex(candidateEnd + 1); adjustedCandidateEnd < start + length) {
                    // Undershot.
                    auto adjustedContentWidth = TextUtil::width(inlineTextBox, fontCascade, start, adjustedCandidateEnd, contentLogicalLeft);
                    if (adjustedContentWidth > availableWidth)
                        return { WordBreakLeft { candidateEnd - start, contentWidthAtCandidatePosition } };
                    if (adjustedContentWidth == availableWidth)
                        return { WordBreakLeft { adjustedCandidateEnd - start, adjustedContentWidth } };
                }
                return { };
            };
            if (auto leftSide = tryEstimateBreakingPosition(startPosition))
                return *leftSide;

            auto nextUserPerceivedCharacterIndex = [&] (auto index) -> size_t {
                if (text.is8Bit())
                    return index + 1;
                U16_FWD_1(text, index, startPosition + length);
                return index;
            };

            auto left = startPosition;
            auto right = left + length - 1;
            // Pathological case of (extremely)long string and narrow lines.
            // Adjust the range so that we can pick a reasonable midpoint.
            auto averageCharacterWidth = InlineLayoutUnit { textWidth / length };
            // Overshot the midpoint so that biscection starts at the left side of the content.
            size_t startOffset = 2 * availableWidth / averageCharacterWidth;
            right = userPerceivedCharacterBoundaryAlignedIndex(std::min(left + startOffset, right));
            // Preserve the left width for the final split position so that we don't need to remeasure the left side again.
            auto leftSideWidth = InlineLayoutUnit { 0 };
            while (left < right) {
                auto middle = userPerceivedCharacterBoundaryAlignedIndex(std::midpoint(left, right));
                ASSERT(middle >= left && middle < right);
                auto endOfMiddleCharacter = nextUserPerceivedCharacterIndex(middle);
                auto width = TextUtil::width(inlineTextBox, fontCascade, startPosition, endOfMiddleCharacter, contentLogicalLeft);
                if (width < availableWidth) {
                    left = endOfMiddleCharacter;
                    leftSideWidth = width;
                } else if (width > availableWidth)
                    right = middle;
                else {
                    right = endOfMiddleCharacter;
                    leftSideWidth = width;
                    break;
                }
            }
            RELEASE_ASSERT(right >= startPosition);
            return WordBreakLeft { right - startPosition, leftSideWidth };
        };
        return findBreakingPositionInSimpleText();
    }

    auto graphemeClusterIterator = NonSharedCharacterBreakIterator { StringView { text }.substring(startPosition, length) };
    auto leftSide = TextUtil::WordBreakLeft { };
    for (auto clusterStartPosition = ubrk_next(graphemeClusterIterator); clusterStartPosition != UBRK_DONE; clusterStartPosition = ubrk_next(graphemeClusterIterator)) {
        auto width = TextUtil::width(inlineTextBox, fontCascade, startPosition, startPosition + clusterStartPosition, contentLogicalLeft);
        if (width > availableWidth)
            return leftSide;
        leftSide = { static_cast<size_t>(clusterStartPosition), width };
    }
    // This content is not supposed to fit availableWidth.
    ASSERT_NOT_REACHED();
    return leftSide;
}

bool TextUtil::mayBreakInBetween(const InlineTextItem& previousInlineItem, const InlineTextItem& nextInlineItem)
{
    // Check if these 2 adjacent non-whitespace inline items are connected at a breakable position.
    ASSERT(!previousInlineItem.isWhitespace() && !nextInlineItem.isWhitespace());
    // Only the next item's leading edge decides breakability here, so when it starts at the beginning of
    // its text box we can pass the box content directly. Only when leading content was dropped (e.g.
    // white-space-trim moves start() past 0) do we take the item's substring and pay for the allocation.
    String nextContent = nextInlineItem.start() ? nextInlineItem.content() : nextInlineItem.inlineTextBox().content();
    return mayBreakInBetween(previousInlineItem.inlineTextBox().content(), protect(previousInlineItem.style()), nextContent, protect(nextInlineItem.style()));
}

bool TextUtil::mayBreakInBetween(String previousContent, const Style::ComputedStyle& previousContentStyle, String nextContent, const Style::ComputedStyle& nextContentStyle)
{
    // Now we need to collect at least 3 adjacent characters to be able to make a decision whether the previous text item ends with breaking opportunity.
    // [ex-][ample] <- second to last[x] last[-] current[a]
    // We need at least 1 character in the current inline text item and 2 more from previous inline items.
    if (!previousContent.is8Bit()) {
        // FIXME: Remove this workaround when we move over to a better way of handling prior-context with Unicode.
        // See the templated CharacterType in nextBreakablePosition for last and lastlast characters.
        nextContent.convertTo16Bit();
    }
    auto lineBreakIteratorFactory = CachedLineBreakIteratorFactory { nextContent, Style::toPlatform(nextContentStyle.usedLocale()), TextUtil::lineBreakIteratorMode(nextContentStyle.lineBreak()), TextUtil::contentAnalysis(nextContentStyle.wordBreak()) };
    auto previousContentLength = previousContent.length();
    // FIXME: We should look into the entire uncommitted content for more text context.
    char16_t lastCharacter = previousContentLength ? previousContent[previousContentLength - 1] : 0;
    if (lastCharacter == softHyphen && previousContentStyle.hyphens() == Hyphens::None)
        return false;
    char16_t secondToLastCharacter = previousContentLength > 1 ? previousContent[previousContentLength - 2] : 0;
    lineBreakIteratorFactory.priorContext().set({ secondToLastCharacter, lastCharacter });
    // Now check if we can break right at the inline item boundary.
    // With the [ex-ample], findNextBreakablePosition should return the startPosition (0).
    // FIXME: Check if there's a more correct way of finding breaking opportunities.
    return !findNextBreakablePosition(lineBreakIteratorFactory, 0, nextContentStyle);
}

unsigned TextUtil::findNextBreakablePosition(CachedLineBreakIteratorFactory& lineBreakIteratorFactory, unsigned startPosition, const Style::ComputedStyle& style)
{
    auto wordBreak = style.wordBreak();
    auto breakNBSP = style.textWrapMode() != TextWrapMode::NoWrap && style.nbspMode() == NBSPMode::Space;

    if (wordBreak == WordBreak::KeepAll) {
        if (breakNBSP)
            return BreakablePositions::next<BreakablePositions::LineBreakRules::Special, BreakablePositions::WordBreakBehavior::KeepAll, BreakablePositions::NoBreakSpaceBehavior::Break>(lineBreakIteratorFactory, startPosition);
        return BreakablePositions::next<BreakablePositions::LineBreakRules::Special, BreakablePositions::WordBreakBehavior::KeepAll, BreakablePositions::NoBreakSpaceBehavior::Normal>(lineBreakIteratorFactory, startPosition);
    }

    if (wordBreak == WordBreak::AutoPhrase)
        return BreakablePositions::next<BreakablePositions::LineBreakRules::Special, BreakablePositions::WordBreakBehavior::AutoPhrase, BreakablePositions::NoBreakSpaceBehavior::Normal>(lineBreakIteratorFactory, startPosition);

    if (lineBreakIteratorFactory.mode() == TextBreakIterator::LineMode::Behavior::Default) {
        if (breakNBSP)
            return BreakablePositions::next<BreakablePositions::LineBreakRules::Normal, BreakablePositions::WordBreakBehavior::Normal, BreakablePositions::NoBreakSpaceBehavior::Break>(lineBreakIteratorFactory, startPosition);
        return BreakablePositions::next<BreakablePositions::LineBreakRules::Normal, BreakablePositions::WordBreakBehavior::Normal, BreakablePositions::NoBreakSpaceBehavior::Normal>(lineBreakIteratorFactory, startPosition);
    }

    if (breakNBSP)
        return BreakablePositions::next<BreakablePositions::LineBreakRules::Special, BreakablePositions::WordBreakBehavior::Normal, BreakablePositions::NoBreakSpaceBehavior::Break>(lineBreakIteratorFactory, startPosition);

    return BreakablePositions::next<BreakablePositions::LineBreakRules::Special, BreakablePositions::WordBreakBehavior::Normal, BreakablePositions::NoBreakSpaceBehavior::Normal>(lineBreakIteratorFactory, startPosition);
}

bool TextUtil::shouldPreserveSpacesAndTabs(const Box& layoutBox)
{
    // https://www.w3.org/TR/css-text-4/#white-space-collapsing
    auto whitespaceCollapse = layoutBox.style().whiteSpaceCollapse();
    return whitespaceCollapse == WhiteSpaceCollapse::Preserve || whitespaceCollapse == WhiteSpaceCollapse::BreakSpaces;
}

bool TextUtil::shouldPreserveNewline(const Box& layoutBox)
{
    // https://www.w3.org/TR/css-text-4/#white-space-collapsing
    auto whitespaceCollapse = layoutBox.style().whiteSpaceCollapse();
    return whitespaceCollapse == WhiteSpaceCollapse::Preserve || whitespaceCollapse == WhiteSpaceCollapse::PreserveBreaks || whitespaceCollapse == WhiteSpaceCollapse::BreakSpaces;
}

bool TextUtil::isWrappingAllowed(const Style::ComputedStyle& style)
{
    // https://www.w3.org/TR/css-text-4/#text-wrap
    return style.textWrapMode() != TextWrapMode::NoWrap;
}

EnumSet<TextUtil::WordBreakRule> TextUtil::wordBreakBehavior(const Style::ComputedStyle& style, bool hasWrapOpportunityAtPreviousPosition, IsMinimumInIntrinsicWidthMode isMinimumInIntrinsicWidthMode, HyphenationIsDisabled hyphenationIsDisabled)
{
    // Disregard any prohibition against line breaks mandated by the word-break property.
    // The different wrapping opportunities must not be prioritized.
    // Note hyphenation is not applied.
    if (style.lineBreak() == LineBreak::Anywhere)
        return { WordBreakRule::AtArbitraryPosition };

    // Breaking is allowed within “words”.
    if (style.wordBreak() == WordBreak::BreakAll)
        return { WordBreakRule::AtArbitraryPositionWithinWords };

    auto includeHyphenationIfAllowed = [&](std::optional<WordBreakRule> wordBreakRule) -> EnumSet<WordBreakRule> {
        auto hyphenationIsAllowed = hyphenationIsDisabled == HyphenationIsDisabled::No && style.hyphens() == Hyphens::Auto && canHyphenate(Style::toPlatform(style.usedLocale()));
        if (hyphenationIsAllowed) {
            if (wordBreakRule)
                return { *wordBreakRule, WordBreakRule::AtHyphenationOpportunities };
            return { WordBreakRule::AtHyphenationOpportunities };
        }
        if (wordBreakRule)
            return *wordBreakRule;
        return { };
    };

    // For compatibility with legacy content, the word-break property also supports a deprecated break-word keyword.
    // When specified, this has the same effect as word-break: normal and overflow-wrap: anywhere, regardless of the actual value of the overflow-wrap property.
    if (style.wordBreak() == WordBreak::BreakWord && !hasWrapOpportunityAtPreviousPosition)
        return includeHyphenationIfAllowed(WordBreakRule::AtArbitraryPosition);
    // OverflowWrap::BreakWord/Anywhere An otherwise unbreakable sequence of characters may be broken at an arbitrary point if there are no otherwise-acceptable break points in the line.
    // Note that this applies to content where CSS properties (e.g. WordBreak::KeepAll) make it unbreakable.
    // Soft wrap opportunities introduced by overflow-wrap/word-wrap: break-word are not considered when calculating min-content intrinsic sizes.
    auto overflowWrapBreakWordIsApplicable = isMinimumInIntrinsicWidthMode == IsMinimumInIntrinsicWidthMode::No;
    if (((overflowWrapBreakWordIsApplicable && style.overflowWrap() == OverflowWrap::BreakWord) || style.overflowWrap() == OverflowWrap::Anywhere) && !hasWrapOpportunityAtPreviousPosition)
        return includeHyphenationIfAllowed(WordBreakRule::AtArbitraryPosition);
    // Breaking is forbidden within “words”.
    if (style.wordBreak() == WordBreak::KeepAll)
        return { };
    return includeHyphenationIfAllowed({ });
}

bool TextUtil::shouldTrailingWhitespaceHang(const Style::ComputedStyle& style)
{
    // https://www.w3.org/TR/css-text-4/#white-space-phase-2
    return style.whiteSpaceCollapse() == WhiteSpaceCollapse::Preserve && style.textWrapMode() != TextWrapMode::NoWrap;
}

TextBreakIterator::LineMode::Behavior TextUtil::lineBreakIteratorMode(LineBreak lineBreak)
{
    switch (lineBreak) {
    case LineBreak::Auto:
    case LineBreak::AfterWhiteSpace:
    case LineBreak::Anywhere:
        return TextBreakIterator::LineMode::Behavior::Default;
    case LineBreak::Loose:
        return TextBreakIterator::LineMode::Behavior::Loose;
    case LineBreak::Normal:
        return TextBreakIterator::LineMode::Behavior::Normal;
    case LineBreak::Strict:
        return TextBreakIterator::LineMode::Behavior::Strict;
    }
    ASSERT_NOT_REACHED();
    return TextBreakIterator::LineMode::Behavior::Default;
}

TextBreakIterator::ContentAnalysis TextUtil::contentAnalysis(WordBreak wordBreak)
{
    switch (wordBreak) {
    case WordBreak::Normal:
    case WordBreak::BreakAll:
    case WordBreak::KeepAll:
    case WordBreak::BreakWord:
        return TextBreakIterator::ContentAnalysis::Mechanical;
    case WordBreak::AutoPhrase:
        return TextBreakIterator::ContentAnalysis::Linguistic;
    }
    return TextBreakIterator::ContentAnalysis::Mechanical;
}

// True if the character may need the Bidi reordering. If false, the
// `Bidi_Class` of `ch` isn't `R`, `AL`, nor Bidi controls.
// https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=%5B%5B%3Abc%3DR%3A%5D%5B%3Abc%3DAL%3A%5D%5D&g=bc
// https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=[:Bidi_C:]
static ALWAYS_INLINE bool NODELETE mayBeBidiRTL(char32_t ch)
{
    if (ch < 0x0590)
        return false;
    // General Punctuation such as curly quotes.
    if (ch >= 0x2010 && ch <= 0x2029)
        return false;
    // CJK etc., up to Surrogate Pairs.
    if (ch >= 0x206A && ch <= 0xD7FF)
        return false;
    // Common in CJK.
    if (ch >= 0xFF00 && ch <= 0xFFFF)
        return false;
    return true;
}

bool TextUtil::isStrongDirectionalityCharacter(char32_t character)
{
    if (!mayBeBidiRTL(character))
        return false;

    auto bidiCategory = u_charDirection(character);
    return bidiCategory == U_RIGHT_TO_LEFT
        || bidiCategory == U_RIGHT_TO_LEFT_ARABIC
        || bidiCategory == U_RIGHT_TO_LEFT_EMBEDDING
        || bidiCategory == U_RIGHT_TO_LEFT_OVERRIDE
        || bidiCategory == U_LEFT_TO_RIGHT_EMBEDDING
        || bidiCategory == U_LEFT_TO_RIGHT_OVERRIDE
        || bidiCategory == U_POP_DIRECTIONAL_FORMAT;
}

template<typename CharacterType> ALWAYS_INLINE constexpr bool isNotBidiRTL(CharacterType character)
{
    return !mayBeBidiRTL(character);
}

bool TextUtil::containsStrongDirectionalityText(StringView text)
{
    if (text.is8Bit())
        return false;

    if (![&](auto span) ALWAYS_INLINE_LAMBDA {
        using UnsignedType = std::make_unsigned_t<typename decltype(span)::value_type>;
        constexpr size_t stride = SIMD::stride<UnsignedType>;
        if (span.size() >= stride) {
            constexpr auto c0590 = SIMD::splat<UnsignedType>(0x0590);
            constexpr auto c2010 = SIMD::splat<UnsignedType>(0x2010);
            constexpr auto c2029 = SIMD::splat<UnsignedType>(0x2029);
            constexpr auto c206A = SIMD::splat<UnsignedType>(0x206A);
            constexpr auto cD7FF = SIMD::splat<UnsignedType>(0xD7FF);
            constexpr auto cFF00 = SIMD::splat<UnsignedType>(0xFF00);
            auto maybeBidiRTL = [&](auto span) ALWAYS_INLINE_LAMBDA {
                auto input = SIMD::load(std::bit_cast<const UnsignedType*>(span.data()));
                // ch < 0x0590
                auto cond0 = SIMD::lessThan(input, c0590);
                // General Punctuation such as curly quotes.
                // ch >= 0x2010 && ch <= 0x2029
                auto cond1 = SIMD::bitAnd(SIMD::greaterThanOrEqual(input, c2010), SIMD::lessThanOrEqual(input, c2029));
                // CJK etc., up to Surrogate Pairs.
                // ch >= 0x206A && ch <= 0xD7FF
                auto cond2 = SIMD::bitAnd(SIMD::greaterThanOrEqual(input, c206A), SIMD::lessThanOrEqual(input, cD7FF));
                // Common in CJK.
                // ch >= 0xFF00 && ch <= 0xFFFF
                auto cond3 = SIMD::greaterThanOrEqual(input, cFF00);
                return SIMD::bitNot(SIMD::bitOr(cond0, cond1, cond2, cond3));
            };

            auto finalStride = span.last(stride);
            auto result = SIMD::splat<UnsignedType>(0);
            for (; stride < span.size(); skip(span, stride))
                result = SIMD::bitOr(result, maybeBidiRTL(span));
            if (!span.empty())
                result = SIMD::bitOr(result, maybeBidiRTL(finalStride));
            return SIMD::isNonZero(result);
        }

        for (auto character : span) {
            if (mayBeBidiRTL(character))
                return true;
        }
        return false;
    }(text.span16()))
        return false;

    for (char32_t character : text.codePoints()) {
        if (isStrongDirectionalityCharacter(character))
            return true;
    }

    return false;
}

size_t TextUtil::firstUserPerceivedCharacterLength(const InlineTextBox& inlineTextBox, size_t startPosition, size_t length)
{
    auto& textContent = inlineTextBox.content();
    RELEASE_ASSERT(!textContent.isEmpty());

    if (textContent.is8Bit())
        return 1;
    if (inlineTextBox.canUseSimpleFontCodePath()) {
        char32_t character;
        size_t endOfCodePoint = startPosition;
        auto characters = textContent.span16();
        U16_NEXT(characters, endOfCodePoint, textContent.length(), character);
        ASSERT(endOfCodePoint > startPosition);
        return endOfCodePoint - startPosition;
    }
    auto graphemeClustersIterator = NonSharedCharacterBreakIterator { textContent };
    auto nextPosition = ubrk_following(graphemeClustersIterator, startPosition);
    if (nextPosition == UBRK_DONE)
        return length;
    return nextPosition - startPosition;
}

size_t TextUtil::firstUserPerceivedCharacterLength(const InlineTextItem& inlineTextItem)
{
    auto length = firstUserPerceivedCharacterLength(protect(inlineTextItem.inlineTextBox()), inlineTextItem.start(), inlineTextItem.length());
    return std::min<size_t>(inlineTextItem.length(), length);
}

TextDirection TextUtil::directionForTextContent(StringView content)
{
    return baseTextDirection(content).value_or(TextDirection::LTR);
}

AtomString TextUtil::ellipsisTextInInlineDirection(bool isHorizontal)
{
    if (isHorizontal) {
        static MainThreadNeverDestroyed<const AtomString> horizontalEllipsisStr(span(horizontalEllipsis));
        return horizontalEllipsisStr;
    }
    static MainThreadNeverDestroyed<const AtomString> verticalEllipsisStr(span(verticalEllipsis));
    return verticalEllipsisStr;
}

InlineLayoutUnit TextUtil::hyphenWidth(const Style::ComputedStyle& style)
{
    return std::max(0.f, protect(style.fontCascade())->width(StringView { style.hyphenString() }));
}

static bool NODELETE isASCIIHangableQuote(char32_t character)
{
    return character == quotationMark || character == apostrophe;
}

static bool isHangableOpenPunctuation(char32_t character)
{
    // https://drafts.csswg.org/css-text-3/#hanging-punctuation-property
    if (isASCIIHangableQuote(character) || character == ideographicSpace)
        return true;
    return U_GET_GC_MASK(character) & (U_GC_PS_MASK | U_GC_PI_MASK | U_GC_PF_MASK);
}

static bool isHangableClosePunctuation(char32_t character)
{
    // https://drafts.csswg.org/css-text-3/#hanging-punctuation-property
    if (isASCIIHangableQuote(character))
        return true;
    return U_GET_GC_MASK(character) & (U_GC_PE_MASK | U_GC_PI_MASK | U_GC_PF_MASK);
}

bool TextUtil::hasHangablePunctuationStart(const InlineTextItem& inlineTextItem, const Style::ComputedStyle& style)
{
    if (!inlineTextItem.length() || !style.hangingPunctuation().contains(Style::HangingPunctuationValue::First))
        return false;
    auto leadingCharacter = inlineTextItem.inlineTextBox().content()[inlineTextItem.start()];
    return isHangableOpenPunctuation(leadingCharacter);
}

float TextUtil::hangablePunctuationStartWidth(const InlineTextItem& inlineTextItem, const Style::ComputedStyle& style)
{
    if (!hasHangablePunctuationStart(inlineTextItem, style))
        return { };
    ASSERT(inlineTextItem.length());
    auto leadingPosition = inlineTextItem.start();
    return width(inlineTextItem, protect(style.fontCascade()), leadingPosition, leadingPosition + 1, { });
}

bool TextUtil::hasHangablePunctuationEnd(const InlineTextItem& inlineTextItem, const Style::ComputedStyle& style)
{
    if (!inlineTextItem.length() || !style.hangingPunctuation().contains(Style::HangingPunctuationValue::Last))
        return false;
    auto trailingCharacter = inlineTextItem.inlineTextBox().content()[inlineTextItem.end() - 1];
    return isHangableClosePunctuation(trailingCharacter);
}

float TextUtil::hangablePunctuationEndWidth(const InlineTextItem& inlineTextItem, const Style::ComputedStyle& style)
{
    if (!hasHangablePunctuationEnd(inlineTextItem, style))
        return { };
    ASSERT(inlineTextItem.length());
    auto trailingPosition = inlineTextItem.end() - 1;
    return width(inlineTextItem, protect(style.fontCascade()), trailingPosition, trailingPosition + 1, { });
}

bool TextUtil::hasHangableStopOrCommaEnd(const InlineTextItem& inlineTextItem, const Style::ComputedStyle& style)
{
    if (!inlineTextItem.length() || !style.hangingPunctuation().containsAny({ Style::HangingPunctuationValue::AllowEnd, Style::HangingPunctuationValue::ForceEnd }))
        return false;
    auto trailingPosition = inlineTextItem.end() - 1;
    auto trailingCharacter = inlineTextItem.inlineTextBox().content()[trailingPosition];
    auto isHangableStopOrComma = trailingCharacter == 0x002C
        || trailingCharacter == 0x002E || trailingCharacter == 0x060C
        || trailingCharacter == 0x06D4 || trailingCharacter == 0x3001
        || trailingCharacter == 0x3002 || trailingCharacter == 0xFF0C
        || trailingCharacter == 0xFF0E || trailingCharacter == 0xFE50
        || trailingCharacter == 0xFE51 || trailingCharacter == 0xFE52
        || trailingCharacter == 0xFF61 || trailingCharacter == 0xFF64;
    return isHangableStopOrComma;
}

float TextUtil::hangableStopOrCommaEndWidth(const InlineTextItem& inlineTextItem, const Style::ComputedStyle& style)
{
    if (!hasHangableStopOrCommaEnd(inlineTextItem, style))
        return { };
    ASSERT(inlineTextItem.length());
    auto trailingPosition = inlineTextItem.end() - 1;
    return width(inlineTextItem, protect(style.fontCascade()), trailingPosition, trailingPosition + 1, { });
}

template<typename CharacterType>
static bool canUseSimplifiedTextMeasuringForCharacters(std::span<const CharacterType> characters, const FontCascade& fontCascade, const Font& primaryFont, bool whitespaceIsCollapsed)
{
    for (auto character : characters) {
        if (!fontCascade.canUseSimplifiedTextMeasuring(character, FontVariant::Auto, whitespaceIsCollapsed, primaryFont))
            return false;
    }
    return true;
}

bool TextUtil::canUseSimplifiedTextMeasuring(StringView textContent, const FontCascade& fontCascade, bool whitespaceIsCollapsed, const Style::ComputedStyle* firstLineStyle)
{
    ASSERT(textContent.is8Bit() || FontCascade::characterRangeCodePath(textContent.span16()) == FontCascade::CodePath::Simple);
    // FIXME: All these checks should be more fine-grained at the inline item level.
    if (fontCascade.wordSpacing() || fontCascade.letterSpacing())
        return false;

#if USE(FONT_VARIANT_VIA_FEATURES)
    auto fontVariantCaps = fontCascade.fontDescription().variantCaps();
    if (fontVariantCaps == FontVariantCaps::Small || fontVariantCaps == FontVariantCaps::AllSmall || fontVariantCaps ==  FontVariantCaps::Petite || fontVariantCaps == FontVariantCaps::AllPetite)
        return false;
#endif

    // Additional check on the font codepath.
    auto run = TextRun { textContent };
    run.setCharacterScanForCodePath(false);
    if (fontCascade.codePath(run) != FontCascade::CodePath::Simple)
        return false;

    if (firstLineStyle && fontCascade != firstLineStyle->fontCascade())
        return false;

    Ref primaryFont = fontCascade.primaryFont();

    if (textContent.is8Bit())
        return canUseSimplifiedTextMeasuringForCharacters(textContent.span8(), fontCascade, primaryFont, whitespaceIsCollapsed);
    return canUseSimplifiedTextMeasuringForCharacters(textContent.span16(), fontCascade, primaryFont, whitespaceIsCollapsed);
}

bool TextUtil::hasPositionDependentContentWidth(StringView textContent)
{
    if (textContent.is8Bit())
        return charactersContain<Latin1Character, tabCharacter>(textContent.span8());
    return charactersContain<char16_t, tabCharacter>(textContent.span16());
}

SUPPRESS_NODELETE char32_t TextUtil::lastBaseCharacterFromText(StringView string)
{
    for (auto codePoint : string.codePoints() | std::views::reverse) {
        if (!isCombiningMark(codePoint))
            return codePoint;
    }
    return 0;
}

}
}
```

## TextUtil.h

Original: [complete immutable source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/text/TextUtil.h). Path: `Source/WebCore/layout/formattingContexts/inline/text/TextUtil.h`. Source bytes: 6884; source lines: 134; SHA-256: `e7c72ae837e70d0669f2103aa3e4efc8952745f245bb0b16e8df46a98d745df5`.

```cpp
/*
 * Copyright (C) 2018-2023 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include <WebCore/Font.h>
#include <WebCore/InlineItem.h>
#include <WebCore/InlineLine.h>
#include <WebCore/LayoutUnits.h>
#include <WebCore/TextSpacing.h>
#include <wtf/EnumSet.h>
#include <wtf/Range.h>
#include <wtf/WeakHashSet.h>
#include <wtf/text/TextBreakIterator.h>

namespace WebCore {

namespace Style {
class ComputedStyle;
}

namespace TextSpacing {
struct SpacingState;
}

struct GlyphOverflow;
class FontCascade;
class TextRun;

namespace Layout {

struct ExpansionInfo;
class InlineTextBox;
class InlineTextItem;

class TextUtil {
public:
    enum class WordBreakRule : uint8_t {
        AtArbitraryPositionWithinWords,
        AtArbitraryPosition,
        AtHyphenationOpportunities
    };
    enum class IsMinimumInIntrinsicWidthMode : bool { No, Yes };
    enum class HyphenationIsDisabled : bool { No, Yes };
    static EnumSet<WordBreakRule> wordBreakBehavior(const Style::ComputedStyle&, bool hasWrapOpportunityAtPreviousPosition, IsMinimumInIntrinsicWidthMode, HyphenationIsDisabled);

    enum class UseTrailingWhitespaceMeasuringOptimization : bool { No, Yes };
    static InlineLayoutUnit width(const InlineTextItem&, const FontCascade&, InlineLayoutUnit contentLogicalLeft);
    static InlineLayoutUnit width(const InlineTextItem&, const FontCascade&, unsigned from, unsigned to, InlineLayoutUnit contentLogicalLeft, UseTrailingWhitespaceMeasuringOptimization = UseTrailingWhitespaceMeasuringOptimization::Yes, TextSpacing::SpacingState spacingState = { }, GlyphOverflow* = nullptr);
    static InlineLayoutUnit width(const InlineTextBox&, const FontCascade&, unsigned from, unsigned to, InlineLayoutUnit contentLogicalLeft, UseTrailingWhitespaceMeasuringOptimization = UseTrailingWhitespaceMeasuringOptimization::Yes, TextSpacing::SpacingState spacingState = { }, GlyphOverflow* = nullptr);

    static InlineLayoutUnit trailingWhitespaceWidth(const InlineTextBox&, const FontCascade&, size_t startPosition, size_t endPosition);
    static InlineLayoutUnit singleSpaceWidth(const FontCascade&, bool canUseSimplifiedContentMeasuring);

    using FallbackFontList = SingleThreadWeakHashSet<const Font>;
    enum class IncludeHyphen : bool { No, Yes };
    static FallbackFontList fallbackFontsForText(StringView, const Style::ComputedStyle&, IncludeHyphen);

    struct EnclosingAscentDescent {
        InlineLayoutUnit ascent { 0.f };
        InlineLayoutUnit descent { 0.f };
    };
    enum class ShouldUseSimpleGlyphOverflowCodePath : bool { No, Yes };
    static EnclosingAscentDescent enclosingGlyphBoundsForText(StringView, const Style::ComputedStyle&, ShouldUseSimpleGlyphOverflowCodePath);

    struct WordBreakLeft {
        size_t length { 0 };
        InlineLayoutUnit logicalWidth { 0 };
    };
    static WordBreakLeft breakWord(const InlineTextBox&, size_t start, size_t length, InlineLayoutUnit width, InlineLayoutUnit availableWidth, InlineLayoutUnit contentLogicalLeft, const FontCascade&);
    static WordBreakLeft breakWord(const InlineTextItem&, const FontCascade&, InlineLayoutUnit textWidth, InlineLayoutUnit availableWidth, InlineLayoutUnit contentLogicalLeft);

    static bool mayBreakInBetween(const InlineTextItem& previousInlineItem, const InlineTextItem& nextInlineItem);
    // FIXME: Remove when computeInlineIntrinsicLogicalWidths is all IFC.
    static bool mayBreakInBetween(String previousContent, const Style::ComputedStyle& previousContentStyle, String nextContent, const Style::ComputedStyle& nextContentStyle);
    static unsigned findNextBreakablePosition(CachedLineBreakIteratorFactory&, unsigned startPosition, const Style::ComputedStyle&);
    static TextBreakIterator::LineMode::Behavior NODELETE lineBreakIteratorMode(LineBreak);
    static TextBreakIterator::ContentAnalysis NODELETE contentAnalysis(WordBreak);

    static bool NODELETE shouldPreserveSpacesAndTabs(const Box&);
    static bool NODELETE shouldPreserveNewline(const Box&);
    static bool NODELETE isWrappingAllowed(const Style::ComputedStyle&);
    static bool NODELETE shouldTrailingWhitespaceHang(const Style::ComputedStyle&);

    static bool isStrongDirectionalityCharacter(char32_t);
    static bool containsStrongDirectionalityText(StringView);

    static AtomString ellipsisTextInInlineDirection(bool isHorizontal = true);

    static InlineLayoutUnit hyphenWidth(const Style::ComputedStyle&);

    static size_t firstUserPerceivedCharacterLength(const InlineTextItem&);
    static size_t firstUserPerceivedCharacterLength(const InlineTextBox&, size_t startPosition, size_t length);
    static TextDirection directionForTextContent(StringView);

    static bool hasHangablePunctuationStart(const InlineTextItem&, const Style::ComputedStyle&);
    static float hangablePunctuationStartWidth(const InlineTextItem&, const Style::ComputedStyle&);

    static bool hasHangablePunctuationEnd(const InlineTextItem&, const Style::ComputedStyle&);
    static float hangablePunctuationEndWidth(const InlineTextItem&, const Style::ComputedStyle&);

    static bool hasHangableStopOrCommaEnd(const InlineTextItem&, const Style::ComputedStyle&);
    static float hangableStopOrCommaEndWidth(const InlineTextItem&, const Style::ComputedStyle&);

    static bool canUseSimplifiedTextMeasuring(StringView, const FontCascade&, bool whitespaceIsCollapsed, const Style::ComputedStyle* firstLineStyle);
    static bool hasPositionDependentContentWidth(StringView);

    static char32_t NODELETE lastBaseCharacterFromText(StringView);
};

} // namespace Layout
} // namespace WebCore
```
