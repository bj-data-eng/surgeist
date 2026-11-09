//! Boa's implementation of ECMAScript's global `Function` object and Native Functions.
//!
//! Objects wrap `Function`s and expose them via call/construct slots.
//!
//! The `Function` object is used for matching text with a pattern.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-function-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function

use crate::{
    Context, JsArgs, JsExpect, JsResult, JsStr, JsString, JsValue, SpannedSourceText,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject, OrdinaryObject,
    },
    bytecompiler::{FunctionCompiler, control::CompilerWork},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    environments::{
        EnvironmentStack, FunctionSlots, PrivateEnvironment, StoredEnvironmentStack,
        ThisBindingStatus,
    },
    error::{JsNativeError, RuntimeLimitError},
    js_string,
    native_function::NativeFunctionObject,
    object::{
        JsData, JsFunction, JsObject, ObjectEdge, PrivateElement, PrivateName,
        StoredPrivateElement,
        internal_methods::{
            CallValue, InternalMethodCallContext, InternalObjectMethods, ORDINARY_INTERNAL_METHODS,
            get_prototype_from_constructor,
        },
    },
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::{Realm, RealmEdge},
    string::StaticJsStrings,
    symbol::JsSymbol,
    value::IntegerOrInfinity,
    vm::{
        ActivatedCode, ActiveRunnable, CallFrame, CallFrameFlags, CodeBlock, FirstDispatchPermit,
        StoredActiveRunnable,
    },
};
use std::{iter, rc::Rc};
use surgeist_js_ast::{
    Position, Span, Spanned, StatementList,
    function::{FormalParameterList, FunctionBody, FunctionExpression},
    operations::{ContainsSymbol, NameAdmission, try_bound_names, try_lexically_declared_names},
    scope::BindingLocatorScope,
};

use surgeist_js_gc::heap::Trace as HeapTrace;
use surgeist_js_interner::{Interner, Sym};
use surgeist_js_macros::js_str;
use surgeist_js_parser::{
    Source,
    frontend::{FrontendControl, FrontendLimit, FrontendWork},
};
use thin_vec::ThinVec;

use super::Proxy;

pub(crate) mod arguments;
mod bound;

pub use bound::BoundFunction;

#[cfg(test)]
mod tests;

/// Represents the `[[ThisMode]]` internal slot of function objects.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ThisMode {
    /// The `this` value refers to the `this` value of a lexically enclosing function.
    Lexical,

    /// The `this` value is used exactly as provided by an invocation of the function.
    Strict,

    /// The `this` value of `undefined` or `null` is interpreted as a reference to the global object,
    /// and any other `this` value is first passed to `ToObject`.
    Global,
}

impl ThisMode {
    /// Returns `true` if the this mode is `Lexical`.
    #[must_use]
    pub const fn is_lexical(&self) -> bool {
        matches!(self, Self::Lexical)
    }

    /// Returns `true` if the this mode is `Strict`.
    #[must_use]
    pub const fn is_strict(&self) -> bool {
        matches!(self, Self::Strict)
    }

    /// Returns `true` if the this mode is `Global`.
    #[must_use]
    pub const fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}

/// Represents the `[[ConstructorKind]]` internal slot of function objects.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructorKind {
    /// The class constructor is not derived.
    Base,

    /// The class constructor is a derived class constructor.
    Derived,
}

impl ConstructorKind {
    /// Returns `true` if the constructor kind is `Base`.
    #[must_use]
    pub const fn is_base(&self) -> bool {
        matches!(self, Self::Base)
    }

    /// Returns `true` if the constructor kind is `Derived`.
    #[must_use]
    pub const fn is_derived(&self) -> bool {
        matches!(self, Self::Derived)
    }
}

/// Record containing the field definition of classes.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-classfielddefinition-record-specification-type
#[derive(Clone, Debug, HeapTrace)]
pub(crate) enum ClassFieldDefinition {
    /// A class field definition with a `string` or `symbol` as a name.
    Public(
        #[heap_trace(skip)] PropertyKey,
        ObjectEdge,
        #[heap_trace(skip)] Option<PropertyKey>,
    ),

    /// A class field definition with a private name.
    Private(PrivateName, ObjectEdge),
}

/// Boa representation of a JavaScript Function Object.
///
/// `FunctionBody` is specific to this interpreter, it will either be Rust code or JavaScript code
/// (AST Node).
///
/// <https://tc39.es/ecma262/#sec-ecmascript-function-objects>
#[derive(Debug, HeapTrace)]
pub struct OrdinaryFunction {
    /// The code block containing the compiled function.
    #[heap_trace(skip)]
    pub(crate) code: Rc<CodeBlock>,

    /// The `[[Environment]]` internal slot.
    pub(crate) environments: StoredEnvironmentStack,

    /// The `[[HomeObject]]` internal slot.
    home_object: Option<ObjectEdge>,

    /// The `[[ScriptOrModule]]` internal slot.
    pub(crate) script_or_module: Option<StoredActiveRunnable>,

    /// The [`Realm`] the function is defined in.
    realm: RealmEdge,

    /// The `[[Fields]]` internal slot.
    fields: ThinVec<ClassFieldDefinition>,

    /// The `[[PrivateMethods]]` internal slot.
    private_methods: ThinVec<(PrivateName, StoredPrivateElement)>,
}

impl JsData for OrdinaryFunction {
    fn internal_methods(&self) -> &'static InternalObjectMethods {
        static FUNCTION_METHODS: InternalObjectMethods = InternalObjectMethods {
            __call__: function_call,
            ..ORDINARY_INTERNAL_METHODS
        };

        static CONSTRUCTOR_METHODS: InternalObjectMethods = InternalObjectMethods {
            __call__: function_call,
            __construct__: function_construct,
            ..ORDINARY_INTERNAL_METHODS
        };

        if self.code.has_prototype_property() {
            &CONSTRUCTOR_METHODS
        } else {
            &FUNCTION_METHODS
        }
    }
}

impl OrdinaryFunction {
    pub(crate) fn new(
        code: ActivatedCode,
        environments: StoredEnvironmentStack,
        script_or_module: Option<ActiveRunnable>,
        realm: Realm,
    ) -> Self {
        Self::from_code(code.into_code(), environments, script_or_module, realm)
    }

    /// Keeps pending function data inside an unpublished object recipe.
    pub(crate) fn prepare_unpublished(
        code: Rc<CodeBlock>,
        environments: StoredEnvironmentStack,
        script_or_module: Option<ActiveRunnable>,
        realm: Realm,
        template: &crate::object::shape::shared_shape::template::ObjectTemplate,
        storage: impl IntoIterator<Item = JsValue>,
    ) -> JsResult<crate::object::UnpublishedObject<Self>> {
        template.create_unpublished(
            Self::from_code(code, environments, script_or_module, realm),
            storage,
        )
    }

    fn from_code(
        code: Rc<CodeBlock>,
        environments: StoredEnvironmentStack,
        script_or_module: Option<ActiveRunnable>,
        realm: Realm,
    ) -> Self {
        Self {
            code,
            environments,
            home_object: None,
            script_or_module: script_or_module.as_ref().map(ActiveRunnable::stored),
            realm: realm.edge(),
            fields: ThinVec::default(),
            private_methods: ThinVec::default(),
        }
    }

    /// Returns the codeblock of the function.
    #[must_use]
    pub fn codeblock(&self) -> &CodeBlock {
        &self.code
    }

    /// Push a private environment to the function.
    pub(crate) fn push_private_environment(&mut self, environment: Rc<PrivateEnvironment>) {
        self.environments.push_private(environment);
    }

    /// Returns true if the function object is a derived constructor.
    pub(crate) fn is_derived_constructor(&self) -> bool {
        self.code.is_derived_constructor()
    }

    /// Does this function have the `[[ClassFieldInitializerName]]` internal slot set to non-empty value.
    pub(crate) fn in_class_field_initializer(&self) -> bool {
        self.code.in_class_field_initializer()
    }

    /// Returns the function `[[HomeObject]]` if present.
    pub(crate) fn get_home_object(&self) -> Option<JsObject> {
        self.home_object.clone().map(JsObject::from_inner)
    }

    ///  Sets the `[[HomeObject]]` slot if present.
    pub(crate) fn set_home_object(&mut self, object: JsObject) {
        self.home_object = Some(object.into_record());
    }

    /// Returns the values of the `[[Fields]]` internal slot.
    pub(crate) fn get_fields(&self) -> &[ClassFieldDefinition] {
        &self.fields
    }

    /// Pushes a value to the `[[Fields]]` internal slot if present.
    pub(crate) fn push_field(
        &mut self,
        key: PropertyKey,
        value: JsFunction,
        function_name: Option<PropertyKey>,
    ) {
        self.fields.push(ClassFieldDefinition::Public(
            key,
            value.into_record(),
            function_name,
        ));
    }

    /// Pushes a private value to the `[[Fields]]` internal slot if present.
    pub(crate) fn push_field_private(&mut self, name: PrivateName, value: JsFunction) {
        self.fields
            .push(ClassFieldDefinition::Private(name, value.into_record()));
    }

    /// Returns the values of the `[[PrivateMethods]]` internal slot.
    pub(crate) fn get_private_methods(&self) -> &[(PrivateName, StoredPrivateElement)] {
        &self.private_methods
    }

    /// Pushes a private method to the `[[PrivateMethods]]` internal slot if present.
    pub(crate) fn push_private_method(&mut self, name: PrivateName, method: &PrivateElement) {
        self.private_methods.push((name, method.into()));
    }

    /// Gets the `Realm` from where this function originates.
    #[must_use]
    pub fn realm(&self) -> Realm {
        Realm::from_edge(self.realm.clone())
    }

    /// Checks if this function is an ordinary function.
    pub(crate) fn is_ordinary(&self) -> bool {
        self.code.is_ordinary()
    }
}

/// The internal representation of a `Function` object.
#[derive(Debug, Clone, Copy)]
pub struct BuiltInFunctionObject;

struct EarlyErrorNames<'a, 'control> {
    control: Option<&'a mut FrontendControl<'control>>,
}

impl EarlyErrorNames<'_, '_> {
    fn collect(
        &mut self,
        collect: impl FnOnce(NameAdmission<'_, FrontendLimit>) -> Result<Vec<Sym>, FrontendLimit>,
    ) -> JsResult<Vec<Sym>> {
        let result = match self.control.as_deref_mut() {
            Some(control) => collect(Some(&mut |work| control(work.into()))),
            None => collect(None),
        };
        result.map_err(|limit| RuntimeLimitError::from(limit).into())
    }

    fn equal(&mut self, left: Sym, right: Sym) -> JsResult<bool> {
        if let Some(control) = self.control.as_deref_mut() {
            control(FrontendWork::NameComparison).map_err(RuntimeLimitError::from)?;
        }
        Ok(left == right)
    }
}

impl IntrinsicObject for BuiltInFunctionObject {
    fn init(realm: &Realm) {
        let has_instance = BuiltInBuilder::callable(realm, Self::has_instance)
            .name(js_string!("[Symbol.hasInstance]"))
            .length(1)
            .build();

        let throw_type_error = realm.intrinsics().objects().throw_type_error();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::apply, js_string!("apply"), 2)
            .method(Self::bind, js_string!("bind"), 1)
            .method(Self::call, js_string!("call"), 1)
            .method(Self::to_string, js_string!("toString"), 0)
            .property(JsSymbol::has_instance(), has_instance, Attribute::default())
            .accessor(
                js_string!("caller"),
                Some(throw_type_error.clone()),
                Some(throw_type_error.clone()),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("arguments"),
                Some(throw_type_error.clone()),
                Some(throw_type_error),
                Attribute::CONFIGURABLE,
            )
            .build();

        let prototype = realm.intrinsics().constructors().function().prototype();

        BuiltInBuilder::callable_with_object(realm, prototype.clone(), Self::prototype)
            .name(js_string!())
            .length(0)
            .build();

        prototype.set_prototype(Some(realm.intrinsics().constructors().object().prototype()));
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for BuiltInFunctionObject {
    const NAME: JsString = StaticJsStrings::FUNCTION;
}

impl BuiltInConstructor for BuiltInFunctionObject {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 10;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::function;

    /// `Function ( p1, p2, … , pn, body )`
    ///
    /// The `apply()` method invokes self with the first argument as the `this` value
    /// and the rest of the arguments provided as an array (or an array-like object).
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-p1-p2-pn-body
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/Function
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let active_function = context
            .active_function_object()
            .unwrap_or_else(|| context.intrinsics().constructors().function().constructor());
        Self::create_dynamic_function(active_function, new_target, args, false, false, context)
            .map(Into::into)
    }
}

impl BuiltInFunctionObject {
    /// `CreateDynamicFunction ( constructor, newTarget, kind, args )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createdynamicfunction
    pub(crate) fn create_dynamic_function(
        constructor: JsObject,
        new_target: &JsValue,
        args: &[JsValue],
        r#async: bool,
        generator: bool,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. If newTarget is undefined, set newTarget to constructor.
        let new_target = if new_target.is_undefined() {
            constructor.into()
        } else {
            new_target.clone()
        };

        let strict = context.is_strict();

        let default = if r#async && generator {
            // 5. Else,
            //     a. Assert: kind is async-generator.
            //     b. Let prefix be "async function*".
            //     c. Let exprSym be the grammar symbol AsyncGeneratorExpression.
            //     d. Let bodySym be the grammar symbol AsyncGeneratorBody.
            //     e. Let parameterSym be the grammar symbol FormalParameters[+Yield, +Await].
            //     f. Let fallbackProto be "%AsyncGeneratorFunction.prototype%".
            StandardConstructors::async_generator_function
        } else if r#async {
            // 4. Else if kind is async, then
            //     a. Let prefix be "async function".
            //     b. Let exprSym be the grammar symbol AsyncFunctionExpression.
            //     c. Let bodySym be the grammar symbol AsyncFunctionBody.
            //     d. Let parameterSym be the grammar symbol FormalParameters[~Yield, +Await].
            //     e. Let fallbackProto be "%AsyncFunction.prototype%".
            StandardConstructors::async_function
        } else if generator {
            // 3. Else if kind is generator, then
            //     a. Let prefix be "function*".
            //     b. Let exprSym be the grammar symbol GeneratorExpression.
            //     c. Let bodySym be the grammar symbol GeneratorBody.
            //     d. Let parameterSym be the grammar symbol FormalParameters[+Yield, ~Await].
            //     e. Let fallbackProto be "%GeneratorFunction.prototype%".
            StandardConstructors::generator_function
        } else {
            // 2. If kind is normal, then
            //     a. Let prefix be "function".
            //     b. Let exprSym be the grammar symbol FunctionExpression.
            //     c. Let bodySym be the grammar symbol FunctionBody[~Yield, ~Await].
            //     d. Let parameterSym be the grammar symbol FormalParameters[~Yield, ~Await].
            //     e. Let fallbackProto be "%Function.prototype%".
            StandardConstructors::function
        };

        // 22. Let proto be ? GetPrototypeFromConstructor(newTarget, fallbackProto).
        let prototype = get_prototype_from_constructor(&new_target, default, context)?;

        // 6. Let argCount be the number of elements in parameterArgs.
        let (body, param_list) = if let Some((body, params)) = args.split_last() {
            // 7. Let parameterStrings be a new empty List.
            let mut parameters = Vec::with_capacity(args.len());

            // 8. For each element arg of parameterArgs, do
            for param in params {
                // a. Append ? ToString(arg) to parameterStrings.
                parameters.push(param.to_string(context.access())?);
            }

            // 9. Let bodyString be ? ToString(bodyArg).
            let body = body.to_string(context.access())?;

            (body, parameters)
        } else {
            (js_string!(), Vec::new())
        };
        let current_realm = context.realm().clone();

        context
            .host_hooks()
            .ensure_can_compile_strings(current_realm, &param_list, &body, false, context.access())
            .map_err(|error| context.runtime().admit_error(error))?;

        let mut interner = Interner::new();
        let parameters = if param_list.is_empty() {
            FormalParameterList::default()
        } else {
            // 12. Let P be the empty String.
            // 13. If argCount > 0, then
            //     a. Set P to parameterStrings[0].
            //     b. Let k be 1.
            //     c. Repeat, while k < argCount,
            //         i. Let nextArgString be parameterStrings[k].
            //         ii. Set P to the string-concatenation of P, "," (a comma), and nextArgString.
            //         iii. Set k to k + 1.

            // TODO: Replace with standard `Iterator::intersperse` iterator method when it's stabilized.
            //       See: <https://github.com/rust-lang/rust/issues/79524>
            // The comma between fragments bounds each step even for empty arguments.
            let mut parameter_units = itertools::Itertools::intersperse(
                param_list.iter().map(JsString::iter),
                js_str!(",").iter(),
            )
            .flatten();
            let identifier = context.realm().next_parser_identifier()?;

            // 17. Let parameters be ParseText(StringToCodePoints(P), parameterSym).
            // 18. If parameters is a List of errors, throw a SyntaxError exception.
            let parameters = context.parse_source(
                Source::from_utf16_units(&mut parameter_units),
                |mut parser| {
                    parser.set_identifier(identifier);
                    parser
                        .parse_formal_parameters(&mut interner, generator, r#async)
                        .map_err(|error| {
                            crate::JsError::from_parser(error, |error| {
                                format!("failed to parse function parameters: {error}")
                            })
                        })
                },
            )?;

            // It is a Syntax Error if FormalParameters Contains YieldExpression is true.
            if generator
                && context.early_error_contains(&parameters, ContainsSymbol::YieldExpression)?
            {
                return Err(JsNativeError::syntax().with_message(
                    if r#async {
                        "yield expression is not allowed in formal parameter list of async generator"
                    } else {
                        "yield expression is not allowed in formal parameter list of generator"
                    }
                ).into());
            }

            // It is a Syntax Error if FormalParameters Contains AwaitExpression is true.
            if r#async
                && context.early_error_contains(&parameters, ContainsSymbol::AwaitExpression)?
            {
                return Err(JsNativeError::syntax()
                    .with_message(
                        if generator {
                            "await expression is not allowed in formal parameter list of async function"
                        } else {
                            "await expression is not allowed in formal parameter list of async generator"
                        })
                    .into());
            }

            parameters
        };

        let body = if body.is_empty() {
            FunctionBody::new(StatementList::default(), Span::new((1, 1), (1, 1)))
        } else {
            // 14. Let bodyParseString be the string-concatenation of 0x000A (LINE FEED), bodyString, and 0x000A (LINE FEED).
            let lf = u16::from(b'\n');
            let mut body_units = iter::once(lf).chain(body.iter()).chain(iter::once(lf));
            let identifier = context.realm().next_parser_identifier()?;

            // 19. Let body be ParseText(StringToCodePoints(bodyParseString), bodySym).
            // 20. If body is a List of errors, throw a SyntaxError exception.
            let body =
                context.parse_source(Source::from_utf16_units(&mut body_units), |mut parser| {
                    parser.set_identifier(identifier);
                    parser
                        .parse_function_body(&mut interner, generator, r#async)
                        .map_err(|error| {
                            crate::JsError::from_parser(error, |error| {
                                format!("failed to parse function body: {error}")
                            })
                        })
                })?;

            // It is a Syntax Error if AllPrivateIdentifiersValid of StatementList with argument « »
            // is false unless the source text containing ScriptBody is eval code that is being
            // processed by a direct eval.
            // https://tc39.es/ecma262/#sec-scripts-static-semantics-early-errors
            if !context.all_private_identifiers_valid(&body, Vec::new())? {
                return Err(JsNativeError::syntax()
                    .with_message("invalid private identifier usage")
                    .into());
            }

            // 21. NOTE: The parameters and body are parsed separately to ensure that each is valid alone. For example, new Function("/*", "*/ ) {") does not evaluate to a function.
            // 22. NOTE: If this step is reached, sourceText must have the syntax of exprSym (although the reverse implication does not hold). The purpose of the next two steps is to enforce any Early Error rules which apply to exprSym directly.
            // 23. Let expr be ParseText(sourceText, exprSym).
            // 24. If expr is a List of errors, throw a SyntaxError exception.
            // Check for errors that apply for the combination of body and parameters.

            // Early Error: If BindingIdentifier is present and the source text matched by BindingIdentifier is strict mode code,
            // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
            if body.strict() {
                context.with_name_checks(|control, _| {
                    let mut names = EarlyErrorNames { control };
                    for name in
                        names.collect(|admission| try_bound_names(&parameters, admission))?
                    {
                        if names.equal(name, Sym::ARGUMENTS)? || names.equal(name, Sym::EVAL)? {
                            return Err(JsNativeError::syntax()
                                .with_message("Unexpected 'eval' or 'arguments' in strict mode")
                                .into());
                        }
                    }
                    Ok(())
                })?;
            }

            // Early Error: If the source code matching FormalParameters is strict mode code,
            // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
            if (body.strict()) && parameters.has_duplicates() {
                return Err(JsNativeError::syntax()
                    .with_message("Duplicate parameter name not allowed in this context")
                    .into());
            }

            // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of GeneratorBody is true
            // and IsSimpleParameterList of FormalParameters is false.
            if body.strict() && !parameters.is_simple() {
                return Err(JsNativeError::syntax()
                    .with_message(
                        "Illegal 'use strict' directive in function with non-simple parameter list",
                    )
                    .into());
            }

            // It is a Syntax Error if FunctionBody Contains SuperProperty is true.
            if context.early_error_contains(&body, ContainsSymbol::SuperProperty)? {
                return Err(JsNativeError::syntax()
                    .with_message("invalid `super` reference")
                    .into());
            }

            // It is a Syntax Error if FunctionBody Contains SuperCall is true.
            if context.early_error_contains(&body, ContainsSymbol::SuperCall)? {
                return Err(JsNativeError::syntax()
                    .with_message("invalid `super` call")
                    .into());
            }

            // It is a Syntax Error if any element of the BoundNames of FormalParameters
            // also occurs in the LexicallyDeclaredNames of FunctionBody.
            // https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors
            context.with_name_checks(|control, _| {
                let mut names = EarlyErrorNames { control };
                let lexical_names =
                    names.collect(|admission| try_lexically_declared_names(&body, admission))?;
                for name in names.collect(|admission| try_bound_names(&parameters, admission))? {
                    for lexical_name in &lexical_names {
                        if names.equal(name, *lexical_name)? {
                            return Err(JsNativeError::syntax()
                                .with_message(format!(
                                    "Redeclaration of formal parameter `{}`",
                                    interner.resolve_expect(name)
                                ))
                                .into());
                        }
                    }
                }
                Ok(())
            })?;

            body
        };

        // Dynamic functions have no enclosing private environment.
        if !parameters.as_ref().is_empty()
            && !context.all_private_identifiers_valid(&parameters, Vec::new())?
        {
            return Err(JsNativeError::syntax()
                .with_message("invalid private identifier usage")
                .into());
        }

        // TODO: create SourceText : "anonymous(" parameters \n ") {" body_parse "}"

        let function_span_start = Position::new(1, 1);
        let function_span_end = body.span().end();
        let function = context.with_direct_eval_admission(|admission| {
            FunctionExpression::try_new(
                None,
                parameters,
                body,
                None,
                false,
                Span::new(function_span_start, function_span_end),
                admission,
            )
        })?;
        let root = context.realm().scope().clone();
        let mut prepared = context.with_scope_admission(|admission| {
            function
                .try_prepare(strict, &root, &interner, admission)
                .map_err(|error| match error {
                    surgeist_js_ast::scope::ScopeAnalysisError::Syntax(reason) => {
                        surgeist_js_ast::scope::ScopeAnalysisError::Syntax(
                            format!("failed to analyze function scope: {reason}").into(),
                        )
                    }
                    other => other,
                })
        })?;
        let in_with = context.compiler_has_object_environment()?;
        let parts = prepared.parts();
        let scope_owner = parts.scopes;
        let (code, extension) = context.with_compilation_work(|mut control| {
            control.admit(CompilerWork::MetadataVisit)?;
            let root = scope_owner.root_handle();
            let mut extension = control.with_scope_admission(|admission| {
                let owner = scope_owner;
                owner
                    .begin_extension(admission)
                    .map_err(crate::JsError::from)
            })?;
            control.admit(CompilerWork::MetadataWrite)?;
            let code = FunctionCompiler::new(SpannedSourceText::new_empty())
                .name(js_string!("anonymous"))
                .generator(generator)
                .r#async(r#async)
                .in_with(in_with)
                .force_function_scope(true)
                .compile(
                    parts.tree.parameters(),
                    parts.tree.body(),
                    root.clone(),
                    root,
                    parts.tree.scopes(),
                    parts.tree.contains_direct_eval(),
                    &mut interner,
                    &mut extension,
                    control.reborrow(),
                )?;
            Ok((code, extension))
        })?;
        extension.commit_private();
        let environments = EnvironmentStack::new();
        let runnable = context.get_active_script_or_module();
        let realm = context.realm().clone();
        let recipe = crate::vm::prepare_function_object_with_prototype(
            code,
            prototype,
            &environments,
            runnable,
            realm,
            context,
        )?;
        let publication = context.with_installation_work(|admission| {
            prepared
                .parts()
                .scopes
                .prepare_publication(admission)
                .map_err(crate::JsError::from)
        })?;
        context.check_execution_stop()?;
        recipe.publish(publication).map_err(|mut error| {
            context.retain_preparation_error(&mut error);
            error
        })
    }

    /// `Function.prototype.apply ( thisArg, argArray )`
    ///
    /// The `apply()` method invokes self with the first argument as the `this` value
    /// and the rest of the arguments provided as an array (or an array-like object).
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.apply
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/apply
    fn apply(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let func be the this value.
        // 2. If IsCallable(func) is false, throw a TypeError exception.
        let func = this.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message(format!("{} is not a function", this.display()))
        })?;

        let this_arg = args.get_or_undefined(0);
        let arg_array = args.get_or_undefined(1);
        // 3. If argArray is undefined or null, then
        if arg_array.is_null_or_undefined() {
            // a. Perform PrepareForTailCall().
            // TODO?: 3.a. PrepareForTailCall

            // b. Return ? Call(func, thisArg).
            return func.call(this_arg, &[], context.access());
        }

        // 4. Let argList be ? CreateListFromArrayLike(argArray).
        let arg_list = arg_array.create_list_from_array_like(&[], context)?;

        // 5. Perform PrepareForTailCall().
        // TODO?: 5. PrepareForTailCall

        // 6. Return ? Call(func, thisArg, argList).
        func.call(this_arg, &arg_list, context.access())
    }

    /// `Function.prototype.bind ( thisArg, ...args )`
    ///
    /// The `bind()` method creates a new function that, when called, has its
    /// this keyword set to the provided value, with a given sequence of arguments
    /// preceding any provided when the new function is called.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.bind
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_objects/Function/bind
    fn bind(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let Target be the this value.
        // 2. If IsCallable(Target) is false, throw a TypeError exception.
        let target = this.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("cannot bind `this` without a `[[Call]]` internal method")
        })?;

        let this_arg = args.get_or_undefined(0).clone();
        let bound_args = args.get(1..).unwrap_or(&[]).to_vec();
        let arg_count = bound_args.len() as i64;

        // 3. Let F be ? BoundFunctionCreate(Target, thisArg, args).
        let f = BoundFunction::create(target.clone(), this_arg, bound_args, context.access())?;

        // 4. Let L be 0.
        let mut l = JsValue::new(0);

        // 5. Let targetHasLength be ? HasOwnProperty(Target, "length").
        // 6. If targetHasLength is true, then
        if target.has_own_property(StaticJsStrings::LENGTH, context.access())? {
            // a. Let targetLen be ? Get(Target, "length").
            let target_len = target.get(StaticJsStrings::LENGTH, context.access())?;
            // b. If Type(targetLen) is Number, then
            if target_len.is_number() {
                // 1. Let targetLenAsInt be ! ToIntegerOrInfinity(targetLen).
                match target_len
                    .to_integer_or_infinity(context.access())
                    .js_expect("to_integer_or_infinity cannot fail for a number")?
                {
                    // i. If targetLen is +∞𝔽, set L to +∞.
                    IntegerOrInfinity::PositiveInfinity => l = f64::INFINITY.into(),
                    // ii. Else if targetLen is -∞𝔽, set L to 0.
                    IntegerOrInfinity::NegativeInfinity => {}
                    // iii. Else,
                    IntegerOrInfinity::Integer(target_len) => {
                        // 2. Assert: targetLenAsInt is finite.
                        // 3. Let argCount be the number of elements in args.
                        // 4. Set L to max(targetLenAsInt - argCount, 0).
                        l = (target_len - arg_count).max(0).into();
                    }
                }
            }
        }

        // 7. Perform ! SetFunctionLength(F, L).
        f.define_property_or_throw(
            StaticJsStrings::LENGTH,
            PropertyDescriptor::builder()
                .value(l)
                .writable(false)
                .enumerable(false)
                .configurable(true),
            context.access(),
        )
        .js_expect("defining the `length` property for a new object should not fail")?;

        // 8. Let targetName be ? Get(Target, "name").
        let target_name = target.get(js_string!("name"), context.access())?;

        // 9. If Type(targetName) is not String, set targetName to the empty String.
        let target_name = target_name.as_string().unwrap_or_default();

        // 10. Perform SetFunctionName(F, targetName, "bound").
        set_function_name(&f, &target_name.into(), Some(js_str!("bound")), context)?;

        // 11. Return F.
        Ok(f.into())
    }

    /// `Function.prototype.call ( thisArg, ...args )`
    ///
    /// The `call()` method calls a function with a given this value and arguments provided individually.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.call
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call
    fn call(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let func be the this value.
        // 2. If IsCallable(func) is false, throw a TypeError exception.
        let func = this.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message(format!("{} is not a function", this.display()))
        })?;
        let this_arg = args.get_or_undefined(0);

        // 3. Perform PrepareForTailCall().
        // TODO?: 3. Perform PrepareForTailCall

        // 4. Return ? Call(func, thisArg, args).
        func.call(this_arg, args.get(1..).unwrap_or(&[]), context.access())
    }

    /// `Function.prototype.toString()`
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/toString
    #[allow(clippy::wrong_self_convention)]
    fn to_string(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let func be the this value.
        let func = this;

        // TODO:
        //    2. If func is an Object, func has a [[SourceText]] internal slot, func.[[SourceText]] is a sequence of Unicode code points,and HostHasSourceTextAvailable(func) is true, then
        //        a. Return CodePointsToString(func.[[SourceText]]).

        // 3. If func is a built-in function object, return an implementation-defined String source code representation of func.
        //    The representation must have the syntax of a NativeFunction. Additionally, if func has an [[InitialName]] internal slot and
        //    func.[[InitialName]] is a String, the portion of the returned String that would be matched by
        //    NativeFunctionAccessor_opt PropertyName must be the value of func.[[InitialName]].

        // 4. If func is an Object and IsCallable(func) is true, return an implementation-defined String source code representation of func.
        //    The representation must have the syntax of a NativeFunction.
        // 5. Throw a TypeError exception.
        let Some(object) = func.as_callable() else {
            return Err(JsNativeError::typ().with_message("not a function").into());
        };

        if object.is::<NativeFunctionObject>() {
            let name = {
                // Is there a case here where if there is no name field on a value
                // name should default to None? Do all functions have names set?
                let value = object.get(js_string!("name"), context.access())?;
                if value.is_null_or_undefined() {
                    js_string!()
                } else {
                    value.to_string(context.access())?
                }
            };
            return Ok(
                js_string!(js_str!("function "), &name, js_str!("() { [native code] }")).into(),
            );
        } else if object.is::<Proxy>() || object.is::<BoundFunction>() {
            return Ok(js_string!("function () { [native code] }").into());
        }

        let function = object
            .downcast_ref::<OrdinaryFunction>()
            .ok_or_else(|| JsNativeError::typ().with_message("not a function"))?;

        let code = function.codeblock();
        if let Some(code_points) = code.source_info().text_spanned().to_code_points() {
            return Ok(JsString::from(code_points).into());
        }

        Ok(js_string!(
            js_str!("function "),
            code.name(),
            js_str!("() { [native code] }")
        )
        .into())
    }

    /// `Function.prototype [ @@hasInstance ] ( V )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype-@@hasinstance
    fn has_instance(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let F be the this value.
        // 2. Return ? OrdinaryHasInstance(F, V).
        Ok(
            JsValue::ordinary_has_instance(this, args.get_or_undefined(0), context.access())?
                .into(),
        )
    }

    #[allow(clippy::unnecessary_wraps)]
    fn prototype(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }
}

/// Abstract operation `SetFunctionName`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-setfunctionname
pub(crate) fn set_function_name(
    function: &JsObject,
    name: &PropertyKey,
    prefix: Option<JsStr<'_>>,
    context: &mut Context,
) -> JsResult<()> {
    // 1. Assert: F is an extensible object that does not have a "name" own property.
    // 2. If Type(name) is Symbol, then
    let mut name = match name {
        PropertyKey::Symbol(sym) => {
            // a. Let description be name's [[Description]] value.
            // b. If description is undefined, set name to the empty String.
            // c. Else, set name to the string-concatenation of "[", description, and "]".
            sym.description().map_or_else(
                || js_string!(),
                |desc| js_string!(js_str!("["), &desc, js_str!("]")),
            )
        }
        PropertyKey::String(string) => string.clone(),
        PropertyKey::Index(index) => js_string!(format!("{}", index.get())),
    };

    // 3. Else if name is a Private Name, then
    // a. Set name to name.[[Description]].
    // todo: implement Private Names

    // 4. If F has an [[InitialName]] internal slot, then
    // a. Set F.[[InitialName]] to name.
    // todo: implement [[InitialName]] for builtins

    // 5. If prefix is present, then
    if let Some(prefix) = prefix {
        name = js_string!(prefix, js_str!(" "), &name);
        // b. If F has an [[InitialName]] internal slot, then
        // i. Optionally, set F.[[InitialName]] to name.
        // todo: implement [[InitialName]] for builtins
    }

    // 6. Return ! DefinePropertyOrThrow(F, "name", PropertyDescriptor { [[Value]]: name,
    // [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }).
    function
        .define_property_or_throw(
            js_string!("name"),
            PropertyDescriptor::builder()
                .value(name)
                .writable(false)
                .enumerable(false)
                .configurable(true),
            context.access(),
        )
        .js_expect("defining the `name` property must not fail per the spec")?;

    Ok(())
}

/// Checks the actual ordinary target before deferred callback arguments are exposed.
pub(crate) fn reserve_prepared_call_entry(
    function_object: &JsObject,
    argument_count: usize,
    context: &mut Context,
) -> JsResult<FirstDispatchPermit> {
    let function = function_object
        .downcast_ref::<OrdinaryFunction>()
        .js_expect("not a function")?;
    context.prepare_guest_call_storage(argument_count, function.code.register_count)?;
    let realm = context.realm().resolve_call_realm(Some(&function.realm));
    let code = ordinary_call_code(&function, realm)?;
    drop(function);
    ActivatedCode::from_installed(code.clone())?;
    context.reserve_guest_entry(code, 0)
}

fn ordinary_call_code(function: &OrdinaryFunction, realm: Realm) -> JsResult<Rc<CodeBlock>> {
    if function.code.is_class_constructor() {
        debug_assert!(
            function.is_ordinary(),
            "only ordinary functions can be classes"
        );
        return Err(JsNativeError::typ()
            .with_message("class constructor cannot be invoked without 'new'")
            .with_realm(realm)
            .into());
    }
    Ok(function.code.clone())
}

/// Call this object.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-prepareforordinarycall>
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
pub(crate) fn function_call(
    function_object: &JsObject,
    argument_count: usize,
    #[allow(
        unused_variables,
        reason = "Only used if native-backtrace feature is enabled"
    )]
    context: &mut InternalMethodCallContext<'_>,
) -> JsResult<CallValue> {
    let function = function_object
        .downcast_ref::<OrdinaryFunction>()
        .js_expect("not a function")?;
    let realm = context.realm().resolve_call_realm(Some(&function.realm));

    let raw_code = function.code.clone();
    let entry = context.take_entry_permit(&raw_code)?;
    if entry.is_none() {
        context.check_runtime_limits()?;
    }
    let code = ordinary_call_code(&function, realm.clone())?;
    let environments = function.environments.to_stack()?;
    let script_or_module = function
        .script_or_module
        .as_ref()
        .map(StoredActiveRunnable::to_runnable)
        .transpose()?;

    drop(function);

    let env_fp = environments.len() as u32;

    let permit = match entry {
        Some(permit) => permit,
        None => context.reserve_guest_entry(code.clone(), 0)?,
    };
    let code = ActivatedCode::from_installed(code)?;
    let frame = CallFrame::new(code, permit, script_or_module, environments, realm)?
        .with_argument_count(argument_count as u32)
        .with_env_fp(env_fp);

    #[cfg(feature = "native-backtrace")]
    {
        let native_source_info = context.native_source_info();
        context
            .vm
            .shadow_stack
            .patch_last_native(native_source_info);
    }

    context.vm.push_frame(frame);

    let context = context.context();

    let lexical_this_mode = context.vm.frame().code_block.this_mode == ThisMode::Lexical;
    let this = if lexical_this_mode {
        ThisBindingStatus::Lexical
    } else {
        let this = context.vm.stack.get_this(context.vm.frame());
        if context.vm.frame().code_block.strict() {
            context.vm.frame_mut().flags |= CallFrameFlags::THIS_VALUE_CACHED;
            ThisBindingStatus::initialized(this)
        } else if this.is_null_or_undefined() {
            context.vm.frame_mut().flags |= CallFrameFlags::THIS_VALUE_CACHED;
            let this: JsValue = context.realm().global_this().into();
            context.vm.stack.set_this(
                context.vm.frames.last().js_expect("frame must exist")?,
                this.clone(),
            );
            ThisBindingStatus::initialized(this)
        } else {
            let this: JsValue = this
                .to_object(context.access())
                .js_expect("conversion cannot fail")?
                .into();
            context.vm.frame_mut().flags |= CallFrameFlags::THIS_VALUE_CACHED;
            context.vm.stack.set_this(
                context.vm.frames.last().js_expect("frame must exist")?,
                this.clone(),
            );
            ThisBindingStatus::initialized(this)
        }
    };

    let mut last_env = 0;

    let has_binding_identifier = context.vm.frame().code_block().has_binding_identifier();
    let has_function_scope = context.vm.frame().code_block().has_function_scope();

    if has_binding_identifier {
        let frame = context.vm.frame_mut();
        let global = frame.realm.environment();
        let index = frame.environments.push_lexical(1, &global);
        frame.environments.put_lexical_value(
            BindingLocatorScope::Stack(index),
            0,
            function_object.clone().into(),
            &global,
        );
        last_env += 1;
    }

    if has_function_scope {
        let scope = context.vm.frame().constant_scope(last_env)?;
        let frame = context.vm.frame_mut();
        let global = frame.realm.environment();
        frame.environments.push_function(
            scope,
            FunctionSlots::new(this, function_object.clone(), None),
            &global,
        );
    }

    Ok(CallValue::Ready)
}

/// Construct an instance of this object with the specified arguments.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
fn function_construct(
    this_function_object: &JsObject,
    argument_count: usize,
    context: &mut InternalMethodCallContext<'_>,
) -> JsResult<CallValue> {
    let function = this_function_object
        .downcast_ref::<OrdinaryFunction>()
        .js_expect("not a function")?;
    let realm = context.realm().resolve_call_realm(Some(&function.realm));

    debug_assert!(
        function.is_ordinary(),
        "only ordinary functions can be constructed"
    );

    let code = function.code.clone();
    let entry = context.take_entry_permit(&code)?;
    if entry.is_none() {
        context.check_runtime_limits()?;
    }
    let environments = function.environments.to_stack()?;
    let script_or_module = function
        .script_or_module
        .as_ref()
        .map(StoredActiveRunnable::to_runnable)
        .transpose()?;
    drop(function);

    let env_fp = environments.len() as u32;

    let new_target = context.vm.stack.pop();

    let this = if code.is_derived_constructor() {
        None
    } else {
        // If the prototype of the constructor is not an object, then use the default object
        // prototype as prototype for the new object
        // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
        // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
        let prototype =
            get_prototype_from_constructor(&new_target, StandardConstructors::object, context)?;
        let this = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrdinaryObject,
        )
        .upcast();

        this.initialize_instance_elements(this_function_object, context)?;

        Some(this)
    };

    let permit = match entry {
        Some(permit) => permit,
        None => context.reserve_guest_entry(code.clone(), 0)?,
    };
    let code = ActivatedCode::from_installed(code)?;
    let mut frame = CallFrame::new(code, permit, script_or_module, environments, realm)?
        .with_argument_count(argument_count as u32)
        .with_env_fp(env_fp)
        .with_flags(CallFrameFlags::CONSTRUCT);

    // We push the `this` below so we can mark this function as having the this value
    // cached if it's initialized.
    frame
        .flags
        .set(CallFrameFlags::THIS_VALUE_CACHED, this.is_some());

    #[cfg(feature = "native-backtrace")]
    {
        let native_source_info = context.native_source_info();
        context
            .vm
            .shadow_stack
            .patch_last_native(native_source_info);
    }

    context.vm.push_frame(frame);

    let mut last_env = 0;

    let has_binding_identifier = context.vm.frame().code_block().has_binding_identifier();
    let has_function_scope = context.vm.frame().code_block().has_function_scope();

    if has_binding_identifier {
        let frame = context.vm.frame_mut();
        let global = frame.realm.environment();
        let index = frame.environments.push_lexical(1, &global);
        frame.environments.put_lexical_value(
            BindingLocatorScope::Stack(index),
            0,
            this_function_object.clone().into(),
            &global,
        );
        last_env += 1;
    }

    if has_function_scope {
        let scope = context.vm.frame().constant_scope(last_env)?;
        let frame = context.vm.frame_mut();
        let global = frame.realm.environment();
        frame.environments.push_function(
            scope,
            FunctionSlots::new(
                this.clone().map_or(ThisBindingStatus::Uninitialized, |o| {
                    ThisBindingStatus::initialized(o.into())
                }),
                this_function_object.clone(),
                Some(
                    new_target
                        .as_object()
                        .js_expect("new.target should be an object")?
                        .clone(),
                ),
            ),
            &global,
        );
    }

    let context = context.context();
    context.vm.stack.set_this(
        context.vm.frames.last().js_expect("frame must exist")?,
        this.map(JsValue::new).unwrap_or_default(),
    );

    Ok(CallValue::Ready)
}
