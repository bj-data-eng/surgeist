#![forbid(unsafe_code)]

use crate::html::support::non_negative_integer as integer;
use std::{cmp::Ordering, num::NonZeroU32};
use surgeist_html::{
    bounded::{
        input::HtmlInputRangeNumber,
        numeric::{
            HtmlBestFiniteNumberCursor, HtmlFiniteDecimalIngressCursor,
            HtmlFloatingPointAllocationError, HtmlFloatingPointGrammarCursor,
            HtmlFloatingPointParseMode, HtmlIntegerPrefixScanner, HtmlIntegerRecoveryError,
            HtmlIntegerUnitStep, HtmlIntegerUnitStepError, HtmlNaturalArithmeticError,
            HtmlNaturalComparison, HtmlNaturalDifference, HtmlNaturalDigits, HtmlNaturalProduct,
            HtmlNumericMaterializationError, HtmlNumericParseError, HtmlRangeBufferHeader,
            HtmlRangeLatticeCursor, HtmlRangeMidpointCursor, HtmlRangeNumericError,
            HtmlRangeNumericHeaders, HtmlRangeNumericReply, HtmlRangeNumericRequest,
            parse_integer_long, parse_nonnegative_integer_capped, try_floating_point_number,
            try_parse_integer,
        },
    },
    microsyntax::numeric::{HtmlInteger, HtmlNonNegativeRational},
    values::DocumentString,
};

#[test]
fn recovery_and_unit_steps_preserve_arbitrary_digits_and_report_capacity() {
    let result = try_parse_integer(&DocumentString::from(" \t-000999tail"), 3, 3).unwrap();
    assert_eq!(result.value().magnitude().decimal_digits(), "999");
    assert!(result.value().is_negative());
    assert!(result.capacity_bytes() >= 3);
    let (value, capacity) = result.into_parts();
    assert!(capacity >= 3);
    let next = value
        .try_unit_step(HtmlIntegerUnitStep::Decrement, 4, 4)
        .unwrap();
    assert_eq!(next.value().magnitude().decimal_digits(), "1000");
    assert!(next.value().is_negative());
    assert_eq!(
        value
            .try_unit_step(HtmlIntegerUnitStep::Decrement, 3, 4)
            .unwrap_err(),
        HtmlIntegerUnitStepError::DigitLimitExceeded {
            limit: 3,
            required: 4
        }
    );
    assert_eq!(
        try_parse_integer(&DocumentString::from("12"), 2, 1).unwrap_err(),
        HtmlIntegerRecoveryError::Allocation(
            HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                limit: 1,
                required: 2,
            }
        )
    );
    assert_eq!(
        parse_integer_long(&DocumentString::from("2147483648")).unwrap(),
        None
    );
    assert_eq!(
        parse_nonnegative_integer_capped(&DocumentString::from("999999999999999999999"), 17)
            .unwrap(),
        17
    );
    assert_eq!(
        parse_nonnegative_integer_capped(&DocumentString::from("-1"), 0),
        Err(HtmlNumericParseError::Negative)
    );
}

#[test]
fn materialization_failure_retains_prior_numerator_allocation() {
    let rational = HtmlNonNegativeRational::new(integer("12345"), integer("67")).unwrap();
    let failure = rational.try_copy(5, 5).unwrap_err();
    assert_eq!(failure.capacity_bytes(), 5);
    assert_eq!(
        failure.kind(),
        HtmlNumericMaterializationError::Boundary(
            HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                limit: 5,
                required: 7
            }
        )
    );
    let copied = rational.try_copy(5, 7).unwrap();
    assert!(copied.value().same_representation(&rational));
    let (copied, capacity) = copied.into_parts();
    assert!(capacity >= 7);
    assert_eq!(copied.denominator().decimal_digits(), "67");
    let divided = integer("100000000000000000001")
        .try_quotient_remainder(NonZeroU32::new(3).unwrap(), 21, 21)
        .unwrap();
    assert_eq!(divided.value().0.decimal_digits(), "33333333333333333333");
    assert_eq!(divided.value().1, 2);
}

#[test]
fn floating_rejection_after_conversion_keeps_temporary_capacity() {
    let result = try_floating_point_number(
        &DocumentString::from("1e999"),
        HtmlFloatingPointParseMode::Author,
        5,
    )
    .unwrap();
    assert_eq!(
        result.value(),
        &Err(HtmlNumericParseError::FloatingPointOverflow)
    );
    assert!(result.capacity_bytes() >= 5);
    let syntax = try_floating_point_number(
        &DocumentString::from("x"),
        HtmlFloatingPointParseMode::Author,
        0,
    )
    .unwrap();
    assert_eq!(
        syntax.into_parts(),
        (Err(HtmlNumericParseError::InvalidSyntax), 0)
    );
    let recovered = try_floating_point_number(
        &DocumentString::from(" +.5tail"),
        HtmlFloatingPointParseMode::Recovery,
        3,
    )
    .unwrap();
    assert_eq!(recovered.value().as_ref().unwrap().value(), 0.5);
    assert_eq!(
        try_floating_point_number(
            &DocumentString::from("12"),
            HtmlFloatingPointParseMode::Author,
            1
        )
        .unwrap_err(),
        HtmlFloatingPointAllocationError::RequestedCapacityLimitExceeded {
            limit: 1,
            required: 2
        }
    );
}

#[test]
fn scanner_readback_is_checked_against_separately_supplied_units() {
    let source: Vec<_> = " -000tail".encode_utf16().collect();
    let mut scanner = HtmlIntegerPrefixScanner::new(true);
    for &unit in &source {
        if !scanner.feed(unit) {
            break;
        }
    }
    let prefix = scanner.finish().unwrap();
    assert!(prefix.is_negative());
    assert_eq!((prefix.start(), prefix.end()), (2, 5));
    assert_eq!(
        prefix.try_nonnegative_digits(&source).unwrap(),
        &[48, 48, 48]
    );
    assert_eq!(
        prefix.try_nonnegative_digits(&[]),
        Err(HtmlNumericParseError::InvalidSyntax)
    );
    assert_eq!(
        prefix.try_nonnegative_digits(&[0, 0, 48, 120, 48]),
        Err(HtmlNumericParseError::InvalidSyntax)
    );
}

#[test]
fn exclusive_scratch_products_lend_checked_operands_without_revalidation() {
    let a = integer("9999999999");
    let left = HtmlNaturalDigits::integer(&a);
    let mut scratch =
        Vec::with_capacity(HtmlNaturalProduct::required_capacity(left, left).unwrap());
    let capacity = scratch.capacity();
    let mut product = HtmlNaturalProduct::new(left, left, &mut scratch).unwrap();
    let mut steps = 0;
    while !product.step() {
        steps += 1;
        assert!(steps < 1000);
    }
    let squared = HtmlNaturalProduct::take(&mut Some(product)).unwrap();
    let expected = integer("99999999980000000001");
    let mut compare = HtmlNaturalComparison::new(squared, HtmlNaturalDigits::integer(&expected));
    while compare.step().is_none() {}
    assert_eq!(compare.result(), Some(Ordering::Equal));
    let mut difference_scratch = Vec::with_capacity(compare.difference_capacity());
    let mut difference =
        HtmlNaturalComparison::take_difference(&mut Some(compare), &mut difference_scratch)
            .unwrap();
    while !difference.step() {}
    let zero = HtmlNaturalDifference::take(&mut Some(difference)).unwrap();
    let mut compare =
        HtmlNaturalComparison::new(zero, HtmlNaturalDigits::try_little(&[0]).unwrap());
    while compare.step().is_none() {}
    assert_eq!(compare.result(), Some(Ordering::Equal));
    assert_eq!(scratch.capacity(), capacity);
}

#[test]
fn external_digits_and_unreserved_scratch_cannot_enter_arithmetic() {
    assert!(matches!(
        HtmlNaturalDigits::try_little(&[]),
        Err(HtmlNaturalArithmeticError::NonCanonical)
    ));
    assert!(matches!(
        HtmlNaturalDigits::try_little(&[1, 0]),
        Err(HtmlNaturalArithmeticError::NonCanonical)
    ));
    assert!(matches!(
        HtmlNaturalDigits::try_little(&[10]),
        Err(HtmlNaturalArithmeticError::InvalidDigit {
            index: 0,
            value: 10
        })
    ));
    assert_eq!(HtmlNaturalDigits::validation_work_bound(2), Some(10));
    let digits = HtmlNaturalDigits::try_little(&[9]).unwrap();
    let mut out = Vec::new();
    assert!(matches!(
        HtmlNaturalProduct::new(digits, digits, &mut out),
        Err(HtmlNaturalArithmeticError::ScratchCapacity {
            required: 2,
            capacity: 0
        })
    ));
    assert!(out.is_empty());
    let comparison = HtmlNaturalComparison::new(digits, digits);
    assert!(matches!(
        HtmlNaturalComparison::take_difference(&mut Some(comparison), &mut out),
        Err(HtmlNaturalArithmeticError::NotReady)
    ));
    assert!(out.is_empty());
}

#[test]
fn completed_comparison_orders_absolute_difference_without_an_unchecked_claim() {
    let one = integer("1");
    let thousand = integer("1000");
    for (left, right) in [(&one, &thousand), (&thousand, &one)] {
        let mut comparison = HtmlNaturalComparison::new(
            HtmlNaturalDigits::integer(left),
            HtmlNaturalDigits::integer(right),
        );
        while comparison.step().is_none() {}
        let mut scratch = Vec::with_capacity(comparison.difference_capacity());
        let mut difference =
            HtmlNaturalComparison::take_difference(&mut Some(comparison), &mut scratch).unwrap();
        while !difference.step() {}
        let result = HtmlNaturalDifference::take(&mut Some(difference)).unwrap();
        let expected = integer("999");
        let mut check = HtmlNaturalComparison::new(result, HtmlNaturalDigits::integer(&expected));
        while check.step().is_none() {}
        assert_eq!(check.result(), Some(Ordering::Equal));
    }
    let digits = HtmlNaturalDigits::integer(&one);
    let mut occupied = vec![7];
    assert!(matches!(
        HtmlNaturalProduct::new(digits, digits, &mut occupied),
        Err(HtmlNaturalArithmeticError::ScratchNotEmpty)
    ));
    assert_eq!(occupied, [7]);
}

#[test]
fn refused_takes_preserve_actual_product_and_difference_progress_for_retry() {
    let left = integer("1000");
    let right = integer("1");
    let a = HtmlNaturalDigits::integer(&left);
    let b = HtmlNaturalDigits::integer(&right);
    let mut product_scratch =
        Vec::with_capacity(HtmlNaturalProduct::required_capacity(a, b).unwrap());
    let mut product = Some(HtmlNaturalProduct::new(a, b, &mut product_scratch).unwrap());
    assert!(!product.as_mut().unwrap().step());
    let before = product.as_ref().unwrap().scratch().to_vec();
    assert!(matches!(
        HtmlNaturalProduct::take(&mut product),
        Err(HtmlNaturalArithmeticError::NotReady)
    ));
    assert_eq!(product.as_ref().unwrap().scratch(), before);
    while !product.as_mut().unwrap().step() {}
    let result = HtmlNaturalProduct::take(&mut product).unwrap();
    assert!(product.is_none());
    assert!(matches!(
        HtmlNaturalProduct::take(&mut product),
        Err(HtmlNaturalArithmeticError::EmptySlot)
    ));
    let mut check = HtmlNaturalComparison::new(result, a);
    while check.step().is_none() {}
    assert_eq!(check.result(), Some(Ordering::Equal));

    let mut comparison = Some(HtmlNaturalComparison::new(a, b));
    let mut scratch = Vec::new();
    assert!(matches!(
        HtmlNaturalComparison::take_difference(&mut comparison, &mut scratch),
        Err(HtmlNaturalArithmeticError::NotReady)
    ));
    assert_eq!(comparison.as_ref().unwrap().result(), None);
    while comparison.as_mut().unwrap().step().is_none() {}
    let ordering = comparison.as_ref().unwrap().result();
    assert!(matches!(
        HtmlNaturalComparison::take_difference(&mut comparison, &mut scratch),
        Err(HtmlNaturalArithmeticError::ScratchCapacity { .. })
    ));
    assert_eq!(comparison.as_ref().unwrap().result(), ordering);
    scratch.reserve_exact(comparison.as_ref().unwrap().difference_capacity());
    scratch.push(7);
    assert!(matches!(
        HtmlNaturalComparison::take_difference(&mut comparison, &mut scratch),
        Err(HtmlNaturalArithmeticError::ScratchNotEmpty)
    ));
    assert_eq!(comparison.as_ref().unwrap().result(), ordering);
    assert_eq!(scratch, [7]);
    scratch.clear();
    let mut difference =
        Some(HtmlNaturalComparison::take_difference(&mut comparison, &mut scratch).unwrap());
    assert!(comparison.is_none());
    assert!(!difference.as_mut().unwrap().step());
    let before = difference.as_ref().unwrap().scratch().to_vec();
    assert!(matches!(
        HtmlNaturalDifference::take(&mut difference),
        Err(HtmlNaturalArithmeticError::NotReady)
    ));
    assert_eq!(difference.as_ref().unwrap().scratch(), before);
    while !difference.as_mut().unwrap().step() {}
    let result = HtmlNaturalDifference::take(&mut difference).unwrap();
    assert!(difference.is_none());
    assert!(matches!(
        HtmlNaturalDifference::take(&mut difference),
        Err(HtmlNaturalArithmeticError::EmptySlot)
    ));
    let expected = integer("999");
    let mut check = HtmlNaturalComparison::new(result, HtmlNaturalDigits::integer(&expected));
    while check.step().is_none() {}
    assert_eq!(check.result(), Some(Ordering::Equal));
}

#[test]
fn rational_allocation_refusal_returns_original_numerator_backing_for_retry() {
    let numerator = integer("123456789012345678901234567890123456789");
    let original_backing = numerator.decimal_digits().as_ptr();
    let failure = HtmlNonNegativeRational::try_from_integer(numerator, 100, 0).unwrap_err();
    assert_eq!(
        failure.numerator().decimal_digits().as_ptr(),
        original_backing
    );
    assert_eq!(
        failure.numerator().decimal_digits(),
        "123456789012345678901234567890123456789"
    );
    assert_eq!(failure.capacity_bytes(), 0);
    assert_eq!(
        failure.kind(),
        HtmlNumericMaterializationError::Boundary(
            HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                limit: 0,
                required: 1
            }
        )
    );
    let (observed, numerator) = failure.into_parts();
    assert_eq!(observed.capacity_bytes(), 0);
    let result = HtmlNonNegativeRational::try_from_integer(numerator, 100, 1).unwrap();
    assert_eq!(
        result.value().numerator().decimal_digits().as_ptr(),
        original_backing
    );
    assert_eq!(result.value().denominator().decimal_digits(), "1");
    assert!(result.capacity_bytes() >= 1);
}

pub(super) fn service(
    request: HtmlRangeNumericRequest,
    source: &[u16],
    buffers: &mut [Vec<u8>; 8],
    output: &mut Vec<u16>,
) -> HtmlRangeNumericReply {
    match request {
        HtmlRangeNumericRequest::SourceUnit { index } => HtmlRangeNumericReply::Unit(source[index]),
        HtmlRangeNumericRequest::ReadCells { cells } => HtmlRangeNumericReply::Cells(
            cells.map(|cell| cell.map(|c| buffers[c.role().index()][c.index()])),
        ),
        HtmlRangeNumericRequest::Reserve { role, required_len } => {
            let buffer = &mut buffers[role.index()];
            buffer.reserve_exact(required_len - buffer.len());
            HtmlRangeNumericReply::Reserved {
                actual_capacity: buffer.capacity(),
                actual_len: buffer.len(),
            }
        }
        HtmlRangeNumericRequest::AppendCell { role, value } => {
            let buffer = &mut buffers[role.index()];
            assert!(buffer.len() < buffer.capacity());
            buffer.push(value);
            HtmlRangeNumericReply::Ack
        }
        HtmlRangeNumericRequest::WriteCell { cell, value } => {
            buffers[cell.role().index()][cell.index()] = value;
            HtmlRangeNumericReply::Ack
        }
        HtmlRangeNumericRequest::PopCell { role } => {
            buffers[role.index()].pop().unwrap();
            HtmlRangeNumericReply::Ack
        }
        HtmlRangeNumericRequest::ScalarStep => HtmlRangeNumericReply::Ack,
        HtmlRangeNumericRequest::ReserveOutput { required_len } => {
            output.reserve_exact(required_len - output.len());
            HtmlRangeNumericReply::OutputReserved {
                actual_capacity: output.capacity(),
                actual_len: output.len(),
            }
        }
        HtmlRangeNumericRequest::EmitUnit { unit } => {
            assert!(output.len() < output.capacity());
            output.push(unit);
            HtmlRangeNumericReply::Ack
        }
    }
}

#[test]
fn range_ingress_converts_every_zero_spelling_to_the_unsigned_zero() {
    // HTML877d rules for parsing floating-point number values, step 15:
    // "Let S be the set of finite IEEE 754 double-precision floating-point
    // values except −0", so a zero conversion is never negative. Rounding
    // to the nearest member keeps the sign of a nonzero negative subnormal:
    // -5e-324 is the least negative subnormal, while -2e-324 lies below half
    // of it and rounds to zero.
    for (spelling, expected) in [
        ("0", 0),
        ("-0", 0),
        ("-0.0", 0),
        ("-0e5", 0),
        ("-0.000e-3", 0),
        ("-.0", 0),
        ("-2e-324", 0),
        ("-5e-324", (-5e-324_f64).to_bits()),
    ] {
        let source: Vec<_> = spelling.encode_utf16().collect();
        let mut grammar =
            HtmlFloatingPointGrammarCursor::new(&source, HtmlFloatingPointParseMode::Author);
        while !grammar.is_finished() {
            grammar.advance();
        }
        // Each spelling is a valid floating-point number, so grammar succeeds.
        let parts = grammar.range_decimal_parts().unwrap();
        let mut ingress = HtmlFiniteDecimalIngressCursor::new(
            parts,
            source.len(),
            HtmlRangeNumericHeaders::default(),
        )
        .unwrap();
        let mut buffers = std::array::from_fn(|_| Vec::new());
        let mut output = Vec::new();
        while let Some(request) = ingress.request() {
            let reply = service(request, &source, &mut buffers, &mut output);
            ingress.reply(reply).unwrap();
        }
        let bits = ingress.result().unwrap().unwrap();
        let value = HtmlInputRangeNumber::try_from(f64::from_bits(bits))
            .unwrap()
            .value();
        assert_eq!(value.to_bits(), expected, "{spelling}");
    }
}

#[test]
fn range_ingress_and_shortest_output_use_only_caller_scratch() {
    let source: Vec<_> = "12.5".encode_utf16().collect();
    let mut grammar =
        HtmlFloatingPointGrammarCursor::new(&source, HtmlFloatingPointParseMode::Author);
    while !grammar.is_finished() {
        grammar.advance();
    }
    let mut ingress = HtmlFiniteDecimalIngressCursor::new(
        grammar.range_decimal_parts().unwrap(),
        source.len(),
        HtmlRangeNumericHeaders::default(),
    )
    .unwrap();
    let pending = ingress.request();
    assert_eq!(
        ingress.reply(HtmlRangeNumericReply::Unit(0)),
        Err(HtmlRangeNumericError::Integrity)
    );
    assert_eq!(ingress.request(), pending);
    let mut buffers = std::array::from_fn(|_| Vec::new());
    let mut output = Vec::new();
    let mut steps = 0;
    let mut refused_scratch_capacity = false;
    while let Some(request) = ingress.request() {
        if let HtmlRangeNumericRequest::Reserve { role, .. } = request
            && !refused_scratch_capacity
        {
            let headers = ingress.headers();
            assert_eq!(
                ingress.reply(HtmlRangeNumericReply::Reserved {
                    actual_capacity: usize::MAX,
                    actual_len: headers.buffer(role).len(),
                }),
                Err(HtmlRangeNumericError::Integrity)
            );
            assert_eq!(ingress.request(), Some(request));
            assert_eq!(ingress.headers(), headers);
            refused_scratch_capacity = true;
        }
        let reply = service(request, &source, &mut buffers, &mut output);
        ingress.reply(reply).unwrap();
        steps += 1;
        assert!(steps < 1_000_000);
    }
    assert_eq!(ingress.result(), Some(Ok(12.5_f64.to_bits())));
    let headers = HtmlRangeNumericHeaders::new(
        std::array::from_fn(|i| {
            HtmlRangeBufferHeader::new(buffers[i].len(), buffers[i].capacity()).unwrap()
        }),
        HtmlRangeBufferHeader::default(),
    );
    let mut best = HtmlBestFiniteNumberCursor::new(12.5, headers).unwrap();
    let mut refused_output_capacity = false;
    while let Some(request) = best.request() {
        if matches!(request, HtmlRangeNumericRequest::ReserveOutput { .. })
            && !refused_output_capacity
        {
            let headers = best.headers();
            assert_eq!(
                best.reply(HtmlRangeNumericReply::OutputReserved {
                    actual_capacity: usize::MAX,
                    actual_len: headers.output().len(),
                }),
                Err(HtmlRangeNumericError::Integrity)
            );
            assert_eq!(best.request(), Some(request));
            assert_eq!(best.headers(), headers);
            refused_output_capacity = true;
        }
        best.reply(service(request, &[], &mut buffers, &mut output))
            .unwrap();
        steps += 1;
        assert!(steps < 2_000_000);
    }
    assert_eq!(String::from_utf16(&output).unwrap(), "12.5");
    assert_eq!(best.result(), Some(4));
    assert!(refused_scratch_capacity && refused_output_capacity);
    assert_eq!(best.headers().output().len(), 4);
    let layout = HtmlFiniteDecimalIngressCursor::layout();
    assert_eq!(
        layout.register_bytes(),
        size_of::<HtmlFiniteDecimalIngressCursor>()
    );
    assert_eq!(layout.request_bytes(), size_of::<HtmlRangeNumericRequest>());
    assert_eq!(layout.reply_bytes(), size_of::<HtmlRangeNumericReply>());
    assert_eq!(
        (layout.scratch_cell_bytes(), layout.output_unit_bytes()),
        (1, 2)
    );
}

#[test]
fn range_midpoint_and_lattice_preserve_exact_rounding_and_reply_atomicity() {
    let mut midpoint = HtmlRangeMidpointCursor::new(0.0_f64.to_bits(), 10.0_f64.to_bits()).unwrap();
    let request = midpoint.request();
    assert_eq!(
        midpoint.reply(HtmlRangeNumericReply::Unit(0)),
        Err(HtmlRangeNumericError::Integrity)
    );
    assert_eq!(midpoint.request(), request);
    for _ in 0..3000 {
        if midpoint.request().is_none() {
            break;
        }
        midpoint.reply(HtmlRangeNumericReply::Ack).unwrap();
    }
    assert_eq!(midpoint.result(), Some(5.0_f64.to_bits()));
    let mut lattice = HtmlRangeLatticeCursor::new(
        3.0_f64.to_bits(),
        0.0_f64.to_bits(),
        Some(10.0_f64.to_bits()),
        0.0_f64.to_bits(),
        2.0_f64.to_bits(),
    )
    .unwrap();
    for _ in 0..100_000 {
        if lattice.request().is_none() {
            break;
        }
        lattice.reply(HtmlRangeNumericReply::Ack).unwrap();
    }
    // HTML resolves an exact step-distance tie toward positive infinity.
    assert_eq!(lattice.result(), Some(Some(4.0_f64.to_bits())));
    assert_eq!(
        HtmlInteger::try_from_usize(0, 1, 1)
            .unwrap()
            .value()
            .magnitude()
            .decimal_digits(),
        "0"
    );
}
