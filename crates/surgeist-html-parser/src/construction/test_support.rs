//! Test-only inspection and fault-injection helpers for the construction owner.

use super::*;
use surgeist_html::{
    construction::HtmlConstructionResult,
    construction::{
        BuildAttribute, ConstructionElementView, ConstructionNodeKind, HtmlFragmentOwner,
    },
    values::{Attribute, DocumentString, OrderedAttributes, QualifiedName},
};

/// Uncharged: the attributes of an element in order. An element whose run its storage does not
/// hold has none.
pub(super) fn attributes_of<'a>(
    element: impl std::borrow::Borrow<ConstructionElementView<'a>>,
) -> Vec<BuildAttribute<'a>> {
    element
        .borrow()
        .attributes()
        .iter()
        .map(|attributes| attributes.collect::<Vec<_>>())
        .unwrap_or_default()
}

/// Uncharged: the units of the value of an attribute, which the storage holds.
pub(super) fn value_units(attribute: BuildAttribute<'_>) -> &[u16] {
    attribute
        .value()
        .code_units()
        .expect("the storage holds the value")
}

/// The attributes the run of the element at `node` holds and the room it has, as the storage
/// reports them to a reservation of none. The call is charged as an edit is.
pub(super) fn run_dimensions(
    storage: &mut SessionStorage,
    control: &mut ParseControl,
    node: usize,
) -> surgeist_html::bounded::construction::HtmlConstructionAttributeCapacity {
    let handle = storage
        .construction
        .node_at(node)
        .expect("a retained record")
        .handle();
    let SessionStorage {
        construction,
        holds,
        control: identity,
        ..
    } = storage;
    let mut admission =
        super::record_admission::RecordAdmission::new(control, identity, holds, &[]);
    match construction.reserve_attributes(&handle, 0, &mut admission) {
        Ok(dimensions) => dimensions,
        Err(_) => panic!("the element has a run"),
    }
}

/// Uncharged: the value of an attribute as text, which the storage holds as well-formed UTF-16.
pub(super) fn value_text(attribute: BuildAttribute<'_>) -> String {
    String::from_utf16(value_units(attribute)).expect("well-formed UTF-16")
}

/// Uncharged: the name and value units of each attribute of an element in order.
pub(super) fn attribute_units<'a>(
    element: impl std::borrow::Borrow<ConstructionElementView<'a>>,
) -> Vec<(Vec<u16>, Vec<u16>)> {
    attributes_of(element)
        .into_iter()
        .map(|attribute| {
            (
                attribute.name().local_name().code_units().to_vec(),
                value_units(attribute).to_vec(),
            )
        })
        .collect()
}

/// Uncharged: the attributes of an element as an owned list, which a test compares or hands to an
/// authored element.
pub(super) fn owned_attributes<'a>(
    element: impl std::borrow::Borrow<ConstructionElementView<'a>>,
) -> OrderedAttributes {
    OrderedAttributes::new(attributes_of(element).into_iter().map(|attribute| {
        Attribute::new(
            attribute.name().clone(),
            DocumentString::from_utf16(value_units(attribute).to_vec()),
        )
    }))
    .expect("the attributes of an element are distinct")
}

/// Lets a test make one step of a cleanup walk refuse after it did its work, as
/// a reported refusal would. Call it before [`keep_first`] for that step.
pub(super) fn inject_walk_refusal(
    control: &mut ParseControl,
    site: WalkSite,
    refused: &mut Option<SessionError>,
) {
    if let Some(issued) = control.take_walk_refusal(site) {
        let injected = if issued == 0 {
            SessionError::WrongToken
        } else {
            latch_registry_row(control)
        };
        refused.get_or_insert(injected);
    }
}

/// The size in bytes of one record of the storage's record bank, as the storage reports it when it
/// asks for the growth of the bank.
pub(super) fn record_element_size() -> usize {
    use surgeist_html::accounting::{HtmlChange, HtmlResource, HtmlWork};
    use surgeist_html::bounded::construction::{
        HtmlConstructionBankGrowth, HtmlConstructionChange, HtmlConstructionGrowthRecord,
        HtmlConstructionStorage, HtmlRecordAdmission,
    };
    use surgeist_ledger::Charge;
    #[derive(Default)]
    struct Recorder(usize);
    impl Charge<HtmlWork, HtmlResource> for Recorder {
        type Stop = ();
        fn charge(&mut self, _: &[HtmlChange]) -> Result<(), ()> {
            Ok(())
        }
        fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), ()> {
            Ok(())
        }
    }
    impl HtmlRecordAdmission for Recorder {
        fn admit_change(
            &mut self,
            _: &HtmlConstructionStorage,
            _: HtmlConstructionChange,
        ) -> Result<(), ()> {
            Ok(())
        }
        fn admit_growth(&mut self, growth: HtmlConstructionBankGrowth) -> Result<usize, ()> {
            self.0 = growth.element_size();
            Ok(growth.minimum_capacity())
        }
        fn record_growth(&mut self, _: HtmlConstructionGrowthRecord) -> Result<(), ()> {
            Ok(())
        }
        fn record_change(
            &mut self,
            _: &HtmlConstructionStorage,
            _: HtmlConstructionChange,
        ) -> Result<(), ()> {
            Ok(())
        }
    }
    let mut recorder = Recorder::default();
    if let Ok(mut storage) = HtmlConstructionStorage::new(&mut recorder) {
        // A reservation asks for the growth of the record bank.
        let _ = storage.reserve_records(1, &mut recorder);
    }
    recorder.0
}

/// The capacity of the record bank and the size of one record, for a session that has no record
/// with children: the parse holds the bytes of that bank alone.
pub(super) fn record_bank(storage: &SessionStorage) -> (usize, usize) {
    let size = record_element_size();
    ((storage.holds.bank_bytes as usize) / size, size)
}

pub(super) fn name_units(
    control: &mut ParseControl,
    name: &QualifiedName,
) -> Result<usize, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 4)?;
    let units = name
        .local_name()
        .len()
        .checked_add(name.namespace().uri().map_or(0, DocumentString::len))
        .and_then(|sum| sum.checked_add(name.prefix().map_or(0, DocumentString::len)));
    match units {
        Some(units) => Ok(units),
        None => control
            .representation(HtmlParseResource::ManagedUtf16Units)
            .map_err(Into::into),
    }
}

pub(super) fn test_owning_depth(result: &HtmlConstructionResult, position: usize) -> usize {
    let mut depth = 0;
    let mut node = result.node_at(position).unwrap();
    loop {
        node = if let Some(parent) = node.parent() {
            parent
        } else if let Some(
            HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner),
        ) = node.fragment_owner()
        {
            result.node_by_result_id(owner).unwrap()
        } else {
            return depth;
        };
        depth += 1;
        assert!(depth <= result.len(), "owning ancestry is acyclic");
    }
}

/// A real CSP protocol error: an association presented to a different owner.
pub(crate) fn csp_protocol_error() -> surgeist_html::security::csp::HtmlCspProtocolError {
    use surgeist_html::security::csp::{HtmlCspError, HtmlCspOwner, HtmlCspProcessorAuthority};
    let admit = &mut |_| Ok::<(), std::convert::Infallible>(());
    let mut owner = HtmlCspOwner::new_with(admit).unwrap();
    let mut other = HtmlCspOwner::new_with(admit).unwrap();
    let association = owner.initial_document_with(admit).unwrap();
    let profile = HtmlCspProcessorAuthority::new_with(admit)
        .unwrap()
        .profile_with(1, admit)
        .unwrap();
    let content = surgeist_html::values::DocumentString::from("default-src 'none'");
    match other.request_meta_insertion_with(&association, &profile, content, admit) {
        Err(rejected) => match rejected.error {
            HtmlCspError::Protocol(error) => error,
            HtmlCspError::Interrupted(never) => match never {},
        },
        Ok(_) => panic!("a foreign association was accepted"),
    }
}

impl SessionStorage {
    /// Uncharged: `node` is an HTML element named `local`.
    pub(super) fn uncharged_html_local(&self, node: usize, local: &str) -> bool {
        let Some(record) = self.construction.node_at(node) else {
            return false;
        };
        let ConstructionNodeKind::Element(element) = record.kind() else {
            return false;
        };
        element
            .name()
            .namespace()
            .uri()
            .is_some_and(|uri| uri.to_utf8().as_deref() == Ok("http://www.w3.org/1999/xhtml"))
            && element.name().local_name().to_utf8().as_deref() == Ok(local)
    }
    pub(super) fn uncharged_attribute(&self, node: usize, local: &str) -> bool {
        let Some(ConstructionNodeKind::Element(element)) =
            self.construction.node_at(node).map(|record| record.kind())
        else {
            return false;
        };
        attributes_of(element).into_iter().any(|attribute| {
            attribute.name().namespace().uri().is_none()
                && attribute.name().local_name().to_utf8().as_deref() == Ok(local)
        })
    }
    pub(super) fn uncharged_parent(&self, node: usize) -> Option<usize> {
        self.construction
            .node_at(node)
            .and_then(|record| record.parent())
            .map(|parent| parent.position())
    }
    /// Uncharged: the style elements of the tree at `root` in tree order.
    pub(super) fn uncharged_style_order(&self, root: usize) -> Vec<usize> {
        let mut found = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            let record = self.construction.node_at(node).expect("a retained node");
            if self.uncharged_html_local(node, "style") {
                found.push(node);
            }
            stack.extend(
                (0..record.child_count())
                    .rev()
                    .filter_map(|position| record.child_at(position).map(|child| child.position())),
            );
        }
        found
    }
    /// Every listed control with a `form` attribute is in the explicit-form
    /// list, which holds only rows of the registry's form-association list,
    /// each once and in registry order.
    pub(super) fn assert_explicit_forms_exact(&self) {
        let listed = self.payloads.kind_list(KindList::FormAssociation);
        let explicit = self.payloads.explicit_form_list();
        let expected: Vec<usize> = listed
            .iter()
            .copied()
            .filter(|slot| {
                matches!(
                    self.payloads.values.get(*slot),
                    Some(RetainedPayload::FormAssociation { node: Some(node), state })
                        if state.kind.listed() && self.uncharged_attribute(*node, "form")
                )
            })
            .collect();
        let held: Vec<usize> = explicit
            .iter()
            .copied()
            .filter(|slot| {
                matches!(
                    self.payloads.values.get(*slot),
                    Some(RetainedPayload::FormAssociation { node: Some(_), .. })
                )
            })
            .collect();
        assert_eq!(
            held, expected,
            "explicit form rows differ from the registry"
        );
        assert!(
            explicit.windows(2).all(|pair| pair[0] < pair[1])
                && explicit.iter().all(|slot| listed.contains(slot)),
            "explicit form rows are a subsequence of the form association rows"
        );
    }
    /// Uncharged, from the registry rather than the joins: whether a Base or form
    /// association row's node lies in the subtree at `root`, or the subtree holds
    /// an id while some listed control has a form attribute.
    pub(super) fn uncharged_form_subtree_is_local(&self, root: usize) -> bool {
        let inside = |mut node: usize| loop {
            if node == root {
                return true;
            }
            match self
                .construction
                .node_at(node)
                .and_then(|record| record.parent())
            {
                Some(parent) => node = parent.position(),
                None => return false,
            }
        };
        let mut listed_explicit = false;
        for row in &self.payloads.values {
            match row {
                RetainedPayload::Base {
                    node: Some(node), ..
                } if inside(*node) => return true,
                RetainedPayload::FormAssociation {
                    node: Some(node),
                    state,
                } => {
                    if inside(*node) {
                        return true;
                    }
                    listed_explicit |=
                        state.kind.listed() && self.uncharged_attribute(*node, "form");
                }
                _ => {}
            }
        }
        if !listed_explicit {
            return false;
        }
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            let record = self.construction.node_at(node).expect("a retained node");
            if let ConstructionNodeKind::Element(element) = record.kind()
                && attributes_of(element)
                    .into_iter()
                    .find(|attribute| {
                        attribute.name().namespace().uri().is_none()
                            && attribute.name().local_name().to_utf8().as_deref() == Ok("id")
                    })
                    .is_some_and(|id| !value_units(id).is_empty())
            {
                return true;
            }
            stack.extend(
                (0..record.child_count())
                    .rev()
                    .filter_map(|position| record.child_at(position).map(|child| child.position())),
            );
        }
        false
    }
}
