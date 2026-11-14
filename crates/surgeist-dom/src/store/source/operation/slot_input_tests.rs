use super::*;
use std::{cell::Cell, rc::Rc};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn limits(bytes: usize) -> DomSourceLimits {
    DomSourceLimits::new(bytes, 1_000_000, 1_000_000, 16, 64, 64, 1)
}

struct Fixture {
    dom: Dom,
    source: crate::DomHostedSource,
    slot: NodeId,
    incoming: [NodeId; 3],
}

fn fixture() -> Fixture {
    let mut dom = Dom::new(
        DomLimits::new(
            128,
            65536,
            512,
            512,
            512,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let slot = batch.create_element_ns(&document, surgeist_html::values::Namespace::html(), "slot");
    let text = batch.create_text(&document, "text");
    let element =
        batch.create_element_ns(&document, surgeist_html::values::Namespace::html(), "span");
    let done = dom.apply(batch, &cancel()).unwrap();
    let slot = done.created(&slot).unwrap().unwrap();
    let text = done.created(&text).unwrap().unwrap();
    let element = done.created(&element).unwrap().unwrap();
    let source = dom
        .register_hosted_source(
            crate::DomHostedLimits::new(
                crate::DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                16_000_000,
                16_000_000,
                4_000_000,
            ),
            &cancel(),
        )
        .unwrap();
    Fixture {
        dom,
        source,
        slot,
        incoming: [text, element, text],
    }
}

struct LocalBacking(Rc<Cell<bool>>);

impl Drop for LocalBacking {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

fn admission(prepared: bool, bytes: usize) -> Result<(usize, u64), DomError> {
    let mut fixture = fixture();
    let baseline = fixture.dom.source_owner_custody_bytes().unwrap();
    let bank = fixture.incoming.len() * size_of::<crate::NodeOperand>();
    let local_live = Rc::new(Cell::new(true));
    let outcome = {
        let result = if prepared {
            let input = DomSourceEdit::assign_slot_with(
                fixture.slot,
                &fixture.incoming,
                || Ok::<_, DomError>(()),
                |bytes| {
                    Ok((
                        fixture.source.retain_runtime_payload(
                            &mut fixture.dom,
                            bytes,
                            &cancel(),
                        )?,
                        LocalBacking(local_live.clone()),
                    ))
                },
            )?;
            assert!(local_live.get());
            assert_eq!(fixture.source.observe().runtime_bytes(), bank);
            fixture
                .dom
                .begin_source_edit(input, limits(bytes), &cancel())
        } else {
            fixture.dom.begin_source_edit(
                DomSourceEdit::assign_slot(fixture.slot, fixture.incoming.to_vec()),
                limits(bytes),
                &cancel(),
            )
        };
        match result {
            Ok(lease) => {
                let op = lease.dom.source_operation.as_ref().unwrap();
                // Same actual bank and same original Source owner obligations.
                assert_eq!(op.command_storage.bytes, bank);
                assert_eq!(op.custody_bytes, baseline);
                Ok((op.input_bytes()?, lease.status().spent_work()))
            }
            Err(error) => Err(error),
        }
    };
    assert!(fixture.dom.pending_source_edit().is_none());
    assert_eq!(fixture.source.observe().runtime_bytes(), 0);
    if prepared {
        assert!(!local_live.get());
    }
    outcome
}

#[test]
fn prepared_slot_adoption_preserves_original_source_extent_and_work() {
    // Moving an existing bank into Source cannot create another physical bank
    // or omit the ordinary input validation/storage scans and their work.
    assert_eq!(
        admission(true, 16_000_000).unwrap(),
        admission(false, 16_000_000).unwrap(),
    );
}

#[test]
fn prepared_slot_adoption_has_the_same_exact_and_one_short_native_limit() {
    // Use the ordinary original-owner entry as the independent reference for
    // all Source setup/capture overlap, rather than repeating its size formula.
    let mut low = 0;
    let mut high = 16_000_000;
    assert!(admission(false, high).is_ok());
    while low < high {
        let middle = low + (high - low) / 2;
        match admission(false, middle) {
            Ok(_) => high = middle,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            }) => low = middle + 1,
            other => panic!("unexpected Source admission outcome: {other:?}"),
        }
    }
    assert!(admission(true, low).is_ok());
    assert!(matches!(
        admission(true, low - 1),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit,
        }) if limit == low - 1
    ));
}
