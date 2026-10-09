//! Exact typed inverses for schema-owned shorthand families.

use crate::*;
use crate::{
    declaration_serialization::{append_break_between, append_ordinary, append_text_orientation},
    expansion::SpecifiedInverseValue as Inverse,
    specified_declaration_block::{
        CssDeclarationBlockError, CssSpecifiedDeclarationEntry, CssSpecifiedDeclarationValueRef,
    },
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssDeclarationBlockError>;

type EqualityResult = std::result::Result<bool, CssSpecifiedValueSerializationError>;
use crate::specified_serialization::SpecifiedSerializationContext;

mod equality;
pub(crate) use equality::BoundedStructuralEquality;

trait InverseValueEquality {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult;
}
macro_rules! literal_equality {
    ($($ty:ty),+ $(,)?) => { $(impl InverseValueEquality for $ty {
        fn inverse_equal(&self, other: &Self, context: &mut SpecifiedSerializationContext) -> EqualityResult {
            match (self.literal_component(), other.literal_component()) {
                (Some(a), Some(b)) => Ok(crate::specified_numeric::ordinary_length_literal_equal(a, b)),
                _ => match (self.calculation(), other.calculation()) {
                    (Some(a), Some(b)) => a.expression.specified_inverse_eq(&b.expression, context),
                    _ => Ok(false),
                },
            }
        }
    })+ };
}
literal_equality!(
    CssSpecifiedLength,
    CssSpecifiedNonNegativeLength,
    CssSpecifiedLengthPercentage,
    CssSpecifiedNonNegativeLengthPercentage
);
macro_rules! ordinary_number_equality {
    ($($ty:ty),+ $(,)?) => { $(impl InverseValueEquality for $ty {
        fn inverse_equal(&self, other: &Self, context: &mut SpecifiedSerializationContext) -> EqualityResult {
            match (self.literal_component(), other.literal_component()) {
                (Some(a), Some(b)) => Ok(crate::specified_numeric::ordinary_literal_equal(a, b)),
                _ => match (self.calculation(), other.calculation()) {
                    (Some(a), Some(b)) => a.expression.specified_inverse_eq(&b.expression, context),
                    _ => Ok(false),
                },
            }
        }
    })+ };
}
ordinary_number_equality!(
    CssSpecifiedNonNegativeNumber,
    CssSpecifiedNonNegativePercentage
);
impl InverseValueEquality for CssCornerRadiusValue {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        Ok(self
            .horizontal()
            .inverse_equal(other.horizontal(), context)?
            && self.vertical().inverse_equal(other.vertical(), context)?)
    }
}
impl InverseValueEquality for CssBackgroundSizeComponent {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Length(a), Self::Length(b)) => a.inverse_equal(b, context),
            _ => Ok(self == other),
        }
    }
}
impl InverseValueEquality for CssBackgroundSize {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (
                Self::Explicit {
                    width: a,
                    height: ah,
                },
                Self::Explicit {
                    width: b,
                    height: bh,
                },
            ) => Ok(a.inverse_equal(b, context)?
                && ah
                    .as_ref()
                    .unwrap_or(&CssBackgroundSizeComponent::Auto)
                    .inverse_equal(
                        bh.as_ref().unwrap_or(&CssBackgroundSizeComponent::Auto),
                        context,
                    )?),
            _ => Ok(self == other),
        }
    }
}
fn numeric_items_equal<T: InverseValueEquality>(
    left: &[T],
    right: &[T],
    context: &mut SpecifiedSerializationContext,
) -> EqualityResult {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        if !left.inverse_equal(right, context)? {
            return Ok(false);
        }
    }
    Ok(true)
}
impl InverseValueEquality for CssBackgroundSizeList {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        numeric_items_equal(self.sizes(), other.sizes(), context)
    }
}
impl InverseValueEquality for CssBorderImageSliceComponent {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.inverse_equal(b, context),
            (Self::HintedNumberCalculation(a), Self::HintedNumberCalculation(b)) => {
                a.bounded_eq(b, context)
            }
            (Self::Percentage(a), Self::Percentage(b)) => a.inverse_equal(b, context),
            _ => Ok(self == other),
        }
    }
}
impl InverseValueEquality for CssBorderImageSlice {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        Ok(self.fill() == other.fill()
            && numeric_items_equal(self.values(), other.values(), context)?)
    }
}
impl InverseValueEquality for CssBorderImageWidthComponent {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::LengthPercentage(a), Self::LengthPercentage(b)) => a.inverse_equal(b, context),
            (Self::Number(a), Self::Number(b)) => a.inverse_equal(b, context),
            (Self::HintedNumberCalculation(a), Self::HintedNumberCalculation(b)) => {
                a.bounded_eq(b, context)
            }
            _ => Ok(self == other),
        }
    }
}
impl InverseValueEquality for CssBorderImageWidth {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        numeric_items_equal(self.values(), other.values(), context)
    }
}
impl InverseValueEquality for CssBorderImageOutsetComponent {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Length(a), Self::Length(b)) => a.inverse_equal(b, context),
            (Self::Number(a), Self::Number(b)) => a.inverse_equal(b, context),
            _ => Ok(self == other),
        }
    }
}
impl InverseValueEquality for CssBorderImageOutset {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        numeric_items_equal(self.values(), other.values(), context)
    }
}
macro_rules! numeric_variant_equality {
    ($ty:ty, $variant:ident) => {
        impl InverseValueEquality for $ty {
            fn inverse_equal(
                &self,
                other: &Self,
                context: &mut SpecifiedSerializationContext,
            ) -> EqualityResult {
                match (self, other) {
                    (Self::$variant(a), Self::$variant(b)) => a.inverse_equal(b, context),
                    _ => Ok(self == other),
                }
            }
        }
    };
}
numeric_variant_equality!(CssMarginValue, LengthPercentage);
numeric_variant_equality!(CssInsetValue, LengthPercentage);
numeric_variant_equality!(CssScrollPaddingValue, LengthPercentage);
numeric_variant_equality!(CssGapValue, LengthPercentage);
numeric_variant_equality!(CssBorderWidth, Length);
impl InverseValueEquality for CssPaddingValue {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        self.length_percentage()
            .inverse_equal(other.length_percentage(), context)
    }
}
impl InverseValueEquality for CssBoxSize {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::LengthPercentage(a), Self::LengthPercentage(b))
            | (Self::FitContentFunction(a), Self::FitContentFunction(b)) => {
                a.inverse_equal(b, context)
            }
            (Self::CalcSize(a), Self::CalcSize(b)) => a
                .as_calc_size()
                .specified_inverse_eq(b.as_calc_size(), context),
            _ => Ok(self == other),
        }
    }
}
numeric_variant_equality!(CssSizeValue, BoxSize);
impl InverseValueEquality for CssMaxSizeValue {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.box_size(), other.box_size()) {
            (Some(a), Some(b)) => a.inverse_equal(b, context),
            _ => Ok(self == other),
        }
    }
}
impl InverseValueEquality for CssContainIntrinsicSizeValue {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        Ok(self.uses_auto() == other.uses_auto()
            && match (self.fallback(), other.fallback()) {
                (
                    CssContainIntrinsicSizeFallback::Length(a),
                    CssContainIntrinsicSizeFallback::Length(b),
                ) => a.inverse_equal(b, context)?,
                (a, b) => a == b,
            })
    }
}
impl InverseValueEquality for CssSpeechBreak {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Time(a), Self::Time(b)) => a.specified_inverse_eq(b, context),
            (Self::None, Self::Time(b)) | (Self::Time(b), Self::None) => {
                Ok(b.is_exact_ordinary_zero())
            }
            _ => Ok(self == other),
        }
    }
}
impl InverseValueEquality for CssInterestDelayValue {
    fn inverse_equal(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Time(a), Self::Time(b)) => a.specified_inverse_eq(b, context),
            _ => Ok(self == other),
        }
    }
}
macro_rules! structural_equality {
    ($($ty:ty),+ $(,)?) => { $(impl InverseValueEquality for $ty {
        fn inverse_equal(&self, other: &Self, _context: &mut SpecifiedSerializationContext) -> EqualityResult { Ok(self == other) }
    })+ };
}
structural_equality!(CssBorderStyle, CssOverflow);
impl InverseValueEquality for CssCue {
    fn inverse_equal(
        &self,
        other: &Self,
        _context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        Ok(self.specified_inverse_eq(other))
    }
}
macro_rules! inverse_value_equal {
    (CssFontSize, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssFontStyle, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssFontWeight, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssLineHeight, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssColumnCount, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssLineWidth, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssColumnWidth, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssGridLine, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssGridTrackList, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssGridTrackSizeList, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssCue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssFlexBasisValue, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssFlowTolerance, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssTextDecorationThickness, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssOutlineWidth, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssEasingList, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssBackgroundPositionList, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssPhysicalPositionList, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssImageValue, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssImageValueList, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssListStyleTypeValue, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssOffsetPosition, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssOffsetAnchor, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssOffsetRotate, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssOffsetPath, $a:ident, $b:ident, $context:ident) => {
        $a.bounded_eq($b, $context)
    };
    (CssSpecifiedLength, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssSpecifiedNonNegativeLength, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssSpecifiedLengthPercentage, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssSpecifiedNonNegativeLengthPercentage, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssSpecifiedNonNegativeNumber, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssCornerRadiusValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssBackgroundSizeList, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssBorderImageSlice, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssBorderImageWidth, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssBorderImageOutset, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssMarginValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssInsetValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssScrollPaddingValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssGapValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssPaddingValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssBorderWidth, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssBoxSize, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssSizeValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssMaxSizeValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssContainIntrinsicSizeValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssSpeechBreak, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    (CssInterestDelayValue, $a:ident, $b:ident, $context:ident) => {
        $a.inverse_equal($b, $context)
    };
    ($ty:ident, $a:ident, $b:ident, $context:ident) => {
        Ok($a == $b)
    };
}

macro_rules! fallible_inverse_value_equal {
    (CssOutlineColor, $a:ident, $b:ident, $context:ident) => {
        match ($a, $b) {
            (CssOutlineColor::Color(a), CssOutlineColor::Color(b)) => {
                a.specified_inverse_eq(b, $context)
            }
            _ => Ok($a == $b),
        }
    };
    (CssCaretColor, $a:ident, $b:ident, $context:ident) => {
        match ($a, $b) {
            (CssCaretColor::Color(a), CssCaretColor::Color(b)) => {
                a.specified_inverse_eq(b, $context)
            }
            _ => Ok($a == $b),
        }
    };
    (CssColor, $a:ident, $b:ident, $context:ident) => {
        $a.specified_inverse_eq($b, $context)
    };
    ($ty:ident, $a:ident, $b:ident, $context:ident) => {
        inverse_value_equal!($ty, $a, $b, $context)
    };
}

macro_rules! longhand_dispatch {
    ($input:ident, $numeric:ident;
        All, $all_name:literal, [$($all_alias:literal),*], $all_id:literal,
        $all_value:ty, $all_parser:ident, $all_dispatch:block, expansion = $all_kind:ident { $($all_metadata:tt)* };
        $( $variant:ident, $name:literal, [$($alias:literal),*], $id:literal,
           crate::$value:ident, $wrapper:ident, $accessor:ident, $parser:ident, $dispatch:block,
           expansion = $kind:ident { $($metadata:tt)* }; )*
    ) => { longhand_dispatch!(@collect []; $( $variant, $value, $kind; )*); };
    (@collect [$($rows:tt)*]; $variant:ident, $value:ident, longhand; $($rest:tt)*) => {
        longhand_dispatch!(@collect [$($rows)* ($variant, $value)]; $($rest)*);
    };
    (@collect [$($rows:tt)*]; $variant:ident, $value:ident, $kind:ident; $($rest:tt)*) => {
        longhand_dispatch!(@collect [$($rows)*]; $($rest)*);
    };
    (@collect [$(($variant:ident, $value:ident))*];) => {
        fn ordinary_values_equal(left: CssLonghandValueRef<'_>, right: CssLonghandValueRef<'_>, context: &mut crate::specified_serialization::SpecifiedSerializationContext) -> std::result::Result<bool, CssSpecifiedValueSerializationError> {
            match (left, right) {
                $((CssLonghandValueRef::$variant(a), CssLonghandValueRef::$variant(b)) => fallible_inverse_value_equal!($value, a, b, context),)*
                _ => Ok(false),
            }
        }
        pub(crate) fn append_longhand(value: CssLonghandValueRef<'_>, writer: &mut SpecifiedRuleWriter)
            -> std::result::Result<(), CssSpecifiedValueSerializationError> {
            match value {
                $(CssLonghandValueRef::$variant(value) => {
                    let _grammar = CssKnownProperty::$variant.grammar();
                    append_ordinary!($variant, $value, value, _grammar, writer)
                },)*
            }
        }
    };
}
crate::properties::property_schema!(longhand_dispatch, inverse_input, inverse_numeric);

pub(crate) fn visit_value(
    value: CssContributionValueRef<'_>,
    writer: &mut SpecifiedRuleWriter,
) -> std::result::Result<(), CssSpecifiedValueSerializationError> {
    match value {
        CssContributionValueRef::Ordinary(value) => {
            writer.visit_semantic(|writer| append_longhand(value, writer))
        }
        _ => writer.context.charge_projection(1),
    }
}

/// Owning time equality compares exact ordinary units while retaining math phase.
/// Other typed owners' equality remains structural and never compares rounded text.
pub(crate) fn values_equal(
    left: CssContributionValueRef<'_>,
    right: CssContributionValueRef<'_>,
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
) -> std::result::Result<bool, CssSpecifiedValueSerializationError> {
    match (left, right) {
        (
            CssContributionValueRef::Ordinary(CssLonghandValueRef::TransitionDuration(a)),
            CssContributionValueRef::Ordinary(CssLonghandValueRef::TransitionDuration(b)),
        )
        | (
            CssContributionValueRef::Ordinary(CssLonghandValueRef::AnimationDuration(a)),
            CssContributionValueRef::Ordinary(CssLonghandValueRef::AnimationDuration(b)),
        ) => {
            if a.values().len() != b.values().len() {
                return Ok(false);
            }
            for (a, b) in a.values().iter().zip(b.values()) {
                if !a.specified_inverse_eq(b, context)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (
            CssContributionValueRef::Ordinary(CssLonghandValueRef::TransitionDelay(a)),
            CssContributionValueRef::Ordinary(CssLonghandValueRef::TransitionDelay(b)),
        )
        | (
            CssContributionValueRef::Ordinary(CssLonghandValueRef::AnimationDelay(a)),
            CssContributionValueRef::Ordinary(CssLonghandValueRef::AnimationDelay(b)),
        ) => {
            if a.values().len() != b.values().len() {
                return Ok(false);
            }
            for (a, b) in a.values().iter().zip(b.values()) {
                if !a.specified_inverse_eq(b, context)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (
            CssContributionValueRef::Ordinary(CssLonghandValueRef::AnimationIterationCount(a)),
            CssContributionValueRef::Ordinary(CssLonghandValueRef::AnimationIterationCount(b)),
        ) => {
            if a.values().len() != b.values().len() {
                return Ok(false);
            }
            for (a, b) in a.values().iter().zip(b.values()) {
                let equal = match (a, b) {
                    (
                        CssAnimationIterationCount::Number(a),
                        CssAnimationIterationCount::Number(b),
                    ) => a.inverse_equal(b, context)?,
                    _ => a == b,
                };
                if !equal {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (CssContributionValueRef::Ordinary(a), CssContributionValueRef::Ordinary(b)) => {
            ordinary_values_equal(a, b, context)
        }
        _ => Ok(left == right),
    }
}

fn ordinary<'a>(
    values: &[&'a CssSpecifiedDeclarationEntry],
    property: CssKnownProperty,
) -> Option<CssLonghandValueRef<'a>> {
    values.iter().find_map(|entry| match entry.value() {
        CssSpecifiedDeclarationValueRef::Completed(CssContributionValueRef::Ordinary(value))
            if entry.property_name() == CssPropertyNameRef::Known(property) =>
        {
            Some(value)
        }
        _ => None,
    })
}
macro_rules! val {
    ($values:ident, $property:ident) => {{
        let CssLonghandValueRef::$property(value) = ordinary($values, CssKnownProperty::$property)?
        else {
            return None;
        };
        value.clone()
    }};
}
macro_rules! optional_value {
    ($values:ident, $property:ident, $writer:ident, $error:ident) => {{
        let value = val!($values, $property);
        let CssPropertyKindRef::Longhand(meta) = CssKnownProperty::$property
            .metadata()
            .expect("schema terminal")
            .kind()
        else {
            unreachable!("terminal inverse child")
        };
        let initial = meta.initial_value();
        let initial = match initial.view() {
            CssInitialValueRef::Value(value) => value.view(),
            _ => return None,
        };
        match values_equal(
            CssContributionValueRef::Ordinary(CssLonghandValueRef::$property(&value)),
            CssContributionValueRef::Ordinary(initial),
            &mut $writer.context,
        ) {
            Ok(equal) => (!equal).then_some(value),
            Err(error) => {
                let entry = $values
                    .iter()
                    .find(|entry| {
                        entry.property_name()
                            == CssPropertyNameRef::Known(CssKnownProperty::$property)
                    })
                    .expect("selected inverse member");
                $error = Some(entry.at(error.into()));
                return None;
            }
        }
    }};
}
macro_rules! initial_value {
    ($property:ident) => {{
        let CssPropertyKindRef::Longhand(meta) = CssKnownProperty::$property
            .metadata()
            .expect("schema terminal")
            .kind()
        else {
            unreachable!("terminal inverse initial")
        };
        let initial = meta.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            return None;
        };
        let CssLonghandValueRef::$property(value) = initial.view() else {
            unreachable!("schema initial identity")
        };
        value.clone()
    }};
}
fn scroll_mode(mode: CssBoxSideKind) -> CssScrollSideKind {
    match mode {
        CssBoxSideKind::Physical => CssScrollSideKind::Physical,
        CssBoxSideKind::Logical => CssScrollSideKind::Logical,
    }
}
fn shorter_second<T: InverseValueEquality>(
    first: &T,
    second: T,
    context: &mut SpecifiedSerializationContext,
) -> std::result::Result<Option<T>, CssSpecifiedValueSerializationError> {
    Ok((!first.inverse_equal(&second, context)?).then_some(second))
}
fn shortest_quad<T: InverseValueEquality>(
    mut values: Vec<T>,
    context: &mut SpecifiedSerializationContext,
    current: &mut usize,
) -> std::result::Result<Vec<T>, CssSpecifiedValueSerializationError> {
    *current = 3;
    if values[1].inverse_equal(&values[3], context)? {
        values.pop();
        *current = 2;
        if values[0].inverse_equal(&values[2], context)? {
            values.pop();
            *current = 1;
            if values[0].inverse_equal(&values[1], context)? {
                values.pop();
            }
        }
    }
    Ok(values)
}

pub(crate) fn reconstruct(
    property: CssKnownProperty,
    mode: Option<CssBoxSideKind>,
    values: &[&CssSpecifiedDeclarationEntry],
    writer: &mut SpecifiedRuleWriter,
) -> Result<Option<Inverse>> {
    let meta = property.metadata().expect("schema inverse candidate");
    let children = match meta.kind() {
        CssPropertyKindRef::Shorthand(meta) => meta.settable_members().len(),
        CssPropertyKindRef::FourSideShorthand(_) => 4,
        _ => unreachable!("only finite shorthand candidates enter the inverse owner"),
    };
    // Hypothetical aggregate and each child are admitted before typed construction.
    writer.context.charge_projection(1)?;
    // Every settable value is inspected before cloning a hypothetical typed
    // aggregate. Descendant trees use their owning semantic visitors.
    let settable = match meta.kind() {
        CssPropertyKindRef::Shorthand(meta) => meta.settable_members(),
        CssPropertyKindRef::FourSideShorthand(meta) => {
            meta.settable_members(mode.expect("selected four-side mode"))
        }
        _ => unreachable!("finite inverse family"),
    };
    debug_assert_eq!(settable.len(), children);
    for property in settable {
        let entry = values
            .iter()
            .find(|entry| {
                entry.property_name() == CssPropertyNameRef::Known(property.known_property())
            })
            .expect("eligible member");
        let CssSpecifiedDeclarationValueRef::Completed(value) = entry.value() else {
            return Ok(None);
        };
        visit_value(value, writer).map_err(|error| entry.at(error.into()))?;
    }
    // Every variable-size result container is reserved after the admitted
    // semantic traversal and before copying its hypothetical items.
    let mut background_layers = Vec::new();
    let mut mask_layers = Vec::new();
    let mut transition_items = Vec::new();
    let mut animation_items = Vec::new();
    macro_rules! reserve_items {
        ($property:ident, $variant:ident, $getter:ident, $items:ident) => {
            if property == CssKnownProperty::$property {
                let Some(CssLonghandValueRef::$variant(value)) =
                    ordinary(values, CssKnownProperty::$variant)
                else {
                    return Ok(None);
                };
                $items.try_reserve(value.$getter().len()).map_err(|_| {
                    CssSpecifiedValueSerializationError::new(
                        CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
            }
        };
    }
    reserve_items!(Background, BackgroundImage, images, background_layers);
    reserve_items!(Mask, MaskImage, images, mask_layers);
    reserve_items!(Transition, TransitionProperty, properties, transition_items);
    reserve_items!(Animation, AnimationName, names, animation_items);
    if property == CssKnownProperty::GridTemplate {
        return grid_template(values).map(|value| value.map(Inverse::GridTemplate));
    }
    if property == CssKnownProperty::Grid {
        return grid(values).map(|value| value.map(Inverse::Grid));
    }
    if matches!(
        property,
        CssKnownProperty::BorderBlockColor
            | CssKnownProperty::BorderInlineColor
            | CssKnownProperty::BorderColor
    ) {
        return reconstruct_colors(property, mode, values, writer);
    }
    let mut equality_error = None;
    let mut allocation_failure = false;
    macro_rules! shorter_second {
        ($first:expr, $second:expr, $property:ident) => {
            match shorter_second($first, $second, &mut writer.context) {
                Ok(value) => value,
                Err(error) => {
                    let entry = values
                        .iter()
                        .find(|entry| {
                            entry.property_name()
                                == CssPropertyNameRef::Known(CssKnownProperty::$property)
                        })
                        .expect("selected inverse comparison member");
                    equality_error = Some(entry.at(error.into()));
                    return None;
                }
            }
        };
    }
    macro_rules! shortest_quad {
        ($items:expr) => {{
            let mut current = 0;
            match shortest_quad($items, &mut writer.context, &mut current) {
                Ok(value) => value,
                Err(error) => {
                    let property = settable[current].known_property();
                    let entry = values
                        .iter()
                        .find(|entry| entry.property_name() == CssPropertyNameRef::Known(property))
                        .expect("selected inverse comparison member");
                    equality_error = Some(entry.at(error.into()));
                    return None;
                }
            }
        }};
    }
    macro_rules! admitted_quad {
        ($($value:expr),+ $(,)?) => {{
            let mut items = Vec::new();
            if items.try_reserve(4).is_err() { allocation_failure = true; return None; }
            items.extend([$($value),+]);
            items
        }};
    }
    let inverse = (|| -> Option<Inverse> {
        Some(match property {
            CssKnownProperty::ScrollPaddingBlock => {
                let first = val!(values, ScrollPaddingBlockStart);
                let second = shorter_second!(
                    &first,
                    val!(values, ScrollPaddingBlockEnd),
                    ScrollPaddingBlockEnd
                );
                Inverse::ScrollPaddingBlock(CssScrollPaddingPair::new(first, second))
            }
            CssKnownProperty::ScrollPaddingInline => {
                let first = val!(values, ScrollPaddingInlineStart);
                let second = shorter_second!(
                    &first,
                    val!(values, ScrollPaddingInlineEnd),
                    ScrollPaddingInlineEnd
                );
                Inverse::ScrollPaddingInline(CssScrollPaddingPair::new(first, second))
            }
            CssKnownProperty::ScrollMarginBlock => {
                let first = val!(values, ScrollMarginBlockStart);
                let second = shorter_second!(
                    &first,
                    val!(values, ScrollMarginBlockEnd),
                    ScrollMarginBlockEnd
                );
                Inverse::ScrollMarginBlock(CssScrollMarginPair::new(first, second))
            }
            CssKnownProperty::ScrollMarginInline => {
                let first = val!(values, ScrollMarginInlineStart);
                let second = shorter_second!(
                    &first,
                    val!(values, ScrollMarginInlineEnd),
                    ScrollMarginInlineEnd
                );
                Inverse::ScrollMarginInline(CssScrollMarginPair::new(first, second))
            }
            CssKnownProperty::Overflow => {
                let first = val!(values, OverflowX);
                let second = shorter_second!(&first, val!(values, OverflowY), OverflowY);
                Inverse::Overflow(CssOverflowValue::new(first, second))
            }
            CssKnownProperty::Size => {
                let first = val!(values, Width);
                let second = shorter_second!(&first, val!(values, Height), Height);
                Inverse::Size(CssSizePair::new(first, second))
            }
            CssKnownProperty::MinSize => {
                let first = val!(values, MinWidth);
                let second = shorter_second!(&first, val!(values, MinHeight), MinHeight);
                Inverse::MinSize(CssSizePair::new(first, second))
            }
            CssKnownProperty::ContainIntrinsicSize => {
                let first = val!(values, ContainIntrinsicWidth);
                let second = shorter_second!(
                    &first,
                    val!(values, ContainIntrinsicHeight),
                    ContainIntrinsicHeight
                );
                Inverse::ContainIntrinsicSize(CssContainIntrinsicSize::new(first, second))
            }
            CssKnownProperty::Gap => {
                let first = val!(values, RowGap);
                let second = shorter_second!(&first, val!(values, ColumnGap), ColumnGap);
                Inverse::Gap(CssGapShorthand::new(first, second))
            }
            CssKnownProperty::InsetBlock => {
                let first = val!(values, InsetBlockStart);
                let second = shorter_second!(&first, val!(values, InsetBlockEnd), InsetBlockEnd);
                Inverse::InsetBlock(CssInsetPair::new(first, second))
            }
            CssKnownProperty::InsetInline => {
                let first = val!(values, InsetInlineStart);
                let second = shorter_second!(&first, val!(values, InsetInlineEnd), InsetInlineEnd);
                Inverse::InsetInline(CssInsetPair::new(first, second))
            }
            CssKnownProperty::MarginBlock => {
                let first = val!(values, MarginBlockStart);
                let second = shorter_second!(&first, val!(values, MarginBlockEnd), MarginBlockEnd);
                Inverse::MarginBlock(CssMarginPair::new(first, second))
            }
            CssKnownProperty::MarginInline => {
                let first = val!(values, MarginInlineStart);
                let second =
                    shorter_second!(&first, val!(values, MarginInlineEnd), MarginInlineEnd);
                Inverse::MarginInline(CssMarginPair::new(first, second))
            }
            CssKnownProperty::PaddingBlock => {
                let first = val!(values, PaddingBlockStart);
                let second =
                    shorter_second!(&first, val!(values, PaddingBlockEnd), PaddingBlockEnd);
                Inverse::PaddingBlock(CssPaddingPair::new(first, second))
            }
            CssKnownProperty::PaddingInline => {
                let first = val!(values, PaddingInlineStart);
                let second =
                    shorter_second!(&first, val!(values, PaddingInlineEnd), PaddingInlineEnd);
                Inverse::PaddingInline(CssPaddingPair::new(first, second))
            }
            CssKnownProperty::BorderBlockWidth => {
                let first = val!(values, BorderBlockStartWidth);
                let second = shorter_second!(
                    &first,
                    val!(values, BorderBlockEndWidth),
                    BorderBlockEndWidth
                );
                Inverse::BorderBlockWidth(CssBorderWidthPair::new(first, second))
            }
            CssKnownProperty::BorderInlineWidth => {
                let first = val!(values, BorderInlineStartWidth);
                let second = shorter_second!(
                    &first,
                    val!(values, BorderInlineEndWidth),
                    BorderInlineEndWidth
                );
                Inverse::BorderInlineWidth(CssBorderWidthPair::new(first, second))
            }
            CssKnownProperty::BorderBlockStyle => {
                let first = val!(values, BorderBlockStartStyle);
                let second = shorter_second!(
                    &first,
                    val!(values, BorderBlockEndStyle),
                    BorderBlockEndStyle
                );
                Inverse::BorderBlockStyle(CssBorderStylePair::new(first, second))
            }
            CssKnownProperty::BorderInlineStyle => {
                let first = val!(values, BorderInlineStartStyle);
                let second = shorter_second!(
                    &first,
                    val!(values, BorderInlineEndStyle),
                    BorderInlineEndStyle
                );
                Inverse::BorderInlineStyle(CssBorderStylePair::new(first, second))
            }
            CssKnownProperty::GridRow => {
                let first = val!(values, GridRowStart);
                let second = val!(values, GridRowEnd);
                Inverse::GridRow(CssGridLineRange::from_specified(first, second))
            }
            CssKnownProperty::GridColumn => {
                let first = val!(values, GridColumnStart);
                let second = val!(values, GridColumnEnd);
                Inverse::GridColumn(CssGridLineRange::from_specified(first, second))
            }
            CssKnownProperty::Pause => {
                let first = val!(values, PauseBefore);
                let second = shorter_second!(&first, val!(values, PauseAfter), PauseAfter);
                Inverse::Pause(CssSpeechBreakPair::from_parser(first, second))
            }
            CssKnownProperty::Rest => {
                let first = val!(values, RestBefore);
                let second = shorter_second!(&first, val!(values, RestAfter), RestAfter);
                Inverse::Rest(CssSpeechBreakPair::from_parser(first, second))
            }
            CssKnownProperty::Cue => {
                let first = val!(values, CueBefore);
                let second = shorter_second!(&first, val!(values, CueAfter), CueAfter);
                Inverse::Cue(CssCuePair::from_parser(first, second))
            }
            CssKnownProperty::InterestDelay => {
                let first = val!(values, InterestDelayStart);
                let second =
                    shorter_second!(&first, val!(values, InterestDelayEnd), InterestDelayEnd);
                Inverse::InterestDelay(CssInterestDelay::from_parser(first, second))
            }
            CssKnownProperty::Container => Inverse::Container(CssContainer::new(
                val!(values, ContainerName),
                val!(values, ContainerType),
            )),
            CssKnownProperty::FlexFlow => Inverse::FlexFlow(CssFlexFlow::new(
                val!(values, FlexDirection),
                val!(values, FlexWrap),
            )),
            CssKnownProperty::PlaceContent => Inverse::PlaceContent(CssPlaceContentValue::new(
                val!(values, AlignContent),
                val!(values, JustifyContent),
            )),
            CssKnownProperty::PlaceItems => Inverse::PlaceItems(CssPlaceItemsValue::new(
                val!(values, AlignItems),
                val!(values, JustifyItems),
            )),
            CssKnownProperty::PlaceSelf => Inverse::PlaceSelf(CssPlaceSelfValue::new(
                val!(values, AlignSelf),
                val!(values, JustifySelf),
            )),
            CssKnownProperty::ListStyle => Inverse::ListStyle(CssListStyleValue::try_new(
                Some(val!(values, ListStyleType)),
                Some(val!(values, ListStylePosition)),
                Some(val!(values, ListStyleImage)),
            )?),
            CssKnownProperty::Columns => Inverse::Columns(CssColumns::new(
                val!(values, ColumnWidth),
                val!(values, ColumnCount),
            )),
            CssKnownProperty::ItemFlow => Inverse::ItemFlow(CssItemFlow::new(
                val!(values, ItemDirection),
                val!(values, ItemWrap),
                val!(values, ItemPack),
                val!(values, FlowTolerance),
            )),
            CssKnownProperty::ColumnRule => {
                let v0 = optional_value!(values, ColumnRuleWidth, writer, equality_error);
                let mut v1 = optional_value!(values, ColumnRuleStyle, writer, equality_error);
                let v2 = optional_value!(values, ColumnRuleColor, writer, equality_error);
                if v0.is_none() && v1.is_none() && v2.is_none() {
                    v1 = Some(val!(values, ColumnRuleStyle));
                }
                Inverse::ColumnRule(CssColumnRule::try_new(v0, v1, v2)?)
            }
            CssKnownProperty::GridArea => {
                let area = CssGridArea::from_specified(
                    val!(values, GridRowStart),
                    val!(values, GridColumnStart),
                    val!(values, GridRowEnd),
                    val!(values, GridColumnEnd),
                    &mut |a, b, index| {
                        a.bounded_eq(b, &mut writer.context).map_err(|error| {
                            let property = settable[index].known_property();
                            let entry = values
                                .iter()
                                .find(|entry| {
                                    entry.property_name() == CssPropertyNameRef::Known(property)
                                })
                                .expect("selected grid prefix member");
                            entry.at(error.into())
                        })
                    },
                );
                match area {
                    Ok(area) => Inverse::GridArea(area),
                    Err(error) => {
                        equality_error = Some(error);
                        return None;
                    }
                }
            }
            CssKnownProperty::TextWrap => {
                let mut mode = optional_value!(values, TextWrapMode, writer, equality_error);
                let style = optional_value!(values, TextWrapStyle, writer, equality_error);
                if mode.is_none() && style.is_none() {
                    mode = Some(val!(values, TextWrapMode));
                }
                Inverse::TextWrap(CssTextWrap::try_new(mode, style)?)
            }
            CssKnownProperty::WhiteSpace => {
                let collapse = val!(values, WhiteSpaceCollapse);
                let mode = val!(values, TextWrapMode);
                let trim = val!(values, WhiteSpaceTrim);
                // Text4's normative special-keyword mapping is exact and more
                // backwards compatible than the equivalent constituent form.
                let keyword = if trim.is_none() {
                    match (collapse, mode) {
                        (CssWhiteSpaceCollapse::Collapse, CssTextWrapMode::Wrap) => {
                            Some(CssWhiteSpaceKeyword::Normal)
                        }
                        (CssWhiteSpaceCollapse::Preserve, CssTextWrapMode::NoWrap) => {
                            Some(CssWhiteSpaceKeyword::Pre)
                        }
                        (CssWhiteSpaceCollapse::Preserve, CssTextWrapMode::Wrap) => {
                            Some(CssWhiteSpaceKeyword::PreWrap)
                        }
                        (CssWhiteSpaceCollapse::PreserveBreaks, CssTextWrapMode::Wrap) => {
                            Some(CssWhiteSpaceKeyword::PreLine)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                let value = if let Some(keyword) = keyword {
                    CssWhiteSpace::from_keyword(keyword)
                } else {
                    CssWhiteSpace::try_new(
                        optional_value!(values, WhiteSpaceCollapse, writer, equality_error),
                        optional_value!(values, TextWrapMode, writer, equality_error),
                        optional_value!(values, WhiteSpaceTrim, writer, equality_error),
                    )?
                };
                Inverse::WhiteSpace(value)
            }
            CssKnownProperty::TextDecoration => {
                let mut v0 = optional_value!(values, TextDecorationLine, writer, equality_error);
                let v1 = optional_value!(values, TextDecorationColor, writer, equality_error);
                let v2 = optional_value!(values, TextDecorationStyle, writer, equality_error);
                let v3 = optional_value!(values, TextDecorationThickness, writer, equality_error);
                if v0.is_none() && v1.is_none() && v2.is_none() && v3.is_none() {
                    v0 = Some(val!(values, TextDecorationLine));
                }
                Inverse::TextDecoration(CssTextDecoration::new(v0, v1, v2, v3))
            }
            CssKnownProperty::TextEmphasis => {
                let mut v0 = optional_value!(values, TextEmphasisStyle, writer, equality_error);
                let v1 = optional_value!(values, TextEmphasisColor, writer, equality_error);
                if v0.is_none() && v1.is_none() {
                    v0 = Some(val!(values, TextEmphasisStyle));
                }
                Inverse::TextEmphasis(CssTextEmphasis::new(v0, v1))
            }
            CssKnownProperty::Flex => {
                let grow = val!(values, FlexGrow);
                let shrink = val!(values, FlexShrink);
                let basis = val!(values, FlexBasis);
                let auto = matches!(basis.view(), CssFlexBasisRef::Size(CssSizeValue::Auto));
                let value = if auto
                    && crate::flex::ordinary_factor(&grow, "0")
                    && crate::flex::ordinary_factor(&shrink, "0")
                {
                    CssFlexValue::None
                } else if auto
                    && crate::flex::ordinary_factor(&grow, "1")
                    && crate::flex::ordinary_factor(&shrink, "1")
                {
                    CssFlexValue::Auto
                } else {
                    CssFlexValue::Components(CssFlexComponents::try_new(
                        Some(grow),
                        Some(shrink),
                        Some(basis),
                    )?)
                };
                Inverse::Flex(value)
            }
            CssKnownProperty::ColorAdjust => Inverse::ColorAdjust(val!(values, PrintColorAdjust)),
            CssKnownProperty::Outline => {
                let v0 = optional_value!(values, OutlineWidth, writer, equality_error);
                let mut v1 = optional_value!(values, OutlineStyle, writer, equality_error);
                let v2 = optional_value!(values, OutlineColor, writer, equality_error);
                if v0.is_none() && v1.is_none() && v2.is_none() {
                    v1 = Some(val!(values, OutlineStyle));
                }
                Inverse::Outline(CssOutline::new(v0, v1, v2))
            }
            CssKnownProperty::Caret => Inverse::Caret(CssCaret::try_new(
                Some(val!(values, CaretColor)),
                Some(val!(values, CaretAnimation)),
                Some(val!(values, CaretShape)),
            )?),
            CssKnownProperty::Offset => {
                let position = optional_value!(values, OffsetPosition, writer, equality_error);
                let mut path = optional_value!(values, OffsetPath, writer, equality_error);
                let distance = optional_value!(values, OffsetDistance, writer, equality_error);
                let rotate = optional_value!(values, OffsetRotate, writer, equality_error);
                let anchor = optional_value!(values, OffsetAnchor, writer, equality_error);
                if path.is_none() && (position.is_none() || distance.is_some() || rotate.is_some())
                {
                    path = Some(val!(values, OffsetPath));
                }
                Inverse::Offset(CssOffset::try_new(position, path, distance, rotate, anchor).ok()?)
            }
            CssKnownProperty::BorderImage => Inverse::BorderImage(CssBorderImage::try_new(
                Some(val!(values, BorderImageSource)),
                Some(val!(values, BorderImageSlice)),
                Some(val!(values, BorderImageWidth)),
                Some(val!(values, BorderImageOutset)),
                Some(val!(values, BorderImageRepeat)),
            )?),
            CssKnownProperty::MaskBorder => {
                let mut source = optional_value!(values, MaskBorderSource, writer, equality_error);
                let mut slice = optional_value!(values, MaskBorderSlice, writer, equality_error);
                let width = optional_value!(values, MaskBorderWidth, writer, equality_error);
                let outset = optional_value!(values, MaskBorderOutset, writer, equality_error);
                let repeat = optional_value!(values, MaskBorderRepeat, writer, equality_error);
                let mode = optional_value!(values, MaskBorderMode, writer, equality_error);
                if slice.is_none() && (width.is_some() || outset.is_some()) {
                    slice = Some(val!(values, MaskBorderSlice));
                }
                if source.is_none() && slice.is_none() && repeat.is_none() && mode.is_none() {
                    source = Some(val!(values, MaskBorderSource));
                }
                Inverse::MaskBorder(CssMaskBorder::try_new(
                    source, slice, width, outset, repeat, mode,
                )?)
            }
            CssKnownProperty::Border => {
                let width = optional_value!(values, BorderTopWidth, writer, equality_error);
                let mut style = optional_value!(values, BorderTopStyle, writer, equality_error);
                let color = optional_value!(values, BorderTopColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderTopStyle));
                }
                Inverse::Border(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderTop => {
                let width = optional_value!(values, BorderTopWidth, writer, equality_error);
                let mut style = optional_value!(values, BorderTopStyle, writer, equality_error);
                let color = optional_value!(values, BorderTopColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderTopStyle));
                }
                Inverse::BorderTop(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderRight => {
                let width = optional_value!(values, BorderRightWidth, writer, equality_error);
                let mut style = optional_value!(values, BorderRightStyle, writer, equality_error);
                let color = optional_value!(values, BorderRightColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderRightStyle));
                }
                Inverse::BorderRight(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderBottom => {
                let width = optional_value!(values, BorderBottomWidth, writer, equality_error);
                let mut style = optional_value!(values, BorderBottomStyle, writer, equality_error);
                let color = optional_value!(values, BorderBottomColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderBottomStyle));
                }
                Inverse::BorderBottom(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderLeft => {
                let width = optional_value!(values, BorderLeftWidth, writer, equality_error);
                let mut style = optional_value!(values, BorderLeftStyle, writer, equality_error);
                let color = optional_value!(values, BorderLeftColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderLeftStyle));
                }
                Inverse::BorderLeft(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderBlockStart => {
                let width = optional_value!(values, BorderBlockStartWidth, writer, equality_error);
                let mut style =
                    optional_value!(values, BorderBlockStartStyle, writer, equality_error);
                let color = optional_value!(values, BorderBlockStartColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderBlockStartStyle));
                }
                Inverse::BorderBlockStart(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderBlockEnd => {
                let width = optional_value!(values, BorderBlockEndWidth, writer, equality_error);
                let mut style =
                    optional_value!(values, BorderBlockEndStyle, writer, equality_error);
                let color = optional_value!(values, BorderBlockEndColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderBlockEndStyle));
                }
                Inverse::BorderBlockEnd(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderInlineStart => {
                let width = optional_value!(values, BorderInlineStartWidth, writer, equality_error);
                let mut style =
                    optional_value!(values, BorderInlineStartStyle, writer, equality_error);
                let color = optional_value!(values, BorderInlineStartColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderInlineStartStyle));
                }
                Inverse::BorderInlineStart(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderInlineEnd => {
                let width = optional_value!(values, BorderInlineEndWidth, writer, equality_error);
                let mut style =
                    optional_value!(values, BorderInlineEndStyle, writer, equality_error);
                let color = optional_value!(values, BorderInlineEndColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderInlineEndStyle));
                }
                Inverse::BorderInlineEnd(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderBlock => {
                let width = optional_value!(values, BorderBlockStartWidth, writer, equality_error);
                let mut style =
                    optional_value!(values, BorderBlockStartStyle, writer, equality_error);
                let color = optional_value!(values, BorderBlockStartColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderBlockStartStyle));
                }
                Inverse::BorderBlock(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::BorderInline => {
                let width = optional_value!(values, BorderInlineStartWidth, writer, equality_error);
                let mut style =
                    optional_value!(values, BorderInlineStartStyle, writer, equality_error);
                let color = optional_value!(values, BorderInlineStartColor, writer, equality_error);
                if width.is_none() && style.is_none() && color.is_none() {
                    style = Some(val!(values, BorderInlineStartStyle));
                }
                Inverse::BorderInline(CssBorder::try_new(width, style, color)?)
            }
            CssKnownProperty::ScrollPadding => {
                let mode = mode?;
                let sides = match mode {
                    CssBoxSideKind::Physical => admitted_quad![
                        val!(values, ScrollPaddingTop),
                        val!(values, ScrollPaddingRight),
                        val!(values, ScrollPaddingBottom),
                        val!(values, ScrollPaddingLeft)
                    ],
                    CssBoxSideKind::Logical => admitted_quad![
                        val!(values, ScrollPaddingBlockStart),
                        val!(values, ScrollPaddingInlineStart),
                        val!(values, ScrollPaddingBlockEnd),
                        val!(values, ScrollPaddingInlineEnd)
                    ],
                };
                Inverse::ScrollPadding(CssScrollPaddingShorthand::try_new(
                    scroll_mode(mode),
                    shortest_quad!(sides),
                )?)
            }
            CssKnownProperty::ScrollMargin => {
                let mode = mode?;
                let sides = match mode {
                    CssBoxSideKind::Physical => admitted_quad![
                        val!(values, ScrollMarginTop),
                        val!(values, ScrollMarginRight),
                        val!(values, ScrollMarginBottom),
                        val!(values, ScrollMarginLeft)
                    ],
                    CssBoxSideKind::Logical => admitted_quad![
                        val!(values, ScrollMarginBlockStart),
                        val!(values, ScrollMarginInlineStart),
                        val!(values, ScrollMarginBlockEnd),
                        val!(values, ScrollMarginInlineEnd)
                    ],
                };
                Inverse::ScrollMargin(CssScrollMarginShorthand::try_new(
                    scroll_mode(mode),
                    shortest_quad!(sides),
                )?)
            }
            CssKnownProperty::Inset => {
                let mode = mode?;
                let sides = match mode {
                    CssBoxSideKind::Physical => admitted_quad![
                        val!(values, Top),
                        val!(values, Right),
                        val!(values, Bottom),
                        val!(values, Left)
                    ],
                    CssBoxSideKind::Logical => admitted_quad![
                        val!(values, InsetBlockStart),
                        val!(values, InsetInlineStart),
                        val!(values, InsetBlockEnd),
                        val!(values, InsetInlineEnd)
                    ],
                };
                Inverse::Inset(CssInsetShorthand::try_new(mode, shortest_quad!(sides))?)
            }
            CssKnownProperty::Margin => {
                let mode = mode?;
                let sides = match mode {
                    CssBoxSideKind::Physical => admitted_quad![
                        val!(values, MarginTop),
                        val!(values, MarginRight),
                        val!(values, MarginBottom),
                        val!(values, MarginLeft)
                    ],
                    CssBoxSideKind::Logical => admitted_quad![
                        val!(values, MarginBlockStart),
                        val!(values, MarginInlineStart),
                        val!(values, MarginBlockEnd),
                        val!(values, MarginInlineEnd)
                    ],
                };
                Inverse::Margin(CssMarginShorthand::try_new(mode, shortest_quad!(sides))?)
            }
            CssKnownProperty::Padding => {
                let mode = mode?;
                let sides = match mode {
                    CssBoxSideKind::Physical => admitted_quad![
                        val!(values, PaddingTop),
                        val!(values, PaddingRight),
                        val!(values, PaddingBottom),
                        val!(values, PaddingLeft)
                    ],
                    CssBoxSideKind::Logical => admitted_quad![
                        val!(values, PaddingBlockStart),
                        val!(values, PaddingInlineStart),
                        val!(values, PaddingBlockEnd),
                        val!(values, PaddingInlineEnd)
                    ],
                };
                Inverse::Padding(CssPaddingShorthand::try_new(mode, shortest_quad!(sides))?)
            }
            CssKnownProperty::BorderWidth => {
                let mode = mode?;
                let sides = match mode {
                    CssBoxSideKind::Physical => admitted_quad![
                        val!(values, BorderTopWidth),
                        val!(values, BorderRightWidth),
                        val!(values, BorderBottomWidth),
                        val!(values, BorderLeftWidth)
                    ],
                    CssBoxSideKind::Logical => admitted_quad![
                        val!(values, BorderBlockStartWidth),
                        val!(values, BorderInlineStartWidth),
                        val!(values, BorderBlockEndWidth),
                        val!(values, BorderInlineEndWidth)
                    ],
                };
                Inverse::BorderWidth(CssBorderWidthShorthand::try_new(
                    mode,
                    shortest_quad!(sides),
                )?)
            }
            CssKnownProperty::BorderStyle => {
                let mode = mode?;
                let sides = match mode {
                    CssBoxSideKind::Physical => admitted_quad![
                        val!(values, BorderTopStyle),
                        val!(values, BorderRightStyle),
                        val!(values, BorderBottomStyle),
                        val!(values, BorderLeftStyle)
                    ],
                    CssBoxSideKind::Logical => admitted_quad![
                        val!(values, BorderBlockStartStyle),
                        val!(values, BorderInlineStartStyle),
                        val!(values, BorderBlockEndStyle),
                        val!(values, BorderInlineEndStyle)
                    ],
                };
                Inverse::BorderStyle(CssBorderStyleShorthand::try_new(
                    mode,
                    shortest_quad!(sides),
                )?)
            }
            CssKnownProperty::MaxSize => {
                let first = val!(values, MaxWidth);
                let second = shorter_second!(&first, val!(values, MaxHeight), MaxHeight);
                Inverse::MaxSize(CssMaxSizePair::new(first, second))
            }
            CssKnownProperty::BorderRadius => {
                let corners = [
                    val!(values, BorderTopLeftRadius),
                    val!(values, BorderTopRightRadius),
                    val!(values, BorderBottomRightRadius),
                    val!(values, BorderBottomLeftRadius),
                ];
                let horizontal = shortest_quad!(admitted_quad![
                    corners[0].horizontal().clone(),
                    corners[1].horizontal().clone(),
                    corners[2].horizontal().clone(),
                    corners[3].horizontal().clone()
                ]);
                let vertical = shortest_quad!(admitted_quad![
                    corners[0].vertical().clone(),
                    corners[1].vertical().clone(),
                    corners[2].vertical().clone(),
                    corners[3].vertical().clone()
                ]);
                let mut equal_axes = horizontal.len() == vertical.len();
                if equal_axes {
                    for (index, (a, b)) in horizontal.iter().zip(&vertical).enumerate() {
                        match a.inverse_equal(b, &mut writer.context) {
                            Ok(true) => {}
                            Ok(false) => {
                                equal_axes = false;
                                break;
                            }
                            Err(error) => {
                                let property = settable[index].known_property();
                                let entry = values
                                    .iter()
                                    .find(|entry| {
                                        entry.property_name() == CssPropertyNameRef::Known(property)
                                    })
                                    .expect("selected radius comparison member");
                                equality_error = Some(entry.at(error.into()));
                                return None;
                            }
                        }
                    }
                }
                let vertical = (!equal_axes).then_some(vertical);
                Inverse::BorderRadius(CssBorderRadiusShorthand::try_new(horizontal, vertical)?)
            }
            CssKnownProperty::TextAlign => {
                let all = val!(values, TextAlignAll);
                let last = val!(values, TextAlignLast);
                if all == CssTextAlignAllValue::Keyword(CssTextAlign::Justify)
                    && last == CssTextAlignLastValue::Keyword(CssTextAlign::Justify)
                {
                    Inverse::TextAlign(CssTextAlignValue::JustifyAll)
                } else {
                    Inverse::TextAlign(CssTextAlignValue::Alignment(all))
                }
            }
            CssKnownProperty::TextDecorationSkip => {
                let this = val!(values, TextDecorationSkipSelf);
                Inverse::TextDecorationSkip(match this {
                    CssTextDecorationSkipSelf::None => CssTextDecorationSkip::None,
                    CssTextDecorationSkipSelf::Objects => CssTextDecorationSkip::Auto,
                })
            }
            CssKnownProperty::TextBox => {
                let trim = val!(values, TextBoxTrim);
                let edge = val!(values, TextBoxEdge);
                Inverse::TextBox(
                    if trim == CssTextBoxTrim::None && edge == CssTextBoxEdge::Auto {
                        CssTextBox::Normal
                    } else {
                        CssTextBox::Components(CssTextBoxValues::try_new(Some(trim), Some(edge))?)
                    },
                )
            }
            CssKnownProperty::TextSpacing => {
                let trim = val!(values, TextSpacingTrim);
                let autospace = val!(values, TextAutospace);
                let value = match (trim, autospace) {
                    (CssTextSpacingTrim::Auto, CssTextAutospace::Auto) => CssTextSpacing::Auto,
                    (
                        CssTextSpacingTrim::Trim(CssSpacingTrim::SpaceAll),
                        CssTextAutospace::Autospace(CssAutospace::NoAutospace),
                    ) => CssTextSpacing::None,
                    (trim, autospace) => {
                        let trim = match trim {
                            CssTextSpacingTrim::Trim(v) => Some(v),
                            _ => return None,
                        };
                        let autospace = match autospace {
                            CssTextAutospace::Autospace(v) => Some(v),
                            CssTextAutospace::Normal => None,
                            _ => return None,
                        };
                        CssTextSpacing::Components(CssTextSpacingValues::try_new(trim, autospace)?)
                    }
                };
                Inverse::TextSpacing(value)
            }
            CssKnownProperty::FontSynthesis => {
                let weight = val!(values, FontSynthesisWeight) == CssFontSynthesisWeight::Auto;
                let style = match val!(values, FontSynthesisStyle) {
                    CssFontSynthesisStyle::Auto => true,
                    CssFontSynthesisStyle::None => false,
                    CssFontSynthesisStyle::ObliqueOnly => return None,
                };
                let caps = val!(values, FontSynthesisSmallCaps) == CssFontSynthesisSmallCaps::Auto;
                let position =
                    val!(values, FontSynthesisPosition) == CssFontSynthesisPosition::Auto;
                Inverse::FontSynthesis(
                    CssFontSynthesisValues::try_new(weight, style, caps, position)
                        .map_or(CssFontSynthesis::None, CssFontSynthesis::Values),
                )
            }
            CssKnownProperty::FontVariant => Inverse::FontVariant(font_variant(values)?),
            CssKnownProperty::Font => Inverse::Font(font(values)?),
            CssKnownProperty::GridTemplate | CssKnownProperty::Grid => {
                unreachable!("fallible grid inverse was handled before fixed families")
            }
            CssKnownProperty::Background => {
                Inverse::Background(background(values, background_layers)?)
            }
            CssKnownProperty::Mask => Inverse::Mask(mask(
                values,
                mask_layers,
                &mut writer.context,
                &mut equality_error,
            )?),
            CssKnownProperty::Transition => {
                Inverse::Transition(transition(values, transition_items)?)
            }
            CssKnownProperty::Animation => Inverse::Animation(animation(values, animation_items)?),
            _ => unreachable!("only schema shorthand families enter reconstruction"),
        })
    })();
    if let Some(error) = equality_error {
        return Err(error);
    }
    if allocation_failure {
        return Err(CssSpecifiedValueSerializationError::new(
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
        .into());
    }
    Ok(inverse)
}

fn reconstruct_colors(
    property: CssKnownProperty,
    mode: Option<CssBoxSideKind>,
    values: &[&CssSpecifiedDeclarationEntry],
    writer: &mut SpecifiedRuleWriter,
) -> Result<Option<Inverse>> {
    let get = |property| match ordinary(values, property) {
        Some(CssLonghandValueRef::BorderBlockStartColor(v))
        | Some(CssLonghandValueRef::BorderBlockEndColor(v))
        | Some(CssLonghandValueRef::BorderInlineStartColor(v))
        | Some(CssLonghandValueRef::BorderInlineEndColor(v))
        | Some(CssLonghandValueRef::BorderTopColor(v))
        | Some(CssLonghandValueRef::BorderRightColor(v))
        | Some(CssLonghandValueRef::BorderBottomColor(v))
        | Some(CssLonghandValueRef::BorderLeftColor(v)) => Some((
            values
                .iter()
                .find(|entry| entry.property_name() == CssPropertyNameRef::Known(property))
                .expect("selected color member"),
            v,
        )),
        _ => None,
    };
    let value = match property {
        CssKnownProperty::BorderBlockColor | CssKnownProperty::BorderInlineColor => {
            let (first, second) = if property == CssKnownProperty::BorderBlockColor {
                (
                    CssKnownProperty::BorderBlockStartColor,
                    CssKnownProperty::BorderBlockEndColor,
                )
            } else {
                (
                    CssKnownProperty::BorderInlineStartColor,
                    CssKnownProperty::BorderInlineEndColor,
                )
            };
            let (Some((first_entry, first)), Some((_, second))) = (get(first), get(second)) else {
                return Ok(None);
            };
            let equal = first
                .specified_inverse_eq(second, &mut writer.context)
                .map_err(|error| first_entry.at(error.into()))?;
            let second = (!equal).then(|| second.clone());
            let pair = CssBorderColorPair::new(first.clone(), second);
            if property == CssKnownProperty::BorderBlockColor {
                Inverse::BorderBlockColor(pair)
            } else {
                Inverse::BorderInlineColor(pair)
            }
        }
        CssKnownProperty::BorderColor => {
            let Some(mode) = mode else {
                return Ok(None);
            };
            let properties = match mode {
                CssBoxSideKind::Physical => [
                    CssKnownProperty::BorderTopColor,
                    CssKnownProperty::BorderRightColor,
                    CssKnownProperty::BorderBottomColor,
                    CssKnownProperty::BorderLeftColor,
                ],
                CssBoxSideKind::Logical => [
                    CssKnownProperty::BorderBlockStartColor,
                    CssKnownProperty::BorderInlineStartColor,
                    CssKnownProperty::BorderBlockEndColor,
                    CssKnownProperty::BorderInlineEndColor,
                ],
            };
            let [
                Some((top_entry, top)),
                Some((right_entry, right)),
                Some((_, bottom)),
                Some((_, left)),
            ] = properties.map(get)
            else {
                return Ok(None);
            };
            let count = if !right
                .specified_inverse_eq(left, &mut writer.context)
                .map_err(|error| right_entry.at(error.into()))?
            {
                4
            } else if !top
                .specified_inverse_eq(bottom, &mut writer.context)
                .map_err(|error| top_entry.at(error.into()))?
            {
                3
            } else if !top
                .specified_inverse_eq(right, &mut writer.context)
                .map_err(|error| top_entry.at(error.into()))?
            {
                2
            } else {
                1
            };
            let mut sides = Vec::new();
            sides.try_reserve(count).map_err(|_| {
                CssSpecifiedValueSerializationError::new(
                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            sides.extend([top, right, bottom, left].into_iter().take(count).cloned());
            Inverse::BorderColor(
                CssBorderColorShorthand::try_new(mode, sides)
                    .expect("nonempty bounded color sides"),
            )
        }
        _ => unreachable!("selected color inverse family"),
    };
    Ok(Some(value))
}

/// Actual Color and retained time-pair providers advance the originating member
/// cursor through canonical output and its bounded temporary. Inspected terminal
/// phases attach at their own visit/comparison before reaching this provider.
pub(crate) fn append_reconstructed<'a>(
    value: &Inverse,
    property: CssKnownProperty,
    mode: Option<CssBoxSideKind>,
    values: &[&'a CssSpecifiedDeclarationEntry],
    writer: &mut SpecifiedRuleWriter,
    current: &mut Option<&'a CssSpecifiedDeclarationEntry>,
) -> std::result::Result<(), CssSpecifiedValueSerializationError> {
    if !matches!(
        value,
        Inverse::BorderBlockColor(_)
            | Inverse::BorderInlineColor(_)
            | Inverse::BorderColor(_)
            | Inverse::Pause(_)
            | Inverse::Rest(_)
            | Inverse::InterestDelay(_)
    ) {
        let (result, property) =
            writer.with_inverse_sources(property, mode, |writer| value.append(writer));
        *current = property.and_then(|property| {
            values
                .iter()
                .copied()
                .find(|entry| entry.property_name() == CssPropertyNameRef::Known(property))
        });
        return result;
    }
    let meta = property.metadata().expect("schema member-aware inverse");
    let members = match meta.kind() {
        CssPropertyKindRef::Shorthand(meta) => meta.settable_members(),
        CssPropertyKindRef::FourSideShorthand(meta) => {
            meta.settable_members(mode.expect("selected color mode"))
        }
        _ => unreachable!("color inverse family"),
    };
    let mut before = |index: usize| {
        let property = members[index].known_property();
        *current = values
            .iter()
            .copied()
            .find(|entry| entry.property_name() == CssPropertyNameRef::Known(property));
    };
    match value {
        Inverse::BorderBlockColor(value) | Inverse::BorderInlineColor(value) => {
            value.append_to_rule_writer_with_color_visit(writer, &mut before)
        }
        Inverse::BorderColor(value) => {
            value.append_to_rule_writer_with_color_visit(writer, &mut before)
        }
        Inverse::Pause(value) | Inverse::Rest(value) => value
            .append_to_rule_writer_with_comparison(
                writer,
                &mut |a, b, context| a.inverse_equal(b, context),
                &mut before,
            ),
        Inverse::InterestDelay(value) => value.append_to_rule_writer_with_comparison(
            writer,
            &mut |a, b, context| a.inverse_equal(b, context),
            &mut before,
        ),
        _ => unreachable!("selected member-aware provider"),
    }
}

fn font_variant(values: &[&CssSpecifiedDeclarationEntry]) -> Option<CssFontVariantValue> {
    let ligatures = val!(values, FontVariantLigatures);
    let caps = val!(values, FontVariantCaps);
    let alternates = val!(values, FontVariantAlternates);
    let numeric = val!(values, FontVariantNumeric);
    let east = val!(values, FontVariantEastAsian);
    let position = val!(values, FontVariantPosition);
    let emoji = val!(values, FontVariantEmoji);
    let remaining_normal = caps == CssFontVariantCaps::Normal
        && alternates == CssFontVariantAlternates::Normal
        && numeric == CssFontVariantNumeric::Normal
        && east == CssFontVariantEastAsian::Normal
        && position == CssFontVariantPosition::Normal
        && emoji == CssFontVariantEmoji::Normal;
    if remaining_normal {
        if ligatures == CssFontVariantLigatures::Normal {
            return Some(CssFontVariantValue::Normal);
        }
        if ligatures == CssFontVariantLigatures::None {
            return Some(CssFontVariantValue::None);
        }
    }
    let ligatures = match ligatures {
        CssFontVariantLigatures::Normal => None,
        CssFontVariantLigatures::Values(v) => Some(v),
        CssFontVariantLigatures::None => return None,
    };
    let numeric = match numeric {
        CssFontVariantNumeric::Normal => None,
        CssFontVariantNumeric::Values(v) => Some(v),
    };
    let east = match east {
        CssFontVariantEastAsian::Normal => None,
        CssFontVariantEastAsian::Values(v) => Some(v),
    };
    let alternates = match alternates {
        CssFontVariantAlternates::Normal => None,
        CssFontVariantAlternates::Values(v) => Some(v),
    };
    let caps = (caps != CssFontVariantCaps::Normal).then_some(caps);
    let position = (position != CssFontVariantPosition::Normal).then_some(position);
    let emoji = (emoji != CssFontVariantEmoji::Normal).then_some(emoji);
    CssFontVariantValues::try_new(ligatures, position, caps, numeric, east, alternates, emoji)
        .map(CssFontVariantValue::Values)
}

fn font(values: &[&CssSpecifiedDeclarationEntry]) -> Option<CssFontValue> {
    if let Some(system) = values.iter().find_map(|entry| match entry.value() {
        CssSpecifiedDeclarationValueRef::Completed(CssContributionValueRef::SystemFont(system)) => {
            Some(system)
        }
        _ => None,
    }) {
        return Some(CssFontValue::System(system));
    }
    let width = match val!(values, FontWidth) {
        CssFontWidth::Keyword(v) => v,
        _ => return None,
    };
    let variant = match val!(values, FontVariantCaps) {
        CssFontVariantCaps::Normal => CssFontVariant::Normal,
        CssFontVariantCaps::SmallCaps => CssFontVariant::SmallCaps,
        _ => return None,
    };
    CssExplicitFont::try_new(
        Some(val!(values, FontStyle)),
        Some(variant),
        Some(val!(values, FontWeight)),
        Some(width),
        val!(values, FontSize),
        Some(val!(values, LineHeight)),
        val!(values, FontFamily),
    )
    .map(CssFontValue::Explicit)
}

fn grid_template(values: &[&CssSpecifiedDeclarationEntry]) -> Result<Option<CssGridTemplate>> {
    let Some((rows, columns, areas)) = (|| {
        Some((
            val!(values, GridTemplateRows),
            val!(values, GridTemplateColumns),
            val!(values, GridTemplateAreas),
        ))
    })() else {
        return Ok(None);
    };
    CssGridTemplate::from_specified_longhands(rows, columns, areas).map_err(Into::into)
}
fn grid(values: &[&CssSpecifiedDeclarationEntry]) -> Result<Option<CssGrid>> {
    let Some(flow) = (|| Some(val!(values, GridAutoFlow)))() else {
        return Ok(None);
    };
    match flow {
        CssGridAutoFlow::Normal => grid_template(values).map(|value| value.map(CssGrid::template)),
        CssGridAutoFlow::ExplicitAxis(mode) => Ok((|| {
            let (tracks, explicit) = match mode.axis() {
                CssGridAutoFlowAxis::Row => (
                    val!(values, GridAutoRows),
                    val!(values, GridTemplateColumns),
                ),
                CssGridAutoFlowAxis::Column => (
                    val!(values, GridAutoColumns),
                    val!(values, GridTemplateRows),
                ),
            };
            Some(CssGrid::from_auto_flow(mode, Some(tracks), explicit))
        })()),
        CssGridAutoFlow::Dense => Ok(None),
    }
}

fn background(
    values: &[&CssSpecifiedDeclarationEntry],
    mut layers: Vec<CssBackgroundLayer>,
) -> Option<CssBackground> {
    let images = val!(values, BackgroundImage);
    let positions = val!(values, BackgroundPosition);
    let sizes = val!(values, BackgroundSize);
    let repeats = val!(values, BackgroundRepeat);
    let attachments = val!(values, BackgroundAttachment);
    let origins = val!(values, BackgroundOrigin);
    let clips = val!(values, BackgroundClip);
    let color = val!(values, BackgroundColor);
    let count = images.images().len();
    if [
        positions.positions().len(),
        sizes.sizes().len(),
        repeats.repeats().len(),
        attachments.attachments().len(),
        origins.boxes().len(),
        clips.boxes().len(),
    ]
    .iter()
    .any(|n| *n != count)
    {
        return None;
    }
    for index in 0..count {
        let origin = origins.boxes()[index];
        let clip = clips.boxes()[index];
        let boxes = if origin == clip {
            CssBackgroundLayerBoxes::One(origin)
        } else {
            CssBackgroundLayerBoxes::OriginAndClip { origin, clip }
        };
        layers.push(
            CssBackgroundLayer::try_new(
                Some(images.images()[index].clone()),
                Some(positions.positions()[index].clone()),
                Some(sizes.sizes()[index].clone()),
                Some(repeats.repeats()[index]),
                Some(attachments.attachments()[index]),
                Some(boxes),
                (index + 1 == count).then(|| color.clone()),
            )
            .ok()?,
        );
    }
    CssBackground::try_new(layers).ok()
}
fn mask(
    values: &[&CssSpecifiedDeclarationEntry],
    mut layers: Vec<CssMaskLayer>,
    context: &mut SpecifiedSerializationContext,
    equality_error: &mut Option<CssDeclarationBlockError>,
) -> Option<CssMaskList> {
    let images = val!(values, MaskImage);
    let positions = val!(values, MaskPosition);
    let sizes = val!(values, MaskSize);
    let repeats = val!(values, MaskRepeat);
    let origins = val!(values, MaskOrigin);
    let clips = val!(values, MaskClip);
    let operators = val!(values, MaskComposite);
    let modes = val!(values, MaskMode);
    let count = images.images().len();
    if [
        positions.positions().len(),
        sizes.sizes().len(),
        repeats.repeats().len(),
        origins.boxes().len(),
        clips.clips().len(),
        operators.operators().len(),
        modes.modes().len(),
    ]
    .iter()
    .any(|n| *n != count)
    {
        return None;
    }
    let initial_positions = initial_value!(MaskPosition);
    let initial_sizes = initial_value!(MaskSize);
    let initial_repeats = initial_value!(MaskRepeat);
    let initial_origins = initial_value!(MaskOrigin);
    let initial_clips = initial_value!(MaskClip);
    let initial_operators = initial_value!(MaskComposite);
    let initial_modes = initial_value!(MaskMode);
    for index in 0..count {
        let origin = origins.boxes()[index];
        let clip = clips.clips()[index];
        let boxes = if origin == initial_origins.boxes()[0] && clip == initial_clips.clips()[0] {
            None
        } else if origin == initial_origins.boxes()[0] && clip == CssMaskClip::NoClip {
            Some(CssMaskLayerBoxes::NoClip)
        } else if clip == CssMaskClip::Box(origin) {
            Some(CssMaskLayerBoxes::Box(origin))
        } else {
            Some(CssMaskLayerBoxes::Pair { origin, clip })
        };
        let mut image = (!matches!(images.images()[index], CssImageValue::None))
            .then(|| images.images()[index].clone());
        let position = match positions.positions()[index]
            .bounded_eq(&initial_positions.positions()[0], context)
        {
            Ok(equal) => (!equal).then(|| positions.positions()[index].clone()),
            Err(error) => {
                let entry = values
                    .iter()
                    .find(|entry| {
                        entry.property_name()
                            == CssPropertyNameRef::Known(CssKnownProperty::MaskPosition)
                    })
                    .expect("selected mask position member");
                *equality_error = Some(entry.at(error.into()));
                return None;
            }
        };
        let size = match sizes.sizes()[index].inverse_equal(&initial_sizes.sizes()[0], context) {
            Ok(equal) => (!equal).then(|| sizes.sizes()[index].clone()),
            Err(error) => {
                let entry = values
                    .iter()
                    .find(|entry| {
                        entry.property_name()
                            == CssPropertyNameRef::Known(CssKnownProperty::MaskSize)
                    })
                    .expect("selected mask size member");
                *equality_error = Some(entry.at(error.into()));
                return None;
            }
        };
        let repeat = (repeats.repeats()[index] != initial_repeats.repeats()[0])
            .then_some(repeats.repeats()[index]);
        let operator = (operators.operators()[index] != initial_operators.operators()[0])
            .then_some(operators.operators()[index]);
        let mode =
            (modes.modes()[index] != initial_modes.modes()[0]).then_some(modes.modes()[index]);
        if image.is_none()
            && position.is_none()
            && size.is_none()
            && repeat.is_none()
            && boxes.is_none()
            && operator.is_none()
            && mode.is_none()
        {
            image = Some(CssImageValue::None);
        }
        layers.push(CssMaskLayer::try_new(
            image, position, size, repeat, boxes, operator, mode,
        )?);
    }
    CssMaskList::try_new(layers)
}
fn transition(
    values: &[&CssSpecifiedDeclarationEntry],
    mut items: Vec<CssTransition>,
) -> Option<CssTransitionList> {
    let properties = val!(values, TransitionProperty);
    let durations = val!(values, TransitionDuration);
    let delays = val!(values, TransitionDelay);
    let easings = val!(values, TransitionTimingFunction);
    let count = properties.properties().len();
    if [
        durations.values().len(),
        delays.values().len(),
        easings.values().len(),
    ]
    .iter()
    .any(|n| *n != count)
    {
        return None;
    }
    for i in 0..count {
        let duration = (!durations.values()[i].is_exact_ordinary_zero())
            .then(|| durations.values()[i].clone());
        let delay = (!ordinary_zero_time(&delays.values()[i])).then(|| delays.values()[i].clone());
        let easing = (easings.values()[i] != CssEasing::Keyword(CssEasingKeyword::Ease))
            .then(|| easings.values()[i].clone());
        items.push(CssTransition::from_parser(
            Some(properties.properties()[i].clone()),
            duration,
            delay,
            easing,
        )?);
    }
    CssTransitionList::from_parser(items)
}
fn animation(
    values: &[&CssSpecifiedDeclarationEntry],
    mut items: Vec<CssAnimation>,
) -> Option<CssAnimationList> {
    let names = val!(values, AnimationName);
    let durations = val!(values, AnimationDuration);
    let delays = val!(values, AnimationDelay);
    let easings = val!(values, AnimationTimingFunction);
    let counts = val!(values, AnimationIterationCount);
    let directions = val!(values, AnimationDirection);
    let fills = val!(values, AnimationFillMode);
    let states = val!(values, AnimationPlayState);
    let count = names.names().len();
    if [
        durations.values().len(),
        delays.values().len(),
        easings.values().len(),
        counts.values().len(),
        directions.directions().len(),
        fills.modes().len(),
        states.states().len(),
    ]
    .iter()
    .any(|n| *n != count)
    {
        return None;
    }
    for i in 0..count {
        let mut components = CssAnimationComponents {
            name: (!matches!(names.names()[i], CssAnimationName::None))
                .then(|| names.names()[i].clone()),
            duration: (!durations.values()[i].is_exact_ordinary_zero())
                .then(|| durations.values()[i].clone()),
            delay: (!ordinary_zero_time(&delays.values()[i])).then(|| delays.values()[i].clone()),
            timing_function: (easings.values()[i] != CssEasing::Keyword(CssEasingKeyword::Ease))
                .then(|| easings.values()[i].clone()),
            iteration_count: (!ordinary_one_count(&counts.values()[i]))
                .then(|| counts.values()[i].clone()),
            direction: (directions.directions()[i] != CssAnimationDirection::Normal)
                .then_some(directions.directions()[i]),
            fill_mode: (fills.modes()[i] != CssAnimationFillMode::None).then_some(fills.modes()[i]),
            play_state: (states.states()[i] != CssAnimationPlayState::Running)
                .then_some(states.states()[i]),
        };
        if components.name.is_none()
            && components.duration.is_none()
            && components.delay.is_none()
            && components.timing_function.is_none()
            && components.iteration_count.is_none()
            && components.direction.is_none()
            && components.fill_mode.is_none()
            && components.play_state.is_none()
        {
            components.fill_mode = Some(CssAnimationFillMode::None);
        }
        items.push(CssAnimation::from_parser(components)?);
    }
    Some(CssAnimationList::from_parser(items))
}

fn ordinary_zero_time(value: &CssTimeValue) -> bool {
    value.literal().is_some_and(|literal| {
        crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation()).len == 0
    })
}
fn ordinary_one_count(value: &CssAnimationIterationCount) -> bool {
    match value {
        CssAnimationIterationCount::Number(value) => value.literal_component().is_some_and(|value| {
            matches!(value.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number))
                if crate::exact_decimal::LexicalDecimal::new(number.representation()).value_eq(&crate::exact_decimal::LexicalDecimal::new("1")))
        }),
        _ => false,
    }
}

#[cfg(test)]
mod generic_math_comparison_tests {
    use super::*;
    use crate::specified_serialization::SpecifiedSerializationContext;

    fn margin(source: &str) -> CssLonghandContributions {
        let report = crate::parse_style_attribute(source);
        assert!(report.is_clean(), "{source}");
        assert_eq!(report.syntax().len(), 1);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            crate::expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("ordinary checked margin contribution")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(
            values.items()[0].property(),
            CssKnownProperty::MarginBlockStart
        );
        assert!(
            values.items()[0]
                .source()
                .same_occurrence(&report.syntax()[0])
        );
        values
    }

    #[test]
    fn generic_math_value_comparison_admits_each_real_graph_work_slot() {
        let left = margin("margin-block-start:calc(1px + 2px)");
        let right = margin("margin-block-start:calc(1px + 2px)");
        assert!(
            !left.items()[0]
                .source()
                .same_occurrence(right.items()[0].source())
        );
        // This is the actual values_equal helper, which charges no comparison
        // initiation itself. Its caller owns that separate initiation visit.
        // Calc, Sum, the 1px leaf and the 2px leaf require four pending slots.
        // Primitive token comparisons can borrow the exact owning relation;
        // the selected bounded consumer needs no component traversal vector.
        let mut context =
            SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::new(0, 4, 0));
        assert!(
            values_equal(
                left.items()[0].value(),
                right.items()[0].value(),
                &mut context
            )
            .unwrap()
        );
        let mut context =
            SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::new(0, 3, 0));
        assert_eq!(
            values_equal(
                left.items()[0].value(),
                right.items()[0].value(),
                &mut context
            )
            .unwrap_err()
            .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        let mut retry =
            SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::new(0, 4, 0));
        assert!(
            values_equal(
                left.items()[0].value(),
                right.items()[0].value(),
                &mut retry
            )
            .unwrap()
        );
    }

    #[test]
    fn generic_math_value_comparison_retains_exact_owner_phase_and_token_identity() {
        for (left, right, expected) in [
            ("margin-block-start:1px", "margin-block-start:1.0px", true),
            ("margin-block-start:0", "margin-block-start:0px", true),
            (
                "margin-block-start:calc(1px + 2px)",
                "  margin-block-start:CALC(1px + 2px)",
                true,
            ),
            (
                "margin-block-start:calc(1px + 2px)",
                "margin-block-start:3px",
                false,
            ),
            (
                "margin-block-start:calc(1px + 2px)",
                "margin-block-start:calc(01px + 2px)",
                false,
            ),
            (
                "margin-block-start:calc(1px + 2px)",
                "margin-block-start:calc(1px - 2px)",
                false,
            ),
            (
                "margin-block-start:calc(1px + 2px)",
                "margin-block-start:calc((1px + 2px))",
                false,
            ),
            (
                "margin-block-start:calc(1px + 2px)",
                "margin-block-start:calc(1px + 2%)",
                false,
            ),
        ] {
            let left = margin(left);
            let right = margin(right);
            let mut context =
                SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::default());
            assert_eq!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut context
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[cfg(test)]
mod composite_math_comparison_tests {
    use super::*;

    fn terminal(source: &str, property: CssKnownProperty) -> CssLonghandContributions {
        let report = crate::parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(report.syntax().len(), 1);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            crate::expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("completed intrinsic terminal: {source}")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), property);
        assert!(
            values.items()[0]
                .source()
                .same_occurrence(&report.syntax()[0])
        );
        values
    }

    #[test]
    fn distinct_composite_owners_admit_their_actual_math_comparison_work() {
        // A calc(a + b) graph schedules Calc, Sum and two leaves: P4.
        // Enum/slice carriers and borrowed primitive-token comparison add none.
        // These are distinct owner forwarding causes, not a property inventory.
        let cases = [
            (CssKnownProperty::FontSize, "font-size:calc(1px + 2px)", 4),
            (CssKnownProperty::LineHeight, "line-height:calc(1 + 2)", 4),
            (CssKnownProperty::ColumnCount, "column-count:calc(1 + 2)", 4),
            (
                CssKnownProperty::FlowTolerance,
                "flow-tolerance:calc(1px + 2px)",
                4,
            ),
            (
                CssKnownProperty::TextDecorationThickness,
                "text-decoration-thickness:calc(1px + 2px)",
                4,
            ),
            (
                CssKnownProperty::OutlineWidth,
                "outline-width:calc(1px + 2px)",
                4,
            ),
            // Only the first of the four effective slice sides contains math.
            (
                CssKnownProperty::BorderImageSlice,
                "border-image-slice:calc(10% + 20%) 0% 0% 0%",
                4,
            ),
            (
                CssKnownProperty::GridRowStart,
                "grid-row-start:calc(1 + 2)",
                4,
            ),
            // Only the horizontal axis contains math; the vertical token borrows.
            (
                CssKnownProperty::BackgroundPosition,
                "background-position:calc(1px + 2px) 0%",
                4,
            ),
            // y1 is math; y2/x1/x2 are exact ordinary scalar tokens.
            (
                CssKnownProperty::TransitionTimingFunction,
                "transition-timing-function:cubic-bezier(0, calc(1 + 2), 1, 1)",
                4,
            ),
            // One gradient stop and the filter blur each schedule four nodes.
            (
                CssKnownProperty::ListStyleImage,
                "list-style-image:filter(linear-gradient(red calc(1px + 2px), blue), blur(calc(1px + 2px)))",
                10,
            ),
            // The counter-symbol carrier forwards to one gradient math stop.
            (
                CssKnownProperty::ListStyleType,
                "list-style-type:symbols(linear-gradient(red calc(1px + 2px), blue))",
                6,
            ),
            // A Motion basic-shape radius reaches its own numeric forwarding.
            (
                CssKnownProperty::OffsetPath,
                "offset-path:circle(calc(1px + 2px))",
                4,
            ),
            // Two calc-size slots, outer Sum/Size/Length (3), inner Size (1).
            (
                CssKnownProperty::Width,
                "width:calc-size(calc-size(auto, size), size + 1px)",
                6,
            ),
        ];
        let mut failures = Vec::new();
        for (property, source, projections) in cases {
            let left = terminal(source, property);
            let right = terminal(source, property);
            assert!(
                !left.items()[0]
                    .source()
                    .same_occurrence(right.items()[0].source())
            );
            let mut exact = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections, 0),
            );
            if !matches!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut exact
                ),
                Ok(true)
            ) {
                failures.push(format!(
                    "{source}: exact P{projections} does not compare equal"
                ));
            }
            let mut short = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections - 1, 0),
            );
            match values_equal(
                left.items()[0].value(),
                right.items()[0].value(),
                &mut short,
            ) {
                Err(error)
                    if error.kind()
                        == CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit => {}
                actual => failures.push(format!(
                    "{source}: one-under P{} is {actual:?}",
                    projections - 1
                )),
            }
            let mut retry = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections, 0),
            );
            if !matches!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut retry
                ),
                Ok(true)
            ) {
                failures.push(format!("{source}: adequate immutable retry fails"));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn calc_size_basis_mismatch_stops_before_scheduling_a_nested_basis() {
        let left = terminal(
            "width:calc-size(calc-size(auto, size), size + 1px)",
            CssKnownProperty::Width,
        );
        let right = terminal(
            "width:calc-size(min-content, size + 1px)",
            CssKnownProperty::Width,
        );
        // Root calc-size slot1, then its three-node Sum/Size/Length comparison.
        // The differing Nested/Keyword basis stops before the nested slot.
        let mut exact =
            SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::new(0, 4, 0));
        assert!(
            !values_equal(
                left.items()[0].value(),
                right.items()[0].value(),
                &mut exact
            )
            .unwrap()
        );
        let mut short =
            SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::new(0, 3, 0));
        assert_eq!(
            values_equal(
                left.items()[0].value(),
                right.items()[0].value(),
                &mut short
            )
            .unwrap_err()
            .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
    }

    #[test]
    fn checked_border_image_owners_keep_four_effective_sides_for_every_authored_cardinality() {
        for (property, one, four) in [
            (
                CssKnownProperty::BorderImageSlice,
                "border-image-slice:10%",
                "border-image-slice:10% 10% 10% 10%",
            ),
            (
                CssKnownProperty::BorderImageWidth,
                "border-image-width:1",
                "border-image-width:1 1 1 1",
            ),
            (
                CssKnownProperty::BorderImageOutset,
                "border-image-outset:0",
                "border-image-outset:0 0 0 0",
            ),
        ] {
            let left = terminal(one, property);
            let right = terminal(four, property);
            for value in [&left, &right] {
                let CssContributionValueRef::Ordinary(value) = value.items()[0].value() else {
                    panic!("ordinary sides")
                };
                let count = match value {
                    CssLonghandValueRef::BorderImageSlice(value) => value.values().len(),
                    CssLonghandValueRef::BorderImageWidth(value) => value.values().len(),
                    CssLonghandValueRef::BorderImageOutset(value) => value.values().len(),
                    _ => panic!("coupled border-image owner"),
                };
                assert_eq!(count, 4);
            }
            let mut context = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, 0, 0),
            );
            assert!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut context
                )
                .unwrap()
            );
        }
    }

    #[test]
    fn bounded_composites_retain_owning_spelling_phase_and_omission_disqualifications() {
        for (property, left, right) in [
            (
                CssKnownProperty::FontSize,
                "font-size:calc(1px + 2px)",
                "font-size:3px",
            ),
            (
                CssKnownProperty::FontWeight,
                "font-weight:400",
                "font-weight:400.0",
            ),
            (
                CssKnownProperty::FontStyle,
                "font-style:oblique calc(1deg + 2deg)",
                "font-style:oblique 3deg",
            ),
            (
                CssKnownProperty::LineHeight,
                "line-height:calc(1 + 2)",
                "line-height:3",
            ),
            (
                CssKnownProperty::BackgroundPosition,
                "background-position:calc(1px + 2px) 0%",
                "background-position:3px 0%",
            ),
            (
                CssKnownProperty::TransitionTimingFunction,
                "transition-timing-function:cubic-bezier(0, calc(1 + 2), 1, 1)",
                "transition-timing-function:cubic-bezier(0, 3, 1, 1)",
            ),
            (
                CssKnownProperty::ListStyleImage,
                "list-style-image:filter(url(a), blur())",
                "list-style-image:filter(url(a), blur(0px))",
            ),
            (
                CssKnownProperty::ListStyleImage,
                "list-style-image:filter(url(a), hue-rotate())",
                "list-style-image:filter(url(a), hue-rotate(0deg))",
            ),
            (
                CssKnownProperty::OffsetPath,
                "offset-path:circle(calc(1px + 2px))",
                "offset-path:circle(3px)",
            ),
            (
                CssKnownProperty::Width,
                "width:calc-size(calc-size(auto, size), size + 1px)",
                "width:calc-size(calc-size(min-content, size), size + 1px)",
            ),
        ] {
            let left = terminal(left, property);
            let right = terminal(right, property);
            let mut context =
                SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::default());
            assert!(
                !values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut context
                )
                .unwrap(),
                "{property:?}"
            );
        }
    }
}

#[cfg(test)]
mod flex_basis_math_comparison_tests {
    use super::*;

    fn basis(source: &str) -> CssLonghandContributions {
        let report = crate::parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(report.syntax().len(), 1);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            crate::expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("completed flex basis")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), CssKnownProperty::FlexBasis);
        assert!(
            values.items()[0]
                .source()
                .same_occurrence(&report.syntax()[0])
        );
        values
    }

    fn ordinary_equal(left: &CssLonghandContributions, right: &CssLonghandContributions) -> bool {
        let CssContributionValueRef::Ordinary(CssLonghandValueRef::FlexBasis(left)) =
            left.items()[0].value()
        else {
            panic!("ordinary flex basis")
        };
        let CssContributionValueRef::Ordinary(CssLonghandValueRef::FlexBasis(right)) =
            right.items()[0].value()
        else {
            panic!("ordinary flex basis")
        };
        left == right
    }

    #[test]
    fn flex_basis_comparison_admits_reached_numeric_and_calc_size_work_lists() {
        // The actual values_equal helper has no initiation fee. LP and
        // fit-content both reach Calc + Sum + two Values (P4). A direct
        // calc-size reaches its owner + bare Sum + two Values (also P4).
        // The matching nested basis repeats those four actual slots (P8).
        // Borrowed variant/primitive checks add no invented carrier fee.
        let mut failures = Vec::new();
        for (source, projections) in [
            ("flex-basis:calc(1px + 2px)", 4),
            ("flex-basis:fit-content(calc(1px + 2px))", 4),
            ("flex-basis:calc-size(any, 1px + 2px)", 4),
            (
                "flex-basis:calc-size(calc-size(any, 1px + 2px), 1px + 2px)",
                8,
            ),
        ] {
            let left = basis(source);
            let right = basis(source);
            assert!(
                !left.items()[0]
                    .source()
                    .same_occurrence(right.items()[0].source())
            );
            assert!(
                ordinary_equal(&left, &right),
                "{source}: original structural relation"
            );
            let mut exact = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections, 0),
            );
            assert!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut exact
                )
                .unwrap(),
                "{source}"
            );
            let mut under = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections - 1, 0),
            );
            match values_equal(
                left.items()[0].value(),
                right.items()[0].value(),
                &mut under,
            ) {
                Err(error)
                    if error.kind()
                        == CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit => {}
                actual => failures.push(format!(
                    "{source}: one-under P{} returned {actual:?}",
                    projections - 1
                )),
            }
            let mut retry = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections, 0),
            );
            assert!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut retry
                )
                .unwrap(),
                "{source}: adequate retry"
            );
            assert!(
                ordinary_equal(&left, &right),
                "{source}: preserved ordinary relation"
            );
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn flex_basis_outer_phase_and_literal_tokens_require_zero_graph_work() {
        for (left, right, expected) in [
            ("flex-basis:content", "flex-basis:content", true),
            ("flex-basis:auto", "flex-basis:auto", true),
            ("flex-basis:1px", "flex-basis:1px", true),
            ("flex-basis:1px", "flex-basis:1.0px", false),
            ("flex-basis:1px", "flex-basis:01px", false),
            ("flex-basis:0", "flex-basis:0px", false),
            ("flex-basis:content", "flex-basis:auto", false),
            (
                "flex-basis:content",
                "flex-basis:calc-size(any, 1px + 2px)",
                false,
            ),
            ("flex-basis:calc(1px + 2px)", "flex-basis:3px", false),
            (
                "flex-basis:calc(1px + 2px)",
                "flex-basis:fit-content(calc(1px + 2px))",
                false,
            ),
            (
                "flex-basis:calc(1px + 2px)",
                "flex-basis:calc-size(any, 1px + 2px)",
                false,
            ),
        ] {
            let left = basis(left);
            let right = basis(right);
            assert_eq!(
                ordinary_equal(&left, &right),
                expected,
                "original owning relation"
            );
            let mut context = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, 0, 0),
            );
            assert_eq!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut context
                )
                .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn flex_basis_false_graphs_keep_raw_syntax_type_and_calculation_before_basis_order() {
        // Function raw syntax/type is checked before its argument is queued:
        // operator, grouping and percent-hint mismatches use just the root P1.
        // A leaf spelling mismatch reaches all four already scheduled pairs.
        // CalcSize compares its three-node calculation after its own root,
        // then rejects Any versus Keyword without scheduling a nested basis.
        let mut failures = Vec::new();
        for (left, right, projections) in [
            (
                "flex-basis:calc(1px + 2px)",
                "flex-basis:calc(01px + 2px)",
                4,
            ),
            (
                "flex-basis:calc(2px + 1px)",
                "flex-basis:calc(2px - 1px)",
                1,
            ),
            (
                "flex-basis:calc(1px + 2px)",
                "flex-basis:calc((1px + 2px))",
                1,
            ),
            ("flex-basis:calc(1px + 2px)", "flex-basis:calc(1px + 2%)", 1),
            (
                "flex-basis:calc-size(any, 1px + 2px)",
                "flex-basis:calc-size(auto, 1px + 2px)",
                4,
            ),
        ] {
            let source = format!("{left} versus {right}");
            let left = basis(left);
            let right = basis(right);
            assert!(
                !ordinary_equal(&left, &right),
                "{source}: original structural relation"
            );
            let mut exact = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections, 0),
            );
            assert!(
                !values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut exact
                )
                .unwrap(),
                "{source}"
            );
            let mut under = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections - 1, 0),
            );
            match values_equal(
                left.items()[0].value(),
                right.items()[0].value(),
                &mut under,
            ) {
                Err(error)
                    if error.kind()
                        == CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit => {}
                actual => failures.push(format!(
                    "{source}: one-under P{} returned {actual:?}",
                    projections - 1
                )),
            }
            let mut retry = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, projections, 0),
            );
            assert!(
                !values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut retry
                )
                .unwrap(),
                "{source}: adequate retry"
            );
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}

#[cfg(test)]
mod mask_position_borrowed_comparison_tests {
    use super::*;

    fn positions(source: &str) -> CssLonghandContributions {
        let report = crate::parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(report.syntax().len(), 1);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            crate::expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("completed mask position")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), CssKnownProperty::MaskPosition);
        values
    }

    fn list(values: &CssLonghandContributions) -> &CssPhysicalPositionList {
        let CssContributionValueRef::Ordinary(CssLonghandValueRef::MaskPosition(value)) =
            values.items()[0].value()
        else {
            panic!("ordinary mask position")
        };
        value
    }

    #[test]
    fn mask_initial_position_uses_borrowed_tokens_and_short_circuits_math_phases() {
        // The schema initial is a real literal pair. Matching primitive tokens
        // and literal/math or axis-variant mismatches enqueue no graph work.
        // P0 is an owning-policy control, not an allocation RED witness.
        let initial = {
            let CssPropertyKindRef::Longhand(metadata) = CssKnownProperty::MaskPosition
                .metadata()
                .expect("schema mask position")
                .kind()
            else {
                panic!("mask position must be a schema longhand")
            };
            let owner = metadata.initial_value();
            let CssInitialValueRef::Value(value) = owner.view() else {
                panic!("mask position must have an ordinary schema initial")
            };
            let CssLonghandValueRef::MaskPosition(value) = value.view() else {
                panic!("schema initial must retain mask position identity")
            };
            assert_eq!(
                value.positions().len(),
                1,
                "schema initial mask layer cardinality"
            );
            value.clone()
        };
        for (source, expected) in [
            ("mask-position:0% 0%", true),
            ("mask-position:0.0% 0%", false),
            ("mask-position:0% 0.0%", false),
            ("mask-position:10% 20%", false),
            ("mask-position:left top", false),
            ("mask-position:calc(1px + 2px) 0%", false),
            ("mask-position:0% calc(1px + 2px)", false),
            ("mask-position:10% calc(1px + 2px)", false),
        ] {
            let values = positions(source);
            let value = &list(&values).positions()[0];
            assert_eq!(
                value == &initial.positions()[0],
                expected,
                "{source}: unchanged ordinary relation"
            );
            let mut context = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, 0, 0),
            );
            assert_eq!(
                value
                    .bounded_eq(&initial.positions()[0], &mut context)
                    .unwrap(),
                expected,
                "{source}"
            );
        }

        // Unlike math versus the literal initial, final matching math values
        // reach Calc + Sum + two Values. The untouched existing list/position
        // forwarding must admit exactly those four real comparison slots.
        for source in [
            "mask-position:calc(1px + 2px) 0%",
            "mask-position:0% calc(1px + 2px)",
        ] {
            let left = positions(source);
            let right = positions(source);
            assert_eq!(
                list(&left),
                list(&right),
                "{source}: ordinary structural relation"
            );
            let mut exact = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, 4, 0),
            );
            assert!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut exact
                )
                .unwrap()
            );
            let mut under = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, 3, 0),
            );
            assert_eq!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut under
                )
                .unwrap_err()
                .kind(),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            );
            let mut retry = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(0, 4, 0),
            );
            assert!(
                values_equal(
                    left.items()[0].value(),
                    right.items()[0].value(),
                    &mut retry
                )
                .unwrap()
            );
        }
    }
}

#[cfg(test)]
mod column_alias_and_grid_count_comparison_tests {
    use super::*;

    fn terminal(property: CssKnownProperty, value: &str) -> CssLonghandContributions {
        let source = format!("{}:{value}", property.canonical_name());
        let report = crate::parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(report.syntax().len(), 1);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            crate::expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("completed schema terminal: {source}")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), property);
        assert!(
            values.items()[0]
                .source()
                .same_occurrence(&report.syntax()[0])
        );
        values
    }

    fn comparison(
        left: &CssLonghandContributions,
        right: &CssLonghandContributions,
        projections: usize,
    ) -> EqualityResult {
        let mut context = SpecifiedSerializationContext::new(
            CssSpecifiedValueSerializationLimits::new(0, projections, 0),
        );
        values_equal(
            left.items()[0].value(),
            right.items()[0].value(),
            &mut context,
        )
    }

    fn reached_work(
        property: CssKnownProperty,
        left: &str,
        right: &str,
        expected: bool,
        projections: usize,
    ) {
        let left = terminal(property, left);
        let right = terminal(property, right);
        assert_eq!(
            comparison(&left, &right, projections).unwrap(),
            expected,
            "{property:?}"
        );
        assert_eq!(
            comparison(&left, &right, projections - 1)
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            "{property:?}"
        );
        assert_eq!(
            comparison(&left, &right, projections).unwrap(),
            expected,
            "{property:?}: adequate retry"
        );
    }

    #[test]
    fn column_schema_aliases_admit_reached_math_and_calc_size_comparisons() {
        // LP/FitContent: Function + Sum + two Values. CalcSize: owner +
        // bare Sum + two Values. Carriers and primitive tokens are borrowed.
        for (property, value) in [
            (CssKnownProperty::ColumnRuleWidth, "calc(1px + 2px)"),
            (CssKnownProperty::ColumnWidth, "calc(1px + 2px)"),
            (
                CssKnownProperty::ColumnWidth,
                "fit-content(calc(1px + 2px))",
            ),
            (CssKnownProperty::ColumnWidth, "calc-size(any, 1px + 2px)"),
        ] {
            reached_work(property, value, value, true, 4);
        }
    }

    #[test]
    fn column_schema_aliases_use_canonical_literal_relation_without_changing_ordinary_eq() {
        for (property, left, right) in [
            (CssKnownProperty::ColumnRuleWidth, "1px", "01px"),
            (CssKnownProperty::ColumnRuleWidth, "1px", "1.0px"),
            (CssKnownProperty::ColumnRuleWidth, "0", "0px"),
            (CssKnownProperty::ColumnWidth, "1px", "01px"),
            (CssKnownProperty::ColumnWidth, "1px", "1.0px"),
            (CssKnownProperty::ColumnWidth, "0", "0px"),
            (
                CssKnownProperty::ColumnWidth,
                "fit-content(1px)",
                "fit-content(01px)",
            ),
        ] {
            let left = terminal(property, left);
            let right = terminal(property, right);
            let ordinary_equal = match (left.items()[0].value(), right.items()[0].value()) {
                (
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::ColumnRuleWidth(a)),
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::ColumnRuleWidth(b)),
                ) => a == b,
                (
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::ColumnWidth(a)),
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::ColumnWidth(b)),
                ) => a == b,
                _ => panic!("column schema alias"),
            };
            assert!(!ordinary_equal, "{property:?}: retained ordinary spelling");
            assert!(
                comparison(&left, &right, 0).unwrap(),
                "{property:?}: canonical coefficient relation"
            );
        }
        for (property, left, right, expected) in [
            (CssKnownProperty::ColumnRuleWidth, "medium", "medium", true),
            (CssKnownProperty::ColumnRuleWidth, "thin", "medium", false),
            (
                CssKnownProperty::ColumnRuleWidth,
                "calc(1px + 2px)",
                "3px",
                false,
            ),
            (
                CssKnownProperty::ColumnRuleWidth,
                "calc(1px + 2px)",
                "medium",
                false,
            ),
            (CssKnownProperty::ColumnWidth, "auto", "auto", true),
            (CssKnownProperty::ColumnWidth, "auto", "min-content", false),
            (
                CssKnownProperty::ColumnWidth,
                "calc(1px + 2px)",
                "3px",
                false,
            ),
            (
                CssKnownProperty::ColumnWidth,
                "calc(1px + 2px)",
                "fit-content(calc(1px + 2px))",
                false,
            ),
            (
                CssKnownProperty::ColumnWidth,
                "calc-size(any, 1px + 2px)",
                "auto",
                false,
            ),
        ] {
            assert_eq!(
                comparison(&terminal(property, left), &terminal(property, right), 0).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn column_alias_false_math_keeps_reached_leaf_syntax_and_calc_size_basis_order() {
        for property in [
            CssKnownProperty::ColumnRuleWidth,
            CssKnownProperty::ColumnWidth,
        ] {
            reached_work(property, "calc(1px + 2px)", "calc(1px + 3px)", false, 4);
            // Raw +/- syntax is retained on Function itself, before Sum.
            reached_work(property, "calc(2px + 1px)", "calc(2px - 1px)", false, 1);
        }
        reached_work(
            CssKnownProperty::ColumnWidth,
            "fit-content(calc(1px + 2px))",
            "fit-content(calc(1px + 3px))",
            false,
            4,
        );
        reached_work(
            CssKnownProperty::ColumnWidth,
            "calc-size(any, 1px + 2px)",
            "calc-size(any, 1px + 3px)",
            false,
            4,
        );
        // CalcSize owner precedes its bare Sum. A differing operator stops
        // there; a differing basis is considered after the whole calculation.
        reached_work(
            CssKnownProperty::ColumnWidth,
            "calc-size(any, 2px + 1px)",
            "calc-size(any, 2px - 1px)",
            false,
            2,
        );
        reached_work(
            CssKnownProperty::ColumnWidth,
            "calc-size(any, 1px + 2px)",
            "calc-size(auto, 1px + 2px)",
            false,
            4,
        );
    }

    #[test]
    fn counted_grid_repeats_admit_only_the_actual_count_graph_before_primitive_content() {
        for value in [
            "repeat(calc(1 + 2), 1px)",
            "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px)",
            "subgrid repeat(calc(1 + 2), [a])",
        ] {
            for property in [
                CssKnownProperty::GridTemplateRows,
                CssKnownProperty::GridTemplateColumns,
            ] {
                let left = terminal(property, value);
                // Intrinsic verification clones the retained typed terminal. This
                // isolates the count schedule from the separate Grid span policy.
                let right = left.clone();
                assert!(
                    comparison(&left, &right, 4).unwrap(),
                    "{property:?}: {value}"
                );
                assert_eq!(
                    comparison(&left, &right, 3).unwrap_err().kind(),
                    CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                    "{property:?}: {value}"
                );
                assert!(
                    comparison(&left, &right, 4).unwrap(),
                    "{property:?}: {value}: adequate retry"
                );
            }
        }
    }

    #[test]
    fn counted_grid_repeats_preserve_count_before_content_and_group_order() {
        // Equal-length source literals keep all primitive content coordinates
        // identical, making these tests independent of Grid coordinate policy.
        for (left, right, projections) in [
            ("repeat(calc(1 + 2), 1px)", "repeat(calc(1 + 3), 1px)", 4),
            ("repeat(calc(2 + 1), 1px)", "repeat(calc(2 - 1), 1px)", 1),
            ("repeat(calc(1 + 2), 1px)", "repeat(calc(1 + 2), 2px)", 4),
            (
                "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px)",
                "repeat(calc(1 + 3), 1px) repeat(auto-fill, 1px)",
                4,
            ),
            (
                "repeat(calc(2 + 1), 1px) repeat(auto-fill, 1px)",
                "repeat(calc(2 - 1), 1px) repeat(auto-fill, 1px)",
                1,
            ),
            (
                "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px)",
                "repeat(calc(1 + 2), 2px) repeat(auto-fill, 1px)",
                4,
            ),
            (
                "subgrid repeat(calc(1 + 2), [a])",
                "subgrid repeat(calc(1 + 3), [a])",
                4,
            ),
            (
                "subgrid repeat(calc(2 + 1), [a])",
                "subgrid repeat(calc(2 - 1), [a])",
                1,
            ),
            (
                "subgrid repeat(calc(1 + 2), [a])",
                "subgrid repeat(calc(1 + 2), [b])",
                4,
            ),
        ] {
            reached_work(
                CssKnownProperty::GridTemplateRows,
                left,
                right,
                false,
                projections,
            );
        }
    }

    #[test]
    fn counted_grid_repeat_literal_phase_variant_and_list_length_stops_require_no_graph_work() {
        for (left, right, expected) in [
            ("repeat(1, 1px)", "repeat(1, 1px)", true),
            ("repeat(1, 1px)", "repeat(2, 2px)", false),
            ("repeat(1, 1px)", "repeat(01, 1px)", false),
            ("repeat(calc(1 + 2), 1px)", "repeat(3, 2px)", false),
            ("repeat(calc(1 + 2), 1px)", "1px", false),
            (
                "repeat(calc(1 + 2), 1px)",
                "repeat(calc(1 + 2), 1px) 1px",
                false,
            ),
            (
                "repeat(calc(1 + 2), 1px)",
                "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px)",
                false,
            ),
            (
                "repeat(1, 1px) repeat(auto-fill, 1px)",
                "repeat(2, 2px) repeat(auto-fill, 1px)",
                false,
            ),
            (
                "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px)",
                "repeat(3, 2px) repeat(auto-fill, 1px)",
                false,
            ),
            (
                "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px)",
                "1px repeat(auto-fill, 1px)",
                false,
            ),
            (
                "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px)",
                "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px) 1px",
                false,
            ),
            ("subgrid repeat(1, [a])", "subgrid repeat(2, [b])", false),
            ("subgrid repeat(1, [a])", "subgrid repeat(01, [a])", false),
            (
                "subgrid repeat(calc(1 + 2), [a])",
                "subgrid repeat(3, [b])",
                false,
            ),
            (
                "subgrid repeat(calc(1 + 2), [a])",
                "subgrid repeat(auto-fill, [b])",
                false,
            ),
            ("subgrid repeat(calc(1 + 2), [a])", "subgrid [a]", false),
            (
                "subgrid repeat(calc(1 + 2), [a])",
                "subgrid repeat(calc(1 + 2), [a]) [b]",
                false,
            ),
        ] {
            assert_eq!(
                comparison(
                    &terminal(CssKnownProperty::GridTemplateRows, left),
                    &terminal(CssKnownProperty::GridTemplateRows, right),
                    0
                )
                .unwrap(),
                expected,
                "{left} / {right}"
            );
        }
    }
}

#[cfg(test)]
mod coordinate_free_inverse_comparison_tests {
    use super::*;

    fn terminal(
        property: CssKnownProperty,
        value: &str,
        shifted: bool,
    ) -> CssLonghandContributions {
        let source = format!(
            "{}{}:{value}",
            if shifted { "  " } else { "" },
            property.canonical_name()
        );
        let report = crate::parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(report.syntax().len(), 1);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            crate::expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("completed intrinsic terminal: {source}")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), property);
        assert!(
            values.items()[0]
                .source()
                .same_occurrence(&report.syntax()[0])
        );
        values
    }

    fn context(projections: usize) -> SpecifiedSerializationContext {
        SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::new(
            0,
            projections,
            0,
        ))
    }

    fn compare(
        left: &CssLonghandContributions,
        right: &CssLonghandContributions,
        projections: usize,
    ) -> EqualityResult {
        values_equal(
            left.items()[0].value(),
            right.items()[0].value(),
            &mut context(projections),
        )
    }

    fn raw_equal(left: &CssLonghandContributions, right: &CssLonghandContributions) -> bool {
        match (left.items()[0].value(), right.items()[0].value()) {
            (
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridTemplateRows(a)),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridTemplateRows(b)),
            ) => a == b,
            (
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridTemplateColumns(a)),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridTemplateColumns(b)),
            ) => a == b,
            (
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridAutoRows(a)),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridAutoRows(b)),
            ) => a == b,
            (
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridAutoColumns(a)),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridAutoColumns(b)),
            ) => a == b,
            (
                CssContributionValueRef::Ordinary(CssLonghandValueRef::ListStyleImage(a)),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::ListStyleImage(b)),
            ) => a == b,
            (
                CssContributionValueRef::Ordinary(CssLonghandValueRef::CueBefore(a)),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::CueBefore(b)),
            ) => a == b,
            (
                CssContributionValueRef::Ordinary(CssLonghandValueRef::CueAfter(a)),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::CueAfter(b)),
            ) => a == b,
            _ => panic!("selected comparison owner"),
        }
    }

    fn reached(
        property: CssKnownProperty,
        left: &str,
        right: &str,
        expected: bool,
        projections: usize,
    ) {
        let left = terminal(property, left, false);
        let right = terminal(property, right, true);
        assert!(
            !left.items()[0]
                .source()
                .same_occurrence(right.items()[0].source())
        );
        assert_eq!(
            compare(&left, &right, projections).unwrap(),
            expected,
            "{property:?}"
        );
        if projections != 0 {
            assert_eq!(
                compare(&left, &right, projections - 1).unwrap_err().kind(),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                "{property:?}"
            );
        }
        assert_eq!(
            compare(&left, &right, projections).unwrap(),
            expected,
            "{property:?}: adequate retry"
        );
    }

    fn image(values: &CssLonghandContributions) -> &CssImageValue {
        let CssContributionValueRef::Ordinary(CssLonghandValueRef::ListStyleImage(value)) =
            values.items()[0].value()
        else {
            panic!("list-style-image")
        };
        value
    }

    fn stop(values: &CssLonghandContributions) -> &CssGradientColorStop {
        let CssImageValue::Gradient(CssGradient::Linear(gradient)) = image(values) else {
            panic!("linear gradient")
        };
        let CssColorStopListItem::Stop(stop) = &gradient.stops().items()[0] else {
            panic!("first real stop")
        };
        stop
    }

    fn shadow(values: &CssLonghandContributions) -> &CssDropShadow {
        let CssImageValue::Filter(filter) = image(values) else {
            panic!("filter image")
        };
        let [CssFilterFunction::DropShadow(shadow)] = filter.filters().functions() else {
            panic!("one real shadow")
        };
        shadow
    }

    fn bounded_work<T: BoundedStructuralEquality + ?Sized>(
        left: &T,
        right: &T,
        expected: bool,
        projections: usize,
    ) {
        assert_eq!(
            left.bounded_eq(right, &mut context(projections)).unwrap(),
            expected
        );
        if projections != 0 {
            assert_eq!(
                left.bounded_eq(right, &mut context(projections - 1))
                    .unwrap_err()
                    .kind(),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            );
        }
        assert_eq!(
            left.bounded_eq(right, &mut context(projections)).unwrap(),
            expected,
            "adequate retry"
        );
    }

    #[test]
    fn shifted_grid_literals_ignore_coordinates_and_preserve_raw_owner_equality() {
        for property in [
            CssKnownProperty::GridTemplateRows,
            CssKnownProperty::GridTemplateColumns,
            CssKnownProperty::GridAutoRows,
            CssKnownProperty::GridAutoColumns,
        ] {
            for value in ["1px", "1fr", "minmax(1px, 2fr)", "fit-content(1px)"] {
                let left = terminal(property, value, false);
                let right = terminal(property, value, true);
                assert!(
                    !raw_equal(&left, &right),
                    "{property:?}: {value}: ordinary coordinates retained"
                );
                assert!(compare(&left, &right, 0).unwrap(), "{property:?}: {value}");
            }
        }
        for value in [
            "repeat(2, 1px)",
            "1px repeat(auto-fill, 2px)",
            "repeat(2, 1px) repeat(auto-fit, 2px)",
        ] {
            reached(CssKnownProperty::GridTemplateRows, value, value, true, 0);
        }
        for value in [
            "none",
            "auto",
            "min-content",
            "max-content",
            "subgrid [a] repeat(2, [b])",
        ] {
            reached(CssKnownProperty::GridTemplateRows, value, value, true, 0);
        }
    }

    #[test]
    fn shifted_grid_math_admits_each_reached_numeric_owner_without_carrier_fees() {
        // Each direct two-leaf calc is Function/Sum/two Values P4.
        // Two MinMax children, or count then math content, are P8.
        for property in [
            CssKnownProperty::GridTemplateRows,
            CssKnownProperty::GridTemplateColumns,
            CssKnownProperty::GridAutoRows,
            CssKnownProperty::GridAutoColumns,
        ] {
            for (value, projections) in [
                ("calc(1px + 2px)", 4),
                ("calc(1fr + 2fr)", 4),
                ("fit-content(calc(1px + 2px))", 4),
                ("minmax(calc(1px + 2px), calc(1fr + 2fr))", 8),
            ] {
                let left = terminal(property, value, false);
                let right = terminal(property, value, true);
                assert!(!raw_equal(&left, &right), "{property:?}: {value}");
                reached(property, value, value, true, projections);
            }
        }
        for (value, projections) in [
            ("repeat(2, calc(1px + 2px))", 4),
            ("repeat(2, fit-content(calc(1px + 2px)))", 4),
            ("repeat(calc(1 + 2), calc(1px + 2px))", 8),
            ("repeat(2, calc(1px + 2px)) repeat(auto-fill, 1px)", 4),
            ("calc(1px + 2px) repeat(auto-fill, 1px)", 4),
            (
                "repeat(2, minmax(calc(1px + 2px), calc(1fr + 2fr))) repeat(auto-fill, 1px)",
                8,
            ),
            ("repeat(auto-fill, calc(1px + 2px))", 4),
            (
                "repeat(auto-fill, minmax(calc(1px + 2px), calc(1fr + 2fr)))",
                8,
            ),
            ("subgrid repeat(calc(1 + 2), [a])", 4),
        ] {
            reached(
                CssKnownProperty::GridTemplateRows,
                value,
                value,
                true,
                projections,
            );
        }
    }

    #[test]
    fn grid_false_descendants_retain_min_max_count_content_and_component_order() {
        for (left, right, projections) in [
            ("calc(1px + 2px)", "calc(1px + 3px)", 4),
            ("calc(1px + 2px)", "calc(2px + 1px)", 4),
            ("calc(1px + 2px)", "calc(01px + 2px)", 4),
            ("calc(1fr + 2fr)", "calc(1fr + 3fr)", 4),
            ("calc(2px + 1px)", "calc(2px - 1px)", 1),
            (
                "fit-content(calc(1px + 2px))",
                "fit-content(calc(1px + 3px))",
                4,
            ),
            (
                "minmax(calc(1px + 2px), calc(1fr + 2fr))",
                "minmax(calc(1px + 3px), calc(1fr + 2fr))",
                4,
            ),
            (
                "minmax(1px, calc(1fr + 2fr))",
                "minmax(2px, calc(1fr + 2fr))",
                0,
            ),
            (
                "minmax(calc(1px + 2px), 1fr)",
                "minmax(calc(1px + 2px), 2fr)",
                4,
            ),
            (
                "repeat(calc(1 + 2), calc(1px + 2px))",
                "repeat(calc(1 + 3), calc(1px + 2px))",
                4,
            ),
            (
                "repeat(1, calc(1px + 2px))",
                "repeat(2, calc(1px + 2px))",
                0,
            ),
            (
                "repeat(auto-fill, calc(1px + 2px))",
                "repeat(auto-fit, calc(1px + 2px))",
                0,
            ),
            ("1px calc(1fr + 2fr)", "2px calc(1fr + 2fr)", 0),
        ] {
            reached(
                CssKnownProperty::GridTemplateRows,
                left,
                right,
                false,
                projections,
            );
        }
        reached(
            CssKnownProperty::GridAutoRows,
            "1px calc(1fr + 2fr)",
            "2px calc(1fr + 2fr)",
            false,
            0,
        );
    }

    #[test]
    fn grid_domains_spelling_math_phase_and_representation_stops_are_exact() {
        for (left, right) in [
            ("1px", "01px"),
            ("1fr", "1.0fr"),
            ("0px", "0fr"),
            ("calc(1px + 2px)", "3px"),
            ("calc(1px + 2px)", "fit-content(calc(1px + 2px))"),
            ("calc(1px + 2px)", "calc(1fr + 2fr)"),
            ("calc(1px + 2px)", "calc(1px + 2px) 1px"),
            (
                "repeat(calc(1 + 2), calc(1px + 2px))",
                "subgrid repeat(calc(1 + 2), [a])",
            ),
        ] {
            // LP/Fraction are rejected by the breadth variant before entering
            // the numeric owner; no type-mismatch graph tariff is invented.
            reached(CssKnownProperty::GridTemplateRows, left, right, false, 0);
        }
        reached(
            CssKnownProperty::GridAutoColumns,
            "calc(1px + 2px)",
            "calc(1px + 2px) 1px",
            false,
            0,
        );
    }

    #[test]
    fn real_gradient_stops_reuse_color_semantics_and_admit_its_selected_work() {
        for (left, right, expected, projections) in [
            ("rgb(1 2 3)", "rgb(1 2 3 / 1)", true, 5),
            ("rgb(01 2 3)", "rgb(1 2 3)", true, 5),
            ("rgb(1 2 3)", "rgb(2 1 3)", false, 2),
            ("rgb(calc(1 + 2) 2 3)", "rgb(calc(1 + 2) 2 3)", true, 9),
            ("rgb(calc(2 + 1) 2 3)", "rgb(calc(2 - 1) 2 3)", false, 3),
            ("rgb(calc(1 + 2) 2 3)", "rgb(3 2 3)", false, 2),
            ("rgb(1 2 3)", "color(srgb 1 2 3)", false, 1),
            ("color(--P 1 2 3)", "color(--p 1 2 3)", false, 1),
        ] {
            let left = terminal(
                CssKnownProperty::ListStyleImage,
                &format!("linear-gradient({left}, blue)"),
                false,
            );
            let right = terminal(
                CssKnownProperty::ListStyleImage,
                &format!("linear-gradient({right}, blue)"),
                true,
            );
            assert_ne!(
                stop(&left),
                stop(&right),
                "ordinary stop retains authored Color"
            );
            bounded_work(stop(&left), stop(&right), expected, projections);
        }
        // Full schema route also reaches the final named blue Color (P1).
        reached(
            CssKnownProperty::ListStyleImage,
            "linear-gradient(rgb(1 2 3), blue)",
            "linear-gradient(rgb(1 2 3 / 1), blue)",
            true,
            6,
        );
        reached(
            CssKnownProperty::ListStyleImage,
            "linear-gradient(rgb(calc(1 + 2) 2 3), blue)",
            "linear-gradient(rgb(calc(1 + 2) 2 3), blue)",
            true,
            10,
        );
    }

    #[test]
    fn gradient_positions_short_circuit_color_and_keep_alpha_source_binding_phase() {
        for (left, right, expected, projections) in [
            ("rgb(1 2 3) 1px", "rgb(9 8 7) 2px", false, 0),
            (
                "rgb(1 2 3) calc(1px + 2px)",
                "rgb(1 2 3) calc(1px + 2px)",
                true,
                9,
            ),
            (
                "rgb(1 2 3) calc(2px + 1px)",
                "rgb(1 2 3) calc(2px - 1px)",
                false,
                1,
            ),
            ("rgb(1 2 3)", "rgb(1 2 3 / calc(1))", false, 5),
        ] {
            let left = terminal(
                CssKnownProperty::ListStyleImage,
                &format!("linear-gradient({left}, blue)"),
                false,
            );
            let right = terminal(
                CssKnownProperty::ListStyleImage,
                &format!("linear-gradient({right}, blue)"),
                true,
            );
            bounded_work(stop(&left), stop(&right), expected, projections);
        }
        for (left, right) in [
            ("rgb(from red r g b)", "rgb(from blue r g b)"),
            ("rgb(from red r g b)", "rgb(from red r g b / alpha)"),
        ] {
            let left = terminal(
                CssKnownProperty::ListStyleImage,
                &format!("linear-gradient({left}, blue)"),
                false,
            );
            let right = terminal(
                CssKnownProperty::ListStyleImage,
                &format!("linear-gradient({right}, blue)"),
                true,
            );
            assert!(
                !stop(&left)
                    .bounded_eq(stop(&right), &mut context(usize::MAX))
                    .unwrap()
            );
        }
    }

    #[test]
    fn real_drop_shadows_compare_offsets_before_present_color_and_keep_optional_defaults() {
        for (left, right, expected, projections) in [
            ("rgb(1 2 3) 1px 2px", "rgb(1 2 3 / 1) 1px 2px", true, 5),
            (
                "rgb(calc(1 + 2) 2 3) 1px 2px",
                "rgb(calc(1 + 2) 2 3) 1px 2px",
                true,
                9,
            ),
            (
                "rgb(calc(2 + 1) 2 3) 1px 2px",
                "rgb(calc(2 - 1) 2 3) 1px 2px",
                false,
                3,
            ),
            (
                "rgb(1 2 3) calc(1px + 2px) 2px",
                "rgb(1 2 3) calc(1px + 2px) 2px",
                true,
                9,
            ),
            ("rgb(1 2 3) 1px 2px", "rgb(9 8 7) 2px 2px", false, 0),
            ("rgb(1 2 3) 1px 2px", "rgb(1 2 3) 1px 2px 0px", false, 0),
            ("1px 2px", "currentcolor 1px 2px", false, 0),
        ] {
            let left = terminal(
                CssKnownProperty::ListStyleImage,
                &format!("filter(url(a), drop-shadow({left}))"),
                false,
            );
            let right = terminal(
                CssKnownProperty::ListStyleImage,
                &format!("filter(url(a), drop-shadow({right}))"),
                true,
            );
            bounded_work(shadow(&left), shadow(&right), expected, projections);
        }
        reached(
            CssKnownProperty::ListStyleImage,
            "filter(url(a), drop-shadow(rgb(1 2 3) 1px 2px))",
            "filter(url(a), drop-shadow(rgb(1 2 3 / 1) 1px 2px))",
            true,
            5,
        );
    }

    fn cue(values: &CssLonghandContributions) -> &CssCue {
        match values.items()[0].value() {
            CssContributionValueRef::Ordinary(CssLonghandValueRef::CueBefore(value))
            | CssContributionValueRef::Ordinary(CssLonghandValueRef::CueAfter(value)) => value,
            _ => panic!("cue terminal"),
        }
    }

    #[test]
    fn cue_dispatch_and_shortening_share_borrowed_exact_url_decibel_equivalence() {
        for property in [CssKnownProperty::CueBefore, CssKnownProperty::CueAfter] {
            for (left, right, expected) in [
                ("url(a) 1dB", "url('a') 01DB", true),
                ("url(a)", "url(a) -0dB", true),
                ("url(a) -3dB", "url(a) -3.0dB", true),
                ("url(a) 1dB", "url(a) 2dB", false),
                ("url(a)", "src('a')", false),
                ("url(a)", "url(b)", false),
                ("src('a' mod(x))", "src('a' mod(y))", false),
                ("url(a) -1e-400dB", "url(a) -2e-400dB", false),
                ("none", "url(a)", false),
            ] {
                let left = terminal(property, left, false);
                let right = terminal(property, right, true);
                assert_eq!(compare(&left, &right, 0).unwrap(), expected, "{property:?}");
                assert_eq!(
                    shorter_second(cue(&left), cue(&right).clone(), &mut context(0))
                        .unwrap()
                        .is_none(),
                    expected
                );
            }
            let left = terminal(property, "url(a) 1dB", false);
            let right = terminal(property, "url(a) 1dB", true);
            assert!(
                !raw_equal(&left, &right),
                "ordinary decibel keeps coordinates"
            );
            assert!(compare(&left, &right, 0).unwrap());
            assert!(
                shorter_second(cue(&left), cue(&right).clone(), &mut context(0))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn selected_inverse_relations_preserve_normal_color_cue_and_grid_provider_contracts() {
        let values = terminal(
            CssKnownProperty::ListStyleImage,
            "linear-gradient(rgb(1 2 3 / 1), blue)",
            false,
        );
        assert_eq!(
            stop(&values).color().to_specified_css().unwrap(),
            "rgb(1, 2, 3)"
        );
        for (value, expected, inputs) in
            [("url(a)", "url(\"a\")", 3), ("url(a) 0dB", "url(\"a\")", 4)]
        {
            let values = terminal(CssKnownProperty::CueBefore, value, false);
            let limits = CssSpecifiedValueSerializationLimits::new(inputs, inputs, 8);
            assert_eq!(
                cue(&values)
                    .serialize_specified_with_limits(limits)
                    .unwrap(),
                expected
            );
            assert_eq!(
                cue(&values)
                    .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                        inputs - 1,
                        inputs,
                        8
                    ))
                    .unwrap_err()
                    .kind(),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit
            );
        }
        for property in [
            CssKnownProperty::GridTemplateRows,
            CssKnownProperty::GridAutoRows,
        ] {
            let values = terminal(property, "calc(1px + 2px)", false);
            let serialize = |limits| match values.items()[0].value() {
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridTemplateRows(value)) => {
                    value.serialize_specified_with_limits(limits)
                }
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridAutoRows(value)) => {
                    value.serialize_specified_with_limits(limits)
                }
                _ => panic!("grid provider"),
            };
            assert_eq!(
                serialize(CssSpecifiedValueSerializationLimits::new(5, 4, 9)).unwrap(),
                "calc(3px)"
            );
            assert_eq!(
                serialize(CssSpecifiedValueSerializationLimits::new(4, 4, 9))
                    .unwrap_err()
                    .kind(),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit
            );
            assert_eq!(
                serialize(CssSpecifiedValueSerializationLimits::new(5, 3, 9))
                    .unwrap_err()
                    .kind(),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            );
            assert_eq!(
                serialize(CssSpecifiedValueSerializationLimits::new(5, 4, 8))
                    .unwrap_err()
                    .kind(),
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            );
        }
    }
}
