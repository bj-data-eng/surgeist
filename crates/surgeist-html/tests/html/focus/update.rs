//! Independent HTML877d raw60227–60311 behavior, including the selected
//! editorial interpretations. This adapter owns finite example chains and
//! operand identities, not a native focus/renderer/Window implementation.
use std::collections::{BTreeMap, BTreeSet};
use surgeist_html::{
    HtmlFocusEventAction as EA, HtmlFocusEventCursor as EC, HtmlFocusEventInterface as Interface,
    HtmlFocusEventName as Name, HtmlFocusEventReply as ER,
    HtmlFocusEventTargetKind as EventTargetKind, HtmlFocusEventView as View,
    HtmlFocusUpdateAction as A, HtmlFocusUpdateChain as Chain, HtmlFocusUpdateCursor as C,
    HtmlFocusUpdateEntryKind as Kind, HtmlFocusUpdateError as Error,
    HtmlFocusUpdateEventTarget as TargetRole, HtmlFocusUpdateInputData as Data,
    HtmlFocusUpdateRelatedTarget as Related, HtmlFocusUpdateReply as R,
};

fn replies() -> Vec<R> {
    let mut values = vec![
        R::OldTailPresent(false),
        R::OldTailPresent(true),
        R::NewTailPresent(false),
        R::NewTailPresent(true),
        R::TailIdentityEqual(false),
        R::TailIdentityEqual(true),
        R::OldEntryPresent(false),
        R::OldEntryPresent(true),
        R::OldEntryIsInput(false),
        R::OldEntryIsInput(true),
        R::ChangeEventApplies(false),
        R::ChangeEventApplies(true),
        R::HasActivationBehavior(false),
        R::HasActivationBehavior(true),
        R::NewEntryPresent(false),
        R::NewEntryPresent(true),
        R::NewEntryFocusableArea(false),
        R::NewEntryFocusableArea(true),
        R::EntryMatchesDesignation(false),
        R::EntryMatchesDesignation(true),
        R::ChangeDispatched,
        R::FocusEventCompleted,
        R::Applied,
    ];
    for data in [Data::Value, Data::SelectedFiles] {
        for value in [false, true] {
            values.push(R::InputUserChangedDataWhileFocused { data, value });
            values.push(R::InputChangeUncommitted { data, value });
            values.push(R::InputDiffersFromFirstFocus { data, value });
        }
    }
    for chain in [Chain::Old, Chain::New] {
        values.push(R::EntryIsLast {
            chain,
            value: false,
        });
        values.push(R::EntryIsLast { chain, value: true });
        values.push(R::OppositeTailKind { chain, kind: None });
        for kind in [Kind::Element, Kind::Document, Kind::Other] {
            values.push(R::EntryKind { chain, kind });
            values.push(R::OppositeTailKind {
                chain,
                kind: Some(kind),
            });
        }
    }
    values
}

// Facts with a different Boolean value are valid alternative observations.
// A different chain/data channel is a wrong occurrence, even at the same stage.
fn same_domain(a: &R, b: &R) -> bool {
    match (a, b) {
        (
            R::InputUserChangedDataWhileFocused { data: a, .. },
            R::InputUserChangedDataWhileFocused { data: b, .. },
        )
        | (R::InputChangeUncommitted { data: a, .. }, R::InputChangeUncommitted { data: b, .. })
        | (
            R::InputDiffersFromFirstFocus { data: a, .. },
            R::InputDiffersFromFirstFocus { data: b, .. },
        ) => a == b,
        (R::EntryKind { chain: a, .. }, R::EntryKind { chain: b, .. })
        | (R::EntryIsLast { chain: a, .. }, R::EntryIsLast { chain: b, .. })
        | (R::OppositeTailKind { chain: a, .. }, R::OppositeTailKind { chain: b, .. }) => a == b,
        _ => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

fn accept(cursor: &mut C, control: &mut C, reply: R) {
    for wrong in replies() {
        if !same_domain(&wrong, &reply) {
            assert_eq!(cursor.reply(wrong), Err(Error::WrongReply));
            assert_eq!(cursor, control);
        }
    }
    cursor.reply(reply).unwrap();
    control.reply(reply).unwrap();
    assert_eq!(cursor, control);
}

#[derive(Clone, Copy, Default)]
struct Change {
    user_changed: bool,
    uncommitted: bool,
    differs: bool,
}
const DIRTY: Change = Change {
    user_changed: true,
    uncommitted: true,
    differs: true,
};
#[derive(Clone, Copy)]
struct Input {
    applies: bool,
    activation: bool,
    value: Change,
    files: Change,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            applies: true,
            activation: false,
            value: Change::default(),
            files: Change::default(),
        }
    }
}
#[derive(Clone, Copy)]
struct Entry {
    kind: Kind,
    area: bool,
    // The area/entry and its anchor are deliberately independent identities.
    anchor: Anchor,
    input: Option<Input>,
}
#[derive(Clone, Copy)]
enum Anchor {
    Element(u8),
    Document(u8),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Target {
    Element(u8),
    Window(u8),
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Event {
    name: &'static str,
    target: Target,
    related: Option<u8>,
    view: Option<u8>,
    interface: Option<Interface>,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
}
#[derive(Debug, Eq, PartialEq)]
enum Effect {
    Pop(Chain, u8),
    Validity(u8),
    Event(Event),
    Platform,
    CaptureDocument(u8, u8),
    NavigationTrue(u8, u8),
    Designation(u8, u8),
}

struct Fixture {
    old: Vec<u8>,
    new: Vec<u8>,
    entries: BTreeMap<u8, Entry>,
    element_documents: BTreeMap<u8, u8>,
    globals: BTreeMap<u8, u8>,
    designation: BTreeMap<u8, Option<u8>>,
    validity: BTreeSet<u8>,
    trace: Vec<A>,
    effects: Vec<Effect>,
    change_adoption: Option<(u8, u8)>,
    blur_adoption: Option<(u8, u8)>,
}
impl Fixture {
    fn new(old: &[u8], new: &[u8]) -> Self {
        Self {
            old: old.to_vec(),
            new: new.to_vec(),
            entries: BTreeMap::new(),
            element_documents: BTreeMap::new(),
            globals: [(10, 101), (20, 102), (30, 103)].into(),
            designation: [(10, None), (20, None), (30, None)].into(),
            validity: BTreeSet::new(),
            trace: Vec::new(),
            effects: Vec::new(),
            change_adoption: None,
            blur_adoption: None,
        }
    }
    fn element(&mut self, id: u8, document: u8, area: bool) {
        self.element_documents.insert(id, document);
        self.entries.insert(
            id,
            Entry {
                kind: Kind::Element,
                area,
                anchor: Anchor::Element(id),
                input: None,
            },
        );
    }
    fn document(&mut self, id: u8) {
        self.entries.insert(
            id,
            Entry {
                kind: Kind::Document,
                area: false,
                anchor: Anchor::Document(id),
                input: None,
            },
        );
    }
    fn other_area(&mut self, id: u8, anchor: Anchor) {
        self.entries.insert(
            id,
            Entry {
                kind: Kind::Other,
                area: true,
                anchor,
                input: None,
            },
        );
    }
    fn input(&mut self, id: u8, document: u8, input: Input) {
        self.element(id, document, true);
        self.entries.get_mut(&id).unwrap().input = Some(input);
    }
    fn chain(&self, chain: Chain) -> &[u8] {
        match chain {
            Chain::Old => &self.old,
            Chain::New => &self.new,
        }
    }
    fn document_of(&self, id: u8) -> u8 {
        match self.entries[&id].anchor {
            Anchor::Element(anchor) => self.element_documents[&anchor],
            Anchor::Document(document) => document,
        }
    }
    fn data(&self, id: u8, data: Data) -> Change {
        let input = self.entries[&id].input.unwrap();
        match data {
            Data::Value => input.value,
            Data::SelectedFiles => input.files,
        }
    }
    fn focus_event(&self, name: Name, target: Target, related: Option<u8>) -> Event {
        let kind = match target {
            Target::Element(_) => EventTargetKind::Element,
            Target::Window(_) => EventTargetKind::Window,
        };
        let mut child = EC::new(kind);
        let view = match (child.action(), target) {
            (EA::CaptureWindowView, Target::Window(window)) => window,
            (EA::CaptureElementDocumentGlobalView, Target::Element(element)) => {
                self.globals[&self.element_documents[&element]]
            }
            other => panic!("wrong actual view request: {other:?}"),
        };
        child.reply(ER::Applied).unwrap();
        let EA::Dispatch {
            interface,
            view: role,
            bubbles,
            cancelable,
            composed,
        } = child.action()
        else {
            panic!("focus-event child must reach actual dispatch")
        };
        assert_eq!(
            role,
            match kind {
                EventTargetKind::Element => View::ElementDocumentGlobal,
                EventTargetKind::Window => View::TargetWindow,
            }
        );
        let event = Event {
            name: name.as_str(),
            target,
            related,
            view: Some(view),
            interface: Some(interface),
            bubbles,
            cancelable,
            composed,
        };
        child.reply(ER::Dispatched).unwrap();
        assert_eq!(child.action(), EA::Complete);
        event
    }
    fn run(&mut self) {
        let mut cursor = C::new();
        let mut control = C::new();
        let mut old_next = 0;
        let mut new_next = 0;
        let mut old_entry = None;
        let mut new_entry = None;
        let mut captured_document = None;
        let mut event_global = None;
        for _ in 0..512 {
            let action = cursor.action();
            self.trace.push(action);
            let reply = match action {
                A::ReadOldTailPresent => R::OldTailPresent(!self.old.is_empty()),
                A::ReadNewTailPresent => R::NewTailPresent(!self.new.is_empty()),
                A::ReadTailIdentityEqual => {
                    assert!(!self.old.is_empty() && !self.new.is_empty());
                    R::TailIdentityEqual(self.old.last() == self.new.last())
                }
                A::PopOldTail => {
                    self.effects
                        .push(Effect::Pop(Chain::Old, self.old.pop().unwrap()));
                    R::Applied
                }
                A::PopNewTail => {
                    self.effects
                        .push(Effect::Pop(Chain::New, self.new.pop().unwrap()));
                    R::Applied
                }
                A::BeginOldEntries => {
                    old_next = 0;
                    R::Applied
                }
                A::NextOldEntry => {
                    old_entry = self.old.get(old_next).copied();
                    old_next += usize::from(old_entry.is_some());
                    R::OldEntryPresent(old_entry.is_some())
                }
                A::ReadOldEntryIsInput => {
                    R::OldEntryIsInput(self.entries[&old_entry.unwrap()].input.is_some())
                }
                A::ReadChangeEventApplies => {
                    R::ChangeEventApplies(self.entries[&old_entry.unwrap()].input.unwrap().applies)
                }
                A::ReadHasActivationBehavior => R::HasActivationBehavior(
                    self.entries[&old_entry.unwrap()].input.unwrap().activation,
                ),
                A::ReadInputUserChangedDataWhileFocused { data } => {
                    R::InputUserChangedDataWhileFocused {
                        data,
                        value: self.data(old_entry.unwrap(), data).user_changed,
                    }
                }
                A::ReadInputChangeUncommitted { data } => R::InputChangeUncommitted {
                    data,
                    value: self.data(old_entry.unwrap(), data).uncommitted,
                },
                A::ReadInputDiffersFromFirstFocus { data } => R::InputDiffersFromFirstFocus {
                    data,
                    value: self.data(old_entry.unwrap(), data).differs,
                },
                A::SetUserValidityTrue => {
                    let id = old_entry.unwrap();
                    self.validity.insert(id);
                    self.effects.push(Effect::Validity(id));
                    R::Applied
                }
                A::FireChange {
                    bubbles,
                    cancelable,
                    composed,
                } => {
                    let id = old_entry.unwrap();
                    assert!(self.validity.contains(&id));
                    self.effects.push(Effect::Event(Event {
                        name: "change",
                        target: Target::Element(id),
                        related: None,
                        view: None,
                        interface: None,
                        bubbles,
                        cancelable,
                        composed,
                    }));
                    if let Some((element, document)) = self.change_adoption.take() {
                        self.element_documents.insert(element, document);
                    }
                    R::ChangeDispatched
                }
                A::ReadEntryKind { chain } => {
                    let id = match chain {
                        Chain::Old => old_entry.unwrap(),
                        Chain::New => new_entry.unwrap(),
                    };
                    R::EntryKind {
                        chain,
                        kind: self.entries[&id].kind,
                    }
                }
                A::CaptureEntryRelevantGlobal { chain } => {
                    let id = match chain {
                        Chain::Old => old_entry.unwrap(),
                        Chain::New => new_entry.unwrap(),
                    };
                    assert_eq!(self.entries[&id].kind, Kind::Document);
                    event_global = Some(self.globals[&id]);
                    R::Applied
                }
                A::ReadEntryIsLast { chain } => {
                    let entry = match chain {
                        Chain::Old => old_entry,
                        Chain::New => new_entry,
                    };
                    R::EntryIsLast {
                        chain,
                        value: entry.as_ref() == self.chain(chain).last(),
                    }
                }
                A::ReadOppositeTailKind { chain } => {
                    let opposite = match chain {
                        Chain::Old => Chain::New,
                        Chain::New => Chain::Old,
                    };
                    R::OppositeTailKind {
                        chain,
                        kind: self.chain(opposite).last().map(|id| self.entries[id].kind),
                    }
                }
                A::FireFocusEvent {
                    name,
                    target,
                    related,
                } => {
                    let (chain, entry) = match name {
                        Name::Blur => (Chain::Old, old_entry.unwrap()),
                        Name::Focus => (Chain::New, new_entry.unwrap()),
                    };
                    let target = match target {
                        TargetRole::EntryElement => Target::Element(entry),
                        TargetRole::EntryDocumentGlobal => Target::Window(event_global.unwrap()),
                    };
                    let related = match related {
                        Related::Null => None,
                        Related::OppositeChainLastElement => {
                            let opposite = match chain {
                                Chain::Old => Chain::New,
                                Chain::New => Chain::Old,
                            };
                            Some(*self.chain(opposite).last().unwrap())
                        }
                    };
                    let event = self.focus_event(name, target, related);
                    self.effects.push(Effect::Event(event));
                    if name == Name::Blur
                        && let Some((element, document)) = self.blur_adoption.take()
                    {
                        self.element_documents.insert(element, document);
                    }
                    R::FocusEventCompleted
                }
                A::ApplyPlatformConventions => {
                    self.effects.push(Effect::Platform);
                    R::Applied
                }
                A::BeginNewEntriesReverse => {
                    new_next = self.new.len();
                    R::Applied
                }
                A::NextNewEntry => {
                    new_entry = if new_next == 0 {
                        None
                    } else {
                        new_next -= 1;
                        Some(self.new[new_next])
                    };
                    R::NewEntryPresent(new_entry.is_some())
                }
                A::ReadNewEntryFocusableArea => {
                    R::NewEntryFocusableArea(self.entries[&new_entry.unwrap()].area)
                }
                A::CaptureEntryDocument => {
                    let id = new_entry.unwrap();
                    let document = self.document_of(id);
                    captured_document = Some(document);
                    self.effects.push(Effect::CaptureDocument(id, document));
                    R::Applied
                }
                A::ReadEntryMatchesDesignation => R::EntryMatchesDesignation(
                    self.designation[&captured_document.unwrap()] == new_entry,
                ),
                A::SetWindowFocusChangedDuringNavigationTrue => {
                    let document = captured_document.unwrap();
                    self.effects
                        .push(Effect::NavigationTrue(document, self.globals[&document]));
                    R::Applied
                }
                A::DesignateEntryFocusedArea => {
                    let document = captured_document.unwrap();
                    let id = new_entry.unwrap();
                    self.designation.insert(document, Some(id));
                    self.effects.push(Effect::Designation(document, id));
                    R::Applied
                }
                A::Complete => {
                    for reply in replies() {
                        assert_eq!(cursor.reply(reply), Err(Error::Terminal));
                        assert_eq!(cursor, control);
                    }
                    return;
                }
            };
            accept(&mut cursor, &mut control, reply);
        }
        panic!("bounded fixture failed to complete: {:?}", cursor.action());
    }
    fn events(&self) -> Vec<Event> {
        self.effects
            .iter()
            .filter_map(|effect| match effect {
                Effect::Event(event) => Some(event.clone()),
                _ => None,
            })
            .collect()
    }
}

fn focus(name: &'static str, target: Target, related: Option<u8>, view: u8) -> Event {
    Event {
        name,
        target,
        related,
        view: Some(view),
        interface: Some(Interface::FocusEvent),
        bubbles: false,
        cancelable: false,
        composed: true,
    }
}

#[test]
fn empty_and_identical_chains_trim_only_real_common_tails_then_still_apply_platform_conventions() {
    for (old, new, expected_pops) in [
        (vec![], vec![], vec![]),
        (vec![10], vec![], vec![]),
        (vec![], vec![10], vec![]),
        (
            vec![10],
            vec![10],
            vec![Effect::Pop(Chain::Old, 10), Effect::Pop(Chain::New, 10)],
        ),
        (
            vec![1, 10],
            vec![1, 10],
            vec![
                Effect::Pop(Chain::Old, 10),
                Effect::Pop(Chain::New, 10),
                Effect::Pop(Chain::Old, 1),
                Effect::Pop(Chain::New, 1),
            ],
        ),
    ] {
        let mut f = Fixture::new(&old, &new);
        for id in [1, 10] {
            f.entries.insert(
                id,
                Entry {
                    kind: Kind::Other,
                    area: false,
                    anchor: Anchor::Document(10),
                    input: None,
                },
            );
        }
        f.run();
        let mut expected = expected_pops;
        expected.push(Effect::Platform);
        assert_eq!(f.effects, expected);
        assert_eq!(
            f.trace
                .iter()
                .filter(|a| **a == A::ApplyPlatformConventions)
                .count(),
            1
        );
    }
}

#[test]
fn old_entries_run_forward_platform_precedes_reverse_new_entries_and_area_anchors_do_not_emit_events()
 {
    let mut f = Fixture::new(&[1, 2, 10], &[3, 4, 20]);
    f.element(1, 10, true);
    f.other_area(2, Anchor::Element(1));
    f.document(10);
    f.element(3, 20, true);
    f.other_area(4, Anchor::Document(20));
    f.document(20);
    f.run();
    assert_eq!(
        f.events(),
        vec![
            focus("blur", Target::Element(1), None, 101),
            focus("blur", Target::Window(101), None, 101),
            focus("focus", Target::Window(102), None, 102),
            focus("focus", Target::Element(3), None, 102),
        ]
    );
    assert_eq!(
        f.effects,
        vec![
            Effect::Event(focus("blur", Target::Element(1), None, 101)),
            Effect::Event(focus("blur", Target::Window(101), None, 101)),
            Effect::Platform,
            Effect::Event(focus("focus", Target::Window(102), None, 102)),
            Effect::CaptureDocument(4, 20),
            Effect::NavigationTrue(20, 102),
            Effect::Designation(20, 4),
            Effect::CaptureDocument(3, 20),
            Effect::NavigationTrue(20, 102),
            Effect::Designation(20, 3),
            Effect::Event(focus("focus", Target::Element(3), None, 102)),
        ]
    );
}

#[test]
fn related_targets_are_the_opposite_trimmed_chain_last_elements_not_anchors_or_last_iteration() {
    let mut f = Fixture::new(&[1, 2, 10], &[3, 4, 10]);
    f.element(2, 10, false);
    f.other_area(1, Anchor::Element(2));
    f.element(4, 20, false);
    f.other_area(3, Anchor::Element(4));
    f.document(10);
    f.run();
    assert_eq!(f.old, vec![1, 2]);
    assert_eq!(f.new, vec![3, 4]);
    assert_eq!(
        f.events(),
        vec![
            focus("blur", Target::Element(2), Some(4), 101),
            focus("focus", Target::Element(4), Some(2), 102),
        ]
    );
    assert_eq!(
        &f.effects[..2],
        &[Effect::Pop(Chain::Old, 10), Effect::Pop(Chain::New, 10)]
    );
}

#[test]
fn already_designated_area_has_no_navigation_or_designation_write_but_still_fires_its_focus_event()
{
    let mut f = Fixture::new(&[], &[1]);
    f.element(1, 10, true);
    f.designation.insert(10, Some(1));
    f.run();
    assert_eq!(
        f.effects,
        vec![
            Effect::Platform,
            Effect::CaptureDocument(1, 10),
            Effect::Event(focus("focus", Target::Element(1), None, 101)),
        ]
    );
}

fn input_fixture(input: Input) -> Fixture {
    let mut f = Fixture::new(&[1], &[]);
    f.input(1, 10, input);
    f
}

#[test]
fn input_change_guards_short_circuit_before_history_and_non_inputs_never_query_applicability() {
    let mut non_input = Fixture::new(&[1], &[]);
    non_input.element(1, 10, true);
    non_input.run();
    assert!(!non_input.trace.contains(&A::ReadChangeEventApplies));
    for input in [
        Input {
            applies: false,
            ..Input::default()
        },
        Input {
            activation: true,
            ..Input::default()
        },
    ] {
        let mut f = input_fixture(input);
        f.run();
        assert!(f.validity.is_empty());
        assert_eq!(
            f.events(),
            vec![focus("blur", Target::Element(1), None, 101)]
        );
        assert!(
            !f.trace
                .iter()
                .any(|a| matches!(a, A::ReadInputUserChangedDataWhileFocused { .. }))
        );
        if !input.applies {
            assert!(!f.trace.contains(&A::ReadHasActivationBehavior));
        }
    }
}

#[test]
fn either_data_channel_requires_all_three_history_facts_and_validity_precedes_bubbling_change() {
    let negative = [
        Change {
            user_changed: false,
            ..DIRTY
        },
        Change {
            uncommitted: false,
            ..DIRTY
        },
        Change {
            differs: false,
            ..DIRTY
        },
    ];
    for value in negative {
        for files in negative {
            let mut f = input_fixture(Input {
                value,
                files,
                ..Input::default()
            });
            f.run();
            assert!(f.validity.is_empty());
            assert_eq!(
                f.events(),
                vec![focus("blur", Target::Element(1), None, 101)]
            );
        }
    }
    for (value, files) in [
        (DIRTY, Change::default()),
        (Change::default(), DIRTY),
        (DIRTY, DIRTY),
    ] {
        let mut f = input_fixture(Input {
            value,
            files,
            ..Input::default()
        });
        f.run();
        let change = Event {
            name: "change",
            target: Target::Element(1),
            related: None,
            view: None,
            interface: None,
            bubbles: true,
            cancelable: false,
            composed: false,
        };
        assert_eq!(
            f.effects,
            vec![
                Effect::Validity(1),
                Effect::Event(change),
                Effect::Event(focus("blur", Target::Element(1), None, 101)),
                Effect::Platform,
            ]
        );
        if value.user_changed {
            assert!(!f.trace.contains(&A::ReadInputUserChangedDataWhileFocused {
                data: Data::SelectedFiles
            }));
        }
        let change_at = f
            .trace
            .iter()
            .position(|a| matches!(a, A::FireChange { .. }))
            .unwrap();
        let classify_at = f
            .trace
            .iter()
            .position(|a| *a == A::ReadEntryKind { chain: Chain::Old })
            .unwrap();
        assert!(change_at < classify_at);
    }
}

#[test]
fn each_failed_value_predicate_still_tries_files_without_reading_later_value_predicates() {
    for value in [
        Change::default(),
        Change {
            uncommitted: false,
            ..DIRTY
        },
        Change {
            differs: false,
            ..DIRTY
        },
    ] {
        let mut f = input_fixture(Input {
            value,
            files: DIRTY,
            ..Input::default()
        });
        f.run();
        assert!(f.validity.contains(&1));
        assert!(f.trace.contains(&A::ReadInputUserChangedDataWhileFocused {
            data: Data::SelectedFiles
        }));
        if !value.user_changed {
            assert!(
                !f.trace
                    .contains(&A::ReadInputChangeUncommitted { data: Data::Value })
            );
            assert!(
                !f.trace
                    .contains(&A::ReadInputDiffersFromFirstFocus { data: Data::Value })
            );
        } else if !value.uncommitted {
            assert!(
                !f.trace
                    .contains(&A::ReadInputDiffersFromFirstFocus { data: Data::Value })
            );
        }
    }
}

#[test]
fn earlier_callbacks_change_later_document_and_view_captures_without_changing_historical_chain_ids()
{
    let mut f = Fixture::new(&[1], &[2]);
    f.input(
        1,
        10,
        Input {
            value: DIRTY,
            ..Input::default()
        },
    );
    f.element(2, 10, true);
    f.change_adoption = Some((1, 20));
    f.blur_adoption = Some((2, 30));
    f.run();
    assert_eq!(
        f.events(),
        vec![
            Event {
                name: "change",
                target: Target::Element(1),
                related: None,
                view: None,
                interface: None,
                bubbles: true,
                cancelable: false,
                composed: false
            },
            focus("blur", Target::Element(1), Some(2), 102),
            focus("focus", Target::Element(2), Some(1), 103),
        ]
    );
    assert_eq!(f.designation[&10], None);
    assert_eq!(f.designation[&20], None);
    assert_eq!(f.designation[&30], Some(2));
    assert!(f.effects.ends_with(&[
        Effect::Platform,
        Effect::CaptureDocument(2, 30),
        Effect::NavigationTrue(30, 103),
        Effect::Designation(30, 2),
        Effect::Event(focus("focus", Target::Element(2), Some(1), 103)),
    ]));
    assert_eq!(f.old, vec![1]);
    assert_eq!(f.new, vec![2]);
}
