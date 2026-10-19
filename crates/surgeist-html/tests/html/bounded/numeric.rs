#![forbid(unsafe_code)]

use crate::html::support::non_negative_integer as integer;
use std::{cmp::Ordering, num::NonZeroU32};
use surgeist_html::{DocumentString, HtmlInteger, HtmlNonNegativeRational, bounded::*};

#[test]
fn recovery_and_unit_steps_preserve_arbitrary_digits_and_report_capacity() {
    let result = try_parse_integer(&DocumentString::from(" \t-000999tail"), 3, 3).unwrap();
    assert_eq!(result.value().magnitude().decimal_digits(), "999");
    assert!(result.value().is_negative());
    assert!(result.capacity_bytes() >= 3);
    let (value, capacity) = result.into_parts();
    assert!(capacity >= 3);
    let next = value
        .try_unit_step(IntegerUnitStep::Decrement, 4, 4)
        .unwrap();
    assert_eq!(next.value().magnitude().decimal_digits(), "1000");
    assert!(next.value().is_negative());
    assert_eq!(
        value
            .try_unit_step(IntegerUnitStep::Decrement, 3, 4)
            .unwrap_err(),
        IntegerUnitStepError::DigitLimitExceeded {
            limit: 3,
            required: 4
        }
    );
    assert_eq!(
        try_parse_integer(&DocumentString::from("12"), 2, 1).unwrap_err(),
        IntegerRecoveryError::Allocation(IntegerUnitStepError::RequestedCapacityLimitExceeded {
            limit: 1,
            required: 2,
        })
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
        Err(NumericParseError::Negative)
    );
}

#[test]
fn materialization_failure_retains_prior_numerator_allocation() {
    let rational = HtmlNonNegativeRational::new(integer("12345"), integer("67")).unwrap();
    let failure = rational.try_copy(5, 5).unwrap_err();
    assert_eq!(failure.capacity_bytes(), 5);
    assert_eq!(
        failure.kind(),
        NumericMaterializationError::Boundary(
            IntegerUnitStepError::RequestedCapacityLimitExceeded {
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
        FloatingPointParseMode::Author,
        5,
    )
    .unwrap();
    assert_eq!(
        result.value(),
        &Err(NumericParseError::FloatingPointOverflow)
    );
    assert!(result.capacity_bytes() >= 5);
    let syntax = try_floating_point_number(
        &DocumentString::from("x"),
        FloatingPointParseMode::Author,
        0,
    )
    .unwrap();
    assert_eq!(
        syntax.into_parts(),
        (Err(NumericParseError::InvalidSyntax), 0)
    );
    let recovered = try_floating_point_number(
        &DocumentString::from(" +.5tail"),
        FloatingPointParseMode::Recovery,
        3,
    )
    .unwrap();
    assert_eq!(recovered.value().as_ref().unwrap().value(), 0.5);
    assert_eq!(
        try_floating_point_number(
            &DocumentString::from("12"),
            FloatingPointParseMode::Author,
            1
        )
        .unwrap_err(),
        FloatingPointAllocationError::RequestedCapacityLimitExceeded {
            limit: 1,
            required: 2
        }
    );
}

#[test]
fn scanner_readback_is_checked_against_separately_supplied_units() {
    let source: Vec<_> = " -000tail".encode_utf16().collect();
    let mut scanner = IntegerPrefixScanner::new(true);
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
        Err(NumericParseError::InvalidSyntax)
    );
    assert_eq!(
        prefix.try_nonnegative_digits(&[0, 0, 48, 120, 48]),
        Err(NumericParseError::InvalidSyntax)
    );
}

#[test]
fn exclusive_scratch_products_lend_checked_operands_without_revalidation() {
    let a = integer("9999999999");
    let left = NaturalDigits::integer(&a);
    let mut scratch = Vec::with_capacity(NaturalProduct::required_capacity(left, left).unwrap());
    let capacity = scratch.capacity();
    let mut product = NaturalProduct::new(left, left, &mut scratch).unwrap();
    let mut steps = 0;
    while !product.step() {
        steps += 1;
        assert!(steps < 1000);
    }
    let squared = NaturalProduct::take(&mut Some(product)).unwrap();
    let expected = integer("99999999980000000001");
    let mut compare = NaturalComparison::new(squared, NaturalDigits::integer(&expected));
    while compare.step().is_none() {}
    assert_eq!(compare.result(), Some(Ordering::Equal));
    let mut difference_scratch = Vec::with_capacity(compare.difference_capacity());
    let mut difference =
        NaturalComparison::take_difference(&mut Some(compare), &mut difference_scratch).unwrap();
    while !difference.step() {}
    let zero = NaturalDifference::take(&mut Some(difference)).unwrap();
    let mut compare = NaturalComparison::new(zero, NaturalDigits::try_little(&[0]).unwrap());
    while compare.step().is_none() {}
    assert_eq!(compare.result(), Some(Ordering::Equal));
    assert_eq!(scratch.capacity(), capacity);
}

#[test]
fn external_digits_and_unreserved_scratch_cannot_enter_arithmetic() {
    assert!(matches!(
        NaturalDigits::try_little(&[]),
        Err(NaturalArithmeticError::NonCanonical)
    ));
    assert!(matches!(
        NaturalDigits::try_little(&[1, 0]),
        Err(NaturalArithmeticError::NonCanonical)
    ));
    assert!(matches!(
        NaturalDigits::try_little(&[10]),
        Err(NaturalArithmeticError::InvalidDigit {
            index: 0,
            value: 10
        })
    ));
    assert_eq!(NaturalDigits::validation_work_bound(2), Some(10));
    let digits = NaturalDigits::try_little(&[9]).unwrap();
    let mut out = Vec::new();
    assert!(matches!(
        NaturalProduct::new(digits, digits, &mut out),
        Err(NaturalArithmeticError::ScratchCapacity {
            required: 2,
            capacity: 0
        })
    ));
    assert!(out.is_empty());
    let comparison = NaturalComparison::new(digits, digits);
    assert!(matches!(
        NaturalComparison::take_difference(&mut Some(comparison), &mut out),
        Err(NaturalArithmeticError::NotReady)
    ));
    assert!(out.is_empty());
}

#[test]
fn completed_comparison_orders_absolute_difference_without_an_unchecked_claim() {
    let one = integer("1");
    let thousand = integer("1000");
    for (left, right) in [(&one, &thousand), (&thousand, &one)] {
        let mut comparison =
            NaturalComparison::new(NaturalDigits::integer(left), NaturalDigits::integer(right));
        while comparison.step().is_none() {}
        let mut scratch = Vec::with_capacity(comparison.difference_capacity());
        let mut difference =
            NaturalComparison::take_difference(&mut Some(comparison), &mut scratch).unwrap();
        while !difference.step() {}
        let result = NaturalDifference::take(&mut Some(difference)).unwrap();
        let expected = integer("999");
        let mut check = NaturalComparison::new(result, NaturalDigits::integer(&expected));
        while check.step().is_none() {}
        assert_eq!(check.result(), Some(Ordering::Equal));
    }
    let digits = NaturalDigits::integer(&one);
    let mut occupied = vec![7];
    assert!(matches!(
        NaturalProduct::new(digits, digits, &mut occupied),
        Err(NaturalArithmeticError::ScratchNotEmpty)
    ));
    assert_eq!(occupied, [7]);
}

#[test]
fn refused_takes_preserve_actual_product_and_difference_progress_for_retry() {
    let left = integer("1000");
    let right = integer("1");
    let a = NaturalDigits::integer(&left);
    let b = NaturalDigits::integer(&right);
    let mut product_scratch = Vec::with_capacity(NaturalProduct::required_capacity(a, b).unwrap());
    let mut product = Some(NaturalProduct::new(a, b, &mut product_scratch).unwrap());
    assert!(!product.as_mut().unwrap().step());
    let before = product.as_ref().unwrap().scratch().to_vec();
    assert!(matches!(
        NaturalProduct::take(&mut product),
        Err(NaturalArithmeticError::NotReady)
    ));
    assert_eq!(product.as_ref().unwrap().scratch(), before);
    while !product.as_mut().unwrap().step() {}
    let result = NaturalProduct::take(&mut product).unwrap();
    assert!(product.is_none());
    assert!(matches!(
        NaturalProduct::take(&mut product),
        Err(NaturalArithmeticError::EmptySlot)
    ));
    let mut check = NaturalComparison::new(result, a);
    while check.step().is_none() {}
    assert_eq!(check.result(), Some(Ordering::Equal));

    let mut comparison = Some(NaturalComparison::new(a, b));
    let mut scratch = Vec::new();
    assert!(matches!(
        NaturalComparison::take_difference(&mut comparison, &mut scratch),
        Err(NaturalArithmeticError::NotReady)
    ));
    assert_eq!(comparison.as_ref().unwrap().result(), None);
    while comparison.as_mut().unwrap().step().is_none() {}
    let ordering = comparison.as_ref().unwrap().result();
    assert!(matches!(
        NaturalComparison::take_difference(&mut comparison, &mut scratch),
        Err(NaturalArithmeticError::ScratchCapacity { .. })
    ));
    assert_eq!(comparison.as_ref().unwrap().result(), ordering);
    scratch.reserve_exact(comparison.as_ref().unwrap().difference_capacity());
    scratch.push(7);
    assert!(matches!(
        NaturalComparison::take_difference(&mut comparison, &mut scratch),
        Err(NaturalArithmeticError::ScratchNotEmpty)
    ));
    assert_eq!(comparison.as_ref().unwrap().result(), ordering);
    assert_eq!(scratch, [7]);
    scratch.clear();
    let mut difference =
        Some(NaturalComparison::take_difference(&mut comparison, &mut scratch).unwrap());
    assert!(comparison.is_none());
    assert!(!difference.as_mut().unwrap().step());
    let before = difference.as_ref().unwrap().scratch().to_vec();
    assert!(matches!(
        NaturalDifference::take(&mut difference),
        Err(NaturalArithmeticError::NotReady)
    ));
    assert_eq!(difference.as_ref().unwrap().scratch(), before);
    while !difference.as_mut().unwrap().step() {}
    let result = NaturalDifference::take(&mut difference).unwrap();
    assert!(difference.is_none());
    assert!(matches!(
        NaturalDifference::take(&mut difference),
        Err(NaturalArithmeticError::EmptySlot)
    ));
    let expected = integer("999");
    let mut check = NaturalComparison::new(result, NaturalDigits::integer(&expected));
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
        NumericMaterializationError::Boundary(
            IntegerUnitStepError::RequestedCapacityLimitExceeded {
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

fn service(
    request: RangeNumericRequest,
    source: &[u16],
    buffers: &mut [Vec<u8>; 8],
    output: &mut Vec<u16>,
) -> RangeNumericReply {
    match request {
        RangeNumericRequest::SourceUnit { index } => RangeNumericReply::Unit(source[index]),
        RangeNumericRequest::ReadCells { cells } => RangeNumericReply::Cells(
            cells.map(|cell| cell.map(|c| buffers[c.role().index()][c.index()])),
        ),
        RangeNumericRequest::Reserve { role, required_len } => {
            let buffer = &mut buffers[role.index()];
            buffer.reserve_exact(required_len - buffer.len());
            RangeNumericReply::Reserved {
                actual_capacity: buffer.capacity(),
                actual_len: buffer.len(),
            }
        }
        RangeNumericRequest::AppendCell { role, value } => {
            let buffer = &mut buffers[role.index()];
            assert!(buffer.len() < buffer.capacity());
            buffer.push(value);
            RangeNumericReply::Ack
        }
        RangeNumericRequest::WriteCell { cell, value } => {
            buffers[cell.role().index()][cell.index()] = value;
            RangeNumericReply::Ack
        }
        RangeNumericRequest::PopCell { role } => {
            buffers[role.index()].pop().unwrap();
            RangeNumericReply::Ack
        }
        RangeNumericRequest::ScalarStep => RangeNumericReply::Ack,
        RangeNumericRequest::ReserveOutput { required_len } => {
            output.reserve_exact(required_len - output.len());
            RangeNumericReply::OutputReserved {
                actual_capacity: output.capacity(),
                actual_len: output.len(),
            }
        }
        RangeNumericRequest::EmitUnit { unit } => {
            assert!(output.len() < output.capacity());
            output.push(unit);
            RangeNumericReply::Ack
        }
    }
}

#[test]
fn range_ingress_and_shortest_output_use_only_caller_scratch() {
    let source: Vec<_> = "12.5".encode_utf16().collect();
    let mut grammar = FloatingPointGrammarCursor::new(&source, FloatingPointParseMode::Author);
    while !grammar.is_finished() {
        grammar.advance();
    }
    let mut ingress = FiniteDecimalIngressCursor::new(
        grammar.range_decimal_parts().unwrap(),
        source.len(),
        RangeNumericHeaders::default(),
    )
    .unwrap();
    let pending = ingress.request();
    assert_eq!(
        ingress.reply(RangeNumericReply::Unit(0)),
        Err(RangeNumericError::Integrity)
    );
    assert_eq!(ingress.request(), pending);
    let mut buffers = std::array::from_fn(|_| Vec::new());
    let mut output = Vec::new();
    let mut steps = 0;
    let mut refused_scratch_capacity = false;
    while let Some(request) = ingress.request() {
        if let RangeNumericRequest::Reserve { role, .. } = request
            && !refused_scratch_capacity
        {
            let headers = ingress.headers();
            assert_eq!(
                ingress.reply(RangeNumericReply::Reserved {
                    actual_capacity: usize::MAX,
                    actual_len: headers.buffer(role).len(),
                }),
                Err(RangeNumericError::Integrity)
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
    let headers = RangeNumericHeaders::new(
        std::array::from_fn(|i| {
            RangeBufferHeader::new(buffers[i].len(), buffers[i].capacity()).unwrap()
        }),
        RangeBufferHeader::default(),
    );
    let mut best = BestFiniteNumberCursor::new(12.5, headers).unwrap();
    let mut refused_output_capacity = false;
    while let Some(request) = best.request() {
        if matches!(request, RangeNumericRequest::ReserveOutput { .. }) && !refused_output_capacity
        {
            let headers = best.headers();
            assert_eq!(
                best.reply(RangeNumericReply::OutputReserved {
                    actual_capacity: usize::MAX,
                    actual_len: headers.output().len(),
                }),
                Err(RangeNumericError::Integrity)
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
    let layout = FiniteDecimalIngressCursor::layout();
    assert_eq!(
        layout.register_bytes(),
        size_of::<FiniteDecimalIngressCursor>()
    );
    assert_eq!(layout.request_bytes(), size_of::<RangeNumericRequest>());
    assert_eq!(layout.reply_bytes(), size_of::<RangeNumericReply>());
    assert_eq!(
        (layout.scratch_cell_bytes(), layout.output_unit_bytes()),
        (1, 2)
    );
}

#[test]
fn range_midpoint_and_lattice_preserve_exact_rounding_and_reply_atomicity() {
    let mut midpoint = RangeMidpointCursor::new(0.0_f64.to_bits(), 10.0_f64.to_bits()).unwrap();
    let request = midpoint.request();
    assert_eq!(
        midpoint.reply(RangeNumericReply::Unit(0)),
        Err(RangeNumericError::Integrity)
    );
    assert_eq!(midpoint.request(), request);
    for _ in 0..3000 {
        if midpoint.request().is_none() {
            break;
        }
        midpoint.reply(RangeNumericReply::Ack).unwrap();
    }
    assert_eq!(midpoint.result(), Some(5.0_f64.to_bits()));
    let mut lattice = RangeLatticeCursor::new(
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
        lattice.reply(RangeNumericReply::Ack).unwrap();
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
