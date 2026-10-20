//! Standard declaration objects over one domain-aware declaration engine.
//!
//! Live getters validate their identity against the supplied store on every
//! call. Captured getters and parent objects read only the capture's owning
//! snapshot. Each read uses explicit request limits; a pending footprint or
//! unavailable native projection remains a typed failure, never an empty value.
//!
//! Named descriptor setters and `css_float` forward to the common setter with
//! empty priority. The finite reflection inventories describe interface members,
//! not a support profile: generic operations still use the supplied domain and
//! whole-declaration usability owners. Mutations return publication outcomes
//! whose owner effects must be handled by the binding after publication.
use crate::{
    declaration::DeclarationRead,
    model::State,
    object_operations::{execute, stage},
    *,
};

/// Actual checked candidates and the context captured during their preparation.
/// The resolver receives no store, batch or host-write capability.
/// Resolver work should be pure; CSSOM does not roll back side effects on
/// external values a caller chooses to capture in its closure.
#[derive(Debug)]
pub struct CssomAdmissionRequest<'a> {
    candidates: &'a [CssomDeclarationCandidate],
    context: &'a CssomContext,
}
impl CssomAdmissionRequest<'_> {
    pub fn candidates(&self) -> &[CssomDeclarationCandidate] {
        self.candidates
    }
    pub fn context(&self) -> &CssomContext {
        self.context
    }
}

/// Supplied decisions and their independently current input versions.
///
/// Collect current versions after decision work. The owning declaration engine
/// validates completeness, identity, role and freshness when applying this packet;
/// construction alone does not assert that it belongs to a particular request.
#[derive(Debug)]
pub struct CssomAdmission {
    decisions: Vec<CssomDeclarationDecision>,
    current_inputs: Vec<CssomInputVersion>,
}
impl CssomAdmission {
    pub fn new(
        decisions: Vec<CssomDeclarationDecision>,
        current_inputs: Vec<CssomInputVersion>,
    ) -> Self {
        Self {
            decisions,
            current_inputs,
        }
    }
    pub fn decisions(&self) -> &[CssomDeclarationDecision] {
        &self.decisions
    }
    pub fn current_inputs(&self) -> &[CssomInputVersion] {
        &self.current_inputs
    }
}

pub(crate) fn admit_prepared(
    batch: &mut CssomBatch,
    prepared: CssomPreparedDeclaration,
    resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
) -> Result<CssomStagedOperation<CssomDeclarationEditResult>, CssomError> {
    stage(batch, |batch| {
        let mut diagnostics = Vec::new();
        diagnostics
            .try_reserve_exact(prepared.diagnostics().len())
            .map_err(|_| CssomError::Limit {
                resource: "object operation diagnostics capacity",
                maximum: prepared.diagnostics().len(),
            })?;
        diagnostics.extend_from_slice(prepared.diagnostics());
        let admission = if prepared.candidates().is_empty() {
            let current_inputs = batch.current_declaration_inputs(prepared.block())?;
            CssomAdmission::new(Vec::new(), current_inputs)
        } else {
            resolver(&CssomAdmissionRequest {
                candidates: prepared.candidates(),
                context: prepared.captured_context(),
            })?
        };
        let value = batch.apply_declaration_with_inputs(
            prepared,
            &admission.decisions,
            &admission.current_inputs,
        )?;
        Ok(CssomStagedOperation::new(value, diagnostics))
    })
}

fn stage_remove(
    batch: &mut CssomBatch,
    object: &CssStyleDeclaration,
    name: &str,
    limits: CssomDeclarationRequestLimits,
) -> Result<CssomStagedOperation<String>, CssomError> {
    stage(batch, |batch| {
        CssStyleDeclaration::from_state(&batch.staged, object.id())?;
        Ok(CssomStagedOperation::new(
            batch.remove_property(object.id(), name, limits)?,
            Vec::new(),
        ))
    })
}
fn stage_set(
    batch: &mut CssomBatch,
    object: &CssStyleDeclaration,
    name: &str,
    value: &str,
    priority: Option<&str>,
    limits: CssomDeclarationRequestLimits,
    resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
) -> Result<CssomStagedOperation<()>, CssomError> {
    stage(batch, |batch| {
        CssStyleDeclaration::from_state(&batch.staged, object.id())?;
        let prepared =
            batch.prepare_set_property(object.id(), name, value, priority.unwrap_or(""), limits)?;
        Ok(admit_prepared(batch, prepared, resolver)?.map(|_| ()))
    })
}

impl CssStyleDeclaration {
    pub fn remove_property(
        &self,
        store: &mut CssomStore,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomOperationOutcome<String>, CssomError> {
        execute(store, |batch| stage_remove(batch, self, name, limits))
    }
    /// Resolve only candidates requiring whole-declaration usability decisions.
    /// `None` priority is the standard empty priority string.
    /// The owner remains exclusively borrowed during admission:
    /// ```compile_fail
    /// use surgeist_cssom::*;
    /// fn reenter(object: &CssStyleDeclaration, store: &mut CssomStore) {
    ///     let _ = object.set_property(store, "width", "2px", None,
    ///         CssomDeclarationRequestLimits::default(), |_| {
    ///             let _ = store.snapshot();
    ///             Ok(CssomAdmission::new(Vec::new(), Vec::new()))
    ///         });
    /// }
    /// ```
    pub fn set_property(
        &self,
        store: &mut CssomStore,
        name: &str,
        value: &str,
        priority: Option<&str>,
        limits: CssomDeclarationRequestLimits,
        resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            stage_set(batch, self, name, value, priority, limits, resolver)
        })
    }
    /// Advanced product editing. This stages against the supplied batch and
    /// never automatically commits a standard object call.
    pub fn edit<'batch>(
        &self,
        batch: &'batch mut CssomBatch,
    ) -> Result<CssomDeclarationEdit<'batch>, CssomError> {
        let _ = stage(batch, |batch| {
            CssStyleDeclaration::from_state(&batch.staged, self.id())?;
            Ok(CssomStagedOperation::new((), Vec::new()))
        })?;
        Ok(CssomDeclarationEdit {
            object: self.clone(),
            batch,
        })
    }
}
/// A typed declaration editor borrowing an explicit caller-owned batch.
pub struct CssomDeclarationEdit<'batch> {
    pub(crate) object: CssStyleDeclaration,
    pub(crate) batch: &'batch mut CssomBatch,
}
impl CssomDeclarationEdit<'_> {
    pub fn remove_property(
        &mut self,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomStagedOperation<String>, CssomError> {
        stage_remove(self.batch, &self.object, name, limits)
    }
    pub fn set_property(
        &mut self,
        name: &str,
        value: &str,
        priority: Option<&str>,
        limits: CssomDeclarationRequestLimits,
        resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
    ) -> Result<CssomStagedOperation<()>, CssomError> {
        stage_set(
            self.batch,
            &self.object,
            name,
            value,
            priority,
            limits,
            resolver,
        )
    }
}

fn reader<'a>(
    state: &'a State,
    object: &CssStyleDeclaration,
    limits: CssomDeclarationRequestLimits,
) -> Result<DeclarationRead<'a>, CssomError> {
    CssStyleDeclaration::from_state(state, object.id())?;
    Ok(DeclarationRead::new(state.block(object.id())?, limits))
}

fn stage_css_text(
    batch: &mut CssomBatch,
    object: &CssStyleDeclaration,
    text: &str,
    limits: CssomDeclarationRequestLimits,
    resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
) -> Result<CssomStagedOperation<()>, CssomError> {
    stage(batch, |batch| {
        CssStyleDeclaration::from_state(&batch.staged, object.id())?;
        let prepared = batch.prepare_css_text(object.id(), text, limits)?;
        Ok(admit_prepared(batch, prepared, resolver)?.map(|_| ()))
    })
}

impl CssStyleDeclaration {
    pub fn css_text(
        &self,
        store: &CssomStore,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        reader(&store.state, self, limits)?.css_text()
    }
    pub fn length(
        &self,
        store: &CssomStore,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<usize, CssomError> {
        reader(&store.state, self, limits)?.length()
    }
    pub fn item(
        &self,
        store: &CssomStore,
        index: usize,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        reader(&store.state, self, limits)?.item(index)
    }
    pub fn get_property_value(
        &self,
        store: &CssomStore,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        reader(&store.state, self, limits)?.get_property_value(name)
    }
    pub fn get_property_priority(
        &self,
        store: &CssomStore,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        reader(&store.state, self, limits)?.get_property_priority(name)
    }
    pub fn parent_rule(&self, store: &CssomStore) -> Result<Option<CssRule>, CssomError> {
        reader(&store.state, self, Default::default())?
            .parent_rule()
            .map(|id| CssRule::from_state(&store.state, id))
            .transpose()
    }
    /// Replace raw declaration contents through the common domain admission
    /// owner. Recovery diagnostics and explicit owner effects belong to the
    /// returned publication; no host writes run inside this operation.
    pub fn set_css_text(
        &self,
        store: &mut CssomStore,
        text: &str,
        limits: CssomDeclarationRequestLimits,
        resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            stage_css_text(batch, self, text, limits, resolver)
        })
    }
}

impl CssomCapture<CssStyleDeclaration> {
    pub fn css_text(&self, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> {
        reader(&self.snapshot.state, &self.handle, limits)?.css_text()
    }
    pub fn length(&self, limits: CssomDeclarationRequestLimits) -> Result<usize, CssomError> {
        reader(&self.snapshot.state, &self.handle, limits)?.length()
    }
    pub fn item(
        &self,
        index: usize,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        reader(&self.snapshot.state, &self.handle, limits)?.item(index)
    }
    pub fn get_property_value(
        &self,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        reader(&self.snapshot.state, &self.handle, limits)?.get_property_value(name)
    }
    pub fn get_property_priority(
        &self,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        reader(&self.snapshot.state, &self.handle, limits)?.get_property_priority(name)
    }
    pub fn parent_rule(&self) -> Result<Option<CssomCapture<CssRule>>, CssomError> {
        reader(&self.snapshot.state, &self.handle, Default::default())?
            .parent_rule()
            .map(|id| {
                Ok(CssomCapture {
                    handle: CssRule::from_state(&self.snapshot.state, id)?,
                    snapshot: self.snapshot.clone(),
                })
            })
            .transpose()
    }
}

impl CssomDeclarationEdit<'_> {
    /// Stages a replacement in this explicit product batch without publication.
    pub fn set_css_text(
        &mut self,
        text: &str,
        limits: CssomDeclarationRequestLimits,
        resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
    ) -> Result<CssomStagedOperation<()>, CssomError> {
        stage_css_text(self.batch, &self.object, text, limits, resolver)
    }
}

// Concrete inherited members keep Rust method membership distinct from the
// domain engine. No public trait or Deref hierarchy widens descriptor facets.
macro_rules! common_declarations {
    ($facet:ident) => {
        impl $facet {
            pub fn css_text(&self, store: &CssomStore, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.as_style_declaration().css_text(store, limits) }
            pub fn length(&self, store: &CssomStore, limits: CssomDeclarationRequestLimits) -> Result<usize, CssomError> { self.as_style_declaration().length(store, limits) }
            pub fn item(&self, store: &CssomStore, index: usize, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.as_style_declaration().item(store, index, limits) }
            pub fn get_property_value(&self, store: &CssomStore, name: &str, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.as_style_declaration().get_property_value(store, name, limits) }
            pub fn get_property_priority(&self, store: &CssomStore, name: &str, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.as_style_declaration().get_property_priority(store, name, limits) }
            pub fn parent_rule(&self, store: &CssomStore) -> Result<Option<CssRule>, CssomError> { self.as_style_declaration().parent_rule(store) }
            pub fn set_css_text(&self, store: &mut CssomStore, text: &str, limits: CssomDeclarationRequestLimits, resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>) -> Result<CssomOperationOutcome<()>, CssomError> { self.as_style_declaration().set_css_text(store, text, limits, resolver) }
            pub fn set_property(&self, store: &mut CssomStore, name: &str, value: &str, priority: Option<&str>, limits: CssomDeclarationRequestLimits, resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>) -> Result<CssomOperationOutcome<()>, CssomError> { self.as_style_declaration().set_property(store, name, value, priority, limits, resolver) }
            pub fn remove_property(&self, store: &mut CssomStore, name: &str, limits: CssomDeclarationRequestLimits) -> Result<CssomOperationOutcome<String>, CssomError> { self.as_style_declaration().remove_property(store, name, limits) }
        }
        impl CssomCapture<$facet> {
            pub fn css_text(&self, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.as_style_declaration().css_text(limits) }
            pub fn length(&self, limits: CssomDeclarationRequestLimits) -> Result<usize, CssomError> { self.as_style_declaration().length(limits) }
            pub fn item(&self, index: usize, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.as_style_declaration().item(index, limits) }
            pub fn get_property_value(&self, name: &str, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.as_style_declaration().get_property_value(name, limits) }
            pub fn get_property_priority(&self, name: &str, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.as_style_declaration().get_property_priority(name, limits) }
            pub fn parent_rule(&self) -> Result<Option<CssomCapture<CssRule>>, CssomError> { self.as_style_declaration().parent_rule() }
        }
    };
}
common_declarations!(CssStyleProperties);
common_declarations!(CssPageDescriptors);
common_declarations!(CssMarginDescriptors);
common_declarations!(CssFontFaceDescriptors);

// These finite IDL inventories delegate generic queries/mutations; they are not
// an additional supported-name or grammar inventory.
macro_rules! named_declarations {
    ($facet:ident; $($get:ident, $set:ident => $name:literal),+ $(,)?) => {
        impl $facet {$(
            pub fn $get(&self, store: &CssomStore, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.get_property_value(store, $name, limits) }
            pub fn $set(&self, store: &mut CssomStore, value: &str, limits: CssomDeclarationRequestLimits, resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>) -> Result<CssomOperationOutcome<()>, CssomError> { self.set_property(store, $name, value, Some(""), limits, resolver) }
        )+}
        impl CssomCapture<$facet> {$(
            pub fn $get(&self, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> { self.get_property_value($name, limits) }
        )+}
    };
}
named_declarations!(CssPageDescriptors;
    margin, set_margin => "margin",
    margin_top, set_margin_top => "margin-top",
    margin_right, set_margin_right => "margin-right",
    margin_bottom, set_margin_bottom => "margin-bottom",
    margin_left, set_margin_left => "margin-left",
    size, set_size => "size",
    page_orientation, set_page_orientation => "page-orientation",
    marks, set_marks => "marks",
    bleed, set_bleed => "bleed",
);
named_declarations!(CssFontFaceDescriptors;
    src, set_src => "src",
    font_family, set_font_family => "font-family",
    font_style, set_font_style => "font-style",
    font_weight, set_font_weight => "font-weight",
    font_stretch, set_font_stretch => "font-stretch",
    font_width, set_font_width => "font-width",
    unicode_range, set_unicode_range => "unicode-range",
    font_feature_settings, set_font_feature_settings => "font-feature-settings",
    font_variation_settings, set_font_variation_settings => "font-variation-settings",
    font_named_instance, set_font_named_instance => "font-named-instance",
    font_display, set_font_display => "font-display",
    font_language_override, set_font_language_override => "font-language-override",
    ascent_override, set_ascent_override => "ascent-override",
    descent_override, set_descent_override => "descent-override",
    line_gap_override, set_line_gap_override => "line-gap-override",
);

impl CssStyleProperties {
    /// The float reflection belongs only to property declarations.
    ///
    /// ```compile_fail,E0599
    /// use surgeist_cssom::{CssFontFaceDescriptors, CssomStore};
    /// fn invalid(value: &CssFontFaceDescriptors, store: &CssomStore) {
    ///     let _ = value.css_float(store, Default::default());
    /// }
    /// ```
    /// ```compile_fail,E0599
    /// use surgeist_cssom::{CssPageDescriptors, CssomStore};
    /// fn invalid(value: &CssPageDescriptors, store: &CssomStore) {
    ///     let _ = value.css_float(store, Default::default());
    /// }
    /// ```
    /// ```compile_fail,E0599
    /// use surgeist_cssom::{CssMarginDescriptors, CssomStore};
    /// fn invalid(value: &CssMarginDescriptors, store: &CssomStore) {
    ///     let _ = value.css_float(store, Default::default());
    /// }
    /// ```
    /// ```compile_fail,E0599
    /// use surgeist_cssom::{CssStyleDeclaration, CssomStore};
    /// fn invalid(value: &CssStyleDeclaration, store: &CssomStore) {
    ///     let _ = value.css_float(store, Default::default());
    /// }
    /// ```
    /// ```compile_fail,E0599
    /// use surgeist_cssom::{CssFontFaceDescriptors, CssomStore};
    /// fn invalid(value: &CssFontFaceDescriptors, store: &CssomStore) {
    ///     let _ = value.width(store, Default::default());
    /// }
    /// ```
    /// ```compile_fail,E0599
    /// use surgeist_cssom::{CssPageDescriptors, CssomStore};
    /// fn invalid(value: &CssPageDescriptors, store: &CssomStore) {
    ///     let _ = value.width(store, Default::default());
    /// }
    /// ```
    /// ```compile_fail,E0599
    /// use surgeist_cssom::{CssMarginDescriptors, CssomStore};
    /// fn invalid(value: &CssMarginDescriptors, store: &CssomStore) {
    ///     let _ = value.margin(store, Default::default());
    /// }
    /// ```
    pub fn css_float(
        &self,
        store: &CssomStore,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        self.get_property_value(store, "float", limits)
    }
    pub fn set_css_float(
        &self,
        store: &mut CssomStore,
        value: &str,
        limits: CssomDeclarationRequestLimits,
        resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        self.set_property(store, "float", value, Some(""), limits, resolver)
    }
}
impl CssomCapture<CssStyleProperties> {
    /// Captured property reflections read this snapshot and have no setters.
    ///
    /// ```compile_fail,E0599
    /// use surgeist_cssom::{CssomCapture, CssStyleProperties, CssomStore};
    /// fn invalid(value: &CssomCapture<CssStyleProperties>, store: &mut CssomStore) {
    ///     let _ = value.remove_property(store, "float", Default::default());
    /// }
    /// ```
    pub fn css_float(&self, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> {
        self.get_property_value("float", limits)
    }
}
