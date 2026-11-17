//! Exact `Display` text and `source()` of every `DomError` variant. The table is the
//! independent oracle: each sentence states the documented meaning of its variant
//! (its doc comment, or the condition at its raise sites), not the `Debug` form.
use crate::{
    DocumentId, DomBaseFactsError, DomDocumentConstructorError, DomEncodingUrlFactsError, DomError,
    DomFullscreenExitFact, DomFullscreenExitId, DomFullscreenExitStep, DomFullscreenModelStep,
    DomInlineStyleChange, DomMetaError, DomOwnerId, DomParsedHtmlAssociationId, DomResource,
    DomRevision, DomSourceDesignModeStep, DomSourceInvariantFailure, DomSourceInvariantReason,
    DomSourceOperationId, DomStamp, DomViewTransitionCssomError, NodeId,
};
use std::error::Error;
use surgeist_css::color::{CssColorFactDefect, CssColorUnavailable, CssColorUnsupported};
use surgeist_html::bounded::lexical::HtmlPiPseudoAttributeError;
use surgeist_html::bounded::numeric::HtmlRangeNumericError;
use surgeist_html::elements::dialog::{
    HtmlCloseWatcherRequestAction, HtmlCloseWatcherRequestCursor, HtmlCloseWatcherRequestReply,
    HtmlDialogAction, HtmlDialogCursor, HtmlDialogFocusingAction, HtmlDialogFocusingCursor,
    HtmlDialogFocusingReply, HtmlDialogOperation, HtmlDialogReply,
};
use surgeist_html::elements::form::HtmlOutputCursorError;
use surgeist_html::elements::media::HtmlMediaResourceEffect;
use surgeist_html::elements::native::{
    HtmlDesignModeCursor, HtmlDesignModeInput, HtmlDesignModeReply, HtmlElementClass,
};
use surgeist_html::elements::style::HtmlStyleUpdateAction;
use surgeist_html::focus::{
    HtmlAutofocusDelegateCursor, HtmlAutofocusDelegateReply, HtmlAutofocusInsertionAction,
    HtmlAutofocusInsertionCursor, HtmlAutofocusInsertionReply, HtmlFocusDelegateCursor,
    HtmlFocusDelegateReply, HtmlFocusDelegateTargetKind, HtmlFocusTriggerKind,
};
use surgeist_html::security::nonce::HtmlNonceMissing;
use surgeist_html_semantics::{HtmlCaptureElementError, HtmlCaptureError, HtmlSemanticResource};

/// Every `DomError` variant except the two whose payload no code outside the HTML crates can
/// build (`ParsedReportUnresolved`, `SelectSemantic`).
const ROWS: usize = 161;

fn owner() -> DomOwnerId {
    DomOwnerId::allocate().unwrap()
}
fn node() -> NodeId {
    NodeId {
        owner: owner(),
        slot: 0,
        generation: 0,
    }
}
fn doc() -> DocumentId {
    DocumentId(node())
}
fn pending() -> DomParsedHtmlAssociationId {
    DomParsedHtmlAssociationId {
        stamp: DomStamp {
            owner: owner(),
            revision: DomRevision(0),
        },
        root: node(),
    }
}
fn exit_id() -> DomFullscreenExitId {
    DomFullscreenExitId {
        owner: owner(),
        slot: 0,
        generation: 0,
    }
}
fn operation() -> DomSourceOperationId {
    DomSourceOperationId {
        owner: owner(),
        serial: 1,
    }
}
fn invariant() -> DomSourceInvariantFailure {
    DomSourceInvariantFailure {
        dialog_operation: HtmlDialogOperation::Show,
        subject: node(),
        action: HtmlDialogAction::ReadOpen,
        basis: DomStamp {
            owner: owner(),
            revision: DomRevision(0),
        },
        reason: DomSourceInvariantReason::MissingDialogCloseWatcher,
    }
}
// Protocol errors come from replying to a fresh cursor that has no pending action.
fn close_watcher_error() -> surgeist_html::elements::dialog::HtmlCloseWatcherRequestError {
    HtmlCloseWatcherRequestCursor::new(false)
        .reply(HtmlCloseWatcherRequestReply::Applied)
        .unwrap_err()
}
fn autofocus_insertion_error() -> surgeist_html::focus::HtmlAutofocusInsertionError {
    HtmlAutofocusInsertionCursor::new()
        .reply(HtmlAutofocusInsertionReply::Applied)
        .unwrap_err()
}
fn design_mode_error() -> surgeist_html::elements::native::HtmlDesignModeError {
    HtmlDesignModeCursor::new(HtmlDesignModeInput::from_utf16(&[]))
        .reply(HtmlDesignModeReply::Applied)
        .unwrap_err()
}
fn dialog_focusing_error() -> surgeist_html::elements::dialog::HtmlDialogFocusingError {
    HtmlDialogFocusingCursor::new()
        .reply(HtmlDialogFocusingReply::Applied)
        .unwrap_err()
}
fn focus_delegate_error() -> surgeist_html::focus::HtmlFocusDelegateError {
    HtmlFocusDelegateCursor::new(HtmlFocusDelegateTargetKind::Dialog)
        .reply(HtmlFocusDelegateReply::Applied)
        .unwrap_err()
}
fn autofocus_delegate_error() -> surgeist_html::focus::HtmlAutofocusDelegateError {
    HtmlAutofocusDelegateCursor::new(HtmlFocusTriggerKind::Other)
        .reply(HtmlAutofocusDelegateReply::Autofocus(true))
        .unwrap_err()
}
fn dialog_error() -> surgeist_html::elements::dialog::HtmlDialogCursorError {
    HtmlDialogCursor::new(HtmlDialogOperation::Show)
        .reply(HtmlDialogReply::Applied)
        .unwrap_err()
}

fn rows() -> Vec<(&'static str, DomError, &'static str)> {
    vec![
        (
            "ParsedAssociationPending",
            DomError::ParsedAssociationPending {
                document: doc(),
                pending: pending(),
            },
            "parsed association history must be admitted before this operation can cross the Document",
        ),
        (
            "ParsedAssociationSynchronousPending",
            DomError::ParsedAssociationSynchronousPending { pending: pending() },
            "no authenticated executed synchronous frontier is available for the parsed notification",
        ),
        (
            "ParsedAssociationProcessingPoisoned",
            DomError::ParsedAssociationProcessingPoisoned { pending: pending() },
            "the parsed association processing guard was poisoned by an earlier failure",
        ),
        (
            "InlineStyleProfileMissing",
            DomError::InlineStyleProfileMissing { element: node() },
            "the inline style operation has no profile for the Element",
        ),
        (
            "InlineStyleProfileConflict",
            DomError::InlineStyleProfileConflict { element: node() },
            "several inline style profiles name the same Element and Document qualification",
        ),
        (
            "InlineStyleContextRequired",
            DomError::InlineStyleContextRequired {
                element: node(),
                change: DomInlineStyleChange::Attribute,
            },
            "a bound inline style change needs an inline style synchronization operation",
        ),
        (
            "InlineStyleOperationPending",
            DomError::InlineStyleOperationPending { element: node() },
            "an inline style feedback effect must complete before the affected operation",
        ),
        (
            "StaleFullscreenExit",
            DomError::StaleFullscreenExit,
            "the fullscreen exit invocation is stale",
        ),
        (
            "FullscreenExitHandoffRequired",
            DomError::FullscreenExitHandoffRequired,
            "a pending fullscreen exit must be handed off to its scheduling owner first",
        ),
        (
            "FullscreenExitAssertionFailed",
            DomError::FullscreenExitAssertionFailed { document: doc() },
            "an assertion of the fullscreen exit algorithm failed",
        ),
        (
            "FullscreenExitCompositionUnsupported",
            DomError::FullscreenExitCompositionUnsupported {
                document: doc(),
                fact: DomFullscreenExitFact::Container,
            },
            "the fullscreen exit cannot be composed with the Document's current state",
        ),
        (
            "FullscreenExitDocumentRetired",
            DomError::FullscreenExitDocumentRetired {
                invocation: exit_id(),
                document: doc(),
                step: DomFullscreenExitStep::FullyUnlockScreenOrientation,
            },
            "the Document of the fullscreen exit invocation was retired",
        ),
        (
            "FullscreenModelUnavailable",
            DomError::FullscreenModelUnavailable {
                step: DomFullscreenModelStep::ReadTopLayer,
            },
            "the fullscreen model cannot proceed at this step",
        ),
        (
            "TopLayerAdoptionUnavailable",
            DomError::TopLayerAdoptionUnavailable {
                subject: Some(node()),
                old_document: None,
            },
            "top-layer state cannot be adopted into the receiving Document",
        ),
        ("StaleRange", DomError::StaleRange, "the Range is stale"),
        (
            "StaleNodeIterator",
            DomError::StaleNodeIterator,
            "the NodeIterator is stale",
        ),
        (
            "StaleChildListObserver",
            DomError::StaleChildListObserver,
            "the child-list observer is stale",
        ),
        (
            "ChildListPublicationUnavailable",
            DomError::ChildListPublicationUnavailable,
            "the child-list observation publication is unavailable",
        ),
        (
            "ChildListDeliveryActive",
            DomError::ChildListDeliveryActive,
            "child-list observation delivery is already active",
        ),
        (
            "ChildListWorkLimit",
            DomError::ChildListWorkLimit { limit: 7 },
            "child-list observation exhausted its work allowance of 7",
        ),
        (
            "NodeIteratorFilterActive",
            DomError::NodeIteratorFilterActive,
            "the NodeIterator filter is already running",
        ),
        (
            "NodeIteratorPublicationUnavailable",
            DomError::NodeIteratorPublicationUnavailable,
            "the NodeIterator publication is unavailable",
        ),
        (
            "NodeIteratorWorkLimit",
            DomError::NodeIteratorWorkLimit { limit: 7 },
            "NodeIterator traversal exhausted its work allowance of 7",
        ),
        (
            "RangeAssociationIneligible",
            DomError::RangeAssociationIneligible,
            "the Range is not rooted in the Document it would be associated with",
        ),
        (
            "RangePublicationUnavailable",
            DomError::RangePublicationUnavailable,
            "a Range has boundaries that are invalid at publication",
        ),
        (
            "HostedSourceUnavailable",
            DomError::HostedSourceUnavailable,
            "the hosted source, association or registry the operation needs is unavailable or no longer current",
        ),
        (
            "DocumentConstructor",
            DomError::DocumentConstructor(DomDocumentConstructorError::Unavailable),
            "the Document constructor refused the request",
        ),
        (
            "HostedCompositionRequired",
            DomError::HostedCompositionRequired,
            "the operation must be composed with its hosted owner",
        ),
        (
            "InvalidDispatchCustody",
            DomError::InvalidDispatchCustody,
            "the dispatch custody, target or occurrence is not valid here",
        ),
        (
            "EventRegistryAlreadyBound",
            DomError::EventRegistryAlreadyBound,
            "a live listener-registry owner is already associated with this DOM",
        ),
        (
            "InvalidEventRegistry",
            DomError::InvalidEventRegistry,
            "the supplied association is not this DOM's current listener-registry owner",
        ),
        (
            "DispatchCustodyWorkLimit",
            DomError::DispatchCustodyWorkLimit { limit: 7 },
            "dispatch custody processing exhausted its work allowance of 7",
        ),
        (
            "SourceLimitReduction",
            DomError::SourceLimitReduction,
            "a Source policy call tried to reduce an allowance, which may only be raised",
        ),
        (
            "ParsedAssociationOperationPending",
            DomError::ParsedAssociationOperationPending { pending: pending() },
            "this parsed contribution's consuming operation is outstanding or has sealed its prefix",
        ),
        (
            "InvalidSourceEvent",
            DomError::InvalidSourceEvent,
            "the Source event, its executor or its custody is not valid for the retained operation at this stage",
        ),
        (
            "SourceInvariantFailed",
            DomError::SourceInvariantFailed {
                operation: operation(),
                failure: invariant(),
            },
            "a Source operation assertion failed",
        ),
        (
            "SourceReceiverRetired",
            DomError::SourceReceiverRetired {
                operation: operation(),
                subject: node(),
                action: HtmlDialogAction::ReadOpen,
            },
            "the receiver of the Source dialog action was retired before the action completed",
        ),
        (
            "SourceCloseWatcherReceiverRetired",
            DomError::SourceCloseWatcherReceiverRetired {
                operation: operation(),
                subject: node(),
                action: HtmlCloseWatcherRequestAction::ReadActive,
            },
            "the receiver of the Source close-watcher request was retired before the action completed",
        ),
        (
            "CloseWatcherProtocol",
            DomError::CloseWatcherProtocol(close_watcher_error()),
            "the close-watcher request protocol was violated",
        ),
        (
            "SourceEventExecutorUnavailable",
            DomError::SourceEventExecutorUnavailable,
            "no event executor is installed for this Source operation",
        ),
        (
            "SourceAttributeGlobalUnavailable",
            DomError::SourceAttributeGlobalUnavailable,
            "attribute verification has no authentic node-document Global",
        ),
        (
            "SourceAttributeExecutorUnavailable",
            DomError::SourceAttributeExecutorUnavailable,
            "attribute verification has no qualified executor",
        ),
        (
            "SourceAttributeVerificationRejected",
            DomError::SourceAttributeVerificationRejected,
            "the registered verifier rejected the proposed attribute value",
        ),
        (
            "SourceAttributeVerificationAbrupt",
            DomError::SourceAttributeVerificationAbrupt,
            "the registered verifier retained an abrupt guest outcome",
        ),
        (
            "DialogInvalidState",
            DomError::DialogInvalidState,
            "the Dialog is in a state that does not allow this operation",
        ),
        (
            "NeedsEventfulOperation",
            DomError::NeedsEventfulOperation {
                subject: None,
                action: HtmlDialogAction::ReadOpen,
            },
            "the Dialog action fires events and needs an eventful Source operation",
        ),
        (
            "SourceNestedOperationLimit",
            DomError::SourceNestedOperationLimit { limit: 7 },
            "nested Source operations exceeded their allowance of 7",
        ),
        (
            "SourceUrlInputsMismatch",
            DomError::SourceUrlInputsMismatch,
            "the Source URL request or observation does not match the pending stage's inputs",
        ),
        (
            "SourceUrlUnavailable",
            DomError::SourceUrlUnavailable,
            "the facts or observation the Source URL stage needs are unavailable",
        ),
        (
            "ForeignOwner",
            DomError::ForeignOwner,
            "the identity, basis or capability belongs to a different DOM owner",
        ),
        (
            "StaleNode",
            DomError::StaleNode,
            "the referenced node, registry or retained fact is stale or unavailable",
        ),
        (
            "NodeRetirementAlreadyWatched",
            DomError::NodeRetirementAlreadyWatched,
            "the node is already watched for retirement",
        ),
        (
            "NodeRetirementWorkLimit",
            DomError::NodeRetirementWorkLimit { limit: 7 },
            "node-retirement watch preparation exhausted its work allowance of 7",
        ),
        (
            "ForeignBatch",
            DomError::ForeignBatch,
            "the ticket, operand or context belongs to a different batch",
        ),
        (
            "UnavailableCreation",
            DomError::UnavailableCreation,
            "a creation requested by this edit has no published result",
        ),
        (
            "WrongNodeKind",
            DomError::WrongNodeKind,
            "the node is not of the kind the operation requires",
        ),
        (
            "WrongDocument",
            DomError::WrongDocument,
            "the node is not the expected Document or belongs to a different Document",
        ),
        (
            "HierarchyRequest",
            DomError::HierarchyRequest,
            "the node hierarchy does not allow this edit or position",
        ),
        (
            "InvalidAdjacentPosition",
            DomError::InvalidAdjacentPosition,
            "the adjacent insertion position is not beforebegin, afterbegin, beforeend or afterend",
        ),
        (
            "InvalidCharacter",
            DomError::InvalidCharacter,
            "the name or value contains a character or sequence that is not allowed",
        ),
        (
            "Namespace",
            DomError::Namespace,
            "the qualified name and namespace are inconsistent",
        ),
        (
            "ScriptPreparationRequired",
            DomError::ScriptPreparationRequired,
            "HTML Script preparation requires a live settings observation",
        ),
        (
            "MetaCspProcessingRequired",
            DomError::MetaCspProcessingRequired,
            "a staged HTML Meta insertion requires its Content Security Policy processor",
        ),
        (
            "NotSupported",
            DomError::NotSupported,
            "the operation is not supported here",
        ),
        (
            "NativeInitializationRequired",
            DomError::NativeInitializationRequired(HtmlElementClass::GenericElement),
            "the Element needs native initialization before it can be created",
        ),
        (
            "NonceInapplicable",
            DomError::NonceInapplicable,
            "the Element has no nonce",
        ),
        (
            "TemplateContentsUnavailable",
            DomError::TemplateContentsUnavailable,
            "the Template element's contents are unavailable",
        ),
        (
            "MarqueeInapplicable",
            DomError::MarqueeInapplicable,
            "the Element is not a Marquee",
        ),
        (
            "DialogInapplicable",
            DomError::DialogInapplicable,
            "the Element is not a Dialog or has no Dialog state",
        ),
        (
            "DialogDocumentStateUnavailable",
            DomError::DialogDocumentStateUnavailable,
            "the Document's dialog and top-layer state is unavailable",
        ),
        (
            "AutofocusDocumentStateUnavailable",
            DomError::AutofocusDocumentStateUnavailable,
            "the Document has no autofocus state",
        ),
        (
            "AutofocusInsertionUnavailable",
            DomError::AutofocusInsertionUnavailable {
                action: HtmlAutofocusInsertionAction::ReadAutofocus,
            },
            "the autofocus insertion step needs a service that is unavailable",
        ),
        (
            "AutofocusInsertionProtocol",
            DomError::AutofocusInsertionProtocol(autofocus_insertion_error()),
            "the autofocus insertion protocol was violated",
        ),
        (
            "DesignModeProcessingUnavailable",
            DomError::DesignModeProcessingUnavailable {
                step: DomSourceDesignModeStep::ReadActiveRange,
            },
            "designMode processing cannot proceed at this step",
        ),
        (
            "DesignModeProtocol",
            DomError::DesignModeProtocol(design_mode_error()),
            "the designMode protocol was violated",
        ),
        (
            "DialogFocusingUnavailable",
            DomError::DialogFocusingUnavailable {
                action: HtmlDialogFocusingAction::ReadAllowFocus,
            },
            "the Dialog focusing step needs a service that is unavailable",
        ),
        (
            "DialogFocusingProtocol",
            DomError::DialogFocusingProtocol(dialog_focusing_error()),
            "the Dialog focusing protocol was violated",
        ),
        (
            "FocusDelegateProtocol",
            DomError::FocusDelegateProtocol(focus_delegate_error()),
            "the focus-delegate protocol was violated",
        ),
        (
            "FocusedAreaDesignationUnavailable",
            DomError::FocusedAreaDesignationUnavailable {
                subject: None,
                document: None,
            },
            "the removing step cannot compare the Element with its Document's unavailable focused-area designation",
        ),
        (
            "AutofocusDelegateProtocol",
            DomError::AutofocusDelegateProtocol(autofocus_delegate_error()),
            "the autofocus-delegate protocol was violated",
        ),
        (
            "TopLayerAlreadyActive",
            DomError::TopLayerAlreadyActive {
                subject: node(),
                document: doc(),
            },
            "the Element is already in the top layer of its Document",
        ),
        (
            "DialogProcessingUnavailable",
            DomError::DialogProcessingUnavailable {
                action: HtmlDialogAction::ReadOpen,
            },
            "the Dialog action cannot be processed at this point",
        ),
        (
            "DialogProtocol",
            DomError::DialogProtocol(dialog_error()),
            "the Dialog protocol was violated",
        ),
        (
            "CustomValidityInapplicable",
            DomError::CustomValidityInapplicable,
            "the Element has no custom validity",
        ),
        (
            "ButtonInapplicable",
            DomError::ButtonInapplicable,
            "the Element is not a Button with this reflected target",
        ),
        (
            "InputInapplicable",
            DomError::InputInapplicable,
            "the Element is not an Input",
        ),
        (
            "TextareaInapplicable",
            DomError::TextareaInapplicable,
            "the Element is not a Textarea",
        ),
        (
            "OutputInapplicable",
            DomError::OutputInapplicable,
            "the Element is not an Output",
        ),
        (
            "OutputProtocol",
            DomError::OutputProtocol(HtmlOutputCursorError),
            "the Output protocol was violated",
        ),
        (
            "TextareaReadWorkLimit",
            DomError::TextareaReadWorkLimit { limit: 7 },
            "reading the Textarea exhausted its work allowance of 7",
        ),
        (
            "InputFilenameInvalidState",
            DomError::InputFilenameInvalidState,
            "the Input cannot take this filename in its current state",
        ),
        (
            "InputProcessingProtocol",
            DomError::InputProcessingProtocol,
            "the Input processing protocol was violated",
        ),
        (
            "InputProcessingWorkLimit",
            DomError::InputProcessingWorkLimit { limit: 7 },
            "Input processing exhausted its work allowance of 7",
        ),
        (
            "InputNumeric",
            DomError::InputNumeric(HtmlRangeNumericError::Integrity),
            "Input numeric processing failed",
        ),
        (
            "InputColorUnavailable",
            DomError::InputColorUnavailable(CssColorUnavailable::CurrentColor),
            "the Input color is unavailable",
        ),
        (
            "InputColorUnsupported",
            DomError::InputColorUnsupported(CssColorUnsupported::ProfileConversion),
            "the Input color is not supported",
        ),
        (
            "InputColorFacts",
            DomError::InputColorFacts(CssColorFactDefect::Foreign),
            "the Input color facts are defective",
        ),
        (
            "StyleCompositionRequired",
            DomError::StyleCompositionRequired(HtmlStyleUpdateAction::RemoveAssociatedSheet),
            "this Style update must be composed with its Style owner",
        ),
        (
            "StyleOwnerAttributesCompositionRequired",
            DomError::StyleOwnerAttributesCompositionRequired,
            "a Style owner attribute change must be composed with its Style owner",
        ),
        (
            "StyleDisabledCompositionRequired",
            DomError::StyleDisabledCompositionRequired,
            "a Style disabled change must be composed with its Style owner",
        ),
        (
            "StyleProcessingUnavailable",
            DomError::StyleProcessingUnavailable,
            "Style processing is unavailable for this operation",
        ),
        (
            "Meta",
            DomError::Meta(DomMetaError::HistoryUnavailable),
            "native Meta processing refused the operation",
        ),
        (
            "StyleProtocol",
            DomError::StyleProtocol,
            "the Style processing protocol was violated",
        ),
        (
            "SvgScriptProcessingUnavailable",
            DomError::SvgScriptProcessingUnavailable,
            "SVG Script processing is unavailable for this operation",
        ),
        (
            "SourceOperationPending",
            DomError::SourceOperationPending,
            "a Source operation, callback or hold is still pending",
        ),
        (
            "IncompleteSourceOperation",
            DomError::IncompleteSourceOperation,
            "an incomplete Source operation must finish first",
        ),
        (
            "InvalidSourceOperation",
            DomError::InvalidSourceOperation,
            "the request does not match the retained operation or task-source state",
        ),
        (
            "SourcePublicationLimit",
            DomError::SourcePublicationLimit,
            "the Source operation exhausted its publication allowance",
        ),
        (
            "SourceOutputLimit",
            DomError::SourceOutputLimit,
            "the Source operation exhausted its output allowance",
        ),
        (
            "SourceIdentityLimit",
            DomError::SourceIdentityLimit,
            "the Source operation exhausted its identity allowance",
        ),
        (
            "StyleReceiptMismatch",
            DomError::StyleReceiptMismatch,
            "the Style receipt does not match the retained state",
        ),
        (
            "StyleFactsMismatch",
            DomError::StyleFactsMismatch,
            "the supplied Style facts do not match the original request",
        ),
        (
            "StyleCssomRootBusy",
            DomError::StyleCssomRootBusy,
            "the Style CSSOM root is busy",
        ),
        (
            "StyleCssomRootPoisoned",
            DomError::StyleCssomRootPoisoned,
            "the Style CSSOM root was poisoned by an earlier failure",
        ),
        (
            "SourceWorkLimit",
            DomError::SourceWorkLimit { limit: 7 },
            "the Source operation exhausted its work allowance of 7",
        ),
        (
            "SelectInapplicable",
            DomError::SelectInapplicable,
            "the Element is not a Select",
        ),
        (
            "OptionInapplicable",
            DomError::OptionInapplicable,
            "the Element is not an Option",
        ),
        (
            "SelectProtocol",
            DomError::SelectProtocol,
            "the Select processing or family discovery protocol was violated",
        ),
        (
            "SelectValueQueriesUnconfigured",
            DomError::SelectValueQueriesUnconfigured,
            "Select value queries were not configured",
        ),
        (
            "SelectCapture",
            DomError::SelectCapture(HtmlCaptureError::NotHtmlLink),
            "capturing the Select semantics failed",
        ),
        (
            "SelectElementValues",
            DomError::SelectElementValues(HtmlCaptureElementError::Capture(
                HtmlCaptureError::NotHtmlLink,
            )),
            "reading the Select element values failed",
        ),
        (
            "InvalidSelectSemanticLimit",
            DomError::InvalidSelectSemanticLimit {
                resource: HtmlSemanticResource::Work,
                limit: 3,
                used: 9,
            },
            "the Select semantic allowance cannot be lowered to 3 because 9 is already in use",
        ),
        (
            "LinkProcessingUnavailable",
            DomError::LinkProcessingUnavailable,
            "Link processing is unavailable for this operation",
        ),
        (
            "ImageProcessingUnavailable",
            DomError::ImageProcessingUnavailable,
            "image processing is unavailable for this operation",
        ),
        (
            "MediaResourceEffectUnavailable",
            DomError::MediaResourceEffectUnavailable(HtmlMediaResourceEffect::Load),
            "the media resource effect needs a service that is unavailable",
        ),
        (
            "MediaRemovalStableStateUnavailable",
            DomError::MediaRemovalStableStateUnavailable,
            "the stable-state step of media removal is unavailable",
        ),
        (
            "ImageUpdateExhausted",
            DomError::ImageUpdateExhausted,
            "no image update identities remain",
        ),
        (
            "HyperlinkProcessingUnavailable",
            DomError::HyperlinkProcessingUnavailable,
            "hyperlink processing is unavailable for this operation",
        ),
        (
            "EncodingUrlFacts",
            DomError::EncodingUrlFacts(DomEncodingUrlFactsError::MissingFacts),
            "the encoding and URL facts were refused",
        ),
        (
            "EncodingUrlWorkLimit",
            DomError::EncodingUrlWorkLimit { limit: 7 },
            "encoding and URL processing exhausted its work allowance of 7",
        ),
        (
            "BaseFacts",
            DomError::BaseFacts(DomBaseFactsError::PolicyMismatch),
            "the base URL facts were refused",
        ),
        (
            "BaseWorkLimit",
            DomError::BaseWorkLimit { limit: 7 },
            "base URL processing exhausted its work allowance of 7",
        ),
        (
            "SelectWorkLimit",
            DomError::SelectWorkLimit { limit: 7 },
            "Select processing exhausted its work allowance of 7",
        ),
        (
            "InvalidNativeTask",
            DomError::InvalidNativeTask,
            "the native task is not valid for this operation in its current state",
        ),
        (
            "SlotInapplicable",
            DomError::SlotInapplicable,
            "the node is not a Slot",
        ),
        (
            "NonceEnvironmentUnavailable",
            DomError::NonceEnvironmentUnavailable(HtmlNonceMissing::Connection),
            "the nonce environment is unavailable",
        ),
        (
            "NotFound",
            DomError::NotFound,
            "the requested node was not found",
        ),
        (
            "InUseAttribute",
            DomError::InUseAttribute,
            "the Attr is already in use by another Element",
        ),
        (
            "InvalidDestruction",
            DomError::InvalidDestruction,
            "the node is attached or is a Document or ShadowRoot, so it cannot be destroyed directly",
        ),
        (
            "CustomElementReactionPending",
            DomError::CustomElementReactionPending(node()),
            "a routed custom element reaction must settle before the receiver can be retired",
        ),
        (
            "IndexSize",
            DomError::IndexSize,
            "the index is out of range",
        ),
        (
            "Cancelled",
            DomError::Cancelled,
            "the operation was cancelled",
        ),
        (
            "ResourceLimit",
            DomError::ResourceLimit {
                resource: DomResource::Ranges,
                limit: 4,
            },
            "ranges would exceed its configured limit of 4",
        ),
        (
            "WorkLimit",
            DomError::WorkLimit {
                resource: DomResource::StringCodeUnits,
                limit: 5,
            },
            "string code units work exhausted its allowance of 5",
        ),
        (
            "InvalidLimit",
            DomError::InvalidLimit(DomResource::ManagedNodes),
            "the configured limit for managed nodes is invalid",
        ),
        (
            "AllocationFailed",
            DomError::AllocationFailed,
            "memory allocation failed",
        ),
        (
            "RepresentationExhausted",
            DomError::RepresentationExhausted,
            "the canonical storage cannot represent this state",
        ),
        (
            "HtmlChargeProtocol",
            DomError::HtmlChargeProtocol,
            "an HTML operation broke its accounting contract with the DOM",
        ),
        (
            "PseudoAttributeCursor",
            DomError::PseudoAttributeCursor(HtmlPiPseudoAttributeError::Integrity),
            "the processing-instruction pseudo-attribute driver and its cursor disagree",
        ),
        (
            "OwnerIdentityExhausted",
            DomError::OwnerIdentityExhausted,
            "no DOM owner identities remain",
        ),
        (
            "RevisionExhausted",
            DomError::RevisionExhausted,
            "the revision counter cannot advance far enough for this operation",
        ),
        (
            "ViewTransitionsUninitialized",
            DomError::ViewTransitionsUninitialized,
            "view transitions were not initialized for the Document",
        ),
        (
            "ViewTransitionRevisionMismatch",
            DomError::ViewTransitionRevisionMismatch,
            "the view transition state was captured at a different revision",
        ),
        (
            "StaleViewTransition",
            DomError::StaleViewTransition,
            "the view transition is stale",
        ),
        (
            "StaleViewTransitionRecord",
            DomError::StaleViewTransitionRecord,
            "the view transition record is stale",
        ),
        (
            "StaleViewTransitionPseudo",
            DomError::StaleViewTransitionPseudo,
            "the view transition pseudo-element is stale",
        ),
        (
            "InvalidViewTransitionState",
            DomError::InvalidViewTransitionState,
            "the view transition is in a state that does not allow this operation",
        ),
        (
            "DuplicateViewTransitionName",
            DomError::DuplicateViewTransitionName,
            "another view transition already uses this name",
        ),
        (
            "ViewTransitionOriginUnavailable",
            DomError::ViewTransitionOriginUnavailable,
            "the origin needed by the view transition is unavailable",
        ),
        (
            "ViewTransitionCssom",
            DomError::ViewTransitionCssom(DomViewTransitionCssomError::StaleRevision),
            "the view transition CSSOM association was refused",
        ),
        (
            "ViewTransitionCaptureWorkLimit",
            DomError::ViewTransitionCaptureWorkLimit { limit: 7 },
            "view transition capture exhausted its work allowance of 7",
        ),
    ]
}

/// A new `DomError` variant fails to compile here until it is given a row above.
fn has_row(error: &DomError) {
    match error {
        DomError::ParsedReportUnresolved(..)
        | DomError::ParsedAssociationPending { .. }
        | DomError::ParsedAssociationSynchronousPending { .. }
        | DomError::ParsedAssociationProcessingPoisoned { .. }
        | DomError::InlineStyleProfileMissing { .. }
        | DomError::InlineStyleProfileConflict { .. }
        | DomError::InlineStyleContextRequired { .. }
        | DomError::InlineStyleOperationPending { .. }
        | DomError::StaleFullscreenExit
        | DomError::FullscreenExitHandoffRequired
        | DomError::FullscreenExitAssertionFailed { .. }
        | DomError::FullscreenExitCompositionUnsupported { .. }
        | DomError::FullscreenExitDocumentRetired { .. }
        | DomError::FullscreenModelUnavailable { .. }
        | DomError::TopLayerAdoptionUnavailable { .. }
        | DomError::StaleRange
        | DomError::StaleNodeIterator
        | DomError::StaleChildListObserver
        | DomError::ChildListPublicationUnavailable
        | DomError::ChildListDeliveryActive
        | DomError::ChildListWorkLimit { .. }
        | DomError::NodeIteratorFilterActive
        | DomError::NodeIteratorPublicationUnavailable
        | DomError::NodeIteratorWorkLimit { .. }
        | DomError::RangeAssociationIneligible
        | DomError::RangePublicationUnavailable
        | DomError::HostedSourceUnavailable
        | DomError::DocumentConstructor(..)
        | DomError::HostedCompositionRequired
        | DomError::InvalidDispatchCustody
        | DomError::EventRegistryAlreadyBound
        | DomError::InvalidEventRegistry
        | DomError::DispatchCustodyWorkLimit { .. }
        | DomError::SourceLimitReduction
        | DomError::ParsedAssociationOperationPending { .. }
        | DomError::InvalidSourceEvent
        | DomError::SourceInvariantFailed { .. }
        | DomError::SourceReceiverRetired { .. }
        | DomError::SourceCloseWatcherReceiverRetired { .. }
        | DomError::CloseWatcherProtocol(..)
        | DomError::SourceEventExecutorUnavailable
        | DomError::SourceAttributeGlobalUnavailable
        | DomError::SourceAttributeExecutorUnavailable
        | DomError::SourceAttributeVerificationRejected
        | DomError::SourceAttributeVerificationAbrupt
        | DomError::DialogInvalidState
        | DomError::NeedsEventfulOperation { .. }
        | DomError::SourceNestedOperationLimit { .. }
        | DomError::SourceUrlInputsMismatch
        | DomError::SourceUrlUnavailable
        | DomError::ForeignOwner
        | DomError::StaleNode
        | DomError::NodeRetirementAlreadyWatched
        | DomError::NodeRetirementWorkLimit { .. }
        | DomError::ForeignBatch
        | DomError::UnavailableCreation
        | DomError::WrongNodeKind
        | DomError::WrongDocument
        | DomError::HierarchyRequest
        | DomError::InvalidAdjacentPosition
        | DomError::InvalidCharacter
        | DomError::Namespace
        | DomError::ScriptPreparationRequired
        | DomError::MetaCspProcessingRequired
        | DomError::NotSupported
        | DomError::NativeInitializationRequired(..)
        | DomError::NonceInapplicable
        | DomError::TemplateContentsUnavailable
        | DomError::MarqueeInapplicable
        | DomError::DialogInapplicable
        | DomError::DialogDocumentStateUnavailable
        | DomError::AutofocusDocumentStateUnavailable
        | DomError::AutofocusInsertionUnavailable { .. }
        | DomError::AutofocusInsertionProtocol(..)
        | DomError::DesignModeProcessingUnavailable { .. }
        | DomError::DesignModeProtocol(..)
        | DomError::DialogFocusingUnavailable { .. }
        | DomError::DialogFocusingProtocol(..)
        | DomError::FocusDelegateProtocol(..)
        | DomError::FocusedAreaDesignationUnavailable { .. }
        | DomError::AutofocusDelegateProtocol(..)
        | DomError::TopLayerAlreadyActive { .. }
        | DomError::DialogProcessingUnavailable { .. }
        | DomError::DialogProtocol(..)
        | DomError::CustomValidityInapplicable
        | DomError::ButtonInapplicable
        | DomError::InputInapplicable
        | DomError::TextareaInapplicable
        | DomError::OutputInapplicable
        | DomError::OutputProtocol(..)
        | DomError::TextareaReadWorkLimit { .. }
        | DomError::InputFilenameInvalidState
        | DomError::InputProcessingProtocol
        | DomError::InputProcessingWorkLimit { .. }
        | DomError::InputNumeric(..)
        | DomError::InputColorUnavailable(..)
        | DomError::InputColorUnsupported(..)
        | DomError::InputColorFacts(..)
        | DomError::StyleCompositionRequired(..)
        | DomError::StyleOwnerAttributesCompositionRequired
        | DomError::StyleDisabledCompositionRequired
        | DomError::StyleProcessingUnavailable
        | DomError::Meta(..)
        | DomError::StyleProtocol
        | DomError::SvgScriptProcessingUnavailable
        | DomError::SourceOperationPending
        | DomError::IncompleteSourceOperation
        | DomError::InvalidSourceOperation
        | DomError::SourcePublicationLimit
        | DomError::SourceOutputLimit
        | DomError::SourceIdentityLimit
        | DomError::StyleReceiptMismatch
        | DomError::StyleFactsMismatch
        | DomError::StyleCssomRootBusy
        | DomError::StyleCssomRootPoisoned
        | DomError::SourceWorkLimit { .. }
        | DomError::SelectInapplicable
        | DomError::OptionInapplicable
        | DomError::SelectProtocol
        | DomError::SelectValueQueriesUnconfigured
        | DomError::SelectCapture(..)
        | DomError::SelectElementValues(..)
        | DomError::SelectSemantic(..)
        | DomError::InvalidSelectSemanticLimit { .. }
        | DomError::LinkProcessingUnavailable
        | DomError::ImageProcessingUnavailable
        | DomError::MediaResourceEffectUnavailable(..)
        | DomError::MediaRemovalStableStateUnavailable
        | DomError::ImageUpdateExhausted
        | DomError::HyperlinkProcessingUnavailable
        | DomError::EncodingUrlFacts(..)
        | DomError::EncodingUrlWorkLimit { .. }
        | DomError::BaseFacts(..)
        | DomError::BaseWorkLimit { .. }
        | DomError::SelectWorkLimit { .. }
        | DomError::InvalidNativeTask
        | DomError::SlotInapplicable
        | DomError::NonceEnvironmentUnavailable(..)
        | DomError::NotFound
        | DomError::InUseAttribute
        | DomError::InvalidDestruction
        | DomError::CustomElementReactionPending(..)
        | DomError::IndexSize
        | DomError::Cancelled
        | DomError::ResourceLimit { .. }
        | DomError::WorkLimit { .. }
        | DomError::InvalidLimit { .. }
        | DomError::AllocationFailed
        | DomError::RepresentationExhausted
        | DomError::HtmlChargeProtocol
        | DomError::PseudoAttributeCursor(..)
        | DomError::OwnerIdentityExhausted
        | DomError::RevisionExhausted
        | DomError::ViewTransitionsUninitialized
        | DomError::ViewTransitionRevisionMismatch
        | DomError::StaleViewTransition
        | DomError::StaleViewTransitionRecord
        | DomError::StaleViewTransitionPseudo
        | DomError::InvalidViewTransitionState
        | DomError::DuplicateViewTransitionName
        | DomError::ViewTransitionOriginUnavailable
        | DomError::ViewTransitionCssom(..)
        | DomError::ViewTransitionCaptureWorkLimit { .. } => {}
    }
}

#[test]
fn every_variant_with_a_buildable_payload_has_one_row_and_its_exact_message() {
    let rows = rows();
    assert_eq!(rows.len(), ROWS);
    let mut seen = std::collections::BTreeSet::new();
    for (name, error, text) in &rows {
        has_row(error);
        assert!(seen.insert(*name), "duplicate row {name}");
        assert_eq!(error.to_string(), *text, "{name}");
        assert_ne!(
            error.to_string(),
            format!("{error:?}"),
            "{name} prints its Debug form"
        );
    }
}

#[test]
fn no_message_embeds_a_debug_rendering() {
    for (name, error, _) in rows() {
        let text = error.to_string();
        assert!(
            !text.contains('{') && !text.contains('}') && !text.contains("::"),
            "{name}: {text}"
        );
        assert!(
            !text.is_empty() && !text.ends_with('.') && text == text.trim(),
            "{name}: {text}"
        );
    }
}

/// The variants that wrap an error implementing `core::error::Error`; `source()` returns it.
const WRAPPING: [&str; 19] = [
    "ParsedReportUnresolved",
    "CloseWatcherProtocol",
    "AutofocusInsertionProtocol",
    "DesignModeProtocol",
    "DialogFocusingProtocol",
    "FocusDelegateProtocol",
    "AutofocusDelegateProtocol",
    "DialogProtocol",
    "OutputProtocol",
    "InputNumeric",
    "SelectCapture",
    "SelectElementValues",
    "SelectSemantic",
    "PseudoAttributeCursor",
    "BaseFacts",
    "DocumentConstructor",
    "EncodingUrlFacts",
    "Meta",
    "ViewTransitionCssom",
];

#[test]
fn source_is_the_wrapped_error_for_exactly_the_wrapping_variants() {
    for (name, error, _) in rows() {
        let source = error.source();
        assert_eq!(source.is_some(), WRAPPING.contains(&name), "{name}");
        if let Some(source) = source {
            assert!(!source.to_string().is_empty(), "{name}");
        }
    }
}

#[test]
fn history_errors_state_why_the_change_coverage_is_unavailable() {
    use crate::DomHistoryError;
    let rows = [
        (
            DomHistoryError::ForeignOwner,
            "the baseline stamp belongs to a different DOM owner",
        ),
        (
            DomHistoryError::FutureRevision,
            "the baseline stamp is newer than the current revision",
        ),
        (
            DomHistoryError::FullRecomputeRequired {
                available_from: DomRevision(3),
            },
            "the retained history no longer reaches back to the baseline; recompute from a snapshot",
        ),
        (
            DomHistoryError::AllocationFailed,
            "memory allocation failed while collecting the change records",
        ),
    ];
    for (error, text) in rows {
        assert_eq!(error.to_string(), text);
        assert_ne!(error.to_string(), format!("{error:?}"));
        assert!(error.source().is_none());
    }
}

#[test]
fn every_resource_is_named_by_a_distinct_phrase() {
    let all = [
        DomResource::ParsedAssociationBytes,
        DomResource::ParsedAssociationUnits,
        DomResource::ParsedAssociationProcessingRetainedBytes,
        DomResource::ParsedAssociationProcessingRetainedUnits,
        DomResource::ParsedAssociationProcessingOutputs,
        DomResource::ParsedAssociationProcessingWork,
        DomResource::ParsedAssociationDispatchWork,
        DomResource::FullscreenExits,
        DomResource::FullscreenEvents,
        DomResource::Ranges,
        DomResource::NodeIterators,
        DomResource::ChildListObservers,
        DomResource::ChildListRegistrations,
        DomResource::ChildListRecords,
        DomResource::ChildListRecordReferences,
        DomResource::ManagedNodes,
        DomResource::SlotReferences,
        DomResource::AutofocusCandidates,
        DomResource::SlotAssignmentInputs,
        DomResource::SlotQueryOutput,
        DomResource::SlotQueryScratch,
        DomResource::StringCodeUnits,
        DomResource::Commands,
        DomResource::HistoryEntries,
        DomResource::HistoryBytes,
        DomResource::ViewTransitions,
        DomResource::ViewTransitionRecords,
        DomResource::ViewTransitionPseudos,
        DomResource::ViewTransitionCaptureBytes,
        DomResource::InputProcessingScratch,
        DomResource::SelectScratch,
        DomResource::SelectTasks,
        DomResource::DetailsToggleIntentions,
        DomResource::DetailsToggleOutput,
        DomResource::BaseObservations,
        DomResource::ImageUpdateCancellations,
        DomResource::EncodingUrlObservations,
        DomResource::DispatchCustodyScopes,
        DomResource::DispatchCustodyReferences,
        DomResource::DispatchCustodyRetainedBytes,
        DomResource::DispatchCustodyScratch,
        DomResource::EventRecipients,
        DomResource::EventRecipientRetainedBytes,
        DomResource::EventRecipientScratch,
        DomResource::EventRecipientWork,
        DomResource::InlineStyleBindings,
        DomResource::InlineStyleStorage,
        DomResource::InlineStyleScratch,
        DomResource::InlineStyleWork,
        DomResource::NodeRetirementCanonicalBytes,
        DomResource::SourceRetainedBytes,
        DomResource::SourceRetainedUnits,
        DomResource::NameRetainedBytes,
        DomResource::NameEntries,
        DomResource::NameWork,
        DomResource::NameScratch,
        DomResource::MetaAtomicRetainedBytes,
        DomResource::MetaAtomicRetainedUnits,
        DomResource::MetaAtomicOutputs,
        DomResource::MetaColorSchemeRetainedBytes,
        DomResource::MetaThemeColorScratch,
    ];
    assert_eq!(all.len(), 61);
    let mut phrases = std::collections::BTreeSet::new();
    for resource in all {
        let text = DomError::InvalidLimit(resource).to_string();
        assert!(phrases.insert(text.clone()), "{resource:?} repeats {text}");
    }
}
