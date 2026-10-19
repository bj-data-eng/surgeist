//! Actual current HTML common-state reads, independent of authored popover text.
use super::*;
#[test]
fn authored_popover_attributes_do_not_change_the_native_hidden_common_state() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            32,
            100_000,
            64,
            0,
            0,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let dialog = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
    batch.set_attribute(&dialog, "popover", "auto");
    let div = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    let created = dom.apply(batch, &cancel).unwrap();
    let dialog = created.created(&dialog).unwrap().unwrap();
    let div = created.created(&div).unwrap().unwrap();
    let draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
    let key = draft.resolve(dialog.into()).unwrap();
    assert!(!draft.dialog_popover_showing(key, &cancel).unwrap());
    assert_eq!(
        draft
            .dialog_popover_showing(draft.resolve(div.into()).unwrap(), &cancel)
            .unwrap_err(),
        DomError::DialogInapplicable
    );
    assert_eq!(
        draft
            .dialog_popover_showing(key, &AtomicBool::new(true))
            .unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(
        dom.snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"popover".into()),
        Some(&DocumentString::from("auto"))
    );
    assert!(
        dom.snapshot()
            .node(dialog)
            .unwrap()
            .as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .popover_is_hidden()
    );
}
