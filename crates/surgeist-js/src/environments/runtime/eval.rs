//! Private eval environment storage and callback-free installation.

use super::{
    DeclarativeEnvironmentKind, DeclarativeEnvironmentRef, EnvironmentStack, LexicalEnvironment,
};
use crate::{
    JsResult,
    realm::{PreparedGlobalEnvironment, Realm},
};
use surgeist_js_ast::scope::ScopeStateError;

pub(crate) struct EvalBindingCounts {
    pub(crate) lexical: u32,
    pub(crate) variable: u32,
    pub(crate) global: u32,
}

pub(crate) struct PreparedEvalEnvironment {
    environments: EnvironmentStack,
    variable: DeclarativeEnvironmentRef,
    variable_count: u32,
    expected_function_count: Option<usize>,
    global: DeclarativeEnvironmentRef,
    expected_global_count: usize,
    global_storage: PreparedGlobalEnvironment,
    poison: bool,
}

impl PreparedEvalEnvironment {
    pub(crate) fn new(
        mut environments: EnvironmentStack,
        variable: DeclarativeEnvironmentRef,
        counts: EvalBindingCounts,
        realm: &Realm,
        strict: bool,
        direct: bool,
    ) -> JsResult<Self> {
        let global = realm.environment();
        let expected_global_count = global
            .kind()
            .as_global()
            .expect("realm global")
            .binding_count();
        let global_storage = realm.prepare_global_env(counts.global)?;
        let expected_function_count = if !strict {
            variable
                .kind()
                .as_function()
                .map(|function| function.prepare_eval_bindings(counts.variable))
                .transpose()?
        } else {
            None
        };
        let (poisoned, with) = environments.compute_poisoned_with(&global);
        environments.push_declarative(
            DeclarativeEnvironmentKind::Lexical(LexicalEnvironment::try_new(counts.lexical)?),
            poisoned,
            with,
            global.runtime(),
        );
        Ok(Self {
            environments,
            variable,
            variable_count: counts.variable,
            expected_function_count,
            global,
            expected_global_count,
            global_storage,
            poison: direct && !strict,
        })
    }

    pub(crate) fn environments(&self) -> &EnvironmentStack {
        &self.environments
    }

    pub(crate) fn changes_caller(&self) -> bool {
        self.poison
            || self.global_storage.creates_bindings()
            || self
                .expected_function_count
                .is_some_and(|expected| self.variable_count as usize > expected)
    }

    /// Check after all callbacks and before committing the scope publication.
    pub(crate) fn validate(&self) -> JsResult<()> {
        if self
            .global
            .kind()
            .as_global()
            .expect("realm global")
            .binding_count()
            != self.expected_global_count
        {
            return Err(ScopeStateError::Conflict.into());
        }
        if let Some(expected) = self.expected_function_count {
            self.variable
                .kind()
                .as_function()
                .expect("prepared function environment")
                .validate_eval_bindings(expected)?;
        }
        Ok(())
    }

    /// Install only appended slots and poison the actual caller chain at entry.
    pub(crate) fn install(mut self) -> EnvironmentStack {
        self.global_storage.install();
        if self.expected_function_count.is_some() {
            self.variable
                .kind()
                .as_function()
                .expect("prepared function environment")
                .install_eval_bindings(self.variable_count);
        }
        if self.poison {
            self.environments.poison_until_last_function(&self.global);
        }
        self.environments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, JsValue};

    #[test]
    fn dropping_private_eval_storage_keeps_caller_slots_and_poisoning_unchanged() {
        let context = Context::builder().build().unwrap();
        let realm = context.realm();
        let global = realm.environment();
        let storage = PreparedEvalEnvironment::new(
            EnvironmentStack::new(),
            global.clone(),
            EvalBindingCounts {
                lexical: 1,
                variable: 2,
                global: 2,
            },
            realm,
            false,
            true,
        )
        .unwrap();
        assert_eq!(global.kind().as_global().unwrap().binding_count(), 0);
        assert!(!global.poisoned());
        assert_eq!(storage.environments().len(), 1);
        drop(storage);
        assert_eq!(global.kind().as_global().unwrap().binding_count(), 0);
        assert!(!global.poisoned());
    }

    #[test]
    fn eval_installation_appends_defaults_and_preserves_callback_owned_values() {
        let context = Context::builder().build().unwrap();
        let realm = context.realm();
        let global = realm.environment();
        let values = global.kind().as_global().unwrap();
        values.extend_bindings(1);
        values.set(0, JsValue::new(1));
        let storage = PreparedEvalEnvironment::new(
            EnvironmentStack::new(),
            global.clone(),
            EvalBindingCounts {
                lexical: 1,
                variable: 2,
                global: 2,
            },
            realm,
            false,
            true,
        )
        .unwrap();
        values.set(0, JsValue::new(42));
        storage.validate().unwrap();
        let installed = storage.install();
        assert_eq!(values.binding_count(), 2);
        assert_eq!(values.get(0), Some(JsValue::new(42)));
        assert_eq!(values.get(1), None);
        assert!(global.poisoned());
        assert_eq!(installed.len(), 1);
    }

    #[test]
    fn eval_storage_rejects_changed_live_length_without_restoring_callback_state() {
        let context = Context::builder().build().unwrap();
        let realm = context.realm();
        let global = realm.environment();
        let values = global.kind().as_global().unwrap();
        let storage = PreparedEvalEnvironment::new(
            EnvironmentStack::new(),
            global.clone(),
            EvalBindingCounts {
                lexical: 1,
                variable: 3,
                global: 3,
            },
            realm,
            false,
            true,
        )
        .unwrap();
        values.extend_bindings(1);
        values.set(0, JsValue::new(42));
        assert_eq!(
            storage.validate().unwrap_err().as_engine(),
            Some(&crate::EngineError::ScopeAnalysis(
                ScopeStateError::Conflict
            ))
        );
        drop(storage);
        assert_eq!(values.binding_count(), 1);
        assert_eq!(values.get(0), Some(JsValue::new(42)));
        assert!(!global.poisoned());
    }
}
