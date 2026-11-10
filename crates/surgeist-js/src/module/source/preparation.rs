//! One private preparation owner for a complete module-link attempt.

use super::*;
use crate::{
    SpannedSourceText,
    bytecompiler::{
        BindingAccessOpcode, ByteCompiler, FunctionSpec,
        control::{CompilerWork, payload_bytes, reserve, string_error},
    },
    environments::EnvironmentStack,
};
use surgeist_js_ast::{
    operations::{
        LexicallyScopedDeclaration, VarScopedDeclaration, try_bound_names,
        try_lexically_scoped_declarations, try_var_scoped_declarations,
    },
    scope::{BindingLocator, InstalledScopes, PreparedScopes, ScopeHandle},
};

/// Authority minted only after this complete link attempt publishes its scope group.
pub(crate) struct ModuleActivation {
    scopes: InstalledScopes,
}

impl ModuleActivation {
    pub(crate) fn installed_scopes(&self) -> &InstalledScopes {
        &self.scopes
    }
}

struct SourceUnit {
    module: Module,
    source: Option<ModuleSource>,
    ancestor_index: usize,
}

struct LinkAttempt {
    identity: Rc<()>,
    stack: Vec<Module>,
    pending: Vec<Module>,
    sources: Vec<SourceUnit>,
}

pub(super) fn link(module: &Module, context: &mut Context) -> JsResult<()> {
    context.check_execution_stop()?;
    let mut attempt = LinkAttempt {
        identity: Rc::new(()),
        stack: Vec::new(),
        pending: Vec::new(),
        sources: Vec::new(),
    };
    attempt.visit(module, 0, context)?;
    attempt.prepare_and_install(context)
}

fn already_preparing() -> JsError {
    JsNativeError::typ()
        .with_message("module linking is already in progress")
        .into()
}

impl LinkAttempt {
    fn prepare_and_install(&mut self, context: &mut Context) -> JsResult<()> {
        let Some(first) = self.pending.first() else {
            return Ok(());
        };
        context.check_execution_stop()?;
        let mut scopes = PreparedScopes::new(first.realm().scope());
        self.take_sources();
        let mut ready = Vec::new();
        ready
            .try_reserve(self.pending.len())
            .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
        for module in &self.pending {
            let (compiled, exports, ancestor_index) = match module.kind() {
                ModuleKind::SourceText(_) => {
                    let unit = self
                        .sources
                        .iter_mut()
                        .find(|unit| unit.module == *module)
                        .expect("the group retains its source syntax");
                    let compiled = compile_source(
                        module,
                        unit.source
                            .as_mut()
                            .expect("source remains owned until publication"),
                        &mut scopes,
                        context,
                    )?;
                    (compiled, Vec::new(), unit.ancestor_index)
                }
                ModuleKind::Synthetic(record) => {
                    let compiled = record.prepare_link(module, &mut scopes, context)?;
                    (
                        SourceCode {
                            scope: compiled.scope,
                            code: compiled.code,
                            functions: Vec::new(),
                            imports: Vec::new(),
                        },
                        compiled.exports,
                        0,
                    )
                }
            };
            ready.push(ReadyModule::prepare(
                module.clone(),
                compiled,
                exports,
                ancestor_index,
                &mut scopes,
                context,
            )?);
        }
        let mut namespaces = Vec::<NamespacePreparation>::new();
        for module in &ready {
            for import in &module.imports {
                let ImportBinding::Namespace { module, .. } = import else {
                    continue;
                };
                if !namespaces
                    .iter()
                    .any(|namespace| namespace.module == *module)
                {
                    namespaces.push(NamespacePreparation::new(module, context)?);
                }
            }
        }
        let permit = context.with_installation_work(|admission| {
            scopes.prepare_publication(admission).map_err(JsError::from)
        })?;
        context.check_execution_stop()?;
        for module in &ready {
            if let Err(failure) = module.validate_publication(&permit) {
                let mut error = JsError::from(failure);
                context.retain_preparation_error(&mut error);
                return Err(error);
            }
        }
        let installed = permit.commit().map_err(|failure| {
            let mut error = JsError::from(failure);
            context.retain_preparation_error(&mut error);
            error
        })?;
        let activation = ModuleActivation { scopes: installed };
        // Every fallible check and callback has finished. The token authorizes
        // only these exact group-owned, prevalidated environment and function recipes.
        for module in &mut ready {
            module.activate_environment(&activation);
        }
        for namespace in &mut namespaces {
            namespace.reconcile();
        }
        for module in &mut ready {
            module.initialize_bindings(&activation, &namespaces);
        }
        if ready.iter().any(|module| module.has_bindings) {
            context.note_execution_effect();
        }
        for namespace in namespaces {
            namespace.publish();
        }
        for module in ready {
            module.publish();
        }
        Ok(())
    }

    fn visit(
        &mut self,
        module: &Module,
        mut index: usize,
        context: &mut Context,
    ) -> JsResult<usize> {
        context.check_execution_stop()?;
        let ModuleKind::SourceText(record) = module.kind() else {
            let ModuleKind::Synthetic(record) = module.kind() else {
                unreachable!("module kind is closed");
            };
            if record.begin_link(&self.identity)? {
                self.pending.push(module.clone());
            }
            return Ok(index);
        };
        {
            let status = record.status.borrow();
            match &*status {
                ModuleStatus::Unlinked { .. } => {}
                ModuleStatus::Linking { attempt, .. } | ModuleStatus::Pending { attempt, .. }
                    if Rc::ptr_eq(attempt, &self.identity) =>
                {
                    return Ok(index);
                }
                ModuleStatus::Linked { .. }
                | ModuleStatus::EvaluatingAsync { .. }
                | ModuleStatus::Evaluated { .. } => return Ok(index),
                _ => return Err(already_preparing()),
            }
        }
        record.status.borrow_mut().transition(|state| match state {
            ModuleStatus::Unlinked { source } => ModuleStatus::Linking {
                source,
                ancestor_index: index,
                attempt: self.identity.clone(),
            },
            _ => unreachable!("owned discovery starts from unlinked syntax"),
        });
        let module_index = index;
        index += 1;
        self.stack.push(module.clone());
        for request in &record.code.requested_modules {
            let required = record
                .loaded_module(request)
                .expect("loading populated the requested module");
            index = self.visit(&required, index, context)?;
            if let ModuleKind::SourceText(required) = required.kind() {
                let required_index = match &*required.status.borrow() {
                    ModuleStatus::Linking { ancestor_index, .. } => Some(*ancestor_index),
                    _ => None,
                };
                if let Some(required_index) = required_index {
                    let mut status = record.status.borrow_mut();
                    let ancestor = status
                        .ancestor_index_mut()
                        .expect("the active DFS owns this linking module");
                    *ancestor = (*ancestor).min(required_index);
                }
            }
        }
        if record.status.borrow().ancestor_index() == Some(module_index) {
            loop {
                let member = self.stack.pop().expect("SCC root remains on the DFS stack");
                let ModuleKind::SourceText(member_record) = member.kind() else {
                    unreachable!("only source records enter the DFS stack");
                };
                member_record
                    .status
                    .borrow_mut()
                    .transition(|state| match state {
                        ModuleStatus::Linking {
                            source,
                            ancestor_index,
                            attempt,
                        } => ModuleStatus::Pending {
                            source,
                            ancestor_index,
                            attempt,
                        },
                        _ => unreachable!("SCC completion owns a linking module"),
                    });
                let done = member == *module;
                self.pending.push(member);
                if done {
                    break;
                }
            }
        }
        Ok(index)
    }

    fn take_sources(&mut self) {
        for module in &self.pending {
            let ModuleKind::SourceText(record) = module.kind() else {
                continue;
            };
            let owned = std::mem::replace(
                &mut *record.status.borrow_mut(),
                ModuleStatus::Preparing {
                    attempt: self.identity.clone(),
                },
            );
            let ModuleStatus::Pending {
                source,
                ancestor_index,
                ..
            } = owned
            else {
                unreachable!("the attempt owns all pending source syntax");
            };
            self.sources.push(SourceUnit {
                module: module.clone(),
                source: Some(source),
                ancestor_index,
            });
        }
    }
}

impl Drop for LinkAttempt {
    fn drop(&mut self) {
        for module in self.stack.iter().chain(&self.pending) {
            match module.kind() {
                ModuleKind::Synthetic(record) => record.abort_link(&self.identity),
                ModuleKind::SourceText(record) => {
                    let mut status = record.status.borrow_mut();
                    let owned = std::mem::replace(&mut *status, ModuleStatus::Transitioning);
                    *status = match owned {
                        ModuleStatus::Linking {
                            source, attempt, ..
                        }
                        | ModuleStatus::Pending {
                            source, attempt, ..
                        } if Rc::ptr_eq(&attempt, &self.identity) => {
                            ModuleStatus::Unlinked { source }
                        }
                        ModuleStatus::Preparing { attempt }
                            if Rc::ptr_eq(&attempt, &self.identity) =>
                        {
                            let unit = self
                                .sources
                                .iter_mut()
                                .find(|unit| unit.module == *module)
                                .expect("preparation retains its source owner");
                            ModuleStatus::Unlinked {
                                source: unit.source.take().expect("source is restored once"),
                            }
                        }
                        state => state,
                    };
                }
            }
        }
    }
}

enum ImportBinding {
    Namespace {
        locator: BindingLocator,
        module: Module,
    },
    Single {
        locator: BindingLocator,
        export_locator: ResolvedBinding,
    },
}

struct SourceCode {
    scope: ScopeHandle,
    code: Rc<CodeBlock>,
    functions: Vec<(u32, BindingLocator)>,
    imports: Vec<ImportBinding>,
}

enum ModuleEnvironmentState {
    Pending(crate::environments::PreparedModuleEnvironment),
    Active(EnvironmentStack),
    Transitioning,
}

struct ReadyModule {
    module: Module,
    environment: ModuleEnvironmentState,
    code: Rc<CodeBlock>,
    functions: Vec<(crate::vm::PreparedModuleFunction, BindingLocator)>,
    imports: Vec<ImportBinding>,
    exports: Vec<BindingLocator>,
    ancestor_index: usize,
    has_bindings: bool,
}

impl ReadyModule {
    fn validate_publication(
        &self,
        permit: &surgeist_js_ast::scope::PublicationPermit<'_>,
    ) -> Result<(), surgeist_js_ast::scope::ScopeStateError> {
        let ModuleEnvironmentState::Pending(environment) = &self.environment else {
            unreachable!("publication validation owns pending environments");
        };
        environment.validate_publication(permit)?;
        for (function, _) in &self.functions {
            function.validate_publication(permit)?;
        }
        Ok(())
    }

    fn prepare(
        module: Module,
        compiled: SourceCode,
        exports: Vec<BindingLocator>,
        ancestor_index: usize,
        scopes: &mut PreparedScopes,
        context: &mut Context,
    ) -> JsResult<Self> {
        let SourceCode {
            scope,
            code,
            functions,
            imports,
        } = compiled;
        let count = context.with_installation_work(|admission| {
            scopes.with_access(admission, |access| {
                access.associate(&code.scope_association)?;
                Ok::<_, JsError>(access.num_bindings(&scope)?)
            })
        })?;
        let runtime = context.runtime().clone();
        let environment = context.with_installation_work(|mut admission| {
            if let Some(admit) = admission.as_mut() {
                admit(surgeist_js_ast::scope::ScopeWork::Capacity {
                    elements: count as usize,
                })
                .map_err(crate::error::RuntimeLimitError::from)?;
            }
            crate::environments::PreparedModuleEnvironment::new(scope, count, &runtime.heap)
        })?;
        let mut recipes = Vec::new();
        recipes
            .try_reserve(functions.len())
            .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
        for (index, locator) in functions {
            let function_code = code.constant_function(index as usize);
            let capture = context.with_installation_work(|admission| {
                scopes.with_access(admission, |access| {
                    environment
                        .capture_for(&function_code, access)
                        .map_err(JsError::from)
                })
            })?;
            let mut previous_realm = module.realm();
            context.swap_realm(&mut previous_realm);
            let recipe = crate::vm::prepare_module_function_object_fast(
                function_code,
                &capture,
                Some(ActiveRunnable::Module(module.clone())),
                module.realm(),
                &mut context.guard(move |context| context.swap_realm(&mut previous_realm)),
            )?;
            recipes.push((recipe, locator));
        }
        Ok(Self {
            module,
            environment: ModuleEnvironmentState::Pending(environment),
            code,
            functions: recipes,
            imports,
            exports,
            ancestor_index,
            has_bindings: count != 0,
        })
    }

    fn activate_environment(&mut self, activation: &ModuleActivation) {
        let pending =
            std::mem::replace(&mut self.environment, ModuleEnvironmentState::Transitioning);
        let ModuleEnvironmentState::Pending(environment) = pending else {
            unreachable!("a ready module activates its private environment once");
        };
        self.environment = ModuleEnvironmentState::Active(environment.activate(activation));
    }

    fn initialize_bindings(
        &mut self,
        activation: &ModuleActivation,
        namespaces: &[NamespacePreparation],
    ) {
        let ModuleEnvironmentState::Active(environments) = &self.environment else {
            unreachable!("all group environments activate before bindings");
        };
        let environment = environments
            .current_declarative_ref(self.module.realm().environment())
            .expect("the prepared stack owns its module environment");
        for import in self.imports.drain(..) {
            match import {
                ImportBinding::Namespace { locator, module } => {
                    let namespace = namespaces
                        .iter()
                        .find(|namespace| namespace.module == module)
                        .expect("every imported namespace was prepared");
                    environment.set(locator.binding_index(), namespace.object.clone().into());
                }
                ImportBinding::Single {
                    locator,
                    export_locator,
                } => {
                    let BindingName::Name(name) = export_locator.binding_name() else {
                        unreachable!("namespace imports have a private namespace recipe");
                    };
                    environment
                        .kind()
                        .as_module()
                        .expect("prepared environment is a module environment")
                        .set_indirect(
                            locator.binding_index(),
                            export_locator.module().clone(),
                            name.clone(),
                        );
                }
            }
        }
        for (function, locator) in self.functions.drain(..) {
            environment.set(
                locator.binding_index(),
                function.activate(activation).into(),
            );
        }
        for export in &self.exports {
            environment.set(export.binding_index(), JsValue::undefined());
        }
    }

    fn publish(self) {
        let ModuleEnvironmentState::Active(environments) = self.environment else {
            unreachable!("group publication owns active private environments");
        };
        let realm = self.module.realm();
        let environment = environments
            .current_declarative_ref(realm.environment())
            .expect("the prepared stack owns its module environment");
        match self.module.kind() {
            ModuleKind::SourceText(record) => {
                *record.status.borrow_mut() = ModuleStatus::Linked {
                    environment: environment.record_ref(),
                    context: SourceTextContext {
                        codeblock: self.code,
                        environments: environments.stored(),
                        realm: realm.edge(),
                    },
                    ancestor_index: self.ancestor_index,
                };
            }
            ModuleKind::Synthetic(record) => {
                record.finish_link(environment, environments, self.code);
            }
        }
    }
}

struct NamespacePreparation {
    module: Module,
    object: JsObject,
}

impl NamespacePreparation {
    fn new(module: &Module, context: &mut Context) -> JsResult<Self> {
        let existing = module.inner.namespace.borrow().clone();
        let object = if let Some(existing) = existing {
            JsObject::from_inner(existing)
        } else {
            let names = module
                .get_exported_names(&mut Vec::new())
                .into_iter()
                .filter(|name| module.resolve_export(name, &mut HashSet::default()).is_ok())
                .collect();
            super::super::ModuleNamespace::create(module.clone(), names, context)?
        };
        Ok(Self {
            module: module.clone(),
            object,
        })
    }

    /// Preserve the identity of any namespace created by an admitted callback.
    fn reconcile(&mut self) {
        if let Some(existing) = &*self.module.inner.namespace.borrow() {
            self.object = JsObject::from_inner(existing.clone());
        }
    }

    fn publish(self) {
        let mut namespace = self.module.inner.namespace.borrow_mut();
        if namespace.is_none() {
            *namespace = Some(self.object.edge());
        }
    }
}

fn compile_source(
    module: &Module,
    source: &mut ModuleSource,
    scopes: &mut PreparedScopes,
    context: &mut Context,
) -> JsResult<SourceCode> {
    let ModuleKind::SourceText(record) = module.kind() else {
        unreachable!("source preparation owns a source module");
    };
    for entry in &record.code.indirect_export_entries {
        module
            .resolve_export(&entry.export_name, &mut HashSet::default())
            .map_err(|error| export_error(error, &entry.export_name))?;
    }
    context.with_scope_admission(|admission| {
        source
            .ast
            .try_prepare_in(module.realm().scope(), scopes, &source.interner, admission)
    })?;
    let scope = source.ast.scope().clone();
    let (result, extension) = context.with_compilation_work(|mut control| {
        let _depth = control.enter()?;
        control.admit(CompilerWork::MetadataVisit)?;
        let mut extension = control.with_scope_admission(|admission| {
            let owner = scopes;
            owner.begin_extension(admission).map_err(JsError::from)
        })?;
        let path = if let Some(path) = record.code.path.as_deref() {
            let bytes = payload_bytes::<u8>(path.as_os_str().len())?;
            control.admit(CompilerWork::Read { bytes })?;
            control.admit(CompilerWork::Capacity { bytes })?;
            control.admit(CompilerWork::Copy { bytes })?;
            crate::vm::source_info::SourcePath::Path(Rc::from(path))
        } else {
            crate::vm::source_info::SourcePath::None
        };
        let mut compiler = ByteCompiler::new(
            js_string!("<main>"),
            true,
            false,
            scope.clone(),
            scope.clone(),
            record.code.has_tla,
            false,
            &mut source.interner,
            false,
            SpannedSourceText::new_source_only(source.text.clone()),
            path,
            &mut extension,
            control.reborrow(),
        )?;
        if record.code.has_tla {
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.async_handler = Some(compiler.push_handler()?);
        }
        let mut imports = Vec::new();
        for entry in &record.code.import_entries {
            compiler.control.admit(CompilerWork::MetadataVisit)?;
            compiler.control.admit(CompilerWork::Read {
                bytes: payload_bytes::<super::ImportEntry>(1)?,
            })?;
            let imported = record
                .loaded_module(&entry.module_request)
                .expect("loading populated the requested module");
            let locator = compiler
                .with_scopes(|access| Ok(access.get_binding(&scope, &entry.local_name)?))?
                .js_expect("analysis created the import binding")?;
            let binding = match &entry.import_name {
                ImportedName::Name(name) => {
                    let resolution = imported
                        .resolve_export(name, &mut HashSet::default())
                        .map_err(|error| export_error(error, name))?;
                    match resolution.binding_name() {
                        BindingName::Name(_) => ImportBinding::Single {
                            locator,
                            export_locator: resolution,
                        },
                        BindingName::Namespace => ImportBinding::Namespace {
                            locator,
                            module: resolution.into_module(),
                        },
                    }
                }
                ImportedName::Namespace => ImportBinding::Namespace {
                    locator,
                    module: imported,
                },
            };
            reserve(&mut imports, 1, &mut compiler.control)?;
            compiler.control.admit(CompilerWork::Copy {
                bytes: payload_bytes::<ImportBinding>(1)?,
            })?;
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            imports.push(binding);
        }
        let mut declared_var_names = Vec::<JsString>::new();
        let variables = compiler
            .control
            .with_name_admission(|admission| try_var_scoped_declarations(&source.ast, admission))?;
        for declaration in variables {
            compiler.control.admit(CompilerWork::MetadataVisit)?;
            let names = compiler
                .control
                .with_name_admission(|admission| match declaration {
                    VarScopedDeclaration::VariableDeclaration(node) => {
                        try_bound_names(node, admission)
                    }
                    VarScopedDeclaration::FunctionDeclaration(node) => {
                        try_bound_names(node, admission)
                    }
                    VarScopedDeclaration::GeneratorDeclaration(node) => {
                        try_bound_names(node, admission)
                    }
                    VarScopedDeclaration::AsyncFunctionDeclaration(node) => {
                        try_bound_names(node, admission)
                    }
                    VarScopedDeclaration::AsyncGeneratorDeclaration(node) => {
                        try_bound_names(node, admission)
                    }
                })?;
            for name in names {
                compiler.control.admit(CompilerWork::MetadataVisit)?;
                let name = name.try_to_js_string(compiler.interner, &mut compiler.control)?;
                let mut duplicate = false;
                for existing in &declared_var_names {
                    compiler.control.admit(CompilerWork::Search)?;
                    if existing
                        .as_str()
                        .try_eq(name.as_str(), &mut |work| {
                            compiler.control.admit(CompilerWork::String(work))
                        })
                        .map_err(string_error)?
                    {
                        duplicate = true;
                        break;
                    }
                }
                if !duplicate {
                    let binding = compiler
                        .with_scopes(|access| Ok(access.get_binding_reference(&scope, &name)?))?
                        .js_expect("analysis created the var binding")?;
                    let index = compiler.insert_binding(binding)?;
                    compiler.emit_binding_access(
                        BindingAccessOpcode::DefInitVar,
                        &index,
                        &CallFrame::undefined_register(),
                    )?;
                    reserve(&mut declared_var_names, 1, &mut compiler.control)?;
                    compiler.control.admit(CompilerWork::Copy {
                        bytes: payload_bytes::<JsString>(1)?,
                    })?;
                    compiler.control.admit(CompilerWork::MetadataWrite)?;
                    declared_var_names.push(name);
                }
            }
        }
        let mut declarations = Vec::new();
        let lexical = compiler.control.with_name_admission(|admission| {
            try_lexically_scoped_declarations(&source.ast, admission)
        })?;
        for declaration in lexical {
            compiler.control.admit(CompilerWork::MetadataVisit)?;
            let (spec, names): (FunctionSpec<'_>, _) = match declaration {
                LexicallyScopedDeclaration::FunctionDeclaration(function) => {
                    let names = compiler
                        .control
                        .with_name_admission(|admission| try_bound_names(function, admission))?;
                    (function.into(), names)
                }
                LexicallyScopedDeclaration::GeneratorDeclaration(function) => {
                    let names = compiler
                        .control
                        .with_name_admission(|admission| try_bound_names(function, admission))?;
                    (function.into(), names)
                }
                LexicallyScopedDeclaration::AsyncFunctionDeclaration(function) => {
                    let names = compiler
                        .control
                        .with_name_admission(|admission| try_bound_names(function, admission))?;
                    (function.into(), names)
                }
                LexicallyScopedDeclaration::AsyncGeneratorDeclaration(function) => {
                    let names = compiler
                        .control
                        .with_name_admission(|admission| try_bound_names(function, admission))?;
                    (function.into(), names)
                }
                _ => continue,
            };
            let name = names
                .first()
                .expect("a function declaration has one bound name");
            let name = name.try_to_js_string(compiler.interner, &mut compiler.control)?;
            let locator = compiler
                .with_scopes(|access| Ok(access.get_binding(&scope, &name)?))?
                .js_expect("analysis created the function binding")?;
            reserve(&mut declarations, 1, &mut compiler.control)?;
            compiler.control.admit(CompilerWork::Copy {
                bytes: payload_bytes::<(FunctionSpec<'_>, BindingLocator)>(1)?,
            })?;
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            declarations.push((spec, locator));
        }
        let mut functions = Vec::new();
        for (spec, locator) in declarations {
            compiler.control.admit(CompilerWork::MetadataVisit)?;
            compiler.control.admit(CompilerWork::Read {
                bytes: payload_bytes::<(FunctionSpec<'_>, BindingLocator)>(1)?,
            })?;
            let index = compiler.function(spec)?;
            reserve(&mut functions, 1, &mut compiler.control)?;
            compiler.control.admit(CompilerWork::Copy {
                bytes: payload_bytes::<(u32, BindingLocator)>(1)?,
            })?;
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            functions.push((index, locator));
        }
        compiler.compile_module_item_list(source.ast.items())?;
        let code = compiler.finish(None)?;
        let bytes = payload_bytes::<CodeBlock>(1)?;
        control.admit(CompilerWork::Capacity { bytes })?;
        control.admit(CompilerWork::Copy { bytes })?;
        control.admit(CompilerWork::MetadataWrite)?;
        Ok((
            SourceCode {
                scope,
                code: Rc::new(code),
                functions,
                imports,
            },
            extension,
        ))
    })?;
    extension.commit_private();
    Ok(result)
}

fn export_error(error: ResolveExportError, name: &JsString) -> JsError {
    let message = match error {
        ResolveExportError::NotFound => {
            format!("could not find export `{}`", name.to_std_string_escaped())
        }
        ResolveExportError::Ambiguous => format!(
            "could not resolve ambiguous export `{}`",
            name.to_std_string_escaped()
        ),
    };
    JsNativeError::syntax().with_message(message).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Source,
        builtins::promise::PromiseState,
        execution::{ExecutionOptions, ProfileMode},
    };
    use std::time::Duration;

    struct Reentry {
        module: Module,
        peer: Context,
        armed: bool,
        calls: usize,
        edit_realm: bool,
    }

    thread_local! {
        static REENTRY: RefCell<Option<Reentry>> = const { RefCell::new(None) };
    }

    struct ClearReentry;

    impl Drop for ClearReentry {
        fn drop(&mut self) {
            REENTRY.with(|state| *state.borrow_mut() = None);
        }
    }

    fn reentering_clock() -> Duration {
        REENTRY.with(|state| {
            let mut state = state.borrow_mut();
            if let Some(state) = state.as_mut()
                && state.armed
            {
                if state.edit_realm {
                    if state.calls == 0 {
                        state
                            .peer
                            .eval(Source::from_bytes("let callbackOwned = 41;"))
                            .unwrap();
                    }
                } else {
                    let error = state.module.link(state.peer.access()).unwrap_err();
                    assert_eq!(
                        error.as_native().unwrap().message(),
                        "module linking is already in progress"
                    );
                }
                state.calls += 1;
            }
        });
        Duration::ZERO
    }

    fn register_link_callback(context: &mut Context, module: Module) {
        context
            .register_global_builtin_callable(
                js_string!("linkModule"),
                0,
                NativeFunction::from_fn_with_captures(
                    |_, _, module, mut context| {
                        REENTRY.with(|state| state.borrow_mut().as_mut().unwrap().armed = true);
                        let result = module.link(context.reborrow());
                        REENTRY.with(|state| state.borrow_mut().as_mut().unwrap().armed = false);
                        result?;
                        Ok(JsValue::undefined())
                    },
                    module,
                ),
            )
            .unwrap();
    }

    #[test]
    fn a_profile_clock_can_reenter_module_linking_without_an_interior_borrow() {
        let mut context = Context::builder().build().unwrap();
        let module = Module::parse(
            Source::from_bytes("export function ready() { return 42; }"),
            None,
            context.access(),
        )
        .unwrap();
        let loaded = module.load(context.access()).unwrap();
        context.run_jobs().unwrap();
        assert_eq!(
            loaded.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        let peer = Context::builder()
            .runtime(context.runtime().clone())
            .build()
            .unwrap();
        REENTRY.with(|state| {
            *state.borrow_mut() = Some(Reentry {
                module: module.clone(),
                peer,
                armed: false,
                calls: 0,
                edit_realm: false,
            })
        });
        let _clear = ClearReentry;
        register_link_callback(&mut context, module.clone());
        let evaluated = context
            .eval_with_budget(
                Source::from_bytes("linkModule()"),
                ExecutionOptions::new(256)
                    .with_profile(ProfileMode::Summary { timing: true })
                    .with_profile_clock(reentering_clock),
            )
            .unwrap();
        assert!(evaluated.result.unwrap().is_undefined());
        REENTRY.with(|state| assert!(state.borrow().as_ref().unwrap().calls > 0));
        assert!(
            module
                .get_value(js_string!("ready"), context.access())
                .unwrap()
                .is_callable()
        );
    }

    #[test]
    fn a_clock_realm_edit_survives_group_conflict_and_retained_module_retry() {
        let mut context = Context::builder().build().unwrap();
        let module = Module::parse(
            Source::from_bytes("export function answer() { return callbackOwned + 1; }"),
            None,
            context.access(),
        )
        .unwrap();
        let loaded = module.load(context.access()).unwrap();
        context.run_jobs().unwrap();
        assert_eq!(
            loaded.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        let mut peer = Context::builder()
            .runtime(context.runtime().clone())
            .build()
            .unwrap();
        peer.enter_realm(module.realm()).unwrap();
        REENTRY.with(|state| {
            *state.borrow_mut() = Some(Reentry {
                module: module.clone(),
                peer,
                armed: false,
                calls: 0,
                edit_realm: true,
            })
        });
        let _clear = ClearReentry;
        register_link_callback(&mut context, module.clone());
        let outcome = context
            .eval_with_budget(
                Source::from_bytes("linkModule()"),
                ExecutionOptions::new(256)
                    .with_profile(ProfileMode::Summary { timing: true })
                    .with_profile_clock(reentering_clock),
            )
            .unwrap();
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&crate::error::EngineError::ScopeAnalysis(
                surgeist_js_ast::scope::ScopeStateError::Conflict,
            ))
        );
        assert_eq!(
            context.eval(Source::from_bytes("callbackOwned")).unwrap(),
            41.into()
        );
        module.link(context.access()).unwrap();
        let answer = module
            .get_value(js_string!("answer"), context.access())
            .unwrap();
        assert_eq!(
            answer
                .as_function()
                .unwrap()
                .call(&JsValue::undefined(), &[], context.access())
                .unwrap(),
            42.into()
        );
    }
}
