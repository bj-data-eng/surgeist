//! Capture the pinned harness's public completion report inside the worker.

use std::{cell::RefCell, rc::Rc};

use surgeist_bindings_js::{
    Arguments, Callback, Completion, Error, FunctionMetadata, ObjectPrototype, Primitive,
    PropertyKey, Scope, Value, ValueKind,
};
use surgeist_js_wpt::{GuestDiagnostic, HarnessResult, HarnessStatus, Subtest, SubtestStatus};

const MAX_SUBTESTS: u32 = 16;
// Even at six JSON bytes per UTF-16 unit, this leaves room in the frame
// for the fixture identity, statuses, field names and resource observations.
const MAX_TEXT_UNITS: usize = 4096;

#[derive(Default)]
pub(super) enum Captured {
    #[default]
    Pending,
    Report(HarnessResult),
    Invalid,
}

pub(super) fn install(scope: &mut Scope<'_>, slot: Rc<RefCell<Captured>>) -> Result<(), Error> {
    let window = scope.global()?;
    let setup = get(scope, &window, "setup")?;
    let settings = scope.object(ObjectPrototype::Default)?;
    let output = scope.primitive(Primitive::Boolean(false))?;
    let output_key: Vec<_> = "output".encode_utf16().collect();
    scope.set(&settings, PropertyKey::String(&output_key), &output)?;
    let receiver = scope.primitive(Primitive::Undefined)?;
    // The worker reads the public report directly. The optional in-page
    // status table requires selector APIs outside this native profile.
    scope.call(&setup, &receiver, &[settings])?;

    let register = get(scope, &window, "add_completion_callback")?;
    let record = Callback::new(move |scope, arguments| {
        let report = read_completion(scope, &arguments)?;
        let mut captured = slot.borrow_mut();
        *captured = match (&*captured, report) {
            (Captured::Pending, Some(report)) => Captured::Report(report),
            _ => Captured::Invalid,
        };
        Ok(Completion::Undefined)
    });
    let callback = scope.fixed_arity_function(
        &record,
        FunctionMetadata {
            name: &[],
            length: 2,
        },
    )?;
    scope.call(&register, &receiver, &[callback])?;
    Ok(())
}

fn get<'scope>(
    scope: &mut Scope<'scope>,
    object: &Value<'scope>,
    name: &str,
) -> Result<Value<'scope>, Error> {
    scope.get(
        object,
        PropertyKey::String(&name.encode_utf16().collect::<Vec<_>>()),
    )
}

fn read_completion<'scope>(
    scope: &mut Scope<'scope>,
    arguments: &Arguments<'scope>,
) -> Result<Option<HarnessResult>, Error> {
    let (Some(tests), Some(harness)) = (arguments.get(0), arguments.get(1)) else {
        return Ok(None);
    };
    if scope.kind(tests)? != ValueKind::Object || scope.kind(harness)? != ValueKind::Object {
        return Ok(None);
    }
    // Numeric codes are the pinned harness's TestsStatus.statuses and Test.statuses.
    let status = match read_integer(scope, harness, "status", 3)? {
        Some(0) => HarnessStatus::Ok,
        Some(1) => HarnessStatus::Error,
        Some(2) => HarnessStatus::Timeout,
        Some(3) => HarnessStatus::PreconditionFailed,
        _ => return Ok(None),
    };
    let Some(length) = read_integer(scope, tests, "length", MAX_SUBTESTS)? else {
        return Ok(None);
    };
    let mut remaining = MAX_TEXT_UNITS;
    let Some(diagnostic) = read_diagnostic(scope, harness, &mut remaining)? else {
        return Ok(None);
    };
    let mut subtests = Vec::with_capacity(length as usize);
    for index in 0..length {
        let test = get(scope, tests, &index.to_string())?;
        if scope.kind(&test)? != ValueKind::Object {
            return Ok(None);
        }
        let status = match read_integer(scope, &test, "status", 4)? {
            Some(0) => SubtestStatus::Pass,
            Some(1) => SubtestStatus::Fail,
            Some(2) => SubtestStatus::Timeout,
            Some(3) => SubtestStatus::NotRun,
            Some(4) => SubtestStatus::PreconditionFailed,
            _ => return Ok(None),
        };
        let (Some(Some(name)), Some(diagnostic)) = (
            read_text(scope, &test, "name", &mut remaining)?,
            read_diagnostic(scope, &test, &mut remaining)?,
        ) else {
            return Ok(None);
        };
        subtests.push(Subtest {
            index,
            name,
            status,
            diagnostic,
        });
    }
    Ok(Some(HarnessResult {
        status,
        diagnostic,
        subtests,
    }))
}

fn read_integer<'scope>(
    scope: &mut Scope<'scope>,
    object: &Value<'scope>,
    key: &str,
    max: u32,
) -> Result<Option<u32>, Error> {
    let value = get(scope, object, key)?;
    if scope.kind(&value)? != ValueKind::Number {
        return Ok(None);
    }
    let number = scope.unrestricted_double(&value)?;
    Ok(
        ((0.0..=f64::from(max)).contains(&number) && number.fract() == 0.0)
            .then_some(number as u32),
    )
}

fn read_diagnostic<'scope>(
    scope: &mut Scope<'scope>,
    object: &Value<'scope>,
    remaining: &mut usize,
) -> Result<Option<GuestDiagnostic>, Error> {
    let message = read_text(scope, object, "message", remaining)?;
    let stack = read_text(scope, object, "stack", remaining)?;
    Ok(message
        .zip(stack)
        .map(|(message, stack)| GuestDiagnostic { message, stack }))
}

// Outer None denotes malformed or over-limit data. A null/undefined guest
// property is valid and is distinct from an empty string in the protocol.
fn read_text<'scope>(
    scope: &mut Scope<'scope>,
    object: &Value<'scope>,
    key: &str,
    remaining: &mut usize,
) -> Result<Option<Option<Vec<u16>>>, Error> {
    let value = get(scope, object, key)?;
    match scope.kind(&value)? {
        ValueKind::Null | ValueKind::Undefined => Ok(Some(None)),
        ValueKind::String => {
            let units = scope.to_utf16(&value)?;
            let Some(rest) = remaining.checked_sub(units.len()) else {
                return Ok(None);
            };
            *remaining = rest;
            Ok(Some(Some(units.to_vec())))
        }
        _ => Ok(None),
    }
}
