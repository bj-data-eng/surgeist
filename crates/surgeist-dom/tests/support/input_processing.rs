// Test-only envelope for nonrecursive native literal Color operations.
// Work is cumulative across the actual bounded UPDATE count; sequential
// operations share the same simultaneous scratch phase maximum.
use std::mem::{align_of, size_of};
use surgeist_color::{
    ActionLayout, DigitSlot, OperationOwner, Serialization, SerializationReceipt,
};
use surgeist_css::{
    CssColorActionDescription as Action, CssColorActionLayout, CssColorFacts, CssColorOperation,
    CssColorParseOutcome, CssColorSlot, CssColorTransformation, CssUsedColorCheckpoint,
};
use surgeist_dom::{DocumentString, DomError};
use surgeist_html::HtmlInputColorSerialization;

fn extent(count: usize, width: usize) -> Result<usize, &'static str> {
    count
        .checked_mul(width)
        .ok_or("test envelope extent overflow")
}
fn sum(values: &[usize]) -> Result<usize, &'static str> {
    values.iter().try_fold(0usize, |sum, value| {
        sum.checked_add(*value).ok_or("test envelope sum overflow")
    })
}
// Exactly six private ledger fields: scratch_limit/live/peak: usize,
// work_limit/work: u64, stopped: Option<DomError>. Each contributes its actual
// size plus at most one maximum-alignment padding; no product API is exposed.
pub fn ledger_frame_bytes() -> usize {
    let alignment = [
        align_of::<usize>(),
        align_of::<u64>(),
        align_of::<Option<DomError>>(),
    ]
    .into_iter()
    .max()
    .unwrap();
    sum(&[
        3 * size_of::<usize>(),
        2 * size_of::<u64>(),
        size_of::<Option<DomError>>(),
        6 * alignment,
    ])
    .unwrap()
}

pub fn color_limits(updates: u64) -> (usize, u64) {
    // A separate predeclared envelope for these <=64-unit, <=4-component
    // literal cases, not a product allowance. CSS bounded_decimal's exact
    // binary64 midpoint search has <=64 boundaries, <=1075 powers and
    // <=1100 digits (numeric/bounded_decimal.rs module proof and test).
    // Each digit requires a read/write plus finite carry/phase transitions;
    // literal NumberAction maps these child requests directly. The 16
    // handoffs conservatively cover lexical/grammar/literal/evaluation
    // wrappers and four finite Color formatting paths, whose <=309 digits
    // and <=971 powers are smaller than the same boundary envelope.
    let boundary = 1075u64 * (2 * 1100 + 4) + 4 * 1100 + 2 * 17 + 8;
    let actions = 16u64
        .checked_mul(4 * 64 * boundary + 4 * boundary + 64 * 64)
        .unwrap();
    let register = [
        CssColorOperation::register_bytes(),
        CssColorTransformation::register_bytes(),
        Serialization::register_bytes(),
    ]
    .into_iter()
    .max()
    .unwrap();
    let guard = [
        CssColorOperation::action_bytes(),
        CssColorTransformation::action_bytes(),
        Serialization::action_bytes(),
    ]
    .into_iter()
    .max()
    .unwrap();
    let growth_header = [
        size_of::<Vec<CssColorSlot<'_>>>() + size_of::<std::vec::Drain<'_, CssColorSlot<'_>>>(),
        size_of::<Vec<DigitSlot<'_>>>() + size_of::<std::vec::Drain<'_, DigitSlot<'_>>>(),
        size_of::<Vec<u16>>() + size_of::<std::vec::Drain<'_, u16>>(),
    ]
    .into_iter()
    .max()
    .unwrap();
    // Each of four literal number conversions can retain its own <=1100
    // digit interval (literal.rs propagates scratch_end to its parent).
    // Sixteen records per source unit bound lexical/header and the finite
    // grammar/syntax/evaluation handoffs for these nonrecursive literals.
    // All three CSS banks coexist; one growing bank additionally retains
    // its old allocation while the replacement is prepared.
    let records = 4usize * 1100 + 16 * 64;
    let parse_capacity = extent(4 * records, CssColorSlot::record_bytes()).unwrap();
    // CSS banks are destroyed before numerical serialization. Four finite
    // binary64 components each require <=309 coefficient digits. One Vec
    // grows at a time, so only the larger replacement must be added.
    let digits = extent(4 * 309, size_of::<DigitSlot<'_>>()).unwrap();
    let output = extent(4 * (309 + 3) + 64, size_of::<u16>()).unwrap();
    let serialize_capacity = sum(&[digits, output, digits.max(output)]).unwrap();
    let initialization = CssColorOperation::initialization_layout();
    let base = sum(&[
        ledger_frame_bytes(),
        initialization.owner_bytes,
        initialization.fact_identity_bytes,
        OperationOwner::layout_bytes(),
        size_of::<CssColorFacts<'_>>(),
        size_of::<Option<CssUsedColorCheckpoint<'_, '_>>>(),
        size_of::<HtmlInputColorSerialization>(),
        size_of::<DocumentString>(),
    ])
    .unwrap();
    let action = sum(&[size_of::<Action>(), size_of::<CssColorActionLayout>()]).unwrap();
    let parse = sum(&[
        size_of::<Option<CssColorOperation<'_, '_>>>(),
        size_of::<[Vec<CssColorSlot<'_>>; 3]>(),
        size_of::<CssColorParseOutcome<'_, '_>>(),
        CssColorOperation::action_bytes(),
        action,
        growth_header,
        parse_capacity,
    ])
    .unwrap();
    let transform = sum(&[
        size_of::<Option<CssColorTransformation<'_, '_, '_>>>(),
        CssColorTransformation::action_bytes(),
        action,
    ])
    .unwrap();
    // A Color action is its own description; it is observed with its layout.
    let numerical_action = size_of::<ActionLayout>();
    let serialize = sum(&[
        size_of::<Option<Serialization<'_>>>(),
        size_of::<Vec<DigitSlot<'_>>>(),
        size_of::<Vec<u16>>(),
        size_of::<SerializationReceipt>(),
        Serialization::action_bytes(),
        numerical_action,
        growth_header,
        serialize_capacity,
    ])
    .unwrap();
    let scratch = sum(&[base, parse.max(transform).max(serialize)]).unwrap();
    // Native fee policy, not parser weights: register read/copy, actual
    // action/layout, largest read+write record envelope, and at most one
    // replacement allocation per action. Admitted scratch S bounds that
    // allocation by S, prefix reads+writes by 2S, and moved records by S;
    // the temporary Vec/Drain headers and two growth steps are separate.
    let cost = sum(&[
        2 * register,
        guard,
        size_of::<Action>(),
        size_of::<CssColorActionLayout>(),
        1,
        2 * CssColorSlot::record_bytes(),
        2 * size_of::<DigitSlot<'_>>(),
        size_of::<CssColorFacts<'_>>(),
        2 * size_of::<u16>(),
        4 * scratch,
        growth_header,
        2,
    ])
    .unwrap();
    // Fixed phase initializations and receipts are dominated by one
    // additional complete charged action per wrapper/handoff.
    let work = actions.checked_mul(u64::try_from(cost).unwrap()).unwrap();
    (
        scratch,
        work.checked_mul(updates).expect("bounded update count"),
    )
}
