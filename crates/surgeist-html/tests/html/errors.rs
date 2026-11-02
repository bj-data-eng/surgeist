//! Public error values state what failed in prose and chain their causes.
//!
//! Each expected message is the documented meaning of the variant: the rule of
//! the pinned HTML Standard the value breaks, or the resource limit it names.
//! A wrapper states its own operation and exposes the wrapped failure through
//! `Error::source`; its text does not repeat the cause.

use std::error::Error;

use surgeist_html::bounded::{
    AutocompleteMaterializationError, FloatingPointAllocationError, IntegerRecoveryError,
    IntegerUnitStepError, NumericMaterializationError, NumericParseError, RangeNumericError,
};
use surgeist_html::{
    DocumentString, HtmlAcceptSpecifierError, HtmlAreaCoordinatesAuthorError,
    HtmlAttributeConformanceError, HtmlAttributeContextConformanceError,
    HtmlAutocompleteSectionError, HtmlAutocompleteSyntaxError, HtmlAutofillField,
    HtmlCustomCommandError, HtmlCustomElementNameError, HtmlDecimalDigitsError,
    HtmlDuplicateTokenError, HtmlIconSizeError, HtmlImageAttributeConformanceError,
    HtmlInputImageAttributeError, HtmlInputSelection, HtmlInputSelectionDirection,
    HtmlLinkDestinationContextError, HtmlMeterAuthorRelation, HtmlMeterNativeError,
    HtmlMeterNumberRole, HtmlMicrodataTokenListError, HtmlNavigableTargetError,
    HtmlNonNegativeInteger, HtmlNonNegativeRational, HtmlNumericAttributeConformanceError,
    HtmlOrderedTokenSet, HtmlProcessingInstructionError, HtmlProgressNumberAuthorError,
    HtmlRationalError, HtmlSpaceToken, HtmlSrcsetAuthorViolation, HtmlSrcsetContextViolation,
    HtmlSrcsetDensityError, HtmlSrcsetInterpretationError, HtmlSrcsetParseError,
    HtmlSrcsetUrlTokenError, HtmlTemporalError, HtmlTokenComparison, HtmlTokenDeclarationError,
    HtmlTokenError, HtmlUtf8EncodingLabel, HtmlValueAdmissionError, HtmlValueFailure,
    HtmlValueInterpreter, HtmlValueRepresentationError, HtmlValueSyntaxError,
    input_filename_setter,
};

/// The admission error of a three-unit value against a two-unit capacity.
fn refused_admission() -> HtmlValueAdmissionError {
    HtmlValueInterpreter::new(2)
        .admit_attribute_raw(Some(&DocumentString::from("abc")))
        .expect_err("three units exceed the two-unit capacity")
}

/// The duplicate-token error of the two spellings `a` and `A` compared ASCII
/// case-insensitively.
fn duplicate_token() -> HtmlDuplicateTokenError {
    let tokens = ["a", "A"].map(|token| HtmlSpaceToken::new(token).expect("valid token"));
    HtmlOrderedTokenSet::new(HtmlTokenComparison::AsciiCaseInsensitive, tokens)
        .expect_err("the spellings collide")
}

fn message(error: &dyn Error) -> String {
    error.to_string()
}

fn cause(error: &dyn Error) -> Option<String> {
    error.source().map(|source| source.to_string())
}

#[test]
fn numeric_parse_errors_name_the_violated_rule() {
    assert_eq!(
        message(&NumericParseError::InvalidSyntax),
        "value does not match the HTML numeric syntax"
    );
    assert_eq!(
        message(&NumericParseError::Negative),
        "value is negative where a nonnegative value is required"
    );
    assert_eq!(
        message(&NumericParseError::FloatingPointOverflow),
        "value overflows the finite binary64 range"
    );
    assert_eq!(
        message(&NumericParseError::ZeroDimension),
        "dimension is zero where a nonzero value is required"
    );
}

#[test]
fn integer_digit_boundaries_report_their_exact_numbers() {
    assert_eq!(
        message(&IntegerUnitStepError::RepresentationExhausted),
        "integer digit count cannot be represented on this platform"
    );
    assert_eq!(
        message(&IntegerUnitStepError::DigitLimitExceeded {
            limit: 4,
            required: 9
        }),
        "integer needs 9 digits, exceeding the limit of 4"
    );
    assert_eq!(
        message(&IntegerUnitStepError::RequestedCapacityLimitExceeded {
            limit: 16,
            required: 40
        }),
        "integer needs 40 bytes of digit storage, exceeding the limit of 16"
    );
    assert_eq!(
        message(&IntegerUnitStepError::ActualCapacityLimitExceeded {
            limit: 16,
            requested: 10,
            capacity_bytes: 32
        }),
        "allocating 10 bytes of digit storage produced 32 bytes, exceeding the limit of 16"
    );
    assert_eq!(
        message(&IntegerUnitStepError::AllocationFailed { requested: 7 }),
        "allocating 7 bytes of digit storage failed"
    );
}

#[test]
fn floating_point_allocation_boundaries_report_their_exact_numbers() {
    assert_eq!(
        message(&FloatingPointAllocationError::RepresentationExhausted),
        "floating-point token length cannot be represented on this platform"
    );
    assert_eq!(
        message(
            &FloatingPointAllocationError::RequestedCapacityLimitExceeded {
                limit: 8,
                required: 12
            }
        ),
        "floating-point token needs 12 bytes of storage, exceeding the limit of 8"
    );
    assert_eq!(
        message(&FloatingPointAllocationError::ActualCapacityLimitExceeded {
            limit: 8,
            requested: 6,
            capacity_bytes: 24
        }),
        "allocating 6 bytes of floating-point token storage produced 24 bytes, exceeding the limit of 8"
    );
    assert_eq!(
        message(&FloatingPointAllocationError::AllocationFailed { requested: 5 }),
        "allocating 5 bytes of floating-point token storage failed"
    );
}

#[test]
fn numeric_wrappers_chain_the_wrapped_failure() {
    let syntax = IntegerRecoveryError::Syntax(NumericParseError::Negative);
    assert_eq!(message(&syntax), "integer recovery rejected the input");
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("value is negative where a nonnegative value is required")
    );
    let allocation =
        IntegerRecoveryError::Allocation(IntegerUnitStepError::AllocationFailed { requested: 3 });
    assert_eq!(
        message(&allocation),
        "integer recovery could not store its digits"
    );
    assert_eq!(
        cause(&allocation).as_deref(),
        Some("allocating 3 bytes of digit storage failed")
    );

    let digits = NumericMaterializationError::Digits(HtmlDecimalDigitsError::Empty);
    assert_eq!(
        message(&digits),
        "numeric digits are not a valid decimal magnitude"
    );
    assert_eq!(
        cause(&digits).as_deref(),
        Some("HTML integer magnitude has no decimal digits")
    );
    let boundary =
        NumericMaterializationError::Boundary(IntegerUnitStepError::RepresentationExhausted);
    assert_eq!(
        message(&boundary),
        "numeric materialization reached an allocation boundary"
    );
    assert_eq!(
        cause(&boundary).as_deref(),
        Some("integer digit count cannot be represented on this platform")
    );
}

#[test]
fn rational_with_zero_denominator_says_so() {
    let zero = HtmlNonNegativeInteger::from(0u128);
    let one = HtmlNonNegativeInteger::from(1u128);
    let error = HtmlNonNegativeRational::new(one, zero).expect_err("zero denominator");
    assert_eq!(error, HtmlRationalError::ZeroDenominator);
    assert_eq!(message(&error), "rational denominator is zero");
}

#[test]
fn range_numeric_errors_name_the_failed_step() {
    assert_eq!(
        message(&RangeNumericError::Integrity),
        "range numeric reply does not match its pending request"
    );
    assert_eq!(
        message(&RangeNumericError::Representation),
        "range numeric length cannot be represented on this platform"
    );
    assert_eq!(
        message(&RangeNumericError::Overflow),
        "range numeric value exceeds the representable range"
    );
    assert_eq!(
        message(&RangeNumericError::InvalidSyntax),
        "range numeric input does not match the HTML numeric syntax"
    );
}

#[test]
fn custom_element_name_errors_follow_the_valid_custom_element_name_rules() {
    assert_eq!(
        message(&HtmlCustomElementNameError::Empty),
        "custom-element name is empty"
    );
    assert_eq!(
        message(&HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii),
        "custom-element name does not start with a lowercase ASCII letter"
    );
    assert_eq!(
        message(&HtmlCustomElementNameError::ForbiddenCodeUnit {
            code_unit_offset: 3,
            code_unit: 0x41
        }),
        "custom-element name contains forbidden code unit U+0041 at UTF-16 offset 3"
    );
    assert_eq!(
        message(&HtmlCustomElementNameError::MissingHyphen),
        "custom-element name contains no hyphen"
    );
    assert_eq!(
        message(&HtmlCustomElementNameError::ReservedName),
        "custom-element name is reserved by HTML"
    );
}

#[test]
fn token_errors_locate_the_offending_code_unit() {
    assert_eq!(
        message(&HtmlTokenError::EmptySpaceToken),
        "space-separated token is empty"
    );
    assert_eq!(
        message(&HtmlTokenError::SpaceTokenWhitespace {
            code_unit_offset: 2
        }),
        "space-separated token contains ASCII whitespace at UTF-16 offset 2"
    );
    assert_eq!(
        message(&HtmlTokenError::CommaTokenBoundaryWhitespace {
            code_unit_offset: 0
        }),
        "comma-separated token has leading or trailing ASCII whitespace at UTF-16 offset 0"
    );
    assert_eq!(
        message(&HtmlTokenError::CommaTokenComma {
            code_unit_offset: 4
        }),
        "comma-separated token contains a comma at UTF-16 offset 4"
    );
}

#[test]
fn processing_instruction_errors_name_the_violated_constraint() {
    assert_eq!(
        message(&HtmlProcessingInstructionError::InvalidTarget),
        "processing-instruction target is not a valid XML name"
    );
    assert_eq!(
        message(&HtmlProcessingInstructionError::ReservedTarget),
        "processing-instruction target is reserved (xml or xml-stylesheet)"
    );
    assert_eq!(
        message(&HtmlProcessingInstructionError::DataContainsGreaterThan),
        "processing-instruction data contains '>'"
    );
    assert_eq!(
        message(&HtmlProcessingInstructionError::DataEndsWithQuestionMark),
        "processing-instruction data ends with '?'"
    );
}

#[test]
fn token_declaration_errors_identify_the_token_and_chain_the_cause() {
    let duplicate = duplicate_token();
    assert_eq!(
        message(&HtmlTokenDeclarationError::Duplicate(duplicate)),
        "token declaration contains a duplicate token"
    );
    assert_eq!(
        message(&HtmlTokenDeclarationError::AccessKeyLength { token_index: 1 }),
        "accesskey token at index 1 is not exactly one code point"
    );
    assert_eq!(
        message(&HtmlTokenDeclarationError::InvalidBlockingToken { token_index: 2 }),
        "blocking token at index 2 is not a recognized keyword"
    );
    let size = HtmlTokenDeclarationError::InvalidIconSize {
        token_index: 0,
        reason: HtmlIconSizeError::LeadingZero,
    };
    assert_eq!(message(&size), "icon size token at index 0 is invalid");
    assert_eq!(
        cause(&size).as_deref(),
        Some("icon size dimension has a leading zero")
    );
}

#[test]
fn icon_size_errors_follow_the_sizes_grammar() {
    assert_eq!(
        message(&HtmlIconSizeError::ZeroDimension),
        "icon size has a zero width or height"
    );
    assert_eq!(
        message(&HtmlIconSizeError::LeadingZero),
        "icon size dimension has a leading zero"
    );
    assert_eq!(
        message(&HtmlIconSizeError::Separator),
        "icon size dimensions are not separated by a single 'x' or 'X'"
    );
    assert_eq!(
        message(&HtmlIconSizeError::DimensionDigits),
        "icon size dimension is not a sequence of ASCII digits"
    );
}

#[test]
fn meter_errors_name_the_attribute_roles_and_the_ordering_rule() {
    let syntax = HtmlMeterNativeError::Syntax {
        role: HtmlMeterNumberRole::Low,
        error: HtmlValueSyntaxError::InvalidNumericSyntax,
    };
    assert_eq!(
        message(&syntax),
        "meter low attribute is not a valid floating-point number"
    );
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("value does not match the selected numeric syntax")
    );
    assert_eq!(
        message(&HtmlMeterNativeError::RelationViolation(
            HtmlMeterAuthorRelation::MinimumValue
        )),
        "meter min must not exceed value"
    );
    assert_eq!(
        message(&HtmlMeterNativeError::RelationViolation(
            HtmlMeterAuthorRelation::OptimumMaximum
        )),
        "meter optimum must not exceed max"
    );
    assert_eq!(
        message(&HtmlMeterNativeError::RepresentationUnavailable),
        "meter attributes cannot be compared in the selected numeric model"
    );
}

#[test]
fn autocomplete_materialization_errors_report_the_charged_resource() {
    assert_eq!(
        message(&AutocompleteMaterializationError::BytesLimit {
            limit: 10,
            required: 20
        }),
        "autocomplete materialization needs 20 allocated bytes, exceeding the limit of 10"
    );
    assert_eq!(
        message(&AutocompleteMaterializationError::OutputLimit {
            limit: 1,
            required: 2
        }),
        "autocomplete materialization needs 2 nested output records, exceeding the limit of 1"
    );
    assert_eq!(
        message(&AutocompleteMaterializationError::ScratchLimit {
            limit: 3,
            required: 5
        }),
        "autocomplete materialization needs 5 scratch records, exceeding the limit of 3"
    );
    assert_eq!(
        message(&AutocompleteMaterializationError::Allocation {
            requested_bytes: 64
        }),
        "allocating 64 bytes for autocomplete materialization failed"
    );
    assert_eq!(
        message(&AutocompleteMaterializationError::Representation),
        "autocomplete materialization size cannot be represented on this platform"
    );
    let admission = refused_admission();
    let refused = AutocompleteMaterializationError::RawAdmission(admission);
    assert_eq!(
        message(&refused),
        "autocomplete value was refused by raw-value admission"
    );
    assert_eq!(cause(&refused), Some(admission.to_string()));
}

#[test]
fn autocomplete_section_errors_chain_the_token_failure() {
    let token = HtmlAutocompleteSectionError::Token(HtmlTokenError::EmptySpaceToken);
    assert_eq!(
        message(&token),
        "autocomplete section is not a valid space-separated token"
    );
    assert_eq!(
        cause(&token).as_deref(),
        Some("space-separated token is empty")
    );
    assert_eq!(
        message(&HtmlAutocompleteSectionError::MissingSectionPrefix),
        "autocomplete section does not start with \"section-\""
    );
}

#[test]
fn input_image_and_microdata_and_accept_errors_state_their_rule() {
    assert_eq!(
        message(&HtmlInputImageAttributeError::EmptyAlt),
        "image-button alt attribute is empty"
    );
    assert_eq!(
        message(&HtmlInputImageAttributeError::EmptySource),
        "image-button src attribute is empty"
    );
    assert_eq!(
        message(&HtmlMicrodataTokenListError::Empty),
        "microdata token list contains no tokens"
    );
    assert_eq!(
        message(&HtmlAcceptSpecifierError::MissingLeadingDot),
        "accept file extension does not start with a full stop"
    );
    let token = HtmlAcceptSpecifierError::Token(HtmlTokenError::CommaTokenComma {
        code_unit_offset: 1,
    });
    assert_eq!(
        message(&token),
        "accept specifier is not a valid comma-separated token"
    );
    assert_eq!(
        cause(&token).as_deref(),
        Some("comma-separated token contains a comma at UTF-16 offset 1")
    );
}

#[test]
fn srcset_value_errors_follow_the_image_candidate_grammar() {
    assert_eq!(
        message(&HtmlSrcsetUrlTokenError::Empty),
        "srcset image candidate URL is empty"
    );
    assert_eq!(
        message(&HtmlSrcsetUrlTokenError::AsciiWhitespace),
        "srcset image candidate URL contains ASCII whitespace"
    );
    assert_eq!(
        message(&HtmlSrcsetUrlTokenError::EdgeComma),
        "srcset image candidate URL starts or ends with a comma"
    );
    assert_eq!(
        message(&HtmlSrcsetDensityError::InvalidSyntax),
        "srcset density is not a valid floating-point number"
    );
    assert_eq!(
        message(&HtmlSrcsetDensityError::Nonpositive),
        "srcset density is not greater than zero"
    );
    assert_eq!(
        message(&HtmlSrcsetDensityError::RepresentationExhausted),
        "srcset density size cannot be represented on this platform"
    );
}

#[test]
fn srcset_author_violations_name_the_candidates_involved() {
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::EmptyList),
        "srcset has no image candidate strings"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::MissingCandidate),
        "srcset has a comma without an image candidate string"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::InvalidUrlToken),
        "srcset image candidate URL is not a valid URL token"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::InvalidDescriptor),
        "srcset image candidate descriptor is not a valid width or density"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::NonpositiveDescriptor),
        "srcset image candidate descriptor is not greater than zero"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::MultipleDescriptors),
        "srcset image candidate has more than one descriptor"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::FutureHeightDescriptor),
        "srcset image candidate uses the unsupported height descriptor"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::DuplicateWidth {
            first: 0,
            duplicate: 2
        }),
        "srcset image candidate 2 repeats the width of candidate 0"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::DuplicateDensity {
            first: 1,
            duplicate: 3
        }),
        "srcset image candidate 3 repeats the density of candidate 1"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::MixedWidthDescriptors),
        "srcset has a candidate without a width descriptor beside width descriptors"
    );
}

#[test]
fn srcset_interpretation_errors_chain_raw_admission() {
    let admission = refused_admission();
    let refused = HtmlSrcsetInterpretationError::RawAdmission(admission);
    assert_eq!(
        message(&refused),
        "srcset value was refused by raw-value admission"
    );
    assert_eq!(cause(&refused), Some(admission.to_string()));
    let exhausted = HtmlSrcsetInterpretationError::RepresentationExhausted;
    assert_eq!(
        message(&exhausted),
        "srcset interpretation size cannot be represented on this platform"
    );
    assert!(exhausted.source().is_none());
}

#[test]
fn temporal_syntax_failures_chain_through_value_syntax() {
    let syntax = HtmlValueSyntaxError::Temporal(HtmlTemporalError::ZeroYear);
    assert_eq!(message(&syntax), "value is not a valid temporal string");
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("HTML year must be greater than zero")
    );
}

#[test]
fn common_value_failures_split_syntax_from_representation() {
    use HtmlValueSyntaxError as Syntax;
    assert_eq!(
        message(&Syntax::EmptyLegacyColor),
        "legacy color value is empty"
    );
    assert_eq!(
        message(&Syntax::TransparentLegacyColor),
        "legacy color value is the transparent keyword"
    );
    assert_eq!(
        message(&Syntax::InvalidNumericSyntax),
        "value does not match the selected numeric syntax"
    );
    assert_eq!(
        message(&Syntax::NegativeNonnegativeInteger),
        "nonnegative integer recovery produced a negative integer"
    );
    assert_eq!(
        message(&Syntax::ZeroNonzeroDimension),
        "nonzero dimension recovery produced zero"
    );
    assert_eq!(
        message(&HtmlValueRepresentationError::FloatingPointOverflow),
        "decimal number rounds to infinity in binary64"
    );
    let syntax = HtmlValueFailure::Syntax(Syntax::EmptyLegacyColor);
    assert_eq!(
        message(&syntax),
        "value recovery failed on source-defined syntax"
    );
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("legacy color value is empty")
    );
    let representation =
        HtmlValueFailure::Representation(HtmlValueRepresentationError::FloatingPointOverflow);
    assert_eq!(
        message(&representation),
        "value recovery produced an unrepresentable value"
    );
    assert_eq!(
        cause(&representation).as_deref(),
        Some("decimal number rounds to infinity in binary64")
    );
}

#[test]
fn attribute_conformance_errors_name_the_violated_author_requirement() {
    use HtmlAttributeConformanceError as E;
    let plain = [
        (
            E::UnrecognizedKeyword,
            "attribute value is not a recognized keyword",
        ),
        (
            E::NonconformingKeyword,
            "attribute keyword is forbidden in authored markup",
        ),
        (
            E::BooleanSpelling,
            "boolean attribute value is neither empty nor the attribute's name",
        ),
        (
            E::BdoDirectionRequired,
            "bdo element requires a dir attribute",
        ),
        (
            E::BdoDirectionMustBeLtrOrRtl,
            "bdo element dir attribute must be ltr or rtl",
        ),
        (
            E::DetailsNameMustNotBeEmpty,
            "details name attribute must not be empty",
        ),
        (
            E::FormNameMustNotBeEmpty,
            "form name attribute must not be empty",
        ),
        (
            E::DirnameMustNotBeEmpty,
            "dirname attribute must not be empty",
        ),
    ];
    for (error, expected) in plain {
        assert_eq!(message(&error), expected);
        assert!(error.source().is_none());
    }
    assert_eq!(
        message(&E::InputPlaceholderContainsNewline {
            code_unit_offset: 5
        }),
        "input placeholder contains a line break at UTF-16 offset 5"
    );
    let id = E::InvalidIdToken(HtmlTokenError::EmptySpaceToken);
    assert_eq!(message(&id), "id attribute is not a valid token");
    assert_eq!(
        cause(&id).as_deref(),
        Some("space-separated token is empty")
    );
    let map = E::InvalidMapNameToken(HtmlTokenError::SpaceTokenWhitespace {
        code_unit_offset: 1,
    });
    assert_eq!(message(&map), "map name attribute is not a valid token");
    assert_eq!(
        cause(&map).as_deref(),
        Some("space-separated token contains ASCII whitespace at UTF-16 offset 1")
    );
    let target = E::InvalidNavigableTarget(HtmlNavigableTargetError::Empty);
    assert_eq!(
        message(&target),
        "attribute is not a valid navigable target name"
    );
    assert_eq!(
        cause(&target).as_deref(),
        Some("navigable target name is empty")
    );
    let label =
        E::InvalidUtf8EncodingLabel(HtmlUtf8EncodingLabel::new("latin1").expect_err("not UTF-8"));
    assert_eq!(
        message(&label),
        "encoding label is not the required UTF-8 spelling"
    );
    assert!(label.source().is_some());
    let name = E::InvalidCustomElementName(HtmlCustomElementNameError::MissingHyphen);
    assert_eq!(
        message(&name),
        "is attribute is not a valid custom-element name"
    );
    assert_eq!(
        cause(&name).as_deref(),
        Some("custom-element name contains no hyphen")
    );
}

#[test]
fn context_conformance_errors_name_the_missing_context() {
    use HtmlAttributeContextConformanceError as E;
    let cases = [
        (
            E::ImageMapRequiresAncestorAnchorWithHref,
            "image map attribute requires an ancestor a element with an href attribute",
        ),
        (
            E::ImageControlsRequireNonemptyAlt,
            "image controls attribute requires a nonempty alt attribute",
        ),
        (
            E::StylesheetDisabledRequiresStylesheetRel,
            "disabled attribute on a link requires the stylesheet relationship",
        ),
        (
            E::ReferrerPolicyRequiresHref,
            "referrerpolicy attribute requires an href attribute",
        ),
        (
            E::InputStateForbidsAttribute,
            "attribute is prohibited for the input element's type state",
        ),
        (
            E::ScriptContextForbidsAttribute,
            "attribute is prohibited for this script type and placement",
        ),
        (
            E::FormnovalidateRequiresSubmitButton,
            "formnovalidate attribute requires a submit button",
        ),
        (
            E::FormOverrideRequiresSubmitButton,
            "formmethod and formenctype attributes require a submit button",
        ),
        (
            E::TextareaHardWrapRequiresCols,
            "textarea with hard wrapping requires a cols attribute",
        ),
        (
            E::BlockingRequiresStylesheetOrExpectRel,
            "blocking attribute on a link requires the stylesheet or expect relationship",
        ),
        (
            E::IconSizesRequireIconRel,
            "sizes attribute on a link requires the icon or apple-touch-icon relationship",
        ),
        (
            E::SourceDimensionsRequirePictureParent,
            "width and height attributes on source require a picture parent",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(message(&error), expected);
    }
}

#[test]
fn navigable_target_and_custom_command_errors_follow_the_name_grammars() {
    assert_eq!(
        message(&HtmlNavigableTargetError::ReservedLeadingUnderscore),
        "navigable target name starts with an underscore"
    );
    assert_eq!(
        message(&HtmlNavigableTargetError::TabOrNewlineWithLessThan {
            tab_or_newline_code_unit_offset: 1,
            less_than_code_unit_offset: 4
        }),
        "navigable target name contains a tab or newline at UTF-16 offset 1 and a less-than sign at offset 4"
    );
    assert_eq!(
        message(&HtmlCustomCommandError::MissingPrefix),
        "custom command does not start with two hyphens"
    );
    assert_eq!(
        message(&HtmlCustomCommandError::BoundaryWhitespace),
        "custom command starts or ends with ASCII whitespace"
    );
}

#[test]
fn numeric_image_progress_link_and_area_conformance_errors_state_their_rule() {
    let common = HtmlNumericAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(
        HtmlValueSyntaxError::InvalidNumericSyntax,
    ));
    assert_eq!(message(&common), "numeric attribute value is not valid");
    assert_eq!(
        cause(&common).as_deref(),
        Some("value recovery failed on source-defined syntax")
    );
    assert_eq!(
        message(&HtmlNumericAttributeConformanceError::ZeroMustBePositive),
        "numeric attribute value must be positive"
    );
    assert_eq!(
        message(&HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }),
        "numeric attribute value exceeds its maximum of 1000"
    );

    assert_eq!(
        message(&HtmlImageAttributeConformanceError::RequiredAttributeMissing),
        "required image attribute is missing"
    );
    assert_eq!(
        message(&HtmlImageAttributeConformanceError::EmptyAlt),
        "image alt attribute is empty"
    );
    assert_eq!(
        message(&HtmlImageAttributeConformanceError::EmptySource),
        "image src attribute is empty"
    );
    let dimension = HtmlImageAttributeConformanceError::CommonValue(
        HtmlValueFailure::Representation(HtmlValueRepresentationError::FloatingPointOverflow),
    );
    assert_eq!(
        message(&dimension),
        "image dimension attribute value is not valid"
    );
    assert!(dimension.source().is_some());

    let syntax = HtmlProgressNumberAuthorError::Syntax(HtmlValueSyntaxError::InvalidNumericSyntax);
    assert_eq!(
        message(&syntax),
        "progress attribute is not a valid floating-point number"
    );
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("value does not match the selected numeric syntax")
    );
    assert_eq!(
        message(&HtmlProgressNumberAuthorError::NegativeValue),
        "progress value attribute is negative"
    );
    assert_eq!(
        message(&HtmlProgressNumberAuthorError::NonpositiveMaximum),
        "progress max attribute is not greater than zero"
    );

    assert_eq!(
        message(&HtmlLinkDestinationContextError::PreloadRequiresAs),
        "preload link requires an as attribute"
    );
    assert_eq!(
        message(&HtmlLinkDestinationContextError::PreloadRequiresPreloadDestination),
        "preload link as attribute must name a preload destination"
    );
    assert_eq!(
        message(&HtmlLinkDestinationContextError::ModulePreloadRequiresModuleDestination),
        "modulepreload link as attribute must name a script-like destination"
    );
    assert_eq!(
        message(&HtmlLinkDestinationContextError::AsRequiresPreloadOrModulePreloadRel),
        "as attribute requires the preload or modulepreload relationship"
    );

    use HtmlAreaCoordinatesAuthorError as Area;
    let area = [
        (
            Area::MissingRequired,
            "area shape requires a coords attribute",
        ),
        (
            Area::ForbiddenPresence,
            "default area shape forbids a coords attribute",
        ),
        (
            Area::InvalidListSyntax,
            "area coords is not a valid list of comma-separated integers",
        ),
        (
            Area::Cardinality,
            "area coords has the wrong number of values",
        ),
        (
            Area::NonintegerItem { index: 2 },
            "area coords item at index 2 is not an integer",
        ),
        (Area::NegativeRadius, "area circle radius is negative"),
        (
            Area::NonincreasingX,
            "area rectangle x coordinates must increase",
        ),
        (
            Area::NonincreasingY,
            "area rectangle y coordinates must increase",
        ),
    ];
    for (error, expected) in area {
        assert_eq!(message(&error), expected);
    }
}

#[test]
fn autocomplete_syntax_and_srcset_diagnostics_state_their_rule() {
    use HtmlAutocompleteSyntaxError as Syntax;
    assert_eq!(
        message(&Syntax::EmptyTokens),
        "autocomplete value has no tokens"
    );
    assert_eq!(
        message(&Syntax::MissingDetailField),
        "autocomplete value has no detail field"
    );
    assert_eq!(
        message(&Syntax::UnrecognizedDetailField { token_index: 2 }),
        "autocomplete token at index 2 is not a recognized detail field"
    );
    assert_eq!(
        message(&Syntax::ContactHintRequiresContactField {
            field: HtmlAutofillField::Name
        }),
        "autocomplete contact hint requires a contact field, but \"name\" is not one"
    );
    assert_eq!(
        message(&Syntax::UnexpectedToken { token_index: 3 }),
        "autocomplete token at index 3 is not expected at that position"
    );

    use HtmlSrcsetParseError as Parse;
    assert_eq!(
        message(&Parse::SkippedCommas),
        "srcset has more than one comma between image candidates"
    );
    assert_eq!(
        message(&Parse::MultipleTrailingCommas),
        "srcset image candidate URL is followed by more than one trailing comma"
    );
    assert_eq!(
        message(&Parse::FutureHeightDescriptor),
        "srcset image candidate uses the unsupported height descriptor"
    );
    assert_eq!(
        message(&Parse::RejectedCandidate),
        "srcset image candidate is rejected and ignored"
    );
    assert_eq!(
        message(&Parse::DensityConversionOverflow),
        "srcset density overflows the finite binary64 range"
    );

    use HtmlSrcsetContextViolation as Context;
    let context = [
        (
            Context::SrcOrSrcsetRequired,
            "img element requires a src or srcset attribute",
        ),
        (
            Context::SourceParentNotPicture,
            "source element with a srcset attribute requires a picture parent",
        ),
        (
            Context::SourceSrcsetRequired,
            "source element in a picture requires a srcset attribute",
        ),
        (
            Context::SizesRequired,
            "srcset with width descriptors requires a sizes attribute",
        ),
        (
            Context::WidthDescriptorsRequiredForSizes,
            "sizes attribute requires every srcset candidate to use a width descriptor",
        ),
        (
            Context::ImagesizesRequired,
            "imagesrcset with width descriptors requires an imagesizes attribute",
        ),
        (
            Context::PreloadRelRequired,
            "imagesrcset requires a preload link relationship",
        ),
        (
            Context::ImageAsStateRequired,
            "imagesrcset requires the image destination",
        ),
    ];
    for (error, expected) in context {
        assert_eq!(message(&error), expected);
    }
}

#[test]
fn input_refusals_state_what_was_refused() {
    let range = HtmlInputSelection::selection(5, 2, HtmlInputSelectionDirection::None)
        .expect_err("start after end");
    assert_eq!(
        message(&range),
        "input selection start 5 is after its end 2"
    );
    let filename = input_filename_setter(&[0x61]).expect_err("only the empty value is settable");
    assert_eq!(
        message(&filename),
        "a file input's value can only be set to the empty string"
    );
}
