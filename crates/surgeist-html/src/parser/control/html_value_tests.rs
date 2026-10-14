//! HTML local-rule cursors driven by the actual parser admission owner.

use crate::bounded::{
    HtmlCustomNameScanAction as CustomNameScanAction, validate_custom_element_name_with,
};
use crate::parser::control::{ParseControl, ParseStop, WorkCategory};
use crate::{
    DocumentString, HtmlInputRuleWork, HtmlInputType, HtmlInputTypeCursor,
    HtmlInputValueComparisonCursor,
};
use crate::{HtmlParseCancellation, HtmlParseLimits, HtmlParseResource};

fn control(work: u64, cancellation: HtmlParseCancellation) -> ParseControl {
    let limits = HtmlParseLimits::builder()
        .source_units(1000)
        .buffered_utf16_units(1000)
        .token_attributes(1000)
        .auxiliary_entries(1000)
        .managed_nodes(1000)
        .managed_edges(1000)
        .managed_attributes(1000)
        .managed_utf16_units(1000)
        .source_map_entries(1000)
        .source_map_payload_units(1000)
        .diagnostics(1000)
        .diagnostic_payload_units(1000)
        .context_effects(1000)
        .open_elements(1000)
        .tree_depth(1000)
        .managed_capacity_bytes(16000)
        .peak_transient_bytes(32000)
        .atomic_layout_bytes(16000)
        .work_credits(work)
        .finish()
        .unwrap();
    ParseControl::new(limits, cancellation)
}
fn admit(c: &mut ParseControl, work: HtmlInputRuleWork) -> Result<(), ParseStop> {
    match work {
        HtmlInputRuleWork::ReadField => c.charge(WorkCategory::RecordVisit, 1),
        HtmlInputRuleWork::ReadValueLengths => c.charge(WorkCategory::RecordVisit, 2),
        HtmlInputRuleWork::CompareAsciiUnit => c.charge(WorkCategory::InspectByte, 3),
        HtmlInputRuleWork::CompareValueUnits | HtmlInputRuleWork::InspectValueUnit => {
            c.charge(WorkCategory::InspectByte, 2)
        }
    }
}
#[test]
fn type_exact_sixteen_and_one_short_stop_before_final_pair_and_keep_original() {
    // hidden row+length2; text row+length2; four real pairs*3 =16.
    let raw = DocumentString::from("TeXt");
    for cap in [15, 16] {
        let mut c = control(cap, HtmlParseCancellation::new());
        let mut cursor = HtmlInputTypeCursor::new(Some(&raw));
        while let Some(work) = cursor.required_work() {
            if admit(&mut c, work).is_err() {
                break;
            }
            cursor.advance();
        }
        if cap == 16 {
            assert_eq!(cursor.result(), Some(HtmlInputType::Text));
            assert_eq!(c.stop(), None);
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                16
            );
        } else {
            assert_eq!(cursor.result(), None);
            assert!(matches!(
                c.stop(),
                Some(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                })
            ));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                13
            );
            assert_eq!(
                cursor.required_work(),
                Some(HtmlInputRuleWork::CompareAsciiUnit)
            );
        }
        assert_eq!(raw.code_units(), [84, 101, 88, 116]);
        assert_eq!(c.usage().actual_capacity_high_water, 0);
    }
}
#[test]
fn comparison_exact_eight_and_one_short_use_actual_controller_without_materialization() {
    let value = [0xd800, 0x41, 0xdfff];
    // Two length metadata reads + three pairs of actual UTF16 units =8.
    for cap in [7, 8] {
        let mut c = control(cap, HtmlParseCancellation::new());
        let mut cursor = HtmlInputValueComparisonCursor::new(&value, &value);
        while let Some(work) = cursor.required_work() {
            if admit(&mut c, work).is_err() {
                break;
            }
            cursor.advance();
        }
        if cap == 8 {
            assert!(cursor.result().unwrap().is_equal());
            assert_eq!(c.stop(), None);
        } else {
            assert_eq!(cursor.result(), None);
            assert!(matches!(
                c.stop(),
                Some(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                })
            ));
        }
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            if cap == 8 { 8 } else { 6 }
        );
        assert_eq!(c.usage().actual_capacity_high_water, 0);
    }
}
#[test]
fn cancellation_before_pair_retains_unfinished_cursor_and_canonical_first_stop() {
    let cancellation = HtmlParseCancellation::new();
    let mut c = control(100, cancellation.clone());
    let old = [1, 2];
    let new = [1, 3];
    let mut cursor = HtmlInputValueComparisonCursor::new(&old, &new);
    admit(&mut c, cursor.required_work().unwrap()).unwrap();
    cursor.advance();
    cancellation.cancel();
    assert_eq!(
        admit(&mut c, cursor.required_work().unwrap()),
        Err(ParseStop::Cancelled)
    );
    assert_eq!(cursor.result(), None);
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        2
    );
    assert_eq!(
        admit(&mut c, cursor.required_work().unwrap()),
        Err(ParseStop::Cancelled)
    );
    assert_eq!(c.first_stop_usage(), Some(c.usage()));
    assert_eq!(c.usage().actual_capacity_high_water, 0);
}

#[test]
fn controlled_scanner_propagates_actual_cancellation_before_next_unit() {
    use crate::parser::control::{ParseControl, ParseStop, WorkCategory};
    let cancellation = crate::HtmlParseCancellation::new();
    let mut control = ParseControl::new(
        crate::parser::tree_builder::tests::limits(100),
        cancellation.clone(),
    );
    let name = DocumentString::from("a-b");
    let mut units = 0;
    let result = validate_custom_element_name_with(&name, |action| {
        match action {
            CustomNameScanAction::Header | CustomNameScanAction::ReservedHeader => {
                control.charge(WorkCategory::RecordVisit, 1)?
            }
            CustomNameScanAction::Unit => {
                if units == 2 {
                    cancellation.cancel();
                }
                control.charge(WorkCategory::InspectByte, 2)?;
                units += 1;
            }
            CustomNameScanAction::ReservedPair => control.charge(WorkCategory::InspectByte, 4)?,
        }
        Ok::<(), ParseStop>(())
    });
    assert_eq!(result, Err(ParseStop::Cancelled));
    assert_eq!(units, 2);
    assert_eq!(control.stop(), Some(ParseStop::Cancelled));
    assert_eq!(
        control.charge(WorkCategory::RecordVisit, 1),
        Err(ParseStop::Cancelled)
    );
}
