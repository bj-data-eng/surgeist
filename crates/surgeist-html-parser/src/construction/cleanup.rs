//! Shared release and cleanup accounting for retained storage.

use super::*;
use surgeist_html::construction::HtmlConstructionUsage;

/// Running cleanup admits its actual bounded metadata/provider inspections.
/// After a latched stop, only ownership disposal over already-admitted entries
/// is allowed; this does not resume execution or refund any work/high-water.
pub(super) fn cleanup_charge(
    control: &mut ParseControl,
    category: HtmlParseWorkCategory,
    units: u64,
) -> Result<(), SessionError> {
    if control.stop().is_none() {
        control.charge(category, units)?;
    }
    Ok(())
}

/// Records the first refusal of a cleanup step; cleanup continues so every
/// other owner is still released.
pub(super) fn keep_first<E: Into<SessionError>>(
    refused: &mut Option<SessionError>,
    result: Result<(), E>,
) {
    if let Err(error) = result {
        refused.get_or_insert(error.into());
    }
}

pub(super) fn release_logical(
    control: &mut ParseControl,
    owner: &ControlIdentity,
    resource: HtmlParseResource,
    units: u64,
) -> Result<(), SessionError> {
    if control.stop().is_none() && control.release(resource, units).is_ok() {
        return Ok(());
    }
    control
        .cleanup_release(owner, resource, units)
        .map_err(SessionError::Stop)
}

pub(super) fn release_vector<T>(
    control: &mut ParseControl,
    vector: Vec<T>,
    allocation: VectorCapacity<T>,
) -> Result<(), SessionError> {
    control.release_vector(vector, allocation).map_err(|error| {
        let (stop, _, _) = error.into_parts();
        SessionError::Stop(stop)
    })
}

pub(super) fn release_parts(
    control: &mut ParseControl,
    owner: &ControlIdentity,
    parts: ReleasedStorage,
) -> Result<(), SessionError> {
    let ReleasedStorage {
        construction,
        payloads,
    } = parts;
    // Retirement consumes the owner and may refuse; the logical counts it held
    // are read first so they are released either way.
    let mut refused = None;
    let usage = construction.usage(control).unwrap_or_else(|error| {
        // Storage that is not at rest holds counts nothing can name.
        refused = Some(error);
        HtmlConstructionUsage::default()
    });
    let (edges, nodes) = (usage.edges() as u64, usage.nodes() as u64);
    if let Err(error) = construction.retire(control) {
        refused.get_or_insert(error);
    }
    for (resource, units) in [
        (HtmlParseResource::ManagedEdges, edges),
        (
            HtmlParseResource::ManagedAttributes,
            payloads.preexisting_attributes as u64,
        ),
        (
            HtmlParseResource::ManagedUtf16Units,
            payloads.preexisting_units as u64,
        ),
        (HtmlParseResource::AuxiliaryEntries, nodes),
    ] {
        if control.stop().is_none() && control.release(resource, units).is_ok() {
            continue;
        }
        keep_first(
            &mut refused,
            control.cleanup_release(owner, resource, units),
        );
    }
    keep_first(&mut refused, payloads.release(control, owner));
    refused.map_or(Ok(()), Err)
}

pub(super) fn cleanup_owner(
    control: &mut ParseControl,
    owner: &ControlIdentity,
) -> Result<(), SessionError> {
    if control.stop().is_some() {
        control.cleanup_release(owner, HtmlParseResource::AuxiliaryEntries, 0)?;
    } else if let Err(stop) = control.require_identity(owner) {
        // A cancellation can latch during the normal identity checkpoint. The
        // terminal cleanup check still verifies ownership before any consumption.
        control
            .cleanup_release(owner, HtmlParseResource::AuxiliaryEntries, 0)
            .map_err(|_| SessionError::Stop(stop))?;
    }
    Ok(())
}

pub(super) fn payload_write<T>(
    control: &mut ParseControl,
    writes: u64,
) -> Result<(), SessionError> {
    control.admit_atomic_layout(mem::size_of::<T>())?;
    control.charge_many_pub(&[
        (HtmlParseWorkCategory::CopyByte, mem::size_of::<T>() as u64),
        (HtmlParseWorkCategory::RecordWrite, writes),
    ])?;
    Ok(())
}

pub(super) fn take_payload<T>(
    input: &mut Option<T>,
    control: &mut ParseControl,
) -> Result<T, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    if input.is_none() {
        return Err(SessionError::Consumed);
    }
    payload_write::<Option<T>>(control, 2)?;
    input.take().ok_or(SessionError::Consumed)
}
