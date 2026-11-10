//! The amounts `ParseControl` charges, compared with the HTML accounting
//! schema's conversions of the same shapes. The expected side is what the
//! controller charges today; the schema side runs through a real ledger.

use crate::parser::control::{HtmlParseStop, ParseControl};
use crate::schema_support::CATEGORIES;
use crate::settings::RESOURCES;
use crate::{HtmlParseCancellation, HtmlParseLimits, HtmlParseResource};
use surgeist_html::accounting::{
    HtmlChange, HtmlCustody, HtmlLimits, HtmlResource, HtmlWork, record_largest,
    retained_provider_storage,
};
use surgeist_ledger::{Change, Dimension, Operation};

fn control() -> ParseControl {
    let limits = HtmlParseLimits::builder()
        .source_units(1_000)
        .buffered_utf16_units(1_000)
        .token_attributes(1_000)
        .auxiliary_entries(1_000)
        .active_formatting_entries(1_000)
        .managed_nodes(1_000)
        .managed_edges(1_000)
        .managed_attributes(1_000)
        .managed_utf16_units(1_000)
        .source_map_entries(1_000)
        .source_map_payload_units(1_000)
        .diagnostics(1_000)
        .diagnostic_payload_units(1_000)
        .context_effects(1_000)
        .open_elements(1_000)
        .tree_depth(1_000)
        .managed_capacity_bytes(16_000)
        .peak_transient_bytes(32_000)
        .atomic_layout_bytes(16_000)
        .work_credits(100_000)
        .finish()
        .unwrap();
    ParseControl::new(limits, HtmlParseCancellation::new())
}

#[test]
fn each_parser_work_category_has_the_schema_dimension_of_the_same_name() {
    for (category, dimension) in CATEGORIES {
        assert_eq!(format!("{category:?}"), format!("{dimension:?}"));
    }
    assert_eq!(CATEGORIES.len(), 8);
}

#[test]
fn parser_resources_are_the_leading_schema_resources_in_precedence_order() {
    for (index, resource) in RESOURCES.iter().enumerate() {
        if *resource == HtmlParseResource::WorkCredits {
            assert_eq!(index, 19);
            continue;
        }
        assert_eq!(*resource as usize, index);
        assert_eq!(
            format!("{resource:?}"),
            format!("{:?}", HtmlResource::ALL[index]),
            "slot {index}"
        );
    }
}

#[test]
fn retained_provider_storage_converts_to_the_resources_the_controller_acquires() {
    let mut c = control();
    c.acquire_retained_provider(640, 12, 3).unwrap();
    for (resource, amount) in retained_provider_storage(640, 12, 3) {
        let parse = RESOURCES[resource as usize];
        assert_eq!(c.usage().resource(parse).current(), amount, "{resource:?}");
        assert_eq!(
            c.usage().resource(parse).high_water(),
            amount,
            "{resource:?}"
        );
    }
}

/// The gauge resources of the controller keep the last value as current and the
/// largest as high water. `record_largest` through a ledger gives the same.
#[test]
fn record_largest_matches_the_controller_gauges() {
    let mut c = control();
    let mut operation: Operation<HtmlWork, HtmlResource> =
        Operation::new("gauges", HtmlLimits::unlimited(), 1).unwrap();
    let root = operation.root();
    let mut layout = 0;
    let mut depth = 0;
    for value in [512, 128, 1_024, 0, 8] {
        c.admit_atomic_layout(value as usize).unwrap();
        operation
            .apply_batch(
                root,
                &record_largest(HtmlResource::AtomicLayoutBytes, layout, value),
            )
            .unwrap();
        layout = value;
        c.observe_tree_depth(value / 2).unwrap();
        operation
            .apply_batch(
                root,
                &record_largest(HtmlResource::TreeDepth, depth, value / 2),
            )
            .unwrap();
        depth = value / 2;
    }
    for (parse, schema) in [
        (
            HtmlParseResource::AtomicLayoutBytes,
            HtmlResource::AtomicLayoutBytes,
        ),
        (HtmlParseResource::TreeDepth, HtmlResource::TreeDepth),
    ] {
        let usage = operation.usage(root).unwrap();
        assert_eq!(usage.live(schema), c.usage().resource(parse).current());
        assert_eq!(usage.peak(schema), c.usage().resource(parse).high_water());
        assert_eq!(schema.custody(), HtmlCustody::Gauge);
    }
}

#[test]
fn the_schema_marks_the_parser_custody_policies() {
    assert_eq!(HtmlResource::ManagedNodes.custody(), HtmlCustody::Permanent);
    assert_eq!(HtmlResource::SourceUnits.custody(), HtmlCustody::Permanent);
    assert_eq!(HtmlResource::Diagnostics.custody(), HtmlCustody::Retention);
    // The controller releases its storage resources and, with provider storage,
    // the two capacity byte resources; nothing else is released.
    for resource in RESOURCES {
        if resource == HtmlParseResource::WorkCredits {
            continue;
        }
        let schema = HtmlResource::ALL[resource as usize];
        let released = (resource.storage() && resource != HtmlParseResource::ManagedNodes)
            || matches!(
                resource,
                HtmlParseResource::ManagedCapacityBytes | HtmlParseResource::PeakTransientBytes
            );
        assert_eq!(
            released,
            matches!(
                schema.custody(),
                HtmlCustody::Balance | HtmlCustody::Retention
            ),
            "{resource:?}"
        );
    }
}

/// The controller refuses retained provider storage at the first resource, in
/// resource precedence order, whose cap the request exceeds. The schema's batch
/// must reach the same first refusal when several caps fail at once.
#[test]
fn retained_provider_storage_is_refused_at_the_resource_the_controller_names() {
    use surgeist_ledger::{Error, Limits};
    let storage = [
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::PeakTransientBytes,
    ];
    // Every subset of two or more of the four storage caps set to zero.
    for mask in 1_u32..16 {
        if mask.count_ones() < 2 {
            continue;
        }
        let failing: Vec<HtmlParseResource> = storage
            .iter()
            .enumerate()
            .filter(|(bit, _)| mask & (1 << bit) != 0)
            .map(|(_, resource)| *resource)
            .collect();
        let mut c = control();
        for resource in &failing {
            c.test_limit(*resource, 0);
        }
        let Err(HtmlParseStop::LimitExceeded { resource, .. }) =
            c.acquire_retained_provider(64, 8, 2)
        else {
            panic!("expected a limit stop for {failing:?}");
        };
        let limits = Limits::from_fn(
            |_| u64::MAX,
            |dimension| {
                if failing
                    .iter()
                    .any(|resource| RESOURCES.get(dimension as usize) == Some(resource))
                {
                    0
                } else {
                    u64::MAX
                }
            },
        );
        let mut operation = Operation::<HtmlWork, HtmlResource>::new("cap", limits, 1).unwrap();
        let root = operation.root();
        let batch: Vec<HtmlChange> = retained_provider_storage(64, 8, 2)
            .into_iter()
            .map(|(dimension, amount)| Change::Acquire(dimension, amount))
            .collect();
        let Err(Error::Refused(refusal)) = operation.apply_batch(root, &batch) else {
            panic!("expected a ledger refusal for {failing:?}");
        };
        let Change::Acquire(refused, _) = refusal.change else {
            panic!("expected an acquire refusal");
        };
        assert_eq!(
            format!("{refused:?}"),
            format!("{resource:?}"),
            "failing caps {failing:?}"
        );
    }
}
