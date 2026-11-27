//! Golden-XML generation for the browser-parity corpus.
//!
//! The command-line driver, the measurement adapter, the browser profiles and the
//! XML serialization that the `surgeist-layout-generate` binary and the
//! qualification driver share.

pub(crate) mod adapter;
pub(crate) mod browser_profile;
mod cli;
pub(crate) mod envelope;
pub(crate) mod measurement;
pub(crate) mod source_owner;
pub(crate) mod xml;

#[cfg(test)]
mod helper_protocol_tests;

/// Run the generator command named on the process command line.
///
/// A browser helper process re-entered through the supervisor protocol is served
/// instead of a command. Failures are reported on standard error and map to a
/// nonzero exit status.
pub fn run() -> std::process::ExitCode {
    let result = match surgeist_generator::browser::run_supervisor_from_env() {
        Some(result) => result.map_err(|error| error.to_string()),
        None => cli::run_from_env(),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("surgeist-layout-generate: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
