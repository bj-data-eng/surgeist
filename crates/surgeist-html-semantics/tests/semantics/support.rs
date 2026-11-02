//! Fixtures shared by every module of the semantics integration binary.
//!
//! Each helper is written once here. Variants are named for the budget they
//! select so a test module states which finite capture, semantic or session
//! allowance it depends on instead of copying a constructor.

use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlHostFactsRevision, HtmlScriptingStatus, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession,
};

/// Builder with one no-quirks document and no scripting or registry input.
pub fn builder_with(limits: HtmlCaptureLimits) -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(limits, HtmlHostFactsRevision::fresh());
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}

/// Standard capture budget, including registry and definition capacity.
pub fn capture_limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(1000, 100000, 1000, 1000000, 1000, 10, 10, 16_777_216)
}

/// Document builder with the standard capture budget.
pub fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    builder_with(capture_limits())
}

/// Standard capture budget without registry or definition capacity.
pub fn builder_without_registries() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    builder_with(HtmlCaptureLimits::new(
        1000, 100000, 1000, 1000000, 1000, 0, 0, 16_777_216,
    ))
}

/// Ten times fewer relationships and UTF-16 units than the standard budget,
/// without registry or definition capacity.
pub fn compact_builder_without_registries() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    builder_with(HtmlCaptureLimits::new(
        1000, 10000, 1000, 100000, 1000, 0, 0, 16_777_216,
    ))
}

/// A tenth of the compact subject, relationship, attribute and depth
/// capacity with the compact UTF-16 capacity, without registry or
/// definition capacity.
pub fn small_builder_without_registries() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    builder_with(HtmlCaptureLimits::new(
        100, 1000, 100, 100000, 100, 0, 0, 16_777_216,
    ))
}

/// The small budget with a tenth of its UTF-16 capacity, without registry
/// or definition capacity.
pub fn small_builder_with_short_text() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    builder_with(HtmlCaptureLimits::new(
        100, 1000, 100, 10000, 100, 0, 0, 16_777_216,
    ))
}

/// Default context: no custom classifications or optional fact inputs.
pub fn context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap()
}

/// Session whose value interpreter admits `raw` raw input units.
pub fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    raw: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(raw), l, cancel).unwrap()
}

/// Session whose value interpreter admits no raw input units, for queries
/// that consume no attribute or text values.
pub fn session_without_raw_values<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    session(c, cx, l, 0, cancel)
}

/// Standard semantic budget: no integer digits.
pub fn semantic_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 100000)
}

/// Standard semantic budget with integer-digit capacity.
pub fn semantic_limits_with_integer_digits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 1000000, 1000000, 100000)
}

/// Standard semantic budget without nested output records.
pub fn semantic_limits_without_nested_output() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 0)
}

/// Standard semantic budget without scratch records.
pub fn semantic_limits_without_scratch() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 0, 0, 1000000, 100000)
}

/// Scalar query budget: work, output records and allocated bytes only.
pub fn scalar_semantic_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 0, 0, 1000000, 0)
}

/// Half the standard work and a tenth of its nested output records.
pub fn moderate_semantic_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 1000, 0, 1000000, 10000)
}

/// The moderate budget with ten times more scratch records.
pub fn moderate_semantic_limits_with_scratch() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 10000, 0, 1000000, 10000)
}

/// A twentieth of the standard work and a tenth of its nested output records.
pub fn small_semantic_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1000000, 100, 1000, 0, 1000000, 10000)
}

/// The small work budget with integer-digit capacity and fewer nested output
/// records.
pub fn small_semantic_limits_with_integer_digits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1000000, 100, 1000, 1000, 1000000, 1000)
}

/// Budget for image-dimension queries: large allocation and integer-digit
/// capacity with few nested output records.
pub fn dimension_semantic_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 10000, 10000, 10000000, 100)
}

/// Work, output records and allocated bytes.
pub const SCALAR_RESOURCES: &[HtmlSemanticResource] = &[
    HtmlSemanticResource::Work,
    HtmlSemanticResource::OutputRecords,
    HtmlSemanticResource::AllocatedBytes,
];

/// Scalar resources plus scratch records.
pub const SCALAR_AND_SCRATCH_RESOURCES: &[HtmlSemanticResource] = &[
    HtmlSemanticResource::Work,
    HtmlSemanticResource::OutputRecords,
    HtmlSemanticResource::ScratchRecords,
    HtmlSemanticResource::AllocatedBytes,
];

/// Scalar resources plus nested output records.
pub const SCALAR_AND_NESTED_RESOURCES: &[HtmlSemanticResource] = &[
    HtmlSemanticResource::Work,
    HtmlSemanticResource::OutputRecords,
    HtmlSemanticResource::AllocatedBytes,
    HtmlSemanticResource::NestedOutputRecords,
];

/// Every resource except integer digits.
pub const COLLECTION_RESOURCES: &[HtmlSemanticResource] = &[
    HtmlSemanticResource::Work,
    HtmlSemanticResource::OutputRecords,
    HtmlSemanticResource::ScratchRecords,
    HtmlSemanticResource::AllocatedBytes,
    HtmlSemanticResource::NestedOutputRecords,
];

/// Every semantic resource.
pub const ALL_RESOURCES: &[HtmlSemanticResource] = &[
    HtmlSemanticResource::Work,
    HtmlSemanticResource::OutputRecords,
    HtmlSemanticResource::ScratchRecords,
    HtmlSemanticResource::IntegerDigits,
    HtmlSemanticResource::AllocatedBytes,
    HtmlSemanticResource::NestedOutputRecords,
];

/// `base` with `resource` lowered to `cap` when `resource` is one of
/// `cappable`; every other allowance keeps its `base` value.
pub fn cap_resources(
    base: HtmlSemanticLimits,
    cappable: &[HtmlSemanticResource],
    resource: HtmlSemanticResource,
    cap: usize,
) -> HtmlSemanticLimits {
    let pick = |candidate: HtmlSemanticResource, value: usize| {
        if candidate == resource && cappable.contains(&candidate) {
            cap
        } else {
            value
        }
    };
    HtmlSemanticLimits::new(
        pick(HtmlSemanticResource::Work, base.work()),
        pick(HtmlSemanticResource::OutputRecords, base.output_records()),
        pick(HtmlSemanticResource::ScratchRecords, base.scratch_records()),
        pick(HtmlSemanticResource::IntegerDigits, base.integer_digits()),
        pick(HtmlSemanticResource::AllocatedBytes, base.allocated_bytes()),
        pick(
            HtmlSemanticResource::NestedOutputRecords,
            base.nested_output_records(),
        ),
    )
}

/// Scalar budget with one scalar resource capped.
pub fn scalar_limits(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    cap_resources(scalar_semantic_limits(), SCALAR_RESOURCES, resource, cap)
}

/// Standard budget with one collection resource capped.
pub fn collection_limits(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    cap_resources(semantic_limits(), COLLECTION_RESOURCES, resource, cap)
}

/// What a test requires of one resource's baseline usage. The expectation is
/// derived by the test from the documented cost model, never read back from the
/// run it judges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Charge {
    /// The query must not touch this resource.
    Zero,
    /// The query must charge this resource; its exact amount is private step
    /// layout and is not part of the contract the test pins.
    Charged,
    /// The query must charge exactly this amount.
    Exactly(usize),
    /// The query must charge at least this amount; the excess is the size of
    /// a private payload type, not part of the contract.
    AtLeast(usize),
}

/// Independently derived baseline usage of one batch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedUsage {
    pub work: Charge,
    pub output_records: Charge,
    pub peak_scratch_records: Charge,
    pub nested_output_records: Charge,
    pub allocated_bytes: Charge,
}

impl ExpectedUsage {
    /// Every resource is pinned to the given amount; use struct update syntax
    /// to relax a field to [`Charge::Charged`].
    pub const fn none() -> Self {
        Self {
            work: Charge::Charged,
            output_records: Charge::Zero,
            peak_scratch_records: Charge::Zero,
            nested_output_records: Charge::Zero,
            allocated_bytes: Charge::Charged,
        }
    }
}

fn check_charge(label: &str, resource: HtmlSemanticResource, expected: Charge, actual: usize) {
    match expected {
        Charge::Zero => assert_eq!(actual, 0, "{label}: {resource:?} must not be charged"),
        Charge::Charged => assert!(actual > 0, "{label}: {resource:?} must be charged"),
        Charge::Exactly(n) => assert_eq!(actual, n, "{label}: {resource:?} usage"),
        Charge::AtLeast(n) => assert!(
            actual >= n,
            "{label}: {resource:?} usage {actual} is below {n}"
        ),
    }
}

/// Bytes of one output record, as the documented tariff multiplies capacity
/// growth by `size_of` the stored record type.
pub fn result_record_bytes() -> usize {
    std::mem::size_of::<surgeist_html_semantics::HtmlSemanticResult>()
}

/// Runs `requests` once under `base` and checks the independently derived
/// usage, then proves for every resource that its exact baseline usage is the
/// smallest admitting cap: at that cap the report equals the baseline report
/// (results and usage) and one unit less fails with exactly that resource,
/// the session's qualification and limits, a sticky repeat and released
/// scratch. Resources expected to be [`Charge::Zero`] are proven by capping
/// them at zero, which must still reproduce the baseline report.
pub fn assert_exact_and_one_short_caps(
    label: &str,
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    raw: usize,
    base: HtmlSemanticLimits,
    requests: &[HtmlSemanticRequest],
    expected: ExpectedUsage,
) -> HtmlSemanticReport {
    let baseline = session(c, cx, base, raw, HtmlSemanticCancellation::new())
        .query(requests)
        .unwrap_or_else(|f| panic!("{label}: baseline failed: {f:?}"));
    let u = baseline.usage();
    use HtmlSemanticResource as R;
    check_charge(label, R::Work, expected.work, u.work());
    check_charge(
        label,
        R::OutputRecords,
        expected.output_records,
        u.output_records(),
    );
    check_charge(
        label,
        R::ScratchRecords,
        expected.peak_scratch_records,
        u.peak_scratch_records(),
    );
    check_charge(
        label,
        R::NestedOutputRecords,
        expected.nested_output_records,
        u.nested_output_records(),
    );
    check_charge(
        label,
        R::AllocatedBytes,
        expected.allocated_bytes,
        u.allocated_bytes(),
    );
    assert_eq!(u.scratch_records(), 0, "{label}: scratch released");
    for (resource, exact) in [
        (R::Work, u.work()),
        (R::OutputRecords, u.output_records()),
        (R::ScratchRecords, u.peak_scratch_records()),
        (R::NestedOutputRecords, u.nested_output_records()),
        (R::AllocatedBytes, u.allocated_bytes()),
    ] {
        let at_cap = cap_resources(base, ALL_RESOURCES, resource, exact);
        let mut ss = session(c, cx, at_cap, raw, HtmlSemanticCancellation::new());
        let r = ss
            .query(requests)
            .unwrap_or_else(|f| panic!("{label}: {resource:?} exact cap {exact}: {f:?}"));
        assert_eq!(
            r.results(),
            baseline.results(),
            "{label}: {resource:?} exact cap results"
        );
        assert_eq!(r.usage(), u, "{label}: {resource:?} exact cap usage");
        assert_eq!(r.capture(), c.identity());
        assert_eq!(r.context(), cx.identity());
        assert_eq!(r.limits(), at_cap);
        if exact == 0 {
            continue;
        }
        let short = cap_resources(base, ALL_RESOURCES, resource, exact - 1);
        let mut ss = session(c, cx, short, raw, HtmlSemanticCancellation::new());
        let f = ss.query(requests).expect_err(&format!(
            "{label}: {resource:?} cap {} must fail",
            exact - 1
        ));
        match f.kind() {
            HtmlSemanticFailureKind::LimitExceeded {
                resource: failed,
                limit,
                required,
            } => {
                assert_eq!(*failed, resource, "{label}: failing resource");
                assert_eq!(*limit, exact - 1, "{label}: {resource:?} limit");
                assert!(*required > *limit, "{label}: {resource:?} required");
            }
            other => panic!("{label}: {resource:?} one short: {other:?}"),
        }
        let q = f.qualification().expect("qualified failure");
        assert_eq!(q.capture(), c.identity());
        assert_eq!(q.context(), cx.identity());
        assert_eq!(q.limits(), short);
        assert_eq!(ss.query(&[]).unwrap_err(), f, "{label}: sticky");
        assert_eq!(ss.usage().scratch_records(), 0, "{label}: scratch released");
    }
    baseline
}
