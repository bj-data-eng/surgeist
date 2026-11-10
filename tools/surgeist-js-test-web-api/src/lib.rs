//! Boa's **`surgeist_js_test_web_api`** crate implements the [WinterTC (TC55) Minimum Common Web API](https://min-common-api.proposal.wintertc.org/)
//! for the `surgeist_js` crate.
//!
//! `WinterTC` (TC55) is an Ecma International Technical Committee working towards a baseline set
//! of Web Platform APIs that all server-side JavaScript runtimes (Deno, Bun, Cloudflare Workers,
//! Node.js, etc.) agree to implement, enabling portable server-side JavaScript.
//!
//! # Relationship to `surgeist_js_test_host`
//!
//! `surgeist_js_test_web_api` is a standalone crate that depends only on `surgeist_js`.
//! `surgeist_js_test_host` depends on `surgeist_js_test_web_api` and re-exports its APIs, so users of `surgeist_js_test_host`
//! automatically get TC55 compliance without any extra setup.
//!
//! If you only want the TC55-mandated APIs and nothing else, depend on `surgeist_js_test_web_api` directly.
//!
//! # Example: Registering all TC55 APIs
//!
//! ```no_run
//! use surgeist_js::Context;
//!
//! let mut context = Context::builder().build().unwrap();
//!
//! surgeist_js_test_web_api::register(None, &mut context)
//!     .expect("failed to register TC55 APIs");
//! ```
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![cfg_attr(test, allow(clippy::needless_raw_string_hashes))] // Makes strings a bit more copy-pastable
#![allow(
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::let_unit_value
)]

pub mod abort;
pub mod base64;
pub mod clone;
pub mod console;
pub mod encoding;
pub mod events;
#[cfg(feature = "fetch")]
pub mod fetch;
pub mod microtask;
#[cfg(test)]
mod publication_test_support;
pub mod store;
pub mod timers;
#[cfg(feature = "url")]
pub mod url;

/// Register all TC55-mandated Web APIs into the given [`surgeist_js::Context`].
///
/// This registers the Minimum Common Web API as specified by `WinterTC` (TC55):
/// <https://min-common-api.proposal.wintertc.org/>
///
/// # Errors
///
/// Returns a [`surgeist_js::JsError`] if any API fails to register (e.g. a global
/// object already exists with a conflicting name).
#[allow(clippy::needless_pass_by_value)]
pub fn register(
    realm: Option<surgeist_js::realm::Realm>,
    ctx: &mut surgeist_js::Context,
) -> surgeist_js::JsResult<()> {
    console::register(realm.clone(), ctx)?;
    // `timers` mirrors `surgeist_js_test_host::interval::register`, which does not take a
    // realm (timer globals are registered directly on the context).
    timers::register(ctx)?;
    encoding::register(realm.clone(), ctx)?;
    microtask::register(realm.clone(), ctx)?;
    clone::register(realm.clone(), ctx)?;
    base64::register(realm.clone(), ctx)?;
    abort::register(realm.clone(), ctx)?;
    #[cfg(feature = "url")]
    url::register(realm.clone(), ctx)?;
    #[cfg(feature = "fetch")]
    fetch::register(realm, ctx)?;

    Ok(())
}

#[cfg(test)]
pub(crate) mod test {
    use crate::register;
    use std::borrow::Cow;
    use std::path::{Path, PathBuf};
    use std::pin::Pin;
    use surgeist_js::{Context, JsError, JsResult, JsValue, Source, builtins};

    /// A test action executed in a test function.
    #[allow(missing_debug_implementations)]
    pub(crate) struct TestAction(Inner);

    #[allow(dead_code)]
    #[allow(clippy::type_complexity)]
    enum Inner {
        RunHarness,
        Run {
            source: Cow<'static, str>,
        },
        RunFile {
            path: PathBuf,
        },
        RunJobs,
        InspectContext {
            op: Box<dyn FnOnce(&mut Context)>,
        },
        InspectContextAsync {
            op: Box<dyn for<'a> FnOnce(&'a mut Context) -> Pin<Box<dyn Future<Output = ()> + 'a>>>,
        },
        Assert {
            source: Cow<'static, str>,
        },
        AssertEq {
            source: Cow<'static, str>,
            expected: JsValue,
        },
        AssertWithOp {
            source: Cow<'static, str>,
            op: fn(JsValue, &mut Context) -> bool,
        },
        AssertOpaqueError {
            source: Cow<'static, str>,
            expected: JsValue,
        },
        AssertNativeError {
            source: Cow<'static, str>,
            kind: builtins::error::ErrorKind,
            message: &'static str,
        },
        AssertContext {
            op: fn(&mut Context) -> bool,
        },
    }

    impl TestAction {
        #[allow(unused)]
        pub(crate) fn harness() -> Self {
            Self(Inner::RunHarness)
        }

        /// Runs `source`, panicking if the execution throws.
        pub(crate) fn run(source: impl Into<Cow<'static, str>>) -> Self {
            Self(Inner::Run {
                source: source.into(),
            })
        }

        /// Executes `op` with the currently active context.
        ///
        /// Useful to make custom assertions that must be done from Rust code.
        #[allow(unused)]
        pub(crate) fn inspect_context(op: impl FnOnce(&mut Context) + 'static) -> Self {
            Self(Inner::InspectContext { op: Box::new(op) })
        }

        /// Executes `op` with the currently active context in an async environment.
        #[allow(unused)]
        pub(crate) fn inspect_context_async(op: impl AsyncFnOnce(&mut Context) + 'static) -> Self {
            Self(Inner::InspectContextAsync {
                op: Box::new(move |ctx| Box::pin(op(ctx))),
            })
        }
    }

    /// Executes a list of test actions on a new, default context with all TC55 APIs registered.
    #[track_caller]
    #[allow(unused)]
    pub(crate) fn run_test_actions(actions: impl IntoIterator<Item = TestAction>) {
        let context = &mut Context::builder().build().unwrap();
        register(None, context).expect("failed to register WinterTC APIs");
        run_test_actions_with(actions, context);
    }

    /// Executes a list of test actions on the provided context.
    #[track_caller]
    #[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
    pub(crate) fn run_test_actions_with(
        actions: impl IntoIterator<Item = TestAction>,
        context: &mut Context,
    ) {
        #[track_caller]
        fn forward_val(context: &mut Context, source: &str) -> JsResult<JsValue> {
            context.eval(Source::from_bytes(source))
        }

        #[track_caller]
        fn forward_file(context: &mut Context, path: impl AsRef<Path>) -> JsResult<JsValue> {
            let p = path.as_ref();
            context.eval(Source::from_filepath(p).map_err(JsError::from_rust)?)
        }

        #[track_caller]
        fn fmt_test(source: &str, test: usize) -> String {
            format!(
                "\n\nTest case {test}: \n```\n{}\n```",
                textwrap::indent(source, "    ")
            )
        }

        // Some unwrapping patterns look weird because they're replaceable
        // by simpler patterns like `unwrap_or_else` or `unwrap_err
        let mut i = 1;
        for action in actions.into_iter().map(|a| a.0) {
            match action {
                Inner::RunHarness => {
                    if let Err(e) = forward_file(context, "./assets/harness.js") {
                        panic!("Uncaught {e} in the test harness");
                    }
                }
                Inner::Run { source } => {
                    if let Err(e) = forward_val(context, &source) {
                        panic!("{}\nUncaught {e}", fmt_test(&source, i));
                    }
                }
                Inner::RunFile { path } => {
                    if let Err(e) = forward_file(context, &path) {
                        panic!("Uncaught {e} in file {path:?}");
                    }
                }
                Inner::RunJobs => {
                    if let Err(e) = context.run_jobs() {
                        panic!("Uncaught {e} in a job");
                    }
                }
                Inner::InspectContext { op } => {
                    op(context);
                }
                Inner::InspectContextAsync { op } => futures_lite::future::block_on(op(context)),
                Inner::Assert { source } => {
                    let val = match forward_val(context, &source) {
                        Err(e) => panic!("{}\nUncaught {e}", fmt_test(&source, i)),
                        Ok(v) => v,
                    };
                    let Some(val) = val.as_boolean() else {
                        panic!(
                            "{}\nTried to assert with the non-boolean value `{}`",
                            fmt_test(&source, i),
                            val.display()
                        )
                    };
                    assert!(val, "{}", fmt_test(&source, i));
                    i += 1;
                }
                Inner::AssertEq { source, expected } => {
                    let val = match forward_val(context, &source) {
                        Err(e) => panic!("{}\nUncaught {e}", fmt_test(&source, i)),
                        Ok(v) => v,
                    };
                    assert_eq!(val, expected, "{}", fmt_test(&source, i));
                    i += 1;
                }
                Inner::AssertWithOp { source, op } => {
                    let val = match forward_val(context, &source) {
                        Err(e) => panic!("{}\nUncaught {e}", fmt_test(&source, i)),
                        Ok(v) => v,
                    };
                    assert!(op(val, context), "{}", fmt_test(&source, i));
                    i += 1;
                }
                Inner::AssertOpaqueError { source, expected } => {
                    let err = match forward_val(context, &source) {
                        Ok(v) => panic!(
                            "{}\nExpected error, got value `{}`",
                            fmt_test(&source, i),
                            v.display()
                        ),
                        Err(e) => e,
                    };
                    let Some(err) = err.as_opaque() else {
                        panic!(
                            "{}\nExpected opaque error, got native error `{}`",
                            fmt_test(&source, i),
                            err
                        )
                    };

                    assert_eq!(err, &expected, "{}", fmt_test(&source, i));
                    i += 1;
                }
                Inner::AssertNativeError {
                    source,
                    kind,
                    message,
                } => {
                    let err = match forward_val(context, &source) {
                        Ok(v) => panic!(
                            "{}\nExpected error, got value `{}`",
                            fmt_test(&source, i),
                            v.display()
                        ),
                        Err(e) => e,
                    };
                    let native = match err.try_native(context.access()) {
                        Ok(err) => err,
                        Err(e) => panic!(
                            "{}\nCouldn't obtain a native error: {e}",
                            fmt_test(&source, i)
                        ),
                    };

                    assert_eq!(native.kind(), &kind, "{}", fmt_test(&source, i));
                    assert_eq!(native.message(), message, "{}", fmt_test(&source, i));
                    i += 1;
                }
                Inner::AssertContext { op } => {
                    assert!(op(context), "Test case {i}");
                    i += 1;
                }
            }
        }
    }
}
