#![forbid(unsafe_code)]
use surgeist_fuzz::{engine, *};
use surgeist_html_fuzz::{MarkupCase, MarkupTarget};

const ENVELOPE: [u64; 19] = [
    100_000,
    100_000,
    10_000,
    1_000_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100,
    100_000,
    100_000_000,
    100_000_000,
    100_000_000,
    100_000_000,
];
const SEEDS: u8 = 16;
const MIRROR: &str = "surgeist-html::capture-mirror";
const FACTS: &str = "surgeist-html::capture-fact-qualification";
const ATOMIC: &str = "surgeist-html::capture-atomic-refusal";

/// Handwritten HCP1 layout (see the README): the HMK1 header, six plan bytes
/// (scripting, registry, `is` slot, shadow facts, fault, pick), then the body.
fn capture_case(plan: [u8; 6], body: &[u8]) -> Vec<u8> {
    let mut v = b"HCP1".to_vec();
    v.extend_from_slice(&[1, 0, 0, 0, 0, 0]); // fragment, UTF-8, div, no shadow, recipe, not cancelled
    for cap in ENVELOPE {
        v.extend_from_slice(&cap.to_le_bytes());
    }
    v.extend_from_slice(&plan);
    v.extend_from_slice(body);
    v
}
/// One text item `a`: a single Unit piece.
const TEXT_A: &[u8] = &[1, 0, 1, 0, 0, 0x61, 0];

fn checked(o: &CaseOutcome, property: &str) -> bool {
    o.checked_properties()
        .iter()
        .any(|p| p.as_str() == property)
}

#[test]
fn strict_codec_carries_the_plan_and_refuses_the_other_targets_encoding() {
    let t = MarkupTarget::capture();
    let bytes = capture_case([1, 2, 2, 5, 3, 7], TEXT_A);
    let c = t.decode(&bytes).unwrap();
    assert_eq!(t.encode(&c).unwrap(), bytes);
    for n in 0..bytes.len() {
        assert!(t.decode(&bytes[..n]).is_err(), "truncated at {n}");
    }
    // Out-of-range scripting, registry, `is`, shadow facts and fault tags.
    for (offset, value) in [(0, 3), (1, 3), (2, 3), (3, 8), (4, 12)] {
        let mut plan = [1, 2, 2, 5, 3, 7];
        plan[offset] = value;
        assert!(
            t.decode(&capture_case(plan, TEXT_A)).is_err(),
            "plan {offset}"
        );
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(t.decode(&trailing).is_err());
    // The parse target's magic is not a capture case, and the reverse.
    let mut parse_magic = bytes.clone();
    parse_magic[..4].copy_from_slice(b"HMK1");
    assert!(t.decode(&parse_magic).is_err());
    assert!(MarkupTarget::parse().decode(&bytes).is_err());
    let parse_case: MarkupCase = MarkupTarget::parse().seed(0).unwrap();
    assert!(t.evaluate(&parse_case).is_err());
    assert!(t.encode(&parse_case).is_err());
}

#[test]
fn every_capture_seed_has_the_independently_stated_outcome() {
    let t = MarkupTarget::capture();
    for id in 0..SEEDS {
        let c = t.seed(id).unwrap();
        let o = t.evaluate(&c).unwrap();
        // Seed 14 stops the adapter on a deliberately tiny work allowance; every
        // other seed completes or is refused exactly as its fault requires.
        let kind = if id == 14 {
            OutcomeKind::ExpectedRejection
        } else {
            OutcomeKind::Evaluated
        };
        assert_eq!(o.kind(), kind, "seed {id}: {o:?}");
        for property in [MIRROR, FACTS, ATOMIC] {
            assert!(checked(&o, property), "seed {id} {property}");
        }
        let r = engine::evaluate_case(&t, &c, &mut |_| Ok(())).unwrap();
        assert_eq!(
            engine::replay(&t, &r.canonical, &mut |_| Ok(()))
                .unwrap()
                .outcome,
            r.outcome
        );
    }
    assert!(t.seed(SEEDS).is_err());
}

#[test]
fn seed_files_are_the_canonical_encoding_of_the_handwritten_cases() {
    let t = MarkupTarget::capture();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds/capture");
    let mut files = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    files.sort();
    let mut wanted = (0..SEEDS).map(|i| format!("{i}.case")).collect::<Vec<_>>();
    wanted.sort();
    assert_eq!(files, wanted);
    for id in 0..SEEDS {
        let b = std::fs::read(root.join(format!("{id}.case"))).unwrap();
        assert_eq!(b, t.encode(&t.seed(id).unwrap()).unwrap(), "{id}");
        assert_eq!(&b[..4], b"HCP1");
    }
}

#[test]
fn external_facts_are_retained_as_supplied_for_every_combination() {
    let t = MarkupTarget::capture();
    // The recipe is one text in a div fragment: a fragment root and a text, with
    // the entry document supplied by the importer. Scripting, registry, `is`
    // slot and shadow facts vary independently; none is inferred from topology.
    for scripting in 0..3 {
        for registry in 0..3 {
            for is_value in 0..3 {
                let c = t
                    .decode(&capture_case(
                        [scripting, registry, is_value, 7, 0, 0],
                        TEXT_A,
                    ))
                    .unwrap();
                let o = t.evaluate(&c).unwrap();
                assert_eq!(
                    o.kind(),
                    OutcomeKind::Evaluated,
                    "{scripting}{registry}{is_value}: {o:?}"
                );
                assert!(checked(&o, FACTS));
            }
        }
    }
}

#[test]
fn each_injected_fault_is_refused_atomically_and_a_retry_completes() {
    let t = MarkupTarget::capture();
    for fault in 1..12u8 {
        for pick in [0u8, 1, 5] {
            let c = t
                .decode(&capture_case([0, 2, 0, 0, fault, pick], TEXT_A))
                .unwrap();
            let o = t.evaluate(&c).unwrap();
            assert!(
                matches!(
                    o.kind(),
                    OutcomeKind::Evaluated | OutcomeKind::ExpectedRejection
                ),
                "fault {fault} pick {pick}: {o:?}"
            );
            assert!(checked(&o, ATOMIC), "fault {fault}");
        }
    }
}

#[test]
fn generated_capture_cases_round_trip_replay_and_never_fail_a_property() {
    let t = MarkupTarget::capture();
    let mut state = 0x5EED_1172_9E37_79B9u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let (mut evaluated, mut refused) = (0, 0);
    for _ in 0..300 {
        let len = 8 + (next() % 160) as usize;
        let entropy: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        let r = engine::evaluate_generated(&t, &entropy, &mut |_| Ok(())).unwrap();
        let engine::GeneratedEvaluation::Evaluated(r) = r else {
            continue;
        };
        match r.outcome.kind() {
            OutcomeKind::Evaluated => evaluated += 1,
            OutcomeKind::ExpectedRejection => refused += 1,
            kind => panic!("{kind:?}: {:?} for {entropy:?}", r.outcome),
        }
        assert_eq!(
            engine::replay(&t, &r.canonical, &mut |_| Ok(()))
                .unwrap()
                .outcome,
            r.outcome
        );
    }
    assert!(evaluated > 150, "{evaluated}/{refused}");
}

#[test]
fn reduction_keeps_the_plan_and_proposes_checked_smaller_cases() {
    let t = MarkupTarget::capture();
    let c = t.seed(8).unwrap();
    let mut proposals = Vec::new();
    t.reduction_candidates(&c, &mut |r| {
        proposals.push(r);
        true
    })
    .unwrap();
    assert!(proposals.len() >= 3);
    for r in &proposals {
        let bytes = t.encode(r).unwrap();
        assert_eq!(&bytes[162..168], &t.encode(&c).unwrap()[162..168]);
        assert_eq!(t.decode(&bytes).unwrap(), *r);
        assert_ne!(t.evaluate(r).unwrap().kind(), OutcomeKind::Failure);
    }
}

#[test]
fn shared_worker_protocol_retains_the_capture_canonical_before_real_evaluation() {
    use surgeist_fuzz::worker::*;
    let t = MarkupTarget::capture();
    let canonical = t.encode(&t.seed(10).unwrap()).unwrap();
    let mut input = Vec::new();
    write_frame(
        &mut input,
        &WorkerRequest::Evaluate {
            canonical: canonical.clone(),
        },
        262144,
    )
    .unwrap();
    let mut output = Vec::new();
    serve_io(
        &t,
        &mut std::io::Cursor::new(input),
        &mut output,
        WorkerLimits::new(262144, 262144, 1_048_576, 4).unwrap(),
    )
    .unwrap();
    let mut r = std::io::Cursor::new(output);
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,262144).unwrap(),Some(WorkerResponse::Captured{canonical:b}) if b==canonical)
    );
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,262144).unwrap(),Some(WorkerResponse::Evaluated{outcome,..}) if outcome.kind()==OutcomeKind::Evaluated)
    );
}

#[cfg(not(feature = "coverage"))]
#[test]
fn actual_capture_worker_binary_serves_shared_frames() {
    use std::io::Write;
    use surgeist_fuzz::worker::*;
    let t = MarkupTarget::capture();
    let canonical = t.encode(&t.seed(3).unwrap()).unwrap();
    let mut b = Vec::new();
    write_frame(
        &mut b,
        &WorkerRequest::Evaluate {
            canonical: canonical.clone(),
        },
        262144,
    )
    .unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_html-capture-worker"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&b).unwrap();
    let o = child.wait_with_output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let mut r = std::io::Cursor::new(o.stdout);
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,262144).unwrap(),Some(WorkerResponse::Captured{canonical:b}) if b==canonical)
    );
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,262144).unwrap(),Some(WorkerResponse::Evaluated{outcome,..}) if outcome.kind()==OutcomeKind::Evaluated)
    );
}

#[test]
fn registry_names_the_capture_worker_with_existing_source_references() {
    use surgeist_fuzz::registry::TargetRegistry;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let output = std::process::Command::new(env!("CARGO"))
        .current_dir(&root)
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let registry = TargetRegistry::from_metadata_json(&root, &output.stdout).unwrap();
    let target = registry
        .get(&TargetId::new("surgeist-html::capture-import").unwrap())
        .unwrap();
    assert_eq!(target.descriptor().binary(), "html-capture-worker");
    assert_eq!(target.descriptor(), MarkupTarget::capture().descriptor());
    assert!(
        target
            .fixtures()
            .iter()
            .any(|p| p.ends_with("seeds/capture") && p.is_dir())
    );
}

#[cfg(feature = "property")]
#[test]
fn finite_property_backend_credits_nonvacuous_capture_success() {
    use proptest::prelude::*;
    let t = MarkupTarget::capture();
    let r = engine::run_property(
        &t,
        &Just(vec![0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        engine::PropertyConfig {
            cases: 4,
            max_rejections: 0,
            max_shrink_iterations: 8,
            seed: 1172,
        },
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(r.counts.evaluated(), 4);
    assert!(r.failure.is_none());
}
#[cfg(feature = "coverage")]
#[test]
fn finite_coverage_backend_and_canonical_replay_share_the_capture_consumer() {
    let t = MarkupTarget::capture();
    let mut driver = engine::CoverageDriver::new(&t, None).unwrap();
    let mut bytes = Vec::new();
    let r = driver
        .test_one(
            &[0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            &mut |b| {
                bytes = b.to_vec();
                Ok(())
            },
        )
        .unwrap();
    let engine::GeneratedEvaluation::Evaluated(r) = r else {
        panic!("generation rejected")
    };
    assert_eq!(r, engine::replay(&t, &bytes, &mut |_| Ok(())).unwrap());
}
