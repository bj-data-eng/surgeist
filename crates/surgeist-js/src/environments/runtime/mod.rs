use std::rc::Rc;

use crate::{
    Context, JsResult, JsString, JsSymbol, JsValue,
    object::{JsObject, ObjectEdge, PrivateName},
    value::StoredValue,
};
use surgeist_js_ast::scope::{
    BindingLocator, BindingLocatorScope, Scope, ScopeAccess, ScopeAnalysisError, ScopeHandle,
    ScopeStateError,
};
use surgeist_js_gc::heap::{Heap, RecordRef, Rooted, RuntimeHeap, Trace};
use thin_vec::ThinVec;

mod declarative;
mod eval;
mod private;

pub(crate) use eval::{EvalBindingCounts, PreparedEvalEnvironment};

use self::declarative::ModuleEnvironment;
pub(crate) use self::{
    declarative::{
        DeclarativeEnvironment, DeclarativeEnvironmentEdge, DeclarativeEnvironmentKind,
        DeclarativeEnvironmentRef, FunctionEnvironment, FunctionSlots, LexicalEnvironment,
        ThisBindingStatus,
    },
    private::PrivateEnvironment,
};

/// A persistent chain node. Its outgoing references are traced from the
/// rooted VM stack or from the record that owns a stored stack.
#[derive(Debug, Trace)]
struct EnvironmentNode {
    env: StoredEnvironment,
    parent: Option<RecordRef<EnvironmentNode>>,
}

impl EnvironmentNode {
    fn ancestor<'a>(&'a self, heap: &'a Heap, steps: usize) -> Option<&'a Self> {
        let mut node = self;
        for _ in 0..steps {
            let parent = node.parent.as_ref()?;
            node = heap.get(parent.handle()).expect("live environment parent");
        }
        Some(node)
    }
}

#[derive(Clone, Debug, Trace)]
enum StoredEnvironment {
    Declarative(DeclarativeEnvironmentEdge),
    Object(ObjectEdge),
}

impl StoredEnvironment {
    fn borrowed_declarative<'a>(&self, heap: &'a Heap) -> Option<&'a DeclarativeEnvironment> {
        match self {
            Self::Declarative(env) => Some(heap.get(env.handle()).expect("live environment")),
            Self::Object(_) => None,
        }
    }

    fn declarative(&self) -> Option<DeclarativeEnvironmentRef> {
        match self {
            Self::Declarative(env) => Some(root(env)),
            Self::Object(_) => None,
        }
    }

    fn to_environment(&self) -> Environment {
        match self {
            Self::Declarative(env) => Environment::Declarative(root(env)),
            Self::Object(object) => Environment::Object(JsObject::from_inner(object.clone())),
        }
    }
}

fn root<T: Trace>(edge: &RecordRef<T>) -> Rooted<T> {
    edge.upgrade()
        .expect("environment heap is available")
        .expect("live owner retains its environment")
}

/// The live runtime environment stack. Cloning roots the same persistent
/// chain tip, so pushing or truncating leaves other snapshots unchanged.
///
/// The global declarative environment lives in the frame's realm.
#[derive(Clone, Debug)]
pub(crate) struct EnvironmentStack {
    tip: Option<Rooted<EnvironmentNode>>,
    depth: u32,
    private_stack: ThinVec<Rc<PrivateEnvironment>>,
}

/// A captured environment stack stored inside a heap record.
#[derive(Clone, Debug, Trace)]
pub(crate) struct StoredEnvironmentStack {
    tip: Option<RecordRef<EnvironmentNode>>,
    depth: u32,
    #[heap_trace(skip)]
    private_stack: ThinVec<Rc<PrivateEnvironment>>,
}

/// Private runtime storage whose module scope has not been published.
/// Captures can feed only non-executable function recipes. The module owner
/// exposes this chain only after installing its exact prepared scope group.
pub(crate) struct PreparedModuleEnvironment {
    scope: ScopeHandle,
    stack: EnvironmentStack,
    environment: DeclarativeEnvironmentRef,
}

/// A private capture qualified for one immutable compiled function.
pub(crate) struct PreparedFunctionCapture {
    code: Rc<crate::vm::CodeBlock>,
    environments: StoredEnvironmentStack,
}

impl PreparedModuleEnvironment {
    pub(crate) fn new(scope: ScopeHandle, bindings: u32, heap: &RuntimeHeap) -> JsResult<Self> {
        let module = ModuleEnvironment::prepare(bindings).map_err(|_| {
            crate::error::EngineError::Resource(crate::error::EngineResourceError::Allocation)
        })?;
        let mut stack = EnvironmentStack::new();
        stack.push_declarative(
            DeclarativeEnvironmentKind::Module(module),
            false,
            false,
            heap,
        );
        let environment = stack
            .tip
            .as_ref()
            .expect("private module chain has one node")
            .env
            .declarative()
            .expect("private module chain has a declarative environment");
        Ok(Self {
            scope,
            stack,
            environment,
        })
    }

    pub(crate) fn capture_for<E>(
        &self,
        code: &Rc<crate::vm::CodeBlock>,
        access: &mut ScopeAccess<'_, E>,
    ) -> Result<PreparedFunctionCapture, ScopeAnalysisError<E>> {
        // A pending environment may not qualify already-executable code, or
        // code from a different preparation. Admission checks happen here,
        // before storage construction and group publication.
        access.scope_index(&self.scope)?;
        access.scope_index(&code.scope_association)?;
        if self.scope.installed_scopes().is_ok()
            || code.scope_association.installed_scopes().is_ok()
        {
            return Err(ScopeAnalysisError::State(
                ScopeStateError::AssociationMismatch,
            ));
        }
        Ok(PreparedFunctionCapture {
            code: code.clone(),
            environments: self.stack.stored(),
        })
    }

    pub(crate) fn activate(self, activation: &crate::module::ModuleActivation) -> EnvironmentStack {
        // The private publication owner validated every artifact under this
        // exact association before minting its activation capability. No
        // user-provided code, callback or new admission occurs in this step.
        let scope = self
            .scope
            .resolve_live(activation.installed_scopes())
            .expect("module group activation installs each prevalidated environment scope");
        self.environment
            .kind()
            .as_module()
            .expect("private module environment retains its kind")
            .install_scope(scope);
        self.stack
    }

    pub(crate) fn validate_publication(
        &self,
        publication: &surgeist_js_ast::scope::PublicationPermit<'_>,
    ) -> Result<(), ScopeStateError> {
        publication.authorize_handle(&self.scope)
    }
}

impl PreparedFunctionCapture {
    pub(crate) fn stored_for(
        &self,
        code: &Rc<crate::vm::CodeBlock>,
    ) -> JsResult<StoredEnvironmentStack> {
        if !Rc::ptr_eq(code, &self.code) {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        Ok(self.environments.clone())
    }
}

impl StoredEnvironmentStack {
    pub(crate) fn to_stack(&self) -> Result<EnvironmentStack, crate::error::RootingError> {
        use crate::error::RootingError;

        let tip = self
            .tip
            .as_ref()
            .map(|edge| {
                edge.upgrade()
                    .map_err(RootingError::Access)?
                    .ok_or(RootingError::Expired)
            })
            .transpose()?;
        Ok(EnvironmentStack {
            tip,
            depth: self.depth,
            private_stack: self.private_stack.clone(),
        })
    }

    pub(crate) fn push_private(&mut self, environment: Rc<PrivateEnvironment>) {
        self.private_stack.push(environment);
    }
}

/// Rooted access to a runtime environment.
#[derive(Debug)]
pub(crate) enum Environment {
    Declarative(DeclarativeEnvironmentRef),
    Object(JsObject),
}

enum StackBindingRead {
    Resolve,
    Found(Option<StoredValue>),
}

impl Environment {
    /// Returns the declarative environment if it is one.
    pub(crate) fn into_declarative(self) -> Option<DeclarativeEnvironmentRef> {
        match self {
            Self::Declarative(env) => Some(env),
            Self::Object(_) => None,
        }
    }
}

impl EnvironmentStack {
    /// Create a new environment stack.
    pub(crate) fn new() -> Self {
        Self {
            tip: None,
            depth: 0,
            private_stack: ThinVec::new(),
        }
    }

    /// Gets the next outer function environment.
    pub(crate) fn outer_function_environment(&self) -> Option<(DeclarativeEnvironmentRef, Scope)> {
        for node in self.iter_from_tip() {
            if let Some(decl) = node.env.declarative()
                && let Some(function_env) = decl.kind().as_function()
            {
                let scope = function_env.compile().clone();
                return Some((decl, scope));
            }
        }
        None
    }

    /// Get the number of current environments (not counting global).
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.depth as usize
    }

    /// Get the environment at the given absolute index (0-based from root).
    ///
    /// Index 0 is the deepest (first pushed) environment in the chain.
    /// Index `len() - 1` is the tip (most recently pushed).
    #[inline]
    pub(crate) fn get(&self, index: usize) -> Option<Environment> {
        let depth = self.len();
        if index >= depth {
            return None;
        }
        let steps = depth - 1 - index;
        let tip = self.tip.as_ref()?;
        if steps == 0 {
            return Some(tip.env.to_environment());
        }
        let environment = tip
            .runtime()
            .read(|heap| Some(tip.ancestor(heap, steps)?.env.clone()))
            .expect("environment heap is available")?;
        Some(environment.to_environment())
    }

    /// Reads a known stack binding without rooting intermediate environments.
    /// Object environments, dynamic resolution and module imports use the general path.
    fn read_clean_binding(&self, index: u32, binding: u32) -> StackBindingRead {
        let depth = self.len();
        let index = index as usize;
        if index >= depth {
            return StackBindingRead::Resolve;
        }
        let steps = depth - 1 - index;
        let tip = self.tip.as_ref().expect("nonempty environment chain");
        tip.runtime()
            .read(|heap| {
                let Some(current) = tip.env.borrowed_declarative(heap) else {
                    return StackBindingRead::Resolve;
                };
                if current.with() || current.poisoned() {
                    return StackBindingRead::Resolve;
                }
                let target = if steps == 0 {
                    current
                } else {
                    let Some(target) = tip
                        .ancestor(heap, steps)
                        .expect("live environment chain")
                        .env
                        .borrowed_declarative(heap)
                    else {
                        return StackBindingRead::Resolve;
                    };
                    target
                };
                match target.kind() {
                    DeclarativeEnvironmentKind::Lexical(env) => {
                        StackBindingRead::Found(env.get_stored(binding))
                    }
                    DeclarativeEnvironmentKind::Function(env)
                        if !env.is_deleted_binding(binding) =>
                    {
                        StackBindingRead::Found(env.get_stored(binding))
                    }
                    _ => StackBindingRead::Resolve,
                }
            })
            .expect("environment heap is available")
    }

    /// Iterate from tip (most recent) toward root (oldest).
    fn iter_from_tip(&self) -> impl Iterator<Item = Rooted<EnvironmentNode>> {
        std::iter::successors(self.tip.clone(), |node| node.parent.as_ref().map(root))
    }

    /// Truncate current environments to the given depth.
    pub(crate) fn truncate(&mut self, len: usize) {
        while self.depth as usize > len {
            self.pop();
        }
    }

    /// `GetThisEnvironment`
    ///
    /// Returns the environment that currently provides a `this` binding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getthisenvironment
    pub(crate) fn get_this_environment(
        &self,
        global: DeclarativeEnvironmentRef,
    ) -> DeclarativeEnvironmentRef {
        for node in self.iter_from_tip() {
            if let Some(decl) = node
                .env
                .declarative()
                .filter(|decl| decl.has_this_binding())
            {
                return decl;
            }
        }

        global
    }

    /// `GetThisBinding`
    ///
    /// Returns the current `this` binding of the environment.
    /// Note: If the current environment is the global environment, this function returns `Ok(None)`.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    pub(crate) fn get_this_binding(&self) -> JsResult<Option<JsValue>> {
        for node in self.iter_from_tip() {
            if let Some(decl) = node.env.declarative()
                && let Some(this) = decl.get_this_binding()?
            {
                return Ok(Some(this));
            }
        }

        Ok(None)
    }

    /// Push a new object environment on the environments stack.
    pub(crate) fn push_object(&mut self, object: JsObject, heap: &RuntimeHeap) {
        self.push_env(StoredEnvironment::Object(object.edge()), heap);
    }

    /// Push a lexical environment on the environments stack and return its index.
    pub(crate) fn push_lexical(
        &mut self,
        bindings_count: u32,
        global: &DeclarativeEnvironmentRef,
    ) -> u32 {
        let (poisoned, with) = self.compute_poisoned_with(global);

        let index = self.depth;

        self.push_declarative(
            DeclarativeEnvironmentKind::Lexical(LexicalEnvironment::new(bindings_count)),
            poisoned,
            with,
            global.runtime(),
        );

        index
    }

    /// Push a function environment on the environments stack.
    pub(crate) fn push_function(
        &mut self,
        scope: Scope,
        function_slots: FunctionSlots,
        global: &DeclarativeEnvironmentRef,
    ) {
        let num_bindings = scope.num_bindings_non_local();

        let (poisoned, with) = self.compute_poisoned_with(global);

        self.push_declarative(
            DeclarativeEnvironmentKind::Function(FunctionEnvironment::new(
                num_bindings,
                function_slots,
                scope,
            )),
            poisoned,
            with,
            global.runtime(),
        );
    }

    /// Pop environment from the environments stack.
    #[track_caller]
    pub(crate) fn pop(&mut self) {
        let node = self
            .tip
            .as_ref()
            .expect("cannot pop empty environment chain");
        self.tip = node.parent.as_ref().map(root);
        self.depth -= 1;
    }

    /// Get the most outer environment.
    pub(crate) fn current_declarative_ref(
        &self,
        global: DeclarativeEnvironmentRef,
    ) -> Option<DeclarativeEnvironmentRef> {
        if let Some(node) = self.tip.as_ref() {
            node.env.declarative()
        } else {
            Some(global)
        }
    }

    /// Mark that there may be added bindings from the current environment to the next function
    /// environment.
    pub(crate) fn poison_until_last_function(&mut self, global: &DeclarativeEnvironmentRef) {
        for node in self.iter_from_tip() {
            if let Some(decl) = node.env.declarative() {
                decl.poison();
                if decl.is_function() {
                    return;
                }
            }
        }
        global.poison();
    }

    /// Set the value of a lexical binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn put_lexical_value(
        &mut self,
        environment: BindingLocatorScope,
        binding_index: u32,
        value: JsValue,
        global: &DeclarativeEnvironmentRef,
    ) {
        let env = match environment {
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => {
                global.clone()
            }
            BindingLocatorScope::Stack(index) => self
                .get(index as usize)
                .and_then(Environment::into_declarative)
                .expect("must be declarative environment"),
        };
        env.set(binding_index, value);
    }

    /// Set the value of a binding if it is uninitialized.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn put_value_if_uninitialized(
        &mut self,
        environment: BindingLocatorScope,
        binding_index: u32,
        value: JsValue,
        global: &DeclarativeEnvironmentRef,
    ) {
        let env = match environment {
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => {
                global.clone()
            }
            BindingLocatorScope::Stack(index) => self
                .get(index as usize)
                .and_then(Environment::into_declarative)
                .expect("must be declarative environment"),
        };
        if env.get(binding_index).is_none() {
            env.set(binding_index, value);
        }
    }

    /// Push a private environment to the private environment stack.
    pub(crate) fn push_private(&mut self, environment: Rc<PrivateEnvironment>) {
        self.private_stack.push(environment);
    }

    /// Pop a private environment from the private environment stack.
    pub(crate) fn pop_private(&mut self) {
        self.private_stack.pop();
    }

    /// `ResolvePrivateIdentifier ( privEnv, identifier )`
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-resolve-private-identifier
    pub(crate) fn resolve_private_identifier(&self, identifier: JsString) -> Option<PrivateName> {
        for environment in self.private_stack.iter().rev() {
            if environment.descriptions().contains(&identifier) {
                return Some(PrivateName::new(identifier, environment.id()));
            }
        }
        None
    }

    /// Visit private environments in name resolution order.
    pub(crate) fn private_environments(&self) -> impl Iterator<Item = &PrivateEnvironment> {
        self.private_stack.iter().rev().map(Rc::as_ref)
    }

    /// Check each environment kind and the terminating empty link only after
    /// admission. The callback runs without an interior environment borrow.
    pub(crate) fn try_has_object_environment<E>(
        &self,
        mut admit: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let mut current = self.tip.clone();
        loop {
            admit()?;
            let Some(node) = current else {
                return Ok(false);
            };
            if matches!(node.env, StoredEnvironment::Object(_)) {
                return Ok(true);
            }
            current = node.parent.as_ref().map(root);
        }
    }

    pub(crate) fn stored(&self) -> StoredEnvironmentStack {
        StoredEnvironmentStack {
            tip: self.tip.as_ref().map(Rooted::record_ref),
            depth: self.depth,
            private_stack: self.private_stack.clone(),
        }
    }

    fn push_declarative(
        &mut self,
        kind: DeclarativeEnvironmentKind,
        poisoned: bool,
        with: bool,
        heap: &RuntimeHeap,
    ) {
        let environment = heap
            .allocate(DeclarativeEnvironment::new(kind, poisoned, with))
            .expect("new declarative environment belongs to the active runtime");
        self.push_env(
            StoredEnvironment::Declarative(environment.record_ref()),
            heap,
        );
    }

    fn push_env(&mut self, env: StoredEnvironment, heap: &RuntimeHeap) {
        let node = EnvironmentNode {
            env,
            parent: self.tip.as_ref().map(Rooted::record_ref),
        };
        self.tip = Some(
            heap.allocate(node)
                .expect("new environment chain node belongs to the active runtime"),
        );
        self.depth += 1;
    }

    /// Compute the `(poisoned, with)` flags for a new environment.
    fn compute_poisoned_with(&self, global: &DeclarativeEnvironmentRef) -> (bool, bool) {
        let with = self
            .tip
            .as_ref()
            .is_some_and(|node| matches!(node.env, StoredEnvironment::Object(_)));

        let environment = self
            .iter_from_tip()
            .find_map(|node| node.env.declarative())
            .unwrap_or_else(|| global.clone());
        (environment.poisoned(), with || environment.with())
    }
}

impl Context {
    pub(crate) fn read_name_binding(
        &mut self,
        locator: &mut BindingLocator,
    ) -> JsResult<Option<JsValue>> {
        if let BindingLocatorScope::Stack(index) = locator.scope()
            && let StackBindingRead::Found(stored) = self
                .vm
                .frame()
                .environments
                .read_clean_binding(index, locator.binding_index())
        {
            // Materializing a stored object may upgrade its heap edge.
            return Ok(stored.map(StoredValue::into_value));
        }
        self.find_runtime_binding(locator)?;
        self.get_binding(locator)
    }

    /// Gets the corresponding runtime binding of the provided `BindingLocator`, modifying
    /// its indexes in place.
    ///
    /// This readjusts a `BindingLocator` to the correct binding if a `with` environment or
    /// `eval` call modified the compile-time bindings.
    ///
    /// Only use if the binding origin is unknown or comes from a `var` declaration.
    /// Statically resolved stack lexical bindings do not need runtime checks. Global names
    /// still need to observe lexical bindings introduced by subsequently installed scripts.
    pub(crate) fn find_runtime_binding(&mut self, locator: &mut BindingLocator) -> JsResult<()> {
        let deleted_binding = self.is_deleted_binding(locator);

        let global = self.vm.frame().realm.environment();
        if let Some(env) = self.vm.frame().environments.current_declarative_ref(global)
            && !env.with()
            && !env.poisoned()
            && !deleted_binding
        {
            self.refresh_global_binding(locator);
            return Ok(());
        }

        let min_index = match locator.scope() {
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => 0,
            BindingLocatorScope::Stack(_) if deleted_binding => 0,
            BindingLocatorScope::Stack(index) => index,
        };
        let max_index = self.vm.frame().environments.len() as u32;

        for index in (min_index..max_index).rev() {
            match self.environment_expect(index) {
                Environment::Declarative(env) => {
                    if env.poisoned() || deleted_binding {
                        if let Some(env) = env.kind().as_function()
                            && let Some(b) = env.compile().get_binding(locator.name())
                        {
                            if env.is_deleted_binding(b.binding_index()) {
                                continue;
                            }
                            locator.set_scope(b.scope());
                            locator.set_binding_index(b.binding_index());
                            return Ok(());
                        }
                    } else if !env.with() {
                        self.refresh_global_binding(locator);
                        return Ok(());
                    }
                }
                Environment::Object(o) => {
                    let key = locator.name().clone();
                    if o.has_property(key.clone(), self.access())? {
                        if let Some(unscopables) =
                            o.get(JsSymbol::unscopables(), self.access())?.as_object()
                            && unscopables.get(key.clone(), self.access())?.to_boolean()
                        {
                            continue;
                        }
                        locator.set_scope(BindingLocatorScope::Stack(index));
                        return Ok(());
                    }
                }
            }
        }

        if deleted_binding
            && let BindingLocatorScope::Stack(index) = locator.scope()
            && let Environment::Declarative(env) = self.environment_expect(index)
            && let Some(function_env) = env.kind().as_function()
        {
            let mut scope = function_env.compile().outer();
            while let Some(current_scope) = scope {
                if let Some(binding) = current_scope.get_binding(locator.name()) {
                    locator.set_scope(binding.scope());
                    locator.set_binding_index(binding.binding_index());
                    return Ok(());
                }
                scope = current_scope.outer();
            }

            locator.set_scope(BindingLocatorScope::GlobalObject);
            locator.set_binding_index(0);
            return Ok(());
        }

        self.refresh_global_binding(locator);
        Ok(())
    }

    /// Refreshes global resolution after nearer runtime environments have been ruled out.
    /// Installing another script can add a lexical binding without poisoning an older
    /// captured environment, or changing a cached global object's property shape.
    fn refresh_global_binding(&self, locator: &mut BindingLocator) {
        if matches!(
            locator.scope(),
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative
        ) && let Some(binding) = self.realm().scope().get_binding(locator.name())
        {
            locator.set_scope(binding.scope());
            locator.set_binding_index(binding.binding_index());
        }
    }

    /// Finds the object environment that contains the binding and returns the `this` value of the object environment.
    pub(crate) fn this_from_object_environment_binding(
        &mut self,
        locator: &BindingLocator,
    ) -> JsResult<Option<JsObject>> {
        let global = self.vm.frame().realm.environment();
        if let Some(env) = self.vm.frame().environments.current_declarative_ref(global)
            && !env.with()
        {
            return Ok(None);
        }

        let min_index = match locator.scope() {
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => 0,
            BindingLocatorScope::Stack(index) => index,
        };
        let max_index = self.vm.frame().environments.len() as u32;

        for index in (min_index..max_index).rev() {
            match self.environment_expect(index) {
                Environment::Declarative(env) => {
                    if env.poisoned() {
                        if let Some(env) = env.kind().as_function()
                            && env.compile().get_binding(locator.name()).is_some()
                        {
                            break;
                        }
                    } else if !env.with() {
                        break;
                    }
                }
                Environment::Object(o) => {
                    let key = locator.name().clone();
                    if o.has_property(key.clone(), self.access())? {
                        if let Some(unscopables) =
                            o.get(JsSymbol::unscopables(), self.access())?.as_object()
                            && unscopables.get(key.clone(), self.access())?.to_boolean()
                        {
                            continue;
                        }
                        return Ok(Some(o));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Checks if the binding pointed by `locator` is initialized.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    pub(crate) fn is_initialized_binding(&mut self, locator: &BindingLocator) -> JsResult<bool> {
        match locator.scope() {
            BindingLocatorScope::GlobalObject => {
                let key = locator.name().clone();
                let obj = self.global_object()?;
                obj.has_property(key, self.access())
            }
            BindingLocatorScope::GlobalDeclarative => {
                let env = self.vm.frame().realm.environment();
                Ok(env.get(locator.binding_index()).is_some())
            }
            BindingLocatorScope::Stack(index) => match self.environment_expect(index) {
                Environment::Declarative(env) => Ok(env.get(locator.binding_index()).is_some()),
                Environment::Object(obj) => {
                    let key = locator.name().clone();
                    obj.has_property(key, self.access())
                }
            },
        }
    }

    pub(crate) fn is_deleted_binding(&self, locator: &BindingLocator) -> bool {
        match locator.scope() {
            BindingLocatorScope::Stack(index) => matches!(
                self.environment_expect(index),
                Environment::Declarative(env) if env.is_deleted_binding(locator.binding_index())
            ),
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => false,
        }
    }

    pub(crate) fn restore_deleted_binding(&self, locator: &BindingLocator) {
        if let BindingLocatorScope::Stack(index) = locator.scope()
            && let Environment::Declarative(env) = self.environment_expect(index)
        {
            env.restore_deleted_binding(locator.binding_index());
        }
    }

    /// Get the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn get_binding(&mut self, locator: &BindingLocator) -> JsResult<Option<JsValue>> {
        match locator.scope() {
            BindingLocatorScope::GlobalObject => {
                let key = locator.name().clone();
                let obj = self.global_object()?;
                obj.try_get(key, self)
            }
            BindingLocatorScope::GlobalDeclarative => {
                let env = self.vm.frame().realm.environment();
                Ok(env.get(locator.binding_index()))
            }
            BindingLocatorScope::Stack(index) => match self.environment_expect(index) {
                Environment::Declarative(env) => Ok(env.get(locator.binding_index())),
                Environment::Object(obj) => {
                    let key = locator.name().clone();
                    obj.get(key, self.access()).map(Some)
                }
            },
        }
    }

    /// Sets the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn set_binding(
        &mut self,
        locator: &BindingLocator,
        value: JsValue,
        strict: bool,
    ) -> JsResult<()> {
        match locator.scope() {
            BindingLocatorScope::GlobalObject => {
                let key = locator.name().clone();
                let obj = self.global_object()?;
                obj.set(key, value, strict, self.access())?;
            }
            BindingLocatorScope::GlobalDeclarative => {
                let env = self.vm.frame().realm.environment();
                env.set(locator.binding_index(), value);
            }
            BindingLocatorScope::Stack(index) => match self.environment_expect(index) {
                Environment::Declarative(decl) => {
                    decl.set(locator.binding_index(), value);
                }
                Environment::Object(obj) => {
                    let key = locator.name().clone();
                    obj.set(key, value, strict, self.access())?;
                }
            },
        }
        Ok(())
    }

    /// Deletes a binding if it exists.
    ///
    /// Returns `true` if the binding was deleted.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    pub(crate) fn delete_binding(&mut self, locator: &BindingLocator) -> JsResult<bool> {
        match locator.scope() {
            BindingLocatorScope::GlobalObject => {
                let key = locator.name().clone();
                let obj = self.global_object()?;
                obj.__delete__(&key.into(), &mut self.into())
            }
            BindingLocatorScope::GlobalDeclarative => Ok(false),
            BindingLocatorScope::Stack(index) => match self.environment_expect(index) {
                Environment::Declarative(env) => Ok(env.delete_binding(locator.binding_index())),
                Environment::Object(obj) => {
                    let key = locator.name().clone();
                    obj.__delete__(&key.into(), &mut self.into())
                }
            },
        }
    }

    /// Return the environment at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the `index` is out of range.
    pub(crate) fn environment_expect(&self, index: u32) -> Environment {
        self.vm
            .frame()
            .environments
            .get(index as usize)
            .expect("environment index must be in range")
    }
}

#[cfg(test)]
mod object_environment_tests {
    use super::*;
    use crate::Runtime;

    fn lexical_chain(length: usize) -> (Runtime, EnvironmentStack) {
        let runtime = Runtime::default();
        let global = runtime
            .heap
            .allocate(DeclarativeEnvironment::global())
            .unwrap();
        let mut stack = EnvironmentStack::new();
        for _ in 0..length {
            stack.push_lexical(0, &global);
        }
        (runtime, stack)
    }

    fn chain_identity(stack: &EnvironmentStack) -> Vec<RecordRef<EnvironmentNode>> {
        stack
            .iter_from_tip()
            .map(|node| node.record_ref())
            .collect()
    }

    #[test]
    fn no_object_search_admits_each_node_and_the_terminating_empty_link() {
        for length in [0, 1, 4] {
            let (_runtime, stack) = lexical_chain(length);
            let before = chain_identity(&stack);
            let mut visits = 0;
            assert!(
                !stack
                    .try_has_object_environment(|| {
                        visits += 1;
                        Ok::<_, ()>(())
                    })
                    .unwrap()
            );
            assert_eq!(visits, length + 1);
            assert_eq!(stack.len(), length);
            assert_eq!(chain_identity(&stack), before);
        }
    }

    #[test]
    fn first_object_stops_before_visiting_its_older_parent() {
        let runtime = Runtime::default();
        let global = runtime
            .heap
            .allocate(DeclarativeEnvironment::global())
            .unwrap();
        let mut stack = EnvironmentStack::new();
        let older = JsObject::with_null_proto(&runtime).unwrap();
        stack.push_object(older, &runtime.heap);
        stack.push_lexical(0, &global);
        let nearer = JsObject::with_null_proto(&runtime).unwrap();
        let nearer_identity = nearer.identity();
        stack.push_object(nearer, &runtime.heap);
        stack.push_lexical(0, &global);
        let before = chain_identity(&stack);
        let mut visits = 0;
        assert!(
            stack
                .try_has_object_environment(|| {
                    visits += 1;
                    assert!(visits <= 2, "search continued beyond its first object");
                    Ok::<_, ()>(())
                })
                .unwrap()
        );
        assert_eq!(visits, 2);
        assert_eq!(stack.len(), 4);
        assert_eq!(chain_identity(&stack), before);
        let Environment::Object(found) = stack.get(2).unwrap() else {
            panic!("the nearer object environment was replaced");
        };
        assert_eq!(found.identity(), nearer_identity);
    }

    #[test]
    fn refusal_preserves_the_accepted_prefix_identity_and_entire_chain() {
        let (_runtime, stack) = lexical_chain(3);
        let before = chain_identity(&stack);
        let marker = Rc::new("environment admission refused");
        for accepted_prefix in 0..=3 {
            let mut accepted = 0;
            let mut refused = false;
            let error = stack
                .try_has_object_environment(|| {
                    assert!(!refused, "environment search continued after refusal");
                    if accepted == accepted_prefix {
                        refused = true;
                        Err(marker.clone())
                    } else {
                        accepted += 1;
                        Ok(())
                    }
                })
                .unwrap_err();
            assert!(Rc::ptr_eq(&error, &marker));
            assert_eq!(accepted, accepted_prefix);
            assert_eq!(stack.len(), 3);
            assert_eq!(chain_identity(&stack), before);
        }
        let mut visits = 0;
        assert!(
            !stack
                .try_has_object_environment(|| {
                    visits += 1;
                    Ok::<_, ()>(())
                })
                .unwrap()
        );
        assert_eq!(visits, 4);
    }
}
