//! Work admission for the native unloaded Video dimensions.

use super::*;

fn video_read_call(name: &str) -> (DocumentScriptHost, DocumentScriptPermit, Root, Root) {
    let mut host = public_node_host(200_000, 200_000);
    let permit = host.permit().unwrap();
    let (receiver, getter) = host
        .in_document(&permit, |scope| {
            let receiver = scope.evaluate(&units("document.createElement('video')"))?;
            let getter = scope.evaluate(&units(&format!(
                "Object.getOwnPropertyDescriptor(HTMLVideoElement.prototype, '{name}')?.get"
            )))?;
            assert!(
                scope.is_callable(&getter)?,
                "missing HTMLVideoElement.prototype.{name} getter"
            );
            Ok((scope.retain(&receiver), scope.retain(&getter)))
        })
        .unwrap();
    fund_guest_operation_after_setup(&host);
    (host, permit, receiver, getter)
}

pub(in crate::document_scripting) fn video_dimensions_admit_each_native_read() {
    fn read(
        host: &mut DocumentScriptHost,
        permit: &DocumentScriptPermit,
        receiver: &Root,
        getter: &Root,
    ) -> Result<(), DocumentScriptError> {
        host.in_document(permit, |scope| {
            let receiver = scope.import(receiver)?;
            let getter = scope.import(getter)?;
            scope.call(&getter, &receiver, &[]).map(drop)
        })
    }
    for name in ["videoWidth", "videoHeight"] {
        let (mut control, permit, receiver, getter) = video_read_call(name);
        let events = control.guest_events().unwrap();
        let before = events.budget.work().unwrap();
        read(&mut control, &permit, &receiver, &getter).unwrap();
        let work = events.budget.work().unwrap() - before;
        assert!(work > 0);
        drop((receiver, getter, events));
        control.stop().into_session().unwrap();
        for available in [work, work - 1] {
            let (mut host, permit, receiver, getter) = video_read_call(name);
            let events = host.guest_events().unwrap();
            events.budget.set_work_remaining(available);
            let stamp = host.snapshot().unwrap().stamp();
            let used = events.budget.used() - events.budget.capture_used();
            match read(&mut host, &permit, &receiver, &getter) {
                Ok(()) => {
                    assert_eq!(available, work);
                    assert_eq!(events.budget.work_remaining().unwrap(), 0);
                }
                Err(DocumentScriptError::Script(Error::HostFailure(failure))) => {
                    assert_eq!(available, work - 1);
                    assert!(matches!(
                        failure.cause::<DocumentGuestEventError>(),
                        Some(DocumentGuestEventError::WorkLimit)
                    ));
                    drop(failure);
                }
                Err(error) => panic!("unexpected Video read admission: {error:?}"),
            }
            assert_eq!(host.snapshot().unwrap().stamp(), stamp);
            assert_eq!(events.budget.used() - events.budget.capture_used(), used);
            let budget = events.budget.clone();
            drop((receiver, getter, events));
            host.stop().into_session().unwrap();
            assert_eq!(budget.used(), 0);
        }
    }
}
