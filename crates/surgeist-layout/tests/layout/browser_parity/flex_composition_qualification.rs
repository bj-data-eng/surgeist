//! Exact qualification for the fixed sideways Flex composition source.

use super::super::capture_authority::{PathDigest, SpecializedEvidence};
use super::*;

#[derive(Serialize)]
struct Policy {
    id: &'static str,
    edition: &'static str,
    source: &'static str,
    sha256: &'static str,
}

const COLLAPSE: Policy = Policy {
    id: "flexbox-collapse-two-round-struts",
    edition: "css-flexbox-1-20251014-native-1",
    source: "semantic/flex-composition-collapse-policy.md",
    sha256: "171dff9f34e0de4b4b1104009b18e9a12fa9610dbebd94f707e63f62078f820a",
};
const SCROLLBAR: Policy = Policy {
    id: "classic-scrollbar-logical-end",
    edition: "css-overflow-3-20251007-native-1",
    source: "semantic/flex-composition-scrollbar-policy.md",
    sha256: "7de645b24bf2d03ebe3ee0c914deb6751d2662b817671aedcea433d35ae1b383",
};

#[derive(Serialize)]
struct Evidence {
    source: &'static str,
    sha256: &'static str,
}

const SCROLLBAR_CONTROL: Evidence = Evidence {
    source: "tests/layout/browser_parity/captured/flex-composition-differences/gutter-control.json",
    sha256: "edcff878101c01973ca991b95a8f7ada4dce5859f9074c63fc1afa91581a71b7",
};

const NATIVE_ORACLES: [Evidence; 2] = [
    Evidence {
        source: "semantic/flex-composition-ltr-native.json",
        sha256: "240a1fcae707b8f9c43e93388a310320d7d117c5148c8ea21069e6f01d28141a",
    },
    Evidence {
        source: "semantic/flex-composition-rtl-native.json",
        sha256: "c9c8024b63447230291bae870a8cc743ee0be950f33d50f2d83ebb58d552f08a",
    },
];

pub(crate) fn input_pins() -> [(&'static str, &'static str); 5] {
    [
        (COLLAPSE.source, COLLAPSE.sha256),
        (SCROLLBAR.source, SCROLLBAR.sha256),
        (SCROLLBAR_CONTROL.source, SCROLLBAR_CONTROL.sha256),
        (NATIVE_ORACLES[0].source, NATIVE_ORACLES[0].sha256),
        (NATIVE_ORACLES[1].source, NATIVE_ORACLES[1].sha256),
    ]
}

pub(crate) fn evidence(variant: &str) -> Option<SpecializedEvidence> {
    let oracle = match variant {
        "border_box_ltr" | "content_box_ltr" => &NATIVE_ORACLES[0],
        "border_box_rtl" | "content_box_rtl" => &NATIVE_ORACLES[1],
        _ => return None,
    };
    let pin = |path: &str, sha256: &str| PathDigest {
        path: path.to_owned(),
        sha256: sha256.to_owned(),
    };
    Some(SpecializedEvidence::FlexComposition {
        collapse_policy: pin(COLLAPSE.source, COLLAPSE.sha256),
        scrollbar_policy: pin(SCROLLBAR.source, SCROLLBAR.sha256),
        native_oracle: pin(oracle.source, oracle.sha256),
        gutter_control: pin(SCROLLBAR_CONTROL.source, SCROLLBAR_CONTROL.sha256),
    })
}

pub struct Inputs {
    ltr: NativeOracle,
    rtl: NativeOracle,
}

fn checked_input(owner: &Path, source: &str, digest: &str) -> Result<Vec<u8>> {
    let bytes =
        std::fs::read(owner.join(source)).map_err(|error| QualificationError::IdentityChanged {
            component: format!("{source}: {error}"),
        })?;
    if super::super::capture_authority::sha256(&bytes) != digest {
        return Err(QualificationError::IdentityChanged {
            component: source.to_owned(),
        });
    }
    Ok(bytes)
}

impl Inputs {
    pub(crate) fn load(owner: &Path) -> Result<Self> {
        let [collapse, scrollbar, control, ltr, rtl] = input_pins();
        for (path, digest) in [collapse, scrollbar, control] {
            checked_input(owner, path, digest)?;
        }
        let read_oracle = |(path, digest)| {
            serde_json::from_slice(&checked_input(owner, path, digest)?)
                .map_err(|error| malformed(error.to_string()))
        };
        Ok(Self {
            ltr: read_oracle(ltr)?,
            rtl: read_oracle(rtl)?,
        })
    }
}

#[derive(Serialize)]
struct Receipt<'a> {
    case: &'a str,
    policies: [&'a Policy; 2],
    scrollbar_control: &'a Evidence,
    qualified: Option<[Assignment<'a>; 2]>,
    observations: &'a [ComparedField],
    raw_differences: Vec<&'a ComparedField>,
    native_packets: &'a [NativeNode],
}

#[derive(Serialize)]
struct Assignment<'a> {
    policy_id: &'a str,
    fields: &'a [ComparedField],
}

pub fn qualify(
    golden: &Golden,
    variant: &str,
    inputs: &Inputs,
    raw: &mut dyn std::io::Write,
) -> Result<(usize, usize)> {
    let comparison = compare(golden, variant, inputs)?;
    let counts = comparison.counts;
    write_receipt(raw, golden, &comparison.completed, comparison.assignments())?;
    comparison.into_outcomes().1?;
    Ok(counts)
}

pub(crate) struct Comparison {
    completed: CompletedComparison,
    expected: Vec<ComparedField>,
    counts: (usize, usize),
    verification: Result<()>,
}

impl Comparison {
    fn assignments(&self) -> Option<[Assignment<'_>; 2]> {
        self.verification.is_ok().then(|| {
            let (collapse, scrollbar) = self.expected.split_at(self.counts.0);
            [
                Assignment {
                    policy_id: COLLAPSE.id,
                    fields: collapse,
                },
                Assignment {
                    policy_id: SCROLLBAR.id,
                    fields: scrollbar,
                },
            ]
        })
    }

    pub(crate) fn into_outcomes(self) -> (Vec<String>, Result<QualifiedComparison>) {
        let raw: Vec<_> = self
            .completed
            .raw_differences()
            .map(|field| format!("{field:?}"))
            .collect();
        let qualified = self.verification.map(|()| {
            let (collapse, scrollbar) = self.expected.split_at(self.counts.0);
            let assignments = [(COLLAPSE, collapse), (SCROLLBAR, scrollbar)]
                .into_iter()
                .filter(|(_, fields)| !fields.is_empty())
                .map(|(policy, fields)| PolicyAssignment {
                    policy: policy.id.to_owned(),
                    edition: policy.edition.to_owned(),
                    fields: fields.iter().map(|field| format!("{field:?}")).collect(),
                })
                .collect();
            QualifiedComparison {
                raw: RawVerdict::Different { fields: raw.len() },
                qualified: QualifiedVerdict::IntentionalFields(assignments),
            }
        });
        (raw, qualified)
    }
}

pub(crate) fn compare(golden: &Golden, variant: &str, inputs: &Inputs) -> Result<Comparison> {
    let rtl = match variant {
        "border_box_ltr" | "content_box_ltr" => false,
        "border_box_rtl" | "content_box_rtl" => true,
        _ => return Err(unsupported("/", "sideways Flex composition variant")),
    };
    if golden.name != format!("fri07_flex_composition__{variant}") {
        return Err(unsupported(
            "/",
            "sideways Flex composition capture authority",
        ));
    }
    let image = golden
        .flex_composition_image
        .as_ref()
        .ok_or_else(|| unsupported("/", "sideways Flex composition capture authority"))?;
    let oracle = if rtl { &inputs.rtl } else { &inputs.ltr };
    let source = source_census(
        &golden.root,
        &golden.expectations,
        None,
        FlexTextCensus::None,
    )?;
    let completed = collect_captured_comparison(golden, &source)?;
    let (mut expected, scrollbar) = expected_pairs(rtl)?;
    let counts = (expected.len(), scrollbar.len());
    expected.extend(scrollbar);
    let verification = completed
        .verify_native(oracle)
        .and_then(|()| verify_selectors(&completed, oracle))
        .and_then(|()| completed.verify_differences(&expected))
        .and_then(|_| verify_native_control(golden, image, &source, oracle));
    Ok(Comparison {
        completed,
        expected,
        counts,
        verification,
    })
}

fn verify_native_control(
    golden: &Golden,
    image: &flex_composition::Proof,
    source: &[SourceNode],
    oracle: &NativeOracle,
) -> Result<()> {
    let map_error = |error: Error| QualificationError::ComparisonFailure {
        reason: format!("collapse-disabled Native control: {error}"),
    };
    let mut tree = TestTree::from_golden(&golden.root, Some(&golden.viewport.root_context))
        .map_err(map_error)?;
    flex_composition::lower(&mut tree, golden, image).map_err(map_error)?;
    let collapsed = tree.nodes[0].children[2];
    let descendant = tree.nodes[collapsed].children[0];
    for index in [collapsed, descendant] {
        let node = &mut tree.nodes[index];
        let mut input = node
            .layout_input
            .as_box()
            .ok_or_else(|| unsupported("/2", "collapse control source box"))?
            .clone();
        input.flex_item_collapse = layout::FlexItemCollapse::Normal;
        node.layout_input = layout::LayoutInput::box_input(input);
    }
    let request = root_request(&golden.viewport.root_context).map_err(map_error)?;
    let batch = compute_actual_batch(&tree, &golden.name, request).map_err(map_error)?;
    let actual = snapshot_batch(&tree, &batch, source)?;

    let [_, _, _, restored, _, scroller, child] = actual.as_slice() else {
        return Err(unsupported("/", "collapse control source census"));
    };
    let [_, _, _, _, _, expected_scroller, expected_child] = oracle.nodes.as_slice() else {
        return Err(unsupported("/", "collapse control oracle census"));
    };
    for (restored, scroller, expected) in [
        (
            &restored.unrounded,
            &scroller.unrounded,
            &expected_scroller.unrounded,
        ),
        (
            &restored.final_output,
            &scroller.final_output,
            &expected_scroller.final_output,
        ),
    ] {
        let (
            Packet::Present { facts: restored },
            Packet::Present { facts: scroller },
            Packet::Present { facts: expected },
        ) = (restored, scroller, expected)
        else {
            return Err(unsupported(
                "/3",
                "collapse control requires both visible packets",
            ));
        };
        if restored.size != extent(layout::Size::new(100.0, 30.0))?
            || scroller.size != expected.size
            || scroller.content_size != expected.content_size
            || scroller.scroll != expected.scroll
        {
            return Err(unsupported(
                "/3",
                "collapse control changed scroller-local geometry",
            ));
        }
    }
    compare_native(expected_child, child)
}

fn write_receipt(
    raw: &mut dyn std::io::Write,
    golden: &Golden,
    completed: &CompletedComparison,
    qualified: Option<[Assignment<'_>; 2]>,
) -> Result<()> {
    let receipt = Receipt {
        case: &golden.name,
        policies: [&COLLAPSE, &SCROLLBAR],
        scrollbar_control: &SCROLLBAR_CONTROL,
        qualified,
        observations: &completed.observations,
        raw_differences: completed.raw_differences().collect(),
        native_packets: &completed.nodes,
    };
    serde_json::to_writer(&mut *raw, &receipt).map_err(|error| malformed(error.to_string()))?;
    raw.write_all(b"\n")
        .map_err(|error| malformed(error.to_string()))
}

fn verify_selectors(completed: &CompletedComparison, oracle: &NativeOracle) -> Result<()> {
    let expected: BTreeSet<_> = oracle
        .nodes
        .iter()
        .flat_map(|node| {
            let ranges = if matches!(node.path.as_str(), "/" | "/2" | "/3") {
                &[
                    CanonicalField::LayoutScrollWidth,
                    CanonicalField::LayoutScrollHeight,
                ][..]
            } else {
                &[]
            };
            CurrentDomain::Layout
                .fields()
                .iter()
                .chain(ranges)
                .map(|&field| Selector {
                    path: node.path.clone(),
                    field,
                })
        })
        .collect();
    let actual: BTreeSet<_> = completed
        .observations
        .iter()
        .map(ComparedField::canonical_selector)
        .collect();
    if actual != expected {
        return Err(QualificationError::IncompleteRepresentation {
            key: "complete sideways Flex comparison selectors".to_owned(),
        });
    }
    Ok(())
}

fn expected_pairs(rtl: bool) -> Result<(Vec<ComparedField>, Vec<ComparedField>)> {
    use NumericComparisonField::{Height, ScrollHeight, ScrollWidth, Width, X, Y};

    let collapse = [
        (
            "/0",
            Y,
            if rtl { 134.0 } else { 6.0 },
            Some(if rtl { 113.0 } else { 27.0 }),
        ),
        ("/2", Y, if rtl { 92.0 } else { 58.0 }, Some(0.0)),
        ("/2", Width, 100.0, Some(0.0)),
        ("/2", Height, 30.0, Some(0.0)),
        ("/2", ScrollWidth, 40.0, None),
        ("/2", ScrollHeight, 0.0, None),
        ("/2/0", X, -40.0, Some(0.0)),
        ("/2/0", Width, 140.0, Some(0.0)),
        ("/2/0", Height, 15.0, Some(0.0)),
    ]
    .into_iter()
    .map(|(path, field, browser, native)| {
        ComparedField::layout(path.into(), field, browser, native)
    })
    .collect::<Result<Vec<_>>>()?;
    let mut scrollbar = Vec::new();
    if rtl {
        scrollbar.push(ComparedField::layout("/3/0".into(), Y, 0.0, Some(15.0))?);
    }
    Ok((collapse, scrollbar))
}
