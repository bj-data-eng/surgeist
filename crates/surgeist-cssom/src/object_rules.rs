//! Concrete rule-family members sharing the native CSSOM engines.
//! Live handles read current state from an explicit owner. Captured child reads
//! retain the same immutable snapshot throughout a chain. Mutations inherit the
//! supplied store’s existing quotas; lookup/formatting reads accept native limits.
use crate::*;
use surgeist_css::diagnostic::CssRecoveryDiagnostic;
use surgeist_css::serialization::CssSerializationLimits;

macro_rules! value_reader {
    ($kind:ident, $name:ident, $native:ident, $result:ty $(, $arg:ident : $ty:ty)*) => {
        impl $kind {
            pub fn $name(&self, store: &CssomStore $(, $arg: $ty)*) -> Result<$result, CssomError> {
                let snapshot = store.snapshot();
                Self::from_state(&snapshot.state, self.id())?;
                snapshot.$native(self.id() $(, $arg)*)
            }
        }
        impl CssomCapture<$kind> {
            pub fn $name(&self $(, $arg: $ty)*) -> Result<$result, CssomError> {
                self.snapshot.$native(self.handle.id() $(, $arg)*)
            }
        }
    };
}
fn owned_text(source: &str) -> Result<String, CssomError> {
    let mut result = String::new();
    result
        .try_reserve_exact(source.len())
        .map_err(|_| CssomError::Limit {
            resource: "object rule string capacity",
            maximum: source.len(),
        })?;
    result.push_str(source);
    Ok(result)
}
macro_rules! string_reader {
    ($kind:ident, $name:ident, $native:ident) => {
        impl $kind {
            pub fn $name(&self, store: &CssomStore) -> Result<String, CssomError> {
                let snapshot = store.snapshot();
                Self::from_state(&snapshot.state, self.id())?;
                snapshot.$native(self.id()).and_then(owned_text)
            }
        }
        impl CssomCapture<$kind> {
            pub fn $name(&self) -> Result<String, CssomError> {
                self.snapshot.$native(self.handle.id()).and_then(owned_text)
            }
        }
    };
}
value_reader!(CssStyleRule, selector_text, style_selector_text, String, limits: CssSerializationLimits);
value_reader!(CssPageRule, selector_text, page_selector_text, String, limits: CssSerializationLimits);
value_reader!(CssKeyframeRule, key_text, keyframe_key_text, String, limits: CssSerializationLimits);
string_reader!(CssMarginRule, name, margin_name);
string_reader!(CssImportRule, href, import_href);
string_reader!(CssNamespaceRule, namespace_uri, namespace_uri);
string_reader!(CssNamespaceRule, prefix, namespace_prefix);
string_reader!(CssSupportsConditionRule, name, named_support_condition_name);
string_reader!(CssKeyframesRule, name, keyframes_name);
value_reader!(CssImportRule, layer_name, import_layer_name, Option<String>, max_bytes: usize);
value_reader!(CssImportRule, supports_text, import_supports_text, Option<String>, max_bytes: usize);
value_reader!(CssLayerBlockRule, name, layer_block_name, String, max_bytes: usize);
value_reader!(CssLayerStatementRule, name_list, layer_statement_names, Vec<String>, max_bytes: usize);
value_reader!(CssScopeRule, start, scope_start, Option<String>, limits: CssSerializationLimits);
value_reader!(CssScopeRule, end, scope_end, Option<String>, limits: CssSerializationLimits);
value_reader!(CssContainerRule, container_name, container_name, String, max_bytes: usize);
value_reader!(CssContainerRule, container_query, container_query, String, max_bytes: usize);
value_reader!(CssContainerRule, conditions, container_conditions, Vec<CssomContainerCondition>, max_bytes: usize);

macro_rules! style_member {
    ($kind:ident, $facet:ident) => {
        impl $kind {
            pub fn style(&self, store: &CssomStore) -> Result<$facet, CssomError> {
                let snapshot = store.snapshot();
                Self::from_state(&snapshot.state, self.id())?;
                $facet::from_state(&snapshot.state, snapshot.rule_style(self.id())?)
            }
        }
        impl CssomCapture<$kind> {
            pub fn style(&self) -> Result<CssomCapture<$facet>, CssomError> {
                $facet::from_state(
                    &self.snapshot.state,
                    self.snapshot.rule_style(self.handle.id())?,
                )?
                .capture(&self.snapshot)
            }
        }
    };
}
style_member!(CssStyleRule, CssStyleProperties);
style_member!(CssNestedDeclarations, CssStyleProperties);
style_member!(CssPageRule, CssPageDescriptors);
style_member!(CssMarginRule, CssMarginDescriptors);
// The selected Animations edition exposes the base declaration interface here.
style_member!(CssKeyframeRule, CssStyleDeclaration);

macro_rules! media_member {
    ($kind:ident, $native:ident) => {
        impl $kind {
            pub fn media(&self, store: &CssomStore) -> Result<MediaList, CssomError> {
                let snapshot = store.snapshot();
                Self::from_state(&snapshot.state, self.id())?;
                MediaList::from_state(&snapshot.state, snapshot.$native(self.id())?)
            }
        }
        impl CssomCapture<$kind> {
            pub fn media(&self) -> Result<CssomCapture<MediaList>, CssomError> {
                MediaList::from_state(
                    &self.snapshot.state,
                    self.snapshot.$native(self.handle.id())?,
                )?
                .capture(&self.snapshot)
            }
        }
    };
}
media_member!(CssMediaRule, group_media);
media_member!(CssImportRule, import_media);
impl CssImportRule {
    pub fn style_sheet(&self, store: &CssomStore) -> Result<Option<CssStyleSheet>, CssomError> {
        let snapshot = store.snapshot();
        Self::from_state(&snapshot.state, self.id())?;
        snapshot
            .import_style_sheet(self.id())?
            .map(|id| CssStyleSheet::from_state(&snapshot.state, id))
            .transpose()
    }
}
impl CssomCapture<CssImportRule> {
    pub fn style_sheet(&self) -> Result<Option<CssomCapture<CssStyleSheet>>, CssomError> {
        self.snapshot
            .import_style_sheet(self.handle.id())?
            .map(|id| CssStyleSheet::from_state(&self.snapshot.state, id)?.capture(&self.snapshot))
            .transpose()
    }
}
macro_rules! condition_members {
    ($kind:ident) => {
        value_reader!($kind, condition_text, condition_text, String, limits: CssSerializationLimits);
    };
}
condition_members!(CssConditionRule);
condition_members!(CssMediaRule);
condition_members!(CssSupportsRule);
condition_members!(CssContainerRule);
// No Container setter or evaluator is emitted: those capabilities are absent.

macro_rules! matching_member {
    ($kind:ident) => {
        value_reader!($kind, matches, condition_matches, bool, result: Option<&CssomQueryResult>);
    };
}
matching_member!(CssMediaRule);
matching_member!(CssSupportsRule);

fn child_list(snapshot: &CssomSnapshot, id: &CssomRuleId) -> Result<CssRuleList, CssomError> {
    if let Some(sheet) = snapshot.parent_style_sheet(id)? {
        snapshot.sheet_rules(sheet)?;
    }
    CssRuleList::from_state(&snapshot.state, snapshot.child_rules(id)?)
}
macro_rules! child_list_member {
    ($kind:ident) => {
        impl $kind {
            pub fn css_rules(&self, store: &CssomStore) -> Result<CssRuleList, CssomError> {
                let snapshot = store.snapshot();
                Self::from_state(&snapshot.state, self.id())?;
                child_list(&snapshot, self.id())
            }
        }
        impl CssomCapture<$kind> {
            pub fn css_rules(&self) -> Result<CssomCapture<CssRuleList>, CssomError> {
                child_list(&self.snapshot, self.handle.id())?.capture(&self.snapshot)
            }
        }
    };
}
child_list_member!(CssKeyframesRule);
macro_rules! grouping_members {
    ($kind:ident) => {
        child_list_member!($kind);
        impl $kind {
            /// Omitted insertion index is zero. The grouping engine owns its
            /// distinct index/grammar/ancestry guards and inert named-support policy.
            pub fn insert_rule(
                &self,
                store: &mut CssomStore,
                text: &str,
                index: Option<usize>,
            ) -> Result<CssomOperationOutcome<usize>, CssomError> {
                crate::object_operations::execute(store, |batch| {
                    Self::from_state(&batch.staged, self.id())?;
                    crate::object_operations::stage(batch, |batch| {
                        let index = index.unwrap_or(0);
                        batch.insert_group_rule(self.id(), text, index)?;
                        let list = batch
                            .staged
                            .rule(self.id())?
                            .data()
                            .children()
                            .ok_or(CssomError::WrongKind)?;
                        let id = &batch.staged.lists[list][index];
                        let diagnostics = rule_diagnostics(batch, id)?;
                        Ok(CssomStagedOperation::new(index, diagnostics))
                    })
                })
            }
            pub fn delete_rule(
                &self,
                store: &mut CssomStore,
                index: usize,
            ) -> Result<CssomOperationOutcome<()>, CssomError> {
                crate::object_operations::execute(store, |batch| {
                    Self::from_state(&batch.staged, self.id())?;
                    crate::object_operations::stage(batch, |batch| {
                        batch.delete_group_rule(self.id(), index)?;
                        Ok(CssomStagedOperation::new((), Vec::new()))
                    })
                })
            }
        }
    };
}
grouping_members!(CssGroupingRule);
grouping_members!(CssConditionRule);
grouping_members!(CssStyleRule);
grouping_members!(CssPageRule);
grouping_members!(CssMediaRule);
grouping_members!(CssSupportsRule);
grouping_members!(CssContainerRule);
grouping_members!(CssScopeRule);
grouping_members!(CssLayerBlockRule);
grouping_members!(CssSupportsConditionRule);

fn rule_diagnostics(
    batch: &CssomBatch,
    id: &CssomRuleId,
) -> Result<Vec<CssRecoveryDiagnostic>, CssomError> {
    crate::rule_operations::copy_rule_diagnostics(batch.staged.rule(id)?.admission_diagnostics())
}
macro_rules! selector_setter {
    ($kind:ident, $name:ident, $native:ident) => {
        impl $kind {
            pub fn $name(
                &self,
                store: &mut CssomStore,
                text: &str,
            ) -> Result<CssomOperationOutcome<()>, CssomError> {
                crate::object_operations::execute(store, |batch| {
                    Self::from_state(&batch.staged, self.id())?;
                    crate::object_operations::stage(batch, |batch| {
                        let (_, diagnostics) = batch.$native(self.id(), text)?;
                        Ok(CssomStagedOperation::new((), diagnostics))
                    })
                })
            }
        }
    };
}
selector_setter!(
    CssStyleRule,
    set_selector_text,
    set_style_selector_text_with_diagnostics
);
selector_setter!(
    CssPageRule,
    set_selector_text,
    set_page_selector_text_with_diagnostics
);
impl CssKeyframeRule {
    /// Keyframe's style is the base declaration interface in the selected source.
    /// ```compile_fail
    /// use surgeist_cssom::{CssKeyframeRule, CssomStore, CssomDeclarationRequestLimits};
    /// fn invalid(rule: &CssKeyframeRule, store: &CssomStore) {
    ///     rule.style(store).unwrap().css_float(store, CssomDeclarationRequestLimits::default());
    /// }
    /// ```
    pub fn set_key_text(
        &self,
        store: &mut CssomStore,
        text: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        crate::object_operations::execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            crate::object_operations::stage(batch, |batch| {
                batch.set_keyframe_key_text(self.id(), text)?;
                Ok(CssomStagedOperation::new(
                    (),
                    rule_diagnostics(batch, self.id())?,
                ))
            })
        })
    }
}
value_reader!(CssKeyframesRule, length, keyframes_length, usize);
fn indexed(
    snapshot: &CssomSnapshot,
    id: &CssomRuleId,
    index: usize,
) -> Result<CssomKeyframeIndex<CssKeyframeRule>, CssomError> {
    Ok(match snapshot.keyframes_index(id, index)? {
        CssomKeyframeIndex::Rule(id) => {
            CssomKeyframeIndex::Rule(CssKeyframeRule::from_state(&snapshot.state, &id)?)
        }
        CssomKeyframeIndex::Undefined => CssomKeyframeIndex::Undefined,
    })
}
impl CssKeyframesRule {
    /// Indexed absence is undefined, independently of nullable list item/find.
    pub fn indexed(
        &self,
        store: &CssomStore,
        index: usize,
    ) -> Result<CssomKeyframeIndex<CssKeyframeRule>, CssomError> {
        let snapshot = store.snapshot();
        Self::from_state(&snapshot.state, self.id())?;
        indexed(&snapshot, self.id(), index)
    }
    pub fn find_rule(
        &self,
        store: &CssomStore,
        text: &str,
        limits: &CssomLimits,
    ) -> Result<Option<CssKeyframeRule>, CssomError> {
        let snapshot = store.snapshot();
        Self::from_state(&snapshot.state, self.id())?;
        snapshot
            .find_keyframe_rule(self.id(), text, limits)?
            .map(|id| CssKeyframeRule::from_state(&snapshot.state, id))
            .transpose()
    }
    pub fn set_name(
        &self,
        store: &mut CssomStore,
        text: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        crate::object_operations::execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            crate::object_operations::stage(batch, |batch| {
                batch.set_keyframes_name(self.id(), text)?;
                Ok(CssomStagedOperation::new((), Vec::new()))
            })
        })
    }
    pub fn append_rule(
        &self,
        store: &mut CssomStore,
        text: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        crate::object_operations::execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            crate::object_operations::stage(batch, |batch| {
                let (_, diagnostics) =
                    batch.append_keyframe_rule_with_diagnostics(self.id(), text)?;
                Ok(CssomStagedOperation::new((), diagnostics))
            })
        })
    }
    pub fn delete_rule(
        &self,
        store: &mut CssomStore,
        text: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        crate::object_operations::execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            crate::object_operations::stage(batch, |batch| {
                batch.delete_keyframe_rule(self.id(), text)?;
                Ok(CssomStagedOperation::new((), Vec::new()))
            })
        })
    }
}
impl CssomCapture<CssKeyframesRule> {
    pub fn indexed(
        &self,
        index: usize,
    ) -> Result<CssomKeyframeIndex<CssomCapture<CssKeyframeRule>>, CssomError> {
        Ok(match indexed(&self.snapshot, self.handle.id(), index)? {
            CssomKeyframeIndex::Rule(handle) => {
                CssomKeyframeIndex::Rule(handle.capture(&self.snapshot)?)
            }
            CssomKeyframeIndex::Undefined => CssomKeyframeIndex::Undefined,
        })
    }
    pub fn find_rule(
        &self,
        text: &str,
        limits: &CssomLimits,
    ) -> Result<Option<CssomCapture<CssKeyframeRule>>, CssomError> {
        self.snapshot
            .find_keyframe_rule(self.handle.id(), text, limits)?
            .map(|id| {
                CssKeyframeRule::from_state(&self.snapshot.state, id)?.capture(&self.snapshot)
            })
            .transpose()
    }
}
