//! The amounts the parser's construction adapters charge, compared with the HTML
//! accounting schema's conversions of the same shapes. The expected side is what
//! the existing adapters charge; the schema side runs through a real ledger.

use super::*;
use crate::meta_csp::{Account, Caller};
use crate::schema_support::{charged, through_ledger};
use surgeist_html::{
    accounting::{HtmlChargedError, HtmlWork},
    bounded::{
        script::HtmlScriptTypeCursor,
        values::{HtmlAttributeNameIndex, HtmlAttributeStorage, HtmlNameIndexSlot},
    },
    elements::input::HtmlInputTypeCursor,
    security::csp::{HtmlCspDisposalWork, HtmlCspLayout, HtmlCspStorage},
    values::{Attribute, DocumentString, Namespace, QualifiedName},
};

#[test]
fn script_classification_charges_what_the_stock_ledger_charger_records() {
    for (explicit, language) in [
        (Some("text/javascript"), None),
        (Some(" MODULE "), None),
        (None, Some("javascript")),
        (Some("text/unknown"), Some("x")),
        (None, None),
    ] {
        let explicit = explicit.map(DocumentString::from);
        let language = language.map(DocumentString::from);
        let mut c = control();
        let mut parser_cursor =
            HtmlScriptTypeCursor::preparation(explicit.as_ref(), language.as_ref());
        while parser_cursor.result().is_none() {
            parser_cursor.advance(&mut c).unwrap();
        }
        let expected = through_ledger(|charge| {
            let mut cursor =
                HtmlScriptTypeCursor::preparation(explicit.as_ref(), language.as_ref());
            while cursor.result().is_none() {
                cursor.advance(charge)?;
            }
            Ok(())
        });
        assert_eq!(charged(&c), expected, "{explicit:?} {language:?}");
        assert_ne!(expected.credits, 0);
    }
}

#[test]
fn input_type_recognition_charges_what_the_stock_ledger_charger_records() {
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
        let expected = through_ledger(|charge| {
            let mut cursor = HtmlInputTypeCursor::new(Some(&raw));
            while cursor.result().is_none() {
                cursor.advance(charge)?;
            }
            Ok(())
        });
        assert_eq!(charged(&c), expected, "{text:?}");
    }
}

#[test]
fn attribute_validation_charges_what_the_stock_ledger_charger_records() {
    // A short list compares every pair. A list of at least the index threshold
    // validates through a table of names, which also copies and writes slots.
    let indexed = super::super::storage_holds::NAME_INDEX_MIN_ENTRIES + 3;
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
            Some(slots) if count >= super::super::storage_holds::NAME_INDEX_MIN_ENTRIES => {
                storage.validate_indexed(HtmlAttributeNameIndex::new(Vec::with_capacity(slots)))
            }
            _ => storage.validate(),
        };
        let mut steps = through_ledger(|charge| {
            while let Some(action) = validation.next_action().unwrap() {
                match action.advance(charge) {
                    Ok(()) => {}
                    Err(HtmlChargedError::Stopped(stop)) => return Err(stop),
                    Err(HtmlChargedError::Faulted(fault)) => panic!("validation fault: {fault}"),
                }
            }
            Ok(())
        });
        if let Some(slots) = HtmlAttributeNameIndex::slots_required(count)
            .filter(|_| count >= super::super::storage_holds::NAME_INDEX_MIN_ENTRIES)
        {
            // The name table's vector is reserved and released around the loop;
            // those charges are not part of any validation action.
            let mut table = control();
            let (mut vector, mut ticket) = table.new_vector::<HtmlNameIndexSlot>().unwrap();
            table
                .reserve_vector(&mut vector, &mut ticket, slots)
                .unwrap();
            super::super::cleanup::release_vector(&mut table, vector, ticket).unwrap();
            let reserve = charged(&table);
            for (total, extra) in steps.categories.iter_mut().zip(reserve.categories) {
                *total += extra;
            }
            steps.credits += reserve.credits;
        }
        if count == indexed {
            assert_ne!(steps.categories[HtmlWork::RecordWrite as usize], 0);
            assert_ne!(steps.categories[HtmlWork::CopyByte as usize], 0);
        }
        assert_eq!(charged(&c), steps, "{count} attributes");
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
    assert_eq!(
        charged(&c),
        through_ledger(|charge| match layout.charge_work(charge) {
            Ok(()) => Ok(()),
            Err(HtmlChargedError::Stopped(stop)) => Err(stop),
            Err(HtmlChargedError::Faulted(overflow)) => panic!("{overflow}"),
        })
    );
    let layout_bytes = layout.atomic_bytes() as u64;
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AtomicLayoutBytes)
            .high_water(),
        layout_bytes
    );
    for (parse, amount) in [
        (HtmlParseResource::AuxiliaryEntries, 2),
        (HtmlParseResource::ManagedUtf16Units, 8),
        (HtmlParseResource::ManagedCapacityBytes, 64),
        (HtmlParseResource::PeakTransientBytes, 64),
    ] {
        assert_eq!(c.usage().resource(parse).current(), amount, "{parse:?}");
    }
}
