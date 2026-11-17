//! Display text and `source()` of `RenderAccountingError`.
//!
//! The enum is `#[non_exhaustive]`, so an external `match` needs a wildcard; the unit test in
//! `src/accounting/tests.rs` holds the exhaustive match that forces a message for a new variant.
//! `Refused(RenderRefusal)` cannot be built outside the crate (the refusal has no public
//! constructor): its text is covered by `tests/error_messages.rs`, which obtains a real refusal.

use core::error::Error as StdError;
use surgeist_ledger::{Defect, Dimension};
use surgeist_render::{
    accounting::RenderAccountingError as E, accounting::RenderCpuScope, accounting::RenderQuantity,
    accounting::RenderRefusalReason, accounting::RenderResource, accounting::RenderWork,
};

#[test]
fn it_is_a_standard_error_and_plain_data() {
    fn assert_error<T: StdError + Send + Sync + 'static>() {}
    assert_error::<E>();
}

#[test]
fn messages_describe_each_variant() {
    let rows: Vec<(E, String)> = vec![
        (
            E::StandaloneAllocation {
                resource: RenderResource::ScratchCpuBytes,
            },
            "allocating the standalone backing for scratch cpu bytes failed".into(),
        ),
        (
            E::StandaloneCapacityOverflow {
                resource: RenderResource::RetainedCpuBytes,
            },
            "the standalone backing size for retained cpu bytes cannot be represented".into(),
        ),
        (
            E::InvalidLimit(RenderWork::GlyphInputs),
            "glyph inputs work is not produced by the operations this limit applies to and cannot be limited"
                .into(),
        ),
        (E::Invariant, "an internal consistency check failed".into()),
        (
            E::GenerationRetired,
            "the device generation is retired and cannot start new work".into(),
        ),
        (
            E::CpuAllocation {
                scope: RenderCpuScope::ImagePayloads,
                resource: RenderResource::RetainedCpuBytes,
            },
            "allocating cpu backing retained cpu bytes for image_payloads failed".into(),
        ),
        (
            E::CpuCapacityOverflow {
                scope: RenderCpuScope::DeviceBacking,
                resource: RenderResource::RetainedCpuBytes,
            },
            "the cpu backing size of retained cpu bytes for device_backing cannot be represented"
                .into(),
        ),
    ];
    for (error, text) in &rows {
        assert_eq!(&error.to_string(), text, "{error:?}");
        assert_ne!(&format!("{error:?}"), text);
        assert!(error.source().is_none(), "{error:?}");
    }
}

#[test]
fn refusal_figures_are_printed_with_their_reason() {
    for (reason, tail) in [
        (RenderRefusalReason::Limit { limit: 6 }, "the limit is 6"),
        (RenderRefusalReason::Overflow, "the total would overflow"),
        (
            RenderRefusalReason::Underflow,
            "the release exceeds the held amount",
        ),
    ] {
        let generation = E::GenerationRefused {
            quantity: RenderQuantity::Resource(RenderResource::GpuBytes),
            current: 5,
            amount: 3,
            reason,
        };
        assert_eq!(
            generation.to_string(),
            format!("generation accounting refused 3 of gpu bytes at usage 5: {tail}")
        );
        let cpu = E::CpuRefused {
            scope: RenderCpuScope::FrameBacking,
            quantity: RenderQuantity::Work(RenderWork::CommandsEmitted),
            current: 2,
            amount: 1,
            reason,
        };
        assert_eq!(
            cpu.to_string(),
            format!(
                "cpu accounting for frame_backing refused 1 of commands emitted work at usage 2: {tail}"
            )
        );
        assert!(generation.source().is_none() && cpu.source().is_none());
    }
}

#[test]
fn a_ledger_defect_prints_the_ledgers_own_description() {
    let defect = Defect::ScopeEntry;
    let error = E::LedgerDefect(defect);
    assert_eq!(
        error.to_string(),
        format!("the ledger reported an internal defect: {defect}")
    );
    // `Defect` is Display but not Error, so it cannot be returned as a cause.
    assert!(error.source().is_none());
}

#[test]
fn every_work_unit_has_a_distinct_lowercase_label_in_the_limit_message() {
    fn words(name: &str) -> String {
        let mut out = String::new();
        for (index, c) in name.chars().enumerate() {
            if c.is_ascii_uppercase() && index > 0 {
                out.push(' ');
            }
            out.push(c.to_ascii_lowercase());
        }
        out
    }
    let mut seen = std::collections::BTreeSet::new();
    for &work in RenderWork::ALL {
        let label = format!(
            "{} work is not produced by the operations this limit applies to and cannot be limited",
            words(&format!("{work:?}"))
        );
        assert_eq!(E::InvalidLimit(work).to_string(), label);
        assert!(seen.insert(label));
    }
    assert_eq!(seen.len(), RenderWork::ALL.len());
}
