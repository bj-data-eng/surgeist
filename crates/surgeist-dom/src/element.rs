use crate::name::AttributeStorage;
use crate::names::qualified_equals;
use crate::store::payload::Payload;
use crate::store::source::formation::{self, Formation};
use crate::{
    DocumentString, DomAttribute, DomError, DomQualifiedName, DomRegistryBinding,
    HtmlCustomElementState, Namespace,
};
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::AtomicBool;
use surgeist_html::{
    HtmlCanvasNoneState, HtmlCommonState, HtmlElementClass as Requirement, HtmlIframeInitialState,
    HtmlMarqueeState, KnownHtmlElement,
};
mod family;
mod groups;
#[cfg(test)]
mod tests;
pub(crate) use family::{IframeState, NativeFamily};
pub(crate) use groups::{
    AttributeBinding, CustomAssociation, ElementChildren, ElementFormation, ElementName,
    FocusReference, FormedElement, HostedContent, NonceSlot, NonceUpdate,
};

fn original_string(
    value: &DocumentString,
    budget: Option<&Formation>,
) -> Result<crate::store::source::storage::Payload, DomError> {
    let mut usage = crate::store::source::storage::Payload::default();
    if budget.is_some() {
        usage.string(value)?;
    }
    Ok(usage)
}

fn empty_string_payload(count: usize) -> Result<crate::store::source::storage::Payload, DomError> {
    Ok(crate::store::source::storage::Payload {
        bytes: size_of::<Vec<u16>>()
            .checked_mul(count)
            .ok_or(DomError::RepresentationExhausted)?,
        units: 0,
    })
}

const CAPTURED: u8 = 1;
const FULLSCREEN: u8 = 2;
const KEYBOARD_LOCK: u8 = 4;

macro_rules! native_payload {
    ($get:ident, $set:ident, $variant:ident, $state:ty) => {
        pub(crate) fn $get(&self) -> Option<&Arc<Payload<$state>>> {
            match &self.family {
                NativeFamily::$variant(value) => Some(value),
                _ => None,
            }
        }
        pub(crate) fn $set(&mut self, value: Arc<Payload<$state>>) {
            assert!(
                matches!(self.family, NativeFamily::$variant(_)),
                "qualified native family replacement"
            );
            self.family = NativeFamily::$variant(value);
        }
    };
}

/// Definition association established by this native generic creation surface.
/// Registry/definition object admission is a separate, unexposed capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomElementDefinition {
    Null,
}

/// Canonical supported native Element state, captured with its owner and tree.
/// Private fields prevent bypassing name validation and native initialization.
#[derive(Clone, Debug)]
pub struct DomElement {
    name: Arc<Payload<ElementName>>,
    attributes: Option<Arc<Payload<AttributeBinding>>>,
    custom_state: HtmlCustomElementState,
    common_scope: surgeist_html::HtmlCommonScope,
    custom: Option<Arc<Payload<CustomAssociation>>>,
    nonce: Option<Arc<Payload<NonceSlot>>>,
    focus: Option<Arc<Payload<FocusReference<crate::NodeId>>>>,
    family: NativeFamily,
    hosted: Option<Arc<Payload<HostedContent>>>,
    flags: u8,
}

impl PartialEq for DomElement {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.attributes() == other.attributes()
            && self.custom_element_registry() == other.custom_element_registry()
            && self.custom_element_definition() == other.custom_element_definition()
            && self.saved_is() == other.saved_is()
            && self.custom_state == other.custom_state
            && self.common_scope == other.common_scope
            && self.focus == other.focus
            && self.nonce == other.nonce
            && self.family == other.family
            && self.hosted == other.hosted
            && self.flags == other.flags
    }
}
impl Eq for DomElement {}

impl DomElement {
    pub(crate) fn name_payload(&self) -> &Arc<Payload<ElementName>> {
        &self.name
    }
    pub(crate) fn custom_payload(&self) -> Option<&Arc<Payload<CustomAssociation>>> {
        self.custom.as_ref()
    }
    pub(crate) fn nonce_payload(&self) -> Option<&Arc<Payload<NonceSlot>>> {
        self.nonce.as_ref()
    }
    pub(crate) fn focus_payload(&self) -> Option<&Arc<Payload<FocusReference<crate::NodeId>>>> {
        self.focus.as_ref()
    }
    pub(crate) fn hosted_payload(&self) -> Option<&Arc<Payload<HostedContent>>> {
        self.hosted.as_ref()
    }
    pub(crate) fn native_family(&self) -> &NativeFamily {
        &self.family
    }
    pub(crate) fn set_focus_payload(
        &mut self,
        value: Option<Arc<Payload<FocusReference<crate::NodeId>>>>,
    ) {
        assert!(
            value.is_none() || self.common_scope == surgeist_html::HtmlCommonScope::Html,
            "stored focus applicability"
        );
        self.focus = value;
    }
    native_payload!(
        input_payload,
        set_input_payload,
        Input,
        crate::DomInputState
    );
    native_payload!(
        textarea_payload,
        set_textarea_payload,
        Textarea,
        crate::DomTextareaState
    );
    native_payload!(
        output_payload,
        set_output_payload,
        Output,
        crate::DomOutputState
    );
    native_payload!(
        select_payload,
        set_select_payload,
        Select,
        crate::DomSelectState
    );
    native_payload!(
        fieldset_payload,
        set_fieldset_payload,
        Fieldset,
        crate::DomFieldsetState
    );
    native_payload!(
        button_payload,
        set_button_payload,
        Button,
        crate::DomButtonState
    );
    pub(crate) fn style_payload(&self) -> Option<&Arc<Payload<crate::DomStyleState>>> {
        match &self.family {
            NativeFamily::Style(value) => Some(value),
            _ => None,
        }
    }
    pub(crate) fn dialog_payload(&self) -> Option<&Arc<Payload<crate::dialog::DomDialogState>>> {
        match &self.family {
            NativeFamily::Dialog(value) => Some(value),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn attributes_payload(
        &self,
    ) -> Result<crate::store::source::storage::Payload, DomError> {
        Ok(self.attributes_backing().map_or_else(
            crate::store::source::storage::Payload::default,
            AttributeStorage::text_payload,
        ))
    }
    pub(crate) fn retained_payload(
        &self,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<crate::store::source::storage::Payload, DomError> {
        crate::store::source::storage::element(self, &self.family, step)
    }
    /// Completed retained materialization. The importer retains its native
    /// control until this result is attached to the receiving language payload.
    #[expect(
        clippy::too_many_arguments,
        reason = "Independent parsed facts and already formed child owners enter one atomic Element formation; grouping them would add an artificial staging record"
    )]
    pub(crate) fn from_parsed(
        name: Arc<Payload<ElementName>>,
        attributes: Option<Arc<Payload<AttributeBinding>>>,
        custom_state: HtmlCustomElementState,
        saved_is: Option<DocumentString>,
        nonce: Option<DocumentString>,
        family: NativeFamily,
        common: HtmlCommonState<crate::NodeId>,
        mut formation: ElementFormation<'_>,
    ) -> Result<FormedElement, DomError> {
        let custom = saved_is
            .map(|value| {
                let original = original_string(&value, formation.budget)?;
                formation.custom(
                    CustomAssociation {
                        registry: DomRegistryBinding::Null,
                        definition: DomElementDefinition::Null,
                        saved_is: Some(value),
                    },
                    original,
                )
            })
            .transpose()?;
        let nonce = nonce
            .map(|value| {
                let original = original_string(&value, formation.budget)?;
                formation.nonce(value, original)
            })
            .transpose()?;
        let focus = common
            .html()
            .and_then(|html| html.previously_focused_element())
            .copied()
            .map(|value| formation.focus(value))
            .transpose()?;
        let value = Self {
            name,
            attributes,
            custom_state,
            common_scope: common.scope(),
            custom,
            nonce,
            focus,
            family,
            hosted: None,
            flags: 0,
        };
        Ok(formation.finish(value))
    }

    /// The name target is already formed and sealed by the ingress owner.
    pub(crate) fn native(
        name: Arc<Payload<ElementName>>,
        environment: crate::DomDocumentEnvironment,
        formation: ElementFormation<'_>,
    ) -> Result<FormedElement, DomError> {
        Self::initialize(name, None, environment, formation)
    }

    /// Clone initialization creates fresh native state; it never interns native
    /// identities by value. The caller supplies the new name occurrence.
    pub(crate) fn clone_initialized(
        &self,
        name: Arc<Payload<ElementName>>,
        environment: crate::DomDocumentEnvironment,
        formation: ElementFormation<'_>,
    ) -> Result<FormedElement, DomError> {
        let saved_is = self
            .saved_is()
            .map(|value| formation::share(value, formation.budget, formation.cancel))
            .transpose()?;
        Self::initialize(name, saved_is, environment, formation)
    }

    /// Construct only the changed binding and feature bodies. An unrelated
    /// attribute edit aliases its original name, native state and nonce.
    pub(crate) fn with_attributes_formed(
        &self,
        attributes: Option<Arc<Payload<AttributeBinding>>>,
        changed: &DomQualifiedName,
        nonce: NonceUpdate,
        mut formation: ElementFormation<'_>,
    ) -> Result<FormedElement, DomError> {
        crate::store::check_cancel(formation.cancel)?;
        let budget = formation.budget;
        let cancel = formation.cancel;
        let family = self.family.after_attribute(
            family::AttributeMutation {
                element_name: self.name(),
                name: changed,
                previous: self.attributes(),
                attributes: attributes
                    .as_ref()
                    .map_or(&[], |binding| binding.value.storage.as_slice()),
                content: self.child_navigable_presence(),
            },
            &mut || formation::step(budget, cancel),
            &mut formation,
        )?;
        let nonce = match nonce {
            NonceUpdate::Unchanged => self.nonce.clone(),
            NonceUpdate::Set(value) => {
                let original = original_string(&value, formation.budget)?;
                Some(formation.nonce(value, original)?)
            }
        };
        let value = Self {
            attributes,
            nonce,
            family,
            ..self.clone()
        };
        Ok(formation.finish(value))
    }

    fn initialize(
        name: Arc<Payload<ElementName>>,
        saved_is: Option<DocumentString>,
        environment: crate::DomDocumentEnvironment,
        mut formation: ElementFormation<'_>,
    ) -> Result<FormedElement, DomError> {
        let budget = formation.budget;
        let cancelled = formation.cancel;
        let initialization =
            surgeist_html::bounded::native_element_initialization_with(&name.value.value, |_| {
                formation::step(budget, cancelled)
            })?;
        if environment.has_ever_had_navigable()
            && matches!(
                initialization.requirement(),
                Requirement::KnownHtml(KnownHtmlElement::Iframe)
                    | Requirement::HtmlFrame
                    | Requirement::SvgScript
            )
        {
            // Child creation and SVG execution/notification owners are not
            // supplied by the admitted native Global/Details profile.
            return Err(DomError::NativeInitializationRequired(
                initialization.requirement(),
            ));
        }
        let family = match initialization.requirement() {
            Requirement::GenericElement
            | Requirement::CommonHtml
            | Requirement::CommonSvg
            | Requirement::CommonMathml => NativeFamily::Common,
            Requirement::KnownHtml(KnownHtmlElement::Form) => {
                NativeFamily::Form(surgeist_html::HtmlFormNativeState::fresh())
            }
            Requirement::KnownHtml(KnownHtmlElement::Template) => NativeFamily::Template,
            Requirement::KnownHtml(KnownHtmlElement::Canvas) => {
                NativeFamily::Canvas(formation.native(
                    empty_string_payload(0)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(HtmlCanvasNoneState::fresh()),
                )?)
            }
            Requirement::KnownHtml(KnownHtmlElement::Iframe) => {
                // Every admitted target Document has the actual native never-BC
                // environment. This is fresh element creation, not reconstruction
                // from current null content or missing imported metadata.
                NativeFamily::Iframe(family::IframeState::fresh())
            }
            Requirement::KnownHtml(KnownHtmlElement::Script) => NativeFamily::Script,
            Requirement::SvgScript => {
                NativeFamily::SvgScript(crate::svg_script::SvgScriptState::fresh())
            }
            Requirement::KnownHtml(KnownHtmlElement::Slot) => {
                let Some(surgeist_html::HtmlNativeFreshSlot::EmptyManualAssignments) =
                    initialization.slot()
                else {
                    unreachable!("actual Slot initialization evidence")
                };
                NativeFamily::Slot
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Button) => {
                NativeFamily::Button(formation.native(
                    empty_string_payload(1)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(crate::DomButtonState::fresh()),
                )?)
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Fieldset) => {
                NativeFamily::Fieldset(formation.native(
                    empty_string_payload(1)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(crate::DomFieldsetState::fresh()),
                )?)
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Input) => {
                NativeFamily::Input(formation.native(
                    empty_string_payload(2)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(crate::DomInputState::fresh(environment.input_platform())),
                )?)
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Textarea) => {
                NativeFamily::Textarea(formation.native(
                    empty_string_payload(2)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(crate::DomTextareaState::fresh(environment.input_platform())),
                )?)
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Output) => {
                NativeFamily::Output(formation.native(
                    empty_string_payload(1)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(crate::DomOutputState::fresh()),
                )?)
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Select) => {
                NativeFamily::Select(formation.native(
                    empty_string_payload(1)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(crate::DomSelectState::fresh()),
                )?)
            }
            Requirement::KnownHtml(KnownHtmlElement::Option) => {
                NativeFamily::Option(crate::DomOptionState::default())
            }
            Requirement::KnownHtml(KnownHtmlElement::Dialog) => {
                NativeFamily::Dialog(formation.native(
                    empty_string_payload(1)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(crate::dialog::DomDialogState::fresh()),
                )?)
            }
            Requirement::KnownHtml(KnownHtmlElement::Details) => NativeFamily::Details,
            Requirement::KnownHtml(KnownHtmlElement::Meta) => NativeFamily::Meta,
            Requirement::KnownHtml(KnownHtmlElement::Base) => NativeFamily::Base,
            Requirement::KnownHtml(KnownHtmlElement::Link) => NativeFamily::Link,
            Requirement::KnownHtml(KnownHtmlElement::Style) => {
                NativeFamily::Style(formation.native(
                    empty_string_payload(0)?,
                    crate::store::source::storage::Payload::default(),
                    || Ok(crate::DomStyleState::fresh()),
                )?)
            }
            Requirement::KnownHtml(KnownHtmlElement::A | KnownHtmlElement::Area) => {
                NativeFamily::Hyperlink
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Image) => {
                NativeFamily::Img
            }
            Requirement::KnownHtml(KnownHtmlElement::Picture) => NativeFamily::Picture,
            Requirement::KnownHtml(KnownHtmlElement::Source) => NativeFamily::Source,
            Requirement::KnownHtml(KnownHtmlElement::Optgroup) => NativeFamily::Optgroup,
            Requirement::KnownHtml(KnownHtmlElement::Selectedcontent) => {
                NativeFamily::Selectedcontent(false)
            }
            Requirement::HtmlMarquee => NativeFamily::Marquee(HtmlMarqueeState::fresh()),
            Requirement::HtmlFrame => NativeFamily::Frame,
            requirement => return Err(DomError::NativeInitializationRequired(requirement)),
        };
        let common: HtmlCommonState<crate::NodeId> =
            initialization.common().map(|reference| match reference {});
        let html = common.html().is_some();
        let custom_name = html
            && (saved_is.is_some()
                || surgeist_html::bounded::validate_custom_element_name_with(
                    name.value.value.local_name(),
                    |_| formation::step(budget, cancelled),
                )?
                .is_ok());
        let custom = saved_is
            .map(|value| {
                let original = original_string(&value, formation.budget)?;
                formation.custom(
                    CustomAssociation {
                        registry: DomRegistryBinding::Null,
                        definition: DomElementDefinition::Null,
                        saved_is: Some(value),
                    },
                    original,
                )
            })
            .transpose()?;
        let nonce = if initialization.nonce().is_some() {
            Some(formation.fresh_nonce()?)
        } else {
            None
        };
        let value = Self {
            name,
            attributes: None,
            custom_state: surgeist_html::custom_element_state_without_definition(html, custom_name),
            common_scope: common.scope(),
            custom,
            nonce,
            focus: None,
            family,
            hosted: None,
            flags: 0,
        };
        Ok(formation.finish(value))
    }
    pub(crate) fn is_template(&self) -> bool {
        matches!(self.family, NativeFamily::Template)
    }

    pub fn form_state(&self) -> Option<surgeist_html::HtmlFormNativeState> {
        match self.family {
            NativeFamily::Form(state) => Some(state),
            _ => None,
        }
    }
    /// Actual stored Fieldset state, or inapplicability for another family.
    pub fn fieldset_state(&self) -> Option<&crate::DomFieldsetState> {
        match &self.family {
            NativeFamily::Fieldset(state) => Some(&state.value),
            _ => None,
        }
    }
    /// Actual stored Button state; raw weak slots belong to its snapshot view.
    pub fn button_state(&self) -> Option<&crate::DomButtonState> {
        match &self.family {
            NativeFamily::Button(state) => Some(&state.value),
            _ => None,
        }
    }
    pub fn input_state(&self) -> Option<&crate::DomInputState> {
        match &self.family {
            NativeFamily::Input(state) => Some(&state.value),
            _ => None,
        }
    }
    pub fn textarea_state(&self) -> Option<&crate::DomTextareaState> {
        match &self.family {
            NativeFamily::Textarea(state) => Some(&state.value),
            _ => None,
        }
    }
    pub fn output_state(&self) -> Option<&crate::DomOutputState> {
        match &self.family {
            NativeFamily::Output(state) => Some(&state.value),
            _ => None,
        }
    }
    pub(crate) fn is_base(&self) -> bool {
        matches!(self.family, NativeFamily::Base)
    }
    pub(crate) fn is_hyperlink(&self) -> bool {
        matches!(self.family, NativeFamily::Hyperlink)
    }
    pub(crate) fn is_img(&self) -> bool {
        matches!(self.family, NativeFamily::Img)
    }
    pub(crate) fn image_tree_element(&self) -> Option<KnownHtmlElement> {
        match self.family {
            NativeFamily::Img => Some(KnownHtmlElement::Img),
            NativeFamily::Picture => Some(KnownHtmlElement::Picture),
            NativeFamily::Source => Some(KnownHtmlElement::Source),
            _ => KnownHtmlElement::from_name(self.name())
                .filter(|tag| matches!(tag, KnownHtmlElement::Audio | KnownHtmlElement::Video)),
        }
    }
    pub(crate) fn set_style_payload(&mut self, value: Arc<Payload<crate::DomStyleState>>) {
        assert!(
            matches!(self.family, NativeFamily::Style(_)),
            "checked Style"
        );
        self.family = NativeFamily::Style(value);
    }
    pub(crate) fn style_state(&self) -> Option<&crate::DomStyleState> {
        match &self.family {
            NativeFamily::Style(state) => Some(&state.value),
            _ => None,
        }
    }
    pub(crate) fn dialog_state(&self) -> Option<&crate::dialog::DomDialogState> {
        match &self.family {
            NativeFamily::Dialog(state) => Some(&state.value),
            _ => None,
        }
    }
    pub(crate) fn set_dialog_payload(
        &mut self,
        value: Arc<Payload<crate::dialog::DomDialogState>>,
    ) {
        assert!(
            matches!(self.family, NativeFamily::Dialog(_)),
            "checked Dialog"
        );
        self.family = NativeFamily::Dialog(value);
    }
    pub(crate) fn is_link(&self) -> bool {
        matches!(self.family, NativeFamily::Link)
    }
    pub(crate) fn is_details(&self) -> bool {
        matches!(self.family, NativeFamily::Details)
    }
    pub(crate) fn is_meta(&self) -> bool {
        matches!(self.family, NativeFamily::Meta)
    }
    /// Native Details state is actual exact no-namespace open presence.
    pub fn details_state(&self) -> Option<surgeist_html::HtmlDetailsState> {
        if !self.is_details() {
            return None;
        }
        Some(
            if self.attributes().iter().any(|a| {
                a.name().namespace_uri().is_none()
                    && a.name().local_name().code_units() == [0x6f, 0x70, 0x65, 0x6e]
            }) {
                surgeist_html::HtmlDetailsState::Open
            } else {
                surgeist_html::HtmlDetailsState::Closed
            },
        )
    }
    pub fn select_state(&self) -> Option<&crate::DomSelectState> {
        match &self.family {
            NativeFamily::Select(s) => Some(&s.value),
            _ => None,
        }
    }
    pub fn option_state(&self) -> Option<crate::DomOptionState> {
        match self.family {
            NativeFamily::Option(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn option_state_mut(&mut self) -> Option<&mut crate::DomOptionState> {
        match &mut self.family {
            NativeFamily::Option(s) => Some(s),
            _ => None,
        }
    }
    pub fn selectedcontent_disabled(&self) -> Option<bool> {
        match self.family {
            NativeFamily::Selectedcontent(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn selectedcontent_disabled_mut(&mut self) -> Option<&mut bool> {
        match &mut self.family {
            NativeFamily::Selectedcontent(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn native_control(
        &self,
    ) -> Option<(
        &crate::control::NativeControlState,
        surgeist_html::HtmlNativeFormAssociationKind,
    )> {
        match &self.family {
            NativeFamily::Select(state) => Some((
                &state.value.control,
                surgeist_html::HtmlNativeFormAssociationKind::Select,
            )),
            NativeFamily::Fieldset(state) => Some((&state.value.control, state.value.kind())),
            NativeFamily::Button(state) => Some((&state.value.control, state.value.kind())),
            NativeFamily::Input(state) => Some((
                &state.value.control,
                surgeist_html::HtmlNativeFormAssociationKind::Input,
            )),
            NativeFamily::Output(state) => Some((
                &state.value.control,
                surgeist_html::HtmlNativeFormAssociationKind::Output,
            )),
            NativeFamily::Textarea(state) => Some((
                &state.value.control,
                surgeist_html::HtmlNativeFormAssociationKind::Textarea,
            )),
            _ => None,
        }
    }
    pub(crate) fn is_slot(&self) -> bool {
        matches!(self.family, NativeFamily::Slot)
    }
    pub(crate) fn svg_script_state(&self) -> Option<&crate::svg_script::SvgScriptState> {
        match &self.family {
            NativeFamily::SvgScript(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn svg_script_state_mut(
        &mut self,
    ) -> Option<&mut crate::svg_script::SvgScriptState> {
        match &mut self.family {
            NativeFamily::SvgScript(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn is_html_script(&self) -> bool {
        matches!(self.family, NativeFamily::Script)
    }
    /// Actual current common values with canonical element references.
    pub fn common_state(&self) -> HtmlCommonState<crate::NodeId> {
        let mut common = HtmlCommonState::new(self.common_scope);
        if let Some(focus) = &self.focus {
            common
                .html_mut()
                .expect("stored HTML focus applicability")
                .remember_focused_element(Some(focus.value.value));
        }
        common
    }
    /// Cryptographic nonce slot, independent of content. None means inapplicable.
    pub fn nonce(&self) -> Option<&DocumentString> {
        self.nonce.as_ref().map(|nonce| &nonce.value.value)
    }
    /// Actual none-mode Canvas state, or inapplicability for another family.
    pub fn canvas_none_state(&self) -> Option<&HtmlCanvasNoneState> {
        match &self.family {
            NativeFamily::Canvas(state) => Some(&state.value),
            _ => None,
        }
    }
    /// Actual native Marquee state, including its closed zero loop index.
    pub fn marquee_state(&self) -> Option<HtmlMarqueeState> {
        match self.family {
            NativeFamily::Marquee(state) => Some(state),
            _ => None,
        }
    }
    /// Actual content navigable for Frame/Iframe. Outer None is inapplicable;
    /// Some(None) is the stored null relationship, not a creation/history proof.
    pub(crate) fn set_hosted_payload(
        &mut self,
        value: Option<Arc<Payload<HostedContent>>>,
    ) -> Result<(), DomError> {
        if self.child_navigable().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        self.hosted = value;
        Ok(())
    }
    pub fn child_navigable(&self) -> Option<Option<crate::DomHostedNavigableId>> {
        matches!(self.family, NativeFamily::Frame | NativeFamily::Iframe(_))
            .then(|| self.hosted.as_ref().map(|value| value.value.value))
    }
    /// Local guards consume this projection of the one canonical relationship.
    pub fn child_navigable_presence(&self) -> Option<surgeist_html::HtmlChildNavigablePresence> {
        self.child_navigable().map(|content| match content {
            None => surgeist_html::HtmlChildNavigablePresence::Null,
            Some(_) => surgeist_html::HtmlChildNavigablePresence::Present,
        })
    }
    /// Independent retained intrinsic Iframe tuple, without child identity.
    pub fn iframe_initial_state(&self) -> Option<HtmlIframeInitialState> {
        match self.family {
            NativeFamily::Iframe(state) => Some(state.initial),
            _ => None,
        }
    }
    pub(crate) fn iframe_sandbox_flags(&self) -> Option<surgeist_html::HtmlSandboxFlags> {
        match self.family {
            NativeFamily::Iframe(state) => Some(state.sandbox),
            _ => None,
        }
    }
    pub(crate) fn establish_iframe_sandbox_flags(
        &mut self,
        sandbox: surgeist_html::HtmlSandboxFlags,
    ) -> Result<(), DomError> {
        let NativeFamily::Iframe(state) = &mut self.family else {
            return Err(DomError::WrongNodeKind);
        };
        state.sandbox = sandbox;
        Ok(())
    }
    pub(crate) fn set_marquee_running(&mut self, running: bool) -> Result<(), DomError> {
        let NativeFamily::Marquee(state) = &mut self.family else {
            return Err(DomError::MarqueeInapplicable);
        };
        if running {
            state.start();
        } else {
            state.stop();
        }
        Ok(())
    }
    pub(crate) fn set_nonce_payload(&mut self, nonce: Arc<Payload<NonceSlot>>) {
        self.nonce = Some(nonce);
    }
    pub fn captured_in_view_transition(&self) -> bool {
        self.flags & CAPTURED != 0
    }
    pub fn fullscreen_flag(&self) -> bool {
        self.flags & FULLSCREEN != 0
    }
    pub fn keyboard_lock_flag(&self) -> bool {
        self.flags & KEYBOARD_LOCK != 0
    }
    pub fn iframe_fullscreen_flag(&self) -> Option<bool> {
        match &self.family {
            NativeFamily::Iframe(state) => Some(state.fullscreen),
            _ => None,
        }
    }
    pub(crate) fn set_fullscreen_flag(&mut self) {
        self.flags |= FULLSCREEN;
    }
    pub(crate) fn clear_fullscreen_flags(&mut self) {
        self.flags &= !(FULLSCREEN | KEYBOARD_LOCK);
        if let NativeFamily::Iframe(state) = &mut self.family {
            state.fullscreen = false;
        }
    }
    pub(crate) fn set_captured_in_view_transition(&mut self, captured: bool) {
        if captured {
            self.flags |= CAPTURED;
        } else {
            self.flags &= !CAPTURED;
        }
    }
    pub(crate) fn same_nonattribute_state(&self, other: &Self) -> bool {
        self.name == other.name
            && self.custom_element_registry() == other.custom_element_registry()
            && self.custom_element_definition() == other.custom_element_definition()
            && self.saved_is() == other.saved_is()
            && self.custom_state == other.custom_state
            && self.common_scope == other.common_scope
            && self.focus == other.focus
            && self.nonce == other.nonce
            && self.family == other.family
            && self.fullscreen_flag() == other.fullscreen_flag()
            && self.keyboard_lock_flag() == other.keyboard_lock_flag()
    }
    pub fn name(&self) -> &DomQualifiedName {
        &self.name.value.value
    }
    pub fn attributes(&self) -> &[DomAttribute] {
        self.attributes
            .as_ref()
            .map_or(&[], |value| value.value.storage.as_slice())
    }
    pub(crate) fn attribute_binding(&self) -> Option<&Arc<Payload<AttributeBinding>>> {
        self.attributes.as_ref()
    }
    pub(crate) fn attributes_backing(&self) -> Option<&AttributeStorage> {
        self.attributes.as_ref().map(|value| &value.value.storage)
    }
    pub(crate) fn same_attribute_storage(&self, other: &Self) -> bool {
        match (self.attributes_backing(), other.attributes_backing()) {
            (None, None) => true,
            (Some(left), Some(right)) => left.same_storage(right),
            _ => false,
        }
    }
    pub fn custom_element_registry(&self) -> DomRegistryBinding {
        self.custom
            .as_ref()
            .map_or(DomRegistryBinding::Null, |value| value.value.registry)
    }
    pub(crate) fn set_custom_payload(&mut self, value: Option<Arc<Payload<CustomAssociation>>>) {
        self.custom = value;
    }
    pub fn custom_element_definition(&self) -> DomElementDefinition {
        self.custom
            .as_ref()
            .map_or(DomElementDefinition::Null, |value| value.value.definition)
    }
    pub fn custom_element_state(&self) -> HtmlCustomElementState {
        self.custom_state
    }
    /// Saved creation is independent of the current content attribute.
    pub fn saved_is(&self) -> Option<&DocumentString> {
        self.custom
            .as_ref()
            .and_then(|value| value.value.saved_is.as_ref())
    }
    /// First exact qualified spelling, without namespace interpretation.
    pub fn attribute(&self, qualified_name: &DocumentString) -> Option<&DocumentString> {
        self.attributes()
            .iter()
            .find(|attribute| qualified_equals(attribute.name(), qualified_name))
            .map(|attribute| attribute.value())
    }
    pub fn attribute_ns(
        &self,
        namespace: &Namespace,
        local_name: &DocumentString,
    ) -> Option<&DocumentString> {
        self.attributes()
            .iter()
            .find(|attribute| {
                attribute.name().namespace_uri() == namespace.uri()
                    && attribute.name().local_name() == local_name
            })
            .map(|attribute| attribute.value())
    }
    pub(crate) fn family_units(&self) -> Option<usize> {
        self.family.units()
    }
    pub(crate) fn units(&self) -> Option<usize> {
        let mut units = self
            .name()
            .logical_units()?
            .checked_add(self.saved_is().map_or(0, DocumentString::len))?
            .checked_add(self.nonce().map_or(0, DocumentString::len))?;
        units = units.checked_add(self.family.units()?)?;
        units = units.checked_add(
            self.attributes_backing()
                .map_or(0, AttributeStorage::logical_units),
        )?;
        Some(units)
    }
}
