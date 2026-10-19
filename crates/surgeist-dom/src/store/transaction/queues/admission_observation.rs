//! Selected allocation policies for private family tests. Preparatory index and
//! record custody remains live before the test installs its finite allowance.
use super::*;
use crate::dialog::DomDialogState;
use std::cell::RefCell;

#[derive(Clone, Copy)]
enum Site {
    Fields(Key),
    Relation(Key, transaction::RelationField),
    Payload,
}
struct State {
    site: Site,
    additional: custody::Usage,
    resource: DomResource,
    short: bool,
    limit: Option<usize>,
    parent_policy: Option<crate::DomSourceLimits>,
}
thread_local! {
    static ACTIVE: RefCell<Option<State>> = const { RefCell::new(None) };
}
pub(in crate::store) struct Binding;
impl Binding {
    fn new(site: Site, additional: custody::Usage, resource: DomResource, short: bool) -> Self {
        ACTIVE.with(|active| {
            assert!(active.borrow().is_none());
            *active.borrow_mut() = Some(State {
                site,
                additional,
                resource,
                short,
                limit: None,
                parent_policy: None,
            });
        });
        Self
    }
    pub fn fields(
        key: Key,
        additional: custody::Usage,
        resource: DomResource,
        short: bool,
    ) -> Self {
        Self::new(Site::Fields(key), additional, resource, short)
    }
    pub fn relation(
        key: Key,
        field: transaction::RelationField,
        bytes: usize,
        short: bool,
    ) -> Self {
        Self::new(
            Site::Relation(key, field),
            custody::Usage::new(bytes, 0),
            DomResource::SourceRetainedBytes,
            short,
        )
    }
    pub fn payload(additional: custody::Usage, resource: DomResource, short: bool) -> Self {
        Self::new(Site::Payload, additional, resource, short)
    }
    pub fn limit(&self) -> usize {
        ACTIVE.with(|active| {
            active
                .borrow()
                .as_ref()
                .unwrap()
                .limit
                .expect("selected allocation reached")
        })
    }
}
impl Drop for Binding {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active.borrow_mut().take();
        });
    }
}
fn before(draft: &mut Draft, site: Site) {
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let Some(state) = active.as_mut() else { return };
        let matches = match (state.site, site) {
            (Site::Fields(a), Site::Fields(b)) => a == b,
            (Site::Relation(a, af), Site::Relation(b, bf)) => {
                a == b && std::mem::discriminant(&af) == std::mem::discriminant(&bf)
            }
            (Site::Payload, Site::Payload) => true,
            _ => false,
        };
        if !matches || state.limit.is_some() {
            return;
        }
        let current = draft.custody_ledger().total();
        let mut policy = draft.source_limits.expect("actual Source allowance");
        state.parent_policy = Some(policy);
        let exact = match state.resource {
            DomResource::SourceRetainedBytes => {
                current.bytes() + draft.source_other_bytes + state.additional.bytes()
            }
            DomResource::SourceRetainedUnits => {
                current.units() + draft.source_other_units + state.additional.units()
            }
            _ => panic!("family allocation resource"),
        };
        let limit = exact - usize::from(state.short);
        match state.resource {
            DomResource::SourceRetainedBytes => policy.retained_bytes = limit,
            DomResource::SourceRetainedUnits => policy.retained_units = limit,
            _ => unreachable!(),
        }
        draft.source_limits = Some(policy);
        state.limit = Some(limit);
    });
}
pub(in crate::store) fn before_fields(draft: &mut Draft, key: Key) {
    before(draft, Site::Fields(key));
}
pub(in crate::store) fn before_relation(
    draft: &mut Draft,
    key: Key,
    field: transaction::RelationField,
) {
    before(draft, Site::Relation(key, field));
}
pub(in crate::store) fn before_payload(draft: &mut Draft) {
    before(draft, Site::Payload);
}
pub(in crate::store) fn after_payload(draft: &mut Draft) {
    ACTIVE.with(|active| {
        if let Some(state) = active.borrow_mut().as_mut()
            && matches!(state.site, Site::Payload)
            && let Some(policy) = state.parent_policy.take()
        {
            draft.source_limits = Some(policy);
        }
    });
}

/// Independent raw occurrence walk for these actual Dialog fixtures. It reads
/// capacities and values, never cached receipts or a measured ledger delta.
pub(in crate::store) fn dialog_fields(draft: &Draft, key: Key) -> custody::Usage {
    fn string(value: &DocumentString, bytes: &mut usize, units: &mut usize) {
        let capacity = value.shared_code_units().capacity();
        *bytes += size_of::<Vec<u16>>() + capacity * size_of::<u16>();
        *units += capacity;
    }
    fn name(
        value: &(impl surgeist_html::HtmlNameInput + ?Sized),
        bytes: &mut usize,
        units: &mut usize,
    ) {
        for value in [
            value.namespace_uri(),
            value.prefix(),
            Some(value.local_name()),
        ]
        .into_iter()
        .flatten()
        {
            string(value, bytes, units);
        }
    }
    let node = draft.node(key).unwrap();
    assert!(
        node.base.is_none()
            && node.base_document.is_none()
            && node.image.is_none()
            && node.hyperlink.is_none()
            && node.encoding_url_document.is_none()
            && node.form_past_names.is_none()
    );
    let DomNodeKind::Element(element) = node.kind else {
        panic!("Dialog element")
    };
    let state = element.dialog_state().expect("actual Dialog");
    // The typed attribute bank and its child frontier have independent
    // custody, prepared before this selected fields-only observation window.
    let mut bytes = size_of::<transaction::NodeFields>() + size_of::<DomDialogState>();
    let mut units = 0;
    name(element.name(), &mut bytes, &mut units);
    for attribute in element.attributes().iter() {
        name(attribute.name(), &mut bytes, &mut units);
        string(attribute.value(), &mut bytes, &mut units);
    }
    for value in [
        element.saved_is(),
        element.nonce(),
        Some(state.return_value()),
        state.request_return_value.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        string(value, &mut bytes, &mut units);
    }
    custody::Usage::new(bytes, units)
}
