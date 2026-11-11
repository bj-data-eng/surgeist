//! Attached Window timer behavior under the pinned HTML timer steps.

use super::*;

pub(super) fn window_schedules_and_cancels_after_guest_entry() {
    let (session, target) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .with_window_timer_limits(
            surgeist_runtime::document_scripting::WindowTimerLimits::new(16, 8, 4096),
        )
        .enable(target, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let script = r#"(() => {
        if (typeof setTimeout !== "function" || typeof clearTimeout !== "function") {
            return "the attached Window must expose the timer operations";
        }
        const marker = {};
        const trace = [];
        const first = setTimeout(function (value, absent) {
            trace.push(this === window && value === marker && absent === undefined);
        }, 0, marker, undefined);
        const canceled = setTimeout(() => { trace.push("cleared callback ran"); });
        clearTimeout(canceled);
        if (setTimeout.length !== 1 || clearTimeout.length !== 0 ||
            !Number.isInteger(first) || first <= 0 || first === canceled ||
            trace.length !== 0) {
            return "timer initialization must return distinct IDs before any callback";
        }
        return "ok";
    })()"#;
    let result = host
        .in_document(&permit, |scope| {
            let observed = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&observed)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}
