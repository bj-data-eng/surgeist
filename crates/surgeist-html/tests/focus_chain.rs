//! HTML877d raw59761–59808 literal chain expectations, including the selected
//! nullable-current-area and child-navigable-container interpretations. This is
//! a small identity/list adapter for pure protocols, not a native focus service.
use surgeist_html::{
    HtmlCurrentFocusChainAction as CA, HtmlCurrentFocusChainCursor as CC,
    HtmlCurrentFocusChainError as CE, HtmlCurrentFocusChainReply as CR,
    HtmlCurrentlyFocusedAreaAction as QA, HtmlCurrentlyFocusedAreaCursor as QC,
    HtmlCurrentlyFocusedAreaOutcome as QO, HtmlCurrentlyFocusedAreaReply as QR,
    HtmlFocusChainAction as A, HtmlFocusChainCursor as C, HtmlFocusChainError as E,
    HtmlFocusChainReply as R, HtmlFocusedAreaKind,
};

type Id = usize;
const TOP: Id = 0;
const CHILD: Id = 1;
const GRAND: Id = 2;
const FRAME: Id = 3;
const INNER: Id = 4;
const INPUT: Id = 5;
const VIEWPORT: Id = 6;
const SHAPE: Id = 7;
const AREA: Id = 8;
const IMAGE: Id = 9;
const WIDGET: Id = 10;
const ORPHAN: Id = 11;
const LOOSE: Id = 12;

#[derive(Clone, Copy)]
enum Fact<T> {
    Known(T),
    Missing,
}
impl<T: Copy> Fact<T> {
    fn get(self) -> Result<T, Wait> {
        match self {
            Self::Known(value) => Ok(value),
            Self::Missing => Err(Wait::Unavailable),
        }
    }
}
#[derive(Clone, Copy)]
enum Kind {
    Document,
    Element(bool),
    Area { anchor: Option<Id> },
}
#[derive(Clone, Copy)]
struct Object {
    kind: Kind,
    document: Id,
    shape_element: Option<Id>,
    navigable: Fact<Option<usize>>,
    content: Option<Fact<Option<usize>>>,
    designation: Fact<Option<Id>>,
}
struct Navigable {
    document: Id,
    parent: Fact<Option<usize>>,
    container: Fact<Option<Id>>,
}
struct Fixture {
    objects: Vec<Object>,
    navigables: Vec<Navigable>,
    basis: u64,
}
#[derive(Debug, Eq, PartialEq)]
enum Wait {
    Unavailable,
    Inconsistent,
    BasisMismatch,
    OutputLimit,
    WorkLimit,
}
impl Fixture {
    fn object(&self, id: Id) -> Result<&Object, Wait> {
        self.objects.get(id).ok_or(Wait::Unavailable)
    }
    fn navigable(&self, index: usize) -> Result<&Navigable, Wait> {
        self.navigables.get(index).ok_or(Wait::Unavailable)
    }
    fn document(&self, id: Id) -> Result<Id, Wait> {
        let document = self.object(id)?.document;
        if !matches!(self.object(document)?.kind, Kind::Document) {
            return Err(Wait::Inconsistent);
        }
        Ok(document)
    }
}
fn fixture() -> Fixture {
    let doc = |id, navigable| Object {
        kind: Kind::Document,
        document: id,
        shape_element: None,
        navigable: Fact::Known(navigable),
        content: None,
        designation: Fact::Known(None),
    };
    let element = |document| Object {
        kind: Kind::Element(true),
        document,
        shape_element: None,
        navigable: Fact::Missing,
        content: None,
        designation: Fact::Missing,
    };
    let mut objects = vec![
        doc(TOP, Some(0)),
        doc(CHILD, Some(1)),
        doc(GRAND, Some(2)),
        element(TOP),
        element(CHILD),
        element(GRAND),
        Object {
            kind: Kind::Area {
                anchor: Some(CHILD),
            },
            document: CHILD,
            shape_element: None,
            navigable: Fact::Missing,
            content: None,
            designation: Fact::Missing,
        },
        Object {
            kind: Kind::Area {
                anchor: Some(IMAGE),
            },
            document: CHILD,
            shape_element: Some(AREA),
            navigable: Fact::Missing,
            content: None,
            designation: Fact::Missing,
        },
        element(CHILD),
        element(CHILD),
        Object {
            kind: Kind::Area {
                anchor: Some(IMAGE),
            },
            document: CHILD,
            shape_element: None,
            navigable: Fact::Missing,
            content: None,
            designation: Fact::Missing,
        },
        doc(ORPHAN, None),
        element(ORPHAN),
    ];
    objects[FRAME].content = Some(Fact::Known(Some(1)));
    objects[INNER].content = Some(Fact::Known(Some(2)));
    objects[TOP].designation = Fact::Known(Some(FRAME));
    objects[CHILD].designation = Fact::Known(Some(INNER));
    objects[GRAND].designation = Fact::Known(Some(INPUT));
    Fixture {
        objects,
        navigables: vec![
            Navigable {
                document: TOP,
                parent: Fact::Known(None),
                container: Fact::Known(None),
            },
            Navigable {
                document: CHILD,
                parent: Fact::Known(Some(0)),
                container: Fact::Known(Some(FRAME)),
            },
            Navigable {
                document: GRAND,
                parent: Fact::Known(Some(1)),
                container: Fact::Known(Some(INNER)),
            },
        ],
        basis: 7,
    }
}

const REPLIES: [R; 13] = [
    R::AreaShape(false),
    R::AreaShape(true),
    R::AnchorIsDifferentElement(false),
    R::AnchorIsDifferentElement(true),
    R::FocusableArea(false),
    R::FocusableArea(true),
    R::Document(false),
    R::Document(true),
    R::NodeNavigablePresent(false),
    R::NodeNavigablePresent(true),
    R::NavigableParentPresent(false),
    R::NavigableParentPresent(true),
    R::Applied,
];
fn step(cursor: &mut C, reply: R) {
    let action = cursor.action();
    for wrong in REPLIES {
        if std::mem::discriminant(&wrong) != std::mem::discriminant(&reply) {
            assert_eq!(cursor.reply(wrong), Err(E::WrongReply));
            assert_eq!(cursor.action(), action);
        }
    }
    cursor.reply(reply).unwrap();
}
fn terminal(cursor: &mut C) {
    assert_eq!(cursor.action(), A::Complete);
    for reply in REPLIES {
        assert_eq!(cursor.reply(reply), Err(E::Terminal));
        assert_eq!(cursor.action(), A::Complete);
    }
}
struct Chain {
    cursor: C,
    subject: Id,
    current: Option<Id>,
    navigable: Option<usize>,
    output: Vec<Id>,
    anchor_queries: Vec<Id>,
    work: usize,
    basis: u64,
}
impl Chain {
    fn new(subject: Id, f: &Fixture) -> Self {
        Self {
            cursor: C::new(),
            subject,
            current: None,
            navigable: None,
            output: Vec::new(),
            anchor_queries: Vec::new(),
            work: 0,
            basis: f.basis,
        }
    }
    fn anchor(&mut self, f: &Fixture) -> Result<Id, Wait> {
        let current = self.current.unwrap();
        let object = f.object(current)?;
        let anchor = match object.kind {
            Kind::Document => panic!("non-area Documents have no DOM-anchor query or storage"),
            Kind::Element(true) => Some(current), // An actual Node area is its own anchor.
            Kind::Element(false) => None,
            Kind::Area { anchor } => anchor,
        };
        self.anchor_queries.push(current);
        anchor.ok_or(Wait::Unavailable)
    }
    fn append(&mut self, id: Id, capacity: usize) -> Result<(), Wait> {
        if self.output.len() == capacity {
            return Err(Wait::OutputLimit);
        }
        self.output.push(id);
        Ok(())
    }
    fn drive(&mut self, f: &Fixture, capacity: usize, work_limit: usize) -> Result<(), Wait> {
        if f.basis != self.basis {
            return Err(Wait::BasisMismatch);
        }
        loop {
            if self.cursor.action() == A::Complete {
                terminal(&mut self.cursor);
                return Ok(());
            }
            if self.work == work_limit {
                return Err(Wait::WorkLimit);
            }
            self.work += 1;
            let reply = match self.cursor.action() {
                A::InitializeEmptyChain => {
                    self.output.clear();
                    R::Applied
                }
                A::CaptureSubject => {
                    f.object(self.subject)?;
                    self.current = Some(self.subject);
                    R::Applied
                }
                A::AppendCurrentObject => {
                    self.append(self.current.unwrap(), capacity)?;
                    R::Applied
                }
                A::ReadAreaShape => {
                    R::AreaShape(f.object(self.current.unwrap())?.shape_element.is_some())
                }
                A::AppendCorrespondingAreaElement => {
                    self.append(
                        f.object(self.current.unwrap())?.shape_element.unwrap(),
                        capacity,
                    )?;
                    R::Applied
                }
                A::ReadAnchorIsDifferentElement => {
                    let current = self.current.unwrap();
                    let anchor = self.anchor(f)?;
                    R::AnchorIsDifferentElement(
                        matches!(f.object(anchor)?.kind, Kind::Element(_)) && anchor != current,
                    )
                }
                A::AppendAnchorElement => {
                    let anchor = self.anchor(f)?;
                    self.append(anchor, capacity)?;
                    R::Applied
                }
                A::ReadFocusableArea => R::FocusableArea(matches!(
                    f.object(self.current.unwrap())?.kind,
                    Kind::Area { .. } | Kind::Element(true)
                )),
                A::CaptureAnchorNodeDocument => {
                    self.current = Some(f.document(self.anchor(f)?)?);
                    R::Applied
                }
                A::ReadDocument => R::Document(matches!(
                    f.object(self.current.unwrap())?.kind,
                    Kind::Document
                )),
                A::ReadNodeNavigable => {
                    let current = self.current.unwrap();
                    self.navigable = f.object(current)?.navigable.get()?;
                    if let Some(index) = self.navigable
                        && f.navigable(index)?.document != current
                    {
                        return Err(Wait::Inconsistent);
                    }
                    R::NodeNavigablePresent(self.navigable.is_some())
                }
                A::ReadNavigableParent => R::NavigableParentPresent(
                    f.navigable(self.navigable.unwrap())?
                        .parent
                        .get()?
                        .is_some(),
                ),
                A::CaptureChildNavigableContainerElement => {
                    let child_index = self.navigable.unwrap();
                    let child = f.navigable(child_index)?;
                    let parent_index = child.parent.get()?.ok_or(Wait::Inconsistent)?;
                    let parent = f.navigable(parent_index)?;
                    if f.object(parent.document)?.navigable.get()? != Some(parent_index) {
                        return Err(Wait::Inconsistent);
                    }
                    let container = child.container.get()?.ok_or(Wait::Inconsistent)?;
                    let element = f.object(container)?;
                    if !matches!(element.kind, Kind::Element(_))
                        || element.content.ok_or(Wait::Inconsistent)?.get()? != Some(child_index)
                        || f.document(container)? != parent.document
                    {
                        return Err(Wait::Inconsistent);
                    }
                    self.current = Some(container);
                    R::Applied
                }
                A::Complete => unreachable!(),
            };
            step(&mut self.cursor, reply);
        }
    }
}
fn chain(subject: Id, f: &Fixture) -> Vec<Id> {
    let mut c = Chain::new(subject, f);
    c.drive(f, 32, 512).unwrap();
    c.output
}

// Actual existing current-area child, driven by this same qualified graph.
fn current_area(f: &Fixture, traversable: usize, system_focus: bool) -> Result<Option<Id>, Wait> {
    let mut query = QC::new();
    let mut document = None;
    let mut loop_area = None;
    let mut content = None;
    let mut result = None;
    for _ in 0..128 {
        let reply = match query.action() {
            QA::ReadSystemFocus => QR::SystemFocus(system_focus),
            QA::CaptureTraversableActiveDocument => {
                document = Some(f.navigable(traversable)?.document);
                QR::Applied
            }
            QA::ReadLoopFocusedArea => {
                loop_area = f.object(document.unwrap())?.designation.get()?;
                QR::LoopFocusedArea(match loop_area {
                    None => HtmlFocusedAreaKind::Null,
                    Some(area) if f.object(area)?.content.is_some() => {
                        HtmlFocusedAreaKind::NavigableContainer
                    }
                    Some(_) => HtmlFocusedAreaKind::OtherArea,
                })
            }
            QA::ReadContentNavigable => {
                content = f
                    .object(loop_area.unwrap())?
                    .content
                    .ok_or(Wait::Inconsistent)?
                    .get()?;
                QR::ContentNavigablePresent(content.is_some())
            }
            QA::CaptureContentActiveDocument => {
                document = Some(f.navigable(content.unwrap())?.document);
                QR::Applied
            }
            QA::ReadResultFocusedArea => {
                result = f.object(document.unwrap())?.designation.get()?;
                QR::ResultFocusedAreaPresent(result.is_some())
            }
            QA::Complete(outcome) => {
                return Ok(match outcome {
                    QO::Null => None,
                    QO::CandidateDocument => document,
                    QO::CandidateFocusedArea => result,
                });
            }
        };
        query.reply(reply).unwrap();
    }
    Err(Wait::WorkLimit)
}
fn current_chain(
    f: &Fixture,
    traversable: Fact<Option<usize>>,
    system_focus: bool,
) -> Result<Vec<Id>, Wait> {
    let mut cursor = CC::new();
    let mut area = None;
    let mut output = vec![LOOSE]; // Empty initialization must actually replace output.
    for _ in 0..16 {
        let reply = match cursor.action() {
            CA::ReadTraversable => CR::TraversablePresent(traversable.get()?.is_some()),
            CA::RunCurrentlyFocusedArea => {
                area = current_area(f, traversable.get()?.unwrap(), system_focus)?;
                CR::CurrentlyFocusedAreaPresent(area.is_some())
            }
            CA::InitializeEmptyChain => {
                output.clear();
                CR::Applied
            }
            CA::RunFocusChain => {
                output = chain(area.unwrap(), f);
                CR::FocusChainReady
            }
            CA::Complete => return Ok(output),
        };
        cursor.reply(reply).unwrap();
    }
    Err(Wait::WorkLimit)
}

#[test]
fn nested_current_area_builds_child_document_container_parent_chain() {
    let f = fixture();
    assert_eq!(
        current_chain(&f, Fact::Known(Some(0)), true).unwrap(),
        vec![INPUT, GRAND, INNER, CHILD, FRAME, TOP]
    );
}
#[test]
fn shape_appends_corresponding_area_element_instead_of_image_anchor() {
    let f = fixture();
    assert_eq!(chain(SHAPE, &f), vec![SHAPE, AREA, CHILD, FRAME, TOP]);
    assert_eq!(chain(WIDGET, &f), vec![WIDGET, IMAGE, CHILD, FRAME, TOP]);
}
#[test]
fn viewport_document_and_element_identities_remain_distinct_without_duplicate_anchor() {
    let f = fixture();
    assert_eq!(chain(VIEWPORT, &f), vec![VIEWPORT, CHILD, FRAME, TOP]);
    assert_eq!(chain(CHILD, &f), vec![CHILD, FRAME, TOP]);
    assert_eq!(chain(FRAME, &f), vec![FRAME, TOP]);
    assert_eq!(
        chain(INPUT, &f),
        vec![INPUT, GRAND, INNER, CHILD, FRAME, TOP]
    );
}
#[test]
fn actual_absent_navigable_and_top_parent_stop() {
    let f = fixture();
    assert_eq!(chain(ORPHAN, &f), vec![ORPHAN]);
    assert_eq!(chain(LOOSE, &f), vec![LOOSE, ORPHAN]);
    assert_eq!(chain(TOP, &f), vec![TOP]);
}
#[test]
fn absent_traversable_or_actual_null_current_area_initializes_empty_chain() {
    let f = fixture();
    assert_eq!(current_chain(&f, Fact::Known(None), true).unwrap(), vec![]);
    assert_eq!(
        current_chain(&f, Fact::Known(Some(0)), false).unwrap(),
        vec![]
    );
    assert_eq!(
        current_chain(&f, Fact::Missing, true),
        Err(Wait::Unavailable)
    );
}
#[test]
fn null_document_designation_returns_candidate_document_instead_of_null_current_area() {
    let mut f = fixture();
    f.objects[GRAND].designation = Fact::Known(None);
    assert_eq!(
        current_chain(&f, Fact::Known(Some(0)), true).unwrap(),
        vec![GRAND, INNER, CHILD, FRAME, TOP]
    );
}
#[test]
fn missing_parent_fact_stays_pending_instead_of_becoming_top_level() {
    let mut f = fixture();
    f.navigables[1].parent = Fact::Missing;
    let mut c = Chain::new(CHILD, &f);
    assert_eq!(c.drive(&f, 32, 512), Err(Wait::Unavailable));
    assert_eq!(c.cursor.action(), A::ReadNavigableParent);
    assert_eq!(c.output, vec![CHILD]);
    f.navigables[1].parent = Fact::Known(Some(0));
    c.drive(&f, 32, 512).unwrap();
    assert_eq!(c.output, vec![CHILD, FRAME, TOP]);
}
#[test]
fn missing_or_inconsistent_child_container_never_shortens_successful_chain() {
    for container in [Fact::Missing, Fact::Known(None), Fact::Known(Some(INNER))] {
        let mut f = fixture();
        f.navigables[1].container = container;
        let mut c = Chain::new(CHILD, &f);
        assert!(matches!(
            c.drive(&f, 32, 512),
            Err(Wait::Unavailable | Wait::Inconsistent)
        ));
        assert_eq!(c.cursor.action(), A::CaptureChildNavigableContainerElement);
        assert_eq!(c.output, vec![CHILD]);
        f.navigables[1].container = Fact::Known(Some(FRAME));
        c.drive(&f, 32, 512).unwrap();
        assert_eq!(c.output, vec![CHILD, FRAME, TOP]);
    }
}
#[test]
fn unavailable_graph_and_mismatched_basis_preserve_exact_pending_continuation() {
    let mut f = fixture();
    f.objects[CHILD].navigable = Fact::Missing;
    let mut c = Chain::new(CHILD, &f);
    assert_eq!(c.drive(&f, 32, 512), Err(Wait::Unavailable));
    assert_eq!(c.cursor.action(), A::ReadNodeNavigable);
    assert_eq!(c.output, vec![CHILD]);
    f.objects[CHILD].navigable = Fact::Known(Some(0));
    assert_eq!(c.drive(&f, 32, 512), Err(Wait::Inconsistent));
    assert_eq!(c.cursor.action(), A::ReadNodeNavigable);
    f.objects[CHILD].navigable = Fact::Known(Some(1));
    f.basis += 1;
    assert_eq!(c.drive(&f, 32, 512), Err(Wait::BasisMismatch));
    assert_eq!(c.cursor.action(), A::ReadNodeNavigable);
    f.basis -= 1;
    c.drive(&f, 32, 512).unwrap();
    assert_eq!(c.output, vec![CHILD, FRAME, TOP]);
}
#[test]
fn refused_append_and_work_boundary_retry_without_losing_or_duplicating_entries() {
    let f = fixture();
    let mut c = Chain::new(SHAPE, &f);
    assert_eq!(c.drive(&f, 1, 512), Err(Wait::OutputLimit));
    assert_eq!(c.cursor.action(), A::AppendCorrespondingAreaElement);
    assert_eq!(c.output, vec![SHAPE]);
    let spent = c.work;
    assert_eq!(c.drive(&f, 32, spent), Err(Wait::WorkLimit));
    assert_eq!(c.cursor.action(), A::AppendCorrespondingAreaElement);
    c.drive(&f, 32, 512).unwrap();
    assert_eq!(c.output, vec![SHAPE, AREA, CHILD, FRAME, TOP]);
}
#[test]
fn current_chain_wrong_stage_and_terminal_replies_preserve_exact_state() {
    let replies = [
        CR::TraversablePresent(false),
        CR::TraversablePresent(true),
        CR::CurrentlyFocusedAreaPresent(false),
        CR::CurrentlyFocusedAreaPresent(true),
        CR::FocusChainReady,
        CR::Applied,
    ];
    for route in [0, 1, 2] {
        let mut c = CC::new();
        let trace = match route {
            0 => vec![CR::TraversablePresent(false), CR::Applied],
            1 => vec![
                CR::TraversablePresent(true),
                CR::CurrentlyFocusedAreaPresent(false),
                CR::Applied,
            ],
            _ => vec![
                CR::TraversablePresent(true),
                CR::CurrentlyFocusedAreaPresent(true),
                CR::FocusChainReady,
            ],
        };
        for actual in trace {
            let before = c.action();
            for wrong in replies {
                if std::mem::discriminant(&wrong) != std::mem::discriminant(&actual) {
                    assert_eq!(c.reply(wrong), Err(CE::WrongReply));
                    assert_eq!(c.action(), before);
                }
            }
            c.reply(actual).unwrap();
        }
        assert_eq!(c.action(), CA::Complete);
        for reply in replies {
            assert_eq!(c.reply(reply), Err(CE::Terminal));
            assert_eq!(c.action(), CA::Complete);
        }
    }
}

#[test]
fn document_entries_never_query_anchors_while_viewport_uses_its_actual_document_anchor() {
    let f = fixture();
    for (subject, expected) in [
        (CHILD, vec![CHILD, FRAME, TOP]),
        (VIEWPORT, vec![VIEWPORT, CHILD, FRAME, TOP]),
    ] {
        let mut c = Chain::new(subject, &f);
        c.drive(&f, 32, 512).unwrap();
        assert_eq!(c.output, expected);
        assert!(!c.anchor_queries.contains(&CHILD));
        assert!(!c.anchor_queries.contains(&TOP));
        assert!(c.anchor_queries.contains(&FRAME));
        assert_eq!(c.anchor_queries.contains(&VIEWPORT), subject == VIEWPORT);
    }
}
#[test]
fn actual_area_missing_anchor_remains_pending_without_document_substitution() {
    let mut f = fixture();
    f.objects[VIEWPORT].kind = Kind::Area { anchor: None };
    let mut c = Chain::new(VIEWPORT, &f);
    assert_eq!(c.drive(&f, 32, 512), Err(Wait::Unavailable));
    assert_eq!(c.cursor.action(), A::ReadAnchorIsDifferentElement);
    assert_eq!(c.output, vec![VIEWPORT]);
    f.objects[VIEWPORT].kind = Kind::Area {
        anchor: Some(CHILD),
    };
    c.drive(&f, 32, 512).unwrap();
    assert_eq!(c.output, vec![VIEWPORT, CHILD, FRAME, TOP]);
}
#[test]
fn non_area_non_document_classification_is_typed_error_and_preserves_pending_cursor() {
    // A defensive inconsistent-classification test, not an admitted arbitrary
    // Node fixture: callers qualify the area-or-Document input before effects.
    let mut c = C::new();
    for reply in [
        R::Applied,
        R::Applied,
        R::Applied,
        R::AreaShape(false),
        R::FocusableArea(false),
    ] {
        step(&mut c, reply);
    }
    assert_eq!(c.action(), A::ReadDocument);
    assert_eq!(c.reply(R::Document(false)), Err(E::InvalidCurrentObject));
    assert_eq!(c.action(), A::ReadDocument);
    // The refused reply has not consumed the pending fact channel.
    step(&mut c, R::Document(true));
    step(&mut c, R::NodeNavigablePresent(false));
    terminal(&mut c);
}
