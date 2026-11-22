use super::evaluate;
use surgeist_bindings_js::{Error, Process};

pub(super) fn native_language_profile(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let mut failures = Vec::new();
    // Expected results were observed on the pinned stock Rusty V8 152.2.0.
    for (feature, member, kind, operation, expected) in [
        (
            "Intl.DateTimeFormat",
            "globalThis.Intl?.DateTimeFormat",
            "function",
            "new Intl.DateTimeFormat('en-US', { timeZone: 'UTC', year: 'numeric' }).format(0)",
            r#""1970""#,
        ),
        (
            "Intl.NumberFormat",
            "globalThis.Intl?.NumberFormat",
            "function",
            "new Intl.NumberFormat('en-US').format(1234.5)",
            r#""1,234.5""#,
        ),
        (
            "Temporal.Instant",
            "globalThis.Temporal?.Instant",
            "function",
            "Temporal.Instant.from('2000-01-01T00:00Z').epochMilliseconds",
            "946684800000",
        ),
        (
            "Float16Array",
            "globalThis.Float16Array",
            "function",
            "[new Float16Array([1.25])[0], Float16Array.BYTES_PER_ELEMENT]",
            "[1.25,2]",
        ),
        (
            "Math.sumPrecise",
            "globalThis.Math?.sumPrecise",
            "function",
            "Math.sumPrecise([1e100, 1, -1e100])",
            "1",
        ),
        (
            "escape",
            "globalThis.escape",
            "function",
            "escape('é')",
            r#""%E9""#,
        ),
        (
            "unescape",
            "globalThis.unescape",
            "function",
            "unescape('%E9')",
            r#""é""#,
        ),
    ] {
        let source = format!(
            "(() => {{
                const kind = typeof {member};
                if (kind !== '{kind}') return 'unavailable (type is ' + kind + ')';
                try {{ return 'result: ' + JSON.stringify({operation}); }}
                catch (error) {{ return 'operation threw ' + String(error); }}
            }})()"
        );
        let output = evaluate(&mut execution, &realm, &source)?;
        let actual = String::from_utf16_lossy(&output);
        if actual != format!("result: {expected}") {
            failures.push(format!("{feature}: expected {expected}, observed {actual}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
