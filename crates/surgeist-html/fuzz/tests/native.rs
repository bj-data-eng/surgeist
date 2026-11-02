#![forbid(unsafe_code)]
use surgeist_fuzz::{engine, *};
use surgeist_html_fuzz::{NativeCase, NativeTarget};

#[test]
fn exact_revision_one_codec_retains_utf16_and_rejects_malformed_transport() {
    let t = NativeTarget::new();
    let c = NativeCase::text_fragment(vec![0, 0xd800, 0xdc00, 13, 10]).unwrap();
    let b = t.encode(&c).unwrap();
    assert_eq!(&b[..6], b"HNA1\0\xff");
    assert_eq!(
        &b[46..],
        &[1, 1, 0, 5, 0, 0, 0, 0, 216, 0, 220, 13, 0, 10, 0]
    );
    assert_eq!(t.decode(&b).unwrap(), c);
    assert_eq!(t.encode(&t.decode(&b).unwrap()).unwrap(), b);
    for n in 0..b.len() {
        assert!(t.decode(&b[..n]).is_err(), "{n}");
    }
    for at in [0, 4, 5, 47] {
        let mut bad = b.clone();
        bad[at] = 254;
        assert!(t.decode(&bad).is_err(), "{at}");
    }
    let mut bad = b.clone();
    bad[48] = 2;
    assert!(t.decode(&bad).is_err());
    let mut bad = b.clone();
    bad[6..14].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(t.decode(&bad).is_err());
    let mut bad = b.clone();
    bad[49..51].copy_from_slice(&513u16.to_le_bytes());
    assert!(t.decode(&bad).is_err());
    let mut bad = b.clone();
    bad.push(0);
    assert!(t.decode(&bad).is_err());
    assert!(NativeCase::text_fragment(vec![0; 513]).is_err());
    let mut oversized_backing = Vec::with_capacity(513);
    oversized_backing.push(0);
    assert_eq!(
        NativeCase::text_fragment(oversized_backing)
            .unwrap_err()
            .kind(),
        FuzzErrorKind::Resource
    );
    assert!(c.clone().with_caps([65, 128, 128, 8192, 8]).is_err());
}

#[test]
fn eight_source_literal_recipes_check_real_public_values_and_continuation_after_rejection() {
    let t = NativeTarget::new();
    for id in 0..8 {
        let c = t.literal(id).unwrap();
        let r = engine::evaluate_case(&t, &c, &mut |_| Ok(())).unwrap();
        assert_eq!(
            r.outcome.kind(),
            OutcomeKind::Evaluated,
            "{id}: {:?}",
            r.outcome
        );
        assert!(
            r.outcome
                .checked_properties()
                .iter()
                .any(|p| p.as_str() == "surgeist-html::native-literal-semantics")
        );
        assert_eq!(
            r.outcome,
            engine::replay(&t, &r.canonical, &mut |_| Ok(()))
                .unwrap()
                .outcome
        );
    }
    assert!(t.literal(8).is_err());
    let mut b = t.encode(&t.literal(0).unwrap()).unwrap();
    *b.last_mut().unwrap() = 0;
    assert!(
        t.decode(&b).is_err(),
        "literal tag cannot bless changed expectation bytes"
    );
    assert!(
        t.literal(0)
            .unwrap()
            .with_caps([64, 128, 128, 8192, 8])
            .is_err()
    );
}

#[test]
fn source_derived_exact_native_quotas_and_one_short_refusals_keep_admitted_prefix() {
    let t = NativeTarget::new();
    // One root, one text, one edge, exactly three units and depth one.
    let c = NativeCase::text_fragment(vec![0xd800, 0, 13]).unwrap();
    let exact = [2, 1, 0, 3, 1];
    assert_eq!(
        t.evaluate(&c.clone().with_caps(exact).unwrap())
            .unwrap()
            .kind(),
        OutcomeKind::Evaluated
    );
    for axis in [0, 1, 3, 4] {
        let mut caps = exact;
        caps[axis] -= 1;
        assert_eq!(
            t.evaluate(&c.clone().with_caps(caps).unwrap())
                .unwrap()
                .kind(),
            OutcomeKind::ExpectedRejection,
            "{axis}"
        );
    }
    let mut caps = exact;
    caps[0] = 0;
    assert_eq!(
        t.evaluate(&c.with_caps(caps).unwrap()).unwrap().kind(),
        OutcomeKind::ExpectedRejection
    );
    // Removing the oracle marker preserves the recipe while allowing a product cap.
    let mut b = t.encode(&t.literal(0).unwrap()).unwrap();
    b[5] = 255;
    let general = t
        .decode(&b)
        .unwrap()
        .with_caps([64, 128, 0, 8192, 8])
        .unwrap();
    assert_eq!(
        t.evaluate(&general).unwrap().kind(),
        OutcomeKind::ExpectedRejection
    );
    // Four elements each retain three ordered attributes, including two distinct
    // expanded `id` names. Attribute count is twelve, independently of payload.
    let mut b = t.encode(&t.literal(1).unwrap()).unwrap();
    b[5] = 255;
    let general = t.decode(&b).unwrap();
    for (a, kind) in [
        (12, OutcomeKind::Evaluated),
        (11, OutcomeKind::ExpectedRejection),
    ] {
        assert_eq!(
            t.evaluate(&general.clone().with_caps([64, 128, a, 8192, 8]).unwrap())
                .unwrap()
                .kind(),
            kind
        );
    }
    // Root -> template -> associated fragment -> text has relationship depth 3.
    let template_text = decode_recipe(
        &t,
        0,
        [64, 128, 128, 8192, 8],
        &[
            command(0, 0, &[2, 0]),
            command(5, 1, &[]),
            command(1, 2, &field(&[97])),
        ],
    );
    for (d, kind) in [
        (3, OutcomeKind::Evaluated),
        (2, OutcomeKind::ExpectedRejection),
    ] {
        assert_eq!(
            t.evaluate(
                &template_text
                    .clone()
                    .with_caps([64, 128, 128, 8192, d])
                    .unwrap()
            )
            .unwrap()
            .kind(),
            kind
        );
    }
}

#[test]
fn resource_rejection_is_nonterminal_and_later_independent_append_remains_observable() {
    let t = NativeTarget::new();
    // Root text `a` fits one unit, `bc` exceeds it, then an empty comment fits.
    // Three nodes and two edges are sufficient for precisely the admitted tree.
    let c = NativeCase::text_fragment(vec![97])
        .unwrap()
        .with_caps([3, 2, 0, 1, 1])
        .unwrap();
    let mut b = t.encode(&c).unwrap();
    b[46] = 3;
    b.extend_from_slice(&[1, 0, 2, 0, 98, 0, 99, 0, 2, 0, 0, 0]);
    let c = t.decode(&b).unwrap();
    assert_eq!(c.operation_count(), 3);
    let r = t.evaluate(&c).unwrap();
    assert_eq!(r.kind(), OutcomeKind::ExpectedRejection);
    assert!(
        r.checked_properties()
            .iter()
            .any(|p| p.as_str() == "surgeist-html::native-exact-values")
    );
}

#[test]
fn generated_cases_replay_and_checked_prefix_reduction_preserve_caps_and_stop_immediately() {
    let t = NativeTarget::new();
    let mut positives = 0;
    for first in [0, 4, 8, 12, 16, 20, 24, 28, 1, 2, 3] {
        let r = engine::evaluate_generated(&t, &[first, 0, 216, 0, 220, 13, 0], &mut |_| Ok(()))
            .unwrap();
        let engine::GeneratedEvaluation::Evaluated(r) = r else {
            panic!("bounded generation rejected")
        };
        assert_ne!(
            r.outcome.kind(),
            OutcomeKind::Failure,
            "{first}: {:?}",
            r.outcome
        );
        assert_eq!(
            r.outcome,
            engine::replay(&t, &r.canonical, &mut |_| Ok(()))
                .unwrap()
                .outcome
        );
        if r.outcome.kind() == OutcomeKind::Evaluated {
            positives += 1;
        }
    }
    // The eight literal recipes always evaluate; generated shapes may refuse.
    assert!(positives >= 8, "{positives}");
    assert!(matches!(
        t.generate(&vec![1; 65537]).unwrap(),
        Generation::Rejected(_)
    ));
    assert!(matches!(
        t.generate(&vec![1; 1026]).unwrap(),
        Generation::Rejected(_)
    ));
    let c = NativeCase::text_fragment(vec![0xd800, 0xdc00, 13, 10])
        .unwrap()
        .with_caps([2, 1, 0, 4, 1])
        .unwrap();
    let mut count = 0;
    t.reduction_candidates(&c, &mut |r| {
        assert_eq!(r.caps(), c.caps());
        assert!(t.encode(&r).is_ok());
        count += 1;
        false
    })
    .unwrap();
    assert_eq!(count, 1);
    let mut count = 0;
    t.reduction_candidates(&c, &mut |r| {
        assert_eq!(r.caps(), c.caps());
        assert_eq!(t.decode(&t.encode(&r).unwrap()).unwrap(), r);
        assert_eq!(t.evaluate(&r).unwrap().kind(), OutcomeKind::Evaluated);
        count += 1;
        true
    })
    .unwrap();
    assert!(count >= 4);
    let mut literal_count = 0;
    t.reduction_candidates(&t.literal(4).unwrap(), &mut |_| {
        literal_count += 1;
        true
    })
    .unwrap();
    assert_eq!(literal_count, 0);
}

#[test]
fn canonical_native_seed_files_are_input_recipes_not_observed_output_goldens() {
    let t = NativeTarget::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for id in 0..8 {
        let b = std::fs::read(root.join(format!("seeds/native/{id}.case"))).unwrap();
        assert_eq!(b, t.encode(&t.literal(id).unwrap()).unwrap());
        assert_eq!(
            engine::replay(&t, &b, &mut |_| Ok(()))
                .unwrap()
                .outcome
                .kind(),
            OutcomeKind::Evaluated
        );
    }
}

#[test]
fn shared_worker_protocol_retains_native_canonical_before_real_evaluation() {
    use surgeist_fuzz::worker::*;
    let t = NativeTarget::new();
    let canonical = t.encode(&t.literal(5).unwrap()).unwrap();
    let mut input = Vec::new();
    write_frame(
        &mut input,
        &WorkerRequest::Evaluate {
            canonical: canonical.clone(),
        },
        65536,
    )
    .unwrap();
    let mut output = Vec::new();
    serve_io(
        &t,
        &mut std::io::Cursor::new(input),
        &mut output,
        WorkerLimits::new(65536, 65536, 262144, 4).unwrap(),
    )
    .unwrap();
    let mut r = std::io::Cursor::new(output);
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,65536).unwrap(),Some(WorkerResponse::Captured{canonical:b}) if b==canonical)
    );
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,65536).unwrap(),Some(WorkerResponse::Evaluated{outcome,..}) if outcome.kind()==OutcomeKind::Evaluated)
    );
    assert!(
        read_frame::<_, WorkerResponse>(&mut r, 65536)
            .unwrap()
            .is_none()
    );
}

#[test]
fn registry_names_actual_native_worker_without_changing_parser_descriptors() {
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
    let native = registry
        .get(&TargetId::new("surgeist-html::native-authoring").unwrap())
        .unwrap();
    assert_eq!(native.package(), "surgeist-html-fuzz");
    assert_eq!(native.descriptor().binary(), "html-native-worker");
    assert_eq!(native.descriptor(), NativeTarget::new().descriptor());
    assert!(
        native
            .fixtures()
            .iter()
            .any(|p| p.ends_with("seeds/native") && p.is_dir())
    );
    for id in [
        "surgeist-html::unicode-document",
        "surgeist-html::unicode-fragment",
    ] {
        let target = registry.get(&TargetId::new(id).unwrap()).unwrap();
        assert_eq!(target.descriptor().format().revision(), 1);
        assert_eq!(target.descriptor().limits().bytes(), 65536);
    }
}

#[cfg(feature = "property")]
#[test]
fn finite_property_backend_credits_nonvacuous_real_native_success() {
    use proptest::prelude::*;
    let t = NativeTarget::new();
    let r = engine::run_property(
        &t,
        &Just(vec![16]),
        engine::PropertyConfig {
            cases: 4,
            max_rejections: 0,
            max_shrink_iterations: 8,
            seed: 1139,
        },
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(r.counts.evaluated(), 4);
    assert!(r.failure.is_none());
}
#[cfg(feature = "coverage")]
#[test]
fn finite_coverage_backend_and_canonical_replay_share_native_consumer() {
    let t = NativeTarget::new();
    let mut driver = engine::CoverageDriver::new(&t, None).unwrap();
    let mut bytes = Vec::new();
    let r = driver
        .test_one(&[20], &mut |b| {
            bytes = b.to_vec();
            Ok(())
        })
        .unwrap();
    let engine::GeneratedEvaluation::Evaluated(r) = r else {
        panic!("literal generation rejected")
    };
    assert_eq!(r, engine::replay(&t, &bytes, &mut |_| Ok(())).unwrap());
}
#[cfg(not(feature = "coverage"))]
#[test]
fn actual_native_worker_binary_serves_shared_frames() {
    use std::io::Write;
    use surgeist_fuzz::worker::*;
    let t = NativeTarget::new();
    let canonical = t.encode(&t.literal(7).unwrap()).unwrap();
    let mut b = Vec::new();
    write_frame(
        &mut b,
        &WorkerRequest::Evaluate {
            canonical: canonical.clone(),
        },
        65536,
    )
    .unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_html-native-worker"))
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
        matches!(read_frame::<_,WorkerResponse>(&mut r,65536).unwrap(),Some(WorkerResponse::Captured{canonical:b}) if b==canonical)
    );
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,65536).unwrap(),Some(WorkerResponse::Evaluated{outcome,..}) if outcome.kind()==OutcomeKind::Evaluated)
    );
    assert!(
        read_frame::<_, WorkerResponse>(&mut r, 65536)
            .unwrap()
            .is_none()
    );
}

fn empty_root_text_frame(count: u8) -> Vec<u8> {
    // Literal general HNA1 input: fragment root, no oracle marker, roomy native
    // limits, then `count` empty Text appends to the issued main root.
    let mut bytes = b"HNA1\0\xff".to_vec();
    for cap in [64u64, 128, 128, 8192, 8] {
        bytes.extend_from_slice(&cap.to_le_bytes());
    }
    bytes.push(count);
    for _ in 0..count {
        bytes.extend_from_slice(&[1, 0, 0, 0]);
    }
    bytes
}

#[test]
fn total_native_node_credit_exact_boundary_runs_all_qualification_probes() {
    let target = NativeTarget::new();
    // 61 Text nodes + main root + foreign builder root + independently frozen
    // inspection root = 64 cumulatively, even though the foreign builder drops.
    let bytes = empty_root_text_frame(61);
    assert_eq!(bytes.len(), 291);
    let case = target.decode(&bytes).unwrap();
    assert_eq!(case.operation_count(), 61);
    assert_eq!(target.encode(&case).unwrap(), bytes);
    let outcome = target.evaluate(&case).unwrap();
    assert_eq!(outcome.kind(), OutcomeKind::Evaluated);
    assert!(
        outcome
            .checked_properties()
            .iter()
            .any(|p| p.as_str() == "surgeist-html::native-identities")
    );
}

#[test]
fn total_native_node_credit_one_over_rejects_before_construction() {
    let target = NativeTarget::new();
    // 62 Text appends require 65 total nodes once the same three actual roots
    // are included. This is a transport admission failure, not native refusal.
    let bytes = empty_root_text_frame(62);
    assert_eq!(bytes.len(), 295);
    assert_eq!(
        target.decode(&bytes).unwrap_err().kind(),
        FuzzErrorKind::InvalidCase
    );
}

/// Handwritten HNA1 layout (see README): one command is its tag, parent slot
/// and tag-specific body. These helpers do not use the crate's encoder.
fn command(tag: u8, parent: u8, body: &[u8]) -> Vec<u8> {
    let mut v = vec![tag, parent];
    v.extend_from_slice(body);
    v
}
fn field(units: &[u16]) -> Vec<u8> {
    let mut v = (units.len() as u16).to_le_bytes().to_vec();
    for u in units {
        v.extend_from_slice(&u.to_le_bytes());
    }
    v
}
fn recipe_bytes(root: u8, caps: [u64; 5], commands: &[Vec<u8>]) -> Vec<u8> {
    let mut v = b"HNA1".to_vec();
    v.extend_from_slice(&[root, 255]);
    for cap in caps {
        v.extend_from_slice(&cap.to_le_bytes());
    }
    v.push(commands.len() as u8);
    for c in commands {
        v.extend_from_slice(c);
    }
    v
}
fn decode_recipe(t: &NativeTarget, root: u8, caps: [u64; 5], commands: &[Vec<u8>]) -> NativeCase {
    t.decode(&recipe_bytes(root, caps, commands)).unwrap()
}
fn evaluate_recipe(commands: &[Vec<u8>]) -> CaseOutcome {
    let t = NativeTarget::new();
    t.evaluate(&decode_recipe(&t, 0, [64, 128, 128, 8192, 8], commands))
        .unwrap()
}
const STRUCTURAL: &str = "surgeist-html::native-rejection";

#[test]
fn contract_rejections_are_predicted_from_the_recipe_and_continue_independent_commands() {
    // Each recipe states its expected refusals in the pinned construction
    // contract: a Text/Comment/PI/Doctype cannot be a parent; a native template
    // takes children only through its contents; a host takes one shadow root;
    // attributes sharing namespace and local name are duplicates whatever their
    // prefixes; PI authoring syntax is strict. The final text always succeeds.
    let template = [2u8, 0]; // known HTML template, no attributes
    let known_div = [0u8, 0];
    let cases: [(&str, Vec<Vec<u8>>); 6] = [
        (
            "text under text",
            vec![
                command(1, 0, &field(&[97])),
                command(1, 1, &field(&[98])),
                command(1, 0, &field(&[99])),
            ],
        ),
        (
            "child appended directly to a template",
            vec![
                command(0, 0, &template),
                command(1, 1, &field(&[98])),
                command(1, 0, &field(&[99])),
            ],
        ),
        (
            "second shadow root on one host",
            vec![
                command(0, 0, &known_div),
                command(6, 1, &[0]),
                command(6, 1, &[1]),
                command(1, 0, &field(&[99])),
            ],
        ),
        (
            "shadow root on the fragment root",
            vec![command(6, 0, &[0]), command(1, 0, &field(&[99]))],
        ),
        (
            "foreign builder handle",
            vec![command(1, 255, &field(&[97])), command(1, 0, &field(&[99]))],
        ),
        (
            "reserved and malformed instruction targets",
            vec![
                command(3, 0, &[field(&[120, 109, 108]), field(&[])].concat()),
                command(3, 0, &[field(&[57]), field(&[])].concat()),
                command(1, 0, &field(&[99])),
            ],
        ),
    ];
    for (name, commands) in cases {
        let outcome = evaluate_recipe(&commands);
        assert_eq!(
            outcome.kind(),
            OutcomeKind::Evaluated,
            "{name}: {outcome:?}"
        );
        assert!(
            outcome
                .checked_properties()
                .iter()
                .any(|p| p.as_str() == STRUCTURAL),
            "{name}"
        );
    }
}

#[test]
fn near_miss_template_spellings_are_ordinary_elements_that_accept_children() {
    // DOM creates a template element only for the HTML namespace and the exact
    // local name `template`; a prefix is not identity, a different case is not
    // the template. Appending a child to the near-misses is therefore valid,
    // and appending to the prefixed HTML template is a contents violation.
    let html = "http://www.w3.org/1999/xhtml"
        .encode_utf16()
        .collect::<Vec<_>>();
    let raw = |uri: &[u16], prefix: Option<&[u16]>, local: &str| {
        let mut b = vec![4];
        b.extend(field(uri));
        match prefix {
            None => b.push(0),
            Some(p) => {
                b.push(1);
                b.extend(field(p));
            }
        }
        b.extend(field(&local.encode_utf16().collect::<Vec<_>>()));
        b.push(0);
        b
    };
    let child = command(1, 1, &field(&[97]));
    let near =
        |uri: &[u16], prefix, local| vec![command(0, 0, &raw(uri, prefix, local)), child.clone()];
    for commands in [
        near(&html, None, "Template"),
        near(&html, None, "TEMPLATE"),
        near(
            &"http://www.w3.org/2000/svg"
                .encode_utf16()
                .collect::<Vec<_>>(),
            None,
            "template",
        ),
    ] {
        let outcome = evaluate_recipe(&commands);
        assert_eq!(outcome.kind(), OutcomeKind::Evaluated, "{outcome:?}");
    }
    // Prefixed but exactly HTML `template`: the evaluator predicts the direct
    // child's refusal (a template takes children only through its contents) and
    // fails the case if the builder disagrees.
    let outcome = evaluate_recipe(&near(&html, Some(&[112]), "template"));
    assert_eq!(outcome.kind(), OutcomeKind::Evaluated, "{outcome:?}");
}

#[test]
fn bounded_generation_reaches_every_command_kind_with_no_property_failure() {
    let t = NativeTarget::new();
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let (mut evaluated, mut refused) = (0, 0);
    let mut tags = [false; 7];
    for _ in 0..3000 {
        let len = 2 + (next() % 160) as usize;
        let entropy: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        let Generation::Generated(case) = t.generate(&entropy).unwrap() else {
            continue;
        };
        let canonical = t.encode(&case).unwrap();
        assert_eq!(t.decode(&canonical).unwrap(), case);
        for tag in canonical_tags(&canonical) {
            tags[usize::from(tag)] = true;
        }
        let outcome = t.evaluate(&case).unwrap();
        match outcome.kind() {
            OutcomeKind::Evaluated => evaluated += 1,
            OutcomeKind::ExpectedRejection => refused += 1,
            kind => panic!("{kind:?}: {outcome:?} for {canonical:?}"),
        }
    }
    assert!(evaluated > 1500 && refused > 20, "{evaluated}/{refused}");
    assert_eq!(tags, [true; 7]);
}

/// Command tags of a canonical recipe, by walking the handwritten layout.
fn canonical_tags(b: &[u8]) -> Vec<u8> {
    let (count, mut at) = (usize::from(b[46]), 47);
    let mut tags = Vec::new();
    let take_field =
        |at: &mut usize| *at += 2 + 2 * usize::from(u16::from_le_bytes([b[*at], b[*at + 1]]));
    let take_optional = |at: &mut usize| {
        *at += 1;
        if b[*at - 1] == 1 {
            *at += 2 + 2 * usize::from(u16::from_le_bytes([b[*at], b[*at + 1]]));
        }
    };
    for _ in 0..count {
        let tag = b[at];
        tags.push(tag);
        at += 2;
        let name = |at: &mut usize| {
            *at += 1;
            if b[*at - 1] == 4 {
                *at += 2 + 2 * usize::from(u16::from_le_bytes([b[*at], b[*at + 1]]));
                take_optional(at);
                *at += 2 + 2 * usize::from(u16::from_le_bytes([b[*at], b[*at + 1]]));
            }
        };
        match tag {
            0 => {
                name(&mut at);
                let attrs = b[at];
                at += 1;
                for _ in 0..attrs {
                    at += 1;
                    if b[at - 1] == 0 {
                        name(&mut at);
                        take_field(&mut at);
                    } else {
                        at += 1;
                    }
                }
            }
            1 | 2 => take_field(&mut at),
            3 => {
                take_field(&mut at);
                take_field(&mut at);
            }
            4 => {
                take_field(&mut at);
                take_optional(&mut at);
                take_optional(&mut at);
            }
            5 => {}
            _ => at += 1,
        }
    }
    assert_eq!(at, b.len(), "canonical layout walk");
    tags
}

#[test]
fn removing_an_unreferenced_command_renumbers_later_handles_without_repair() {
    let t = NativeTarget::new();
    // slot 1 text, slot 2 comment (unreferenced), slot 3 element, slot 4 text under slot 3.
    let case = decode_recipe(
        &t,
        0,
        [64, 128, 128, 8192, 8],
        &[
            command(1, 0, &field(&[97])),
            command(2, 0, &field(&[98])),
            command(0, 0, &[0, 0]),
            command(1, 3, &field(&[99])),
        ],
    );
    let mut reduced = Vec::new();
    t.reduction_candidates(&case, &mut |r| {
        reduced.push(r);
        true
    })
    .unwrap();
    let wanted = decode_recipe(
        &t,
        0,
        [64, 128, 128, 8192, 8],
        &[
            command(1, 0, &field(&[97])),
            command(0, 0, &[0, 0]),
            command(1, 2, &field(&[99])),
        ],
    );
    assert!(reduced.contains(&wanted), "{} candidates", reduced.len());
    for r in &reduced {
        assert_eq!(r.caps(), case.caps());
        assert_eq!(t.evaluate(r).unwrap().kind(), OutcomeKind::Evaluated);
    }
}

#[test]
fn native_generator_never_discards_entropy_over_a_fixed_range() {
    // A generator defect must not hide as a generation rejection: every entropy
    // slice of 1 to 300 bytes from this fixed xorshift stream yields a case, and
    // that case encodes and decodes unchanged.
    let t = NativeTarget::new();
    let mut state = 0x1172_C0DE_9E37_79B9u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for i in 0..4000 {
        let len = 1 + (next() % 300) as usize;
        let entropy: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        let Generation::Generated(case) = t.generate(&entropy).unwrap() else {
            panic!("entropy {i} was discarded: {entropy:?}")
        };
        assert_eq!(t.decode(&t.encode(&case).unwrap()).unwrap(), case);
    }
}
