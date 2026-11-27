//! What a settle decides where the reference census of the whole state takes an
//! order from the iteration of its hash tables, and the figures a state keeps
//! for the states its contexts link.
//!
//! Each expectation is derived from the limits and the structure of the stores
//! built here, and the reference census, which accounts for the whole state from
//! nothing, is asked what it answers on the same state.
use super::{probe, reference};
use crate::{model::State, *};

fn version(name: &str) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: name.into(),
        revision: 1,
    }
}

fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}

/// A store holding one external sheet parsed from `text`, whose context is `context`.
fn store_of(text: &str, context: CssomContext) -> (CssomStore, CssomSheetId) {
    let mut store = CssomStore::new(CssomLimits::default(), context).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let inputs = CssomSheetInputs::external(version("sheet"), true);
    let ticket = edit.create_parsed_sheet(text, inputs).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    (store, sheet)
}

/// A context that links the sheet of `store`, under the identity `name`.
fn linking(name: &str, store: &CssomStore, sheet: &CssomSheetId) -> CssomLinkedInput {
    CssomLinkedInput {
        version: version(name),
        snapshot: store.snapshot(),
        sheet: sheet.clone(),
    }
}

fn context_of(linked: Vec<CssomLinkedInput>) -> CssomContext {
    CssomContext {
        inputs: Vec::new(),
        linked,
    }
}

/// The answer of a settle of every record of `state` under `limits`, without the
/// comparison with the reference that every other settle makes.
fn settled(state: &State, limits: &CssomLimits) -> Result<(), CssomError> {
    probe::set_cross_check(false);
    let answer = state.unsettled_copy().settle(limits, &[]);
    probe::set_cross_check(true);
    answer
}

fn whole(state: &State, limits: &CssomLimits) -> Result<(), CssomError> {
    reference::of(state, limits, std::iter::empty())
        .result
        .map(drop)
}

fn limit(resource: &'static str, maximum: usize) -> String {
    format!("{:?}", CssomError::Limit { resource, maximum })
}

fn rendered(answer: Result<(), CssomError>) -> String {
    format!("{:?}", answer.expect_err("a refusal"))
}

/// The smallest value of a limit, from `0`, at which `admits` holds.
fn smallest(admits: impl Fn(usize) -> bool) -> usize {
    (0..1 << 12).find(|value| admits(*value)).expect("a bound")
}

#[test]
fn several_exceeded_source_limits_are_refused_in_one_order_whatever_the_walk_order() {
    let (store, _) = store_of(
        ".a { color: red } .b { margin: 1px 0 }",
        CssomContext::default(),
    );
    let exceeded = |depth, bytes, steps| CssomLimits {
        max_source_traversal_depth: depth,
        max_retained_source_bytes: bytes,
        max_source_traversal_steps: steps,
        ..CssomLimits::default()
    };
    let all = (0, 0, 0);
    let lifted = usize::MAX;
    // The depth, then the bytes, then the steps.
    for ((depth, bytes, steps), resource, maximum) in [
        (all, "retained source traversal depth", 0),
        ((lifted, 0, 0), "retained parse source bytes", 0),
        ((lifted, lifted, 0), "retained source traversal steps", 0),
    ] {
        let limits = exceeded(depth, bytes, steps);
        assert_eq!(
            rendered(settled(&store.state, &limits)),
            limit(resource, maximum),
            "{limits:?}"
        );
    }
    // The reference's walk charges its first step before it records anything, so it
    // reports the steps where the settle reports the depth.
    assert_eq!(
        rendered(whole(&store.state, &exceeded(0, 0, 0))),
        limit("retained source traversal steps", 0)
    );
}

#[test]
fn the_linked_depth_is_refused_before_the_linked_visits() {
    let (leaf_one, one) = store_of("a { color: red }", CssomContext::default());
    let (leaf_two, two) = store_of("b { color: red }", CssomContext::default());
    let (below, sheet) = store_of(
        "c { color: red }",
        context_of(vec![linking("low", &leaf_one, &one)]),
    );
    // The chain is two deep; two leaves beside it make the visits pass two first
    // in a walk that pops the last input first.
    let context = context_of(vec![
        linking("chain", &below, &sheet),
        linking("one", &leaf_one, &one),
        linking("two", &leaf_two, &two),
    ]);
    let root = CssomStore::new(CssomLimits::default(), context).unwrap();
    let limits = CssomLimits {
        max_depth: 1,
        max_entries: 2,
        ..CssomLimits::default()
    };
    // Four visits: the chain, the leaf it links and the two leaves beside it.
    assert_eq!(
        rendered(settled(&root.state, &limits)),
        limit("linked input depth", 1)
    );
    assert_eq!(
        rendered(whole(&root.state, &limits)),
        limit("linked input visits", 2)
    );
    // With the depth admitted the visits are refused.
    let shallow = CssomLimits {
        max_depth: 2,
        ..limits
    };
    assert_eq!(
        rendered(settled(&root.state, &shallow)),
        limit("linked input visits", 2)
    );
}

#[test]
fn the_depth_of_a_state_several_inputs_reach_is_the_greatest_over_the_paths() {
    // A leaf, and a state that links the leaf: the root links both, the state last, so a
    // walk that takes the last input first reaches the leaf by the short path and never
    // by the long one.
    let (leaf, leaf_sheet) = store_of(".l { color: red; margin: 1px 0 }", CssomContext::default());
    let (middle, middle_sheet) = store_of(
        "m { color: red }",
        context_of(vec![linking("through", &leaf, &leaf_sheet)]),
    );
    let context = context_of(vec![
        linking("middle", &middle, &middle_sheet),
        linking("direct", &leaf, &leaf_sheet),
    ]);
    let root = CssomStore::new(CssomLimits::default(), context).unwrap();
    // The leaf alone is walked at depth zero: the smallest depth limit that admits
    // it is the nesting its records reach.
    let alone = |depth| {
        whole(
            &leaf.state,
            &CssomLimits {
                max_source_traversal_depth: depth,
                ..CssomLimits::default()
            },
        )
        .is_ok()
    };
    let reach = smallest(alone);
    let limits = CssomLimits {
        max_source_traversal_depth: reach + 1,
        ..CssomLimits::default()
    };
    // The leaf is one input below the root by the direct path, and two by the path
    // through the middle state: its records reach `reach + 2` below the root.
    assert_eq!(
        rendered(settled(&root.state, &limits)),
        limit("retained source traversal depth", reach + 1)
    );
    assert!(whole(&root.state, &limits).is_ok());
    let admitted = CssomLimits {
        max_source_traversal_depth: reach + 2,
        ..CssomLimits::default()
    };
    assert!(settled(&root.state, &admitted).is_ok());
    assert!(whole(&root.state, &admitted).is_ok());
}

#[test]
fn a_refused_settle_leaves_the_figures_unusable() {
    let (store, sheet) = store_of(".a { color: red }", CssomContext::default());
    let mut staged = (*store.state).clone();
    staged.sheets.get_mut(&sheet).unwrap().inputs.disabled = true;
    let tight = CssomLimits {
        max_string_bytes: 0,
        ..CssomLimits::default()
    };
    assert!(matches!(
        staged.settle(&tight, &[]),
        Err(CssomError::Limit {
            resource: "host and live string bytes",
            ..
        })
    ));
    assert!(staged.figures.is_poisoned());
    // Whatever the limits, the figures answer no more.
    assert!(matches!(
        staged.settle(&CssomLimits::default(), &[]),
        Err(CssomError::BatchAborted)
    ));
    // The state the batch was staged from is not touched.
    assert!(store.state.is_settled());
}

#[test]
fn a_state_with_unsettled_records_is_not_published() {
    let (mut store, sheet) = store_of(".a { color: red }", CssomContext::default());
    let published = store.snapshot();
    let mut edit = store.batch(guard(&store)).unwrap();
    // A record is changed behind the check of the operation, and the commit does not settle it.
    edit.staged.sheets.get_mut(&sheet).unwrap().inputs.disabled = true;
    edit.affect(
        CssomObjectId::Sheet(sheet.clone()),
        &[CssomChange::Disabled],
    );
    probe::set_unsettled_publication(true);
    let outcome = store.commit(edit);
    probe::set_unsettled_publication(false);
    assert!(matches!(outcome, Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), published.revision());
    assert!(!store.snapshot().sheet(&sheet).unwrap().inputs().disabled);
}

#[test]
fn a_published_state_has_no_journal_and_unusable_figures_never_publish() {
    let (store, sheet) = store_of(".a { color: red }", CssomContext::default());
    assert!(store.state.is_settled());
    let mut edit = store.batch(guard(&store)).unwrap();
    edit.set_disabled(&sheet, true).unwrap();
    assert!(edit.staged.is_settled());
    // A refused operation poisons the batch, which no commit publishes.
    let refused = edit.update_context(context_of(vec![CssomLinkedInput {
        version: version("missing"),
        snapshot: store.snapshot(),
        sheet: CssomSheetId {
            owner: store.state.owner.clone(),
            serial: u64::MAX,
        },
    }]));
    assert!(refused.is_err());
    assert!(!edit.staged.is_settled());
}

/// The figures of `state` against the reference census of it.
fn assert_figures_match(state: &State, at: &str) {
    let census = reference::of(state, &CssomLimits::default(), std::iter::empty())
        .result
        .unwrap_or_else(|error| panic!("{at}: {error:?}"));
    assert_eq!(state.whole_figures(&[]), census.figures(), "{at}");
}

#[test]
fn linked_states_join_and_leave_the_figures_as_the_linked_inputs_change() {
    let (donor, donor_sheet) = store_of(".d { color: red }", CssomContext::default());
    let (other, other_sheet) = store_of(
        "@media print { .o { margin: 1px 0 } }",
        context_of(vec![linking("deep", &donor, &donor_sheet)]),
    );
    let (mut store, _) = store_of(".s { color: blue }", CssomContext::default());
    assert_figures_match(&store.state, "no linked input");
    let contexts = [
        context_of(vec![linking("donor", &donor, &donor_sheet)]),
        // The donor is linked twice, once directly and once through the other state.
        context_of(vec![
            linking("donor", &donor, &donor_sheet),
            linking("other", &other, &other_sheet),
        ]),
        context_of(vec![linking("other", &other, &other_sheet)]),
        CssomContext::default(),
        context_of(vec![
            linking("other", &other, &other_sheet),
            linking("donor", &donor, &donor_sheet),
        ]),
        CssomContext::default(),
    ];
    for (index, context) in contexts.into_iter().enumerate() {
        let mut edit = store.batch(guard(&store)).unwrap();
        assert!(edit.update_context(context).unwrap(), "context {index}");
        store.commit(edit).unwrap();
        assert_figures_match(&store.state, &format!("context {index}"));
    }
}

#[test]
fn a_record_whose_contribution_cannot_be_measured_before_it_changes_refuses_the_settle() {
    let (store, sheet) = store_of(".a { color: red }", CssomContext::default());
    let mut staged = (*store.state).clone();
    // The measurement of the record fails, as an allocation failure makes it, when the
    // table is first asked for the record to change it.
    probe::set_unmeasurable_before(true);
    staged.sheets.get_mut(&sheet).unwrap().inputs.disabled = true;
    probe::set_unmeasurable_before(false);
    // The figures are short of what the record retained, so the settle refuses rather than
    // account for less, and poisons the figures.
    probe::set_cross_check(false);
    let outcome = staged.settle(&CssomLimits::default(), &[]);
    probe::set_cross_check(true);
    assert!(
        matches!(
            outcome,
            Err(CssomError::Limit {
                resource: "retained source traversal storage",
                ..
            })
        ),
        "{outcome:?}"
    );
    assert!(staged.figures.is_poisoned());
    assert!(matches!(
        staged.settle(&CssomLimits::default(), &[]),
        Err(CssomError::BatchAborted)
    ));
}
