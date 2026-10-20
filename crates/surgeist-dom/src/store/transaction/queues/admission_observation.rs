//! Selected allocation policies for private family tests. Preparatory index and
//! record custody remains live before the test installs its finite allowance.
use super::*;
use crate::dialog::DomDialogState;
use std::cell::RefCell;

#[derive(Clone, Copy)]
enum Site {
    NativeTarget(Key),
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
    pub fn native_target(
        key: Key,
        additional: custody::Usage,
        resource: DomResource,
        short: bool,
    ) -> Self {
        Self::new(Site::NativeTarget(key), additional, resource, short)
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
            (Site::NativeTarget(a), Site::NativeTarget(b)) => a == b,
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
pub(in crate::store) fn before_native_target(draft: &mut Draft, key: Key) {
    before(draft, Site::NativeTarget(key));
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
fn after(draft: &mut Draft, matches: impl FnOnce(Site) -> bool) {
    ACTIVE.with(|active| {
        if let Some(state) = active.borrow_mut().as_mut()
            && matches(state.site)
            && let Some(policy) = state.parent_policy.take()
        {
            draft.source_limits = Some(policy);
        }
    });
}
pub(in crate::store) fn after_payload(draft: &mut Draft) {
    after(draft, |site| matches!(site, Site::Payload));
}
pub(in crate::store) fn after_native_target(draft: &mut Draft, key: Key) {
    after(
        draft,
        |site| matches!(site, Site::NativeTarget(selected) if selected == key),
    );
}

/// Exact selected target-header admission, independently derived from the
/// actual Rust layouts. Dialog controllers have already admitted the raw
/// DomDialogState body and all new retained string occurrences before this
/// gate; neither old Element children nor those live inputs are charged again.
///
/// This is not a complete custody/index/compact-binding-plan bound. The native
/// factory restores the original fixture policy after materializing this
/// target, before separately admitted control administration. Its zero unit
/// delta verifies the current retained-unit limit, not newly copied units.
pub(in crate::store) fn dialog_native_target(draft: &Draft, key: Key) -> custody::Usage {
    assert!(
        draft
            .node(key)
            .unwrap()
            .element()
            .unwrap()
            .dialog_state()
            .is_some(),
        "actual Dialog"
    );
    let bytes = size_of::<payload::Payload<DomDialogState>>()
        .checked_sub(size_of::<DomDialogState>())
        .expect("typed target contains its exact native state");
    custody::Usage::new(bytes, 0)
}
