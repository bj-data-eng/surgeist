use crate::name::AttributeStorage;
use crate::names::qualified_equals;
use crate::store::source::formation::{self, Formation};
use crate::{
    DocumentString, DomAttribute, DomError, DomQualifiedName, DomRegistryBinding,
    HtmlCustomElementState, Namespace,
};
use std::sync::atomic::AtomicBool;
use surgeist_html::{
    HtmlCanvasNoneState, HtmlCommonState, HtmlElementClass as Requirement, HtmlIframeInitialState,
    HtmlMarqueeState, KnownHtmlElement,
};
mod family;
#[cfg(test)]
mod tests;
pub(crate) use family::{IframeState, NativeFamily};

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
    name: DomQualifiedName,
    attributes: AttributeStorage,
    registry: DomRegistryBinding,
    definition: DomElementDefinition,
    custom_state: HtmlCustomElementState,
    saved_is: Option<DocumentString>,
    common: HtmlCommonState<crate::NodeId>,
    nonce: Option<DocumentString>,
    family: NativeFamily,
    content_navigable: Option<crate::DomHostedNavigableId>,
    captured_in_view_transition: bool,
    fullscreen: bool,
    keyboard_lock: bool,
}

impl PartialEq for DomElement {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.registry == other.registry
            && self.definition == other.definition
            && self.custom_state == other.custom_state
            && self.saved_is == other.saved_is
            && self.common == other.common
            && self.nonce == other.nonce
            && self.family == other.family
            && self.content_navigable == other.content_navigable
            && self.captured_in_view_transition == other.captured_in_view_transition
            && self.fullscreen == other.fullscreen
            && self.keyboard_lock == other.keyboard_lock
    }
}
impl Eq for DomElement {}

impl DomElement {
    #[cfg(test)]
    pub(crate) fn inspect_compact_baseline(
        &self,
        graph: &mut crate::store::compact_tests::AllocationGraph,
    ) {
        self.name.inspect_allocations(graph);
        self.attributes.inspect_allocations(graph);
        for attribute in self.attributes.iter() {
            attribute.name().inspect_allocations(graph);
            graph.string(attribute.value());
        }
        for value in [self.saved_is.as_ref(), self.nonce.as_ref()]
            .into_iter()
            .flatten()
        {
            graph.string(value);
        }
        match &self.family {
            NativeFamily::Common | NativeFamily::Form(_) => {}
            NativeFamily::Canvas(value) => {
                assert_eq!(
                    **value,
                    HtmlCanvasNoneState::fresh(),
                    "this raw baseline covers the actual fresh Canvas body"
                );
                graph.target(value, "native.canvas");
            }
            _ => panic!(
                "compact baseline must explicitly describe this native family's owned allocations"
            ),
        }
    }

    pub(crate) fn attributes_payload(
        &self,
    ) -> Result<crate::store::source::storage::Payload, DomError> {
        Ok(self.attributes.text_payload())
    }
    pub(crate) fn retained_payload(
        &self,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<crate::store::source::storage::Payload, DomError> {
        crate::store::source::storage::element(self, &self.family, step)
    }
    /// Completed retained materialization. The crate-owned importer proves the
    /// ordinary-report/profile boundary; this does not replay native hooks.
    pub(crate) fn from_parsed(
        name: DomQualifiedName,
        attributes: AttributeStorage,
        custom_state: HtmlCustomElementState,
        saved_is: Option<DocumentString>,
        nonce: Option<DocumentString>,
        family: NativeFamily,
        common: HtmlCommonState<crate::NodeId>,
    ) -> Self {
        Self {
            name,
            attributes,
            registry: DomRegistryBinding::Null,
            definition: DomElementDefinition::Null,
            custom_state,
            saved_is,
            common,
            nonce,
            family,
            content_navigable: None,
            captured_in_view_transition: false,
            fullscreen: false,
            keyboard_lock: false,
        }
    }
    /// Initialize an already validated, owner-resolved and admitted name.
    /// Native ingress owns its original validation and case-processing rules.
    pub(crate) fn native(
        name: DomQualifiedName,
        environment: crate::DomDocumentEnvironment,
        cancelled: &AtomicBool,
        budget: Option<&Formation>,
    ) -> Result<Self, DomError> {
        Self::initialize(name, None, environment, cancelled, budget)
    }
    /// Initialize trusted captured name fields; copied attributes and the
    /// source nonce slot are applied separately in source cloning order.
    pub(crate) fn clone_initialized(
        &self,
        name: DomQualifiedName,
        environment: crate::DomDocumentEnvironment,
        cancelled: &AtomicBool,
        budget: Option<&Formation>,
    ) -> Result<Self, DomError> {
        Self::initialize(
            name,
            self.saved_is
                .as_ref()
                .map(|value| formation::share(value, budget, cancelled))
                .transpose()?,
            environment,
            cancelled,
            budget,
        )
    }
    fn shared_name(
        name: &DomQualifiedName,
        budget: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomQualifiedName, DomError> {
        if let Some(budget) = budget {
            budget.retain_payload(name.retained_payload()?, cancel)?;
        }
        Ok(name.clone())
    }
    /// Construct the actual candidate without retaining a superseded old
    /// attribute bank. The caller owns the already formed attributes and nonce
    /// under the same candidate lease until canonical transfer.
    pub(crate) fn with_attributes_formed(
        &self,
        attributes: AttributeStorage,
        changed: &DomQualifiedName,
        nonce: Option<DocumentString>,
        budget: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        crate::store::check_cancel(cancel)?;
        let name = Self::shared_name(&self.name, budget, cancel)?;
        let saved_is = self
            .saved_is
            .as_ref()
            .map(|value| formation::share(value, budget, cancel))
            .transpose()?;
        let family = self.family.after_attribute(
            family::AttributeMutation {
                element_name: &self.name,
                name: changed,
                previous: self.attributes.as_slice(),
                attributes: attributes.as_slice(),
                content: self.child_navigable_presence(),
            },
            &mut || formation::step(budget, cancel),
            budget,
            cancel,
        )?;
        Ok(Self {
            name,
            attributes,
            registry: self.registry,
            definition: self.definition,
            custom_state: self.custom_state,
            saved_is,
            common: self.common,
            nonce,
            family,
            content_navigable: self.content_navigable,
            captured_in_view_transition: self.captured_in_view_transition,
            fullscreen: self.fullscreen,
            keyboard_lock: self.keyboard_lock,
        })
    }
    fn initialize(
        name: DomQualifiedName,
        saved_is: Option<DocumentString>,
        environment: crate::DomDocumentEnvironment,
        cancelled: &AtomicBool,
        budget: Option<&Formation>,
    ) -> Result<Self, DomError> {
        let initialization =
            surgeist_html::bounded::native_element_initialization_with(&name, |_| {
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
        if let Some(budget) = budget {
            use std::mem::size_of;
            let string = size_of::<Vec<u16>>();
            let bytes = match initialization.requirement() {
                Requirement::FormAssociated(
                    surgeist_html::HtmlNativeFormAssociationKind::Button,
                )
                | Requirement::FormAssociated(
                    surgeist_html::HtmlNativeFormAssociationKind::Fieldset,
                ) => string,
                Requirement::FormAssociated(
                    surgeist_html::HtmlNativeFormAssociationKind::Input,
                ) => size_of::<crate::DomInputState>() + 2 * string,
                Requirement::FormAssociated(
                    surgeist_html::HtmlNativeFormAssociationKind::Textarea,
                ) => size_of::<crate::DomTextareaState>() + 2 * string,
                Requirement::FormAssociated(
                    surgeist_html::HtmlNativeFormAssociationKind::Output,
                ) => size_of::<crate::DomOutputState>() + string,
                Requirement::FormAssociated(
                    surgeist_html::HtmlNativeFormAssociationKind::Select,
                ) => size_of::<crate::DomSelectState>() + string,
                Requirement::KnownHtml(KnownHtmlElement::Dialog) => {
                    size_of::<crate::dialog::DomDialogState>() + string
                }
                Requirement::KnownHtml(KnownHtmlElement::Canvas) => {
                    size_of::<HtmlCanvasNoneState>()
                }
                Requirement::KnownHtml(KnownHtmlElement::Style) => {
                    size_of::<crate::DomStyleState>()
                }
                _ => 0,
            };
            budget.retain_bytes(bytes, cancelled)?;
            if initialization.nonce().is_some() {
                budget.retain_bytes(string, cancelled)?;
            }
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
                NativeFamily::Canvas(std::sync::Arc::new(HtmlCanvasNoneState::fresh()))
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
                NativeFamily::Button(crate::DomButtonState::fresh())
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Fieldset) => {
                NativeFamily::Fieldset(crate::DomFieldsetState::fresh())
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Input) => {
                NativeFamily::Input(std::sync::Arc::new(crate::DomInputState::fresh(
                    environment.input_platform(),
                )))
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Textarea) => {
                NativeFamily::Textarea(std::sync::Arc::new(crate::DomTextareaState::fresh(
                    environment.input_platform(),
                )))
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Output) => {
                NativeFamily::Output(std::sync::Arc::new(crate::DomOutputState::fresh()))
            }
            Requirement::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Select) => {
                NativeFamily::Select(std::sync::Arc::new(crate::DomSelectState::fresh()))
            }
            Requirement::KnownHtml(KnownHtmlElement::Option) => {
                NativeFamily::Option(crate::DomOptionState::default())
            }
            Requirement::KnownHtml(KnownHtmlElement::Dialog) => {
                NativeFamily::Dialog(std::sync::Arc::new(crate::dialog::DomDialogState::fresh()))
            }
            Requirement::KnownHtml(KnownHtmlElement::Details) => NativeFamily::Details,
            Requirement::KnownHtml(KnownHtmlElement::Meta) => NativeFamily::Meta,
            Requirement::KnownHtml(KnownHtmlElement::Base) => NativeFamily::Base,
            Requirement::KnownHtml(KnownHtmlElement::Link) => NativeFamily::Link,
            Requirement::KnownHtml(KnownHtmlElement::Style) => {
                NativeFamily::Style(std::sync::Arc::new(crate::DomStyleState::fresh()))
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
                    name.local_name(),
                    |_| formation::step(budget, cancelled),
                )?
                .is_ok());
        let attributes = AttributeStorage::default();
        Ok(Self {
            name,
            attributes,
            registry: DomRegistryBinding::Null,
            definition: DomElementDefinition::Null,
            custom_state: surgeist_html::custom_element_state_without_definition(html, custom_name),
            saved_is,
            common,
            nonce: initialization.nonce().map(|_| DocumentString::from("")),
            family,
            content_navigable: None,
            captured_in_view_transition: false,
            fullscreen: false,
            keyboard_lock: false,
        })
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
            NativeFamily::Fieldset(state) => Some(state),
            _ => None,
        }
    }
    /// Actual stored Button state; raw weak slots belong to its snapshot view.
    pub fn button_state(&self) -> Option<&crate::DomButtonState> {
        match &self.family {
            NativeFamily::Button(state) => Some(state),
            _ => None,
        }
    }
    pub fn input_state(&self) -> Option<&crate::DomInputState> {
        match &self.family {
            NativeFamily::Input(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn shared_family_payload(
        &self,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<crate::store::source::storage::Payload>, DomError> {
        let shared = match &self.family {
            NativeFamily::Input(state) => std::sync::Arc::strong_count(state) > 1,
            NativeFamily::Textarea(state) => std::sync::Arc::strong_count(state) > 1,
            NativeFamily::Output(state) => std::sync::Arc::strong_count(state) > 1,
            NativeFamily::Select(state) => std::sync::Arc::strong_count(state) > 1,
            _ => false,
        };
        if shared {
            crate::store::source::storage::family_payload(&self.family, step).map(Some)
        } else {
            Ok(None)
        }
    }
    pub(crate) fn make_family_unique(&mut self) {
        match &mut self.family {
            NativeFamily::Input(state) => {
                formation::make_arc_mut(state);
            }
            NativeFamily::Textarea(state) => {
                formation::make_arc_mut(state);
            }
            NativeFamily::Output(state) => {
                formation::make_arc_mut(state);
            }
            NativeFamily::Select(state) => {
                formation::make_arc_mut(state);
            }
            _ => {}
        }
    }
    pub(crate) fn input_state_mut(&mut self) -> Option<&mut crate::DomInputState> {
        match &mut self.family {
            NativeFamily::Input(state) => Some(formation::make_arc_mut(state)),
            _ => None,
        }
    }
    pub fn textarea_state(&self) -> Option<&crate::DomTextareaState> {
        match &self.family {
            NativeFamily::Textarea(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn textarea_state_mut(&mut self) -> Option<&mut crate::DomTextareaState> {
        match &mut self.family {
            NativeFamily::Textarea(state) => Some(formation::make_arc_mut(state)),
            _ => None,
        }
    }
    pub fn output_state(&self) -> Option<&crate::DomOutputState> {
        match &self.family {
            NativeFamily::Output(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn output_state_mut(&mut self) -> Option<&mut crate::DomOutputState> {
        match &mut self.family {
            NativeFamily::Output(state) => Some(formation::make_arc_mut(state)),
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
    pub(crate) fn set_style_state(&mut self, value: crate::DomStyleState) {
        match &mut self.family {
            NativeFamily::Style(state) => *state = formation::allocated_arc(value),
            _ => unreachable!("checked Style"),
        }
    }
    pub(crate) fn style_state(&self) -> Option<&crate::DomStyleState> {
        match &self.family {
            NativeFamily::Style(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn dialog_state(&self) -> Option<&crate::dialog::DomDialogState> {
        match &self.family {
            NativeFamily::Dialog(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn set_dialog_state(&mut self, value: crate::dialog::DomDialogState) {
        match &mut self.family {
            NativeFamily::Dialog(state) => *state = formation::allocated_arc(value),
            _ => unreachable!("checked Dialog"),
        }
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
            NativeFamily::Select(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn select_state_mut(&mut self) -> Option<&mut crate::DomSelectState> {
        match &mut self.family {
            NativeFamily::Select(s) => Some(formation::make_arc_mut(s)),
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
                &state.control,
                surgeist_html::HtmlNativeFormAssociationKind::Select,
            )),
            NativeFamily::Fieldset(state) => Some((&state.control, state.kind())),
            NativeFamily::Button(state) => Some((&state.control, state.kind())),
            NativeFamily::Input(state) => Some((
                &state.control,
                surgeist_html::HtmlNativeFormAssociationKind::Input,
            )),
            NativeFamily::Output(state) => Some((
                &state.control,
                surgeist_html::HtmlNativeFormAssociationKind::Output,
            )),
            NativeFamily::Textarea(state) => Some((
                &state.control,
                surgeist_html::HtmlNativeFormAssociationKind::Textarea,
            )),
            _ => None,
        }
    }
    pub(crate) fn native_control_mut(&mut self) -> Option<&mut crate::control::NativeControlState> {
        match &mut self.family {
            NativeFamily::Select(state) => Some(&mut formation::make_arc_mut(state).control),
            NativeFamily::Fieldset(state) => Some(&mut state.control),
            NativeFamily::Button(state) => Some(&mut state.control),
            NativeFamily::Input(state) => Some(&mut formation::make_arc_mut(state).control),
            NativeFamily::Textarea(state) => Some(&mut formation::make_arc_mut(state).control),
            NativeFamily::Output(state) => Some(&mut formation::make_arc_mut(state).control),
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
    pub(crate) fn remember_no_focused_element(&mut self) {
        self.common
            .html_mut()
            .expect("qualified native Dialog")
            .remember_focused_element(None);
    }
    /// Actual current common values with canonical element references.
    pub fn common_state(&self) -> HtmlCommonState<crate::NodeId> {
        self.common
    }
    /// Cryptographic nonce slot, independent of content. None means inapplicable.
    pub fn nonce(&self) -> Option<&DocumentString> {
        self.nonce.as_ref()
    }
    /// Actual none-mode Canvas state, or inapplicability for another family.
    pub fn canvas_none_state(&self) -> Option<&HtmlCanvasNoneState> {
        match &self.family {
            NativeFamily::Canvas(state) => Some(state),
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
    pub(crate) fn establish_child_navigable(
        &mut self,
        value: Option<crate::DomHostedNavigableId>,
    ) -> Result<(), crate::DomError> {
        if self.child_navigable().is_none() {
            return Err(crate::DomError::WrongNodeKind);
        }
        self.content_navigable = value;
        Ok(())
    }
    pub fn child_navigable(&self) -> Option<Option<crate::DomHostedNavigableId>> {
        matches!(self.family, NativeFamily::Frame | NativeFamily::Iframe(_))
            .then_some(self.content_navigable)
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
    pub(crate) fn set_nonce(&mut self, nonce: DocumentString) {
        self.nonce = Some(nonce);
    }
    pub fn captured_in_view_transition(&self) -> bool {
        self.captured_in_view_transition
    }
    pub fn fullscreen_flag(&self) -> bool {
        self.fullscreen
    }
    pub fn keyboard_lock_flag(&self) -> bool {
        self.keyboard_lock
    }
    pub fn iframe_fullscreen_flag(&self) -> Option<bool> {
        match &self.family {
            NativeFamily::Iframe(state) => Some(state.fullscreen),
            _ => None,
        }
    }
    pub(crate) fn set_fullscreen_flag(&mut self) {
        self.fullscreen = true;
    }
    pub(crate) fn clear_fullscreen_flags(&mut self) {
        self.fullscreen = false;
        self.keyboard_lock = false;
        if let NativeFamily::Iframe(state) = &mut self.family {
            state.fullscreen = false;
        }
    }
    pub(crate) fn set_captured_in_view_transition(&mut self, captured: bool) {
        self.captured_in_view_transition = captured;
    }
    pub(crate) fn same_nonattribute_state(&self, other: &Self) -> bool {
        self.name == other.name
            && self.registry == other.registry
            && self.definition == other.definition
            && self.custom_state == other.custom_state
            && self.saved_is == other.saved_is
            && self.common == other.common
            && self.nonce == other.nonce
            && self.family == other.family
            && self.fullscreen == other.fullscreen
            && self.keyboard_lock == other.keyboard_lock
    }
    pub fn name(&self) -> &DomQualifiedName {
        &self.name
    }
    pub fn attributes(&self) -> &[DomAttribute] {
        self.attributes.as_slice()
    }
    pub(crate) fn attributes_backing(&self) -> &AttributeStorage {
        &self.attributes
    }
    pub fn custom_element_registry(&self) -> DomRegistryBinding {
        self.registry
    }
    pub(crate) fn establish_registry(&mut self, registry: DomRegistryBinding) {
        self.registry = registry;
    }
    pub fn custom_element_definition(&self) -> DomElementDefinition {
        self.definition
    }
    pub fn custom_element_state(&self) -> HtmlCustomElementState {
        self.custom_state
    }
    /// Saved creation `is` is independent of the current content attribute.
    pub fn saved_is(&self) -> Option<&DocumentString> {
        self.saved_is.as_ref()
    }
    /// First exact qualified spelling, without namespace interpretation.
    pub fn attribute(&self, qualified_name: &DocumentString) -> Option<&DocumentString> {
        self.attributes
            .iter()
            .find(|attribute| qualified_equals(attribute.name(), qualified_name))
            .map(|attribute| attribute.value())
    }
    pub fn attribute_ns(
        &self,
        namespace: &Namespace,
        local_name: &DocumentString,
    ) -> Option<&DocumentString> {
        self.attributes
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
            .name
            .logical_units()?
            .checked_add(self.saved_is.as_ref().map_or(0, DocumentString::len))?
            .checked_add(self.nonce.as_ref().map_or(0, DocumentString::len))?;
        units = units.checked_add(self.family.units()?)?;
        units = units.checked_add(self.attributes.logical_units())?;
        Some(units)
    }
}
