//! The report of a `closedby` attribute that borrows a block is charged by its length.
use super::*;
use surgeist_html::values::TextBlock;

/// The result of `close_watcher_enabled` for a dialog whose `closedby` attribute is `value`, in an
/// operation that retains at most `margin` units beyond the store, and the number of blocks listed.
fn enabled_within(
    value: &DocumentString,
    margin: usize,
    unlist: Option<&TextBlock>,
) -> (Result<bool, DomError>, usize) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            128,
            100_000,
            256,
            128,
            128,
            32,
            4_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            crate::DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000)
                .unwrap(),
        ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let dialog = batch.create_element_ns(
        &document,
        surgeist_html::values::Namespace::html(),
        "dialog",
    );
    batch.set_attribute(&dialog, "closedby", value.clone());
    batch.insert_before(&dialog, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let subject = commit.created(&dialog).unwrap().unwrap();
    let mut source = dom
        .register_hosted_source(
            crate::DomHostedLimits::new(
                crate::DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                8_000_000,
                8_000_000,
                1_000_000,
            ),
            &cancel,
        )
        .unwrap();
    let global = source
        .issue_global_for_canonical_source_fixture(&cancel)
        .unwrap();
    let units = dom.custody_registry.ledger().total().units() + margin;
    let tight = DomSourceLimits::new(64_000_000, units, 100_000_000, 64, 4096, 4096, 4);
    let lease = dom
        .begin_source_edit(
            DomSourceEdit::request_close_dialog(subject, None),
            tight,
            &cancel,
        )
        .unwrap();
    let id = lease.operation();
    let mut operation = lease.dom.source_operation.take().unwrap();
    let key = operation.draft.resolve(subject.into()).unwrap();
    // The request that asks whether the watcher is enabled: the original requestClose call is at
    // its second read, after the active read answered true.
    let mut cursor = HtmlCloseWatcherRequestCursor::new(false);
    cursor.reply(R::Active(true)).unwrap();
    assert_eq!(cursor.action(), A::ReadEnabled);
    let mut caller = surgeist_html::elements::dialog::HtmlDialogCursor::new(
        surgeist_html::elements::dialog::HtmlDialogOperation::RequestClose,
    );
    for reply in [
        HtmlDialogReply::Open(true),
        HtmlDialogReply::Connected(true),
        HtmlDialogReply::FullyActive(true),
    ] {
        caller.reply(reply).unwrap();
    }
    let call = Call {
        outer: crate::store::dialog::MethodFrame {
            subject: key,
            cursor: caller,
            state: crate::store::dialog::MethodState::Ready,
            invocation: Invocation::Unissued,
            locals: crate::store::dialog::MethodLocals::RequestClose { watcher: None },
            focusing: None,
        },
        cursor,
        watcher: crate::DomCloseWatcherId {
            operation: id,
            command: 0,
            occurrence: 0,
            target: subject,
            global,
            window_document: document,
        },
        invocation: DomSourceCloseWatcherInvocationId {
            operation: id,
            method: 0,
        },
        occurrence: Some(0),
        window: document,
        nested: false,
        settled: false,
        result: None,
    };
    let request = call.request(None).unwrap();
    operation.request_close = Some(call);
    lease.dom.source_operation = Some(operation);
    if let Some(block) = unlist {
        lease.dom.custody_registry.unlist_block(block);
        assert_eq!(lease.dom.custody_registry.listed_blocks(), 0);
    }
    let outcome = lease.close_watcher_enabled(&request, &source, &cancel);
    (outcome, lease.dom.custody_registry.listed_blocks())
}

#[test]
fn the_closedby_report_of_a_dialog_that_borrows_a_block_is_charged_by_length() {
    let mut units = Vec::with_capacity(8192);
    units.extend([97_u16, 98, 99, 100, 101, 102, 103, 104]);
    let block = TextBlock::from_utf16(units);
    assert!(block.capacity() >= 8192);
    // The operation holds the units of the store and at most 4000 more: the report of the
    // attribute fits when it is priced by its four units, and cannot when it is priced by the
    // capacity of its block, which the transaction lists once.
    let (outcome, listed) = enabled_within(&block.string(0..4).unwrap(), 4000, None);
    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(listed, 1);
}

#[test]
fn the_closedby_report_lists_the_block_it_borrows_for_the_operation() {
    let mut units = Vec::with_capacity(8192);
    units.extend([97_u16, 98, 99, 100, 101, 102, 103, 104]);
    let block = TextBlock::from_utf16(units);
    // No path has listed the block when the report is retained, as when the attribute bank had
    // not: the report lists it itself, so the operation holds the block once.
    let (outcome, listed) = enabled_within(&block.string(0..4).unwrap(), 100_000, Some(&block));
    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(listed, 1);
}
