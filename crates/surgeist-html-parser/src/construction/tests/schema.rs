//! The amounts the parser's construction adapters charge, compared with the HTML
//! accounting schema's conversions of the same shapes. The expected side is what
//! the existing adapters charge; the schema side runs through a real ledger.

use super::*;
use crate::meta_csp::{Account, Caller};
use crate::schema_support::{charged, through_schema};
use surgeist_html::{
    accounting::{
        HtmlChange, HtmlResource, HtmlWork, attribute_validation_work, csp_storage, csp_work,
        input_rule_work, script_type_work,
    },
    bounded::{
        script::HtmlScriptTypeWork,
        values::{HtmlAttributeNameIndex, HtmlAttributeStorage, HtmlNameIndexSlot},
    },
    elements::input::{HtmlInputRuleWork, HtmlInputTypeCursor},
    security::csp::{HtmlCspDisposalWork, HtmlCspLayout, HtmlCspStorage},
    values::{Attribute, DocumentString, Namespace, QualifiedName},
};

#[test]
fn script_type_work_converts_to_the_categories_the_adapter_charges() {
    for work in [
        HtmlScriptTypeWork::RecordVisit(5),
        HtmlScriptTypeWork::InspectByte(7),
    ] {
        let mut c = control();
        super::super::script_type::script_type_charge(&mut c, work).unwrap();
        assert_eq!(charged(&c), through_schema(&[script_type_work(work)]));
    }
}

#[test]
fn input_type_recognition_converts_each_cursor_step_to_the_adapter_charge() {
    for text in [
        "text",
        "TeXt",
        "checkbox",
        "datetime-local",
        "no-such-type",
        "",
    ] {
        let raw = DocumentString::from(text);
        let mut c = control();
        super::super::input_values::input_type_state(&mut c, &raw).unwrap();
        let mut steps: Vec<HtmlChange> = Vec::new();
        let mut cursor = HtmlInputTypeCursor::new(Some(&raw));
        while let Some(work) = cursor.required_work() {
            steps.push(input_rule_work(work));
            cursor.advance();
        }
        assert_eq!(charged(&c), through_schema(&steps), "{text:?}");
    }
}

#[test]
fn every_input_rule_work_converts_to_the_amount_the_adapters_charge() {
    // The type recognizer charges `RecordVisit` 1 and 2 for field and length
    // reads, three inspected bytes for an ASCII pair and two for a value unit.
    for (work, visits, inspected) in [
        (HtmlInputRuleWork::ReadField, 1, 0),
        (HtmlInputRuleWork::ReadValueLengths, 2, 0),
        (HtmlInputRuleWork::CompareAsciiUnit, 0, 3),
        (HtmlInputRuleWork::CompareValueUnits, 0, 2),
        (HtmlInputRuleWork::InspectValueUnit, 0, 2),
    ] {
        let mut expected = control();
        if visits != 0 {
            expected
                .charge(HtmlParseWorkCategory::RecordVisit, visits)
                .unwrap();
        }
        if inspected != 0 {
            expected
                .charge(HtmlParseWorkCategory::InspectByte, inspected)
                .unwrap();
        }
        assert_eq!(
            charged(&expected),
            through_schema(&[input_rule_work(work)]),
            "{work:?}"
        );
    }
}

#[test]
fn attribute_validation_layouts_convert_to_the_charges_of_the_validation_loop() {
    // A short list compares every pair. A list of at least the index threshold
    // validates through a table of names, which also copies and writes slots.
    let indexed = super::super::construction_owner::NAME_INDEX_MIN_ENTRIES + 3;
    for count in [2, 5, indexed] {
        let backing: Vec<Attribute> = (0..count)
            .map(|index| {
                Attribute::new(
                    QualifiedName::new(Namespace::from_uri("urn:a"), None, format!("a{index}")),
                    "v",
                )
            })
            .collect();
        let backing = Arc::new(backing);
        let mut c = control();
        validate_parser_attributes(backing.clone(), &mut c).unwrap();
        let storage = HtmlAttributeStorage::from_shared_attributes(backing);
        let mut validation = match HtmlAttributeNameIndex::slots_required(count) {
            Some(slots) if count >= super::super::construction_owner::NAME_INDEX_MIN_ENTRIES => {
                storage.validate_indexed(HtmlAttributeNameIndex::new(Vec::with_capacity(slots)))
            }
            _ => storage.validate(),
        };
        let mut steps: Vec<HtmlChange> = Vec::new();
        while let Some(action) = validation.next_action().unwrap() {
            steps.extend(attribute_validation_work(action.layout()));
            action.advance().unwrap();
        }
        let mut expected = through_schema(&steps);
        if let Some(slots) = HtmlAttributeNameIndex::slots_required(count)
            .filter(|_| count >= super::super::construction_owner::NAME_INDEX_MIN_ENTRIES)
        {
            // The name table's vector is reserved and released around the loop;
            // those charges are not part of any validation layout.
            let mut table = control();
            let (mut vector, mut ticket) = table.new_vector::<HtmlNameIndexSlot>().unwrap();
            table
                .reserve_vector(&mut vector, &mut ticket, slots)
                .unwrap();
            super::super::cleanup::release_vector(&mut table, vector, ticket).unwrap();
            let reserve = charged(&table);
            for (total, extra) in expected.categories.iter_mut().zip(reserve.categories) {
                *total += extra;
            }
            expected.credits += reserve.credits;
        }
        assert_eq!(charged(&c), expected, "{count} attributes");
        let amounts = through_schema(&steps);
        if count == indexed {
            assert_ne!(amounts.categories[HtmlWork::RecordWrite as usize], 0);
            assert_ne!(amounts.categories[HtmlWork::CopyByte as usize], 0);
        }
    }
}

#[test]
fn csp_actions_convert_to_the_work_layout_and_storage_the_account_admits() {
    let storage = HtmlCspStorage::try_new(64, 8, 2).unwrap();
    let closing = HtmlCspDisposalWork::try_new(5, 16).unwrap();
    let layout = HtmlCspLayout::processing(37, storage, closing, 32).unwrap();
    let mut c = control();
    let mut account = Account::default();
    account.admit(&mut c, layout, Caller::Parser).unwrap();
    assert_eq!(charged(&c), through_schema(&[csp_work(&layout).unwrap()]));
    let layout_bytes = layout.atomic_bytes() as u64;
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AtomicLayoutBytes)
            .high_water(),
        layout_bytes
    );
    for (resource, amount) in csp_storage(storage) {
        let parse = match resource {
            HtmlResource::ManagedCapacityBytes => HtmlParseResource::ManagedCapacityBytes,
            HtmlResource::PeakTransientBytes => HtmlParseResource::PeakTransientBytes,
            HtmlResource::ManagedUtf16Units => HtmlParseResource::ManagedUtf16Units,
            HtmlResource::AuxiliaryEntries => HtmlParseResource::AuxiliaryEntries,
            other => panic!("unexpected storage resource {other:?}"),
        };
        assert_eq!(c.usage().resource(parse).current(), amount, "{resource:?}");
    }
}
