use super::*;
use surgeist_dom::DomBatch;

pub(in crate::document_scripting) fn extend_guest_allowance_for_callback_conversion(
    host: &DocumentScriptHost,
    code_units: u64,
) {
    let budget = host.guest_events().unwrap().budget.clone();
    // The nullable string return visits its nullable and scalar descriptors,
    // reserves one payload buffer and copies the returned code units.
    budget.set_work_remaining(budget.work_remaining().unwrap() + 2 + 1 + code_units);
}

pub(in crate::document_scripting) fn conversion_work_precedes_guest_coercion() {
    // Compilation and evaluation each prepare one outcome record. The setter
    // prepares its callback frame, receiver, argument buffer and argument root.
    let preparation_prefix = 2 + 1 + 1 + 1 + 1;
    // Direct DOMString conversion visits one value, then prepares one outcome
    // record before ToString. This setter has no argument-presence decision.
    let required = preparation_prefix + 1 + 1;
    for (available, reaches_coercion) in [(required - 1, false), (required, true)] {
        let cancellation = Arc::new(AtomicBool::new(false));
        let (mut session, target) =
            crate::document_scripting::cross_realm_tests::hosted_session(&cancellation, 0);
        let mut batch = DomBatch::new();
        let text = batch.create_text(target.document(), "original");
        let text = session
            .dom()
            .apply(batch, &cancellation)
            .unwrap()
            .created(&text)
            .unwrap()
            .unwrap();
        let mut host = attach_public_nodes(session, target, cancellation, 8_000, 200_000);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, text, "conversionText");
        let marker = host
            .in_document(&permit, |scope| {
                let marker = scope.evaluate(&units(
                    "globalThis.conversionMarker = {};\n\
                     globalThis.conversionInput = { toString() { throw conversionMarker; } };\n\
                     conversionMarker",
                ))?;
                Ok(scope.retain(&marker))
            })
            .unwrap();
        let events = host.guest_events().unwrap();
        events.budget.set_work_remaining(available);
        let before = host.snapshot().unwrap().stamp();
        let result = host.in_document(&permit, |scope| {
            scope
                .evaluate(&units("conversionText.data = conversionInput"))
                .map(drop)
        });
        assert_eq!(host.snapshot().unwrap().stamp(), before);
        if let Err(DocumentScriptError::Script(Error::Exception(exception))) = &result {
            // Verify that any unexpected guest exception really came from
            // the reached coercion, using only existing identity imports.
            host.in_document(&permit, |scope| {
                let expected = scope.import(&marker)?;
                let actual = scope.import(exception.value())?;
                assert!(scope.strict_equals(&actual, &expected)?);
                Ok(())
            })
            .unwrap();
        }
        if reaches_coercion {
            assert!(matches!(
                result,
                Err(DocumentScriptError::Script(Error::Exception(_)))
            ));
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
            drop(result);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("semantic conversion work escaped the original guest budget: {result:?}");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::WorkLimit)
            ));
            let Err(DocumentScriptError::Script(Error::HostFailure(repeated))) =
                host.in_document(&permit, |_| Ok(()))
            else {
                panic!("the original aggregate refusal must remain terminal");
            };
            assert!(std::ptr::eq(
                failure.cause::<DocumentGuestEventError>().unwrap(),
                repeated.cause::<DocumentGuestEventError>().unwrap(),
            ));
        }
        drop(marker);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

fn profiled_host(mode: surgeist_ledger::ProfileMode) -> DocumentScriptHost {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancellation, 0);
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 2,
        nested_dispatches: 2,
        storage_bytes: 32_000,
        work: 10_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancellation).unwrap();
    enable_public_nodes(
        PreparedDocumentScriptHost::prepare(session, 2_000_000, cancellation)
            .unwrap()
            .with_guest_accounting_profile(mode),
        target,
        (8_000, public_node_source_limits(200_000, 64), 4),
        &mut Some(registry),
    )
}

pub(in crate::document_scripting) fn original_conversion_scopes_report_semantics_without_changing_profile_admission()
 {
    use surgeist_ledger::{EventFilter, ProfileMode};
    let mut baseline = None;
    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 128,
            filter: EventFilter::All,
        },
    ] {
        let mut host = profiled_host(mode);
        let permit = host.permit().unwrap();
        host.in_document(&permit, |scope| {
            scope.evaluate(&units(
                "globalThis.conversionText = document.createTextNode('original')",
            ))?;
            Ok(())
        })
        .unwrap();
        let events = host.guest_events().unwrap();
        let before = events.budget.work().unwrap();
        let before_report = host.guest_accounting().unwrap().snapshot().unwrap();
        host.in_document(&permit, |scope| {
            assert_guest(
                scope,
                r#"(() => {
                conversionText.data = 'abc';
                return CSS.escape('a b') === 'a\\ b' &&
                    new DOMException('x', 'TypeError').message === 'x';
            })()"#,
            )
        })
        .unwrap();
        let work = events.budget.work().unwrap() - before;
        if let Some(expected) = baseline {
            assert_eq!(work, expected);
        } else {
            baseline = Some(work);
        }
        let report = host.guest_accounting().unwrap().snapshot().unwrap();
        if let (Some(report), Some(before)) = (report, before_report) {
            assert_eq!(report.scopes.len(), 6);
            // Direct setter: one value and three input units. CSS: one
            // argument, one value and three input units. DOMException: two
            // arguments/values, ten incoming and ten outgoing units.
            for (name, values, arguments, units) in [
                ("js-conversion", 1, 0, 3),
                ("window-css", 1, 1, 3),
                ("dom-exception", 2, 2, 20),
            ] {
                let current = report
                    .scopes
                    .iter()
                    .find(|scope| scope.name == name)
                    .unwrap();
                let previous = before
                    .scopes
                    .iter()
                    .find(|scope| scope.name == name)
                    .unwrap();
                assert_eq!(
                    current.usage.work(DocumentGuestWork::ConversionValue)
                        - previous.usage.work(DocumentGuestWork::ConversionValue),
                    values
                );
                assert_eq!(
                    current.usage.work(DocumentGuestWork::ConversionArgument)
                        - previous.usage.work(DocumentGuestWork::ConversionArgument),
                    arguments
                );
                assert_eq!(
                    current.usage.work(DocumentGuestWork::ConversionCodeUnit)
                        - previous.usage.work(DocumentGuestWork::ConversionCodeUnit),
                    units
                );
                for dimension in [
                    DocumentGuestWork::ConversionEnumerationLookup,
                    DocumentGuestWork::ConversionDictionaryMember,
                    DocumentGuestWork::ConversionIteratorOperation,
                    DocumentGuestWork::ConversionSequenceItem,
                    DocumentGuestWork::ConversionSequenceMove,
                ] {
                    assert_eq!(
                        current.usage.work(dimension),
                        previous.usage.work(dimension)
                    );
                }
                assert!(current.phases > previous.phases);
                assert_eq!(
                    current.usage.live(DocumentGuestResource::BackingBytes),
                    previous.usage.live(DocumentGuestResource::BackingBytes)
                );
            }
        } else {
            assert_eq!(mode, ProfileMode::Off);
        }
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

struct StorageProbeHost {
    inner: preparation::NativeConversionHost,
    owner: Weak<NativeOwner>,
    budget: GuestBudget,
    limit: usize,
    short: bool,
    filler: Rc<RefCell<Option<GuestBacking>>>,
    request: Rc<Cell<Option<surgeist_bindings_js::conversion::BufferRequest>>>,
    dropped: Rc<Cell<bool>>,
}

impl Drop for StorageProbeHost {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}

impl surgeist_bindings_js::conversion::Host for StorageProbeHost {
    type CleanupError = preparation::ConversionCleanupError;
    type Buffer = preparation::ConversionBacking;
    fn charge(
        &self,
        work: surgeist_bindings_js::conversion::Work,
        units: u64,
    ) -> Result<(), Error> {
        self.inner.charge(work, units)
    }
    fn retain_buffer(
        &self,
        request: surgeist_bindings_js::conversion::BufferRequest,
    ) -> Result<Self::Buffer, Error> {
        assert_eq!(request.payload_bytes(), 6);
        assert!(self.request.replace(Some(request)).is_none());
        // Use the producer's declared physical request as the contract. The
        // actual Runtime host must admit its full payload plus receipt, once.
        let spare =
            self.limit - self.budget.used() - request.backing_bytes() + usize::from(self.short);
        let filler = self
            .budget
            .reserve(AccountScope::Adapter, &self.owner.upgrade().unwrap(), spare)
            .expect("the control filler must fit before the conversion request");
        *self.filler.borrow_mut() = Some(filler);
        let result = self.inner.retain_buffer(request);
        if !self.short {
            assert_eq!(self.budget.used(), self.limit);
        }
        result
    }
    fn begin_phase(&self) -> Result<Option<surgeist_ledger::PhaseObservation>, Error> {
        self.inner.begin_phase()
    }
    fn end_phase(
        &self,
        marker: surgeist_ledger::PhaseObservation,
    ) -> Result<(), Self::CleanupError> {
        self.inner.end_phase(marker)
    }
}

pub(in crate::document_scripting) fn converted_payload_and_receipt_use_the_original_exact_storage_boundary()
 {
    use surgeist_bindings_js::conversion::{Hosted, ResourceTable, WorkTable, types};
    for short in [false, true] {
        let mut host = profiled_host(surgeist_ledger::ProfileMode::Summary { timing: false });
        let permit = host.permit().unwrap();
        let events = host.guest_events().unwrap();
        let filler = Rc::new(RefCell::new(None));
        let request = Rc::new(Cell::new(None));
        let dropped = Rc::new(Cell::new(false));
        let engine = host.engine.as_ref().unwrap();
        let hosted = Hosted::new(StorageProbeHost {
            inner: engine
                .preparation
                .conversion_host(ConversionOrigin::General),
            owner: Rc::downgrade(&host.native),
            budget: events.budget.clone(),
            limit: events.limits.storage_bytes,
            short,
            filler: filler.clone(),
            request: request.clone(),
            dropped: dropped.clone(),
        });
        let metadata = engine
            .preparation
            .reserve_conversion_metadata(
                ConversionOrigin::General,
                hosted
                    .storage_bytes::<preparation::ConversionBacking>()
                    .unwrap(),
            )
            .unwrap();
        let account = host
            .in_document(&permit, |scope| {
                hosted.install(
                    scope,
                    surgeist_ledger::Limits::new(
                        WorkTable::splat(100),
                        ResourceTable { BufferBytes: 6 },
                    ),
                    metadata,
                )
            })
            .unwrap();
        let result = host.in_document(&permit, |scope| {
            let value = scope.string(&units("abc"))?;
            account
                .run(scope, |conversion| {
                    conversion.value(&types::DOM_STRING, &value)
                })
                .map_err(Error::from)
        });
        let requested = request.get().unwrap();
        assert_eq!(requested.payload_bytes(), 6);
        let report = host
            .guest_accounting()
            .unwrap()
            .snapshot()
            .unwrap()
            .unwrap();
        let conversion = report
            .scopes
            .iter()
            .find(|scope| scope.name == "js-conversion")
            .unwrap();
        assert_eq!(conversion.usage.work(DocumentGuestWork::ConversionValue), 1);
        assert_eq!(
            conversion.usage.work(DocumentGuestWork::ConversionCodeUnit),
            if short { 0 } else { 3 }
        );
        drop(account);
        assert!(
            dropped.get(),
            "converted storage must not retain its factory"
        );
        drop(filler.borrow_mut().take());
        let budget = events.budget.clone();
        drop(events);
        if short {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one byte short must refuse the original guest storage");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::Document(
                    DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                ))
            ));
            drop(failure);
            host.stop().into_session().unwrap();
            assert_eq!(budget.used(), 0);
        } else {
            let converted = result.unwrap();
            assert_eq!(converted.get().as_code_units(), units("abc"));
            host.stop().into_session().unwrap();
            // The metadata and payload remain admitted after Process retirement.
            assert!(budget.used() >= requested.backing_bytes());
            assert_eq!(converted.get().as_code_units(), units("abc"));
            drop(converted);
            assert_eq!(budget.used(), 0);
        }
    }
}
