use super::{evaluate, units};
use surgeist_bindings_js::{Error, Process};

pub(super) fn module_resolution_stays_host_owned(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    evaluate(
        &mut execution,
        &realm,
        "globalThis.importOutcome = 'pending';
         import('surgeist:unregistered').then(
             () => { importOutcome = 'loaded'; },
             error => { importOutcome = error.name + ': ' + error.message; }
         );
         importOutcome",
    )?;
    execution.checkpoint()?;
    assert_eq!(
        evaluate(&mut execution, &realm, "importOutcome")?,
        units("TypeError: module resolution is disabled for this context"),
    );
    Ok(())
}
