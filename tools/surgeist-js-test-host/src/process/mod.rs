//! Boa's implementation of Node.js' `process` object.  
//!  
//! The `process` object can be accessed from any global object.  
//!  
//! More information:  
//!  - [Node.js documentation][node]  
//!  
//! [node]: https://nodejs.org/api/process.html  

#[cfg(test)]
pub(crate) mod tests;

use std::rc::Rc;
use surgeist_js::{
    Context, ContextAccess, JsData, JsObject, JsResult, JsString, JsSymbol, JsValue,
    gc::heap::Trace as HeapTrace,
    js_error, js_string,
    native_function::{NativeCapture, NativeFunction},
    object::ObjectInitializer,
    property::Attribute,
};

/// A trait that can be used to forward process provider to an implementation.  
pub trait ProcessProvider: HeapTrace {
    /// Get current working directory (`process.cwd()`)  
    ///  
    /// # Errors  
    /// Returns an error if the current directory cannot be obtained.  
    fn cwd(&self) -> JsResult<JsString>;

    /// Get environment variables so as to allow env property (`process.env`)
    fn env(&self) -> impl IntoIterator<Item = (JsString, JsString)>;
}

/// The default std implementation of the process provider.  
///  
/// Implements the [`ProcessProvider`] trait. Outputs the process properties'
/// values on the basis of std.  
#[derive(Debug, HeapTrace)]
pub struct StdProcessProvider;

impl ProcessProvider for StdProcessProvider {
    fn cwd(&self) -> JsResult<JsString> {
        let path = std::env::current_dir().map_err(
            |e| js_error!(TypeError: "failed to get current working directory: {}", e.to_string()),
        )?;
        Ok(js_string!(path.to_string_lossy()))
    }

    fn env(&self) -> impl IntoIterator<Item = (JsString, JsString)> {
        std::env::vars().map(|(k, v)| (js_string!(k), js_string!(v)))
    }
}

#[derive(HeapTrace)]
struct ProviderCapture<P: ProcessProvider>(Rc<P>);

impl<P: ProcessProvider> NativeCapture for ProviderCapture<P> {
    type Stored = Self;

    fn store(&self) -> Self {
        Self(self.0.clone())
    }

    fn restore(stored: &Self) -> Result<Self, surgeist_js::error::RootingError> {
        Ok(stored.store())
    }
}

/// Boa's implementation of Node.js' `process` object.  
#[derive(Debug, JsData, HeapTrace)]
pub struct Process;

impl Process {
    /// Name of the built-in `process` property.  
    pub const NAME: JsString = js_string!("process");

    /// Initializes the `process` built-in object with a custom provider.  
    ///  
    /// # Errors  
    ///  
    /// Returns a `JsError` if:  
    /// - Custom process provider returns an error  
    /// - Defining the `cwd` and `env` properties on the `process` object fails  
    pub fn init_with_provider<P>(mut context: ContextAccess<'_>, provider: P) -> JsResult<JsObject>
    where
        P: ProcessProvider + 'static,
    {
        let env = JsObject::default(context.intrinsics())?;
        for (key, value) in provider.env() {
            env.set(key, JsValue::from(value), false, context.reborrow())?;
        }

        ObjectInitializer::new(context.reborrow())
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("env"),
                env,
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .function(
                NativeFunction::from_fn_with_captures(
                    |_, _, provider, _| provider.0.cwd().map(JsValue::from),
                    ProviderCapture(Rc::new(provider)),
                ),
                js_string!("cwd"),
                0,
            )
            .build()
    }

    /// Register the `process` object globally by a custom provider.  
    ///  
    /// # Errors  
    /// This function will return an error if the property cannot be defined on the global object.  
    pub fn register_with_provider<P>(context: &mut Context, provider: P) -> JsResult<()>
    where
        P: ProcessProvider + 'static,
    {
        let process_object = Self::init_with_provider(context.access(), provider)?;

        context.register_global_property(
            js_string!(Self::NAME),
            process_object,
            Attribute::CONFIGURABLE,
        )?;

        Ok(())
    }

    /// Initializes the `process` built-in object with the default std provider.  
    ///  
    /// # Errors  
    ///  
    /// Returns a `JsError` if:  
    /// - Defining the `cwd` and `env` properties on the `process` object fails  
    pub fn init(mut context: ContextAccess<'_>) -> JsResult<JsObject> {
        Self::init_with_provider(context.reborrow(), StdProcessProvider)
    }

    /// Register the `process` object globally by the default std provider.  
    ///  
    /// # Errors  
    /// This function will return an error if the property cannot be defined on the global object.  
    pub fn register(context: &mut Context) -> JsResult<()> {
        Self::register_with_provider(context, StdProcessProvider)
    }
}
