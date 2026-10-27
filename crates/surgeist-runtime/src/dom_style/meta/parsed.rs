//! Whole original parsed Meta contributions through the sole preference owner.
use super::*;

/// A whole parsed Meta operation or its actual producer refused.
#[derive(Debug)]
pub enum DomStyleParsedMetaError {
    /// The actual Runtime owner, phase or finite admission refused.
    Runtime(DomStyleRuntimeError),
    /// Original parsed association qualification or producer preparation refused.
    Meta(surgeist_dom::DomParsedHtmlMetaError),
}
impl std::fmt::Display for DomStyleParsedMetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(f, "parsed association Runtime refused: {error}"),
            Self::Meta(error) => write!(f, "parsed association execution refused: {error}"),
        }
    }
}
impl std::error::Error for DomStyleParsedMetaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Meta(error) => Some(error),
        }
    }
}
impl From<DomStyleRuntimeError> for DomStyleParsedMetaError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Runtime(error)
    }
}
impl From<surgeist_dom::DomParsedHtmlMetaError> for DomStyleParsedMetaError {
    fn from(error: surgeist_dom::DomParsedHtmlMetaError) -> Self {
        Self::Meta(error)
    }
}
impl From<surgeist_dom::DomParsedHtmlAssociationAdmissionError> for DomStyleParsedMetaError {
    fn from(error: surgeist_dom::DomParsedHtmlAssociationAdmissionError) -> Self {
        Self::Meta(error.into())
    }
}
/// Pure Session intake refused before any CSSOM owner is accessed.
#[derive(Debug)]
pub enum DomStyleParsedAdmissionError {
    /// Runtime owner setup or finite discovery refused.
    Runtime(DomStyleRuntimeError),
    /// Original whole-contribution classification or admission refused.
    Association(surgeist_dom::DomParsedHtmlAssociationAdmissionError),
}
impl std::fmt::Display for DomStyleParsedAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(f, "parsed Session intake refused: {error}"),
            Self::Association(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleParsedAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Association(error) => Some(error),
        }
    }
}
impl From<DomStyleRuntimeError> for DomStyleParsedAdmissionError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Runtime(error)
    }
}
impl From<surgeist_dom::DomParsedHtmlAssociationAdmissionError> for DomStyleParsedAdmissionError {
    fn from(error: surgeist_dom::DomParsedHtmlAssociationAdmissionError) -> Self {
        Self::Association(error)
    }
}
/// Structural installation refused without processing association effects.
#[derive(Debug)]
pub enum DomStyleParsedInstallError {
    /// The actual Session's phase or finite admission refused.
    Runtime(DomStyleRuntimeError),
    /// The immutable report or destination structural owner refused.
    Installation(surgeist_dom::DomParsedHtmlInstallError),
}
impl std::fmt::Display for DomStyleParsedInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(f, "parsed Session installation refused: {error}"),
            Self::Installation(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleParsedInstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Installation(error) => Some(error),
        }
    }
}
impl From<DomStyleRuntimeError> for DomStyleParsedInstallError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Runtime(error)
    }
}
impl From<surgeist_dom::DomParsedHtmlInstallError> for DomStyleParsedInstallError {
    fn from(error: surgeist_dom::DomParsedHtmlInstallError) -> Self {
        Self::Installation(error)
    }
}

/// Exact original occurrence and its actual destination preference transition.
#[derive(Debug)]
pub struct DomStyleParsedMetaPreference {
    id: surgeist_html_parser::HtmlParseAssociationOccurrenceId,
    subject: surgeist_html::HtmlNodeId,
    observation: DomStyleSheetSetObservation,
    root_commit_index: usize,
}
impl DomStyleParsedMetaPreference {
    /// Immutable original-controller occurrence, not a native invocation.
    pub fn id(&self) -> &surgeist_html_parser::HtmlParseAssociationOccurrenceId {
        &self.id
    }
    /// Original historical subject; it need not have a surviving live NodeId.
    pub fn subject(&self) -> &surgeist_html::HtmlNodeId {
        &self.subject
    }
    /// Actual ordered name transition with the final real producer basis.
    pub fn observation(&self) -> &DomStyleSheetSetObservation {
        &self.observation
    }
}
/// Authentic whole-contribution acknowledgement and producer/name evidence.
#[must_use]
pub struct DomStyleParsedMetaOutcome {
    completion: surgeist_dom::DomParsedHtmlMetaCompletion,
    preferences: Vec<DomStyleParsedMetaPreference>,
}
impl std::fmt::Debug for DomStyleParsedMetaOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStyleParsedMetaOutcome")
            .field("contribution", &self.completion.contribution())
            .field("stamp", &self.completion.stamp())
            .field("root_count", &self.completion.roots().len())
            .field("preferences", &self.preferences)
            .finish()
    }
}
impl DomStyleParsedMetaOutcome {
    /// Actual canonical acknowledgement, not a structural-installation receipt.
    pub fn completion(&self) -> &surgeist_dom::DomParsedHtmlMetaCompletion {
        &self.completion
    }
    /// Every original preferred-name operation in causal order, including no-ops.
    pub fn preferences(&self) -> &[DomStyleParsedMetaPreference] {
        &self.preferences
    }
    /// Move the authentic outputs once; consuming the same contribution again refuses.
    pub fn into_parts(
        self,
    ) -> (
        surgeist_dom::DomParsedHtmlMetaCompletion,
        Vec<DomStyleParsedMetaPreference>,
    ) {
        (self.completion, self.preferences)
    }
}

struct ParsedSteps<'a> {
    work: &'a Cell<u64>,
    maximum: u64,
    cancel: &'a AtomicBool,
}
impl ParsedSteps<'_> {
    fn runtime(&self) -> Result<(), DomStyleRuntimeError> {
        StyleState::step_counter(self.work, self.cancel, self.maximum)
    }
    fn prefix(
        &self,
        hold: &surgeist_dom::DomParsedHtmlMetaRuntimeHold<'_>,
    ) -> Result<(), DomStyleRuntimeError> {
        hold.step(self.cancel)?;
        self.runtime()
    }
    fn joined(
        &self,
        preparation: &surgeist_dom::DomParsedHtmlMetaPreparation<'_, '_, '_>,
    ) -> Result<(), DomStyleRuntimeError> {
        preparation.step(self.cancel)?;
        self.runtime()
    }
    fn same(
        &self,
        preparation: &surgeist_dom::DomParsedHtmlMetaPreparation<'_, '_, '_>,
        left: &[u16],
        right: &[u16],
    ) -> Result<bool, DomStyleRuntimeError> {
        self.joined(preparation)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for (a, b) in left.iter().zip(right) {
            self.joined(preparation)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn storage(
    state: &StyleState,
    hold: &mut surgeist_dom::DomParsedHtmlMetaRuntimeHold<'_>,
    backing: &Backing,
    steps: &ParsedSteps<'_>,
) -> Result<usize, DomStyleRuntimeError> {
    let headers = state.storage_headers(None, || steps.prefix(hold))?;
    let remaining = steps.maximum.saturating_sub(steps.work.get());
    let mut fault = None;
    let mut admit = |action| {
        if fault.is_some() {
            return false;
        }
        let result = match action {
            surgeist_css::source::CssSourceRetentionAction::Step => steps.prefix(hold),
            surgeist_css::source::CssSourceRetentionAction::Scratch { live_bytes } => (|| {
                steps.prefix(hold)?;
                let bytes = backing
                    .bytes
                    .checked_add(live_bytes)
                    .ok_or(DomStyleRuntimeError::Admission)?;
                hold.admit_runtime(
                    surgeist_dom::DomParsedHtmlAssociationRuntimeUsage::new(
                        bytes,
                        backing.units,
                        backing.outputs,
                    )?,
                    steps.cancel,
                )?;
                Ok(())
            })(
            ),
        };
        match result {
            Ok(()) => true,
            Err(error) => {
                if fault.is_none() {
                    fault = Some(error);
                }
                false
            }
        }
    };
    let measured = (|| {
        let mut sources = CssSourceRetention::with_admission(
            usize::MAX,
            usize::try_from(remaining).unwrap_or(usize::MAX),
            usize::try_from(remaining).unwrap_or(usize::MAX),
            &mut admit,
        )
        .map_err(|_| DomStyleRuntimeError::Admission)?;
        state.processing(None, |value| {
            sources
                .charge_step()
                .map_err(|_| DomStyleRuntimeError::Admission)?;
            for diagnostic in value.diagnostics() {
                sources
                    .visit(diagnostic)
                    .map_err(|_| DomStyleRuntimeError::Admission)?;
            }
            Ok(())
        })?;
        sources
            .complete_payload_bytes_admitted()
            .map_err(|_| DomStyleRuntimeError::Admission)
    })();
    // The collector and its bank are already destroyed before the typed caller
    // refusal is recovered or its external hold charge can be released.
    if let Some(error) = fault {
        return Err(error);
    }
    headers
        .checked_add(measured?)
        .ok_or(DomStyleRuntimeError::Admission)
}
impl Backing {
    fn parsed_usage(
        &self,
    ) -> Result<surgeist_dom::DomParsedHtmlAssociationRuntimeUsage, DomStyleRuntimeError> {
        Ok(surgeist_dom::DomParsedHtmlAssociationRuntimeUsage::new(
            self.bytes - self.dom_shared_bytes,
            self.units - self.dom_shared_units,
            self.outputs,
        )?)
    }
    fn reserve_initial<T>(
        &mut self,
        values: &mut Vec<T>,
        count: usize,
        hold: &mut surgeist_dom::DomParsedHtmlMetaRuntimeHold<'_>,
        steps: &ParsedSteps<'_>,
    ) -> Result<(), DomStyleRuntimeError> {
        let requested = count
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(requested)?;
        steps.prefix(hold)?;
        hold.admit_runtime(self.parsed_usage()?, steps.cancel)?;
        values
            .try_reserve_exact(count)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(
            actual
                .checked_sub(requested)
                .ok_or(DomStyleRuntimeError::Admission)?,
        )?;
        steps.prefix(hold)?;
        hold.admit_runtime(self.parsed_usage()?, steps.cancel)?;
        Ok(())
    }
    fn reserve_parsed<T>(
        &mut self,
        values: &mut Vec<T>,
        count: usize,
        preparation: &mut surgeist_dom::DomParsedHtmlMetaPreparation<'_, '_, '_>,
        steps: &ParsedSteps<'_>,
    ) -> Result<(), DomStyleRuntimeError> {
        let requested = count
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(requested)?;
        steps.joined(preparation)?;
        preparation.admit_runtime(self.parsed_usage()?, steps.cancel)?;
        values
            .try_reserve_exact(count)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(
            actual
                .checked_sub(requested)
                .ok_or(DomStyleRuntimeError::Admission)?,
        )?;
        steps.joined(preparation)?;
        preparation.admit_runtime(self.parsed_usage()?, steps.cancel)?;
        Ok(())
    }
}
impl DomStyleRuntime {
    /// Consume a complete supported original contribution once, without native
    /// replay. Influencing Style histories remain pending. Refused attempts keep
    /// original contribution work/policy; names, flags and acknowledgement publish
    /// together only after every fallible preparation has succeeded.
    #[expect(
        clippy::result_large_err,
        reason = "Returns the actual root-qualified producer failure without allocation or loss of its original diagnostic"
    )]
    pub fn consume_parsed_meta(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: surgeist_dom::DomParsedHtmlAssociationId,
        limits: surgeist_dom::DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomStyleParsedMetaOutcome, DomStyleParsedMetaError> {
        let owner = self.original()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        if state.dom != dom.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        let view = state.require_sheet_set_idle(dom)?;
        let stamp = view.stamp();
        drop(view);
        // Declaration order is custody order: every represented bank is formed
        // after this affine hold and is destroyed before its charge on failure.
        let mut hold = dom
            .raw_mut()
            .begin_parsed_html_meta(contribution, limits, cancel)?;
        let steps = ParsedSteps {
            work: &state.work,
            maximum: state.limits.work,
            cancel,
        };
        let mut backing = Backing {
            bytes: size_of::<DomStyleParsedMetaOutcome>()
                + size_of::<Vec<Rc<DomStyleCssomRoot>>>()
                + size_of::<Vec<&DomStyleCssomRoot>>()
                + size_of::<Vec<Candidate>>()
                + size_of::<Vec<NameRow<'_>>>()
                + size_of::<Vec<sheet_sets::State>>()
                + size_of::<sheet_sets::State>()
                + size_of::<DocumentString>()
                + size_of::<DomStyleParsedMetaPreference>(),
            units: 0,
            dom_shared_bytes: 0,
            dom_shared_units: 0,
            outputs: 0,
            current: 0,
            runtime_limit: state.limits.storage_bytes,
            limits: BackingLimits::parsed(limits),
        };
        steps.prefix(&hold)?;
        hold.admit_runtime(backing.parsed_usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        // Persistent Runtime storage excludes this temporary collector, as in
        // the existing owner contract. Its actual scratch is additionally
        // prepaid by the original contribution before allocation or traversal.
        backing.current = storage(&state, &mut hold, &backing, &steps)?;
        backing.check()?;
        let mut capabilities = Vec::new();
        let mut references = Vec::new();
        backing.reserve_initial(&mut capabilities, state.roots.len(), &mut hold, &steps)?;
        backing.reserve_initial(&mut references, state.roots.len(), &mut hold, &steps)?;
        for root in &state.roots {
            steps.prefix(&hold)?;
            if let RootResource::Bound(owner) = &root.resource {
                capabilities.push(owner.clone());
            }
        }
        for capability in &capabilities {
            steps.prefix(&hold)?;
            references.push(capability.as_ref());
        }
        steps.prefix(&hold)?;
        let mut preparation = hold.prepare(&references, cancel).map_err(|error| {
            if let surgeist_dom::DomParsedHtmlMetaError::MissingOwner { root } = &error
                && let Err(reason) = state.require_bound_if_registered(*root, cancel)
            {
                return DomStyleParsedMetaError::Runtime(reason);
            }
            DomStyleParsedMetaError::Meta(error)
        })?;
        let StyleState {
            roots,
            work,
            limits: runtime_limits,
            ..
        } = &mut *state;
        let steps = ParsedSteps {
            work,
            maximum: runtime_limits.work,
            cancel,
        };
        let count = preparation.occurrences().len();
        backing.outputs = count;
        backing.check()?;
        let mut candidates: Vec<Candidate> = Vec::new();
        let mut preferences: Vec<DomStyleParsedMetaPreference> = Vec::new();
        let mut names = Vec::new();
        let mut retired = Vec::new();
        backing.reserve_parsed(&mut candidates, count, &mut preparation, &steps)?;
        backing.reserve_parsed(&mut preferences, count, &mut preparation, &steps)?;
        backing.reserve_parsed(&mut names, count, &mut preparation, &steps)?;
        backing.reserve_parsed(&mut retired, count, &mut preparation, &steps)?;
        for occurrence_index in 0..count {
            steps.joined(&preparation)?;
            let occurrence = &preparation.occurrences()[occurrence_index];
            let document = occurrence.document().node();
            let id = occurrence.id().clone();
            let subject = occurrence.subject().clone();
            let content = occurrence.content().clone();
            let mut candidate_index = None;
            for (index, candidate) in candidates.iter().enumerate() {
                steps.joined(&preparation)?;
                if roots[candidate.root_index].node == document {
                    candidate_index = Some(index);
                    break;
                }
            }
            let index = if let Some(index) = candidate_index {
                index
            } else {
                let mut root_index = None;
                for (index, root) in roots.iter().enumerate() {
                    steps.joined(&preparation)?;
                    if root.node == document {
                        root_index = Some(index);
                        break;
                    }
                }
                let root_index = root_index
                    .ok_or(DomStyleRuntimeError::SheetSetRootUnavailable { root: document })?;
                let current = roots[root_index]
                    .sheet_sets
                    .state()
                    .ok_or(DomStyleRuntimeError::SheetSetDocumentRequired { root: document })?;
                let index = candidates.len();
                candidates.push(Candidate {
                    root_index,
                    root_commit_index: index,
                    next: Some(current.clone()),
                });
                index
            };
            let current = candidates[index].next.as_ref().expect("prepared candidate");
            let (next, enable) =
                current.transition(&content, sheet_sets::Action::Preferred, |left, right| {
                    steps.same(&preparation, left, right)
                })?;
            // Only successor/output names actually retained by Runtime belong
            // to its old/new union. Native recording already owns any alias in
            // the occurrence bank; exclude that backing from the shared native
            // envelope while still checking Runtime's own storage limit.
            for (name_index, name) in next.names().enumerate() {
                steps.joined(&preparation)?;
                let allocation = name.shared_code_units();
                let mut retained = false;
                for root in roots.iter() {
                    steps.joined(&preparation)?;
                    if let Some(current) = root.sheet_sets.state() {
                        for old in current.names() {
                            steps.joined(&preparation)?;
                            retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                        }
                    }
                }
                for previous in &preferences {
                    steps.joined(&preparation)?;
                    for old in std::iter::once(previous.observation.preferred_name())
                        .chain(previous.observation.last_selected_name())
                    {
                        steps.joined(&preparation)?;
                        retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                    }
                }
                for old in next.names().take(name_index) {
                    steps.joined(&preparation)?;
                    retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                }
                if retained {
                    continue;
                }
                let mut shared_dom = false;
                for original in preparation.occurrences() {
                    steps.joined(&preparation)?;
                    shared_dom |= Arc::ptr_eq(allocation, original.content().shared_code_units());
                }
                let units = allocation.capacity();
                let bytes = units
                    .checked_mul(size_of::<u16>())
                    .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                    .ok_or(DomStyleRuntimeError::Admission)?;
                backing.retain_name(bytes, units, shared_dom)?;
                steps.joined(&preparation)?;
                preparation
                    .admit_runtime(backing.parsed_usage()?, cancel)
                    .map_err(DomStyleRuntimeError::from)?;
            }
            steps.joined(&preparation)?;
            let cssom = preparation.root_snapshot(document, cancel)?;
            let producer_revision = cssom.revision().clone();
            let sheet_count = cssom.sheets().len();
            if enable {
                for sheet_index in 0..sheet_count {
                    steps.joined(&preparation)?;
                    steps.joined(&preparation)?;
                    let cssom = preparation.root_snapshot(document, cancel)?;
                    let sheet = cssom.sheets()[sheet_index].clone();
                    let title = &cssom
                        .sheet(&sheet)
                        .map_err(DomStyleRuntimeError::from)?
                        .inputs()
                        .title;
                    let disabled = sheet_sets::named_sheet_disabled(
                        title,
                        content.code_units(),
                        |left, right| steps.same(&preparation, left, right),
                    )?;
                    if let Some(disabled) = disabled {
                        // Always edit the genuine staged batch: a later B -> A
                        // occurrence must undo B even when A equals base flags.
                        steps.joined(&preparation)?;
                        preparation.set_disabled(document, &sheet, disabled, cancel)?;
                    }
                }
            }
            preferences.push(DomStyleParsedMetaPreference {
                id,
                subject,
                observation: DomStyleSheetSetObservation::from_state(
                    stamp,
                    document,
                    &next,
                    producer_revision,
                ),
                root_commit_index: candidates[index].root_commit_index,
            });
            candidates[index].next = Some(next);
        }
        // Split only already-qualified exact destinations. No root lookup,
        // variant match or RefCell borrowing remains after producer publication.
        for (index, root) in roots.iter_mut().enumerate() {
            steps.joined(&preparation)?;
            for candidate in &mut candidates {
                steps.joined(&preparation)?;
                if candidate.root_index == index {
                    names.push(NameRow {
                        destination: root
                            .sheet_sets
                            .state_mut()
                            .expect("qualified Document destination"),
                        next: candidate.next.take(),
                    });
                    break;
                }
            }
        }
        steps.joined(&preparation)?;
        let published = preparation.commit(cancel)?;
        // No fallible service, metering, callback or allocation follows the
        // first producer publication. Names install while all guards are held.
        for row in &mut names {
            row.install(&mut retired);
        }
        let (commit, retirement) = published.into_parts();
        let stamp = commit.stamp();
        for preference in &mut preferences {
            let actual = commit.roots()[preference.root_commit_index].cssom();
            let revision = match actual.publication() {
                surgeist_cssom::CssomPublication::Changed(summary) => summary.to(),
                surgeist_cssom::CssomPublication::Unchanged { revision } => revision,
            };
            preference
                .observation
                .finish_publication(stamp, revision.clone());
        }
        let outcome = DomStyleParsedMetaOutcome {
            completion: commit,
            preferences,
        };
        drop(names);
        drop(candidates);
        drop(retired);
        drop(retirement);
        drop(references);
        drop(capabilities);
        drop(hold);
        Ok(outcome)
    }
}

// Initial discovery reads only destination IDs. Original report/history reads
// begin exclusively under each contribution's durable Hold below.
pub(in crate::dom_style) fn admit(
    dom: &mut Dom,
    limits: DomStyleRuntimeLimits,
    processing: surgeist_dom::DomParsedHtmlAssociationLimits,
    cancel: &AtomicBool,
) -> Result<u64, DomStyleParsedAdmissionError> {
    let work = Cell::new(0);
    let mut ids = Vec::new();
    let fixed = size_of::<std::cell::RefCell<StyleState>>()
        .checked_add(size_of::<Vec<surgeist_dom::DomParsedHtmlAssociationId>>())
        .ok_or(DomStyleRuntimeError::Admission)?;
    let check = |extra: usize| {
        if fixed
            .checked_add(extra)
            .is_none_or(|bytes| bytes > limits.storage_bytes)
        {
            Err(DomStyleRuntimeError::Admission)
        } else {
            Ok(())
        }
    };
    check(0)?;
    StyleState::step_counter(&work, cancel, limits.work)?;
    {
        let mut pending = dom.pending_parsed_html_associations();
        loop {
            StyleState::step_counter(&work, cancel, limits.work)?;
            let Some(entry) = pending.next() else {
                break;
            };
            if ids.len() == ids.capacity() {
                let count = ids
                    .len()
                    .checked_add(1)
                    .ok_or(DomStyleRuntimeError::Admission)?;
                let old = ids
                    .capacity()
                    .checked_mul(size_of::<surgeist_dom::DomParsedHtmlAssociationId>())
                    .ok_or(DomStyleRuntimeError::Admission)?;
                let requested = count
                    .checked_mul(size_of::<surgeist_dom::DomParsedHtmlAssociationId>())
                    .ok_or(DomStyleRuntimeError::Admission)?;
                check(
                    old.checked_add(requested)
                        .ok_or(DomStyleRuntimeError::Admission)?,
                )?;
                StyleState::step_counter(&work, cancel, limits.work)?;
                for _ in 0..ids.len() {
                    StyleState::step_counter(&work, cancel, limits.work)?;
                }
                let mut replacement = Vec::new();
                replacement
                    .try_reserve_exact(count)
                    .map_err(|_| DomStyleRuntimeError::Admission)?;
                let actual = replacement
                    .capacity()
                    .checked_mul(size_of::<surgeist_dom::DomParsedHtmlAssociationId>())
                    .ok_or(DomStyleRuntimeError::Admission)?;
                check(
                    old.checked_add(actual)
                        .ok_or(DomStyleRuntimeError::Admission)?,
                )?;
                replacement.append(&mut ids);
                ids = replacement;
            }
            ids.push(entry.id());
        }
    }
    for id in &ids {
        StyleState::step_counter(&work, cancel, limits.work)?;
        let mut hold = dom.begin_parsed_html_meta(*id, processing, cancel)?;
        hold.validate_meta(cancel)?;
    }
    Ok(work.get())
}

impl DomSession {
    /// Borrow original canonical pending contributions independently of any
    /// installation output. Each iterator step visits the finite sparse bank;
    /// callers choose their read budget. This lends no execution authority.
    pub fn pending_parsed_html_associations(
        &self,
    ) -> impl Iterator<Item = surgeist_dom::DomParsedHtmlPendingAssociations<'_>> {
        self.dom.pending_parsed_html_associations()
    }
    fn prepare_parsed_install(&mut self, cancel: &AtomicBool) -> Result<(), DomStyleRuntimeError> {
        let state = self
            .state
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        state.step(cancel)?;
        state.require_sheet_set_idle(&DomEventDom::borrow(&mut self.dom))?;
        Ok(())
    }
    /// Install immutable structure and original pending evidence into this
    /// Session. This separate operation performs no CSSOM/preference execution.
    pub fn install_parsed_html_document(
        &mut self,
        basis: surgeist_dom::DomStamp,
        report: Arc<surgeist_html_parser::HtmlParseReport>,
        environment: surgeist_dom::DomParsedDocumentEnvironment,
        limits: surgeist_dom::DomParsedHtmlInstallLimits,
        cancel: &AtomicBool,
    ) -> Result<surgeist_dom::DomParsedHtmlInstallation, DomStyleParsedInstallError> {
        self.prepare_parsed_install(cancel)?;
        Ok(self
            .dom
            .install_parsed_html_document(basis, report, environment, limits, cancel)?)
    }
    /// Install an original contextual fragment into its actual current Session
    /// destination, preserving that owner's existing preference/resource history.
    /// The authentic structural receipt remains Pending until separate execution.
    pub fn install_parsed_html_fragment(
        &mut self,
        report: Arc<surgeist_html_parser::HtmlParseReport>,
        capture: &surgeist_dom::DomHtmlCapture,
        limits: surgeist_dom::DomParsedHtmlInstallLimits,
        cancel: &AtomicBool,
    ) -> Result<surgeist_dom::DomParsedHtmlInstallation, DomStyleParsedInstallError> {
        self.prepare_parsed_install(cancel)?;
        Ok(self
            .dom
            .install_parsed_html_fragment(report, capture, limits, cancel)?)
    }
}
