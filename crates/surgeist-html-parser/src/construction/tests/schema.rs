//! The amounts the parser's construction adapters charge, compared with the HTML
//! accounting schema's conversions of the same shapes. The expected side is what
//! the existing adapters charge; the schema side runs through a real ledger.

use super::*;
use crate::meta_csp::{Account, Caller};
use crate::schema_support::{charged, through_ledger};
use surgeist_html::{
    accounting::HtmlChargedError,
    bounded::script::HtmlScriptTypeCursor,
    elements::input::HtmlInputTypeCursor,
    security::csp::{HtmlCspDisposalWork, HtmlCspLayout, HtmlCspStorage},
    values::DocumentString,
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
        let mut parser_cursor = HtmlScriptTypeCursor::preparation(
            explicit.as_ref().map(DocumentString::code_units),
            language.as_ref().map(DocumentString::code_units),
        );
        while parser_cursor.result().is_none() {
            parser_cursor.advance(&mut c).unwrap();
        }
        let expected = through_ledger(|charge| {
            let mut cursor = HtmlScriptTypeCursor::preparation(
                explicit.as_ref().map(DocumentString::code_units),
                language.as_ref().map(DocumentString::code_units),
            );
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
        super::super::input_values::input_type_state(&mut c, raw.code_units()).unwrap();
        let expected = through_ledger(|charge| {
            let mut cursor = HtmlInputTypeCursor::new(Some(raw.code_units()));
            while cursor.result().is_none() {
                cursor.advance(charge)?;
            }
            Ok(())
        });
        assert_eq!(charged(&c), expected, "{text:?}");
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
