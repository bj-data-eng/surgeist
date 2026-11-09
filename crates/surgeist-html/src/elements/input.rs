//! Shared intrinsic input rules, including the color, image and range input states.

pub use crate::internal::elements::input::{
    HtmlInputChargedStep, HtmlInputCheckedAttributeChange, HtmlInputColorEffect,
    HtmlInputColorSerialization, HtmlInputColorSpaceCursor, HtmlInputEmailMode,
    HtmlInputEmptySelectionSupport, HtmlInputFileSelection, HtmlInputFilenameInvalidState,
    HtmlInputFlags, HtmlInputImageCoordinate, HtmlInputInvalidSelectionRange,
    HtmlInputNoneDirectionSupport, HtmlInputReset, HtmlInputResetEffect, HtmlInputRetainedRange,
    HtmlInputSanitization, HtmlInputSanitizationCursor, HtmlInputSelection,
    HtmlInputSelectionDirection, HtmlInputTemporalFormat, HtmlInputTypeChange, HtmlInputTypeCursor,
    HtmlInputTypeEffect, HtmlInputValueAttributeEffect, HtmlInputValueComparison,
    HtmlInputValueComparisonCursor, HtmlInputValueMode, HtmlInputValueSanitization,
    HtmlInputValueSetterStart, HtmlRadioTrigger, input_filename_setter, input_value_setter_finish,
};
pub use crate::internal::elements::input_color::{
    HtmlInputColorValueSource, input_color_standard_failure_fallback, input_color_value_source,
};
pub use crate::internal::elements::input_image::{
    HtmlInputImageHistory, HtmlInputImageUpdateGuard, HtmlInputImagesDisabled,
};
